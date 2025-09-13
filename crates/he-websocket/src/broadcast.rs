use crate::client::WebSocketClient;
use crate::events::{WebSocketMessage, GameEvent};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::{interval, Duration, Instant};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// High-level broadcasting system for managing real-time events
pub struct BroadcastSystem {
    /// Global event broadcaster
    global_sender: broadcast::Sender<BroadcastEvent>,
    /// Topic-specific broadcasters
    topic_broadcasters: DashMap<String, broadcast::Sender<BroadcastEvent>>,
    /// User-specific broadcasters
    user_broadcasters: DashMap<Uuid, broadcast::Sender<BroadcastEvent>>,
    /// Active subscriptions by client
    client_subscriptions: Arc<RwLock<HashMap<Uuid, Vec<String>>>>,
    /// Broadcast statistics
    stats: Arc<RwLock<BroadcastStats>>,
    /// Configuration
    config: BroadcastConfig,
}

#[derive(Debug, Clone)]
pub struct BroadcastConfig {
    /// Maximum number of events in each broadcast channel
    pub channel_capacity: usize,
    /// Maximum number of topic-specific channels
    pub max_topic_channels: usize,
    /// How often to clean up unused channels (in seconds)
    pub cleanup_interval: u64,
    /// How long a channel can be inactive before cleanup (in seconds)
    pub channel_ttl: u64,
    /// Whether to persist broadcast events
    pub persist_events: bool,
    /// Rate limiting configuration
    pub rate_limit: Option<RateLimitConfig>,
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum events per second per user
    pub max_events_per_second: u32,
    /// Maximum events per minute per user
    pub max_events_per_minute: u32,
    /// Window size for rate limiting
    pub window_size: Duration,
}

