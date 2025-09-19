//! WebSocket connection manager

use actix::Addr;
use dashmap::DashMap;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

use crate::{Broadcast, ServerMessage, WebSocketSession};

/// Manages all WebSocket connections
#[derive(Clone)]
pub struct ConnectionManager {
    /// All active connections
    connections: Arc<DashMap<Uuid, ConnectionInfo>>,
    /// User ID to connection mapping
    user_connections: Arc<DashMap<i64, Vec<Uuid>>>,
}

#[derive(Clone)]
struct ConnectionInfo {
    session_id: Uuid,
    user_id: Option<i64>,
    addr: Addr<WebSocketSession>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
            user_connections: Arc::new(DashMap::new()),
        }
    }

    /// Register a new connection
    pub fn register_connection(&self, session_id: Uuid, addr: Addr<WebSocketSession>) {
        info!("Registering connection: {}", session_id);
        self.connections.insert(
            session_id,
            ConnectionInfo {
                session_id,
                user_id: None,
                addr,
            },
        );
    }

    /// Unregister a connection
    pub fn unregister_connection(&self, session_id: Uuid) {
        info!("Unregistering connection: {}", session_id);

        // Remove from user_connections if authenticated
        if let Some((_, conn_info)) = self.connections.remove(&session_id) {
            if let Some(user_id) = conn_info.user_id {
                if let Some(mut user_conns) = self.user_connections.get_mut(&user_id) {
                    user_conns.retain(|id| *id != session_id);
                    if user_conns.is_empty() {
                        drop(user_conns);
                        self.user_connections.remove(&user_id);
                    }
                }
            }
        }
    }

    /// Authenticate a connection
    pub fn authenticate_connection(&self, session_id: Uuid, user_id: i64) {
        info!("Authenticating connection {} for user {}", session_id, user_id);

        if let Some(mut conn_info) = self.connections.get_mut(&session_id) {
            conn_info.user_id = Some(user_id);

            // Add to user_connections
            self.user_connections
                .entry(user_id)
                .or_insert_with(Vec::new)
                .push(session_id);
        }
    }

    /// Send message to specific user
    pub fn send_to_user(&self, user_id: i64, message: ServerMessage) {
        if let Some(session_ids) = self.user_connections.get(&user_id) {
            for session_id in session_ids.iter() {
                if let Some(conn_info) = self.connections.get(session_id) {
                    conn_info.addr.do_send(message.clone());
                }
            }
        }
    }

    /// Send message to specific session
    pub fn send_to_session(&self, session_id: Uuid, message: ServerMessage) {
        if let Some(conn_info) = self.connections.get(&session_id) {
            conn_info.addr.do_send(message);
        }
    }

    /// Broadcast to all connected users
    pub fn broadcast_all(&self, message: ServerMessage) {
        for conn in self.connections.iter() {
            conn.addr.do_send(message.clone());
        }
    }

    /// Broadcast to specific users
    pub fn broadcast_to_users(&self, user_ids: Vec<i64>, message: ServerMessage) {
        for user_id in user_ids {
            self.send_to_user(user_id, message.clone());
        }
    }

    /// Get online users count
    pub fn online_users_count(&self) -> usize {
        self.user_connections.len()
    }

    /// Get total connections count
    pub fn total_connections(&self) -> usize {
        self.connections.len()
    }

    /// Check if user is online
    pub fn is_user_online(&self, user_id: i64) -> bool {
        self.user_connections.contains_key(&user_id)
    }

    /// Get all online user IDs
    pub fn get_online_users(&self) -> Vec<i64> {
        self.user_connections.iter().map(|entry| *entry.key()).collect()
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}