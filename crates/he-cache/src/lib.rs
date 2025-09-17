//! High-performance caching layer for HackerExperience

use bb8_redis::{bb8, RedisConnectionManager};
use redis::{AsyncCommands, RedisError};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tracing::{debug, warn};
use uuid::Uuid;
use prometheus::{IntCounter, Histogram, register_int_counter, register_histogram};

pub type RedisPool = bb8::Pool<RedisConnectionManager>;

lazy_static::lazy_static! {
    static ref CACHE_HITS: IntCounter = register_int_counter!(
        "cache_hits_total",
        "Total number of cache hits"
    ).unwrap();

    static ref CACHE_MISSES: IntCounter = register_int_counter!(
        "cache_misses_total",
        "Total number of cache misses"
    ).unwrap();

    static ref CACHE_LATENCY: Histogram = register_histogram!(
        "cache_operation_duration_seconds",
        "Cache operation latency"
    ).unwrap();
}

/// Main cache manager
pub struct CacheManager {
    redis_pool: RedisPool,
    default_ttl: Duration,
}

impl CacheManager {
    /// Create new cache manager
    pub async fn new(redis_url: &str) -> Result<Self, CacheError> {
        let manager = RedisConnectionManager::new(redis_url)?;
        let pool = bb8::Pool::builder()
            .max_size(50)
            .min_idle(Some(10))
            .build(manager)
            .await?;

        Ok(Self {
            redis_pool: pool,
            default_ttl: Duration::from_secs(300), // 5 minutes default
        })
    }

    /// Get from cache
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, CacheError> {
        let timer = CACHE_LATENCY.start_timer();
        let mut conn = self.redis_pool.get().await?;

        let result: Option<String> = conn.get(key).await?;
        timer.observe_duration();

        match result {
            Some(data) => {
                CACHE_HITS.inc();
                debug!("Cache hit for key: {}", key);
                Ok(Some(serde_json::from_str(&data)?))
            }
            None => {
                CACHE_MISSES.inc();
                debug!("Cache miss for key: {}", key);
                Ok(None)
            }
        }
    }

    /// Set in cache with TTL
    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<(), CacheError> {
        let timer = CACHE_LATENCY.start_timer();
        let mut conn = self.redis_pool.get().await?;

        let ttl = ttl.unwrap_or(self.default_ttl);
        let data = serde_json::to_string(value)?;

        conn.set_ex(key, data, ttl.as_secs() as u64).await?;
        timer.observe_duration();

        debug!("Cached key: {} with TTL: {:?}", key, ttl);
        Ok(())
    }

    /// Delete from cache
    pub async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut conn = self.redis_pool.get().await?;
        conn.del(key).await?;
        debug!("Deleted cache key: {}", key);
        Ok(())
    }

    /// Delete multiple keys by pattern
    pub async fn delete_pattern(&self, pattern: &str) -> Result<u64, CacheError> {
        let mut conn = self.redis_pool.get().await?;
        let keys: Vec<String> = conn.keys(pattern).await?;

        if keys.is_empty() {
            return Ok(0);
        }

        let count = keys.len() as u64;
        conn.del(keys).await?;
        debug!("Deleted {} keys matching pattern: {}", count, pattern);
        Ok(count)
    }

    /// Invalidate user cache
    pub async fn invalidate_user(&self, user_id: Uuid) -> Result<(), CacheError> {
        let pattern = format!("user:{}:*", user_id);
        self.delete_pattern(&pattern).await?;
        Ok(())
    }

    /// Invalidate server cache
    pub async fn invalidate_server(&self, server_ip: &str) -> Result<(), CacheError> {
        let pattern = format!("server:{}:*", server_ip);
        self.delete_pattern(&pattern).await?;
        Ok(())
    }
}

/// Cache keys builder
pub struct CacheKeys;

impl CacheKeys {
    /// User profile key
    pub fn user_profile(user_id: Uuid) -> String {
        format!("user:{}:profile", user_id)
    }

    /// User stats key
    pub fn user_stats(user_id: Uuid) -> String {
        format!("user:{}:stats", user_id)
    }

    /// User processes key
    pub fn user_processes(user_id: Uuid) -> String {
        format!("user:{}:processes", user_id)
    }

    /// Server info key
    pub fn server_info(ip: &str) -> String {
        format!("server:{}:info", ip)
    }

    /// Server files key
    pub fn server_files(ip: &str) -> String {
        format!("server:{}:files", ip)
    }

    /// Leaderboard key
    pub fn leaderboard(board_type: &str) -> String {
        format!("leaderboard:{}", board_type)
    }

