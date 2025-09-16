//! Authentication middleware for Actix-Web

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::{ErrorForbidden, ErrorUnauthorized},
    http::{header, StatusCode},
    Error, FromRequest, HttpMessage, HttpRequest,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use serde::{Deserialize, Serialize};
use std::future::{ready, Future};
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{AuthService, ValidatedUser, JwtManager, RoleManager, RateLimiter};

/// Authentication middleware factory
pub struct AuthMiddleware {
    auth_service: Arc<AuthService>,
    require_auth: bool,
    require_verified_email: bool,
    allowed_roles: Vec<String>,
    rate_limit_enabled: bool,
}

impl AuthMiddleware {
    /// Create new authentication middleware
    pub fn new(auth_service: Arc<AuthService>) -> Self {
        Self {
            auth_service,
            require_auth: true,
            require_verified_email: false,
            allowed_roles: Vec::new(),
            rate_limit_enabled: true,
        }
    }

    /// Set whether authentication is required
    pub fn require_auth(mut self, require: bool) -> Self {
        self.require_auth = require;
        self
    }

    /// Set whether verified email is required
    pub fn require_verified_email(mut self, require: bool) -> Self {
        self.require_verified_email = require;
        self
    }

    /// Set allowed roles
    pub fn allowed_roles(mut self, roles: Vec<String>) -> Self {
        self.allowed_roles = roles;
        self
    }

    /// Enable/disable rate limiting
    pub fn rate_limit(mut self, enabled: bool) -> Self {
        self.rate_limit_enabled = enabled;
        self
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
        ok(AuthMiddlewareService {
            service: Rc::new(service),
            auth_service: self.auth_service.clone(),
            require_auth: self.require_auth,
            require_verified_email: self.require_verified_email,
            allowed_roles: self.allowed_roles.clone(),
            rate_limit_enabled: self.rate_limit_enabled,
        })
    }
}

/// Authentication middleware service
pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    auth_service: Arc<AuthService>,
    require_auth: bool,
    require_verified_email: bool,
    allowed_roles: Vec<String>,
    rate_limit_enabled: bool,
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
        let service = self.service.clone();
        let auth_service = self.auth_service.clone();
        let require_auth = self.require_auth;
        let require_verified_email = self.require_verified_email;
        let allowed_roles = self.allowed_roles.clone();
        let rate_limit_enabled = self.rate_limit_enabled;

        Box::pin(async move {
            // Extract client IP for rate limiting
            let client_ip = extract_client_ip(&req);

            // Check rate limiting
            if rate_limit_enabled {
                let path = req.path();
                let method = req.method().as_str();

                if let Some(ip) = &client_ip {
                    let rate_limit = auth_service
                        .rate_limiter
                        .check_endpoint_rate_limit(ip, path, method)
                        .await;

                    if !rate_limit.allowed {
                        req.headers_mut().insert(
                            header::HeaderName::from_static("x-ratelimit-limit"),
                            header::HeaderValue::from_str(&rate_limit.limit.to_string()).unwrap(),
                        );
                        req.headers_mut().insert(
                            header::HeaderName::from_static("x-ratelimit-remaining"),
                            header::HeaderValue::from_str(&rate_limit.remaining.to_string()).unwrap(),
                        );
                        req.headers_mut().insert(
                            header::HeaderName::from_static("x-ratelimit-reset"),
                            header::HeaderValue::from_str(&rate_limit.reset_at.to_string()).unwrap(),
                        );

                        if let Some(retry_after) = rate_limit.retry_after {
                            req.headers_mut().insert(
                                header::RETRY_AFTER,
                                header::HeaderValue::from_str(&retry_after.to_string()).unwrap(),
                            );
                        }

                        return Err(ErrorForbidden("Rate limit exceeded"));
                    }
                }
            }

            // Extract authorization token
            let token = extract_bearer_token(&req);

            if require_auth {
                // Validate token
                let token = token.ok_or_else(|| ErrorUnauthorized("Missing authorization token"))?;

                let user = auth_service
                    .validate_token(&token)
                    .await
                    .map_err(|e| {
                        error!("Token validation failed: {}", e);
                        ErrorUnauthorized("Invalid token")
                    })?
                    .ok_or_else(|| ErrorUnauthorized("Invalid or expired token"))?;

                // Check email verification if required
                if require_verified_email && !user.email.contains("@verified") {
                    return Err(ErrorForbidden("Email verification required"));
                }

                // Check roles if specified
                if !allowed_roles.is_empty() {
                    let has_required_role = allowed_roles
                        .iter()
                        .any(|required| user.roles.contains(required));

                    if !has_required_role {
                        return Err(ErrorForbidden("Insufficient permissions"));
                    }
                }

                // Store authenticated user in request extensions
                req.extensions_mut().insert(AuthenticatedUser {
                    user_id: user.user_id,
                    email: user.email,
                    roles: user.roles,
                    session_id: user.session_id,
                });

                debug!("Authenticated user: {}", user.user_id);
            } else if let Some(token) = token {
                // Optional authentication - validate if token provided
                if let Ok(Some(user)) = auth_service.validate_token(&token).await {
                    req.extensions_mut().insert(AuthenticatedUser {
                        user_id: user.user_id,
                        email: user.email,
                        roles: user.roles,
                        session_id: user.session_id,
                    });
                }
            }

            // Call the actual service
            service.call(req).await
        })
    }
}

