use crate::events::{WebSocketMessage, ChannelEvent, ChannelResponse, ChannelStatus};
use futures_util::{SinkExt, StreamExt};
use serde_json;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration, Instant};
use tokio_tungstenite::{accept_async, WebSocketStream};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub type WebSocket = WebSocketStream<TcpStream>;

/// Represents a connected WebSocket client
#[derive(Debug)]
pub struct WebSocketClient {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub authenticated: bool,
    pub subscriptions: Arc<RwLock<HashSet<String>>>,
    pub last_heartbeat: Arc<RwLock<Instant>>,
    pub sender: mpsc::UnboundedSender<WebSocketMessage>,
    pub metadata: Arc<RwLock<ClientMetadata>>,
}

#[derive(Debug, Clone)]
pub struct ClientMetadata {
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub connected_at: Instant,
    pub last_activity: Instant,
    pub message_count: u64,
    pub subscribed_topics: u32,
}

impl Default for ClientMetadata {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            ip_address: String::new(),
            user_agent: None,
            connected_at: now,
            last_activity: now,
            message_count: 0,
            subscribed_topics: 0,
        }
    }
}

impl WebSocketClient {
    pub fn new(id: Uuid, sender: mpsc::UnboundedSender<WebSocketMessage>) -> Self {
        Self {
            id,
            user_id: None,
            authenticated: false,
            subscriptions: Arc::new(RwLock::new(HashSet::new())),
            last_heartbeat: Arc::new(RwLock::new(Instant::now())),
            sender,
            metadata: Arc::new(RwLock::new(ClientMetadata::default())),
        }
    }

    pub async fn authenticate(&mut self, user_id: Uuid) {
        self.user_id = Some(user_id);
        self.authenticated = true;
        info!("Client {} authenticated as user {}", self.id, user_id);
    }

    pub async fn subscribe(&self, topic: String) -> Result<(), ClientError> {
        let mut subscriptions = self.subscriptions.write().await;
        if subscriptions.insert(topic.clone()) {
            let mut metadata = self.metadata.write().await;
            metadata.subscribed_topics = subscriptions.len() as u32;
            debug!("Client {} subscribed to topic: {}", self.id, topic);
        }
        Ok(())
    }

    pub async fn unsubscribe(&self, topic: &str) -> Result<(), ClientError> {
        let mut subscriptions = self.subscriptions.write().await;
        if subscriptions.remove(topic) {
            let mut metadata = self.metadata.write().await;
            metadata.subscribed_topics = subscriptions.len() as u32;
            debug!("Client {} unsubscribed from topic: {}", self.id, topic);
        }
        Ok(())
    }

    pub async fn is_subscribed(&self, topic: &str) -> bool {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.contains(topic)
    }

