//! Comprehensive Event Publishing and Subscription System
//! 
//! Complete event system with filtering, routing, persistence, and distributed coordination.

use crate::{HelixError, HelixResult, ProcessId};
use async_trait::async_trait;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::any::{Any, TypeId};
use std::collections::{HashMap, BTreeMap, VecDeque};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, Mutex, broadcast, mpsc, oneshot, watch};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Event trait that all events must implement
pub trait Event: Send + Sync + Debug + Clone + 'static {
    /// Event type identifier
    fn event_type(&self) -> &'static str;
    
    /// Event metadata
    fn metadata(&self) -> &EventMetadata;
    
    /// Event priority (higher numbers = higher priority)
    fn priority(&self) -> u8 {
        5 // Default medium priority
    }
    
    /// Whether this event should be persisted
    fn persist(&self) -> bool {
        false
    }
    
    /// Event TTL (time to live)
    fn ttl(&self) -> Option<Duration> {
        None
    }
    
    /// Serialize event for storage/transmission
    fn serialize(&self) -> HelixResult<Vec<u8>>;
    
    /// Deserialize event from storage/transmission
    fn deserialize(data: &[u8]) -> HelixResult<Self> where Self: Sized;
}

/// Event metadata for tracking and correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id: EventId,
    pub source_id: Option<ProcessId>,
    pub source_type: String,
    pub timestamp: SystemTime,
    pub correlation_id: Option<String>,
    pub causation_id: Option<EventId>,
    pub version: u32,
    pub tags: Vec<String>,
    pub attributes: HashMap<String, String>,
    pub trace_context: Option<TraceContext>,
}

impl EventMetadata {
    pub fn new(source_type: String) -> Self {
        Self {
            event_id: EventId::new(),
            source_id: None,
            source_type,
            timestamp: SystemTime::now(),
            correlation_id: None,
            causation_id: None,
            version: 1,
            tags: Vec::new(),
            attributes: HashMap::new(),
            trace_context: None,
        }
    }

    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    pub fn with_causation_id(mut self, causation_id: EventId) -> Self {
        self.causation_id = Some(causation_id);
        self
    }

    pub fn with_source_id(mut self, source_id: ProcessId) -> Self {
        self.source_id = Some(source_id);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn add_attribute(&mut self, key: String, value: String) {
        self.attributes.insert(key, value);
    }
}

/// Event ID type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub Uuid);

impl EventId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Trace context for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub trace_flags: u8,
    pub trace_state: Option<String>,
}

/// Event subscription filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    /// Event types to match (empty = all types)
    pub event_types: Vec<String>,
    /// Source types to match (empty = all sources)
    pub source_types: Vec<String>,
    /// Source IDs to match (empty = all sources)
    pub source_ids: Vec<ProcessId>,
    /// Required tags (all must be present)
    pub required_tags: Vec<String>,
    /// Forbidden tags (none must be present)
    pub forbidden_tags: Vec<String>,
    /// Attribute filters (key -> regex pattern)
    pub attribute_filters: HashMap<String, String>,
    /// Priority range (min, max)
    pub priority_range: Option<(u8, u8)>,
    /// Time range for filtering
    pub time_range: Option<(SystemTime, SystemTime)>,
    /// Maximum events per second (rate limiting)
    pub max_rate: Option<u32>,
}

impl EventFilter {
    pub fn new() -> Self {
        Self {
            event_types: Vec::new(),
            source_types: Vec::new(),
            source_ids: Vec::new(),
            required_tags: Vec::new(),
            forbidden_tags: Vec::new(),
            attribute_filters: HashMap::new(),
            priority_range: None,
            time_range: None,
            max_rate: None,
        }
    }

    pub fn for_event_type(mut self, event_type: String) -> Self {
        self.event_types.push(event_type);
        self
    }

    pub fn for_source_type(mut self, source_type: String) -> Self {
        self.source_types.push(source_type);
        self
    }

    pub fn for_source_id(mut self, source_id: ProcessId) -> Self {
        self.source_ids.push(source_id);
        self
    }