/// Authenticated user information stored in request extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: String,
    pub roles: Vec<String>,
    pub session_id: Option<String>,
}

impl AuthenticatedUser {
    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    /// Check if user has any of the specified roles
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|role| self.has_role(role))
    }

    /// Check if user has all of the specified roles
    pub fn has_all_roles(&self, roles: &[&str]) -> bool {
        roles.iter().all(|role| self.has_role(role))
    }

    /// Check if user is admin
    pub fn is_admin(&self) -> bool {
        self.has_role("admin")
    }

    /// Check if user is moderator
    pub fn is_moderator(&self) -> bool {
        self.has_role("moderator") || self.is_admin()
    }
}

/// Extract authenticated user from request
impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        if let Some(user) = req.extensions().get::<AuthenticatedUser>() {
            ready(Ok(user.clone()))
        } else {
            ready(Err(ErrorUnauthorized("Not authenticated")))
        }
    }
}

/// Optional authenticated user extractor
pub struct OptionalAuth(pub Option<AuthenticatedUser>);

impl FromRequest for OptionalAuth {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let user = req.extensions().get::<AuthenticatedUser>().cloned();
        ready(Ok(OptionalAuth(user)))
    }
}

/// Require specific role middleware
pub struct RequireRole {
    role: String,
}

impl RequireRole {
    pub fn new(role: &str) -> Self {
        Self {
            role: role.to_string(),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequireRole
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequireRoleService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RequireRoleService {
            service: Rc::new(service),
            role: self.role.clone(),
        })
    }
}

pub struct RequireRoleService<S> {
    service: Rc<S>,
    role: String,
}

impl<S, B> Service<ServiceRequest> for RequireRoleService<S>
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
        let service = self.service.clone();
        let required_role = self.role.clone();

        Box::pin(async move {
            // Check if user has required role
            if let Some(user) = req.extensions().get::<AuthenticatedUser>() {
                if !user.has_role(&required_role) {
                    return Err(ErrorForbidden(format!(
                        "Role '{}' required",
                        required_role
                    )));
                }
            } else {
                return Err(ErrorUnauthorized("Authentication required"));
            }

            service.call(req).await
        })
    }
}

/// Require authentication middleware (simple version)
pub struct RequireAuth;

impl<S, B> Transform<S, ServiceRequest> for RequireAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequireAuthService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RequireAuthService {
            service: Rc::new(service),
        })
    }
}

pub struct RequireAuthService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequireAuthService<S>
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
        let service = self.service.clone();

        Box::pin(async move {
            // Check if user is authenticated
            if req.extensions().get::<AuthenticatedUser>().is_none() {
                return Err(ErrorUnauthorized("Authentication required"));
            }

            service.call(req).await
        })
    }
}

/// Extract bearer token from request
fn extract_bearer_token(req: &ServiceRequest) -> Option<String> {
    // Try Authorization header first
    if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].to_string());
            }
        }
    }

    // Try cookie
    if let Some(cookie_header) = req.headers().get(header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
                if parts.len() == 2 && parts[0] == "auth_token" {
                    return Some(parts[1].to_string());
                }
            }
        }
    }

    // Try query parameter (for WebSocket connections)
    if let Some(query) = req.uri().query() {
        for param in query.split('&') {
            let parts: Vec<&str> = param.splitn(2, '=').collect();
            if parts.len() == 2 && parts[0] == "token" {
                return Some(parts[1].to_string());
            }
        }
    }

    None
}

