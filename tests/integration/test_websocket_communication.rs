use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade}, 
        State, ConnectInfo,
    },
    http::StatusCode,
    response::Response,
    routing::get,
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, RwLock, Mutex},
    time::{timeout, sleep},
};
use tokio_tungstenite::{
    connect_async, accept_async, 
    tungstenite::{Message as WsMessage, Error as WsError},
    WebSocketStream,
};
use futures_util::{SinkExt, StreamExt};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::common::{TestDb, TestFixtures};
use crate::{assert_ok, assert_err};

// ===== WEBSOCKET MESSAGE TYPES =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsClientMessage {
    pub id: Option<String>,
    pub message_type: String,
    pub payload: Value,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsServerMessage {
    pub id: String,
    pub message_type: String,
    pub payload: Value,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsAuthMessage {
    pub token: String,
    pub user_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsProcessUpdate {
    pub process_id: u64,
    pub progress: f32,
    pub status: String,
    pub remaining_time: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsNotification {
    pub title: String,
    pub body: String,
    pub notification_type: String,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsGameEvent {
    pub event_type: String,
    pub source_user: Option<u64>,
    pub target_user: Option<u64>,
    pub data: Value,
}

// ===== WEBSOCKET CONNECTION MANAGER =====

#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    pub connection_id: Uuid,
    pub user_id: Option<u64>,
    pub connected_at: DateTime<Utc>,
    pub last_ping: DateTime<Utc>,
    pub message_sender: mpsc::UnboundedSender<WsServerMessage>,
}

pub struct WebSocketManager {
    connections: Arc<RwLock<HashMap<Uuid, WebSocketConnection>>>,
    user_connections: Arc<RwLock<HashMap<u64, Vec<Uuid>>>>,
    message_history: Arc<RwLock<Vec<WsServerMessage>>>,
    stats: Arc<RwLock<WebSocketStats>>,
}

#[derive(Debug, Default)]
pub struct WebSocketStats {
    pub total_connections: u64,
    pub active_connections: usize,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub authentication_attempts: u64,
    pub failed_authentications: u64,
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            user_connections: Arc::new(RwLock::new(HashMap::new())),
            message_history: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(WebSocketStats::default())),
        }
    }

    pub async fn add_connection(&self, connection_id: Uuid, message_sender: mpsc::UnboundedSender<WsServerMessage>) {
        let connection = WebSocketConnection {
            connection_id,
            user_id: None,
            connected_at: Utc::now(),
            last_ping: Utc::now(),
            message_sender,
        };

        let mut connections = self.connections.write().await;
        connections.insert(connection_id, connection);

        let mut stats = self.stats.write().await;
        stats.total_connections += 1;
        stats.active_connections = connections.len();
    }

    pub async fn authenticate_connection(&self, connection_id: Uuid, user_id: u64) -> bool {
        let mut stats = self.stats.write().await;
        stats.authentication_attempts += 1;

        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(&connection_id) {
            connection.user_id = Some(user_id);

            let mut user_connections = self.user_connections.write().await;
            user_connections.entry(user_id).or_insert_with(Vec::new).push(connection_id);

            true
        } else {
            stats.failed_authentications += 1;
            false
        }
    }

    pub async fn remove_connection(&self, connection_id: Uuid) {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.remove(&connection_id) {
            if let Some(user_id) = connection.user_id {
                let mut user_connections = self.user_connections.write().await;
                if let Some(user_conns) = user_connections.get_mut(&user_id) {
                    user_conns.retain(|&id| id != connection_id);
                    if user_conns.is_empty() {
                        user_connections.remove(&user_id);
                    }
                }
            }

            let mut stats = self.stats.write().await;
            stats.active_connections = connections.len();
        }
    }

    pub async fn ping_connection(&self, connection_id: Uuid) {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(&connection_id) {
            connection.last_ping = Utc::now();
        }
    }

    pub async fn send_message_to_user(&self, user_id: u64, message: WsServerMessage) -> usize {
        let user_connections = self.user_connections.read().await;
        let mut delivered = 0;

        if let Some(connection_ids) = user_connections.get(&user_id) {
            let connections = self.connections.read().await;
            for &connection_id in connection_ids {
                if let Some(connection) = connections.get(&connection_id) {
                    if connection.message_sender.send(message.clone()).is_ok() {
                        delivered += 1;
                    }
                }
            }
        }

        // Store in history
        let mut history = self.message_history.write().await;
        history.push(message);

        let mut stats = self.stats.write().await;
        stats.messages_sent += delivered as u64;

        delivered
    }

    pub async fn broadcast_message(&self, message: WsServerMessage) -> usize {
        let connections = self.connections.read().await;
        let mut delivered = 0;

        for connection in connections.values() {
            if connection.message_sender.send(message.clone()).is_ok() {
                delivered += 1;
            }
        }

        // Store in history
        let mut history = self.message_history.write().await;
        history.push(message);

        let mut stats = self.stats.write().await;
        stats.messages_sent += delivered as u64;

        delivered
    }

    pub async fn get_connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    pub async fn get_user_connection_count(&self, user_id: u64) -> usize {
        let user_connections = self.user_connections.read().await;
        user_connections.get(&user_id).map(|conns| conns.len()).unwrap_or(0)
    }

    pub async fn get_stats(&self) -> WebSocketStats {
        self.stats.read().await.clone()
    }

    pub async fn get_message_history(&self) -> Vec<WsServerMessage> {
        self.message_history.read().await.clone()
    }

    pub async fn increment_messages_received(&self) {
        let mut stats = self.stats.write().await;
        stats.messages_received += 1;
    }
}