    pub fn with_tag(mut self, tag: String) -> Self {
        self.required_tags.push(tag);
        self
    }

    pub fn without_tag(mut self, tag: String) -> Self {
        self.forbidden_tags.push(tag);
        self
    }

    pub fn with_priority_range(mut self, min: u8, max: u8) -> Self {
        self.priority_range = Some((min, max));
        self
    }

    pub fn with_max_rate(mut self, max_rate: u32) -> Self {
        self.max_rate = Some(max_rate);
        self
    }

    /// Check if an event matches this filter
    pub fn matches<E: Event>(&self, event: &E) -> bool {
        let metadata = event.metadata();

        // Check event type
        if !self.event_types.is_empty() && !self.event_types.contains(&event.event_type().to_string()) {
            return false;
        }

        // Check source type
        if !self.source_types.is_empty() && !self.source_types.contains(&metadata.source_type) {
            return false;
        }

        // Check source ID
        if !self.source_ids.is_empty() {
            if let Some(source_id) = metadata.source_id {
                if !self.source_ids.contains(&source_id) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check required tags
        for required_tag in &self.required_tags {
            if !metadata.tags.contains(required_tag) {
                return false;
            }
        }

        // Check forbidden tags
        for forbidden_tag in &self.forbidden_tags {
            if metadata.tags.contains(forbidden_tag) {
                return false;
            }
        }

        // Check priority range
        if let Some((min, max)) = self.priority_range {
            let priority = event.priority();
            if priority < min || priority > max {
                return false;
            }
        }

        // Check time range
        if let Some((start, end)) = self.time_range {
            if metadata.timestamp < start || metadata.timestamp > end {
                return false;
            }
        }

        // Check attribute filters (simplified - would use regex in production)
        for (key, pattern) in &self.attribute_filters {
            if let Some(value) = metadata.attributes.get(key) {
                if !value.contains(pattern) { // Simplified pattern matching
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

/// Event subscription configuration
#[derive(Debug, Clone)]
pub struct SubscriptionConfig {
    pub subscription_id: SubscriptionId,
    pub name: String,
    pub filter: EventFilter,
    pub handler: SubscriptionHandler,
    pub buffer_size: usize,
    pub enable_replay: bool,
    pub replay_from: Option<SystemTime>,
    pub enable_ordering: bool,
    pub enable_deduplication: bool,
    pub acknowledgment_required: bool,
    pub max_retries: u32,
    pub retry_delay: Duration,
}

/// Subscription ID type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubscriptionId(pub Uuid);

impl SubscriptionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Event subscription handler types
#[derive(Debug, Clone)]
pub enum SubscriptionHandler {
    /// Send to a channel
    Channel(mpsc::UnboundedSender<EventEnvelope>),
    /// Call a webhook URL
    Webhook { url: String, headers: HashMap<String, String> },
    /// Write to a file
    File { path: String, format: EventFormat },
    /// Forward to another event bus
    Forward { destination: String },
    /// Custom handler function
    Custom { handler_id: String },
}

/// Event envelope for delivery
#[derive(Debug, Clone)]
pub struct EventEnvelope {
    pub event_id: EventId,
    pub event_type: String,
    pub event_data: Vec<u8>,
    pub metadata: EventMetadata,
    pub delivery_attempt: u32,
    pub delivery_timestamp: SystemTime,
}

/// Event format for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventFormat {
    Json,
    Avro,
    Protobuf,
    MessagePack,
    Custom(String),
}

/// Event storage interface
#[async_trait]
pub trait EventStore: Send + Sync {
    /// Store an event
    async fn store(&self, event: &dyn Event) -> HelixResult<()>;
    
    /// Retrieve events by filter
    async fn retrieve(&self, filter: &EventFilter, limit: Option<usize>) -> HelixResult<Vec<EventEnvelope>>;
    
    /// Get events since a specific event ID
    async fn get_since(&self, since: EventId, limit: Option<usize>) -> HelixResult<Vec<EventEnvelope>>;
    
    /// Get events in a time range
    async fn get_range(&self, start: SystemTime, end: SystemTime, limit: Option<usize>) -> HelixResult<Vec<EventEnvelope>>;
    
    /// Clean up old events
    async fn cleanup(&self, older_than: SystemTime) -> HelixResult<u64>;
    
    /// Get storage statistics
    async fn stats(&self) -> HelixResult<StorageStats>;
}

/// Event storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_events: u64,
    pub storage_size: u64,
    pub oldest_event: Option<SystemTime>,
    pub newest_event: Option<SystemTime>,
    pub events_by_type: HashMap<String, u64>,
}

/// In-memory event store implementation
pub struct InMemoryEventStore {
    events: Arc<RwLock<BTreeMap<SystemTime, Vec<EventEnvelope>>>>,
    event_index: Arc<RwLock<HashMap<EventId, SystemTime>>>,
    type_index: Arc<RwLock<HashMap<String, Vec<EventId>>>>,
    max_events: usize,
}

impl InMemoryEventStore {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(BTreeMap::new())),
            event_index: Arc::new(RwLock::new(HashMap::new())),
            type_index: Arc::new(RwLock::new(HashMap::new())),
            max_events,
        }
    }
}

#[async_trait]
impl EventStore for InMemoryEventStore {
    async fn store(&self, event: &dyn Event) -> HelixResult<()> {
        let metadata = event.metadata();
        let envelope = EventEnvelope {
            event_id: metadata.event_id,
            event_type: event.event_type().to_string(),
            event_data: event.serialize()?,
            metadata: metadata.clone(),
            delivery_attempt: 0,
            delivery_timestamp: SystemTime::now(),
        };

        let mut events = self.events.write().await;
        let mut event_index = self.event_index.write().await;
        let mut type_index = self.type_index.write().await;

        // Add to main storage
        events.entry(metadata.timestamp)
            .or_insert_with(Vec::new)
            .push(envelope);

        // Update indices
        event_index.insert(metadata.event_id, metadata.timestamp);
        type_index.entry(event.event_type().to_string())
            .or_insert_with(Vec::new)
            .push(metadata.event_id);

        // Cleanup if we exceed max events
        let total_events: usize = events.values().map(|v| v.len()).sum();
        if total_events > self.max_events {
            self.cleanup_old_events(&mut events, &mut event_index, &mut type_index).await;
        }

        Ok(())
    }

    async fn retrieve(&self, filter: &EventFilter, limit: Option<usize>) -> HelixResult<Vec<EventEnvelope>> {
        let events = self.events.read().await;
        let mut results = Vec::new();
        let max_results = limit.unwrap_or(1000);

        for envelopes in events.values() {
            for envelope in envelopes {
                if results.len() >= max_results {
                    break;
                }

                // Simple filtering (would need to deserialize event for full filtering)
                if !filter.event_types.is_empty() && !filter.event_types.contains(&envelope.event_type) {
                    continue;
                }

                if let Some((start, end)) = filter.time_range {
                    if envelope.metadata.timestamp < start || envelope.metadata.timestamp > end {
                        continue;
                    }
                }

                results.push(envelope.clone());
            }
        }

        Ok(results)
    }

    async fn get_since(&self, since: EventId, limit: Option<usize>) -> HelixResult<Vec<EventEnvelope>> {
        let events = self.events.read().await;
        let event_index = self.event_index.read().await;

        let since_timestamp = event_index.get(&since)
            .ok_or_else(|| HelixError::NotFound(format!("Event {} not found", since)))?;

        let mut results = Vec::new();
        let max_results = limit.unwrap_or(1000);

        for (&timestamp, envelopes) in events.range(since_timestamp..) {
            for envelope in envelopes {
                if results.len() >= max_results {
                    break;
                }
                if envelope.metadata.timestamp > *since_timestamp {
                    results.push(envelope.clone());
                }
            }
        }

        Ok(results)
    }

    async fn get_range(&self, start: SystemTime, end: SystemTime, limit: Option<usize>) -> HelixResult<Vec<EventEnvelope>> {
        let events = self.events.read().await;
        let mut results = Vec::new();
        let max_results = limit.unwrap_or(1000);

        for (&timestamp, envelopes) in events.range(start..=end) {
            for envelope in envelopes {
                if results.len() >= max_results {
                    break;
                }
                results.push(envelope.clone());
            }
        }

        Ok(results)
    }

    async fn cleanup(&self, older_than: SystemTime) -> HelixResult<u64> {
        let mut events = self.events.write().await;
        let mut event_index = self.event_index.write().await;
        let mut type_index = self.type_index.write().await;

        let mut removed_count = 0u64;
        let timestamps_to_remove: Vec<SystemTime> = events.range(..older_than)
            .map(|(&timestamp, _)| timestamp)
            .collect();

        for timestamp in timestamps_to_remove {
            if let Some(envelopes) = events.remove(&timestamp) {
                for envelope in envelopes {
                    event_index.remove(&envelope.event_id);
                    
                    // Remove from type index
                    if let Some(type_events) = type_index.get_mut(&envelope.event_type) {
                        type_events.retain(|id| *id != envelope.event_id);
                    }
                    
                    removed_count += 1;
                }
            }
        }

        Ok(removed_count)
    }

    async fn stats(&self) -> HelixResult<StorageStats> {
        let events = self.events.read().await;
        let type_index = self.type_index.read().await;

        let total_events = events.values().map(|v| v.len() as u64).sum();
        let oldest_event = events.keys().next().copied();
        let newest_event = events.keys().last().copied();

        let events_by_type: HashMap<String, u64> = type_index.iter()
            .map(|(event_type, ids)| (event_type.clone(), ids.len() as u64))
            .collect();

        Ok(StorageStats {
            total_events,
            storage_size: total_events * 1000, // Rough estimate
            oldest_event,
            newest_event,
            events_by_type,
        })
    }
}

impl InMemoryEventStore {
    async fn cleanup_old_events(
        &self,
        events: &mut BTreeMap<SystemTime, Vec<EventEnvelope>>,
        event_index: &mut HashMap<EventId, SystemTime>,
        type_index: &mut HashMap<String, Vec<EventId>>,
    ) {
        // Remove oldest 20% of events
        let total_events: usize = events.values().map(|v| v.len()).sum();
        let events_to_remove = total_events / 5; // 20%
        let mut removed = 0;

        let mut timestamps_to_remove = Vec::new();
        for (&timestamp, envelopes) in events.iter() {
            if removed >= events_to_remove {
                break;
            }
            
            timestamps_to_remove.push(timestamp);
            removed += envelopes.len();
        }

        for timestamp in timestamps_to_remove {
            if let Some(envelopes) = events.remove(&timestamp) {
                for envelope in envelopes {
                    event_index.remove(&envelope.event_id);
                    if let Some(type_events) = type_index.get_mut(&envelope.event_type) {
                        type_events.retain(|id| *id != envelope.event_id);
                    }
                }
            }
        }

        info!("Cleaned up {} old events from storage", removed);
    }
}

/// Event bus configuration
#[derive(Debug, Clone)]
pub struct EventBusConfig {
    pub name: String,
    pub buffer_size: usize,
    pub enable_persistence: bool,
    pub enable_ordering: bool,
    pub enable_deduplication: bool,
    pub max_subscribers: usize,
    pub cleanup_interval: Duration,
    pub metrics_enabled: bool,
    pub distributed_mode: bool,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            buffer_size: 10000,
            enable_persistence: false,
            enable_ordering: false,
            enable_deduplication: false,
            max_subscribers: 1000,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            metrics_enabled: true,
            distributed_mode: false,
        }
    }
}

