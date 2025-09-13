//! Helix WebSocket handlers

pub mod join;
pub mod request;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum WebSocketError {
    #[error("Invalid request: {message}")]
    InvalidRequest { message: String },
    #[error("Channel not found: {channel}")]
    ChannelNotFound { channel: String },
    #[error("Permission denied")]
    PermissionDenied,
}

pub type WebSocketResult<T> = Result<T, WebSocketError>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSocketRequest {
    Join { channel: String, entity_id: Uuid },
    Leave { channel: String },
    Ping,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSocketResponse {
    Joined { channel: String, success: bool },
    Left { channel: String },
    Pong,
    Error { message: String },
}

/// Handle WebSocket requests
pub async fn handle_request(request: WebSocketRequest) -> WebSocketResult<WebSocketResponse> {
    match request {
        WebSocketRequest::Join { channel, entity_id } => {
            tracing::info!("Entity {} joining channel {}", entity_id, channel);
            Ok(WebSocketResponse::Joined { channel, success: true })
        }
        WebSocketRequest::Leave { channel } => {
            tracing::info!("Leaving channel {}", channel);
            Ok(WebSocketResponse::Left { channel })
        }
        WebSocketRequest::Ping => Ok(WebSocketResponse::Pong),
    }
}