// ===== WEBSOCKET HANDLERS =====

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(manager): State<Arc<WebSocketManager>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_websocket_connection(socket, addr, manager))
}

async fn handle_websocket_connection(
    socket: WebSocket,
    _addr: SocketAddr,
    manager: Arc<WebSocketManager>,
) {
    let connection_id = Uuid::new_v4();
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (message_sender, mut message_receiver) = mpsc::unbounded_channel::<WsServerMessage>();

    // Add connection to manager
    manager.add_connection(connection_id, message_sender).await;

    // Spawn task to send messages to client
    let sender_manager = manager.clone();
    let sender_task = tokio::spawn(async move {
        while let Some(message) = message_receiver.recv().await {
            let json_message = serde_json::to_string(&message).unwrap_or_default();
            if ws_sender.send(Message::Text(json_message)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    let receiver_manager = manager.clone();
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                receiver_manager.increment_messages_received().await;
                
                if let Ok(client_message) = serde_json::from_str::<WsClientMessage>(&text) {
                    handle_client_message(&receiver_manager, connection_id, client_message).await;
                }
            }
            Ok(Message::Ping(_)) => {
                receiver_manager.ping_connection(connection_id).await;
            }
            Ok(Message::Close(_)) => break,
            Err(_) => break,
            _ => {}
        }
    }

    // Cleanup
    sender_task.abort();
    manager.remove_connection(connection_id).await;
}

async fn handle_client_message(
    manager: &WebSocketManager,
    connection_id: Uuid,
    message: WsClientMessage,
) {
    match message.message_type.as_str() {
        "auth" => {
            if let Ok(auth_data) = serde_json::from_value::<WsAuthMessage>(message.payload) {
                // Simple token validation for testing
                if auth_data.token.starts_with("valid_token") {
                    manager.authenticate_connection(connection_id, auth_data.user_id).await;
                    
                    let response = WsServerMessage {
                        id: Uuid::new_v4().to_string(),
                        message_type: "auth_success".to_string(),
                        payload: json!({"user_id": auth_data.user_id}),
                        timestamp: Utc::now(),
                        user_id: Some(auth_data.user_id),
                    };
                    
                    // Send directly to this connection
                    let connections = manager.connections.read().await;
                    if let Some(connection) = connections.get(&connection_id) {
                        let _ = connection.message_sender.send(response);
                    }
                } else {
                    let response = WsServerMessage {
                        id: Uuid::new_v4().to_string(),
                        message_type: "auth_error".to_string(),
                        payload: json!({"error": "Invalid token"}),
                        timestamp: Utc::now(),
                        user_id: None,
                    };
                    
                    let connections = manager.connections.read().await;
                    if let Some(connection) = connections.get(&connection_id) {
                        let _ = connection.message_sender.send(response);
                    }
                }
            }
        }
        "ping" => {
            manager.ping_connection(connection_id).await;
            
            let response = WsServerMessage {
                id: Uuid::new_v4().to_string(),
                message_type: "pong".to_string(),
                payload: json!({"timestamp": Utc::now()}),
                timestamp: Utc::now(),
                user_id: None,
            };
            
            let connections = manager.connections.read().await;
            if let Some(connection) = connections.get(&connection_id) {
                let _ = connection.message_sender.send(response);
            }
        }
        "game_action" => {
            // Echo back the game action for testing
            let response = WsServerMessage {
                id: Uuid::new_v4().to_string(),
                message_type: "game_action_ack".to_string(),
                payload: message.payload,
                timestamp: Utc::now(),
                user_id: None,
            };
            
            let connections = manager.connections.read().await;
            if let Some(connection) = connections.get(&connection_id) {
                let _ = connection.message_sender.send(response);
            }
        }
        _ => {
            // Unknown message type
            let response = WsServerMessage {
                id: Uuid::new_v4().to_string(),
                message_type: "error".to_string(),
                payload: json!({"error": "Unknown message type"}),
                timestamp: Utc::now(),
                user_id: None,
            };
            
            let connections = manager.connections.read().await;
            if let Some(connection) = connections.get(&connection_id) {
                let _ = connection.message_sender.send(response);
            }
        }
    }
}

// ===== TEST WEBSOCKET CLIENT =====

pub struct TestWebSocketClient {
    stream: WebSocketStream<TcpStream>,
    connection_id: Option<Uuid>,
    user_id: Option<u64>,
}

impl TestWebSocketClient {
    pub async fn connect(url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, _) = connect_async(url).await?;
        
        Ok(Self {
            stream,
            connection_id: None,
            user_id: None,
        })
    }

    pub async fn send_message(&mut self, message: WsClientMessage) -> Result<(), Box<dyn std::error::Error>> {
        let json_message = serde_json::to_string(&message)?;
        self.stream.send(WsMessage::Text(json_message)).await?;
        Ok(())
    }

    pub async fn receive_message(&mut self) -> Result<WsServerMessage, Box<dyn std::error::Error>> {
        if let Some(msg) = self.stream.next().await {
            match msg? {
                WsMessage::Text(text) => {
                    let message: WsServerMessage = serde_json::from_str(&text)?;
                    Ok(message)
                }
                _ => Err("Expected text message".into()),
            }
        } else {
            Err("Connection closed".into())
        }
    }

    pub async fn authenticate(&mut self, token: &str, user_id: u64) -> Result<bool, Box<dyn std::error::Error>> {
        let auth_message = WsClientMessage {
            id: Some(Uuid::new_v4().to_string()),
            message_type: "auth".to_string(),
            payload: json!({
                "token": token,
                "user_id": user_id
            }),
            timestamp: Some(Utc::now()),
        };

        self.send_message(auth_message).await?;
        
        let response = timeout(Duration::from_secs(5), self.receive_message()).await??;
        
        if response.message_type == "auth_success" {
            self.user_id = Some(user_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn ping(&mut self) -> Result<Duration, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();
        
        let ping_message = WsClientMessage {
            id: Some(Uuid::new_v4().to_string()),
            message_type: "ping".to_string(),
            payload: json!({}),
            timestamp: Some(Utc::now()),
        };

        self.send_message(ping_message).await?;
        
        let response = timeout(Duration::from_secs(5), self.receive_message()).await??;
        
        if response.message_type == "pong" {
            Ok(start.elapsed())
        } else {
            Err("Expected pong response".into())
        }
    }

    pub async fn close(mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.stream.send(WsMessage::Close(None)).await?;
        Ok(())
    }
}

// Helper function to create test WebSocket server
async fn create_test_websocket_server() -> (String, Arc<WebSocketManager>) {
    let manager = Arc::new(WebSocketManager::new());
    
    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(manager.clone());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("ws://{}/ws", addr);

    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
    });

    // Wait for server to be ready
    sleep(Duration::from_millis(100)).await;

    (url, manager)
}

