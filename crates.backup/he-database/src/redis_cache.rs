//! Redis-based distributed caching implementation for query results
//!
//! Provides a high-performance, distributed caching layer using Redis
//! with automatic serialization, TTL management, and connection pooling.

use anyhow::{Context, Result};
use bb8_redis::{bb8, RedisConnectionManager};
use redis::{AsyncCommands, Script};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, instrument, warn};

/// Redis cache configuration
#[derive(Clone, Debug)]
pub struct RedisCacheConfig {
    /// Redis connection URL
    pub url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Default TTL for cached values (in seconds)
    pub default_ttl: u64,
    /// Enable compression for values over this size (bytes)
    pub compression_threshold: usize,
    /// Key prefix for namespace isolation
    pub key_prefix: String,
}

impl Default for RedisCacheConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            max_connections: 20,
            connection_timeout: 5,
            default_ttl: 300, // 5 minutes
            compression_threshold: 1024, // 1KB
            key_prefix: "he:".to_string(),
        }
    }
}

/// Redis-backed distributed cache manager
pub struct RedisCache {
    /// Connection pool
    pool: bb8::Pool<RedisConnectionManager>,
    /// Configuration
    config: RedisCacheConfig,
    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,
}

impl RedisCache {
    /// Create a new Redis cache instance
    pub async fn new(config: RedisCacheConfig) -> Result<Self> {
        let manager = RedisConnectionManager::new(config.url.as_str())
            .context("Failed to create Redis connection manager")?;

        let pool = bb8::Pool::builder()
            .max_size(config.max_connections)
            .connection_timeout(Duration::from_secs(config.connection_timeout))
            .build(manager)
            .await
            .context("Failed to create Redis connection pool")?;

        Ok(Self {
            pool,
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        })
    }

    /// Get a value from cache
    #[instrument(skip(self))]
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let full_key = self.build_key(key);
        let mut conn = self.pool.get().await
            .context("Failed to get Redis connection")?;

        let mut stats = self.stats.write().await;

