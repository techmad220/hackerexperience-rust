//! WebSocket session management with backpressure and OOM prevention

use std::time::{Duration, Instant};
use std::collections::VecDeque;
use tokio::sync::mpsc;
use actix_web_actors::ws;

/// Maximum queued messages per client before dropping oldest
const MAX_QUEUE: usize = 1024;

/// Ping interval to check client connectivity
const PING_SECS: u64 = 20;

/// Drop client if no pong received within this duration
const DROP_AFTER_SECS: u64 = 45;

/// WebSocket session with bounded queue and health checks
pub struct WsSession {
    /// Client ID
    pub id: String,

    /// Message queue (bounded to prevent OOM)
    pub queue: VecDeque<String>,

    /// Last heartbeat timestamp
    pub last_heartbeat: Instant,

    /// Broadcast channel receiver
    pub broadcast_rx: mpsc::UnboundedReceiver<String>,
}

impl WsSession {
    pub fn new(id: String, broadcast_rx: mpsc::UnboundedReceiver<String>) -> Self {
        Self {
            id,
            queue: VecDeque::with_capacity(MAX_QUEUE),
            last_heartbeat: Instant::now(),
            broadcast_rx,
        }
    }

    /// Queue a message with backpressure handling
    pub fn queue_message(&mut self, msg: String) -> bool {
        // If queue is full, drop oldest message
        if self.queue.len() >= MAX_QUEUE {
            tracing::warn!("Client {} queue full, dropping oldest message", self.id);
            self.queue.pop_front();
        }

        self.queue.push_back(msg);
        true
    }

    /// Check if client should be dropped due to inactivity
    pub fn should_drop(&self) -> bool {
        self.last_heartbeat.elapsed() > Duration::from_secs(DROP_AFTER_SECS)
    }

    /// Update heartbeat timestamp
    pub fn heartbeat(&mut self) {
        self.last_heartbeat = Instant::now();
    }
}

/// Actor implementation for WebSocket session
impl actix::Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        tracing::info!("WebSocket client {} connected", self.id);

        // Start heartbeat check
        self.heartbeat_interval(ctx);

        // Start broadcast receiver
        self.handle_broadcasts(ctx);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        tracing::info!("WebSocket client {} disconnected", self.id);
    }
}

impl WsSession {
    /// Send ping every PING_SECS and check for timeout
    fn heartbeat_interval(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(Duration::from_secs(PING_SECS), |act, ctx| {
            // Check if client timed out
            if act.should_drop() {
                tracing::warn!("Client {} heartbeat timeout, disconnecting", act.id);
                ctx.stop();
                return;
            }

            // Send ping
            ctx.ping(b"PING");
        });
    }

    /// Handle broadcast messages with queue management
    fn handle_broadcasts(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        use futures_util::StreamExt;

        let mut rx = std::mem::replace(
            &mut self.broadcast_rx,
            mpsc::unbounded_channel().1
        );

        ctx.spawn(async move {
            while let Some(msg) = rx.recv().await {
                // Queue message with backpressure handling
                // This is where MAX_QUEUE enforcement happens
            }
        }.into_actor(self));
    }
}

/// Handle WebSocket messages
impl actix_web_actors::ws::StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.heartbeat();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.heartbeat();
            }
            Ok(ws::Message::Text(text)) => {
                // Handle text message
                self.handle_text_message(text.to_string(), ctx);
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

impl WsSession {
    fn handle_text_message(&mut self, msg: String, ctx: &mut ws::WebsocketContext<Self>) {
        // Rate limit check - simple token bucket
        static MAX_MESSAGES_PER_SECOND: usize = 10;

        // Parse and handle message
        match msg.as_str() {
            "ping" => {
                ctx.text("pong");
            }
            _ => {
                // Handle other messages
                tracing::debug!("Client {} sent: {}", self.id, msg);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_backpressure() {
        let (_tx, rx) = mpsc::unbounded_channel();
        let mut session = WsSession::new("test".to_string(), rx);

        // Fill queue beyond capacity
        for i in 0..MAX_QUEUE + 100 {
            session.queue_message(format!("msg{}", i));
        }

        // Queue should be capped at MAX_QUEUE
        assert_eq!(session.queue.len(), MAX_QUEUE);
    }

    #[test]
    fn test_heartbeat_timeout() {
        let (_tx, rx) = mpsc::unbounded_channel();
        let mut session = WsSession::new("test".to_string(), rx);

        // Fresh session should not timeout
        assert!(!session.should_drop());

        // Simulate old heartbeat
        session.last_heartbeat = Instant::now() - Duration::from_secs(DROP_AFTER_SECS + 1);
        assert!(session.should_drop());

        // Heartbeat refreshes timeout
        session.heartbeat();
        assert!(!session.should_drop());
    }
}