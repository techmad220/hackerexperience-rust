use crate::auth::{WebSocketAuth, TopicAccessControl};
use crate::client::{WebSocketClient, ClientError};
use crate::events::{WebSocketMessage, GameEvent, ChannelResponse, ChannelStatus};
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Channel manager that handles topic subscriptions and message routing
pub struct ChannelManager {
    channels: DashMap<String, Arc<Channel>>,
    access_control: Arc<TopicAccessControl>,
    global_broadcaster: broadcast::Sender<WebSocketMessage>,
}

/// Represents a single communication channel/topic
pub struct Channel {
    pub topic: String,
    pub subscribers: Arc<RwLock<HashMap<Uuid, Arc<WebSocketClient>>>>,
    pub broadcaster: broadcast::Sender<WebSocketMessage>,
    pub handler: Option<Arc<dyn ChannelHandler>>,
    pub config: ChannelConfig,
    pub stats: Arc<RwLock<ChannelStats>>,
}

#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub max_subscribers: Option<usize>,
    pub require_authentication: bool,
    pub allow_anonymous: bool,
    pub message_rate_limit: Option<u32>, // messages per minute
    pub persistent: bool, // whether to persist messages
    pub ttl: Option<u64>, // time to live in seconds
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            max_subscribers: Some(1000),
            require_authentication: true,
            allow_anonymous: false,
            message_rate_limit: Some(60), // 1 message per second
            persistent: false,
            ttl: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ChannelStats {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub subscriber_count: usize,
    pub total_messages: u64,
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
    pub peak_subscribers: usize,
}

/// Trait for handling channel-specific logic
#[async_trait]
pub trait ChannelHandler: Send + Sync {
    async fn on_join(
        &self,
        client: &WebSocketClient,
        topic: &str,
        payload: &serde_json::Value,
    ) -> Result<ChannelResponse, ChannelError>;

    async fn on_leave(
        &self,
        client: &WebSocketClient,
        topic: &str,
    ) -> Result<(), ChannelError>;

    async fn on_message(
        &self,
        client: &WebSocketClient,
        topic: &str,
        event: &str,
        payload: &serde_json::Value,
    ) -> Result<Option<WebSocketMessage>, ChannelError>;

    async fn can_join(
        &self,
        client: &WebSocketClient,
        topic: &str,
    ) -> Result<bool, ChannelError>;
}

impl ChannelManager {
    pub fn new(access_control: Arc<TopicAccessControl>) -> Self {
        let (global_broadcaster, _) = broadcast::channel(10000);
        
        Self {
            channels: DashMap::new(),
            access_control,
            global_broadcaster,
        }
    }

    /// Create or get a channel for a topic
    pub async fn get_or_create_channel(&self, topic: String, config: Option<ChannelConfig>) -> Arc<Channel> {
        if let Some(channel) = self.channels.get(&topic) {
            return channel.clone();
        }

        let (broadcaster, _) = broadcast::channel(1000);
        let channel = Arc::new(Channel {
            topic: topic.clone(),
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            broadcaster,
            handler: self.create_handler_for_topic(&topic),
            config: config.unwrap_or_default(),
            stats: Arc::new(RwLock::new(ChannelStats {
                created_at: chrono::Utc::now(),
                ..Default::default()
            })),
        });

        self.channels.insert(topic.clone(), channel.clone());
        info!("Created new channel: {}", topic);
        
        channel
    }