// ===== WEBSOCKET INTEGRATION TESTS =====

#[tokio::test]
async fn test_websocket_connection_lifecycle() {
    let (url, manager) = create_test_websocket_server().await;

    // Test connection establishment
    let mut client = assert_ok!(TestWebSocketClient::connect(&url).await);
    
    // Give time for connection to be registered
    sleep(Duration::from_millis(50)).await;
    assert_eq!(manager.get_connection_count().await, 1);

    // Test ping/pong
    let ping_time = assert_ok!(client.ping().await);
    assert!(ping_time < Duration::from_millis(100));

    // Test connection cleanup
    assert_ok!(client.close().await);
    sleep(Duration::from_millis(50)).await;
    assert_eq!(manager.get_connection_count().await, 0);
}

#[tokio::test]
async fn test_websocket_authentication() {
    let (url, manager) = create_test_websocket_server().await;

    let mut client = assert_ok!(TestWebSocketClient::connect(&url).await);
    sleep(Duration::from_millis(50)).await;

    // Test successful authentication
    let auth_result = assert_ok!(client.authenticate("valid_token_123", 456).await);
    assert!(auth_result);

    // Verify user connection is tracked
    sleep(Duration::from_millis(50)).await;
    assert_eq!(manager.get_user_connection_count(456).await, 1);

    assert_ok!(client.close().await);
}

