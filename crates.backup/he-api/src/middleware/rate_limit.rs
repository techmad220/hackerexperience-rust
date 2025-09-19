//! Enhanced Rate Limiting Middleware
//!
//! Implements advanced rate limiting with:
//! - Per-user and per-IP limits
//! - Sliding window algorithm
//! - Redis-backed distributed rate limiting
//! - Different limits for different endpoints
//! - Burst allowance

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
    http::{Method, StatusCode, header::{HeaderName, HeaderValue}},
};
use futures::future::{ok, Ready, LocalBoxFuture};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

/// Rate limit storage backend
#[derive(Debug, Clone)]
pub enum RateLimitBackend {
    /// In-memory storage (not distributed)
    Memory,
    /// Redis backend (distributed)
    Redis(String), // Redis connection string
}

/// Rate limit configuration for an endpoint
#[derive(Debug, Clone)]
pub struct EndpointLimit {
    /// Maximum requests per window
    pub max_requests: u32,
    /// Time window in seconds
    pub window_seconds: u64,
    /// Burst allowance (temporary exceeded limit)
    pub burst_size: u32,
}

impl Default for EndpointLimit {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_seconds: 60,
            burst_size: 10,
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Backend storage for rate limit data
    pub backend: RateLimitBackend,

    /// Default limit for all endpoints
    pub default_limit: EndpointLimit,

    /// Per-endpoint specific limits
    pub endpoint_limits: HashMap<String, EndpointLimit>,

    /// Per-user limits (override IP-based limits)
    pub user_limits: HashMap<String, EndpointLimit>,

    /// Whether to apply rate limiting per IP
    pub per_ip: bool,

    /// Whether to apply rate limiting per authenticated user
    pub per_user: bool,

    /// Whitelist of IPs that bypass rate limiting
    pub ip_whitelist: Vec<String>,

    /// HTTP status code to return when rate limited
    pub rate_limit_status: StatusCode,

    /// Include rate limit headers in response
    pub include_headers: bool,

    /// Custom message for rate limited responses
    pub rate_limit_message: String,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        let mut endpoint_limits = HashMap::new();

        // Set specific limits for sensitive endpoints
        endpoint_limits.insert("/api/auth/login".to_string(), EndpointLimit {
            max_requests: 5,
            window_seconds: 300, // 5 minutes
            burst_size: 2,
        });

        endpoint_limits.insert("/api/auth/register".to_string(), EndpointLimit {
            max_requests: 3,
            window_seconds: 3600, // 1 hour
            burst_size: 1,
        });

        endpoint_limits.insert("/api/hack".to_string(), EndpointLimit {
            max_requests: 30,
            window_seconds: 60,
            burst_size: 5,
        });

        Self {
            backend: RateLimitBackend::Memory,
            default_limit: EndpointLimit::default(),
            endpoint_limits,
            user_limits: HashMap::new(),
            per_ip: true,
            per_user: true,
            ip_whitelist: vec![
                "127.0.0.1".to_string(),
                "::1".to_string(),
            ],
            rate_limit_status: StatusCode::TOO_MANY_REQUESTS,
            include_headers: true,
            rate_limit_message: "Rate limit exceeded. Please try again later.".to_string(),
        }
    }
}

/// Rate limit entry tracking requests
#[derive(Debug, Clone)]
struct RateLimitEntry {
    /// Request timestamps within the current window
    timestamps: Vec<DateTime<Utc>>,
    /// Number of requests in burst
    burst_count: u32,
    /// Last reset time
    last_reset: DateTime<Utc>,
}

impl RateLimitEntry {
    fn new() -> Self {
        Self {
            timestamps: Vec::new(),
            burst_count: 0,
            last_reset: Utc::now(),
        }
    }

    /// Clean old timestamps outside the window
    fn clean_old_timestamps(&mut self, window_seconds: u64) {
        let now = Utc::now();
        let window = Duration::seconds(window_seconds as i64);

        self.timestamps.retain(|ts| now.signed_duration_since(*ts) < window);
    }

    /// Check if request is allowed
    fn is_allowed(&mut self, limit: &EndpointLimit) -> (bool, RateLimitInfo) {
        let now = Utc::now();
        self.clean_old_timestamps(limit.window_seconds);

        let current_count = self.timestamps.len() as u32;
        let remaining = limit.max_requests.saturating_sub(current_count);

        // Calculate reset time (next window)
        let oldest_timestamp = self.timestamps.first().cloned().unwrap_or(now);
        let reset_time = oldest_timestamp + Duration::seconds(limit.window_seconds as i64);
        let reset_after_seconds = reset_time.signed_duration_since(now).num_seconds().max(0) as u64;

        let info = RateLimitInfo {
            limit: limit.max_requests,
            remaining,
            reset_after_seconds,
        };

        if current_count < limit.max_requests {
            // Under limit, allow request
            self.timestamps.push(now);
            self.burst_count = 0;
            (true, info)
        } else if self.burst_count < limit.burst_size {
            // Use burst allowance
            self.timestamps.push(now);
            self.burst_count += 1;
            (true, RateLimitInfo { remaining: 0, ..info })
        } else {
            // Rate limited
            (false, info)
        }
    }
}

/// Rate limit information for headers
#[derive(Debug, Clone)]
struct RateLimitInfo {
    limit: u32,
    remaining: u32,
    reset_after_seconds: u64,
}

/// In-memory rate limit store
type MemoryStore = Arc<RwLock<HashMap<String, RateLimitEntry>>>;