/// Event bus metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBusMetrics {
    pub events_published: u64,
    pub events_delivered: u64,
    pub events_failed: u64,
    pub active_subscriptions: usize,
    pub buffer_usage: f64,
    pub average_delivery_time: Duration,
    pub last_updated: SystemTime,
}

/// Main event bus implementation
pub struct EventBus {
    config: EventBusConfig,
    event_store: Option<Arc<dyn EventStore>>,
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, SubscriptionConfig>>>,
    subscribers: Arc<RwLock<HashMap<SubscriptionId, SubscriptionState>>>,
    event_tx: broadcast::Sender<EventEnvelope>,
    metrics: Arc<RwLock<EventBusMetrics>>,
    deduplication_cache: Arc<RwLock<HashMap<EventId, SystemTime>>>,
}

/// Subscription state tracking
#[derive(Debug)]
struct SubscriptionState {
    last_event_id: Option<EventId>,
    last_delivery: SystemTime,
    delivery_count: u64,
    failure_count: u64,
    rate_limiter: RateLimiter,
}

/// Simple rate limiter
#[derive(Debug)]
struct RateLimiter {
    max_rate: Option<u32>,
    tokens: u32,
    last_refill: SystemTime,
}

impl RateLimiter {
    fn new(max_rate: Option<u32>) -> Self {
        Self {
            max_rate,
            tokens: max_rate.unwrap_or(u32::MAX),
            last_refill: SystemTime::now(),
        }
    }