#[tokio::test]
async fn test_websocket_authentication_failure() {
    let (url, _manager) = create_test_websocket_server().await;

    let mut client = assert_ok!(TestWebSocketClient::connect(&url).await);
    sleep(Duration::from_millis(50)).await;

    // Test failed authentication
    let auth_result = assert_ok!(client.authenticate("invalid_token", 456).await);
    assert!(!auth_result);

    assert_ok!(client.close().await);
}

#[tokio::test]
async fn test_websocket_message_broadcasting() {
    let (url, manager) = create_test_websocket_server().await;

    // Connect multiple clients
    let mut client1 = assert_ok!(TestWebSocketClient::connect(&url).await);
    let mut client2 = assert_ok!(TestWebSocketClient::connect(&url).await);
    let mut client3 = assert_ok!(TestWebSocketClient::connect(&url).await);
    
    sleep(Duration::from_millis(100)).await;

    // Authenticate clients
    assert!(assert_ok!(client1.authenticate("valid_token_1", 100).await));
    assert!(assert_ok!(client2.authenticate("valid_token_2", 200).await));
    assert!(assert_ok!(client3.authenticate("valid_token_3", 300).await));

    sleep(Duration::from_millis(50)).await;

    // Broadcast message
    let broadcast_message = WsServerMessage {
        id: Uuid::new_v4().to_string(),
        message_type: "system_announcement".to_string(),
        payload: json!({"message": "System maintenance in 10 minutes"}),
        timestamp: Utc::now(),
        user_id: None,
    };

    let delivered = manager.broadcast_message(broadcast_message).await;
    assert_eq!(delivered, 3);

    // All clients should receive the message
    let msg1 = assert_ok!(timeout(Duration::from_secs(1), client1.receive_message()).await);
    let msg2 = assert_ok!(timeout(Duration::from_secs(1), client2.receive_message()).await);
    let msg3 = assert_ok!(timeout(Duration::from_secs(1), client3.receive_message()).await);

    assert_eq!(msg1.message_type, "system_announcement");
    assert_eq!(msg2.message_type, "system_announcement");
    assert_eq!(msg3.message_type, "system_announcement");

    assert_ok!(client1.close().await);
    assert_ok!(client2.close().await);
    assert_ok!(client3.close().await);
}

#[tokio::test]
async fn test_websocket_user_specific_messaging() {
    let (url, manager) = create_test_websocket_server().await;

    let mut client1 = assert_ok!(TestWebSocketClient::connect(&url).await);
    let mut client2 = assert_ok!(TestWebSocketClient::connect(&url).await);
    
    sleep(Duration::from_millis(100)).await;

    assert!(assert_ok!(client1.authenticate("valid_token_1", 100).await));
    assert!(assert_ok!(client2.authenticate("valid_token_2", 200).await));

    sleep(Duration::from_millis(50)).await;

    // Send message to specific user
    let user_message = WsServerMessage {
        id: Uuid::new_v4().to_string(),
        message_type: "private_notification".to_string(),
        payload: json!({"message": "You have a new message"}),
        timestamp: Utc::now(),
        user_id: Some(100),
    };

    let delivered = manager.send_message_to_user(100, user_message).await;
    assert_eq!(delivered, 1);

    // Only client1 should receive the message
    let msg1 = assert_ok!(timeout(Duration::from_secs(1), client1.receive_message()).await);
    assert_eq!(msg1.message_type, "private_notification");

    // client2 should not receive anything (timeout expected)
    let result = timeout(Duration::from_millis(100), client2.receive_message()).await;
    assert!(result.is_err());

    assert_ok!(client1.close().await);
    assert_ok!(client2.close().await);
}