/// Extract client IP address
fn extract_client_ip(req: &ServiceRequest) -> Option<String> {
    // Try X-Forwarded-For header (for proxies)
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(ip) = forwarded_str.split(',').next() {
                return Some(ip.trim().to_string());
            }
        }
    }

    // Try X-Real-IP header
    if let Some(real_ip) = req.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return Some(ip_str.to_string());
        }
    }

    // Fall back to peer address
    req.peer_addr().map(|addr| addr.ip().to_string())
}

/// CORS configuration helper
pub fn configure_cors() -> actix_cors::Cors {
    use actix_cors::Cors;

    Cors::default()
        .allowed_origin_fn(|origin, _req_head| {
            // Allow localhost for development
            origin.as_bytes().starts_with(b"http://localhost") ||
            origin.as_bytes().starts_with(b"https://localhost") ||
            origin.as_bytes().starts_with(b"http://127.0.0.1") ||
            origin.as_bytes().starts_with(b"https://127.0.0.1")
        })
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allowed_headers(vec![
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
        ])
        .expose_headers(vec![
            header::HeaderName::from_static("x-ratelimit-limit"),
            header::HeaderName::from_static("x-ratelimit-remaining"),
            header::HeaderName::from_static("x-ratelimit-reset"),
        ])
        .supports_credentials()
        .max_age(3600)
}

/// Security headers middleware
pub struct SecurityHeaders;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SecurityHeadersService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(SecurityHeadersService {
            service: Rc::new(service),
        })
    }
}

pub struct SecurityHeadersService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersService<S>
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
        let service = self.service.clone();

        Box::pin(async move {
            let mut res = service.call(req).await?;

            // Add security headers
            let headers = res.headers_mut();

            headers.insert(
                header::HeaderName::from_static("x-content-type-options"),
                header::HeaderValue::from_static("nosniff"),
            );

            headers.insert(
                header::HeaderName::from_static("x-frame-options"),
                header::HeaderValue::from_static("DENY"),
            );

            headers.insert(
                header::HeaderName::from_static("x-xss-protection"),
                header::HeaderValue::from_static("1; mode=block"),
            );

            headers.insert(
                header::HeaderName::from_static("referrer-policy"),
                header::HeaderValue::from_static("strict-origin-when-cross-origin"),
            );

            headers.insert(
                header::HeaderName::from_static("content-security-policy"),
                header::HeaderValue::from_static(
                    "default-src 'self'; \
                     script-src 'self' 'unsafe-inline' 'unsafe-eval'; \
                     style-src 'self' 'unsafe-inline'; \
                     img-src 'self' data: https:; \
                     font-src 'self' data:; \
                     connect-src 'self' wss: https:; \
                     frame-ancestors 'none';"
                ),
            );

            headers.insert(
                header::HeaderName::from_static("strict-transport-security"),
                header::HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
            );

            headers.insert(
                header::HeaderName::from_static("permissions-policy"),
                header::HeaderValue::from_static(
                    "accelerometer=(), camera=(), geolocation=(), \
                     gyroscope=(), magnetometer=(), microphone=(), \
                     payment=(), usb=()"
                ),
            );

            Ok(res)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[test]
    fn test_extract_bearer_token() {
        let req = test::TestRequest::default()
            .insert_header((header::AUTHORIZATION, "Bearer test_token_123"))
            .to_srv_request();

        let token = extract_bearer_token(&req);
        assert_eq!(token, Some("test_token_123".to_string()));
    }

    #[test]
    fn test_extract_bearer_token_from_cookie() {
        let req = test::TestRequest::default()
            .insert_header((header::COOKIE, "auth_token=cookie_token_456; other=value"))
            .to_srv_request();

        let token = extract_bearer_token(&req);
        assert_eq!(token, Some("cookie_token_456".to_string()));
    }

    #[test]
    fn test_extract_client_ip() {
        let req = test::TestRequest::default()
            .insert_header(("x-forwarded-for", "192.168.1.1, 10.0.0.1"))
            .to_srv_request();

        let ip = extract_client_ip(&req);
        assert_eq!(ip, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_authenticated_user_roles() {
        let user = AuthenticatedUser {
            user_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            roles: vec!["player".to_string(), "premium".to_string()],
            session_id: None,
        };

        assert!(user.has_role("player"));
        assert!(user.has_role("premium"));
        assert!(!user.has_role("admin"));

        assert!(user.has_any_role(&["admin", "player"]));
        assert!(!user.has_all_roles(&["admin", "player"]));
        assert!(user.has_all_roles(&["player", "premium"]));
    }
}