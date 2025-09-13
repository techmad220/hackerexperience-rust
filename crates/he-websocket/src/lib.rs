//! # HackerExperience WebSocket Library
//! 
//! A comprehensive WebSocket real-time system for HackerExperience, providing Phoenix channels-like
//! functionality with Rust and tokio. This crate handles real-time communication between the game
//! server and clients, including authentication, channel management, broadcasting, and message handling.
//! 
//! ## Features
//! 
//! - **WebSocket Server**: High-performance WebSocket server using tokio-tungstenite
//! - **Channel System**: Phoenix channels-inspired topic-based communication
//! - **Authentication**: JWT-based authentication with session management  
//! - **Broadcasting**: Efficient event broadcasting to subscribers
//! - **Client Management**: Connection lifecycle management with heartbeat monitoring
//! - **Game Events**: Comprehensive game event system for real-time updates
//! - **Topic Subscriptions**: Flexible subscription model for different game areas
//! 
//! ## Architecture
//! 
//! The library is organized into several key components:
//! 
//! - [`server`]: WebSocket server implementation
//! - [`client`]: Client connection management
//! - [`channel`]: Channel/topic management system
//! - [`auth`]: Authentication and authorization
//! - [`broadcast`]: Event broadcasting system
//! - [`handler`]: Message processing and routing
//! - [`events`]: Event type definitions
//! 
//! ## Quick Start
//! 
//! ```rust,no_run
//! use he_websocket::{WebSocketServer, ServerConfig};
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create server configuration
//!     let config = ServerConfig {
//!         bind_address: "127.0.0.1:4000".parse()?,
//!         jwt_secret: "your-secret-key".to_string(),
//!         max_connections: 1000,
//!         ..Default::default()
//!     };
//! 
//!     // Create and start the WebSocket server
//!     let mut server = WebSocketServer::new(config);
//!     server.start().await?;
//! 
//!     Ok(())
//! }
//! ```
//! 
//! ## Event System
//! 
//! The library provides a comprehensive event system for game-related real-time updates:
//! 
//! ```rust
//! use he_websocket::events::{GameEvent, WebSocketMessage};
//! 
//! // Create a game event
//! let event = GameEvent::ProcessStarted {
//!     process_id: uuid::Uuid::new_v4(),
//!     process_type: "hack".to_string(),
//!     target_id: Some("192.168.1.1".to_string()),
//!     estimated_completion: 1234567890,
//! };
//! 
//! // Convert to WebSocket message
//! let message = WebSocketMessage::from_game_event(
//!     "user:123".to_string(),
//!     event
//! )?;
//! ```
//! 
//! ## Channel Topics
//! 
//! The system supports various topic patterns for organizing communications:
//! 
//! - `user:{user_id}` - User-specific notifications
//! - `server:{server_id}` - Server-specific events
//! - `process:{process_id}` - Process-specific updates
//! - `chat:{channel}` - Chat channels
//! - `lobby:global` - Global lobby
//! - `system:announcements` - System-wide announcements
//! 
//! ## Authentication
//! 
//! Clients authenticate using JWT tokens:
//! 
//! ```rust
//! use he_websocket::auth::AuthMessage;
//! 
//! let auth_message = AuthMessage::Authenticate {
//!     token: "jwt-token-here".to_string(),
//! };
//! ```

pub mod auth;
pub mod broadcast;
pub mod channel;
pub mod client;
pub mod events;
pub mod handler;
pub mod server;

// Re-export main types for convenience
pub use auth::{AuthProvider, JwtAuthProvider, WebSocketAuth, AuthMessage, AuthResponse};
pub use broadcast::{BroadcastSystem, BroadcastConfig, BroadcastEvent, EventPriority};
pub use channel::{ChannelManager, ChannelHandler, ChannelConfig, ChannelStats};
pub use client::{WebSocketClient, ClientConnection, ClientStats, ClientError};
pub use events::{
    GameEvent, WebSocketMessage, ChannelEvent, ChannelResponse, ChannelStatus,
    ProcessResult, FileInfo, MissionRewards, topics,
};
pub use handler::{MessageHandler, GameMessageHandler, DefaultGameHandler};
pub use server::{WebSocketServer, ServerConfig, ServerStats, ServerError};

