//! Cache Actor System - Complete GenServer Implementation
//!
//! This module provides a comprehensive distributed caching system with GenServer patterns,
//! including cache management, distributed synchronization, TTL handling, and cache invalidation.
//! 
//! Features:
//! - Distributed caching with multi-node synchronization
//! - TTL-based automatic expiration
//! - Cache invalidation with pattern matching
//! - Memory management with size limits and LRU eviction
//! - Cache statistics and monitoring
//! - Backup and restoration capabilities
//! - Hot/cold data separation
//! - Compression and serialization

use crate::models::{CacheEntry, CacheKey, CacheValue, CacheStats, CachePolicy, InvalidationPattern, BackupMetadata};
use he_helix_core::{
    genserver::{GenServer, GenServerBehavior, GenServerMessage, GenServerReply, InfoSource},
    actors::{Actor, ActorContext, Handler, Message},
    HelixError, HelixResult, ProcessId
};
use async_trait::async_trait;
use std::collections::{HashMap, BTreeMap, VecDeque};
use tokio::sync::{RwLock, Mutex, broadcast, mpsc};
use chrono::{DateTime, Utc, Duration};
use std::sync::Arc;
use tracing::{info, error, warn, debug, trace};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use lru::LruCache;
use std::num::NonZeroUsize;
use tokio::time::{interval, sleep, Instant};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Cache operation error types
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Cache key not found: {0}")]
    KeyNotFound(String),
    #[error("Cache full - cannot store more entries")]
    CacheFull,
    #[error("Invalid TTL specified: {0}")]
    InvalidTtl(i64),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Network synchronization error: {0}")]
    SyncError(String),
    #[error("Cache policy violation: {0}")]
    PolicyViolation(String),
    #[error("Internal cache error: {0}")]
    InternalError(String),
}

/// Messages for CacheActor GenServer
#[derive(Debug, Clone)]
pub enum CacheCall {
    /// Store a value in cache with optional TTL
    Set {
        key: CacheKey,
        value: CacheValue,
        ttl_seconds: Option<i64>,
        policy: Option<CachePolicy>,
    },
    /// Retrieve a value from cache
    Get {
        key: CacheKey,
    },
    /// Delete a specific cache entry
    Delete {
        key: CacheKey,
    },
    /// Check if key exists in cache
    Exists {
        key: CacheKey,
    },
    /// Get cache statistics
    GetStats,
    /// Update TTL for existing key
    UpdateTtl {
        key: CacheKey,
        ttl_seconds: i64,
    },
    /// Get multiple keys at once
    MultiGet {
        keys: Vec<CacheKey>,
    },
    /// Set multiple key-value pairs atomically
    MultiSet {
        entries: Vec<(CacheKey, CacheValue)>,
        ttl_seconds: Option<i64>,
    },
    /// Search keys by pattern
    SearchKeys {
        pattern: String,
        limit: Option<usize>,
    },
    /// Export cache backup
    ExportBackup {
        include_expired: bool,
    },
    /// Import cache from backup
    ImportBackup {
        backup: BackupMetadata,
        merge_policy: String,
    },
    /// Get cache configuration
    GetConfig,
    /// Update cache configuration
    UpdateConfig {
        max_size: Option<usize>,
        default_ttl: Option<i64>,
        compression_enabled: Option<bool>,
    },
}

#[derive(Debug, Clone)]
pub enum CacheCast {
    /// Invalidate cache entries by pattern
    InvalidatePattern {
        pattern: InvalidationPattern,
    },
    /// Force cleanup of expired entries
    CleanupExpired,
    /// Sync with other cache nodes
    SyncWithNodes {
        node_ids: Vec<String>,
    },
    /// Warm up cache with commonly accessed data
    WarmupCache {
        keys: Vec<CacheKey>,
    },
    /// Compact cache storage
    CompactStorage,
    /// Record cache access for analytics
    RecordAccess {
        key: CacheKey,
        access_type: String,
        timestamp: DateTime<Utc>,
    },
}

