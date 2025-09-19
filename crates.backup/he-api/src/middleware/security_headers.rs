//! Security Headers Middleware for Axum
//!
//! Adds important security headers to all HTTP responses including:
//! - Strict-Transport-Security (HSTS)
//! - Content-Security-Policy (CSP)
//! - X-Frame-Options
//! - X-Content-Type-Options
//! - X-XSS-Protection
//! - Referrer-Policy
//! - Permissions-Policy

use axum::{
    body::Body,
    http::{header::{HeaderName, HeaderValue}, Request, Response, StatusCode},
    middleware::Next,
};
use std::str::FromStr;

/// Security headers configuration
#[derive(Debug, Clone)]
pub struct SecurityHeadersConfig {
    /// Enable HSTS (HTTP Strict Transport Security)
    pub hsts_enabled: bool,

    /// HSTS max age in seconds (default: 1 year)
    pub hsts_max_age: u64,

    /// Include subdomains in HSTS
    pub hsts_include_subdomains: bool,

    /// Add preload flag to HSTS (be careful with this!)
    pub hsts_preload: bool,

    /// Content Security Policy directives
    pub csp_directives: String,

    /// X-Frame-Options value (DENY, SAMEORIGIN, or ALLOW-FROM uri)
    pub x_frame_options: String,

    /// Enable X-Content-Type-Options: nosniff
    pub x_content_type_options: bool,

    /// Enable X-XSS-Protection
    pub x_xss_protection: bool,

    /// Referrer Policy
    pub referrer_policy: String,

    /// Permissions Policy (formerly Feature Policy)
    pub permissions_policy: String,
}

impl Default for SecurityHeadersConfig {
    fn default() -> Self {
        Self {
            hsts_enabled: true,
            hsts_max_age: 31536000, // 1 year
            hsts_include_subdomains: true,
            hsts_preload: false,
            csp_directives: Self::default_csp(),
            x_frame_options: "DENY".to_string(),
            x_content_type_options: true,
            x_xss_protection: true,
            referrer_policy: "strict-origin-when-cross-origin".to_string(),
            permissions_policy: Self::default_permissions_policy(),
        }
    }
}

impl SecurityHeadersConfig {
    /// Default Content Security Policy (Production-Ready without unsafe-inline)
    fn default_csp() -> String {
        vec![
            "default-src 'self'",
            "script-src 'self'", // No unsafe-inline or unsafe-eval
            "style-src 'self'",  // No unsafe-inline
            "img-src 'self' data: https:",
            "font-src 'self' data:",
            "connect-src 'self' wss: https:",
            "media-src 'self'",
            "object-src 'none'",
            "child-src 'self'",
            "frame-ancestors 'none'",
            "form-action 'self'",
            "base-uri 'self'",
            "upgrade-insecure-requests",
            "block-all-mixed-content",
        ].join("; ")
    }

    /// Default Permissions Policy
    fn default_permissions_policy() -> String {
        vec![
            "accelerometer=()",
            "camera=()",
            "geolocation=()",
            "gyroscope=()",
            "magnetometer=()",
            "microphone=()",
            "payment=()",
            "usb=()",
        ].join(", ")
    }

    /// Create a strict configuration for production
    pub fn strict() -> Self {
        Self {
            hsts_enabled: true,
            hsts_max_age: 63072000, // 2 years
            hsts_include_subdomains: true,
            hsts_preload: true,
            csp_directives: Self::strict_csp(),
            x_frame_options: "DENY".to_string(),
            x_content_type_options: true,
            x_xss_protection: true,
            referrer_policy: "no-referrer".to_string(),
            permissions_policy: Self::strict_permissions_policy(),
        }
    }

    /// Strict Content Security Policy (no unsafe-inline)
    fn strict_csp() -> String {
        vec![
            "default-src 'none'",
            "script-src 'self'",
            "style-src 'self'",
            "img-src 'self' data:",
            "font-src 'self'",
            "connect-src 'self' wss: https:",
            "media-src 'none'",
            "object-src 'none'",
            "child-src 'none'",
            "frame-ancestors 'none'",
            "form-action 'self'",
            "base-uri 'self'",
            "upgrade-insecure-requests",
            "block-all-mixed-content",
        ].join("; ")
    }

    /// Strict Permissions Policy (deny all)
    fn strict_permissions_policy() -> String {
        vec![
            "accelerometer=()",
            "ambient-light-sensor=()",
            "autoplay=()",
            "battery=()",
            "camera=()",
            "cross-origin-isolated=()",
            "display-capture=()",
            "document-domain=()",
            "encrypted-media=()",
            "execution-while-not-rendered=()",
            "execution-while-out-of-viewport=()",
            "fullscreen=()",
            "geolocation=()",
            "gyroscope=()",
            "magnetometer=()",
            "microphone=()",
            "midi=()",
            "navigation-override=()",
            "payment=()",
            "picture-in-picture=()",
            "publickey-credentials-get=()",
            "screen-wake-lock=()",
            "sync-xhr=()",
            "usb=()",
            "web-share=()",
            "xr-spatial-tracking=()",
        ].join(", ")
    }