/// Result type for WebSocket operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize tracing for the WebSocket library
/// 
/// This should be called once at the start of your application to set up logging.
/// 
/// # Example
/// 
/// ```rust
/// use he_websocket::init_tracing;
/// 
/// #[tokio::main]
/// async fn main() {
///     init_tracing();
///     // ... rest of your application
/// }
/// ```
pub fn init_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "he_websocket=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Create a default WebSocket server with reasonable defaults
/// 
/// This is a convenience function for quickly setting up a WebSocket server
/// with default configuration.
/// 
/// # Arguments
/// 
/// * `bind_address` - The address to bind the server to
/// * `jwt_secret` - Secret key for JWT authentication
/// 
/// # Example
/// 
/// ```rust,no_run
/// use he_websocket::create_default_server;
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut server = create_default_server(
///         "127.0.0.1:4000".parse()?,
///         "your-secret-key".to_string()
///     );
///     
///     server.start().await?;
///     Ok(())
/// }
/// ```
pub fn create_default_server(
    bind_address: std::net::SocketAddr,
    jwt_secret: String,
) -> WebSocketServer {
    let config = ServerConfig {
        bind_address,
        jwt_secret,
        ..Default::default()
    };
    
    WebSocketServer::new(config)
}

/// Utility functions for working with WebSocket messages
pub mod utils {
    use crate::events::{WebSocketMessage, GameEvent};
    use serde_json::Value;
    
    /// Create a success response message
    pub fn create_success_response(
        topic: String,
        event: String,
        data: Value,
        ref_id: Option<String>,
    ) -> WebSocketMessage {
        let payload = serde_json::json!({
            "status": "ok",
            "data": data
        });
        
        let mut message = WebSocketMessage::new(topic, event, payload);
        if let Some(ref_id) = ref_id {
            message = message.with_ref(ref_id);
        }
        
        message
    }
    
    /// Create an error response message
    pub fn create_error_response(
        topic: String,
        event: String,
        error_code: String,
        error_message: String,
        ref_id: Option<String>,
    ) -> WebSocketMessage {
        let payload = serde_json::json!({
            "status": "error",
            "error": {
                "code": error_code,
                "message": error_message
            }
        });
        
        let mut message = WebSocketMessage::new(topic, event, payload);
        if let Some(ref_id) = ref_id {
            message = message.with_ref(ref_id);
        }
        
        message
    }
    
    /// Validate a topic name according to Phoenix channels conventions
    pub fn validate_topic(topic: &str) -> bool {
        // Topics should follow the pattern: namespace:identifier
        // Examples: user:123, server:main, chat:general
        
        if topic.is_empty() || topic.len() > 255 {
            return false;
        }
        
        // Must contain exactly one colon
        let parts: Vec<&str> = topic.split(':').collect();
        if parts.len() != 2 {
            return false;
        }
        
        let namespace = parts[0];
        let identifier = parts[1];
        
        // Namespace and identifier must not be empty
        if namespace.is_empty() || identifier.is_empty() {
            return false;
        }
        
        // Check for valid characters (alphanumeric, underscore, hyphen)
        let is_valid_char = |c: char| c.is_alphanumeric() || c == '_' || c == '-';
        
        namespace.chars().all(is_valid_char) && identifier.chars().all(is_valid_char)
    }
    
    /// Extract user ID from user-specific topics
    pub fn extract_user_id_from_topic(topic: &str) -> Option<uuid::Uuid> {
        if let Some(id_str) = topic.strip_prefix("user:") {
            uuid::Uuid::parse_str(id_str).ok()
        } else {
            None
        }
    }
    
    /// Check if a topic is public (accessible without authentication)
    pub fn is_public_topic(topic: &str) -> bool {
        matches!(topic, "lobby:global" | "system:announcements")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_constant() {
        assert!(!VERSION.is_empty());
    }
    
    #[test]
    fn test_topic_validation() {
        assert!(utils::validate_topic("user:123"));
        assert!(utils::validate_topic("server:main"));
        assert!(utils::validate_topic("chat:general"));
        assert!(utils::validate_topic("process:abc-def_123"));
        
        assert!(!utils::validate_topic(""));
        assert!(!utils::validate_topic("invalid"));
        assert!(!utils::validate_topic("too:many:colons"));
        assert!(!utils::validate_topic(":empty_namespace"));
        assert!(!utils::validate_topic("empty_identifier:"));
        assert!(!utils::validate_topic("invalid:chars!"));
    }
    
    #[test]
    fn test_user_id_extraction() {
        let user_id = uuid::Uuid::new_v4();
        let topic = format!("user:{}", user_id);
        
        assert_eq!(utils::extract_user_id_from_topic(&topic), Some(user_id));
        assert_eq!(utils::extract_user_id_from_topic("server:main"), None);
        assert_eq!(utils::extract_user_id_from_topic("user:invalid"), None);
    }
    
    #[test]
    fn test_public_topic_check() {
        assert!(utils::is_public_topic("lobby:global"));
        assert!(utils::is_public_topic("system:announcements"));
        assert!(!utils::is_public_topic("user:123"));
        assert!(!utils::is_public_topic("server:main"));
    }
    
    #[test]
    fn test_message_creation() {
        let success_msg = utils::create_success_response(
            "test:topic".to_string(),
            "test_reply".to_string(),
            serde_json::json!({"result": "ok"}),
            Some("ref123".to_string()),
        );
        
        assert_eq!(success_msg.topic, "test:topic");
        assert_eq!(success_msg.event, "test_reply");
        assert_eq!(success_msg.ref_id, Some("ref123".to_string()));
        
        let error_msg = utils::create_error_response(
            "test:topic".to_string(),
            "test_error".to_string(),
            "VALIDATION_ERROR".to_string(),
            "Invalid input".to_string(),
            None,
        );
        
        assert_eq!(error_msg.topic, "test:topic");
        assert_eq!(error_msg.event, "test_error");
        assert_eq!(error_msg.ref_id, None);
    }
}