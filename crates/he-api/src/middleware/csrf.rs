//! CSRF (Cross-Site Request Forgery) Protection Middleware
//!
//! Implements double-submit cookie pattern and synchronizer token pattern
//! for comprehensive CSRF protection.

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse, HttpRequest,
    http::{header::{HeaderName, HeaderValue, COOKIE}, Method, StatusCode},
    cookie::{Cookie, SameSite, time::Duration},
    web,
};
use futures::future::{ok, Ready, LocalBoxFuture, FutureExt};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use rand::Rng;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration as ChronoDuration};

/// CSRF token storage
type TokenStore = Arc<RwLock<HashMap<String, CsrfToken>>>;

/// CSRF token structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrfToken {
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub user_id: Option<String>,
    pub session_id: String,
}

/// CSRF Protection Configuration
#[derive(Debug, Clone)]
pub struct CsrfConfig {
    /// Secret key for token generation
    pub secret_key: String,
    /// Token expiry duration (default: 1 hour)
    pub token_expiry: Duration,
    /// Cookie name for CSRF token
    pub cookie_name: String,
    /// Header name for CSRF token
    pub header_name: String,
    /// Form field name for CSRF token
    pub form_field_name: String,
    /// Excluded paths that don't require CSRF protection
    pub excluded_paths: Vec<String>,
    /// Safe methods that don't require CSRF protection
    pub safe_methods: Vec<Method>,
    /// Whether to use double-submit cookie pattern
    pub double_submit: bool,
    /// Whether to use synchronizer token pattern
    pub synchronizer_token: bool,
    /// SameSite cookie attribute
    pub same_site: SameSite,
    /// Secure cookie flag (HTTPS only)
    pub secure: bool,
}

impl Default for CsrfConfig {
    fn default() -> Self {
        Self {
            secret_key: std::env::var("CSRF_SECRET_KEY")
                .unwrap_or_else(|_| "default_csrf_secret_key_change_in_production".to_string()),
            token_expiry: Duration::hours(1),
            cookie_name: "csrf_token".to_string(),
            header_name: "X-CSRF-Token".to_string(),
            form_field_name: "csrf_token".to_string(),
            excluded_paths: vec![
                "/api/health".to_string(),
                "/api/metrics".to_string(),
                "/api/auth/login".to_string(),
                "/api/auth/register".to_string(),
            ],
            safe_methods: vec![
                Method::GET,
                Method::HEAD,
                Method::OPTIONS,
            ],
            double_submit: true,
            synchronizer_token: true,
            same_site: SameSite::Strict,
            secure: true, // Should be true in production
        }
    }
}

/// CSRF Protection Middleware
pub struct CsrfProtection {
    config: Arc<CsrfConfig>,
    token_store: TokenStore,
}