#[tokio::test]
async fn test_websocket_multiple_connections_per_user() {
    let (url, manager) = create_test_websocket_server().await;

    // Same user connecting from multiple devices/tabs
    let mut client1 = assert_ok!(TestWebSocketClient::connect(&url).await);
    let mut client2 = assert_ok!(TestWebSocketClient::connect(&url).await);
    let mut client3 = assert_ok!(TestWebSocketClient::connect(&url).await);
    
    sleep(Duration::from_millis(100)).await;

    // All clients authenticate as the same user
    assert!(assert_ok!(client1.authenticate("valid_token_1", 100).await));
    assert!(assert_ok!(client2.authenticate("valid_token_2", 100).await));
    assert!(assert_ok!(client3.authenticate("valid_token_3", 100).await));

    sleep(Duration::from_millis(50)).await);
    assert_eq!(manager.get_user_connection_count(100).await, 3);

    // Send message to user (should reach all connections)
    let user_message = WsServerMessage {
        id: Uuid::new_v4().to_string(),
        message_type: "notification".to_string(),
        payload: json!({"message": "Multiple device notification"}),
        timestamp: Utc::now(),
        user_id: Some(100),
    };

    let delivered = manager.send_message_to_user(100, user_message).await;
    assert_eq!(delivered, 3);

    // All clients should receive the message
    let msg1 = assert_ok!(timeout(Duration::from_secs(1), client1.receive_message()).await);
    let msg2 = assert_ok!(timeout(Duration::from_secs(1), client2.receive_message()).await);
    let msg3 = assert_ok!(timeout(Duration::from_secs(1), client3.receive_message()).await);

    assert_eq!(msg1.message_type, "notification");
    assert_eq!(msg2.message_type, "notification");
    assert_eq!(msg3.message_type, "notification");

    assert_ok!(client1.close().await);
    assert_ok!(client2.close().await);
    assert_ok!(client3.close().await);
}

#[tokio::test]
async fn test_websocket_game_message_handling() {
    let (url, _manager) = create_test_websocket_server().await;

    let mut client = assert_ok!(TestWebSocketClient::connect(&url).await);
    sleep(Duration::from_millis(100)).await;

    assert!(assert_ok!(client.authenticate("valid_token_1", 123).await));

    // Send game action message
    let game_action = WsClientMessage {
        id: Some(Uuid::new_v4().to_string()),
        message_type: "game_action".to_string(),
        payload: json!({
            "action": "start_process",
            "process_type": "cracker",
            "target": "192.168.1.100"
        }),
        timestamp: Some(Utc::now()),
    };

    assert_ok!(client.send_message(game_action).await);

    // Should receive acknowledgment
    let response = assert_ok!(timeout(Duration::from_secs(1), client.receive_message()).await);
    assert_eq!(response.message_type, "game_action_ack");
    assert_eq!(response.payload["action"], "start_process");

    assert_ok!(client.close().await);
}

#[tokio::test]
async fn test_websocket_error_handling() {
    let (url, _manager) = create_test_websocket_server().await;

    let mut client = assert_ok!(TestWebSocketClient::connect(&url).await);
    sleep(Duration::from_millis(100)).await);

    // Send invalid message type
    let invalid_message = WsClientMessage {
        id: Some(Uuid::new_v4().to_string()),
        message_type: "unknown_message_type".to_string(),
        payload: json!({}),
        timestamp: Some(Utc::now()),
    };

    assert_ok!(client.send_message(invalid_message).await);

    // Should receive error response
    let response = assert_ok!(timeout(Duration::from_secs(1), client.receive_message()).await);
    assert_eq!(response.message_type, "error");
    assert!(response.payload.get("error").is_some());

    assert_ok!(client.close().await);
}

#[tokio::test]
async fn test_websocket_concurrent_connections() {
    let (url, manager) = create_test_websocket_server().await;

    // Create many concurrent connections
    let mut clients = Vec::new();
    let num_clients = 50;

    for i in 0..num_clients {
        let client = assert_ok!(TestWebSocketClient::connect(&url).await);
        clients.push(client);
    }

    sleep(Duration::from_millis(200)).await;
    assert_eq!(manager.get_connection_count().await, num_clients);

    // Authenticate all clients
    for (i, client) in clients.iter_mut().enumerate() {
        let auth_result = assert_ok!(client.authenticate(&format!("valid_token_{}", i), i as u64).await);
        assert!(auth_result);
    }

    sleep(Duration::from_millis(100)).await;

    // Broadcast message to all
    let broadcast_message = WsServerMessage {
        id: Uuid::new_v4().to_string(),
        message_type: "load_test".to_string(),
        payload: json!({"test": "concurrent_broadcast"}),
        timestamp: Utc::now(),
        user_id: None,
    };

    let delivered = manager.broadcast_message(broadcast_message).await;
    assert_eq!(delivered, num_clients);

    // Close all connections
    for client in clients {
        assert_ok!(client.close().await);
    }

    sleep(Duration::from_millis(100)).await;
    assert_eq!(manager.get_connection_count().await, 0);
}