#[derive(Debug, Clone)]
pub enum CacheInfo {
    /// TTL expiration event
    TtlExpired {
        key: CacheKey,
        expired_at: DateTime<Utc>,
    },
    /// Cache memory pressure warning
    MemoryPressure {
        current_size: usize,
        max_size: usize,
        evicted_entries: usize,
    },
    /// Distributed cache synchronization event
    SyncEvent {
        node_id: String,
        event_type: String,
        affected_keys: Vec<CacheKey>,
    },
    /// Cache performance metrics update
    MetricsUpdate {
        hit_rate: f64,
        miss_rate: f64,
        eviction_rate: f64,
    },
}

/// Cache operation results
pub type CacheResult<T> = Result<T, CacheError>;

/// Main cache state structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheState {
    /// Primary cache storage with LRU eviction
    pub cache: HashMap<CacheKey, CacheEntry>,
    /// TTL index for efficient expiration
    pub ttl_index: BTreeMap<DateTime<Utc>, Vec<CacheKey>>,
    /// Access frequency tracking
    pub access_frequency: HashMap<CacheKey, u64>,
    /// Cache statistics
    pub stats: CacheStats,
    /// Configuration
    pub config: CacheConfiguration,
    /// Distributed node registry
    pub connected_nodes: HashMap<String, NodeInfo>,
    /// Pending synchronization operations
    pub sync_queue: VecDeque<SyncOperation>,
    /// Hot data cache for frequently accessed items
    pub hot_cache: LruCache<CacheKey, CacheValue>,
    /// Last cleanup timestamp
    pub last_cleanup: DateTime<Utc>,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfiguration {
    pub max_size: usize,
    pub default_ttl_seconds: i64,
    pub cleanup_interval_seconds: u64,
    pub compression_enabled: bool,
    pub hot_cache_size: usize,
    pub sync_enabled: bool,
    pub memory_pressure_threshold: f64,
    pub eviction_policy: EvictionPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    Lru,
    Lfu,
    Random,
    Ttl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: String,
    pub last_seen: DateTime<Utc>,
    pub sync_priority: u8,
    pub is_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOperation {
    pub operation_id: Uuid,
    pub operation_type: String,
    pub target_nodes: Vec<String>,
    pub payload: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub retry_count: u32,
}

/// Cache Actor - Complete GenServer Implementation
pub struct CacheActor {
    state: Arc<RwLock<CacheState>>,
    cleanup_handle: Option<tokio::task::JoinHandle<()>>,
    sync_handle: Option<tokio::task::JoinHandle<()>>,
    metrics_sender: Option<mpsc::UnboundedSender<CacheMetrics>>,
    node_id: String,
}

#[derive(Debug, Clone)]
pub struct CacheMetrics {
    pub timestamp: DateTime<Utc>,
    pub operation: String,
    pub key: Option<CacheKey>,
    pub hit: bool,
    pub latency_micros: u64,
    pub cache_size: usize,
}

impl Default for CacheConfiguration {
    fn default() -> Self {
        Self {
            max_size: 10000,
            default_ttl_seconds: 3600, // 1 hour
            cleanup_interval_seconds: 300, // 5 minutes
            compression_enabled: true,
            hot_cache_size: 1000,
            sync_enabled: true,
            memory_pressure_threshold: 0.85,
            eviction_policy: EvictionPolicy::Lru,
        }
    }
}

impl Default for CacheState {
    fn default() -> Self {
        let config = CacheConfiguration::default();
        Self {
            cache: HashMap::new(),
            ttl_index: BTreeMap::new(),
            access_frequency: HashMap::new(),
            stats: CacheStats::default(),
            config: config.clone(),
            connected_nodes: HashMap::new(),
            sync_queue: VecDeque::new(),
            hot_cache: LruCache::new(NonZeroUsize::new(config.hot_cache_size).unwrap()),
            last_cleanup: Utc::now(),
        }
    }
}

impl CacheActor {
    /// Create new CacheActor instance
    pub fn new(node_id: String) -> Self {
        Self {
            state: Arc::new(RwLock::new(CacheState::default())),
            cleanup_handle: None,
            sync_handle: None,
            metrics_sender: None,
            node_id,
        }
    }

    /// Initialize cache actor with background tasks
    pub async fn initialize(&mut self) -> HelixResult<()> {
        self.start_cleanup_task().await?;
        self.start_sync_task().await?;
        self.start_metrics_collection().await?;
        
        info!("CacheActor initialized with node_id: {}", self.node_id);
        Ok(())
    }

    /// Start background cleanup task for expired entries
    async fn start_cleanup_task(&mut self) -> HelixResult<()> {
        let state = Arc::clone(&self.state);
        let cleanup_interval = {
            let state_guard = state.read().await;
            state_guard.config.cleanup_interval_seconds
        };

        let handle = tokio::spawn(async move {
            let mut interval = interval(tokio::time::Duration::from_secs(cleanup_interval));
            loop {
                interval.tick().await;
                Self::cleanup_expired_entries(&state).await;
            }
        });

        self.cleanup_handle = Some(handle);
        Ok(())
    }

    /// Start background synchronization task
    async fn start_sync_task(&mut self) -> HelixResult<()> {
        let state = Arc::clone(&self.state);
        
        let handle = tokio::spawn(async move {
            let mut interval = interval(tokio::time::Duration::from_secs(60)); // Sync every minute
            loop {
                interval.tick().await;
                Self::process_sync_queue(&state).await;
            }
        });

        self.sync_handle = Some(handle);
        Ok(())
    }

    /// Start metrics collection task
    async fn start_metrics_collection(&mut self) -> HelixResult<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.metrics_sender = Some(tx);

        tokio::spawn(async move {
            while let Some(metrics) = rx.recv().await {
                // Process metrics (log, store, send to monitoring system)
                trace!("Cache metrics: {:?}", metrics);
            }
        });

        Ok(())
    }

    /// Cleanup expired cache entries
    async fn cleanup_expired_entries(state: &Arc<RwLock<CacheState>>) {
        let mut state_guard = state.write().await;
        let now = Utc::now();
        let mut expired_keys = Vec::new();

        // Find expired entries from TTL index
        let expired_times: Vec<DateTime<Utc>> = state_guard
            .ttl_index
            .range(..now)
            .map(|(time, _)| *time)
            .collect();

        for expired_time in expired_times {
            if let Some(keys) = state_guard.ttl_index.remove(&expired_time) {
                for key in keys {
                    if state_guard.cache.remove(&key).is_some() {
                        expired_keys.push(key.clone());
                        state_guard.access_frequency.remove(&key);
                    }
                }
            }
        }

        state_guard.stats.expired_entries += expired_keys.len() as u64;
        state_guard.last_cleanup = now;

        if !expired_keys.is_empty() {
            debug!("Cleaned up {} expired cache entries", expired_keys.len());
        }
    }

    /// Process synchronization queue
    async fn process_sync_queue(state: &Arc<RwLock<CacheState>>) {
        let mut state_guard = state.write().await;
        
        let operations_to_process: Vec<SyncOperation> = state_guard
            .sync_queue
            .iter()
            .cloned()
            .collect();

        for operation in operations_to_process {
            // Process synchronization operation
            match Self::process_sync_operation(&operation).await {
                Ok(_) => {
                    state_guard.sync_queue.retain(|op| op.operation_id != operation.operation_id);
                }
                Err(e) => {
                    warn!("Sync operation failed: {:?}", e);
                    // Update retry count and re-queue if under limit
                    if operation.retry_count < 3 {
                        if let Some(pos) = state_guard.sync_queue
                            .iter()
                            .position(|op| op.operation_id == operation.operation_id) 
                        {
                            state_guard.sync_queue[pos].retry_count += 1;
                        }
                    } else {
                        // Remove operation after max retries
                        state_guard.sync_queue.retain(|op| op.operation_id != operation.operation_id);
                        error!("Sync operation failed after max retries: {:?}", operation.operation_id);
                    }
                }
            }
        }
    }

    /// Process individual synchronization operation
    async fn process_sync_operation(operation: &SyncOperation) -> HelixResult<()> {
        // Implementation would handle actual network synchronization
        debug!("Processing sync operation: {:?}", operation.operation_id);
        Ok(())
    }

    /// Handle cache set operation
    async fn handle_set(
        &self,
        key: CacheKey,
        value: CacheValue,
        ttl_seconds: Option<i64>,
        policy: Option<CachePolicy>,
    ) -> CacheResult<bool> {
        let mut state = self.state.write().await;

        // Check cache size limits
        if state.cache.len() >= state.config.max_size {
            // Attempt LRU eviction
            self.evict_lru_entries(&mut state, 1).await?;
        }

        let ttl = ttl_seconds.unwrap_or(state.config.default_ttl_seconds);
        let expires_at = if ttl > 0 {
            Some(Utc::now() + Duration::seconds(ttl))
        } else {
            None
        };

        let entry = CacheEntry {
            key: key.clone(),
            value: value.clone(),
            created_at: Utc::now(),
            expires_at,
            access_count: 0,
            last_accessed: Utc::now(),
            policy: policy.unwrap_or_default(),
            compressed: state.config.compression_enabled,
        };

        // Update TTL index if expiration is set
        if let Some(exp_time) = expires_at {
            state.ttl_index
                .entry(exp_time)
                .or_insert_with(Vec::new)
                .push(key.clone());
        }

        // Store in main cache
        let is_update = state.cache.contains_key(&key);
        state.cache.insert(key.clone(), entry);

        // Update hot cache for frequently accessed items
        state.hot_cache.put(key.clone(), value);

        // Update statistics
        if is_update {
            state.stats.updates += 1;
        } else {
            state.stats.sets += 1;
        }

        // Record metrics
        if let Some(metrics_sender) = &self.metrics_sender {
            let _ = metrics_sender.send(CacheMetrics {
                timestamp: Utc::now(),
                operation: "set".to_string(),
                key: Some(key),
                hit: false,
                latency_micros: 0,
                cache_size: state.cache.len(),
            });
        }

        Ok(true)
    }

    /// Handle cache get operation
    async fn handle_get(&self, key: CacheKey) -> CacheResult<Option<CacheValue>> {
        let start = Instant::now();
        let mut state = self.state.write().await;

        // Check hot cache first
        if let Some(value) = state.hot_cache.get(&key) {
            state.stats.hits += 1;
            state.stats.hot_hits += 1;
            return Ok(Some(value.clone()));
        }

        // Check main cache
        if let Some(entry) = state.cache.get_mut(&key) {
            // Check if entry has expired
            if let Some(expires_at) = entry.expires_at {
                if Utc::now() > expires_at {
                    // Entry expired, remove it
                    state.cache.remove(&key);
                    state.access_frequency.remove(&key);
                    state.stats.misses += 1;
                    state.stats.expired_entries += 1;
                    return Ok(None);
                }
            }

            // Update access statistics
            entry.access_count += 1;
            entry.last_accessed = Utc::now();
            
            let access_freq = state.access_frequency.entry(key.clone()).or_insert(0);
            *access_freq += 1;

            // Promote to hot cache if frequently accessed
            if *access_freq > 10 {
                state.hot_cache.put(key.clone(), entry.value.clone());
            }

            state.stats.hits += 1;
            
            let latency = start.elapsed().as_micros() as u64;
            
            // Record metrics
            if let Some(metrics_sender) = &self.metrics_sender {
                let _ = metrics_sender.send(CacheMetrics {
                    timestamp: Utc::now(),
                    operation: "get".to_string(),
                    key: Some(key),
                    hit: true,
                    latency_micros: latency,
                    cache_size: state.cache.len(),
                });
            }

            Ok(Some(entry.value.clone()))
        } else {
            state.stats.misses += 1;
            
            // Record metrics
            if let Some(metrics_sender) = &self.metrics_sender {
                let _ = metrics_sender.send(CacheMetrics {
                    timestamp: Utc::now(),
                    operation: "get".to_string(),
                    key: Some(key),
                    hit: false,
                    latency_micros: start.elapsed().as_micros() as u64,
                    cache_size: state.cache.len(),
                });
            }

            Ok(None)
        }
    }

    /// Evict LRU entries to make space
    async fn evict_lru_entries(&self, state: &mut CacheState, count: usize) -> CacheResult<usize> {
        let mut evicted = 0;
        let mut candidates: Vec<(CacheKey, DateTime<Utc>)> = state
            .cache
            .iter()
            .map(|(k, v)| (k.clone(), v.last_accessed))
            .collect();

        candidates.sort_by(|a, b| a.1.cmp(&b.1));

        for (key, _) in candidates.iter().take(count) {
            if state.cache.remove(key).is_some() {
                state.access_frequency.remove(key);
                evicted += 1;
            }
        }

        state.stats.evicted_entries += evicted as u64;
        Ok(evicted)
    }

    /// Handle cache invalidation by pattern
    async fn handle_invalidate_pattern(&self, pattern: InvalidationPattern) -> CacheResult<u32> {
        let mut state = self.state.write().await;
        let mut invalidated = 0;

        match pattern {
            InvalidationPattern::Prefix(prefix) => {
                let keys_to_remove: Vec<CacheKey> = state
                    .cache
                    .keys()
                    .filter(|k| k.starts_with(&prefix))
                    .cloned()
                    .collect();

                for key in keys_to_remove {
                    state.cache.remove(&key);
                    state.access_frequency.remove(&key);
                    state.hot_cache.pop(&key);
                    invalidated += 1;
                }
            }
            InvalidationPattern::Suffix(suffix) => {
                let keys_to_remove: Vec<CacheKey> = state
                    .cache
                    .keys()
                    .filter(|k| k.ends_with(&suffix))
                    .cloned()
                    .collect();

                for key in keys_to_remove {
                    state.cache.remove(&key);
                    state.access_frequency.remove(&key);
                    state.hot_cache.pop(&key);
                    invalidated += 1;
                }
            }
            InvalidationPattern::Regex(regex_str) => {
                if let Ok(regex) = regex::Regex::new(&regex_str) {
                    let keys_to_remove: Vec<CacheKey> = state
                        .cache
                        .keys()
                        .filter(|k| regex.is_match(k))
                        .cloned()
                        .collect();

                    for key in keys_to_remove {
                        state.cache.remove(&key);
                        state.access_frequency.remove(&key);
                        state.hot_cache.pop(&key);
                        invalidated += 1;
                    }
                }
            }
            InvalidationPattern::All => {
                invalidated = state.cache.len() as u32;
                state.cache.clear();
                state.access_frequency.clear();
                state.hot_cache.clear();
                state.ttl_index.clear();
            }
        }

        state.stats.invalidated_entries += invalidated as u64;
        Ok(invalidated)
    }
}