    /// Subscribe a client to a channel
    pub async fn join_channel(
        &self,
        client: Arc<WebSocketClient>,
        topic: String,
        payload: Option<serde_json::Value>,
    ) -> Result<ChannelResponse, ChannelError> {
        // Check access permissions
        if !self.access_control.can_join_topic(&client, &topic).await? {
            return Ok(ChannelResponse {
                status: ChannelStatus::Error,
                response: serde_json::json!({ "reason": "Permission denied" }),
            });
        }

        let channel = self.get_or_create_channel(topic.clone(), None).await;

        // Check channel-specific join conditions
        if let Some(handler) = &channel.handler {
            if !handler.can_join(&client, &topic).await? {
                return Ok(ChannelResponse {
                    status: ChannelStatus::Error,
                    response: serde_json::json!({ "reason": "Channel join conditions not met" }),
                });
            }
        }

        // Check subscriber limits
        let subscriber_count = {
            let subscribers = channel.subscribers.read().await;
            subscribers.len()
        };

        if let Some(max) = channel.config.max_subscribers {
            if subscriber_count >= max {
                return Ok(ChannelResponse {
                    status: ChannelStatus::Error,
                    response: serde_json::json!({ "reason": "Channel is full" }),
                });
            }
        }

        // Add subscriber to channel
        {
            let mut subscribers = channel.subscribers.write().await;
            subscribers.insert(client.id, client.clone());
        }

        // Update client subscriptions
        client.subscribe(topic.clone()).await.map_err(ChannelError::ClientError)?;

        // Update channel stats
        {
            let mut stats = channel.stats.write().await;
            stats.subscriber_count = subscriber_count + 1;
            stats.last_activity = Some(chrono::Utc::now());
            if stats.subscriber_count > stats.peak_subscribers {
                stats.peak_subscribers = stats.subscriber_count;
            }
        }

        // Call handler's on_join method
        let response = if let Some(handler) = &channel.handler {
            handler.on_join(&client, &topic, &payload.unwrap_or_default()).await?
        } else {
            ChannelResponse {
                status: ChannelStatus::Ok,
                response: serde_json::json!({}),
            }
        };

        info!("Client {} joined channel: {}", client.id, topic);
        
        // Broadcast join event to other subscribers
        let join_event = WebSocketMessage::new(
            topic.clone(),
            "user_joined".to_string(),
            serde_json::json!({
                "user_id": client.user_id,
                "client_id": client.id
            }),
        );

        if let Err(e) = channel.broadcaster.send(join_event) {
            warn!("Failed to broadcast join event: {}", e);
        }

        Ok(response)
    }

    /// Unsubscribe a client from a channel
    pub async fn leave_channel(&self, client: &WebSocketClient, topic: &str) -> Result<(), ChannelError> {
        if let Some(channel) = self.channels.get(topic) {
            // Remove from subscribers
            {
                let mut subscribers = channel.subscribers.write().await;
                subscribers.remove(&client.id);
            }

            // Update client subscriptions
            client.unsubscribe(topic).await.map_err(ChannelError::ClientError)?;

            // Update channel stats
            {
                let mut stats = channel.stats.write().await;
                let subscribers = channel.subscribers.read().await;
                stats.subscriber_count = subscribers.len();
                stats.last_activity = Some(chrono::Utc::now());
            }

            // Call handler's on_leave method
            if let Some(handler) = &channel.handler {
                handler.on_leave(client, topic).await?;
            }

            info!("Client {} left channel: {}", client.id, topic);

            // Broadcast leave event
            let leave_event = WebSocketMessage::new(
                topic.to_string(),
                "user_left".to_string(),
                serde_json::json!({
                    "user_id": client.user_id,
                    "client_id": client.id
                }),
            );

            if let Err(e) = channel.broadcaster.send(leave_event) {
                warn!("Failed to broadcast leave event: {}", e);
            }

            // Clean up empty channels if not persistent
            if !channel.config.persistent {
                let subscribers = channel.subscribers.read().await;
                if subscribers.is_empty() {
                    drop(subscribers);
                    self.channels.remove(topic);
                    info!("Cleaned up empty channel: {}", topic);
                }
            }
        }

        Ok(())
    }

    /// Send a message to a channel
    pub async fn send_to_channel(
        &self,
        topic: &str,
        message: WebSocketMessage,
    ) -> Result<usize, ChannelError> {
        if let Some(channel) = self.channels.get(topic) {
            let subscribers = channel.subscribers.read().await;
            let mut sent_count = 0;

            // Update stats
            {
                let mut stats = channel.stats.write().await;
                stats.total_messages += 1;
                stats.last_activity = Some(chrono::Utc::now());
            }

            // Send to all subscribers
            for (client_id, client) in subscribers.iter() {
                if let Err(e) = client.send_message(message.clone()).await {
                    warn!("Failed to send message to client {}: {}", client_id, e);
                } else {
                    sent_count += 1;
                }
            }

            debug!("Sent message to {} subscribers in channel: {}", sent_count, topic);
            Ok(sent_count)
        } else {
            Ok(0)
        }
    }