    pub async fn get_subscriptions(&self) -> HashSet<String> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.clone()
    }

    pub async fn send_message(&self, message: WebSocketMessage) -> Result<(), ClientError> {
        self.sender.send(message).map_err(ClientError::SendError)?;
        
        // Update activity
        let mut metadata = self.metadata.write().await;
        metadata.last_activity = Instant::now();
        metadata.message_count += 1;
        
        Ok(())
    }

    pub async fn send_error(&self, topic: String, error: String) -> Result<(), ClientError> {
        let response = ChannelResponse {
            status: ChannelStatus::Error,
            response: serde_json::json!({ "reason": error }),
        };

        let message = WebSocketMessage::new(
            topic,
            "phx_error".to_string(),
            serde_json::to_value(response).map_err(ClientError::SerializationError)?,
        );

        self.send_message(message).await
    }

    pub async fn send_heartbeat_reply(&self) -> Result<(), ClientError> {
        let message = WebSocketMessage::new(
            "phoenix".to_string(),
            "phx_reply".to_string(),
            serde_json::json!({ "status": "ok", "response": {} }),
        );

        self.send_message(message).await?;
        
        // Update heartbeat timestamp
        let mut last_heartbeat = self.last_heartbeat.write().await;
        *last_heartbeat = Instant::now();
        
        Ok(())
    }

    pub async fn update_heartbeat(&self) {
        let mut last_heartbeat = self.last_heartbeat.write().await;
        *last_heartbeat = Instant::now();
    }

    pub async fn is_alive(&self, timeout_duration: Duration) -> bool {
        let last_heartbeat = self.last_heartbeat.read().await;
        last_heartbeat.elapsed() < timeout_duration
    }

    pub async fn get_stats(&self) -> ClientStats {
        let metadata = self.metadata.read().await;
        let subscriptions = self.subscriptions.read().await;

        ClientStats {
            id: self.id,
            user_id: self.user_id,
            authenticated: self.authenticated,
            connected_duration: metadata.connected_at.elapsed(),
            last_activity: metadata.last_activity.elapsed(),
            message_count: metadata.message_count,
            subscribed_topics: subscriptions.len(),
            ip_address: metadata.ip_address.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClientStats {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub authenticated: bool,
    pub connected_duration: Duration,
    pub last_activity: Duration,
    pub message_count: u64,
    pub subscribed_topics: usize,
    pub ip_address: String,
}

/// Manages the connection lifecycle for a WebSocket client
pub struct ClientConnection {
    client: Arc<WebSocketClient>,
    websocket: WebSocket,
    receiver: mpsc::UnboundedReceiver<WebSocketMessage>,
    heartbeat_interval: Duration,
    connection_timeout: Duration,
}

impl ClientConnection {
    pub fn new(
        client: Arc<WebSocketClient>,
        websocket: WebSocket,
        receiver: mpsc::UnboundedReceiver<WebSocketMessage>,
    ) -> Self {
        Self {
            client,
            websocket,
            receiver,
            heartbeat_interval: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(60),
        }
    }

    pub async fn run(mut self) -> Result<(), ClientError> {
        info!("Starting client connection for {}", self.client.id);

        let mut heartbeat_timer = interval(self.heartbeat_interval);
        let mut cleanup_timer = interval(Duration::from_secs(30));

        loop {
            tokio::select! {
                // Handle incoming WebSocket messages
                message = self.websocket.next() => {
                    match message {
                        Some(Ok(msg)) => {
                            if let Err(e) = self.handle_websocket_message(msg).await {
                                error!("Error handling WebSocket message: {}", e);
                                break;
                            }
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                        None => {
                            info!("WebSocket connection closed by client");
                            break;
                        }
                    }
                }

                // Handle outgoing messages to WebSocket
                message = self.receiver.recv() => {
                    match message {
                        Some(msg) => {
                            if let Err(e) = self.send_to_websocket(msg).await {
                                error!("Error sending message to WebSocket: {}", e);
                                break;
                            }
                        }
                        None => {
                            info!("Message sender closed");
                            break;
                        }
                    }
                }

                // Handle heartbeat
                _ = heartbeat_timer.tick() => {
                    if !self.client.is_alive(self.connection_timeout).await {
                        warn!("Client {} heartbeat timeout", self.client.id);
                        break;
                    }
                }

                // Cleanup timer for connection maintenance
                _ = cleanup_timer.tick() => {
                    // Perform any periodic maintenance here
                    debug!("Connection maintenance for client {}", self.client.id);
                }
            }
        }

        info!("Client connection {} ended", self.client.id);
        Ok(())
    }

    async fn handle_websocket_message(
        &mut self,
        message: tokio_tungstenite::tungstenite::Message,
    ) -> Result<(), ClientError> {
        match message {
            tokio_tungstenite::tungstenite::Message::Text(text) => {
                let ws_message: WebSocketMessage = 
                    serde_json::from_str(&text).map_err(ClientError::DeserializationError)?;
                
                self.handle_channel_message(ws_message).await?;
            }
            tokio_tungstenite::tungstenite::Message::Binary(_) => {
                warn!("Binary messages not supported");
            }
            tokio_tungstenite::tungstenite::Message::Ping(data) => {
                self.websocket
                    .send(tokio_tungstenite::tungstenite::Message::Pong(data))
                    .await
                    .map_err(ClientError::WebSocketError)?;
                
                self.client.update_heartbeat().await;
            }
            tokio_tungstenite::tungstenite::Message::Pong(_) => {
                self.client.update_heartbeat().await;
            }
            tokio_tungstenite::tungstenite::Message::Close(_) => {
                info!("WebSocket close frame received");
                return Err(ClientError::ConnectionClosed);
            }
            _ => {}
        }

        Ok(())
    }

    async fn handle_channel_message(
        &mut self,
        message: WebSocketMessage,
    ) -> Result<(), ClientError> {
        debug!("Handling channel message: {:?}", message);

        match message.event.as_str() {
            "phx_join" => {
                self.client.subscribe(message.topic.clone()).await?;
                
                let response = ChannelResponse {
                    status: ChannelStatus::Ok,
                    response: serde_json::json!({}),
                };

                let reply = WebSocketMessage::new(
                    message.topic,
                    "phx_reply".to_string(),
                    serde_json::to_value(response).map_err(ClientError::SerializationError)?,
                ).with_ref(message.ref_id.unwrap_or_default());

                self.client.send_message(reply).await?;
            }
            "phx_leave" => {
                self.client.unsubscribe(&message.topic).await?;
                
                let response = ChannelResponse {
                    status: ChannelStatus::Ok,
                    response: serde_json::json!({}),
                };

                let reply = WebSocketMessage::new(
                    message.topic,
                    "phx_reply".to_string(),
                    serde_json::to_value(response).map_err(ClientError::SerializationError)?,
                ).with_ref(message.ref_id.unwrap_or_default());

                self.client.send_message(reply).await?;
            }
            "heartbeat" => {
                self.client.send_heartbeat_reply().await?;
            }
            _ => {
                // Forward other messages to the message handler
                debug!("Forwarding message to handler: {}", message.event);
            }
        }

        Ok(())
    }

    async fn send_to_websocket(
        &mut self,
        message: WebSocketMessage,
    ) -> Result<(), ClientError> {
        let text = serde_json::to_string(&message)
            .map_err(ClientError::SerializationError)?;

        self.websocket
            .send(tokio_tungstenite::tungstenite::Message::Text(text))
            .await
            .map_err(ClientError::WebSocketError)?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(serde_json::Error),
    
    #[error("Send error: {0}")]
    SendError(#[from] mpsc::error::SendError<WebSocketMessage>),
    
    #[error("Connection closed")]
    ConnectionClosed,
    
    #[error("Authentication required")]
    AuthenticationRequired,
    
    #[error("Invalid topic: {0}")]
    InvalidTopic(String),
    
    #[error("Permission denied")]
    PermissionDenied,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_client_creation() {
        let (sender, _receiver) = mpsc::unbounded_channel();
        let client = WebSocketClient::new(Uuid::new_v4(), sender);
        
        assert!(!client.authenticated);
        assert!(client.user_id.is_none());
        assert_eq!(client.get_subscriptions().await.len(), 0);
    }

    #[tokio::test]
    async fn test_client_subscription() {
        let (sender, _receiver) = mpsc::unbounded_channel();
        let client = WebSocketClient::new(Uuid::new_v4(), sender);
        
        let topic = "test:topic".to_string();
        client.subscribe(topic.clone()).await.unwrap();
        
        assert!(client.is_subscribed(&topic).await);
        assert_eq!(client.get_subscriptions().await.len(), 1);
        
        client.unsubscribe(&topic).await.unwrap();
        assert!(!client.is_subscribed(&topic).await);
        assert_eq!(client.get_subscriptions().await.len(), 0);
    }

    #[tokio::test]
    async fn test_client_authentication() {
        let (sender, _receiver) = mpsc::unbounded_channel();
        let mut client = WebSocketClient::new(Uuid::new_v4(), sender);
        
        let user_id = Uuid::new_v4();
        client.authenticate(user_id).await;
        
        assert!(client.authenticated);
        assert_eq!(client.user_id, Some(user_id));
    }
}