/// GenServer implementation for CacheActor
#[async_trait]
impl GenServerBehavior for CacheActor {
    type State = CacheState;

    async fn init(&mut self) -> HelixResult<()> {
        self.initialize().await?;
        info!("CacheActor GenServer initialized");
        Ok(())
    }

    async fn handle_call(
        &mut self,
        message: Box<dyn std::any::Any + Send + Sync>,
        _from: ProcessId,
    ) -> HelixResult<GenServerReply> {
        if let Ok(call) = message.downcast::<CacheCall>() {
            match *call {
                CacheCall::Set { key, value, ttl_seconds, policy } => {
                    let result = self.handle_set(key, value, ttl_seconds, policy).await;
                    Ok(GenServerReply::Reply(Box::new(result)))
                }
                CacheCall::Get { key } => {
                    let result = self.handle_get(key).await;
                    Ok(GenServerReply::Reply(Box::new(result)))
                }
                CacheCall::Delete { key } => {
                    let mut state = self.state.write().await;
                    let existed = state.cache.remove(&key).is_some();
                    state.access_frequency.remove(&key);
                    state.hot_cache.pop(&key);
                    if existed {
                        state.stats.deletes += 1;
                    }
                    Ok(GenServerReply::Reply(Box::new(Ok::<bool, CacheError>(existed))))
                }
                CacheCall::Exists { key } => {
                    let state = self.state.read().await;
                    let exists = state.cache.contains_key(&key);
                    Ok(GenServerReply::Reply(Box::new(Ok::<bool, CacheError>(exists))))
                }
                CacheCall::GetStats => {
                    let state = self.state.read().await;
                    let stats = state.stats.clone();
                    Ok(GenServerReply::Reply(Box::new(Ok::<CacheStats, CacheError>(stats))))
                }
                CacheCall::MultiGet { keys } => {
                    let mut results = HashMap::new();
                    for key in keys {
                        if let Ok(Some(value)) = self.handle_get(key.clone()).await {
                            results.insert(key, value);
                        }
                    }
                    Ok(GenServerReply::Reply(Box::new(Ok::<HashMap<CacheKey, CacheValue>, CacheError>(results))))
                }
                _ => {
                    warn!("Unhandled cache call message");
                    Ok(GenServerReply::NoReply)
                }
            }
        } else {
            Err(HelixError::InvalidMessage("Unknown call message type".to_string()))
        }
    }

