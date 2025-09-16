//! Rate limiting implementation with Redis support for distributed systems

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use redis::{aio::ConnectionManager, AsyncCommands, Client};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Use Redis for distributed rate limiting
    pub use_redis: bool,
    /// Redis connection URL
    pub redis_url: Option<String>,
    /// Default requests per window
    pub default_requests: u32,
    /// Default time window in seconds
    pub default_window_seconds: u64,
    /// Login attempts per window
    pub login_attempts: u32,
    /// Login window in seconds
    pub login_window_seconds: u64,
    /// API requests per minute
    pub api_requests_per_minute: u32,
    /// API burst allowance
    pub api_burst_size: u32,
    /// Enable IP-based rate limiting
    pub ip_based: bool,
    /// Enable user-based rate limiting
    pub user_based: bool,
    /// Whitelist IPs (no rate limiting)
    pub whitelist_ips: Vec<IpAddr>,
    /// Blacklist IPs (always blocked)
    pub blacklist_ips: Vec<IpAddr>,
    /// Clean up interval in seconds
    pub cleanup_interval_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            use_redis: false,
            redis_url: None,
            default_requests: 100,
            default_window_seconds: 60,
            login_attempts: 5,
            login_window_seconds: 300,
            api_requests_per_minute: 60,
            api_burst_size: 10,
            ip_based: true,
            user_based: true,
            whitelist_ips: Vec::new(),
            blacklist_ips: Vec::new(),
            cleanup_interval_seconds: 300,
        }
    }
}

/// Rate limit rule for specific endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitRule {
    pub id: Uuid,
    pub name: String,
    pub path_pattern: String,
    pub method: Option<String>,
    pub requests: u32,
    pub window_seconds: u64,
    pub burst_size: u32,
    pub enabled: bool,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
}

impl RateLimitRule {
    pub fn new(name: &str, path_pattern: &str, requests: u32, window_seconds: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            path_pattern: path_pattern.to_string(),
            method: None,
            requests,
            window_seconds,
            burst_size: requests / 10,
            enabled: true,
            priority: 0,
            created_at: Utc::now(),
        }
    }

    pub fn with_method(mut self, method: &str) -> Self {
        self.method = Some(method.to_string());
        self
    }

    pub fn with_burst(mut self, burst_size: u32) -> Self {
        self.burst_size = burst_size;
        self
    }

    pub fn matches(&self, path: &str, method: &str) -> bool {
        if !self.enabled {
            return false;
        }

        let path_matches = if self.path_pattern.contains('*') {
            let pattern = self.path_pattern.replace("*", ".*");
            regex::Regex::new(&pattern)
                .map(|re| re.is_match(path))
                .unwrap_or(false)
        } else {
            path == self.path_pattern
        };

        let method_matches = self.method
            .as_ref()
            .map(|m| m == method)
            .unwrap_or(true);

        path_matches && method_matches
    }
}

/// Rate limit entry tracking requests
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RateLimitEntry {
    identifier: String,
    requests: Vec<u64>,  // Timestamps of requests
    window_start: u64,
    blocked_until: Option<u64>,
    violations: u32,
}

impl RateLimitEntry {
    fn new(identifier: String) -> Self {
        Self {
            identifier,
            requests: Vec::new(),
            window_start: current_timestamp(),
            blocked_until: None,
            violations: 0,
        }
    }

    fn is_blocked(&self) -> bool {
        if let Some(blocked_until) = self.blocked_until {
            current_timestamp() < blocked_until
        } else {
            false
        }
    }

    fn cleanup_old_requests(&mut self, window_seconds: u64) {
        let cutoff = current_timestamp() - (window_seconds * 1000);
        self.requests.retain(|&timestamp| timestamp > cutoff);
    }

    fn add_request(&mut self) {
        self.requests.push(current_timestamp());
    }

    fn request_count(&self, window_seconds: u64) -> u32 {
        let cutoff = current_timestamp() - (window_seconds * 1000);
        self.requests
            .iter()
            .filter(|&&timestamp| timestamp > cutoff)
            .count() as u32
    }
}

/// Token bucket for rate limiting
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenBucket {
    tokens: f64,
    capacity: f64,
    refill_rate: f64,
    last_refill: u64,
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill_rate,
            last_refill: current_timestamp(),
        }
    }

    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();

        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = current_timestamp();
        let elapsed = (now - self.last_refill) as f64 / 1000.0;
        let tokens_to_add = elapsed * self.refill_rate;

        self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
        self.last_refill = now;
    }
}

