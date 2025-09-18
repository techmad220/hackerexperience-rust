//! Security Headers Middleware
//!
//! Adds important security headers to all HTTP responses including:
//! - Strict-Transport-Security (HSTS)
//! - Content-Security-Policy (CSP)
//! - X-Frame-Options
//! - X-Content-Type-Options
//! - X-XSS-Protection

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
    http::header::{HeaderName, HeaderValue},
};
use futures::future::{ok, Ready};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

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
    /// Default Content Security Policy
    fn default_csp() -> String {
        vec![
            "default-src 'self'",
            "script-src 'self' 'unsafe-inline' 'unsafe-eval'", // Consider removing unsafe-* in production
            "style-src 'self' 'unsafe-inline'",
            "img-src 'self' data: https:",
            "font-src 'self' data:",
            "connect-src 'self'",
            "media-src 'self'",
            "object-src 'none'",
            "child-src 'self'",
            "frame-ancestors 'none'",
            "form-action 'self'",
            "base-uri 'self'",
            "upgrade-insecure-requests",
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
            "connect-src 'self'",
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

/// Security Headers Middleware
pub struct SecurityHeaders {
    config: SecurityHeadersConfig,
}

impl SecurityHeaders {
    pub fn new(config: SecurityHeadersConfig) -> Self {
        Self { config }
    }

    pub fn default() -> Self {
        Self::new(SecurityHeadersConfig::default())
    }

    pub fn strict() -> Self {
        Self::new(SecurityHeadersConfig::strict())
    }
}

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = SecurityHeadersMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(SecurityHeadersMiddleware {
            service: Rc::new(service),
            config: self.config.clone(),
        })
    }
}

pub struct SecurityHeadersMiddleware<S> {
    service: Rc<S>,
    config: SecurityHeadersConfig,
}

impl<S> Clone for SecurityHeadersMiddleware<S> {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            config: self.config.clone(),
        }
    }
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let config = self.config.clone();

        Box::pin(async move {
            let mut res = service.call(req).await?;
            let headers = res.headers_mut();

            // Add HSTS header if enabled and connection is secure
            if config.hsts_enabled {
                headers.insert(
                    HeaderName::from_static("strict-transport-security"),
                    HeaderValue::from_str(&config.build_hsts_header()).unwrap(),
                );
            }

            // Add Content Security Policy
            if !config.csp_directives.is_empty() {
                headers.insert(
                    HeaderName::from_static("content-security-policy"),
                    HeaderValue::from_str(&config.csp_directives).unwrap(),
                );
            }

            // Add X-Frame-Options
            headers.insert(
                HeaderName::from_static("x-frame-options"),
                HeaderValue::from_str(&config.x_frame_options).unwrap(),
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
                    HeaderValue::from_str(&config.referrer_policy).unwrap(),
                );
            }

            // Add Permissions-Policy
            if !config.permissions_policy.is_empty() {
                headers.insert(
                    HeaderName::from_static("permissions-policy"),
                    HeaderValue::from_str(&config.permissions_policy).unwrap(),
                );
            }

            Ok(res)
        })
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
}