impl Default for BroadcastConfig {
    fn default() -> Self {
        Self {
            channel_capacity: 1000,
            max_topic_channels: 10000,
            cleanup_interval: 300, // 5 minutes
            channel_ttl: 3600,     // 1 hour
            persist_events: false,
            rate_limit: Some(RateLimitConfig {
                max_events_per_second: 10,
                max_events_per_minute: 600,
                window_size: Duration::from_secs(60),
            }),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BroadcastStats {
    pub total_events: u64,
    pub events_per_topic: HashMap<String, u64>,
    pub active_channels: usize,
    pub active_subscriptions: usize,
    pub failed_broadcasts: u64,
    pub rate_limited_events: u64,
    pub last_cleanup: Option<Instant>,
}

/// Wrapper for events with metadata
#[derive(Debug, Clone)]
pub struct BroadcastEvent {
    pub id: Uuid,
    pub timestamp: Instant,
    pub topic: String,
    pub user_id: Option<Uuid>,
    pub event: GameEvent,
    pub priority: EventPriority,
    pub metadata: EventMetadata,
}

#[derive(Debug, Clone)]
pub struct EventMetadata {
    pub source: String,
    pub correlation_id: Option<String>,
    pub retry_count: u32,
    pub expires_at: Option<Instant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Event subscription filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    pub topics: Option<Vec<String>>,
    pub event_types: Option<Vec<String>>,
    pub user_ids: Option<Vec<Uuid>>,
    pub priority: Option<EventPriority>,
}

impl BroadcastSystem {
    pub fn new(config: BroadcastConfig) -> Self {
        let (global_sender, _) = broadcast::channel(config.channel_capacity);
        
        Self {
            global_sender,
            topic_broadcasters: DashMap::new(),
            user_broadcasters: DashMap::new(),
            client_subscriptions: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(BroadcastStats::default())),
            config,
        }
    }

    /// Start the broadcast system background tasks
    pub async fn start(&self) -> mpsc::Receiver<BroadcastEvent> {
        let (event_sender, event_receiver) = mpsc::channel(1000);
        
        // Start cleanup task
        self.start_cleanup_task().await;
        
        // Start stats collection task
        self.start_stats_task().await;
        
        event_receiver
    }

    /// Broadcast an event to all relevant subscribers
    pub async fn broadcast_event(&self, event: GameEvent) -> Result<BroadcastResult, BroadcastError> {
        let broadcast_event = BroadcastEvent {
            id: Uuid::new_v4(),
            timestamp: Instant::now(),
            topic: self.determine_primary_topic(&event),
            user_id: self.extract_user_id(&event),
            event,
            priority: EventPriority::Normal,
            metadata: EventMetadata {
                source: "system".to_string(),
                correlation_id: None,
                retry_count: 0,
                expires_at: None,
            },
        };

        self.broadcast_event_internal(broadcast_event).await
    }

    /// Broadcast an event with custom metadata
    pub async fn broadcast_event_with_metadata(
        &self,
        event: GameEvent,
        priority: EventPriority,
        metadata: EventMetadata,
    ) -> Result<BroadcastResult, BroadcastError> {
        let broadcast_event = BroadcastEvent {
            id: Uuid::new_v4(),
            timestamp: Instant::now(),
            topic: self.determine_primary_topic(&event),
            user_id: self.extract_user_id(&event),
            event,
            priority,
            metadata,
        };

        self.broadcast_event_internal(broadcast_event).await
    }

    /// Broadcast to a specific topic
    pub async fn broadcast_to_topic(
        &self,
        topic: String,
        event: GameEvent,
    ) -> Result<BroadcastResult, BroadcastError> {
        let broadcast_event = BroadcastEvent {
            id: Uuid::new_v4(),
            timestamp: Instant::now(),
            topic,
            user_id: self.extract_user_id(&event),
            event,
            priority: EventPriority::Normal,
            metadata: EventMetadata {
                source: "topic".to_string(),
                correlation_id: None,
                retry_count: 0,
                expires_at: None,
            },
        };

        self.broadcast_event_internal(broadcast_event).await
    }

    /// Broadcast to a specific user
    pub async fn broadcast_to_user(
        &self,
        user_id: Uuid,
        event: GameEvent,
    ) -> Result<BroadcastResult, BroadcastError> {
        let broadcast_event = BroadcastEvent {
            id: Uuid::new_v4(),
            timestamp: Instant::now(),
            topic: format!("user:{}", user_id),
            user_id: Some(user_id),
            event,
            priority: EventPriority::Normal,
            metadata: EventMetadata {
                source: "user".to_string(),
                correlation_id: None,
                retry_count: 0,
                expires_at: None,
            },
        };

        // Send to user-specific broadcaster if it exists
        if let Some(user_broadcaster) = self.user_broadcasters.get(&user_id) {
            let result = user_broadcaster.send(broadcast_event.clone());
            match result {
                Ok(subscriber_count) => {
                    self.update_stats(&broadcast_event, subscriber_count).await;
                    Ok(BroadcastResult {
                        event_id: broadcast_event.id,
                        subscribers_notified: subscriber_count,
                        topic: broadcast_event.topic,
                    })
                }
                Err(_) => {
                    // Channel closed, remove it
                    self.user_broadcasters.remove(&user_id);
                    Err(BroadcastError::ChannelClosed)
                }
            }
        } else {
            // No active subscribers for this user
            Ok(BroadcastResult {
                event_id: broadcast_event.id,
                subscribers_notified: 0,
                topic: broadcast_event.topic,
            })
        }
    }

    /// Subscribe to events for a topic
    pub async fn subscribe_to_topic(
        &self,
        client_id: Uuid,
        topic: String,
    ) -> broadcast::Receiver<BroadcastEvent> {
        // Get or create topic broadcaster
        let broadcaster = self.get_or_create_topic_broadcaster(&topic).await;
        let receiver = broadcaster.subscribe();

        // Track subscription
        {
            let mut subscriptions = self.client_subscriptions.write().await;
            subscriptions.entry(client_id).or_insert_with(Vec::new).push(topic.clone());
        }

        debug!("Client {} subscribed to topic: {}", client_id, topic);
        receiver
    }

    /// Subscribe to events for a user
    pub async fn subscribe_to_user(
        &self,
        client_id: Uuid,
        user_id: Uuid,
    ) -> broadcast::Receiver<BroadcastEvent> {
        // Get or create user broadcaster
        let broadcaster = self.get_or_create_user_broadcaster(user_id).await;
        let receiver = broadcaster.subscribe();

        // Track subscription
        let topic = format!("user:{}", user_id);
        {
            let mut subscriptions = self.client_subscriptions.write().await;
            subscriptions.entry(client_id).or_insert_with(Vec::new).push(topic);
        }

        debug!("Client {} subscribed to user: {}", client_id, user_id);
        receiver
    }

    /// Unsubscribe from all topics for a client
    pub async fn unsubscribe_client(&self, client_id: Uuid) {
        let mut subscriptions = self.client_subscriptions.write().await;
        if let Some(topics) = subscriptions.remove(&client_id) {
            debug!("Removed {} subscriptions for client {}", topics.len(), client_id);
        }
    }

    /// Get broadcast statistics
    pub async fn get_stats(&self) -> BroadcastStats {
        let stats = self.stats.read().await;
        let mut stats = stats.clone();
        
        // Update current active channel count
        stats.active_channels = self.topic_broadcasters.len() + self.user_broadcasters.len();
        
        // Update subscription count
        let subscriptions = self.client_subscriptions.read().await;
        stats.active_subscriptions = subscriptions.values().map(|v| v.len()).sum();
        
        stats
    }

    /// Internal broadcast implementation
    async fn broadcast_event_internal(
        &self,
        event: BroadcastEvent,
    ) -> Result<BroadcastResult, BroadcastError> {
        // Check rate limits if configured
        if let Some(rate_limit) = &self.config.rate_limit {
            if let Some(user_id) = event.user_id {
                if !self.check_rate_limit(user_id, rate_limit).await {
                    let mut stats = self.stats.write().await;
                    stats.rate_limited_events += 1;
                    return Err(BroadcastError::RateLimitExceeded);
                }
            }
        }

        let mut total_subscribers = 0;
        
        // Broadcast to global channel
        if let Ok(count) = self.global_sender.send(event.clone()) {
            total_subscribers += count;
        }

        // Broadcast to topic-specific channel
        if let Some(topic_broadcaster) = self.topic_broadcasters.get(&event.topic) {
            if let Ok(count) = topic_broadcaster.send(event.clone()) {
                total_subscribers += count;
            }
        }

        // Broadcast to user-specific channel if applicable
        if let Some(user_id) = event.user_id {
            if let Some(user_broadcaster) = self.user_broadcasters.get(&user_id) {
                if let Ok(count) = user_broadcaster.send(event.clone()) {
                    total_subscribers += count;
                }
            }
        }

        // Update statistics
        self.update_stats(&event, total_subscribers).await;

        Ok(BroadcastResult {
            event_id: event.id,
            subscribers_notified: total_subscribers,
            topic: event.topic,
        })
    }

    /// Get or create a topic broadcaster
    async fn get_or_create_topic_broadcaster(
        &self,
        topic: &str,
    ) -> broadcast::Sender<BroadcastEvent> {
        if let Some(broadcaster) = self.topic_broadcasters.get(topic) {
            return broadcaster.clone();
        }

        // Check if we've hit the channel limit
        if self.topic_broadcasters.len() >= self.config.max_topic_channels {
            // Clean up old channels first
            self.cleanup_inactive_channels().await;
            
            // If still at limit, use global broadcaster
            if self.topic_broadcasters.len() >= self.config.max_topic_channels {
                return self.global_sender.clone();
            }
        }

        let (sender, _) = broadcast::channel(self.config.channel_capacity);
        self.topic_broadcasters.insert(topic.to_string(), sender.clone());
        
        info!("Created topic broadcaster for: {}", topic);
        sender
    }

    /// Get or create a user broadcaster
    async fn get_or_create_user_broadcaster(
        &self,
        user_id: Uuid,
    ) -> broadcast::Sender<BroadcastEvent> {
        if let Some(broadcaster) = self.user_broadcasters.get(&user_id) {
            return broadcaster.clone();
        }

        let (sender, _) = broadcast::channel(self.config.channel_capacity);
        self.user_broadcasters.insert(user_id, sender.clone());
        
        debug!("Created user broadcaster for: {}", user_id);
        sender
    }

    /// Update broadcast statistics
    async fn update_stats(&self, event: &BroadcastEvent, subscriber_count: usize) {
        let mut stats = self.stats.write().await;
        stats.total_events += 1;
        *stats.events_per_topic.entry(event.topic.clone()).or_insert(0) += 1;
        
        if subscriber_count == 0 {
            stats.failed_broadcasts += 1;
        }
    }

    /// Determine primary topic for an event
    fn determine_primary_topic(&self, event: &GameEvent) -> String {
        match event {
            GameEvent::ProcessStarted { .. } |
            GameEvent::ProcessCompleted { .. } |
            GameEvent::ProcessFailed { .. } |
            GameEvent::ProcessProgress { .. } => "processes".to_string(),
            
            GameEvent::ServerConnected { server_id, .. } |
            GameEvent::ServerDisconnected { server_id, .. } => format!("server:{}", server_id),
            
            GameEvent::ChatMessage { channel, .. } => format!("chat:{}", channel),
            
            GameEvent::SystemStatus { .. } |
            GameEvent::Maintenance { .. } => "system".to_string(),
            
            GameEvent::ExperienceGained { .. } |
            GameEvent::MoneyChanged { .. } |
            GameEvent::NotificationReceived { .. } => "user".to_string(),
            
            _ => "global".to_string(),
        }
    }

    /// Extract user ID from event if applicable
    fn extract_user_id(&self, _event: &GameEvent) -> Option<Uuid> {
        // TODO: Implement based on event content
        // For now, return None as events don't directly contain user IDs
        None
    }

    /// Check rate limits for a user
    async fn check_rate_limit(&self, _user_id: Uuid, _config: &RateLimitConfig) -> bool {
        // TODO: Implement actual rate limiting with sliding window
        // For now, allow all requests
        true
    }

    /// Start background cleanup task
    async fn start_cleanup_task(&self) {
        let topic_broadcasters = self.topic_broadcasters.clone();
        let user_broadcasters = self.user_broadcasters.clone();
        let cleanup_interval = Duration::from_secs(self.config.cleanup_interval);
        let channel_ttl = Duration::from_secs(self.config.channel_ttl);

        tokio::spawn(async move {
            let mut interval = interval(cleanup_interval);
            
            loop {
                interval.tick().await;
                
                // Clean up topic broadcasters with no receivers
                let mut to_remove = Vec::new();
                for entry in topic_broadcasters.iter() {
                    if entry.receiver_count() == 0 {
                        to_remove.push(entry.key().clone());
                    }
                }
                
                for topic in to_remove {
                    topic_broadcasters.remove(&topic);
                    debug!("Cleaned up inactive topic broadcaster: {}", topic);
                }
                
                // Clean up user broadcasters with no receivers
                let mut user_to_remove = Vec::new();
                for entry in user_broadcasters.iter() {
                    if entry.receiver_count() == 0 {
                        user_to_remove.push(*entry.key());
                    }
                }
                
                for user_id in user_to_remove {
                    user_broadcasters.remove(&user_id);
                    debug!("Cleaned up inactive user broadcaster: {}", user_id);
                }
            }
        });
    }

    /// Start background stats collection task
    async fn start_stats_task(&self) {
        let stats = self.stats.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60)); // Update stats every minute
            
            loop {
                interval.tick().await;
                
                // Perform periodic stats updates
                let mut stats_lock = stats.write().await;
                stats_lock.last_cleanup = Some(Instant::now());
            }
        });
    }

    /// Clean up inactive channels
    async fn cleanup_inactive_channels(&self) {
        // This is called from get_or_create_topic_broadcaster when at capacity
        let mut to_remove = Vec::new();
        
        for entry in self.topic_broadcasters.iter() {
            if entry.receiver_count() == 0 {
                to_remove.push(entry.key().clone());
            }
        }
        
        for topic in to_remove {
            self.topic_broadcasters.remove(&topic);
        }
    }
}