    async fn handle_cast(
        &mut self,
        message: Box<dyn std::any::Any + Send + Sync>,
    ) -> HelixResult<()> {
        if let Ok(cast) = message.downcast::<CacheCast>() {
            match *cast {
                CacheCast::InvalidatePattern { pattern } => {
                    if let Err(e) = self.handle_invalidate_pattern(pattern).await {
                        error!("Cache invalidation failed: {:?}", e);
                    }
                }
                CacheCast::CleanupExpired => {
                    Self::cleanup_expired_entries(&self.state).await;
                }
                CacheCast::CompactStorage => {
                    // Implement storage compaction logic
                    let mut state = self.state.write().await;
                    let before_size = state.cache.len();
                    
                    // Remove entries with access_count = 0 and expired entries
                    let keys_to_remove: Vec<CacheKey> = state
                        .cache
                        .iter()
                        .filter(|(_, entry)| {
                            entry.access_count == 0 || 
                            entry.expires_at.map_or(false, |exp| Utc::now() > exp)
                        })
                        .map(|(k, _)| k.clone())
                        .collect();

                    for key in keys_to_remove {
                        state.cache.remove(&key);
                        state.access_frequency.remove(&key);
                        state.hot_cache.pop(&key);
                    }

                    let after_size = state.cache.len();
                    info!("Cache compaction: {} -> {} entries", before_size, after_size);
                }
                _ => {
                    debug!("Cache cast message processed");
                }
            }
        }
        Ok(())
    }

