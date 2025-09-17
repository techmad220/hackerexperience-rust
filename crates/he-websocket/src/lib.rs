//! WebSocket real-time communication system

use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, StreamHandler};
use actix_web_actors::ws;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub mod events;
pub mod manager;

#[cfg(test)]
mod tests;

pub use events::*;
pub use manager::*;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

/// WebSocket session actor
pub struct WebSocketSession {
    /// Unique session ID
    pub id: Uuid,
    /// User ID if authenticated
    pub user_id: Option<i64>,
    /// Last heartbeat time
    pub hb: Instant,
    /// Connection manager
    pub manager: Arc<ConnectionManager>,
    /// Subscribed event channels
    pub subscriptions: Vec<String>,
}

impl WebSocketSession {
    pub fn new(manager: Arc<ConnectionManager>) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: None,
            hb: Instant::now(),
            manager,
            subscriptions: Vec::new(),
        }
    }

    /// Send heartbeat ping
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                warn!("WebSocket client timeout, disconnecting");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }

    /// Handle authentication
    async fn authenticate(&mut self, token: &str) -> Result<i64, String> {
        // TODO: Validate JWT and get user_id
        // For now, return mock user_id
        Ok(1)
    }
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        self.manager.register_connection(self.id, ctx.address());
        info!("WebSocket session {} started", self.id);
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        self.manager.unregister_connection(self.id);
        info!("WebSocket session {} stopped", self.id);
    }
}

/// Handle incoming WebSocket messages
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                if let Ok(msg) = serde_json::from_str::<ClientMessage>(&text) {
                    ctx.address().do_send(msg);
                }
            }
            Ok(ws::Message::Binary(_)) => {}
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

/// Client -> Server message
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub msg_type: String,
    pub data: serde_json::Value,
}

impl Handler<ClientMessage> for WebSocketSession {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, ctx: &mut Self::Context) {
        match msg.msg_type.as_str() {
            "auth" => {
                if let Some(token) = msg.data.get("token").and_then(|v| v.as_str()) {
                    let token = token.to_string();
                    let addr = ctx.address();
                    let session_id = self.id;
                    let manager = self.manager.clone();

                    ctx.spawn(
                        async move {
                            // Authenticate and get user_id
                            match WebSocketSession::authenticate(&mut Default::default(), &token).await {
                                Ok(user_id) => {
                                    manager.authenticate_connection(session_id, user_id);
                                    addr.do_send(ServerMessage {
                                        event_type: "auth_success".to_string(),
                                        data: serde_json::json!({
                                            "user_id": user_id,
                                            "message": "Authentication successful"
                                        }),
                                    });
                                }
                                Err(e) => {
                                    addr.do_send(ServerMessage {
                                        event_type: "auth_error".to_string(),
                                        data: serde_json::json!({
                                            "error": e
                                        }),
                                    });
                                }
                            }
                        }
                        .into_actor(self),
                    );
                }
            }
            "subscribe" => {
                if let Some(channel) = msg.data.get("channel").and_then(|v| v.as_str()) {
                    self.subscriptions.push(channel.to_string());
                    ctx.text(
                        serde_json::to_string(&ServerMessage {
                            event_type: "subscribed".to_string(),
                            data: serde_json::json!({
                                "channel": channel
                            }),
                        })
                        .unwrap(),
                    );
                }
            }
            "unsubscribe" => {
                if let Some(channel) = msg.data.get("channel").and_then(|v| v.as_str()) {
                    self.subscriptions.retain(|c| c != channel);
                    ctx.text(
                        serde_json::to_string(&ServerMessage {
                            event_type: "unsubscribed".to_string(),
                            data: serde_json::json!({
                                "channel": channel
                            }),
                        })
                        .unwrap(),
                    );
                }
            }
            _ => {
                warn!("Unknown message type: {}", msg.msg_type);
            }
        }
    }
}

/// Server -> Client message
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[rtype(result = "()")]
pub struct ServerMessage {
    pub event_type: String,
    pub data: serde_json::Value,
}

impl Handler<ServerMessage> for WebSocketSession {
    type Result = ();

    fn handle(&mut self, msg: ServerMessage, ctx: &mut Self::Context) {
        ctx.text(serde_json::to_string(&msg).unwrap());
    }
}

/// Broadcast message to all connections
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct Broadcast {
    pub event_type: String,
    pub data: serde_json::Value,
    pub user_filter: Option<Vec<i64>>,
}

impl Handler<Broadcast> for WebSocketSession {
    type Result = ();

    fn handle(&mut self, msg: Broadcast, ctx: &mut Self::Context) {
        // Check if this session should receive the broadcast
        if let Some(filter) = &msg.user_filter {
            if let Some(user_id) = self.user_id {
                if !filter.contains(&user_id) {
                    return;
                }
            } else {
                return;
            }
        }

        ctx.text(
            serde_json::to_string(&ServerMessage {
                event_type: msg.event_type,
                data: msg.data,
            })
            .unwrap(),
        );
    }
}