    fn try_acquire(&mut self) -> bool {
        if let Some(max_rate) = self.max_rate {
            let now = SystemTime::now();
            let elapsed = now.duration_since(self.last_refill).unwrap_or(Duration::ZERO);
            
            // Refill tokens based on elapsed time
            let tokens_to_add = (elapsed.as_secs() as u32 * max_rate) / 60; // per minute
            if tokens_to_add > 0 {
                self.tokens = (self.tokens + tokens_to_add).min(max_rate);
                self.last_refill = now;
            }
            
            if self.tokens > 0 {
                self.tokens -= 1;
                true
            } else {
                false
            }
        } else {
            true
        }
    }
}

impl EventBus {
    pub fn new(config: EventBusConfig) -> Self {
        let (event_tx, _) = broadcast::channel(config.buffer_size);
        
        let metrics = EventBusMetrics {
            events_published: 0,
            events_delivered: 0,
            events_failed: 0,
            active_subscriptions: 0,
            buffer_usage: 0.0,
            average_delivery_time: Duration::ZERO,
            last_updated: SystemTime::now(),
        };

        Self {
            config,
            event_store: None,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            metrics: Arc::new(RwLock::new(metrics)),
            deduplication_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_event_store(mut self, event_store: Arc<dyn EventStore>) -> Self {
        self.event_store = Some(event_store);
        self
    }

    /// Publish an event to the bus
    pub async fn publish<E: Event>(&self, event: E) -> HelixResult<()> {
        let event_id = event.metadata().event_id;
        
        // Check for duplicates if enabled
        if self.config.enable_deduplication {
            let mut dedup_cache = self.deduplication_cache.write().await;
            if dedup_cache.contains_key(&event_id) {
                debug!("Duplicate event {} ignored", event_id);
                return Ok(());
            }
            dedup_cache.insert(event_id, SystemTime::now());
        }

        // Store event if persistence is enabled
        if self.config.enable_persistence {
            if let Some(store) = &self.event_store {
                store.store(&event).await?;
            }
        }

        // Create envelope
        let envelope = EventEnvelope {
            event_id,
            event_type: event.event_type().to_string(),
            event_data: event.serialize()?,
            metadata: event.metadata().clone(),
            delivery_attempt: 0,
            delivery_timestamp: SystemTime::now(),
        };

        // Broadcast to all subscribers
        if let Err(_) = self.event_tx.send(envelope.clone()) {
            warn!("No active subscribers for event {}", event_id);
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.events_published += 1;
            metrics.last_updated = SystemTime::now();
        }

        info!("Published event {} of type {}", event_id, event.event_type());
        Ok(())
    }

    /// Subscribe to events with a filter
    pub async fn subscribe(&self, mut config: SubscriptionConfig) -> HelixResult<broadcast::Receiver<EventEnvelope>> {
        if self.subscriptions.read().await.len() >= self.config.max_subscribers {
            return Err(HelixError::LimitExceeded("Maximum subscribers reached".to_string()));
        }

        let subscription_id = config.subscription_id;
        
        // Create subscriber state
        let state = SubscriptionState {
            last_event_id: None,
            last_delivery: SystemTime::now(),
            delivery_count: 0,
            failure_count: 0,
            rate_limiter: RateLimiter::new(config.filter.max_rate),
        };

        // Create filtered event receiver
        let mut event_rx = self.event_tx.subscribe();
        let (filtered_tx, filtered_rx) = broadcast::channel(config.buffer_size);
        
        // Start filter task
        let filter = config.filter.clone();
        let subscription_name = config.name.clone();
        tokio::spawn(async move {
            while let Ok(envelope) = event_rx.recv().await {
                // Apply filter (simplified - would deserialize and check event)
                if filter.event_types.is_empty() || filter.event_types.contains(&envelope.event_type) {
                    if let Err(_) = filtered_tx.send(envelope) {
                        debug!("Subscriber '{}' channel closed", subscription_name);
                        break;
                    }
                }
            }
        });

        // Store subscription
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(subscription_id, config);
        }
        {
            let mut subscribers = self.subscribers.write().await;
            subscribers.insert(subscription_id, state);
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.active_subscriptions = self.subscriptions.read().await.len();
        }

        info!("Created subscription {} with filter", subscription_id);
        Ok(filtered_rx)
    }

    /// Unsubscribe from events
    pub async fn unsubscribe(&self, subscription_id: SubscriptionId) -> HelixResult<()> {
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.remove(&subscription_id);
        }
        {
            let mut subscribers = self.subscribers.write().await;
            subscribers.remove(&subscription_id);
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.active_subscriptions = self.subscriptions.read().await.len();
        }

        info!("Removed subscription {}", subscription_id);
        Ok(())
    }

