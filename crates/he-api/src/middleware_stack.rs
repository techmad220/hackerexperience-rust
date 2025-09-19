use anyhow::{anyhow, Result};
//! Production middleware stack with auth, rate limiting, and security

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse, web,
    http::StatusCode,
};
use std::future::{Ready, ready};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // user_id
    pub exp: usize,   // expiry
    pub iat: usize,   // issued at
}

/// Authenticated user extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthedUser {
    pub id: String,
    pub token_issued: Instant,
}

/// Auth middleware - validates JWT and extracts user
pub struct AuthMiddleware {
    jwt_secret: String,
    excluded_paths: Vec<String>,
}

impl AuthMiddleware {
    pub fn new(jwt_secret: String) -> Self {
        Self {
            jwt_secret,
            excluded_paths: vec![
                "/health".to_string(),
                "/api/login".to_string(),
                "/api/register".to_string(),
                "/metrics".to_string(),
            ],
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
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
            service,
            jwt_secret: self.jwt_secret.clone(),
            excluded_paths: self.excluded_paths.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
    jwt_secret: String,
    excluded_paths: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path();

        // Skip auth for excluded paths
        if self.excluded_paths.iter().any(|p| path.starts_with(p)) {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await });
        }

        // Extract and validate JWT
        let auth_header = req.headers().get("Authorization");

        let jwt_secret = self.jwt_secret.clone();
        let fut = self.service.call(req);

        Box::pin(async move {
            match auth_header {
                Some(header_value) => {
                    let header_str = header_value.to_str().map_err(|_| {
                        actix_web::error::ErrorUnauthorized("Invalid authorization header")
                    })?;

                    if !header_str.starts_with("Bearer ") {
                        return Err(actix_web::error::ErrorUnauthorized("Invalid token format"));
                    }

                    let token = &header_str[7..];
                    let key = DecodingKey::from_secret(jwt_secret.as_bytes());
                    let validation = Validation::default();

                    match decode::<Claims>(token, &key, &validation) {
                        Ok(token_data) => {
                            // Token is valid, proceed
                            fut.await
                        }
                        Err(_) => {
                            Err(actix_web::error::ErrorUnauthorized("Invalid or expired token"))
                        }
                    }
                }
                None => {
                    Err(actix_web::error::ErrorUnauthorized("Authorization required"))
                }
            }
        })
    }
}

/// Rate limiter with per-IP and per-route limits
pub struct RateLimiter {
    limits: Arc<Mutex<HashMap<String, RateLimitState>>>,
    max_requests: usize,
    window: Duration,
}

#[derive(Clone)]
struct RateLimitState {
    count: usize,
    window_start: Instant,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_seconds: u64) -> Self {
        Self {
            limits: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_seconds),
        }
    }

    pub fn per_route_limits() -> HashMap<String, (usize, u64)> {
        let mut limits = HashMap::new();
        // route -> (max_requests, window_seconds)
        limits.insert("/api/login".to_string(), (5, 60));  // 5 per minute
        limits.insert("/api/register".to_string(), (3, 60));  // 3 per minute
        limits.insert("/api/processes/start".to_string(), (30, 60));  // 30 per minute
        limits.insert("/api/bank/transfer".to_string(), (10, 60));  // 10 per minute
        limits.insert("/api/missions/complete".to_string(), (5, 60));  // 5 per minute
        limits
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimiterService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimiterService {
            service,
            limits: self.limits.clone(),
            max_requests: self.max_requests,
            window: self.window,
        }))
    }
}

pub struct RateLimiterService<S> {
    service: S,
    limits: Arc<Mutex<HashMap<String, RateLimitState>>>,
    max_requests: usize,
    window: Duration,
}

impl<S, B> Service<ServiceRequest> for RateLimiterService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let now = Instant::now();
        let conn_info = req.connection_info().clone();
        let ip = conn_info.realip_remote_addr().unwrap_or("unknown").to_string();
        let path = req.path().to_string();

        // Create unique key for this IP + route
        let key = format!("{}:{}", ip, path);

        let mut limits = self.limits.lock().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let state = limits.entry(key.clone()).or_insert(RateLimitState {
            count: 0,
            window_start: now,
        });

        // Check if we need to reset the window
        if now.duration_since(state.window_start) > self.window {
            state.count = 0;
            state.window_start = now;
        }

        // Check rate limit
        if state.count >= self.max_requests {
            return Box::pin(async move {
                Ok(ServiceResponse::new(
                    req.into_parts().0,
                    HttpResponse::TooManyRequests()
                        .json(serde_json::json!({
                            "error": "Rate limit exceeded",
                            "retry_after_seconds": 60
                        }))
                ))
            });
        }

        // Increment counter
        state.count += 1;
        drop(limits); // Release lock before async operation

        let fut = self.service.call(req);
        Box::pin(async move { fut.await })
    }
}

/// Security headers middleware
pub struct SecurityHeaders;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SecurityHeadersService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityHeadersService { service }))
    }
}

pub struct SecurityHeadersService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            let headers = res.headers_mut();

            // Add security headers
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-frame-options"),
                actix_web::http::header::HeaderValue::from_static("DENY"),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-content-type-options"),
                actix_web::http::header::HeaderValue::from_static("nosniff"),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-xss-protection"),
                actix_web::http::header::HeaderValue::from_static("1; mode=block"),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("referrer-policy"),
                actix_web::http::header::HeaderValue::from_static("strict-origin-when-cross-origin"),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("content-security-policy"),
                actix_web::http::header::HeaderValue::from_static(
                    "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"
                ),
            );

            Ok(res)
        })
    }
}