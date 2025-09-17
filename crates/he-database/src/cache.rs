//! Advanced caching strategies for improved performance

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, instrument};

/// Multi-layer cache with TTL and LRU eviction
pub struct CacheManager {
    /// L1 cache - hot data, small, fast
    l1_cache: Arc<RwLock<LruCache<String, CachedValue>>>,
    /// L2 cache - warm data, larger, slightly slower
    l2_cache: Arc<RwLock<LruCache<String, CachedValue>>>,
    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,
}

impl CacheManager {
    pub fn new(l1_size: usize, l2_size: usize) -> Self {
        Self {
            l1_cache: Arc::new(RwLock::new(LruCache::new(l1_size))),
            l2_cache: Arc::new(RwLock::new(LruCache::new(l2_size))),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Get value from cache with automatic promotion
    #[instrument(skip(self))]
    pub async fn get<T: Clone + DeserializeOwned>(&self, key: &str) -> Option<T> {
        let mut stats = self.stats.write().await;

        // Check L1 cache first
        {
            let mut l1 = self.l1_cache.write().await;
            if let Some(value) = l1.get(key) {
                if !value.is_expired() {
                    stats.l1_hits += 1;
                    debug!("L1 cache hit for key: {}", key);
                    return value.deserialize();
                }
                l1.remove(key);
            }
        }

        // Check L2 cache
        {
            let mut l2 = self.l2_cache.write().await;
            if let Some(value) = l2.get(key) {
                if !value.is_expired() {
                    stats.l2_hits += 1;
                    debug!("L2 cache hit for key: {}, promoting to L1", key);

                    // Promote to L1
                    let mut l1 = self.l1_cache.write().await;
                    l1.put(key.to_string(), value.clone());

                    return value.deserialize();
                }
                l2.remove(key);
            }
        }

        stats.misses += 1;
        None
    }

    /// Set value in cache with TTL
    #[instrument(skip(self, value))]
    pub async fn set<T: Serialize>(&self, key: String, value: T, ttl: Duration) {
        let cached_value = CachedValue::new(value, ttl);

        let mut l1 = self.l1_cache.write().await;

        // If L1 is full, demote oldest to L2
        if l1.is_full() {
            if let Some((old_key, old_value)) = l1.pop_oldest() {
                let mut l2 = self.l2_cache.write().await;
                l2.put(old_key, old_value);
            }
        }

        l1.put(key, cached_value);
    }

    /// Invalidate specific cache key
    pub async fn invalidate(&self, key: &str) {
        let mut l1 = self.l1_cache.write().await;
        let mut l2 = self.l2_cache.write().await;

        l1.remove(key);
        l2.remove(key);
    }

    /// Invalidate cache keys matching pattern
    pub async fn invalidate_pattern(&self, pattern: &str) {
        let mut l1 = self.l1_cache.write().await;
        let mut l2 = self.l2_cache.write().await;

        l1.remove_matching(pattern);
        l2.remove_matching(pattern);
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// Clear all caches
    pub async fn clear_all(&self) {
        let mut l1 = self.l1_cache.write().await;
        let mut l2 = self.l2_cache.write().await;

        l1.clear();
        l2.clear();
    }
}

/// LRU cache implementation
struct LruCache<K: Clone + Eq + std::hash::Hash, V: Clone> {
    capacity: usize,
    cache: HashMap<K, (V, usize)>,
    access_order: Vec<K>,
    access_counter: usize,
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> LruCache<K, V> {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: HashMap::with_capacity(capacity),
            access_order: Vec::with_capacity(capacity),
            access_counter: 0,
        }
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        if let Some((value, last_access)) = self.cache.get_mut(key) {
            self.access_counter += 1;
            *last_access = self.access_counter;

            // Move to end of access order
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                let key_clone = self.access_order.remove(pos);
                self.access_order.push(key_clone);
            }

            Some(value)
        } else {
            None
        }
    }

    fn put(&mut self, key: K, value: V) {
        if self.cache.len() >= self.capacity && !self.cache.contains_key(&key) {
            // Evict least recently used
            if let Some(lru_key) = self.access_order.first().cloned() {
                self.cache.remove(&lru_key);
                self.access_order.remove(0);
            }
        }

        self.access_counter += 1;
        self.cache.insert(key.clone(), (value, self.access_counter));

        // Update access order
        if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
            self.access_order.remove(pos);
        }
        self.access_order.push(key);
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        if let Some((value, _)) = self.cache.remove(key) {
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.remove(pos);
            }
            Some(value)
        } else {
            None
        }
    }

    fn is_full(&self) -> bool {
        self.cache.len() >= self.capacity
    }

    fn pop_oldest(&mut self) -> Option<(K, V)> {
        if let Some(key) = self.access_order.first().cloned() {
            if let Some((value, _)) = self.cache.remove(&key) {
                self.access_order.remove(0);
                return Some((key, value));
            }
        }
        None
    }

    fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
        self.access_counter = 0;
    }
}

