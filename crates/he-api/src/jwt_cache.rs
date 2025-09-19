use anyhow::{anyhow, Result};
//! JWT caching for WebSocket connections

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use he_auth::jwt::JwtClaims;

/// Cache entry for validated JWT tokens
#[derive(Clone, Debug)]
pub struct CachedJwtClaims {
    pub claims: JwtClaims,
    pub validated_at: Instant,
    pub token_hash: String,
}

/// JWT cache configuration
#[derive(Clone, Debug)]
pub struct JwtCacheConfig {
    /// Maximum number of cached tokens
    pub max_entries: usize,
    /// How long to cache validated tokens (default: 5 minutes)
    pub ttl: Duration,
    /// How often to clean expired entries (default: 1 minute)
    pub cleanup_interval: Duration,
}

impl Default for JwtCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            ttl: Duration::from_secs(300), // 5 minutes
            cleanup_interval: Duration::from_secs(60), // 1 minute
        }
    }
}

/// Thread-safe JWT cache for WebSocket connections
pub struct JwtCache {
    cache: Arc<RwLock<HashMap<String, CachedJwtClaims>>>,
    config: JwtCacheConfig,
}

impl JwtCache {
    /// Create a new JWT cache with default configuration
    pub fn new() -> Self {
        Self::with_config(JwtCacheConfig::default())
    }

    /// Create a new JWT cache with custom configuration
    pub fn with_config(config: JwtCacheConfig) -> Self {
        let cache = Arc::new(RwLock::new(HashMap::new()));
        let cache_clone = cache.clone();
        let cleanup_interval = config.cleanup_interval;
        let ttl = config.ttl;

        // Spawn cleanup task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            loop {
                interval.tick().await;
                Self::cleanup_expired_entries(cache_clone.clone(), ttl).await;
            }
        });

        Self { cache, config }
    }

    /// Get cached JWT claims if available and not expired
    pub async fn get(&self, token: &str) -> Option<JwtClaims> {
        let token_hash = Self::hash_token(token);
        let cache = self.cache.read().await;

        if let Some(entry) = cache.get(&token_hash) {
            // Check if entry is still valid
            if entry.validated_at.elapsed() < self.config.ttl {
                return Some(entry.claims.clone());
            }
        }

        None
    }

    /// Store validated JWT claims in cache
    pub async fn insert(&self, token: &str, claims: JwtClaims) {
        let token_hash = Self::hash_token(token);
        let entry = CachedJwtClaims {
            claims,
            validated_at: Instant::now(),
            token_hash: token_hash.clone(),
        };

        let mut cache = self.cache.write().await;

        // Check cache size limit
        if cache.len() >= self.config.max_entries {
            // Remove oldest entry
            if let Some(oldest_key) = cache
                .iter()
                .min_by_key(|(_, v)| v.validated_at)
                .map(|(k, _)| k.clone())
            {
                cache.remove(&oldest_key);
            }
        }

        cache.insert(token_hash, entry);
    }

    /// Remove a token from the cache
    pub async fn remove(&self, token: &str) {
        let token_hash = Self::hash_token(token);
        let mut cache = self.cache.write().await;
        cache.remove(&token_hash);
    }

    /// Clear all cached entries
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get the current number of cached entries
    pub async fn size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }

    /// Hash the token to use as cache key
    fn hash_token(token: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Remove expired entries from cache
    async fn cleanup_expired_entries(
        cache: Arc<RwLock<HashMap<String, CachedJwtClaims>>>,
        ttl: Duration,
    ) {
        let mut cache = cache.write().await;
        let now = Instant::now();

        cache.retain(|_, entry| {
            now.duration_since(entry.validated_at) < ttl
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_claims() -> JwtClaims {
        JwtClaims {
            user_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            exp: chrono::Utc::now().timestamp() + 3600,
            iat: chrono::Utc::now().timestamp(),
            jti: Uuid::new_v4(),
        }
    }

    #[tokio::test]
    async fn test_cache_insert_and_get() {
        let cache = JwtCache::new();
        let token = "test_token";
        let claims = create_test_claims();

        // Insert claims
        cache.insert(token, claims.clone()).await;

        // Get claims
        let cached_claims = cache.get(token).await;
        assert!(cached_claims.is_some());
        assert_eq!(cached_claims.map_err(|e| anyhow::anyhow!("Error: {}", e))?.user_id, claims.user_id);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let config = JwtCacheConfig {
            max_entries: 100,
            ttl: Duration::from_millis(100),
            cleanup_interval: Duration::from_secs(60),
        };

        let cache = JwtCache::with_config(config);
        let token = "test_token";
        let claims = create_test_claims();

        // Insert claims
        cache.insert(token, claims).await;

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should return None after expiration
        let cached_claims = cache.get(token).await;
        assert!(cached_claims.is_none());
    }

    #[tokio::test]
    async fn test_cache_size_limit() {
        let config = JwtCacheConfig {
            max_entries: 2,
            ttl: Duration::from_secs(300),
            cleanup_interval: Duration::from_secs(60),
        };

        let cache = JwtCache::with_config(config);

        // Insert 3 tokens (exceeding max_entries)
        for i in 0..3 {
            let token = format!("token_{}", i);
            let claims = create_test_claims();
            cache.insert(&token, claims).await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        // Cache should only have 2 entries
        assert_eq!(cache.size().await, 2);

        // First token should be evicted
        assert!(cache.get("token_0").await.is_none());
        // Last two tokens should be present
        assert!(cache.get("token_1").await.is_some());
        assert!(cache.get("token_2").await.is_some());
    }

    #[tokio::test]
    async fn test_cache_remove() {
        let cache = JwtCache::new();
        let token = "test_token";
        let claims = create_test_claims();

        // Insert and verify
        cache.insert(token, claims).await;
        assert!(cache.get(token).await.is_some());

        // Remove and verify
        cache.remove(token).await;
        assert!(cache.get(token).await.is_none());
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = JwtCache::new();

        // Insert multiple tokens
        for i in 0..5 {
            let token = format!("token_{}", i);
            let claims = create_test_claims();
            cache.insert(&token, claims).await;
        }

        assert_eq!(cache.size().await, 5);

        // Clear cache
        cache.clear().await;
        assert_eq!(cache.size().await, 0);
    }
}