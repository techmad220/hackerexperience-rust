use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// L1 Cache implementation for hot data
pub struct L1Cache {
    cache: Cache<String, Vec<u8>>,
    stats: Arc<RwLock<CacheStats>>,
    config: L1CacheConfig,
}

#[derive(Debug, Clone)]
pub struct L1CacheConfig {
    pub max_capacity: u64,
    pub time_to_live: Duration,
    pub time_to_idle: Duration,
    pub enable_stats: bool,
}

impl Default for L1CacheConfig {
    fn default() -> Self {
        Self {
            max_capacity: 10_000,
            time_to_live: Duration::from_secs(30),
            time_to_idle: Duration::from_secs(10),
            enable_stats: true,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub size: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64) / (total as f64)
        }
    }
}

impl L1Cache {
    pub fn new(config: L1CacheConfig) -> Self {
        let cache = Cache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(config.time_to_live)
            .time_to_idle(config.time_to_idle)
            .build();

        Self {
            cache,
            stats: Arc::new(RwLock::new(CacheStats::default())),
            config,
        }
    }

    pub async fn get<T>(&self, key: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        match self.cache.get(key).await {
            Some(data) => {
                if self.config.enable_stats {
                    let mut stats = self.stats.write().await;
                    stats.hits += 1;
                }
                bincode::deserialize(&data).ok()
            }
            None => {
                if self.config.enable_stats {
                    let mut stats = self.stats.write().await;
                    stats.misses += 1;
                }
                None
            }
        }
    }

    pub async fn set<T>(&self, key: String, value: &T) -> Result<(), bincode::Error>
    where
        T: Serialize,
    {
        let data = bincode::serialize(value)?;
        self.cache.insert(key, data).await;

        if self.config.enable_stats {
            let mut stats = self.stats.write().await;
            stats.size = self.cache.weighted_size();
        }

        Ok(())
    }

    pub async fn invalidate(&self, key: &str) {
        self.cache.invalidate(key).await;

        if self.config.enable_stats {
            let mut stats = self.stats.write().await;
            stats.evictions += 1;
            stats.size = self.cache.weighted_size();
        }
    }

    pub async fn invalidate_all(&self) {
        self.cache.invalidate_all().await;

        if self.config.enable_stats {
            let mut stats = self.stats.write().await;
            stats.evictions += stats.size;
            stats.size = 0;
        }
    }

    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// Get entry count
    pub async fn entry_count(&self) -> u64 {
        self.cache.entry_count()
    }

    /// Get weighted size
    pub fn weighted_size(&self) -> u64 {
        self.cache.weighted_size()
    }
}

/// Multi-tier cache with L1 (memory) and L2 (Redis) layers
pub struct MultiTierCache {
    l1: L1Cache,
    l2: Option<Arc<dyn L2CacheProvider>>,
}

#[async_trait::async_trait]
pub trait L2CacheProvider: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, anyhow::Error>;
    async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<Duration>) -> Result<(), anyhow::Error>;
    async fn delete(&self, key: &str) -> Result<(), anyhow::Error>;
}

impl MultiTierCache {
    pub fn new(l1_config: L1CacheConfig, l2_provider: Option<Arc<dyn L2CacheProvider>>) -> Self {
        Self {
            l1: L1Cache::new(l1_config),
            l2: l2_provider,
        }
    }

    pub async fn get<T>(&self, key: &str) -> Result<Option<T>, anyhow::Error>
    where
        T: for<'de> Deserialize<'de> + Serialize,
    {
        // Try L1 cache first
        if let Some(value) = self.l1.get::<T>(key).await {
            return Ok(Some(value));
        }

        // Try L2 cache if available
        if let Some(l2) = &self.l2 {
            if let Some(data) = l2.get(key).await? {
                if let Ok(value) = bincode::deserialize::<T>(&data) {
                    // Populate L1 cache
                    self.l1.set(key.to_string(), &value).await.ok();
                    return Ok(Some(value));
                }
            }
        }

        Ok(None)
    }