    /// Build HSTS header value
    fn build_hsts_header(&self) -> String {
        let mut hsts = format!("max-age={}", self.hsts_max_age);

        if self.hsts_include_subdomains {
            hsts.push_str("; includeSubDomains");
        }

        if self.hsts_preload {
            hsts.push_str("; preload");
        }

        hsts
    }
}

/// Axum middleware function for adding security headers
pub async fn security_headers_middleware(
    config: SecurityHeadersConfig,
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    // Add HSTS header if enabled
    if config.hsts_enabled {
        headers.insert(
            HeaderName::from_static("strict-transport-security"),
            HeaderValue::from_str(&config.build_hsts_header())
                .unwrap_or_else(|_| HeaderValue::from_static("max-age=31536000")),
        );
    }

    // Add Content Security Policy
    if !config.csp_directives.is_empty() {
        headers.insert(
            HeaderName::from_static("content-security-policy"),
            HeaderValue::from_str(&config.csp_directives)
                .unwrap_or_else(|_| HeaderValue::from_static("default-src 'self'")),
        );
    }

    // Add X-Frame-Options
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_str(&config.x_frame_options)
            .unwrap_or_else(|_| HeaderValue::from_static("DENY")),
    );

    // Add X-Content-Type-Options
    if config.x_content_type_options {
        headers.insert(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        );
    }

    // Add X-XSS-Protection
    if config.x_xss_protection {
        headers.insert(
            HeaderName::from_static("x-xss-protection"),
            HeaderValue::from_static("1; mode=block"),
        );
    }

    // Add Referrer-Policy
    if !config.referrer_policy.is_empty() {
        headers.insert(
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_str(&config.referrer_policy)
                .unwrap_or_else(|_| HeaderValue::from_static("strict-origin-when-cross-origin")),
        );
    }

    // Add Permissions-Policy
    if !config.permissions_policy.is_empty() {
        headers.insert(
            HeaderName::from_static("permissions-policy"),
            HeaderValue::from_str(&config.permissions_policy)
                .unwrap_or_else(|_| HeaderValue::from_static("geolocation=(), camera=(), microphone=()")),
        );
    }

    Ok(response)
}

/// Create middleware layer with default configuration
pub fn security_headers_layer() -> impl Fn(Request<Body>, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response<Body>, StatusCode>> + Send>> + Clone {
    let config = SecurityHeadersConfig::default();
    move |req, next| {
        let config = config.clone();
        Box::pin(security_headers_middleware(config, req, next))
    }
}

/// Create middleware layer with strict configuration
pub fn strict_security_headers_layer() -> impl Fn(Request<Body>, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response<Body>, StatusCode>> + Send>> + Clone {
    let config = SecurityHeadersConfig::strict();
    move |req, next| {
        let config = config.clone();
        Box::pin(security_headers_middleware(config, req, next))
    }
}

/// Create middleware layer with custom configuration
pub fn custom_security_headers_layer(config: SecurityHeadersConfig) -> impl Fn(Request<Body>, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response<Body>, StatusCode>> + Send>> + Clone {
    move |req, next| {
        let config = config.clone();
        Box::pin(security_headers_middleware(config, req, next))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SecurityHeadersConfig::default();
        assert!(config.hsts_enabled);
        assert_eq!(config.hsts_max_age, 31536000);
        assert!(config.hsts_include_subdomains);
        assert!(!config.hsts_preload);
    }

    #[test]
    fn test_strict_config() {
        let config = SecurityHeadersConfig::strict();
        assert!(config.hsts_enabled);
        assert_eq!(config.hsts_max_age, 63072000);
        assert!(config.hsts_preload);
        assert!(config.csp_directives.contains("default-src 'none'"));
    }

    #[test]
    fn test_hsts_header_building() {
        let mut config = SecurityHeadersConfig::default();
        config.hsts_max_age = 86400;
        config.hsts_include_subdomains = false;
        config.hsts_preload = false;

        assert_eq!(config.build_hsts_header(), "max-age=86400");

        config.hsts_include_subdomains = true;
        assert_eq!(config.build_hsts_header(), "max-age=86400; includeSubDomains");

        config.hsts_preload = true;
        assert_eq!(config.build_hsts_header(), "max-age=86400; includeSubDomains; preload");
    }

    #[test]
    fn test_csp_no_unsafe_inline() {
        let default_csp = SecurityHeadersConfig::default_csp();
        assert!(!default_csp.contains("unsafe-inline"));
        assert!(!default_csp.contains("unsafe-eval"));

        let strict_csp = SecurityHeadersConfig::strict_csp();
        assert!(!strict_csp.contains("unsafe-inline"));
        assert!(!strict_csp.contains("unsafe-eval"));
    }
}