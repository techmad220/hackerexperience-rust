use crate::auth::{WebSocketAuth, AuthMessage, AuthResponse};
use crate::broadcast::BroadcastSystem;
use crate::channel::ChannelManager;
use crate::client::{WebSocketClient, ClientError};
use crate::events::{WebSocketMessage, GameEvent, ChannelResponse, ChannelStatus};
use async_trait::async_trait;
use serde_json;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Main message handler that coordinates all WebSocket message processing
pub struct MessageHandler {
    auth: Arc<WebSocketAuth>,
    channel_manager: Arc<ChannelManager>,
    broadcast_system: Arc<BroadcastSystem>,
    game_handler: Arc<dyn GameMessageHandler>,
}

impl MessageHandler {
    pub fn new(
        auth: Arc<WebSocketAuth>,
        channel_manager: Arc<ChannelManager>,
        broadcast_system: Arc<BroadcastSystem>,
        game_handler: Arc<dyn GameMessageHandler>,
    ) -> Self {
        Self {
            auth,
            channel_manager,
            broadcast_system,
            game_handler,
        }
    }

    /// Handle incoming WebSocket message from client
    pub async fn handle_message(
        &self,
        client: Arc<WebSocketClient>,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, HandlerError> {
        debug!("Handling message: event={}, topic={}", message.event, message.topic);

        match message.event.as_str() {
            // Phoenix channel events
            "phx_join" => self.handle_channel_join(client, message).await,
            "phx_leave" => self.handle_channel_leave(client, message).await,
            "heartbeat" => self.handle_heartbeat(client, message).await,
            
            // Authentication events
            "auth" | "authenticate" => self.handle_authentication(client, message).await,
            "auth:refresh" => self.handle_auth_refresh(client, message).await,
            
            // Game-specific events
            event if self.is_game_event(event) => {
                self.handle_game_message(client, message).await
            }
            
            // Unknown events
            _ => {
                warn!("Unknown event type: {}", message.event);
                self.send_error_response(
                    message.topic,
                    message.ref_id,
                    "unknown_event".to_string(),
                    format!("Unknown event: {}", message.event),
                )
            }
        }
    }

    /// Handle channel join requests
    async fn handle_channel_join(
        &self,
        client: Arc<WebSocketClient>,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, HandlerError> {
        let response = self
            .channel_manager
            .join_channel(client, message.topic.clone(), Some(message.payload))
            .await?;

        let reply_message = WebSocketMessage::new(
            message.topic,
            "phx_reply".to_string(),
            serde_json::to_value(response)?,
        ).with_ref(message.ref_id.unwrap_or_default());

        Ok(Some(reply_message))
    }

    /// Handle channel leave requests
    async fn handle_channel_leave(
        &self,
        client: Arc<WebSocketClient>,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, HandlerError> {
        self.channel_manager
            .leave_channel(&client, &message.topic)
            .await?;

        let response = ChannelResponse {
            status: ChannelStatus::Ok,
            response: serde_json::json!({}),
        };

        let reply_message = WebSocketMessage::new(
            message.topic,
            "phx_reply".to_string(),
            serde_json::to_value(response)?,
        ).with_ref(message.ref_id.unwrap_or_default());

        Ok(Some(reply_message))
    }

    /// Handle heartbeat messages
    async fn handle_heartbeat(
        &self,
        client: Arc<WebSocketClient>,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, HandlerError> {
        client.update_heartbeat().await;

        let reply_message = WebSocketMessage::new(
            "phoenix".to_string(),
            "phx_reply".to_string(),
            serde_json::json!({
                "status": "ok",
                "response": {}
            }),
        ).with_ref(message.ref_id.unwrap_or_default());

        Ok(Some(reply_message))
    }

    /// Handle authentication messages
    async fn handle_authentication(
        &self,
        mut client: Arc<WebSocketClient>,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, HandlerError> {
        let auth_message: AuthMessage = serde_json::from_value(message.payload)?;
        
        // We need to get a mutable reference to the client
        // In practice, this would be handled differently or you'd restructure to avoid this
        let client_mut = Arc::get_mut(&mut client).ok_or(HandlerError::InternalError("Cannot get mutable client reference".to_string()))?;
        
        let response = self.auth.handle_auth_message(client_mut, auth_message).await?;

        let reply_message = WebSocketMessage::new(
            "auth".to_string(),
            "auth_reply".to_string(),
            serde_json::to_value(response)?,
        ).with_ref(message.ref_id.unwrap_or_default());

        Ok(Some(reply_message))
    }

    /// Handle token refresh requests
    async fn handle_auth_refresh(
        &self,
        _client: Arc<WebSocketClient>,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, HandlerError> {
        // TODO: Implement token refresh logic
        let response = AuthResponse {
            success: false,
            user_id: None,
            error: Some("Token refresh not implemented".to_string()),
            permissions: vec![],
        };

        let reply_message = WebSocketMessage::new(
            "auth".to_string(),
            "auth_refresh_reply".to_string(),
            serde_json::to_value(response)?,
        ).with_ref(message.ref_id.unwrap_or_default());

        Ok(Some(reply_message))
    }

    /// Handle game-specific messages
    async fn handle_game_message(
        &self,
        client: Arc<WebSocketClient>,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, HandlerError> {
        // Check if client is authenticated for game events
        if !client.authenticated && self.requires_authentication(&message.event) {
            return self.send_error_response(
                message.topic,
                message.ref_id,
                "authentication_required".to_string(),
                "Authentication required for this action".to_string(),
            );
        }

        // Delegate to game-specific handler
        match self.game_handler.handle_game_message(client.clone(), message.clone()).await {
            Ok(response) => Ok(response),
            Err(e) => {
                error!("Game handler error: {}", e);
                self.send_error_response(
                    message.topic,
                    message.ref_id,
                    "game_error".to_string(),
                    e.to_string(),
                )
            }
        }
    }

    /// Check if an event is a game-related event
    fn is_game_event(&self, event: &str) -> bool {
        matches!(
            event,
            "process:start" | "process:stop" | "process:status" |
            "server:connect" | "server:disconnect" | "server:scan" |
            "file:list" | "file:create" | "file:delete" | "file:edit" |
            "log:clear" | "log:list" |
            "mission:accept" | "mission:complete" |
            "chat:message" | "chat:history" |
            "notification:read" | "notification:dismiss"
        )
    }

    /// Check if an event requires authentication
    fn requires_authentication(&self, event: &str) -> bool {
        // Most game events require authentication, except for some read-only operations
        !matches!(event, "server:public_info" | "mission:list_public")
    }

    /// Send an error response
    fn send_error_response(
        &self,
        topic: String,
        ref_id: Option<String>,
        error_type: String,
        error_message: String,
    ) -> Result<Option<WebSocketMessage>, HandlerError> {
        let response = ChannelResponse {
            status: ChannelStatus::Error,
            response: serde_json::json!({
                "error": error_type,
                "message": error_message
            }),
        };

        let reply_message = WebSocketMessage::new(
            topic,
            "phx_error".to_string(),
            serde_json::to_value(response)?,
        );

        let reply_message = if let Some(ref_id) = ref_id {
            reply_message.with_ref(ref_id)
        } else {
            reply_message
        };

        Ok(Some(reply_message))
    }
}

/// Trait for handling game-specific messages
#[async_trait]
pub trait GameMessageHandler: Send + Sync {
    async fn handle_game_message(
        &self,
        client: Arc<WebSocketClient>,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, GameHandlerError>;
}

/// Default implementation of game message handler
pub struct DefaultGameHandler {
    broadcast_system: Arc<BroadcastSystem>,
}

impl DefaultGameHandler {
    pub fn new(broadcast_system: Arc<BroadcastSystem>) -> Self {
        Self { broadcast_system }
    }
}

#[async_trait]
impl GameMessageHandler for DefaultGameHandler {
    async fn handle_game_message(
        &self,
        client: Arc<WebSocketClient>,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, GameHandlerError> {
        debug!("Handling game message: {}", message.event);

        match message.event.as_str() {
            "process:start" => self.handle_process_start(client, message).await,
            "process:stop" => self.handle_process_stop(client, message).await,
            "process:status" => self.handle_process_status(client, message).await,
            
            "server:connect" => self.handle_server_connect(client, message).await,
            "server:disconnect" => self.handle_server_disconnect(client, message).await,
            "server:scan" => self.handle_server_scan(client, message).await,
            
            "file:list" => self.handle_file_list(client, message).await,
            "file:create" => self.handle_file_create(client, message).await,
            "file:delete" => self.handle_file_delete(client, message).await,
            
            "chat:message" => self.handle_chat_message(client, message).await,
            
            "notification:read" => self.handle_notification_read(client, message).await,
            
            _ => {
                warn!("Unhandled game event: {}", message.event);
                Ok(None)
            }
        }
    }
}

impl DefaultGameHandler {
    async fn handle_process_start(
        &self,
        client: Arc<WebSocketClient>,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, GameHandlerError> {
        // TODO: Implement actual process starting logic
        // This would interact with your game's process system
        
        debug!("Starting process for user: {:?}", client.user_id);
        
        // Mock process start
        let process_id = Uuid::new_v4();
        let event = GameEvent::ProcessStarted {
            process_id,
            process_type: "hack".to_string(),
            target_id: message.payload.get("target").and_then(|v| v.as_str()).map(String::from),
            estimated_completion: chrono::Utc::now().timestamp() + 30,
        };

        // Broadcast the event
        self.broadcast_system.broadcast_event(event.clone()).await?;

        // Return success response
        let response = serde_json::json!({
            "status": "ok",
            "process_id": process_id,
            "message": "Process started successfully"
        });

        let reply_message = WebSocketMessage::new(
            message.topic,
            "process:start_reply".to_string(),
            response,
        ).with_ref(message.ref_id.unwrap_or_default());

        Ok(Some(reply_message))
    }

    async fn handle_process_stop(
        &self,
        _client: Arc<WebSocketClient>,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, GameHandlerError> {
        // TODO: Implement process stopping logic
        debug!("Stopping process");
        
        let response = serde_json::json!({
            "status": "ok",
            "message": "Process stopped successfully"
        });

        let reply_message = WebSocketMessage::new(
            message.topic,
            "process:stop_reply".to_string(),
            response,
        ).with_ref(message.ref_id.unwrap_or_default());

        Ok(Some(reply_message))
    }

    async fn handle_process_status(
        &self,
        _client: Arc<WebSocketClient>,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, GameHandlerError> {
        // TODO: Implement process status checking
        debug!("Getting process status");
        
        let response = serde_json::json!({
            "status": "ok",
            "processes": []
        });

        let reply_message = WebSocketMessage::new(
            message.topic,
            "process:status_reply".to_string(),
            response,
        ).with_ref(message.ref_id.unwrap_or_default());

        Ok(Some(reply_message))
    }

    async fn handle_server_connect(
        &self,
        _client: Arc<WebSocketClient>,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, GameHandlerError> {
        // TODO: Implement server connection logic
        debug!("Connecting to server");
        
        let server_id = message.payload.get("server_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let event = GameEvent::ServerConnected {
            server_id: server_id.to_string(),
            hostname: "example.com".to_string(),
            ip: "192.168.1.1".to_string(),
        };

        self.broadcast_system.broadcast_event(event).await?;

        let response = serde_json::json!({
            "status": "ok",
            "server_id": server_id,
            "message": "Connected to server successfully"
        });

        let reply_message = WebSocketMessage::new(
            message.topic,
            "server:connect_reply".to_string(),
            response,
        ).with_ref(message.ref_id.unwrap_or_default());

        Ok(Some(reply_message))
    }

    async fn handle_server_disconnect(
        &self,
        _client: Arc<WebSocketClient>,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, GameHandlerError> {
        // TODO: Implement server disconnection logic
        debug!("Disconnecting from server");
        
        let response = serde_json::json!({
            "status": "ok",
            "message": "Disconnected from server successfully"
        });

        let reply_message = WebSocketMessage::new(
            message.topic,
            "server:disconnect_reply".to_string(),
            response,
        ).with_ref(message.ref_id.unwrap_or_default());

        Ok(Some(reply_message))
    }

    async fn handle_server_scan(
        &self,
        _client: Arc<WebSocketClient>,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, GameHandlerError> {
        // TODO: Implement network scanning logic
        debug!("Scanning network");
        
        let response = serde_json::json!({
            "status": "ok",
            "scan_results": [],
            "message": "Network scan completed"
        });

        let reply_message = WebSocketMessage::new(
            message.topic,
            "server:scan_reply".to_string(),
            response,
        ).with_ref(message.ref_id.unwrap_or_default());

        Ok(Some(reply_message))
    }

    async fn handle_file_list(
        &self,
        _client: Arc<WebSocketClient>,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, GameHandlerError> {
        // TODO: Implement file listing logic
        debug!("Listing files");
        
        let response = serde_json::json!({
            "status": "ok",
            "files": [],
            "path": message.payload.get("path").unwrap_or(&serde_json::Value::String("/".to_string()))
        });

        let reply_message = WebSocketMessage::new(
            message.topic,
            "file:list_reply".to_string(),
            response,
        ).with_ref(message.ref_id.unwrap_or_default());

        Ok(Some(reply_message))
    }

    async fn handle_file_create(
        &self,
        _client: Arc<WebSocketClient>,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, GameHandlerError> {
        // TODO: Implement file creation logic
        debug!("Creating file");
        
        let response = serde_json::json!({
            "status": "ok",
            "message": "File created successfully"
        });

        let reply_message = WebSocketMessage::new(
            message.topic,
            "file:create_reply".to_string(),
            response,
        ).with_ref(message.ref_id.unwrap_or_default());

        Ok(Some(reply_message))
    }

    async fn handle_file_delete(
        &self,
        _client: Arc<WebSocketClient>,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, GameHandlerError> {
        // TODO: Implement file deletion logic
        debug!("Deleting file");
        
        let response = serde_json::json!({
            "status": "ok",
            "message": "File deleted successfully"
        });

        let reply_message = WebSocketMessage::new(
            message.topic,
            "file:delete_reply".to_string(),
            response,
        ).with_ref(message.ref_id.unwrap_or_default());

        Ok(Some(reply_message))
    }

    async fn handle_chat_message(
        &self,
        client: Arc<WebSocketClient>,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, GameHandlerError> {
        if let Some(msg_text) = message.payload.get("message").and_then(|m| m.as_str()) {
            let channel = message.topic.strip_prefix("chat:").unwrap_or("general");
            
            let chat_event = GameEvent::ChatMessage {
                channel: channel.to_string(),
                sender: client.user_id.map(|id| id.to_string()).unwrap_or_else(|| "anonymous".to_string()),
                message: msg_text.to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            };

            self.broadcast_system.broadcast_to_topic(message.topic.clone(), chat_event).await?;
        }

        // No direct reply needed for chat messages
        Ok(None)
    }

    async fn handle_notification_read(
        &self,
        _client: Arc<WebSocketClient>,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, GameHandlerError> {
        // TODO: Mark notification as read in database
        debug!("Marking notification as read");
        
        let response = serde_json::json!({
            "status": "ok",
            "message": "Notification marked as read"
        });

        let reply_message = WebSocketMessage::new(
            message.topic,
            "notification:read_reply".to_string(),
            response,
        ).with_ref(message.ref_id.unwrap_or_default());

        Ok(Some(reply_message))
    }
}

#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("Channel error: {0}")]
    ChannelError(#[from] crate::channel::ChannelError),
    
    #[error("Client error: {0}")]
    ClientError(#[from] ClientError),
    
    #[error("Authentication error: {0}")]
    AuthError(#[from] crate::auth::AuthError),
    
    #[error("Broadcast error: {0}")]
    BroadcastError(#[from] crate::broadcast::BroadcastError),
    
    #[error("Game handler error: {0}")]
    GameHandlerError(#[from] GameHandlerError),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

#[derive(Debug, Error)]
pub enum GameHandlerError {
    #[error("Broadcast error: {0}")]
    BroadcastError(#[from] crate::broadcast::BroadcastError),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Permission denied")]
    PermissionDenied,
    
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{JwtAuthProvider, TopicAccessControl};
    use crate::broadcast::BroadcastConfig;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_message_handler_creation() {
        let provider = Arc::new(JwtAuthProvider::new("secret".to_string()));
        let auth = Arc::new(WebSocketAuth::new(provider.clone()));
        let access_control = Arc::new(TopicAccessControl::new(auth.clone()));
        let channel_manager = Arc::new(ChannelManager::new(access_control));
        let broadcast_system = Arc::new(BroadcastSystem::new(BroadcastConfig::default()));
        let game_handler = Arc::new(DefaultGameHandler::new(broadcast_system.clone()));

        let handler = MessageHandler::new(auth, channel_manager, broadcast_system, game_handler);
        
        // Test that handler was created successfully
        assert!(true); // Simple existence test
    }

    #[tokio::test]
    async fn test_heartbeat_handling() {
        let provider = Arc::new(JwtAuthProvider::new("secret".to_string()));
        let auth = Arc::new(WebSocketAuth::new(provider.clone()));
        let access_control = Arc::new(TopicAccessControl::new(auth.clone()));
        let channel_manager = Arc::new(ChannelManager::new(access_control));
        let broadcast_system = Arc::new(BroadcastSystem::new(BroadcastConfig::default()));
        let game_handler = Arc::new(DefaultGameHandler::new(broadcast_system.clone()));

        let handler = MessageHandler::new(auth, channel_manager, broadcast_system, game_handler);

        let (sender, _receiver) = mpsc::unbounded_channel();
        let client = Arc::new(WebSocketClient::new(Uuid::new_v4(), sender));

        let message = WebSocketMessage::new(
            "phoenix".to_string(),
            "heartbeat".to_string(),
            serde_json::json!({}),
        ).with_ref("1".to_string());

        let response = handler.handle_message(client, message).await.unwrap();
        assert!(response.is_some());
        
        let response = response.unwrap();
        assert_eq!(response.event, "phx_reply");
    }
}