    /// Get event bus metrics
    pub async fn get_metrics(&self) -> EventBusMetrics {
        self.metrics.read().await.clone()
    }

    /// Replay events for a subscriber
    pub async fn replay_events(
        &self,
        subscription_id: SubscriptionId,
        from: SystemTime,
        limit: Option<usize>,
    ) -> HelixResult<Vec<EventEnvelope>> {
        if let Some(store) = &self.event_store {
            let filter = {
                let subscriptions = self.subscriptions.read().await;
                subscriptions.get(&subscription_id)
                    .map(|config| config.filter.clone())
                    .ok_or_else(|| HelixError::NotFound("Subscription not found".to_string()))?
            };

            let mut replay_filter = filter;
            replay_filter.time_range = Some((from, SystemTime::now()));

            store.retrieve(&replay_filter, limit).await
        } else {
            Err(HelixError::NotImplemented("Event store not configured".to_string()))
        }
    }

    /// Clean up old events and subscriptions
    pub async fn cleanup(&self) -> HelixResult<()> {
        let cutoff = SystemTime::now() - self.config.cleanup_interval;

        // Clean up deduplication cache
        {
            let mut dedup_cache = self.deduplication_cache.write().await;
            dedup_cache.retain(|_, timestamp| *timestamp >= cutoff);
        }

        // Clean up event store
        if let Some(store) = &self.event_store {
            let removed = store.cleanup(cutoff).await?;
            info!("Cleaned up {} old events from store", removed);
        }

        Ok(())
    }