        match conn.get::<_, Option<Vec<u8>>>(&full_key).await {
            Ok(Some(data)) => {
                stats.hits += 1;
                debug!("Cache hit for key: {}", full_key);

                // Decompress if needed
                let decompressed = if data.len() > self.config.compression_threshold {
                    Self::decompress(&data)?
                } else {
                    data
                };

                // Deserialize
                let value = bincode::deserialize(&decompressed)
                    .context("Failed to deserialize cached value")?;
                Ok(Some(value))
            }
            Ok(None) => {
                stats.misses += 1;
                debug!("Cache miss for key: {}", full_key);
                Ok(None)
            }
            Err(e) => {
                error!("Redis get error: {}", e);
                stats.errors += 1;
                Ok(None) // Fail open - don't break on cache errors
            }
        }
    }

    /// Set a value in cache with custom TTL
    #[instrument(skip(self, value))]
    pub async fn set<T>(&self, key: &str, value: &T, ttl_seconds: Option<u64>) -> Result<()>
    where
        T: Serialize,
    {
        let full_key = self.build_key(key);
        let ttl = ttl_seconds.unwrap_or(self.config.default_ttl);

        let mut conn = self.pool.get().await
            .context("Failed to get Redis connection")?;

        // Serialize value
        let serialized = bincode::serialize(value)
            .context("Failed to serialize value")?;

        // Compress if needed
        let data = if serialized.len() > self.config.compression_threshold {
            Self::compress(&serialized)?
        } else {
            serialized
        };

        // Set with expiration
        conn.set_ex(&full_key, data, ttl as usize).await
            .context("Failed to set value in Redis")?;

        debug!("Cached key: {} with TTL: {}s", full_key, ttl);
        Ok(())
    }

    /// Set multiple values atomically
    #[instrument(skip(self, pairs))]
    pub async fn set_many<T>(&self, pairs: &[(&str, &T)], ttl_seconds: Option<u64>) -> Result<()>
    where
        T: Serialize,
    {
        let mut conn = self.pool.get().await
            .context("Failed to get Redis connection")?;
        let ttl = ttl_seconds.unwrap_or(self.config.default_ttl);

        // Use pipeline for atomic multi-set
        let mut pipe = redis::pipe();
        pipe.atomic();

        for (key, value) in pairs {
            let full_key = self.build_key(key);
            let serialized = bincode::serialize(value)?;
            let data = if serialized.len() > self.config.compression_threshold {
                Self::compress(&serialized)?
            } else {
                serialized
            };
            pipe.set_ex(full_key, data, ttl as usize);
        }

        pipe.query_async(&mut *conn).await
            .context("Failed to set multiple values")?;

        Ok(())
    }

    /// Delete a key from cache
    #[instrument(skip(self))]
    pub async fn delete(&self, key: &str) -> Result<()> {
        let full_key = self.build_key(key);
        let mut conn = self.pool.get().await
            .context("Failed to get Redis connection")?;

        conn.del(&full_key).await
            .context("Failed to delete key")?;

        debug!("Deleted cache key: {}", full_key);
        Ok(())
    }

    /// Delete keys matching a pattern
    #[instrument(skip(self))]
    pub async fn delete_pattern(&self, pattern: &str) -> Result<u64> {
        let full_pattern = self.build_key(pattern);
        let mut conn = self.pool.get().await
            .context("Failed to get Redis connection")?;

        // Use SCAN to find matching keys
        let keys: Vec<String> = conn.scan_match(&full_pattern).await
            .context("Failed to scan keys")?;

        if keys.is_empty() {
            return Ok(0);
        }

        let count = keys.len() as u64;

        // Delete in batches to avoid blocking
        for chunk in keys.chunks(1000) {
            conn.del(chunk).await
                .context("Failed to delete keys")?;
        }

        debug!("Deleted {} keys matching pattern: {}", count, full_pattern);
        Ok(count)
    }

    /// Clear entire cache (use with caution!)
    #[instrument(skip(self))]
    pub async fn flush_all(&self) -> Result<()> {
        let mut conn = self.pool.get().await
            .context("Failed to get Redis connection")?;

        // Only flush keys with our prefix
        let pattern = format!("{}*", self.config.key_prefix);
        let keys: Vec<String> = conn.scan_match(&pattern).await?;

        if !keys.is_empty() {
            conn.del(keys).await?;
        }

        warn!("Flushed all cache keys with prefix: {}", self.config.key_prefix);
        Ok(())
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// Check if key exists
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let full_key = self.build_key(key);
        let mut conn = self.pool.get().await?;
        let exists: bool = conn.exists(&full_key).await?;
        Ok(exists)
    }

    /// Get remaining TTL for a key
    pub async fn ttl(&self, key: &str) -> Result<Option<u64>> {
        let full_key = self.build_key(key);
        let mut conn = self.pool.get().await?;
        let ttl: i64 = conn.ttl(&full_key).await?;

        match ttl {
            -2 => Ok(None), // Key doesn't exist
            -1 => Ok(Some(u64::MAX)), // Key exists but has no TTL
            t => Ok(Some(t as u64)),
        }
    }

    /// Build full key with prefix
    fn build_key(&self, key: &str) -> String {
        format!("{}{}", self.config.key_prefix, key)
    }

    /// Compress data using zstd
    fn compress(data: &[u8]) -> Result<Vec<u8>> {
        zstd::encode_all(data, 3)
            .context("Failed to compress data")
    }

    /// Decompress data using zstd
    fn decompress(data: &[u8]) -> Result<Vec<u8>> {
        zstd::decode_all(data)
            .context("Failed to decompress data")
    }
}

/// Cache statistics
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub errors: u64,
    pub evictions: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// Query result caching layer
pub struct QueryCache {
    redis: Arc<RedisCache>,
    enabled: bool,
}

impl QueryCache {
    pub fn new(redis: Arc<RedisCache>) -> Self {
        Self {
            redis,
            enabled: true,
        }
    }

    /// Cache a database query result
    #[instrument(skip(self, result))]
    pub async fn cache_query<T>(&self, query: &str, params: &[&str], result: &T, ttl: Option<u64>) -> Result<()>
    where
        T: Serialize,
    {
        if !self.enabled {
            return Ok(());
        }

        let key = self.build_query_key(query, params);
        self.redis.set(&key, result, ttl).await
    }

