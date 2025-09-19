use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::middleware::auth::Claims;

#[derive(Debug, Clone)]
pub struct WebSocketState {
    pub connections: Arc<RwLock<dashmap::DashMap<i64, tokio::sync::mpsc::Sender<String>>>>,
    pub jwt_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "auth")]
    Auth { token: String },

    #[serde(rename = "ping")]
    Ping,

    #[serde(rename = "pong")]
    Pong,

    #[serde(rename = "message")]
    Message { content: String },

    #[serde(rename = "game_action")]
    GameAction { action: String, data: serde_json::Value },

    #[serde(rename = "error")]
    Error { message: String },

    #[serde(rename = "authenticated")]
    Authenticated { user_id: i64, username: String },

    #[serde(rename = "notification")]
    Notification { title: String, message: String },
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<WebSocketState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<WebSocketState>) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);

    // Task to send messages to the client
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Authentication state
    let mut user_id: Option<i64> = None;
    let mut authenticated = false;

    // Handle incoming messages
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    match serde_json::from_str::<WebSocketMessage>(&text) {
                        Ok(ws_msg) => {
                            match ws_msg {
                                WebSocketMessage::Auth { token } => {
                                    // Validate JWT token
                                    let validation = Validation::new(Algorithm::HS256);
                                    let key = DecodingKey::from_secret(state.jwt_secret.as_bytes());

                                    match decode::<Claims>(&token, &key, &validation) {
                                        Ok(token_data) => {
                                            user_id = Some(token_data.claims.sub);
                                            authenticated = true;

                                            // Store connection
                                            state.connections.write().await.insert(
                                                token_data.claims.sub,
                                                tx.clone(),
                                            );

                                            // Send authenticated response
                                            let response = WebSocketMessage::Authenticated {
                                                user_id: token_data.claims.sub,
                                                username: token_data.claims.username,
                                            };

                                            let _ = tx
                                                .send(serde_json::to_string(&response).unwrap())
                                                .await;

                                            info!("WebSocket authenticated for user {}", token_data.claims.sub);
                                        }
                                        Err(e) => {
                                            let error = WebSocketMessage::Error {
                                                message: format!("Authentication failed: {}", e),
                                            };
                                            let _ = tx.send(serde_json::to_string(&error).unwrap()).await;
                                            break;
                                        }
                                    }
                                }
                                WebSocketMessage::Ping => {
                                    if authenticated {
                                        let pong = WebSocketMessage::Pong;
                                        let _ = tx.send(serde_json::to_string(&pong).unwrap()).await;
                                    } else {
                                        let error = WebSocketMessage::Error {
                                            message: "Not authenticated".to_string(),
                                        };
                                        let _ = tx.send(serde_json::to_string(&error).unwrap()).await;
                                    }
                                }
                                WebSocketMessage::GameAction { action, data } => {
                                    if authenticated {
                                        // Handle game actions
                                        match action.as_str() {
                                            "start_process" => {
                                                // Handle starting a process
                                                info!("Starting process for user {:?}: {:?}", user_id, data);
                                            }
                                            "cancel_process" => {
                                                // Handle canceling a process
                                                info!("Canceling process for user {:?}: {:?}", user_id, data);
                                            }
                                            "hack_server" => {
                                                // Handle hacking action
                                                info!("Hacking action for user {:?}: {:?}", user_id, data);
                                            }
                                            _ => {
                                                warn!("Unknown game action: {}", action);
                                            }
                                        }
                                    } else {
                                        let error = WebSocketMessage::Error {
                                            message: "Not authenticated".to_string(),
                                        };
                                        let _ = tx.send(serde_json::to_string(&error).unwrap()).await;
                                    }
                                }
                                _ => {
                                    if !authenticated {
                                        let error = WebSocketMessage::Error {
                                            message: "Please authenticate first".to_string(),
                                        };
                                        let _ = tx.send(serde_json::to_string(&error).unwrap()).await;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse WebSocket message: {}", e);
                            let error = WebSocketMessage::Error {
                                message: "Invalid message format".to_string(),
                            };
                            let _ = tx.send(serde_json::to_string(&error).unwrap()).await;
                        }
                    }
                }
                Message::Binary(_) => {
                    warn!("Binary messages not supported");
                }
                Message::Close(_) => {
                    break;
                }
                _ => {}
            }
        }

        // Clean up connection on disconnect
        if let Some(uid) = user_id {
            state.connections.write().await.remove(&uid);
            info!("WebSocket disconnected for user {}", uid);
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
}

/// Broadcast a message to specific user
pub async fn broadcast_to_user(
    state: &WebSocketState,
    user_id: i64,
    message: WebSocketMessage,
) -> Result<(), anyhow::Error> {
    let connections = state.connections.read().await;
    if let Some(tx) = connections.get(&user_id) {
        tx.send(serde_json::to_string(&message)?).await?;
    }
    Ok(())
}

/// Broadcast a message to all connected users
pub async fn broadcast_to_all(
    state: &WebSocketState,
    message: WebSocketMessage,
) -> Result<(), anyhow::Error> {
    let connections = state.connections.read().await;
    let message_str = serde_json::to_string(&message)?;

    for tx in connections.iter() {
        let _ = tx.send(message_str.clone()).await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_message_serialization() {
        let msg = WebSocketMessage::Ping;
        let serialized = serde_json::to_string(&msg).unwrap();
        assert_eq!(serialized, r#"{"type":"ping"}"#);

        let msg = WebSocketMessage::Auth {
            token: "test_token".to_string(),
        };
        let serialized = serde_json::to_string(&msg).unwrap();
        assert!(serialized.contains("auth"));
        assert!(serialized.contains("test_token"));
    }

    #[test]
    fn test_websocket_message_deserialization() {
        let json = r#"{"type":"ping"}"#;
        let msg: WebSocketMessage = serde_json::from_str(json).unwrap();
        matches!(msg, WebSocketMessage::Ping);

        let json = r#"{"type":"auth","token":"test_token"}"#;
        let msg: WebSocketMessage = serde_json::from_str(json).unwrap();
        if let WebSocketMessage::Auth { token } = msg {
            assert_eq!(token, "test_token");
        } else {
            panic!("Expected Auth message");
        }
    }
}