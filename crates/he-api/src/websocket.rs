//! WebSocket handler for real-time game updates

use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::{web, HttpRequest, HttpResponse, Result};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::game_server::{AppState, GameState};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    // Client -> Server
    Subscribe { client_id: String },
    Unsubscribe { client_id: String },
    Ping,

    // Server -> Client
    StateUpdate { state: GameState },
    ProcessStarted { process_id: Uuid },
    ProcessCompleted { process_id: Uuid },
    Error { message: String },
    Pong,
}

pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse> {
    ws::start(WsConnection::new(app_state), &req, stream)
}

pub struct WsConnection {
    app_state: web::Data<AppState>,
    last_heartbeat: Instant,
    update_counter: u64,
}

impl WsConnection {
    pub fn new(app_state: web::Data<AppState>) -> Self {
        Self {
            app_state,
            last_heartbeat: Instant::now(),
            update_counter: 0,
        }
    }

    fn send_state_update(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        let mut engine = self.app_state.engine.lock().unwrap();
        engine.update();
        let state = engine.get_state();

        let msg = WsMessage::StateUpdate { state };
        if let Ok(json) = serde_json::to_string(&msg) {
            ctx.text(json);
        }
    }

    fn heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(Duration::from_secs(5), |act, ctx| {
            if Instant::now().duration_since(act.last_heartbeat) > Duration::from_secs(10) {
                log::warn!("WebSocket heartbeat failed, disconnecting");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }

    fn start_update_loop(&self, ctx: &mut ws::WebsocketContext<Self>) {
        // Send state updates every 100ms for smooth real-time experience
        ctx.run_interval(Duration::from_millis(100), |act, ctx| {
            act.update_counter += 1;

            // Only send updates if there are active processes or every 10 updates (1 second)
            let should_update = {
                let engine = act.app_state.engine.lock().unwrap();
                let state = engine.get_state();
                !state.processes.is_empty() || act.update_counter % 10 == 0
            };

            if should_update {
                act.send_state_update(ctx);
            }
        });
    }
}

impl Actor for WsConnection {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);
        self.start_update_loop(ctx);
        self.send_state_update(ctx); // Send initial state
        log::info!("WebSocket connection established");
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        log::info!("WebSocket connection closed");
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.last_heartbeat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.last_heartbeat = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                    match ws_msg {
                        WsMessage::Subscribe { client_id } => {
                            log::info!("WebSocket client {} subscribed", client_id);
                            self.send_state_update(ctx);
                        }
                        WsMessage::Ping => {
                            let pong = serde_json::to_string(&WsMessage::Pong).unwrap();
                            ctx.text(pong);
                        }
                        _ => {}
                    }
                }
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}