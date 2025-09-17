//! WebSocket handler for real-time updates

use actix::{Actor, StreamHandler, AsyncContext, Handler, Message};
use actix_web::{web, HttpRequest, HttpResponse, Error};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Serialize, Deserialize)]
pub struct WSMessage {
    pub event_type: String,
    pub data: serde_json::Value,
}

pub struct WebSocketSession {
    hb: Instant,
    user_id: Option<i64>,
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

impl WebSocketSession {
    pub fn new() -> Self {
        Self {
            hb: Instant::now(),
            user_id: None,
        }
    }

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

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
                // Parse message
                if let Ok(msg) = serde_json::from_str::<WSMessage>(&text) {
                    match msg.event_type.as_str() {
                        "auth" => {
                            // Authenticate WebSocket connection
                            if let Some(token) = msg.data.get("token").and_then(|v| v.as_str()) {
                                // TODO: Validate token and set user_id
                                self.user_id = Some(1); // Placeholder

                                let response = WSMessage {
                                    event_type: "auth_success".to_string(),
                                    data: serde_json::json!({
                                        "message": "Authenticated successfully"
                                    }),
                                };
                                ctx.text(serde_json::to_string(&response).unwrap());
                            }
                        }
                        "subscribe" => {
                            // Subscribe to events
                            if let Some(event) = msg.data.get("event").and_then(|v| v.as_str()) {
                                let response = WSMessage {
                                    event_type: "subscribed".to_string(),
                                    data: serde_json::json!({
                                        "event": event,
                                        "message": format!("Subscribed to {}", event)
                                    }),
                                };
                                ctx.text(serde_json::to_string(&response).unwrap());
                            }
                        }
                        _ => {
                            // Unknown message type
                            let response = WSMessage {
                                event_type: "error".to_string(),
                                data: serde_json::json!({
                                    "message": "Unknown message type"
                                }),
                            };
                            ctx.text(serde_json::to_string(&response).unwrap());
                        }
                    }
                }
            }
            Ok(ws::Message::Binary(_bin)) => {
                // Handle binary messages if needed
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

// Message types for actor communication
#[derive(Message)]
#[rtype(result = "()")]
pub struct ProcessUpdate {
    pub user_id: i64,
    pub process_id: i64,
    pub status: String,
}

impl Handler<ProcessUpdate> for WebSocketSession {
    type Result = ();

    fn handle(&mut self, msg: ProcessUpdate, ctx: &mut Self::Context) {
        if Some(msg.user_id) == self.user_id {
            let update = WSMessage {
                event_type: "process_update".to_string(),
                data: serde_json::json!({
                    "process_id": msg.process_id,
                    "status": msg.status
                }),
            };
            ctx.text(serde_json::to_string(&update).unwrap());
        }
    }
}

pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    ws::start(WebSocketSession::new(), &req, stream)
}