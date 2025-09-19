use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpMessage, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use std::rc::Rc;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,  // user_id
    pub exp: i64,  // expiration
    pub iat: i64,  // issued at
    pub email: String,
    pub username: String,
}

/// Authentication middleware factory
pub struct AuthMiddleware {
    pub required: bool,
}

impl AuthMiddleware {
    pub fn required() -> Self {
        Self { required: true }
    }

    pub fn optional() -> Self {
        Self { required: false }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
            required: self.required,
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    required: bool,
}

impl<S> Clone for AuthMiddlewareService<S> {
    fn clone(&self) -> Self {
        Self {
            service: Rc::clone(&self.service),
            required: self.required,
        }
    }
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let required = self.required;

        Box::pin(async move {
            // Extract token from Authorization header
            let token = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "));

            if let Some(token) = token {
                // Get JWT secret from app data
                let jwt_secret = std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "your-secret-key".to_string());

                // Validate token
                let validation = Validation::new(Algorithm::HS256);
                let key = DecodingKey::from_secret(jwt_secret.as_bytes());

                match decode::<Claims>(token, &key, &validation) {
                    Ok(token_data) => {
                        // Store user_id in request extensions for later use
                        req.extensions_mut().insert(token_data.claims);
                        service.call(req).await
                    }
                    Err(_) if !required => {
                        // Token invalid but auth is optional
                        service.call(req).await
                    }
                    Err(_) => {
                        // Token invalid and auth is required
                        let response = HttpResponse::Unauthorized()
                            .json(serde_json::json!({
                                "success": false,
                                "message": "Invalid or expired token"
                            }))
                            .map_into_boxed_body()
                            .map_into_left_body();

                        Ok(ServiceResponse::new(req.into_parts().0, response))
                    }
                }
            } else if required {
                // No token provided and auth is required
                let response = HttpResponse::Unauthorized()
                    .json(serde_json::json!({
                        "success": false,
                        "message": "Authorization token required"
                    }))
                    .map_into_boxed_body()
                    .map_into_left_body();

                Ok(ServiceResponse::new(req.into_parts().0, response))
            } else {
                // No token but auth is optional
                service.call(req).await
            }
        })
    }
}

/// Helper function to extract user_id from request (use in handlers)
pub fn extract_user_id(req: &actix_web::HttpRequest) -> Option<i64> {
    req.extensions()
        .get::<Claims>()
        .map(|claims| claims.sub)
}

/// Helper function to extract full claims from request
pub fn extract_claims(req: &actix_web::HttpRequest) -> Option<Claims> {
    req.extensions().get::<Claims>().cloned()
}

/// Macro to simplify auth checking in handlers
#[macro_export]
macro_rules! require_auth {
    ($req:expr) => {
        match $crate::middleware::auth::extract_user_id($req) {
            Some(user_id) => user_id,
            None => {
                return HttpResponse::Unauthorized().json(serde_json::json!({
                    "success": false,
                    "message": "Unauthorized"
                }));
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn test_auth_middleware_required() {
        let app = test::init_service(
            App::new()
                .wrap(AuthMiddleware::required())
                .route("/test", web::get().to(|| async { HttpResponse::Ok() }))
        ).await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
    }

    #[actix_web::test]
    async fn test_auth_middleware_optional() {
        let app = test::init_service(
            App::new()
                .wrap(AuthMiddleware::optional())
                .route("/test", web::get().to(|| async { HttpResponse::Ok() }))
        ).await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }
}