    /// Clan info key
    pub fn clan_info(clan_id: Uuid) -> String {
        format!("clan:{}:info", clan_id)
    }

    /// PvP match key
    pub fn pvp_match(match_id: Uuid) -> String {
        format!("pvp:match:{}", match_id)
    }

    /// Market listings key
    pub fn market_listings(category: &str) -> String {
        format!("market:listings:{}", category)
    }

    /// Chat room messages key
    pub fn chat_messages(room_id: &str) -> String {
        format!("chat:{}:messages", room_id)
    }
}

/// Cached data wrapper with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedData<T> {
    pub data: T,
    pub cached_at: chrono::DateTime<chrono::Utc>,
    pub ttl: u64,
}

impl<T> CachedData<T> {
    pub fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            cached_at: chrono::Utc::now(),
            ttl: ttl.as_secs(),
        }
    }

    pub fn is_expired(&self) -> bool {
        let age = chrono::Utc::now() - self.cached_at;
        age.num_seconds() as u64 > self.ttl
    }
}

/// Cache warming service
pub struct CacheWarmer {
    cache: CacheManager,
}

impl CacheWarmer {
    pub fn new(cache: CacheManager) -> Self {
        Self { cache }
    }

    /// Warm up frequently accessed data
    pub async fn warm_up(&self) -> Result<(), CacheError> {
        // Warm leaderboards
        self.warm_leaderboards().await?;

        // Warm popular servers
        self.warm_popular_servers().await?;

        // Warm active clan data
        self.warm_active_clans().await?;

        Ok(())
    }

    async fn warm_leaderboards(&self) -> Result<(), CacheError> {
        debug!("Warming leaderboard cache...");
        // Implementation would fetch from database and cache
        Ok(())
    }

    async fn warm_popular_servers(&self) -> Result<(), CacheError> {
        debug!("Warming popular servers cache...");
        // Implementation would fetch from database and cache
        Ok(())
    }

    async fn warm_active_clans(&self) -> Result<(), CacheError> {
        debug!("Warming active clans cache...");
        // Implementation would fetch from database and cache
        Ok(())
    }
}

/// Cache invalidation rules
pub struct CacheInvalidator {
    cache: CacheManager,
}

impl CacheInvalidator {
    pub fn new(cache: CacheManager) -> Self {
        Self { cache }
    }

    /// Invalidate on user level up
    pub async fn on_level_up(&self, user_id: Uuid) -> Result<(), CacheError> {
        self.cache.delete(&CacheKeys::user_profile(user_id)).await?;
        self.cache.delete(&CacheKeys::user_stats(user_id)).await?;
        self.cache.delete(&CacheKeys::leaderboard("level")).await?;
        Ok(())
    }

    /// Invalidate on hack complete
    pub async fn on_hack_complete(&self, user_id: Uuid, server_ip: &str) -> Result<(), CacheError> {
        self.cache.delete(&CacheKeys::user_stats(user_id)).await?;
        self.cache.delete(&CacheKeys::server_info(server_ip)).await?;
        Ok(())
    }

    /// Invalidate on PvP match end
    pub async fn on_pvp_end(&self, match_id: Uuid, winner_id: Uuid, loser_id: Uuid) -> Result<(), CacheError> {
        self.cache.delete(&CacheKeys::pvp_match(match_id)).await?;
        self.cache.delete(&CacheKeys::user_stats(winner_id)).await?;
        self.cache.delete(&CacheKeys::user_stats(loser_id)).await?;
        self.cache.delete(&CacheKeys::leaderboard("pvp")).await?;
        Ok(())
    }

    /// Invalidate on clan change
    pub async fn on_clan_change(&self, clan_id: Uuid, user_id: Uuid) -> Result<(), CacheError> {
        self.cache.delete(&CacheKeys::clan_info(clan_id)).await?;
        self.cache.delete(&CacheKeys::user_profile(user_id)).await?;
        Ok(())
    }
}

/// Cache error types
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),

    #[error("Pool error: {0}")]
    Pool(#[from] bb8::RunError<RedisError>),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Cache key not found")]
    KeyNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_keys() {
        let user_id = Uuid::new_v4();
        let key = CacheKeys::user_profile(user_id);
        assert!(key.starts_with("user:"));
        assert!(key.ends_with(":profile"));
    }

    #[test]
    fn test_cached_data() {
        let data = CachedData::new("test", Duration::from_secs(60));
        assert!(!data.is_expired());
        assert_eq!(data.ttl, 60);
    }
}