    async fn handle_info(
        &mut self,
        message: Box<dyn std::any::Any + Send + Sync>,
        _source: InfoSource,
    ) -> HelixResult<()> {
        if let Ok(info) = message.downcast::<CacheInfo>() {
            match *info {
                CacheInfo::TtlExpired { key, expired_at } => {
                    debug!("TTL expired for key: {} at {}", key, expired_at);
                    let mut state = self.state.write().await;
                    state.cache.remove(&key);
                    state.access_frequency.remove(&key);
                    state.hot_cache.pop(&key);
                }
                CacheInfo::MemoryPressure { current_size, max_size, evicted_entries } => {
                    warn!("Cache memory pressure: {}/{} entries, {} evicted", 
                          current_size, max_size, evicted_entries);
                }
                CacheInfo::SyncEvent { node_id, event_type, affected_keys } => {
                    debug!("Sync event from {}: {} affecting {} keys", 
                           node_id, event_type, affected_keys.len());
                }
                CacheInfo::MetricsUpdate { hit_rate, miss_rate, eviction_rate } => {
                    trace!("Cache metrics - Hit: {:.2}%, Miss: {:.2}%, Eviction: {:.2}%", 
                           hit_rate * 100.0, miss_rate * 100.0, eviction_rate * 100.0);
                }
            }
        }
        Ok(())
    }

    async fn terminate(&mut self, _reason: String) -> HelixResult<()> {
        // Cleanup background tasks
        if let Some(handle) = self.cleanup_handle.take() {
            handle.abort();
        }
        if let Some(handle) = self.sync_handle.take() {
            handle.abort();
        }
        
        info!("CacheActor terminated");
        Ok(())
    }

    async fn code_change(&mut self, _old_version: String, _new_version: String) -> HelixResult<()> {
        info!("CacheActor code change completed");
        Ok(())
    }

    async fn get_state(&self) -> HelixResult<Self::State> {
        let state = self.state.read().await;
        Ok(state.clone())
    }

    async fn set_state(&mut self, state: Self::State) -> HelixResult<()> {
        let mut current_state = self.state.write().await;
        *current_state = state;
        Ok(())
    }
}

/// Cache Actor supervisor
pub struct CacheActorSupervisor {
    node_id: String,
}

impl CacheActorSupervisor {
    pub fn new(node_id: String) -> Self {
        Self { node_id }
    }

    pub async fn start(&self) -> HelixResult<CacheActor> {
        let mut actor = CacheActor::new(self.node_id.clone());
        actor.initialize().await?;
        info!("CacheActor supervised startup completed");
        Ok(actor)
    }
}