    pub async fn set<T>(&self, key: String, value: &T, ttl: Option<Duration>) -> Result<(), anyhow::Error>
    where
        T: Serialize,
    {
        // Set in L1 cache
        self.l1.set(key.clone(), value).await?;

        // Set in L2 cache if available
        if let Some(l2) = &self.l2 {
            let data = bincode::serialize(value)?;
            l2.set(&key, data, ttl).await?;
        }

        Ok(())
    }

    pub async fn invalidate(&self, key: &str) -> Result<(), anyhow::Error> {
        // Invalidate L1
        self.l1.invalidate(key).await;

        // Invalidate L2 if available
        if let Some(l2) = &self.l2 {
            l2.delete(key).await?;
        }

        Ok(())
    }

    pub async fn get_l1_stats(&self) -> CacheStats {
        self.l1.get_stats().await
    }
}

/// Cache prewarming for frequently accessed data
pub struct CachePrewarmer {
    cache: Arc<MultiTierCache>,
}

impl CachePrewarmer {
    pub fn new(cache: Arc<MultiTierCache>) -> Self {
        Self { cache }
    }

    pub async fn prewarm_critical_data(&self) -> Result<(), anyhow::Error> {
        // Prewarm leaderboard data
        self.prewarm_leaderboard().await?;

        // Prewarm online player count
        self.prewarm_online_count().await?;

        // Prewarm server statistics
        self.prewarm_server_stats().await?;

        Ok(())
    }

    async fn prewarm_leaderboard(&self) -> Result<(), anyhow::Error> {
        // This would fetch from database in real implementation
        let leaderboard_data = vec![
            ("player1", 1000),
            ("player2", 950),
            ("player3", 900),
        ];

        self.cache
            .set("leaderboard:top:10".to_string(), &leaderboard_data, Some(Duration::from_secs(300)))
            .await?;

        Ok(())
    }

    async fn prewarm_online_count(&self) -> Result<(), anyhow::Error> {
        // This would fetch from database in real implementation
        let online_count = 1337;

        self.cache
            .set("stats:online_count".to_string(), &online_count, Some(Duration::from_secs(60)))
            .await?;

        Ok(())
    }

    async fn prewarm_server_stats(&self) -> Result<(), anyhow::Error> {
        // This would fetch from database in real implementation
        let stats = ServerStats {
            total_users: 50000,
            active_processes: 234,
            cpu_usage: 45.2,
            memory_usage: 67.8,
        };

        self.cache
            .set("stats:server".to_string(), &stats, Some(Duration::from_secs(30)))
            .await?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerStats {
    total_users: u64,
    active_processes: u64,
    cpu_usage: f64,
    memory_usage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_l1_cache_basic() {
        let cache = L1Cache::new(L1CacheConfig::default());

        // Test set and get
        cache.set("test_key".to_string(), &"test_value").await.unwrap();
        let value: Option<String> = cache.get("test_key").await;
        assert_eq!(value, Some("test_value".to_string()));

        // Test stats
        let stats = cache.get_stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let cache = L1Cache::new(L1CacheConfig::default());

        cache.set("key1".to_string(), &"value1").await.unwrap();
        cache.set("key2".to_string(), &"value2").await.unwrap();

        cache.invalidate("key1").await;

        let value1: Option<String> = cache.get("key1").await;
        let value2: Option<String> = cache.get("key2").await;

        assert_eq!(value1, None);
        assert_eq!(value2, Some("value2".to_string()));
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = L1Cache::new(L1CacheConfig::default());

        // Generate some hits and misses
        let _: Option<String> = cache.get("missing_key").await;
        cache.set("existing_key".to_string(), &"value").await.unwrap();
        let _: Option<String> = cache.get("existing_key").await;
        let _: Option<String> = cache.get("another_missing").await;

        let stats = cache.get_stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 2);
        assert_eq!(stats.hit_rate(), 1.0 / 3.0);
    }
}