impl CsrfProtection {
    pub fn new(config: CsrfConfig) -> Self {
        Self {
            config: Arc::new(config),
            token_store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a new CSRF token
    pub fn generate_token(&self, session_id: &str, user_id: Option<String>) -> String {
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 32] = rng.gen();

        let mut hasher = Sha256::new();
        hasher.update(&random_bytes);
        hasher.update(self.config.secret_key.as_bytes());
        hasher.update(session_id.as_bytes());
        hasher.update(Utc::now().timestamp().to_string().as_bytes());

        if let Some(ref uid) = user_id {
            hasher.update(uid.as_bytes());
        }

        let result = hasher.finalize();
        BASE64.encode(result)
    }

    /// Validate CSRF token
    async fn validate_token(&self, request_token: &str, session_id: &str) -> bool {
        let store = self.token_store.read().await;

        if let Some(stored_token) = store.get(session_id) {
            // Check if token matches
            if stored_token.token != request_token {
                return false;
            }

            // Check if token is expired
            let now = Utc::now();
            let expiry = stored_token.created_at + ChronoDuration::seconds(self.config.token_expiry.whole_seconds());

            if now > expiry {
                return false;
            }

            return true;
        }

        false
    }

    /// Store token in memory
    async fn store_token(&self, session_id: String, token: String, user_id: Option<String>) {
        let csrf_token = CsrfToken {
            token: token.clone(),
            created_at: Utc::now(),
            user_id,
            session_id: session_id.clone(),
        };

        let mut store = self.token_store.write().await;
        store.insert(session_id, csrf_token);

        // Clean up expired tokens
        self.cleanup_expired_tokens(&mut store).await;
    }

    /// Clean up expired tokens
    async fn cleanup_expired_tokens(&self, store: &mut HashMap<String, CsrfToken>) {
        let now = Utc::now();
        let expiry_duration = ChronoDuration::seconds(self.config.token_expiry.whole_seconds());

        store.retain(|_, token| {
            now < token.created_at + expiry_duration
        });
    }

    /// Check if path is excluded from CSRF protection
    fn is_excluded_path(&self, path: &str) -> bool {
        self.config.excluded_paths.iter().any(|excluded| {
            path.starts_with(excluded) || path == excluded
        })
    }

    /// Check if method is safe (doesn't require CSRF protection)
    fn is_safe_method(&self, method: &Method) -> bool {
        self.config.safe_methods.contains(method)
    }

    /// Extract token from request
    fn extract_token_from_request(&self, req: &ServiceRequest) -> Option<String> {
        // Try to get from header first
        if let Some(header_value) = req.headers().get(&self.config.header_name) {
            if let Ok(token) = header_value.to_str() {
                return Some(token.to_string());
            }
        }

        // Try to get from cookie (double-submit pattern)
        if self.config.double_submit {
            if let Some(cookie_header) = req.headers().get(COOKIE) {
                if let Ok(cookie_str) = cookie_header.to_str() {
                    for cookie_pair in cookie_str.split(';') {
                        let parts: Vec<&str> = cookie_pair.trim().splitn(2, '=').collect();
                        if parts.len() == 2 && parts[0] == self.config.cookie_name {
                            return Some(parts[1].to_string());
                        }
                    }
                }
            }
        }

        // Try to get from form data (for traditional form submissions)
        // This would require parsing the body, which is more complex in middleware
        // For now, we'll rely on header and cookie methods

        None
    }

    /// Get or create session ID
    fn get_session_id(&self, req: &ServiceRequest) -> String {
        // In production, this should be tied to your actual session management
        // For now, we'll use a combination of IP and user agent as a simple session identifier
        let ip = req.connection_info().realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();

        let user_agent = req.headers()
            .get("User-Agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        let mut hasher = Sha256::new();
        hasher.update(ip.as_bytes());
        hasher.update(user_agent.as_bytes());

        BASE64.encode(hasher.finalize())
    }
}

impl<S, B> Transform<S, ServiceRequest> for CsrfProtection
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = CsrfProtectionMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CsrfProtectionMiddleware {
            service: Rc::new(service),
            config: self.config.clone(),
            token_store: self.token_store.clone(),
        })
    }
}

pub struct CsrfProtectionMiddleware<S> {
    service: Rc<S>,
    config: Arc<CsrfConfig>,
    token_store: TokenStore,
}

impl<S> Clone for CsrfProtectionMiddleware<S> {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            config: self.config.clone(),
            token_store: self.token_store.clone(),
        }
    }
}