    /// Start background tasks
    pub async fn start(&self) -> HelixResult<()> {
        info!("Starting event bus '{}'", self.config.name);

        // Start cleanup task
        let cleanup_interval = self.config.cleanup_interval;
        let event_bus = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            loop {
                interval.tick().await;
                if let Err(e) = event_bus.cleanup().await {
                    error!("Event bus cleanup failed: {}", e);
                }
            }
        });

        Ok(())
    }
}

// Implement Clone for EventBus to allow sharing
impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            event_store: self.event_store.clone(),
            subscriptions: self.subscriptions.clone(),
            subscribers: self.subscribers.clone(),
            event_tx: self.event_tx.clone(),
            metrics: self.metrics.clone(),
            deduplication_cache: self.deduplication_cache.clone(),
        }
    }
}

/// Example event implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleEvent {
    pub metadata: EventMetadata,
    pub data: String,
}

impl Event for ExampleEvent {
    fn event_type(&self) -> &'static str {
        "example"
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    fn serialize(&self) -> HelixResult<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| HelixError::Serialization(e.to_string()))
    }

    fn deserialize(data: &[u8]) -> HelixResult<Self> {
        serde_json::from_slice(data).map_err(|e| HelixError::Serialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus_creation() {
        let config = EventBusConfig::default();
        let event_bus = EventBus::new(config);
        
        let metrics = event_bus.get_metrics().await;
        assert_eq!(metrics.events_published, 0);
        assert_eq!(metrics.active_subscriptions, 0);
    }

    #[tokio::test]
    async fn test_event_publishing_and_subscription() {
        let config = EventBusConfig::default();
        let event_bus = EventBus::new(config);

        // Create a subscription
        let subscription_config = SubscriptionConfig {
            subscription_id: SubscriptionId::new(),
            name: "test_subscription".to_string(),
            filter: EventFilter::new().for_event_type("example".to_string()),
            handler: SubscriptionHandler::Custom { handler_id: "test".to_string() },
            buffer_size: 100,
            enable_replay: false,
            replay_from: None,
            enable_ordering: false,
            enable_deduplication: false,
            acknowledgment_required: false,
            max_retries: 3,
            retry_delay: Duration::from_millis(100),
        };

        let mut event_rx = event_bus.subscribe(subscription_config).await.expect("Failed to subscribe");

        // Publish an event
        let event = ExampleEvent {
            metadata: EventMetadata::new("test".to_string()),
            data: "Hello, World!".to_string(),
        };

        event_bus.publish(event.clone()).await.expect("Failed to publish event");

        // Receive the event
        let received = tokio::time::timeout(Duration::from_millis(100), event_rx.recv()).await;
        assert!(received.is_ok());
        
        let envelope = received.unwrap().expect("Failed to receive event");
        assert_eq!(envelope.event_type, "example");

        // Check metrics
        let metrics = event_bus.get_metrics().await;
        assert_eq!(metrics.events_published, 1);
        assert_eq!(metrics.active_subscriptions, 1);
    }

    #[tokio::test]
    async fn test_event_filtering() {
        let filter = EventFilter::new()
            .for_event_type("test".to_string())
            .with_tag("important".to_string())
            .with_priority_range(5, 10);

        let event = ExampleEvent {
            metadata: EventMetadata::new("test".to_string())
                .with_tags(vec!["important".to_string(), "urgent".to_string()]),
            data: "Test data".to_string(),
        };

        assert!(filter.matches(&event));
        
        // Test non-matching event
        let event2 = ExampleEvent {
            metadata: EventMetadata::new("test".to_string())
                .with_tags(vec!["not_important".to_string()]),
            data: "Test data".to_string(),
        };

        assert!(!filter.matches(&event2));
    }

    #[tokio::test]
    async fn test_in_memory_event_store() {
        let store = InMemoryEventStore::new(1000);
        
        let event = ExampleEvent {
            metadata: EventMetadata::new("test".to_string()),
            data: "Test event".to_string(),
        };

        // Store the event
        store.store(&event).await.expect("Failed to store event");

        // Retrieve events
        let filter = EventFilter::new().for_event_type("example".to_string());
        let events = store.retrieve(&filter, Some(10)).await.expect("Failed to retrieve events");
        
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "example");

        // Check stats
        let stats = store.stats().await.expect("Failed to get stats");
        assert_eq!(stats.total_events, 1);
        assert!(stats.events_by_type.contains_key("example"));
    }
}