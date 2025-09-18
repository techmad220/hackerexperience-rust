//! API versioning middleware for backward compatibility
//!
//! Supports URL-based and header-based versioning strategies

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
    http::header::{HeaderName, HeaderValue},
};
use futures::future::{ok, Ready, LocalBoxFuture};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

/// API Version enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiVersion {
    V1,
    V2,
    Latest,
}

impl ApiVersion {
    /// Parse version from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "v1" | "1" | "1.0" => Some(ApiVersion::V1),
            "v2" | "2" | "2.0" => Some(ApiVersion::V2),
            "latest" => Some(ApiVersion::Latest),
            _ => None,
        }
    }

    /// Get version string
    pub fn as_str(&self) -> &str {
        match self {
            ApiVersion::V1 => "v1",
            ApiVersion::V2 => "v2",
            ApiVersion::Latest => "v2", // Latest points to newest stable version
        }
    }
}

/// API versioning configuration
#[derive(Debug, Clone)]
pub struct VersioningConfig {
    /// Default version when none specified
    pub default_version: ApiVersion,
    /// Whether to allow version in URL path
    pub allow_url_versioning: bool,
    /// Whether to allow version in Accept header
    pub allow_header_versioning: bool,
    /// Custom header name for API version
    pub version_header: String,
    /// Whether to add version to response headers
    pub add_version_header: bool,
}

impl Default for VersioningConfig {
    fn default() -> Self {
        Self {
            default_version: ApiVersion::V1,
            allow_url_versioning: true,
            allow_header_versioning: true,
            version_header: "X-API-Version".to_string(),
            add_version_header: true,
        }
    }
}

/// API Versioning Middleware
pub struct VersioningMiddleware {
    config: Rc<VersioningConfig>,
}

impl VersioningMiddleware {
    pub fn new(config: VersioningConfig) -> Self {
        Self {
            config: Rc::new(config),
        }
    }

    /// Extract version from request
    fn extract_version(req: &ServiceRequest, config: &VersioningConfig) -> ApiVersion {
        // Check URL path first
        if config.allow_url_versioning {
            let path = req.path();
            if path.starts_with("/api/v1") {
                return ApiVersion::V1;
            } else if path.starts_with("/api/v2") {
                return ApiVersion::V2;
            }
        }

        // Check custom version header
        if config.allow_header_versioning {
            if let Some(header_value) = req.headers().get(&config.version_header) {
                if let Ok(version_str) = header_value.to_str() {
                    if let Some(version) = ApiVersion::from_str(version_str) {
                        return version;
                    }
                }
            }

            // Check Accept header for version
            if let Some(accept_header) = req.headers().get("Accept") {
                if let Ok(accept_str) = accept_header.to_str() {
                    // Parse version from Accept header (e.g., application/vnd.api+json;version=2)
                    if let Some(version_part) = accept_str.split(';').find(|s| s.contains("version=")) {
                        if let Some(version_str) = version_part.split('=').nth(1) {
                            if let Some(version) = ApiVersion::from_str(version_str.trim()) {
                                return version;
                            }
                        }
                    }
                }
            }
        }

        // Return default version
        config.default_version
    }
}

impl<S, B> Transform<S, ServiceRequest> for VersioningMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = VersioningMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(VersioningMiddlewareService {
            service: Rc::new(service),
            config: self.config.clone(),
        })
    }
}

pub struct VersioningMiddlewareService<S> {
    service: Rc<S>,
    config: Rc<VersioningConfig>,
}

impl<S> Clone for VersioningMiddlewareService<S> {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            config: self.config.clone(),
        }
    }
}

impl<S, B> Service<ServiceRequest> for VersioningMiddlewareService<S>
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

        Box::pin(async move {
            // Extract API version from request
            let version = VersioningMiddleware::extract_version(&req, &config);

            // Store version in request extensions for handlers to use
            req.extensions_mut().insert(version);

            // Rewrite path if using URL versioning but path doesn't include version
            let path = req.path().to_string();
            if !path.starts_with("/api/v") && path.starts_with("/api") {
                let version_prefix = format!("/api/{}", version.as_str());
                let new_path = path.replacen("/api", &version_prefix, 1);

                // Note: In production, you'd need to properly reconstruct the request
                // with the new path. This is a simplified example.
            }

            // Call the service
            let mut res = service.call(req).await?;

            // Add version header to response if configured
            if config.add_version_header {
                res.headers_mut().insert(
                    HeaderName::from_static("x-api-version"),
                    HeaderValue::from_static(version.as_str()),
                );
            }

            Ok(res)
        })
    }
}

/// Extension trait for extracting API version from request
pub trait ApiVersionExt {
    fn api_version(&self) -> ApiVersion;
}

impl ApiVersionExt for actix_web::HttpRequest {
    fn api_version(&self) -> ApiVersion {
        self.extensions()
            .get::<ApiVersion>()
            .copied()
            .unwrap_or(ApiVersion::V1)
    }
}

/// Version-specific route configuration helper
pub struct VersionedRoutes;

impl VersionedRoutes {
    /// Configure routes for a specific API version
    pub fn configure(version: ApiVersion) -> impl Fn(&mut actix_web::web::ServiceConfig) {
        move |cfg: &mut actix_web::web::ServiceConfig| {
            match version {
                ApiVersion::V1 => {
                    cfg.service(
                        actix_web::web::scope("/api/v1")
                            .configure(crate::v1::configure)
                    );
                }
                ApiVersion::V2 | ApiVersion::Latest => {
                    // V2 routes would go here when implemented
                    // For now, V2 falls back to V1 with deprecation warnings
                    cfg.service(
                        actix_web::web::scope("/api/v2")
                            .configure(crate::v1::configure)
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[test]
    fn test_version_parsing() {
        assert_eq!(ApiVersion::from_str("v1"), Some(ApiVersion::V1));
        assert_eq!(ApiVersion::from_str("1"), Some(ApiVersion::V1));
        assert_eq!(ApiVersion::from_str("v2"), Some(ApiVersion::V2));
        assert_eq!(ApiVersion::from_str("2.0"), Some(ApiVersion::V2));
        assert_eq!(ApiVersion::from_str("latest"), Some(ApiVersion::Latest));
        assert_eq!(ApiVersion::from_str("v3"), None);
    }

    #[test]
    fn test_version_string() {
        assert_eq!(ApiVersion::V1.as_str(), "v1");
        assert_eq!(ApiVersion::V2.as_str(), "v2");
        assert_eq!(ApiVersion::Latest.as_str(), "v2");
    }

    #[test]
    fn test_default_config() {
        let config = VersioningConfig::default();
        assert_eq!(config.default_version, ApiVersion::V1);
        assert!(config.allow_url_versioning);
        assert!(config.allow_header_versioning);
        assert_eq!(config.version_header, "X-API-Version");
        assert!(config.add_version_header);
    }
}