impl LruCache<String, CachedValue> {
    fn remove_matching(&mut self, pattern: &str) {
        let keys_to_remove: Vec<String> = self.cache.keys()
            .filter(|k| k.contains(pattern))
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.remove(&key);
        }
    }
}

/// Cached value with expiration
#[derive(Clone, Debug)]
struct CachedValue {
    data: Vec<u8>,
    expires_at: Instant,
}

impl CachedValue {
    fn new<T: Serialize>(value: T, ttl: Duration) -> Self {
        Self {
            data: bincode::serialize(&value).unwrap_or_default(),
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }

    fn deserialize<T: DeserializeOwned>(&self) -> Option<T> {
        bincode::deserialize(&self.data).ok()
    }
}

/// Cache statistics
#[derive(Clone, Debug, Default)]
pub struct CacheStats {
    pub l1_hits: u64,
    pub l2_hits: u64,
    pub misses: u64,
    pub evictions: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.l1_hits + self.l2_hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.l1_hits + self.l2_hits) as f64 / total as f64
        }
    }

    pub fn l1_hit_rate(&self) -> f64 {
        let total = self.l1_hits + self.l2_hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.l1_hits as f64 / total as f64
        }
    }
}

/// Cache key builders for consistent key generation
pub mod cache_keys {
    pub fn user_key(user_id: i64) -> String {
        format!("user:{}", user_id)
    }

    pub fn user_stats_key(user_id: i64) -> String {
        format!("user:stats:{}", user_id)
    }

    pub fn process_key(process_id: i64) -> String {
        format!("process:{}", process_id)
    }

    pub fn server_key(server_id: &str) -> String {
        format!("server:{}", server_id)
    }

    pub fn leaderboard_key(page: i32) -> String {
        format!("leaderboard:page:{}", page)
    }

    pub fn clan_key(clan_id: i64) -> String {
        format!("clan:{}", clan_id)
    }

    pub fn session_key(session_id: &str) -> String {
        format!("session:{}", session_id)
    }
}

/// Cache warming strategies
pub struct CacheWarmer {
    cache: Arc<CacheManager>,
}

impl CacheWarmer {
    pub fn new(cache: Arc<CacheManager>) -> Self {
        Self { cache }
    }

    /// Pre-load frequently accessed data
    #[instrument(skip(self))]
    pub async fn warm_cache(&self) {
        debug!("Starting cache warming");

        // Warm up common data like:
        // - Active user sessions
        // - Popular server information
        // - Leaderboard top pages
        // - Active process lists

        // This would be implemented based on actual usage patterns
    }

    /// Periodic cache refresh for critical data
    pub async fn refresh_critical_cache(&self) {
        // Refresh data that should always be cached
        // - System configuration
        // - Popular game content
        // - Active event data
    }
}

// Re-export common trait bounds
use serde::de::DeserializeOwned;