    /// Get cached query result
    #[instrument(skip(self))]
    pub async fn get_query<T>(&self, query: &str, params: &[&str]) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        if !self.enabled {
            return Ok(None);
        }

        let key = self.build_query_key(query, params);
        self.redis.get(&key).await
    }

    /// Invalidate queries related to a table
    pub async fn invalidate_table(&self, table: &str) -> Result<u64> {
        let pattern = format!("query:{}:*", table);
        self.redis.delete_pattern(&pattern).await
    }

    /// Build cache key for a query
    fn build_query_key(&self, query: &str, params: &[&str]) -> String {
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();
        hasher.update(query);
        for param in params {
            hasher.update(param);
        }
        let hash = format!("{:x}", hasher.finalize());

        // Extract table name from query for invalidation
        let table = Self::extract_table_name(query).unwrap_or("unknown");

        format!("query:{}:{}", table, hash)
    }

    /// Extract table name from SQL query (simple heuristic)
    fn extract_table_name(query: &str) -> Option<String> {
        let query_lower = query.to_lowercase();

        // Look for FROM clause
        if let Some(from_pos) = query_lower.find(" from ") {
            let after_from = &query_lower[from_pos + 6..];
            let table_name = after_from
                .split_whitespace()
                .next()
                .map(|s| s.trim_end_matches(',').trim_end_matches(';'))
                .map(|s| s.to_string());
            return table_name;
        }

        // Look for INSERT INTO
        if let Some(insert_pos) = query_lower.find("insert into ") {
            let after_insert = &query_lower[insert_pos + 12..];
            let table_name = after_insert
                .split_whitespace()
                .next()
                .map(|s| s.to_string());
            return table_name;
        }

        // Look for UPDATE
        if let Some(update_pos) = query_lower.find("update ") {
            let after_update = &query_lower[update_pos + 7..];
            let table_name = after_update
                .split_whitespace()
                .next()
                .map(|s| s.to_string());
            return table_name;
        }

        None
    }
}

/// Cache key builders for common query patterns
pub mod query_keys {
    pub fn user_by_id(id: i64) -> String {
        format!("user:id:{}", id)
    }

    pub fn user_by_username(username: &str) -> String {
        format!("user:username:{}", username)
    }

    pub fn user_processes(user_id: i64) -> String {
        format!("user:{}:processes", user_id)
    }

    pub fn server_info(server_id: &str) -> String {
        format!("server:{}", server_id)
    }

    pub fn leaderboard(page: u32, per_page: u32) -> String {
        format!("leaderboard:{}:{}", page, per_page)
    }

    pub fn clan_members(clan_id: i64) -> String {
        format!("clan:{}:members", clan_id)
    }

    pub fn hardware_info(server_id: &str) -> String {
        format!("hardware:{}", server_id)
    }

    pub fn software_list(server_id: &str) -> String {
        format!("software:{}", server_id)
    }

    pub fn process_info(process_id: i64) -> String {
        format!("process:{}", process_id)
    }

    pub fn mission_list(user_id: i64) -> String {
        format!("missions:user:{}", user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_key_generation() {
        let cache = QueryCache::new(Arc::new(
            RedisCache::new(RedisCacheConfig::default()).await.unwrap()
        ));

        let key1 = cache.build_query_key("SELECT * FROM users WHERE id = $1", &["123"]);
        let key2 = cache.build_query_key("SELECT * FROM users WHERE id = $1", &["456"]);
        let key3 = cache.build_query_key("SELECT * FROM users WHERE id = $1", &["123"]);

        assert_ne!(key1, key2); // Different params = different keys
        assert_eq!(key1, key3); // Same query+params = same key
        assert!(key1.starts_with("query:users:"));
    }

    #[test]
    fn test_table_extraction() {
        assert_eq!(
            QueryCache::extract_table_name("SELECT * FROM users WHERE id = 1"),
            Some("users".to_string())
        );
        assert_eq!(
            QueryCache::extract_table_name("INSERT INTO players (name) VALUES ('test')"),
            Some("players".to_string())
        );
        assert_eq!(
            QueryCache::extract_table_name("UPDATE servers SET status = 'online'"),
            Some("servers".to_string())
        );
    }
}