    /// Broadcast a game event to the appropriate channels
    pub async fn broadcast_event(&self, event: GameEvent) -> Result<(), ChannelError> {
        let topics = self.determine_topics_for_event(&event);
        
        for topic in topics {
            let message = WebSocketMessage::from_game_event(topic.clone(), event.clone())
                .map_err(ChannelError::SerializationError)?;
            
            self.send_to_channel(&topic, message).await?;
        }

        Ok(())
    }

    /// Handle a client message within a channel context
    pub async fn handle_channel_message(
        &self,
        client: &WebSocketClient,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, ChannelError> {
        // Check if client can send to this topic
        if !self.access_control.can_send_to_topic(client, &message.topic).await? {
            return Err(ChannelError::PermissionDenied);
        }

        if let Some(channel) = self.channels.get(&message.topic) {
            if let Some(handler) = &channel.handler {
                return handler.on_message(client, &message.topic, &message.event, &message.payload).await;
            }
        }

        // Default handling - just echo the message back to the channel
        Ok(Some(message))
    }

    /// Get channel statistics
    pub async fn get_channel_stats(&self, topic: &str) -> Option<ChannelStats> {
        if let Some(channel) = self.channels.get(topic) {
            let stats = channel.stats.read().await;
            Some(stats.clone())
        } else {
            None
        }
    }

    /// Get all active channels
    pub fn get_active_channels(&self) -> Vec<String> {
        self.channels.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Remove a client from all channels
    pub async fn remove_client_from_all_channels(&self, client_id: Uuid) {
        for channel_entry in self.channels.iter() {
            let channel = channel_entry.value();
            let mut subscribers = channel.subscribers.write().await;
            if subscribers.remove(&client_id).is_some() {
                info!("Removed client {} from channel {}", client_id, channel_entry.key());
            }
        }
    }

    /// Create channel handlers based on topic patterns
    fn create_handler_for_topic(&self, topic: &str) -> Option<Arc<dyn ChannelHandler>> {
        match topic {
            t if t.starts_with("user:") => Some(Arc::new(UserChannelHandler)),
            t if t.starts_with("server:") => Some(Arc::new(ServerChannelHandler)),
            t if t.starts_with("process:") => Some(Arc::new(ProcessChannelHandler)),
            t if t.starts_with("chat:") => Some(Arc::new(ChatChannelHandler)),
            "lobby:global" => Some(Arc::new(LobbyChannelHandler)),
            _ => None,
        }
    }

    /// Determine which topics should receive a game event
    fn determine_topics_for_event(&self, event: &GameEvent) -> Vec<String> {
        match event {
            GameEvent::ProcessStarted { .. } | 
            GameEvent::ProcessCompleted { .. } | 
            GameEvent::ProcessFailed { .. } | 
            GameEvent::ProcessProgress { .. } => {
                // Send to user and process-specific channels
                // TODO: Extract user_id and process_id from event
                vec!["system:processes".to_string()]
            }
            GameEvent::ServerConnected { server_id, .. } | 
            GameEvent::ServerDisconnected { server_id, .. } => {
                vec![format!("server:{}", server_id)]
            }
            GameEvent::ChatMessage { channel, .. } => {
                vec![format!("chat:{}", channel)]
            }
            GameEvent::SystemStatus { .. } | 
            GameEvent::Maintenance { .. } => {
                vec!["system:announcements".to_string()]
            }
            _ => vec!["system:global".to_string()],
        }
    }
}

/// Built-in channel handlers
pub struct UserChannelHandler;
pub struct ServerChannelHandler;
pub struct ProcessChannelHandler;
pub struct ChatChannelHandler;
pub struct LobbyChannelHandler;

#[async_trait]
impl ChannelHandler for UserChannelHandler {
    async fn on_join(&self, client: &WebSocketClient, topic: &str, _payload: &serde_json::Value) -> Result<ChannelResponse, ChannelError> {
        // Extract user ID from topic
        let topic_user_id = topic.strip_prefix("user:")
            .and_then(|id_str| Uuid::parse_str(id_str).ok());

        // Only allow users to join their own user channel
        if client.user_id != topic_user_id {
            return Ok(ChannelResponse {
                status: ChannelStatus::Error,
                response: serde_json::json!({ "reason": "Can only join your own user channel" }),
            });
        }

        Ok(ChannelResponse {
            status: ChannelStatus::Ok,
            response: serde_json::json!({ "user_id": client.user_id }),
        })
    }

    async fn on_leave(&self, _client: &WebSocketClient, _topic: &str) -> Result<(), ChannelError> {
        Ok(())
    }

    async fn on_message(&self, _client: &WebSocketClient, _topic: &str, _event: &str, _payload: &serde_json::Value) -> Result<Option<WebSocketMessage>, ChannelError> {
        // User channels are typically for receiving events, not sending
        Ok(None)
    }

    async fn can_join(&self, client: &WebSocketClient, topic: &str) -> Result<bool, ChannelError> {
        let topic_user_id = topic.strip_prefix("user:")
            .and_then(|id_str| Uuid::parse_str(id_str).ok());
        Ok(client.user_id == topic_user_id)
    }
}

#[async_trait]
impl ChannelHandler for ServerChannelHandler {
    async fn on_join(&self, _client: &WebSocketClient, _topic: &str, _payload: &serde_json::Value) -> Result<ChannelResponse, ChannelError> {
        // TODO: Check if user has access to this server
        Ok(ChannelResponse {
            status: ChannelStatus::Ok,
            response: serde_json::json!({}),
        })
    }

    async fn on_leave(&self, _client: &WebSocketClient, _topic: &str) -> Result<(), ChannelError> {
        Ok(())
    }

    async fn on_message(&self, _client: &WebSocketClient, _topic: &str, _event: &str, _payload: &serde_json::Value) -> Result<Option<WebSocketMessage>, ChannelError> {
        // Handle server-specific commands
        debug!("Server channel message - event: {}, payload: {}", _event, _payload);
        Ok(None)
    }

    async fn can_join(&self, _client: &WebSocketClient, _topic: &str) -> Result<bool, ChannelError> {
        // TODO: Implement server access checking
        Ok(true)
    }
}

#[async_trait]
impl ChannelHandler for ProcessChannelHandler {
    async fn on_join(&self, _client: &WebSocketClient, _topic: &str, _payload: &serde_json::Value) -> Result<ChannelResponse, ChannelError> {
        Ok(ChannelResponse {
            status: ChannelStatus::Ok,
            response: serde_json::json!({}),
        })
    }

    async fn on_leave(&self, _client: &WebSocketClient, _topic: &str) -> Result<(), ChannelError> {
        Ok(())
    }

    async fn on_message(&self, _client: &WebSocketClient, _topic: &str, _event: &str, _payload: &serde_json::Value) -> Result<Option<WebSocketMessage>, ChannelError> {
        // Handle process control commands
        debug!("Process channel message - event: {}, payload: {}", _event, _payload);
        Ok(None)
    }

    async fn can_join(&self, _client: &WebSocketClient, _topic: &str) -> Result<bool, ChannelError> {
        // TODO: Check if user owns the process
        Ok(true)
    }
}

#[async_trait]
impl ChannelHandler for ChatChannelHandler {
    async fn on_join(&self, _client: &WebSocketClient, _topic: &str, _payload: &serde_json::Value) -> Result<ChannelResponse, ChannelError> {
        Ok(ChannelResponse {
            status: ChannelStatus::Ok,
            response: serde_json::json!({ "channel": _topic }),
        })
    }

    async fn on_leave(&self, _client: &WebSocketClient, _topic: &str) -> Result<(), ChannelError> {
        Ok(())
    }

    async fn on_message(&self, client: &WebSocketClient, topic: &str, event: &str, payload: &serde_json::Value) -> Result<Option<WebSocketMessage>, ChannelError> {
        if event == "message" {
            if let Some(message_text) = payload.get("message").and_then(|m| m.as_str()) {
                // Create chat event
                let chat_event = GameEvent::ChatMessage {
                    channel: topic.strip_prefix("chat:").unwrap_or(topic).to_string(),
                    sender: client.user_id.map(|id| id.to_string()).unwrap_or_else(|| "anonymous".to_string()),
                    message: message_text.to_string(),
                    timestamp: chrono::Utc::now().timestamp(),
                };

                return Ok(Some(WebSocketMessage::from_game_event(topic.to_string(), chat_event)?));
            }
        }
        Ok(None)
    }

    async fn can_join(&self, _client: &WebSocketClient, _topic: &str) -> Result<bool, ChannelError> {
        // TODO: Implement chat channel access rules
        Ok(true)
    }
}

#[async_trait]
impl ChannelHandler for LobbyChannelHandler {
    async fn on_join(&self, _client: &WebSocketClient, _topic: &str, _payload: &serde_json::Value) -> Result<ChannelResponse, ChannelError> {
        Ok(ChannelResponse {
            status: ChannelStatus::Ok,
            response: serde_json::json!({ "welcome": "Welcome to HackerExperience!" }),
        })
    }

    async fn on_leave(&self, _client: &WebSocketClient, _topic: &str) -> Result<(), ChannelError> {
        Ok(())
    }

    async fn on_message(&self, _client: &WebSocketClient, _topic: &str, _event: &str, _payload: &serde_json::Value) -> Result<Option<WebSocketMessage>, ChannelError> {
        // Lobby is typically read-only
        Ok(None)
    }

    async fn can_join(&self, _client: &WebSocketClient, _topic: &str) -> Result<bool, ChannelError> {
        // Lobby is open to everyone
        Ok(true)
    }
}

#[derive(Debug, Error)]
pub enum ChannelError {
    #[error("Client error: {0}")]
    ClientError(#[from] ClientError),
    
    #[error("Authentication error: {0}")]
    AuthenticationError(#[from] crate::auth::AuthError),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Channel not found: {0}")]
    ChannelNotFound(String),
    
    #[error("Permission denied")]
    PermissionDenied,
    
    #[error("Channel is full")]
    ChannelFull,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Invalid message format")]
    InvalidMessageFormat,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{JwtAuthProvider, WebSocketAuth};
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_channel_creation() {
        let provider = Arc::new(JwtAuthProvider::new("secret".to_string()));
        let auth = Arc::new(WebSocketAuth::new(provider));
        let access_control = Arc::new(TopicAccessControl::new(auth));
        let manager = ChannelManager::new(access_control);

        let channel = manager.get_or_create_channel("test:channel".to_string(), None).await;
        assert_eq!(channel.topic, "test:channel");
        
        let stats = channel.stats.read().await;
        assert_eq!(stats.subscriber_count, 0);
    }

    #[tokio::test]
    async fn test_channel_subscription() {
        let provider = Arc::new(JwtAuthProvider::new("secret".to_string()));
        let auth = Arc::new(WebSocketAuth::new(provider));
        let access_control = Arc::new(TopicAccessControl::new(auth));
        let manager = ChannelManager::new(access_control);

        let (sender, _receiver) = mpsc::unbounded_channel();
        let mut client = WebSocketClient::new(Uuid::new_v4(), sender);
        client.authenticate(Uuid::new_v4()).await;
        let client = Arc::new(client);

        let topic = "lobby:global".to_string();
        let response = manager.join_channel(client.clone(), topic.clone(), None).await.unwrap();
        
        match response.status {
            ChannelStatus::Ok => {
                assert!(client.is_subscribed(&topic).await);
            }
            _ => panic!("Expected successful join"),
        }
    }
}