#[tokio::test]
async fn test_websocket_stats_tracking() {
    let (url, manager) = create_test_websocket_server().await;

    let mut client1 = assert_ok!(TestWebSocketClient::connect(&url).await);
    let mut client2 = assert_ok!(TestWebSocketClient::connect(&url).await);
    
    sleep(Duration::from_millis(100)).await;

    let initial_stats = manager.get_stats().await;
    assert_eq!(initial_stats.active_connections, 2);
    assert_eq!(initial_stats.total_connections, 2);

    // Authenticate clients (increases authentication attempts)
    assert!(assert_ok!(client1.authenticate("valid_token_1", 100).await));
    assert!(assert_ok!(client2.authenticate("invalid_token", 200).await) == false);

    sleep(Duration::from_millis(50)).await;

    let stats = manager.get_stats().await;
    assert_eq!(stats.authentication_attempts, 2);
    assert_eq!(stats.failed_authentications, 1);

    // Send some messages
    assert_ok!(client1.ping().await);

    let final_stats = manager.get_stats().await;
    assert!(final_stats.messages_sent > 0);
    assert!(final_stats.messages_received > 0);

    assert_ok!(client1.close().await);
    assert_ok!(client2.close().await);
}

#[tokio::test]
async fn test_websocket_message_history() {
    let (url, manager) = create_test_websocket_server().await;

    let mut client = assert_ok!(TestWebSocketClient::connect(&url).await);
    sleep(Duration::from_millis(100)).await;

    assert!(assert_ok!(client.authenticate("valid_token_1", 123).await));

    // Send a broadcast message
    let message1 = WsServerMessage {
        id: Uuid::new_v4().to_string(),
        message_type: "test_message_1".to_string(),
        payload: json!({"data": "first_message"}),
        timestamp: Utc::now(),
        user_id: None,
    };

    manager.broadcast_message(message1.clone()).await;

    // Send a user-specific message
    let message2 = WsServerMessage {
        id: Uuid::new_v4().to_string(),
        message_type: "test_message_2".to_string(),
        payload: json!({"data": "second_message"}),
        timestamp: Utc::now(),
        user_id: Some(123),
    };

    manager.send_message_to_user(123, message2.clone()).await;

    // Check message history
    let history = manager.get_message_history().await;
    
    // Should have auth_success + test messages
    assert!(history.len() >= 3);
    
    // Find our test messages in history
    let test_messages: Vec<_> = history.iter()
        .filter(|msg| msg.message_type.starts_with("test_message"))
        .collect();
    
    assert_eq!(test_messages.len(), 2);

    assert_ok!(client.close().await);
}

#[tokio::test]
async fn test_websocket_connection_cleanup_on_disconnect() {
    let (url, manager) = create_test_websocket_server().await;

    let mut client = assert_ok!(TestWebSocketClient::connect(&url).await);
    sleep(Duration::from_millis(100)).await;

    assert!(assert_ok!(client.authenticate("valid_token_1", 456).await));
    sleep(Duration::from_millis(50)).await;

    assert_eq!(manager.get_connection_count().await, 1);
    assert_eq!(manager.get_user_connection_count(456).await, 1);

    // Simulate abrupt disconnection
    drop(client);
    sleep(Duration::from_millis(100)).await;

    // Connection should be cleaned up
    assert_eq!(manager.get_connection_count().await, 0);
    assert_eq!(manager.get_user_connection_count(456).await, 0);
}

#[tokio::test]
async fn test_websocket_performance() {
    let (url, manager) = create_test_websocket_server().await;

    let mut client = assert_ok!(TestWebSocketClient::connect(&url).await);
    sleep(Duration::from_millis(100)).await;

    assert!(assert_ok!(client.authenticate("valid_token_1", 999).await));

    let start_time = std::time::Instant::now();
    let num_pings = 100;

    // Measure ping performance
    for _ in 0..num_pings {
        let ping_time = assert_ok!(client.ping().await);
        assert!(ping_time < Duration::from_millis(100)); // Should be fast
    }

    let total_time = start_time.elapsed();
    let avg_ping = total_time / num_pings;

    println!("WebSocket ping performance: {} pings in {:?} (avg: {:?})", 
        num_pings, total_time, avg_ping);

    // Performance assertion: average ping should be under 10ms
    assert!(avg_ping < Duration::from_millis(10), 
        "WebSocket ping too slow: avg {:?}", avg_ping);

    assert_ok!(client.close().await);
}