impl<S, B> Service<ServiceRequest> for CsrfProtectionMiddleware<S>
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
        let config = self.config.clone();
        let token_store = self.token_store.clone();

        Box::pin(async move {
            let path = req.path();
            let method = req.method();

            // Get session ID
            let session_id = {
                let ip = req.connection_info().realip_remote_addr()
                    .unwrap_or("unknown")
                    .to_string();

                let user_agent = req.headers()
                    .get("User-Agent")
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or("unknown")
                    .to_string();

                let mut hasher = Sha256::new();
                hasher.update(ip.as_bytes());
                hasher.update(user_agent.as_bytes());

                BASE64.encode(hasher.finalize())
            };

            // Check if path is excluded
            let is_excluded = config.excluded_paths.iter().any(|excluded| {
                path.starts_with(excluded) || path == excluded
            });

            // Check if method is safe
            let is_safe = config.safe_methods.contains(method);

            // If this is a safe method or excluded path, proceed without CSRF check
            if is_safe || is_excluded {
                // For GET requests, generate and set a new CSRF token
                if *method == Method::GET && !is_excluded {
                    let protection = CsrfProtection {
                        config: config.clone(),
                        token_store: token_store.clone(),
                    };

                    let new_token = protection.generate_token(&session_id, None);
                    protection.store_token(session_id.clone(), new_token.clone(), None).await;

                    let mut res = service.call(req).await?;

                    // Add CSRF token to response as a cookie
                    let cookie = Cookie::build(&config.cookie_name, new_token.clone())
                        .path("/")
                        .same_site(config.same_site)
                        .secure(config.secure)
                        .http_only(false) // JavaScript needs to read this for AJAX requests
                        .max_age(Duration::hours(1))
                        .finish();

                    res.headers_mut().append(
                        HeaderName::from_static("set-cookie"),
                        HeaderValue::from_str(&cookie.to_string()).unwrap(),
                    );

                    // Also add token as a response header for SPA applications
                    res.headers_mut().insert(
                        HeaderName::from_bytes(config.header_name.as_bytes()).unwrap(),
                        HeaderValue::from_str(&new_token).unwrap(),
                    );

                    return Ok(res);
                }

                return service.call(req).await;
            }

            // For state-changing requests, validate CSRF token
            let request_token = {
                // Try to get from header first
                if let Some(header_value) = req.headers().get(&config.header_name) {
                    header_value.to_str().ok().map(|s| s.to_string())
                } else if config.double_submit {
                    // Try to get from cookie
                    req.headers()
                        .get(COOKIE)
                        .and_then(|h| h.to_str().ok())
                        .and_then(|cookie_str| {
                            cookie_str.split(';')
                                .find_map(|cookie_pair| {
                                    let parts: Vec<&str> = cookie_pair.trim().splitn(2, '=').collect();
                                    if parts.len() == 2 && parts[0] == config.cookie_name {
                                        Some(parts[1].to_string())
                                    } else {
                                        None
                                    }
                                })
                        })
                } else {
                    None
                }
            };

            if let Some(token) = request_token {
                let protection = CsrfProtection {
                    config: config.clone(),
                    token_store: token_store.clone(),
                };

                if protection.validate_token(&token, &session_id).await {
                    // Token is valid, proceed with request
                    return service.call(req).await;
                }
            }

            // CSRF token is missing or invalid
            Err(actix_web::error::ErrorForbidden(
                "CSRF token validation failed. Please refresh the page and try again."
            ))
        })
    }
}

/// Helper function to inject CSRF token into forms
pub fn csrf_token_input(token: &str) -> String {
    format!(
        r#"<input type="hidden" name="csrf_token" value="{}" />"#,
        token
    )
}

/// Extension trait for HttpRequest to get CSRF token
pub trait CsrfTokenExt {
    fn csrf_token(&self) -> Option<String>;
}

impl CsrfTokenExt for HttpRequest {
    fn csrf_token(&self) -> Option<String> {
        self.extensions()
            .get::<String>()
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation() {
        let config = CsrfConfig::default();
        let protection = CsrfProtection::new(config);

        let token1 = protection.generate_token("session1", None);
        let token2 = protection.generate_token("session1", None);

        // Tokens should be different even for same session
        assert_ne!(token1, token2);

        // Tokens should be valid base64
        assert!(BASE64.decode(&token1).is_ok());
        assert!(BASE64.decode(&token2).is_ok());
    }

    #[test]
    fn test_excluded_paths() {
        let config = CsrfConfig::default();
        let protection = CsrfProtection::new(config);

        assert!(protection.is_excluded_path("/api/health"));
        assert!(protection.is_excluded_path("/api/auth/login"));
        assert!(!protection.is_excluded_path("/api/users"));
    }

    #[test]
    fn test_safe_methods() {
        let config = CsrfConfig::default();
        let protection = CsrfProtection::new(config);

        assert!(protection.is_safe_method(&Method::GET));
        assert!(protection.is_safe_method(&Method::HEAD));
        assert!(!protection.is_safe_method(&Method::POST));
        assert!(!protection.is_safe_method(&Method::DELETE));
    }
}