/// Rate limiter implementation
pub struct RateLimiter {
    config: RateLimitConfig,
    redis_client: Option<Arc<RwLock<ConnectionManager>>>,
    memory_store: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    token_buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    rules: Arc<RwLock<Vec<RateLimitRule>>>,
    statistics: Arc<RwLock<RateLimitStatistics>>,
}

/// Rate limit statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RateLimitStatistics {
    total_requests: u64,
    blocked_requests: u64,
    unique_identifiers: usize,
    violations: u64,
    last_reset: DateTime<Utc>,
}

/// Rate limit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub allowed: bool,
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: u64,
    pub retry_after: Option<u64>,
    pub reason: Option<String>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        let mut limiter = Self {
            config: config.clone(),
            redis_client: None,
            memory_store: Arc::new(RwLock::new(HashMap::new())),
            token_buckets: Arc::new(RwLock::new(HashMap::new())),
            rules: Arc::new(RwLock::new(Vec::new())),
            statistics: Arc::new(RwLock::new(RateLimitStatistics::default())),
        };

        // Initialize Redis if configured
        if config.use_redis {
            if let Some(redis_url) = &config.redis_url {
                limiter.init_redis(redis_url.clone());
            }
        }

        // Initialize default rules
        limiter.init_default_rules();

        // Start cleanup task
        limiter.start_cleanup_task();

        limiter
    }

    /// Initialize Redis connection
    fn init_redis(&mut self, redis_url: String) {
        let redis_client = self.redis_client.clone();
        tokio::spawn(async move {
            match Client::open(redis_url.as_str()) {
                Ok(client) => {
                    match ConnectionManager::new(client).await {
                        Ok(conn) => {
                            info!("Redis connection established for rate limiting");
                        }
                        Err(e) => {
                            error!("Failed to create Redis connection manager: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to connect to Redis: {}", e);
                }
            }
        });
    }

    /// Initialize default rate limit rules
    fn init_default_rules(&self) {
        let rules = self.rules.clone();
        tokio::spawn(async move {
            let mut rules_guard = rules.write().await;

            // Login endpoint
            rules_guard.push(
                RateLimitRule::new("/api/auth/login", "/api/auth/login", 5, 300)
                    .with_method("POST")
                    .with_burst(2)
            );

            // Registration endpoint
            rules_guard.push(
                RateLimitRule::new("/api/auth/register", "/api/auth/register", 3, 3600)
                    .with_method("POST")
                    .with_burst(1)
            );

            // Password reset
            rules_guard.push(
                RateLimitRule::new("/api/auth/reset-password", "/api/auth/reset-password", 3, 3600)
                    .with_method("POST")
            );

            // API endpoints
            rules_guard.push(
                RateLimitRule::new("API Default", "/api/*", 100, 60)
                    .with_burst(20)
            );

            // Game actions
            rules_guard.push(
                RateLimitRule::new("Game Actions", "/api/game/*", 60, 60)
                    .with_burst(10)
            );

            info!("Initialized {} default rate limit rules", rules_guard.len());
        });
    }

    /// Check rate limit for an identifier
    pub async fn check_rate_limit(
        &self,
        identifier: &str,
        requests: u32,
        window_seconds: u64,
    ) -> RateLimit {
        if !self.config.enabled {
            return RateLimit {
                allowed: true,
                limit: requests,
                remaining: requests,
                reset_at: 0,
                retry_after: None,
                reason: None,
            };
        }

        // Check blacklist
        if let Ok(ip) = identifier.parse::<IpAddr>() {
            if self.config.blacklist_ips.contains(&ip) {
                return RateLimit {
                    allowed: false,
                    limit: 0,
                    remaining: 0,
                    reset_at: 0,
                    retry_after: Some(3600),
                    reason: Some("IP address is blacklisted".to_string()),
                };
            }

            // Check whitelist
            if self.config.whitelist_ips.contains(&ip) {
                return RateLimit {
                    allowed: true,
                    limit: u32::MAX,
                    remaining: u32::MAX,
                    reset_at: 0,
                    retry_after: None,
                    reason: None,
                };
            }
        }

        // Try Redis first if available
        if self.redis_client.is_some() {
            if let Ok(result) = self.check_redis_rate_limit(identifier, requests, window_seconds).await {
                return result;
            }
        }

        // Fall back to memory store
        self.check_memory_rate_limit(identifier, requests, window_seconds).await
    }

    /// Check rate limit using Redis
    async fn check_redis_rate_limit(
        &self,
        identifier: &str,
        requests: u32,
        window_seconds: u64,
    ) -> Result<RateLimit> {
        if let Some(redis) = &self.redis_client {
            let mut conn = redis.write().await;
            let key = format!("rate_limit:{}", identifier);
            let now = current_timestamp();
            let window_ms = window_seconds * 1000;

            // Remove old entries
            let _: Result<(), _> = conn.zremrangebyscore(
                &key,
                0,
                (now - window_ms) as isize
            ).await;

            // Count requests in window
            let count: u32 = conn.zcount(&key, (now - window_ms) as isize, now as isize).await?;

            if count >= requests {
                // Rate limit exceeded
                let oldest: Vec<(String, f64)> = conn.zrange_withscores(&key, 0, 0).await?;
                let reset_at = if !oldest.is_empty() {
                    (oldest[0].1 as u64) + window_ms
                } else {
                    now + window_ms
                };

                return Ok(RateLimit {
                    allowed: false,
                    limit: requests,
                    remaining: 0,
                    reset_at: reset_at / 1000,
                    retry_after: Some((reset_at - now) / 1000),
                    reason: Some("Rate limit exceeded".to_string()),
                });
            }

            // Add current request
            let _: Result<(), _> = conn.zadd(&key, &identifier, now as f64).await;
            let _: Result<(), _> = conn.expire(&key, window_seconds as usize).await;

            Ok(RateLimit {
                allowed: true,
                limit: requests,
                remaining: requests - count - 1,
                reset_at: (now + window_ms) / 1000,
                retry_after: None,
                reason: None,
            })
        } else {
            Err(anyhow!("Redis client not available"))
        }
    }

    /// Check rate limit using memory store
    async fn check_memory_rate_limit(
        &self,
        identifier: &str,
        requests: u32,
        window_seconds: u64,
    ) -> RateLimit {
        let mut store = self.memory_store.write().await;
        let entry = store.entry(identifier.to_string())
            .or_insert_with(|| RateLimitEntry::new(identifier.to_string()));

        // Update statistics
        {
            let mut stats = self.statistics.write().await;
            stats.total_requests += 1;
            stats.unique_identifiers = store.len();
        }

        // Check if blocked
        if entry.is_blocked() {
            let blocked_until = entry.blocked_until.unwrap();
            let now = current_timestamp();

            {
                let mut stats = self.statistics.write().await;
                stats.blocked_requests += 1;
            }

            return RateLimit {
                allowed: false,
                limit: requests,
                remaining: 0,
                reset_at: blocked_until / 1000,
                retry_after: Some((blocked_until - now) / 1000),
                reason: Some("Temporarily blocked due to violations".to_string()),
            };
        }

        // Clean up old requests
        entry.cleanup_old_requests(window_seconds);

        // Check current request count
        let current_count = entry.request_count(window_seconds);

        if current_count >= requests {
            // Rate limit exceeded
            entry.violations += 1;

            // Block if too many violations
            if entry.violations >= 3 {
                entry.blocked_until = Some(current_timestamp() + (300 * 1000)); // Block for 5 minutes
                warn!("Identifier {} blocked due to repeated violations", identifier);
            }

            {
                let mut stats = self.statistics.write().await;
                stats.blocked_requests += 1;
                stats.violations += 1;
            }

            return RateLimit {
                allowed: false,
                limit: requests,
                remaining: 0,
                reset_at: (current_timestamp() + (window_seconds * 1000)) / 1000,
                retry_after: Some(window_seconds),
                reason: Some("Rate limit exceeded".to_string()),
            };
        }

        // Allow request
        entry.add_request();

        RateLimit {
            allowed: true,
            limit: requests,
            remaining: requests - current_count - 1,
            reset_at: (current_timestamp() + (window_seconds * 1000)) / 1000,
            retry_after: None,
            reason: None,
        }
    }

    /// Check rate limit for a specific endpoint
    pub async fn check_endpoint_rate_limit(
        &self,
        identifier: &str,
        path: &str,
        method: &str,
    ) -> RateLimit {
        let rules = self.rules.read().await;

        // Find matching rule with highest priority
        let matching_rule = rules
            .iter()
            .filter(|r| r.matches(path, method))
            .max_by_key(|r| r.priority);

        if let Some(rule) = matching_rule {
            self.check_rate_limit(
                &format!("{}:{}", identifier, rule.id),
                rule.requests,
                rule.window_seconds,
            ).await
        } else {
            // Use default limits
            self.check_rate_limit(
                identifier,
                self.config.default_requests,
                self.config.default_window_seconds,
            ).await
        }
    }

    /// Check login rate limit
    pub async fn check_login_rate(&self, identifier: &str) -> bool {
        let result = self.check_rate_limit(
            &format!("login:{}", identifier),
            self.config.login_attempts,
            self.config.login_window_seconds,
        ).await;

        result.allowed
    }

    /// Record failed login attempt
    pub async fn record_failed_login(&self, identifier: &str) {
        let key = format!("failed_login:{}", identifier);

        if let Some(redis) = &self.redis_client {
            let mut conn = redis.write().await;
            let _: Result<(), _> = conn.incr(&key, 1).await;
            let _: Result<(), _> = conn.expire(&key, 3600).await;
        } else {
            let mut store = self.memory_store.write().await;
            let entry = store.entry(key)
                .or_insert_with(|| RateLimitEntry::new(key.clone()));
            entry.add_request();
        }

        let mut stats = self.statistics.write().await;
        stats.violations += 1;
    }

    /// Get failed login attempts count
    pub async fn get_failed_attempts_count(&self) -> u64 {
        let stats = self.statistics.read().await;
        stats.violations
    }

    /// Use token bucket algorithm for burst handling
    pub async fn check_token_bucket(
        &self,
        identifier: &str,
        tokens: f64,
        capacity: f64,
        refill_rate: f64,
    ) -> bool {
        let mut buckets = self.token_buckets.write().await;
        let bucket = buckets.entry(identifier.to_string())
            .or_insert_with(|| TokenBucket::new(capacity, refill_rate));

        bucket.try_consume(tokens)
    }

    /// Add custom rate limit rule
    pub async fn add_rule(&self, rule: RateLimitRule) -> Result<()> {
        let mut rules = self.rules.write().await;

        // Check for duplicate
        if rules.iter().any(|r| r.name == rule.name) {
            return Err(anyhow!("Rule with name '{}' already exists", rule.name));
        }

        rules.push(rule);
        rules.sort_by_key(|r| -r.priority);

        Ok(())
    }

    /// Remove rate limit rule
    pub async fn remove_rule(&self, rule_id: &Uuid) -> Result<()> {
        let mut rules = self.rules.write().await;
        let before_len = rules.len();
        rules.retain(|r| r.id != *rule_id);

        if rules.len() < before_len {
            Ok(())
        } else {
            Err(anyhow!("Rule not found"))
        }
    }

    /// Get all rules
    pub async fn get_rules(&self) -> Vec<RateLimitRule> {
        let rules = self.rules.read().await;
        rules.clone()
    }

    /// Reset rate limit for an identifier
    pub async fn reset_rate_limit(&self, identifier: &str) -> Result<()> {
        if let Some(redis) = &self.redis_client {
            let mut conn = redis.write().await;
            let key = format!("rate_limit:{}", identifier);
            let _: Result<(), _> = conn.del(&key).await;
        }

        let mut store = self.memory_store.write().await;
        store.remove(identifier);

        info!("Reset rate limit for identifier: {}", identifier);
        Ok(())
    }

    /// Get rate limit statistics
    pub async fn get_statistics(&self) -> RateLimitStats {
        let stats = self.statistics.read().await;
        let store = self.memory_store.read().await;
        let rules = self.rules.read().await;

        RateLimitStats {
            total_requests: stats.total_requests,
            blocked_requests: stats.blocked_requests,
            unique_identifiers: stats.unique_identifiers,
            violations: stats.violations,
            active_entries: store.len(),
            total_rules: rules.len(),
            enabled: self.config.enabled,
            using_redis: self.redis_client.is_some(),
        }
    }

    /// Start cleanup task
    fn start_cleanup_task(&self) {
        let store = self.memory_store.clone();
        let buckets = self.token_buckets.clone();
        let interval = self.config.cleanup_interval_seconds;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval));

            loop {
                interval.tick().await;

                // Clean up old entries
                {
                    let mut store_guard = store.write().await;
                    let now = current_timestamp();

                    store_guard.retain(|_, entry| {
                        // Keep if blocked or has recent activity
                        entry.is_blocked() ||
                        !entry.requests.is_empty() &&
                        entry.requests.last().map_or(false, |&last| {
                            now - last < 3600000 // Keep for 1 hour
                        })
                    });
                }

                // Clean up unused token buckets
                {
                    let mut buckets_guard = buckets.write().await;
                    let now = current_timestamp();

                    buckets_guard.retain(|_, bucket| {
                        now - bucket.last_refill < 3600000 // Keep for 1 hour
                    });
                }

                debug!("Rate limiter cleanup completed");
            }
        });
    }
}