#[derive(Debug, Clone)]
pub struct BroadcastResult {
    pub event_id: Uuid,
    pub subscribers_notified: usize,
    pub topic: String,
}

#[derive(Debug, Error)]
pub enum BroadcastError {
    #[error("Channel closed")]
    ChannelClosed,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Channel capacity exceeded")]
    ChannelCapacityExceeded,
    
    #[error("Invalid topic: {0}")]
    InvalidTopic(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_broadcast_system_creation() {
        let config = BroadcastConfig::default();
        let system = BroadcastSystem::new(config);
        
        let stats = system.get_stats().await;
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.active_channels, 0);
    }

    #[tokio::test]
    async fn test_event_broadcasting() {
        let config = BroadcastConfig::default();
        let system = BroadcastSystem::new(config);
        
        let event = GameEvent::SystemStatus {
            cpu_usage: 50.0,
            memory_usage: 60.0,
            active_processes: 10,
            uptime: 3600,
        };
        
        let result = system.broadcast_event(event).await.unwrap();
        assert_eq!(result.subscribers_notified, 0); // No subscribers yet
    }

    #[tokio::test]
    async fn test_topic_subscription() {
        let config = BroadcastConfig::default();
        let system = BroadcastSystem::new(config);
        
        let client_id = Uuid::new_v4();
        let topic = "test:topic".to_string();
        
        let _receiver = system.subscribe_to_topic(client_id, topic.clone()).await;
        
        let event = GameEvent::SystemStatus {
            cpu_usage: 50.0,
            memory_usage: 60.0,
            active_processes: 10,
            uptime: 3600,
        };
        
        let result = system.broadcast_to_topic(topic, event).await.unwrap();
        assert_eq!(result.subscribers_notified, 1);
    }
}