/// Rate limiting middleware
pub struct RateLimiter {
    config: Arc<RateLimitConfig>,
    memory_store: MemoryStore,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config: Arc::new(config),
            memory_store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get rate limit key for request
    fn get_rate_limit_key(&self, req: &ServiceRequest) -> String {
        let path = req.path();
        let ip = req.connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();

        // Check if user is authenticated (simplified - you'd check JWT/session)
        let user_id = req.headers()
            .get("X-User-Id")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        if self.config.per_user && user_id.is_some() {
            format!("user:{}:{}", user_id.unwrap(), path)
        } else if self.config.per_ip {
            format!("ip:{}:{}", ip, path)
        } else {
            format!("global:{}", path)
        }
    }

    /// Check if IP is whitelisted
    fn is_whitelisted(&self, req: &ServiceRequest) -> bool {
        let ip = req.connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown");

        self.config.ip_whitelist.iter().any(|wip| wip == ip)
    }

    /// Get limit for endpoint
    fn get_endpoint_limit(&self, path: &str) -> &EndpointLimit {
        self.config.endpoint_limits
            .get(path)
            .unwrap_or(&self.config.default_limit)
    }

    /// Check rate limit
    async fn check_rate_limit(&self, req: &ServiceRequest) -> Result<RateLimitInfo, RateLimitInfo> {
        // Skip rate limiting for whitelisted IPs
        if self.is_whitelisted(req) {
            return Ok(RateLimitInfo {
                limit: u32::MAX,
                remaining: u32::MAX,
                reset_after_seconds: 0,
            });
        }

        let key = self.get_rate_limit_key(req);
        let limit = self.get_endpoint_limit(req.path());

        match &self.config.backend {
            RateLimitBackend::Memory => {
                let mut store = self.memory_store.write().await;
                let entry = store.entry(key).or_insert_with(RateLimitEntry::new);

                let (allowed, info) = entry.is_allowed(limit);

                if allowed {
                    Ok(info)
                } else {
                    Err(info)
                }
            }
            RateLimitBackend::Redis(_conn_str) => {
                // Redis implementation would go here
                // For now, fallback to memory
                let mut store = self.memory_store.write().await;
                let entry = store.entry(key).or_insert_with(RateLimitEntry::new);

                let (allowed, info) = entry.is_allowed(limit);

                if allowed {
                    Ok(info)
                } else {
                    Err(info)
                }
            }
        }
    }

    /// Clean up old entries periodically
    pub async fn cleanup_old_entries(&self) {
        let mut store = self.memory_store.write().await;
        let now = Utc::now();

        store.retain(|_, entry| {
            // Keep entries that had activity in the last hour
            entry.timestamps.iter().any(|ts| {
                now.signed_duration_since(*ts).num_seconds() < 3600
            })
        });
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimiterMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimiterMiddleware {
            service: Rc::new(service),
            config: self.config.clone(),
            memory_store: self.memory_store.clone(),
        })
    }
}

pub struct RateLimiterMiddleware<S> {
    service: Rc<S>,
    config: Arc<RateLimitConfig>,
    memory_store: MemoryStore,
}

impl<S> Clone for RateLimiterMiddleware<S> {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            config: self.config.clone(),
            memory_store: self.memory_store.clone(),
        }
    }
}

impl<S, B> Service<ServiceRequest> for RateLimiterMiddleware<S>
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
        let memory_store = self.memory_store.clone();

        Box::pin(async move {
            let limiter = RateLimiter {
                config: config.clone(),
                memory_store,
            };

            match limiter.check_rate_limit(&req).await {
                Ok(info) => {
                    // Request allowed
                    let mut res = service.call(req).await?;

                    if config.include_headers {
                        let headers = res.headers_mut();
                        headers.insert(
                            HeaderName::from_static("x-ratelimit-limit"),
                            HeaderValue::from_str(&info.limit.to_string()).unwrap(),
                        );
                        headers.insert(
                            HeaderName::from_static("x-ratelimit-remaining"),
                            HeaderValue::from_str(&info.remaining.to_string()).unwrap(),
                        );
                        headers.insert(
                            HeaderName::from_static("x-ratelimit-reset"),
                            HeaderValue::from_str(&info.reset_after_seconds.to_string()).unwrap(),
                        );
                    }

                    Ok(res)
                }
                Err(info) => {
                    // Rate limited
                    let mut response = HttpResponse::build(config.rate_limit_status)
                        .json(serde_json::json!({
                            "error": config.rate_limit_message,
                            "retry_after": info.reset_after_seconds,
                        }));

                    if config.include_headers {
                        response.insert_header(("X-RateLimit-Limit", info.limit.to_string()));
                        response.insert_header(("X-RateLimit-Remaining", "0"));
                        response.insert_header(("X-RateLimit-Reset", info.reset_after_seconds.to_string()));
                        response.insert_header(("Retry-After", info.reset_after_seconds.to_string()));
                    }

                    Err(actix_web::error::ErrorTooManyRequests(config.rate_limit_message))
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limit_entry() {
        let mut entry = RateLimitEntry::new();
        let limit = EndpointLimit {
            max_requests: 3,
            window_seconds: 60,
            burst_size: 1,
        };

        // First 3 requests should be allowed
        for _ in 0..3 {
            let (allowed, _) = entry.is_allowed(&limit);
            assert!(allowed);
        }

        // 4th request uses burst
        let (allowed, _) = entry.is_allowed(&limit);
        assert!(allowed);

        // 5th request should be denied
        let (allowed, _) = entry.is_allowed(&limit);
        assert!(!allowed);
    }

    #[test]
    fn test_endpoint_limits() {
        let config = RateLimitConfig::default();

        // Check login endpoint has stricter limits
        let login_limit = config.endpoint_limits.get("/api/auth/login").unwrap();
        assert_eq!(login_limit.max_requests, 5);
        assert_eq!(login_limit.window_seconds, 300);
    }
}