/// Rate limit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitStats {
    pub total_requests: u64,
    pub blocked_requests: u64,
    pub unique_identifiers: usize,
    pub violations: u64,
    pub active_entries: usize,
    pub total_rules: usize,
    pub enabled: bool,
    pub using_redis: bool,
}

/// Get current timestamp in milliseconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_rate_limit() {
        let config = RateLimitConfig {
            enabled: true,
            default_requests: 5,
            default_window_seconds: 1,
            ..Default::default()
        };

        let limiter = RateLimiter::new(config);

        // Should allow first 5 requests
        for i in 0..5 {
            let result = limiter.check_rate_limit("test_user", 5, 1).await;
            assert!(result.allowed, "Request {} should be allowed", i + 1);
            assert_eq!(result.remaining, 4 - i);
        }

        // 6th request should be blocked
        let result = limiter.check_rate_limit("test_user", 5, 1).await;
        assert!(!result.allowed);
        assert_eq!(result.remaining, 0);
        assert!(result.retry_after.is_some());
    }

    #[tokio::test]
    async fn test_token_bucket() {
        let limiter = RateLimiter::new(RateLimitConfig::default());

        // Create bucket with capacity 10, refill rate 5/sec
        let identifier = "test_bucket";

        // Should allow consuming 5 tokens
        assert!(limiter.check_token_bucket(identifier, 5.0, 10.0, 5.0).await);

        // Should allow consuming another 5 tokens
        assert!(limiter.check_token_bucket(identifier, 5.0, 10.0, 5.0).await);

        // Should not allow consuming 1 more token (bucket empty)
        assert!(!limiter.check_token_bucket(identifier, 1.0, 10.0, 5.0).await);

        // Wait for refill
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Should allow consuming tokens after refill
        assert!(limiter.check_token_bucket(identifier, 2.0, 10.0, 5.0).await);
    }

    #[tokio::test]
    async fn test_rate_limit_rules() {
        let limiter = RateLimiter::new(RateLimitConfig::default());

        let rule = RateLimitRule::new("test_rule", "/api/test", 10, 60)
            .with_method("GET");

        limiter.add_rule(rule.clone()).await.unwrap();

        let rules = limiter.get_rules().await;
        assert!(rules.iter().any(|r| r.name == "test_rule"));

        // Test rule matching
        assert!(rule.matches("/api/test", "GET"));
        assert!(!rule.matches("/api/test", "POST"));
        assert!(!rule.matches("/api/other", "GET"));
    }

    #[tokio::test]
    async fn test_wildcard_rules() {
        let rule = RateLimitRule::new("api_rule", "/api/*", 100, 60);

        assert!(rule.matches("/api/users", "GET"));
        assert!(rule.matches("/api/posts/123", "POST"));
        assert!(!rule.matches("/auth/login", "POST"));
    }

    #[tokio::test]
    async fn test_blacklist_whitelist() {
        let config = RateLimitConfig {
            enabled: true,
            whitelist_ips: vec!["192.168.1.1".parse().unwrap()],
            blacklist_ips: vec!["10.0.0.1".parse().unwrap()],
            ..Default::default()
        };

        let limiter = RateLimiter::new(config);

        // Whitelisted IP should always be allowed
        let result = limiter.check_rate_limit("192.168.1.1", 1, 1).await;
        assert!(result.allowed);
        assert_eq!(result.remaining, u32::MAX);

        // Blacklisted IP should always be blocked
        let result = limiter.check_rate_limit("10.0.0.1", 100, 100).await;
        assert!(!result.allowed);
        assert_eq!(result.reason, Some("IP address is blacklisted".to_string()));
    }

    #[tokio::test]
    async fn test_login_rate_limit() {
        let config = RateLimitConfig {
            enabled: true,
            login_attempts: 3,
            login_window_seconds: 2,
            ..Default::default()
        };

        let limiter = RateLimiter::new(config);

        // Should allow first 3 login attempts
        for _ in 0..3 {
            assert!(limiter.check_login_rate("user@example.com").await);
        }

        // 4th attempt should be blocked
        assert!(!limiter.check_login_rate("user@example.com").await);

        // Record failed login
        limiter.record_failed_login("user@example.com").await;
        assert!(limiter.get_failed_attempts_count().await > 0);
    }
}