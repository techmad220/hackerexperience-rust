use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{Duration, timeout};
use serde_json::{json, Value};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::common::{TestDb, TestFixtures};
use crate::{assert_ok, assert_err};

// ===== WEBSOCKET INFRASTRUCTURE TESTS =====

#[derive(Debug, Clone)]
pub struct WebSocketMessage {
    pub id: Uuid,
    pub user_id: u64,
    pub message_type: String,
    pub payload: Value,
    pub timestamp: DateTime<Utc>,
}

impl WebSocketMessage {
    pub fn new(user_id: u64, message_type: &str, payload: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            message_type: message_type.to_string(),
            payload,
            timestamp: Utc::now(),
        }
    }
}

pub struct WebSocketConnection {
    pub connection_id: Uuid,
    pub user_id: u64,
    pub connected_at: DateTime<Utc>,
    pub last_ping: Option<DateTime<Utc>>,
    pub message_queue: Vec<WebSocketMessage>,
}

impl WebSocketConnection {
    pub fn new(user_id: u64) -> Self {
        Self {
            connection_id: Uuid::new_v4(),
            user_id,
            connected_at: Utc::now(),
            last_ping: None,
            message_queue: Vec::new(),
        }
    }

    pub fn ping(&mut self) {
        self.last_ping = Some(Utc::now());
    }

    pub fn is_alive(&self, timeout_duration: Duration) -> bool {
        if let Some(last_ping) = self.last_ping {
            Utc::now().signed_duration_since(last_ping) < chrono::Duration::from_std(timeout_duration).unwrap()
        } else {
            // If never pinged, check connection time
            Utc::now().signed_duration_since(self.connected_at) < chrono::Duration::from_std(timeout_duration).unwrap()
        }
    }

    pub fn queue_message(&mut self, message: WebSocketMessage) {
        self.message_queue.push(message);
    }

    pub fn drain_messages(&mut self) -> Vec<WebSocketMessage> {
        self.message_queue.drain(..).collect()
    }
}

pub struct WebSocketManager {
    connections: Arc<RwLock<HashMap<Uuid, WebSocketConnection>>>,
    user_connections: Arc<RwLock<HashMap<u64, Vec<Uuid>>>>,
    message_sender: mpsc::UnboundedSender<WebSocketMessage>,
    message_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<WebSocketMessage>>>>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        let (message_sender, message_receiver) = mpsc::unbounded_channel();
        
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            user_connections: Arc::new(RwLock::new(HashMap::new())),
            message_sender,
            message_receiver: Arc::new(RwLock::new(Some(message_receiver))),
        }
    }

    pub async fn add_connection(&self, user_id: u64) -> Uuid {
        let connection = WebSocketConnection::new(user_id);
        let connection_id = connection.connection_id;

        let mut connections = self.connections.write().await;
        connections.insert(connection_id, connection);

        let mut user_connections = self.user_connections.write().await;
        user_connections.entry(user_id).or_insert_with(Vec::new).push(connection_id);

        connection_id
    }

    pub async fn remove_connection(&self, connection_id: Uuid) {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.remove(&connection_id) {
            let mut user_connections = self.user_connections.write().await;
            if let Some(user_conn_list) = user_connections.get_mut(&connection.user_id) {
                user_conn_list.retain(|&id| id != connection_id);
                if user_conn_list.is_empty() {
                    user_connections.remove(&connection.user_id);
                }
            }
        }
    }

    pub async fn ping_connection(&self, connection_id: Uuid) -> bool {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(&connection_id) {
            connection.ping();
            true
        } else {
            false
        }
    }

    pub async fn broadcast_to_user(&self, user_id: u64, message: WebSocketMessage) -> usize {
        let user_connections = self.user_connections.read().await;
        if let Some(connection_ids) = user_connections.get(&user_id) {
            let mut connections = self.connections.write().await;
            let mut delivered = 0;
            
            for &connection_id in connection_ids {
                if let Some(connection) = connections.get_mut(&connection_id) {
                    connection.queue_message(message.clone());
                    delivered += 1;
                }
            }
            delivered
        } else {
            0
        }
    }

    pub async fn get_connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    pub async fn cleanup_dead_connections(&self, timeout_duration: Duration) -> usize {
        let mut connections = self.connections.write().await;
        let mut user_connections = self.user_connections.write().await;
        let mut removed_count = 0;

        let dead_connections: Vec<_> = connections.iter()
            .filter(|(_, conn)| !conn.is_alive(timeout_duration))
            .map(|(&id, conn)| (id, conn.user_id))
            .collect();

        for (connection_id, user_id) in dead_connections {
            connections.remove(&connection_id);
            if let Some(user_conn_list) = user_connections.get_mut(&user_id) {
                user_conn_list.retain(|&id| id != connection_id);
                if user_conn_list.is_empty() {
                    user_connections.remove(&user_id);
                }
            }
            removed_count += 1;
        }

        removed_count
    }
}

#[tokio::test]
async fn test_websocket_connection_lifecycle() {
    let conn = WebSocketConnection::new(123);
    
    assert_eq!(conn.user_id, 123);
    assert!(conn.last_ping.is_none());
    assert!(conn.message_queue.is_empty());

    // Test connection is initially alive
    assert!(conn.is_alive(Duration::from_secs(30)));

    let mut conn = conn;
    conn.ping();
    assert!(conn.last_ping.is_some());
    assert!(conn.is_alive(Duration::from_secs(30)));

    // Test message queuing
    let message = WebSocketMessage::new(123, "test", json!({"data": "test"}));
    conn.queue_message(message);
    assert_eq!(conn.message_queue.len(), 1);

    let messages = conn.drain_messages();
    assert_eq!(messages.len(), 1);
    assert!(conn.message_queue.is_empty());
}

#[tokio::test]
async fn test_websocket_manager_connection_management() {
    let manager = WebSocketManager::new();
    
    // Test adding connections
    let conn1 = manager.add_connection(123).await;
    let conn2 = manager.add_connection(123).await; // Same user, different connection
    let conn3 = manager.add_connection(456).await; // Different user

    assert_eq!(manager.get_connection_count().await, 3);

    // Test removing connections
    manager.remove_connection(conn1).await;
    assert_eq!(manager.get_connection_count().await, 2);

    // Test ping
    assert!(manager.ping_connection(conn2).await);
    assert!(!manager.ping_connection(conn1).await); // Already removed

    // Cleanup remaining connections
    manager.remove_connection(conn2).await;
    manager.remove_connection(conn3).await;
    assert_eq!(manager.get_connection_count().await, 0);
}

#[tokio::test]
async fn test_websocket_message_broadcasting() {
    let manager = WebSocketManager::new();
    
    let conn1 = manager.add_connection(123).await;
    let conn2 = manager.add_connection(123).await; // Same user
    let conn3 = manager.add_connection(456).await; // Different user

    let message = WebSocketMessage::new(123, "notification", json!({
        "title": "Test Notification",
        "body": "This is a test message"
    }));

    // Broadcast to user 123 (should reach 2 connections)
    let delivered = manager.broadcast_to_user(123, message).await;
    assert_eq!(delivered, 2);

    // Broadcast to user 789 (should reach 0 connections)
    let message2 = WebSocketMessage::new(789, "notification", json!({"test": "data"}));
    let delivered = manager.broadcast_to_user(789, message2).await;
    assert_eq!(delivered, 0);
}

#[tokio::test]
async fn test_websocket_dead_connection_cleanup() {
    let manager = WebSocketManager::new();
    
    let conn1 = manager.add_connection(123).await;
    let conn2 = manager.add_connection(456).await;

    // Simulate one connection being alive
    manager.ping_connection(conn1).await;

    // Wait a bit then cleanup with very short timeout
    tokio::time::sleep(Duration::from_millis(10)).await;
    let removed = manager.cleanup_dead_connections(Duration::from_millis(5)).await;
    
    // Should have removed the connection that wasn't pinged recently
    assert_eq!(removed, 1);
    assert_eq!(manager.get_connection_count().await, 1);
}

// ===== DATABASE INFRASTRUCTURE TESTS =====

#[derive(Debug, Clone)]
pub struct DatabasePool {
    connections: usize,
    max_connections: usize,
    active_queries: Arc<RwLock<usize>>,
}

impl DatabasePool {
    pub fn new(max_connections: usize) -> Self {
        Self {
            connections: 0,
            max_connections,
            active_queries: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn get_connection(&mut self) -> Result<DatabaseConnection, DatabaseError> {
        if self.connections >= self.max_connections {
            return Err(DatabaseError::ConnectionPoolExhausted);
        }

        self.connections += 1;
        Ok(DatabaseConnection {
            id: Uuid::new_v4(),
            active_queries: self.active_queries.clone(),
        })
    }

    pub fn release_connection(&mut self) {
        if self.connections > 0 {
            self.connections -= 1;
        }
    }

    pub async fn get_active_query_count(&self) -> usize {
        *self.active_queries.read().await
    }
}

#[derive(Debug)]
pub struct DatabaseConnection {
    pub id: Uuid,
    active_queries: Arc<RwLock<usize>>,
}

impl DatabaseConnection {
    pub async fn execute_query(&self, query: &str) -> Result<QueryResult, DatabaseError> {
        {
            let mut count = self.active_queries.write().await;
            *count += 1;
        }

        // Simulate query execution
        tokio::time::sleep(Duration::from_millis(10)).await;

        let result = if query.contains("SELECT") {
            QueryResult::Select(vec![
                HashMap::from([("id".to_string(), "1".to_string())]),
                HashMap::from([("id".to_string(), "2".to_string())]),
            ])
        } else if query.contains("INSERT") {
            QueryResult::Insert(1) // Affected rows
        } else if query.contains("UPDATE") {
            QueryResult::Update(1)
        } else if query.contains("DELETE") {
            QueryResult::Delete(1)
        } else if query.contains("INVALID") {
            {
                let mut count = self.active_queries.write().await;
                *count -= 1;
            }
            return Err(DatabaseError::QueryError("Invalid SQL".to_string()));
        } else {
            QueryResult::Other
        };

        {
            let mut count = self.active_queries.write().await;
            *count -= 1;
        }

        Ok(result)
    }

    pub async fn begin_transaction(&self) -> Result<Transaction, DatabaseError> {
        Ok(Transaction {
            id: Uuid::new_v4(),
            is_committed: false,
            is_rolled_back: false,
        })
    }
}

#[derive(Debug)]
pub enum QueryResult {
    Select(Vec<HashMap<String, String>>),
    Insert(usize),
    Update(usize),
    Delete(usize),
    Other,
}

#[derive(Debug)]
pub struct Transaction {
    pub id: Uuid,
    pub is_committed: bool,
    pub is_rolled_back: bool,
}

impl Transaction {
    pub fn commit(&mut self) -> Result<(), DatabaseError> {
        if self.is_rolled_back {
            return Err(DatabaseError::TransactionError("Transaction already rolled back".to_string()));
        }
        self.is_committed = true;
        Ok(())
    }

    pub fn rollback(&mut self) -> Result<(), DatabaseError> {
        if self.is_committed {
            return Err(DatabaseError::TransactionError("Transaction already committed".to_string()));
        }
        self.is_rolled_back = true;
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum DatabaseError {
    ConnectionPoolExhausted,
    QueryError(String),
    TransactionError(String),
    NetworkError,
}

#[tokio::test]
async fn test_database_pool_management() {
    let mut pool = DatabasePool::new(2);
    
    // Test getting connections within limit
    let conn1 = pool.get_connection().await;
    assert!(conn1.is_ok());
    
    let conn2 = pool.get_connection().await;
    assert!(conn2.is_ok());
    
    // Test exceeding pool limit
    let conn3 = pool.get_connection().await;
    assert_eq!(conn3, Err(DatabaseError::ConnectionPoolExhausted));
    
    // Test releasing connections
    pool.release_connection();
    let conn4 = pool.get_connection().await;
    assert!(conn4.is_ok());
}

#[tokio::test]
async fn test_database_query_execution() {
    let mut pool = DatabasePool::new(5);
    let connection = assert_ok!(pool.get_connection().await);
    
    // Test SELECT query
    let result = connection.execute_query("SELECT * FROM users").await;
    assert!(result.is_ok());
    match result.unwrap() {
        QueryResult::Select(rows) => assert_eq!(rows.len(), 2),
        _ => panic!("Expected Select result"),
    }
    
    // Test INSERT query
    let result = connection.execute_query("INSERT INTO users (name) VALUES ('test')").await;
    assert!(result.is_ok());
    match result.unwrap() {
        QueryResult::Insert(affected) => assert_eq!(affected, 1),
        _ => panic!("Expected Insert result"),
    }
    
    // Test invalid query
    let result = connection.execute_query("INVALID SQL").await;
    assert_eq!(result, Err(DatabaseError::QueryError("Invalid SQL".to_string())));
}

#[tokio::test]
async fn test_database_transactions() {
    let mut pool = DatabasePool::new(5);
    let connection = assert_ok!(pool.get_connection().await);
    
    // Test transaction lifecycle
    let mut tx = assert_ok!(connection.begin_transaction().await);
    assert!(!tx.is_committed);
    assert!(!tx.is_rolled_back);
    
    // Test commit
    assert_ok!(tx.commit());
    assert!(tx.is_committed);
    
    // Test committing again should fail
    let result = tx.rollback();
    assert_eq!(result, Err(DatabaseError::TransactionError("Transaction already committed".to_string())));
    
    // Test rollback
    let mut tx2 = assert_ok!(connection.begin_transaction().await);
    assert_ok!(tx2.rollback());
    assert!(tx2.is_rolled_back);
    
    // Test rolling back again should fail
    let result = tx2.commit();
    assert_eq!(result, Err(DatabaseError::TransactionError("Transaction already rolled back".to_string())));
}

#[tokio::test]
async fn test_concurrent_database_operations() {
    let mut pool = DatabasePool::new(10);
    
    let futures: Vec<_> = (0..5).map(|_| {
        let conn = pool.get_connection();
        async move {
            let connection = conn.await?;
            connection.execute_query("SELECT * FROM test_table").await
        }
    }).collect();
    
    let results = futures::future::join_all(futures).await;
    
    for result in results {
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), QueryResult::Select(_)));
    }
}

// ===== EVENT SYSTEM TESTS =====

#[derive(Debug, Clone)]
pub struct Event {
    pub id: Uuid,
    pub event_type: String,
    pub user_id: Option<u64>,
    pub payload: Value,
    pub timestamp: DateTime<Utc>,
}

impl Event {
    pub fn new(event_type: &str, user_id: Option<u64>, payload: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            user_id,
            payload,
            timestamp: Utc::now(),
        }
    }
}

pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &Event) -> Result<(), EventError>;
}

pub struct EventBus {
    handlers: Arc<RwLock<HashMap<String, Vec<Box<dyn EventHandler>>>>>,
    event_history: Arc<RwLock<Vec<Event>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            event_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn subscribe<H: EventHandler + 'static>(&self, event_type: &str, handler: H) {
        let mut handlers = self.handlers.write().await;
        handlers.entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(Box::new(handler));
    }

    pub async fn publish(&self, event: Event) -> Result<(), EventError> {
        // Store event in history
        {
            let mut history = self.event_history.write().await;
            history.push(event.clone());
        }

        // Get handlers for this event type
        let handlers = self.handlers.read().await;
        if let Some(event_handlers) = handlers.get(&event.event_type) {
            // Process handlers sequentially to maintain order
            for handler in event_handlers {
                if let Err(e) = handler.handle(&event).await {
                    eprintln!("Event handler error: {:?}", e);
                    // Continue processing other handlers
                }
            }
        }

        Ok(())
    }

    pub async fn get_event_history(&self) -> Vec<Event> {
        self.event_history.read().await.clone()
    }

    pub async fn clear_history(&self) {
        self.event_history.write().await.clear();
    }
}

#[derive(Debug, PartialEq)]
pub enum EventError {
    HandlerError(String),
    SerializationError,
    NetworkError,
}

// Example event handlers for testing
pub struct LoggingHandler {
    pub logged_events: Arc<RwLock<Vec<Event>>>,
}

impl LoggingHandler {
    pub fn new() -> Self {
        Self {
            logged_events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get_logged_events(&self) -> Vec<Event> {
        self.logged_events.read().await.clone()
    }
}

impl EventHandler for LoggingHandler {
    async fn handle(&self, event: &Event) -> Result<(), EventError> {
        let mut logged = self.logged_events.write().await;
        logged.push(event.clone());
        println!("Logged event: {} ({})", event.event_type, event.id);
        Ok(())
    }
}

pub struct NotificationHandler;

impl EventHandler for NotificationHandler {
    async fn handle(&self, event: &Event) -> Result<(), EventError> {
        if event.event_type == "user_action" {
            println!("Sending notification for user action: {}", event.user_id.unwrap_or(0));
            Ok(())
        } else if event.event_type == "error" {
            Err(EventError::HandlerError("Simulated error".to_string()))
        } else {
            Ok(())
        }
    }
}

#[tokio::test]
async fn test_event_creation() {
    let event = Event::new("user_login", Some(123), json!({"ip": "192.168.1.1"}));
    
    assert_eq!(event.event_type, "user_login");
    assert_eq!(event.user_id, Some(123));
    assert_eq!(event.payload["ip"], "192.168.1.1");
    assert!(Utc::now().signed_duration_since(event.timestamp) < chrono::Duration::seconds(1));
}

#[tokio::test]
async fn test_event_bus_subscription_and_publishing() {
    let event_bus = EventBus::new();
    let logging_handler = LoggingHandler::new();
    
    // Subscribe handler
    event_bus.subscribe("test_event", logging_handler.clone()).await;
    
    // Publish event
    let event = Event::new("test_event", Some(123), json!({"data": "test"}));
    let result = event_bus.publish(event.clone()).await;
    assert!(result.is_ok());
    
    // Check event was handled
    tokio::time::sleep(Duration::from_millis(10)).await;
    let logged_events = logging_handler.get_logged_events().await;
    assert_eq!(logged_events.len(), 1);
    assert_eq!(logged_events[0].event_type, "test_event");
    
    // Check event history
    let history = event_bus.get_event_history().await;
    assert_eq!(history.len(), 1);
}

#[tokio::test]
async fn test_event_bus_multiple_handlers() {
    let event_bus = EventBus::new();
    let logging_handler1 = LoggingHandler::new();
    let logging_handler2 = LoggingHandler::new();
    let notification_handler = NotificationHandler;
    
    // Subscribe multiple handlers to same event
    event_bus.subscribe("user_action", logging_handler1.clone()).await;
    event_bus.subscribe("user_action", logging_handler2.clone()).await;
    event_bus.subscribe("user_action", notification_handler).await;
    
    // Publish event
    let event = Event::new("user_action", Some(456), json!({"action": "login"}));
    let result = event_bus.publish(event).await;
    assert!(result.is_ok());
    
    // Check all handlers processed the event
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert_eq!(logging_handler1.get_logged_events().await.len(), 1);
    assert_eq!(logging_handler2.get_logged_events().await.len(), 1);
}

#[tokio::test]
async fn test_event_bus_error_handling() {
    let event_bus = EventBus::new();
    let logging_handler = LoggingHandler::new();
    let notification_handler = NotificationHandler;
    
    // Subscribe handlers
    event_bus.subscribe("error", logging_handler.clone()).await;
    event_bus.subscribe("error", notification_handler).await;
    
    // Publish event that will cause handler error
    let event = Event::new("error", Some(123), json!({"error": "test"}));
    let result = event_bus.publish(event).await;
    
    // Event bus should continue processing despite handler error
    assert!(result.is_ok());
    
    // Logging handler should still have processed the event
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert_eq!(logging_handler.get_logged_events().await.len(), 1);
}

// ===== AUTHENTICATION INFRASTRUCTURE TESTS =====

#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: Uuid,
    pub user_id: u64,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

impl Session {
    pub fn new(user_id: u64, duration: Duration) -> Self {
        let now = Utc::now();
        Self {
            session_id: Uuid::new_v4(),
            user_id,
            expires_at: now + chrono::Duration::from_std(duration).unwrap(),
            created_at: now,
            last_accessed: now,
        }
    }

    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }

    pub fn refresh(&mut self) {
        self.last_accessed = Utc::now();
    }
}

pub struct AuthService {
    users: Arc<RwLock<HashMap<u64, User>>>,
    sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
    username_to_id: Arc<RwLock<HashMap<String, u64>>>,
    next_user_id: Arc<RwLock<u64>>,
}

impl AuthService {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            username_to_id: Arc::new(RwLock::new(HashMap::new())),
            next_user_id: Arc::new(RwLock::new(1)),
        }
    }

    pub async fn create_user(&self, username: &str, email: &str, password: &str) -> Result<u64, AuthError> {
        // Check if username exists
        {
            let username_map = self.username_to_id.read().await;
            if username_map.contains_key(username) {
                return Err(AuthError::UsernameExists);
            }
        }

        // Get next user ID
        let user_id = {
            let mut next_id = self.next_user_id.write().await;
            let id = *next_id;
            *next_id += 1;
            id
        };

        // Create user
        let user = User {
            id: user_id,
            username: username.to_string(),
            email: email.to_string(),
            password_hash: Self::hash_password(password),
            is_active: true,
            created_at: Utc::now(),
            last_login: None,
        };

        // Store user
        {
            let mut users = self.users.write().await;
            users.insert(user_id, user);
        }

        {
            let mut username_map = self.username_to_id.write().await;
            username_map.insert(username.to_string(), user_id);
        }

        Ok(user_id)
    }

    pub async fn authenticate(&self, username: &str, password: &str) -> Result<Session, AuthError> {
        // Get user ID from username
        let user_id = {
            let username_map = self.username_to_id.read().await;
            *username_map.get(username).ok_or(AuthError::InvalidCredentials)?
        };

        // Get user and verify password
        {
            let mut users = self.users.write().await;
            if let Some(user) = users.get_mut(&user_id) {
                if !user.is_active {
                    return Err(AuthError::UserDeactivated);
                }

                if !Self::verify_password(password, &user.password_hash) {
                    return Err(AuthError::InvalidCredentials);
                }

                // Update last login
                user.last_login = Some(Utc::now());
            } else {
                return Err(AuthError::InvalidCredentials);
            }
        }

        // Create session
        let session = Session::new(user_id, Duration::from_secs(3600)); // 1 hour
        let session_id = session.session_id;

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id, session.clone());
        }

        Ok(session)
    }

    pub async fn validate_session(&self, session_id: Uuid) -> Result<u64, AuthError> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(&session_id) {
            if session.is_valid() {
                session.refresh();
                Ok(session.user_id)
            } else {
                sessions.remove(&session_id);
                Err(AuthError::SessionExpired)
            }
        } else {
            Err(AuthError::InvalidSession)
        }
    }

    pub async fn logout(&self, session_id: Uuid) -> Result<(), AuthError> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(&session_id);
        Ok(())
    }

    pub async fn cleanup_expired_sessions(&self) -> usize {
        let mut sessions = self.sessions.write().await;
        let initial_count = sessions.len();
        
        sessions.retain(|_, session| session.is_valid());
        
        initial_count - sessions.len()
    }

    fn hash_password(password: &str) -> String {
        // Simple hash for testing - use proper crypto in production
        format!("hash_{}", password)
    }

    fn verify_password(password: &str, hash: &str) -> bool {
        hash == format!("hash_{}", password)
    }
}

#[derive(Debug, PartialEq)]
pub enum AuthError {
    UsernameExists,
    InvalidCredentials,
    UserDeactivated,
    SessionExpired,
    InvalidSession,
    DatabaseError,
}

#[tokio::test]
async fn test_user_creation() {
    let auth_service = AuthService::new();
    
    // Create user
    let result = auth_service.create_user("testuser", "test@example.com", "password123").await;
    assert!(result.is_ok());
    let user_id = result.unwrap();
    assert_eq!(user_id, 1);
    
    // Try to create duplicate user
    let result = auth_service.create_user("testuser", "test2@example.com", "password456").await;
    assert_eq!(result, Err(AuthError::UsernameExists));
}

#[tokio::test]
async fn test_user_authentication() {
    let auth_service = AuthService::new();
    
    // Create user
    let user_id = assert_ok!(auth_service.create_user("testuser", "test@example.com", "password123").await);
    
    // Test successful authentication
    let result = auth_service.authenticate("testuser", "password123").await;
    assert!(result.is_ok());
    let session = result.unwrap();
    assert_eq!(session.user_id, user_id);
    assert!(session.is_valid());
    
    // Test wrong password
    let result = auth_service.authenticate("testuser", "wrongpassword").await;
    assert_eq!(result, Err(AuthError::InvalidCredentials));
    
    // Test non-existent user
    let result = auth_service.authenticate("nonexistent", "password123").await;
    assert_eq!(result, Err(AuthError::InvalidCredentials));
}

#[tokio::test]
async fn test_session_validation() {
    let auth_service = AuthService::new();
    
    // Create user and authenticate
    let user_id = assert_ok!(auth_service.create_user("testuser", "test@example.com", "password123").await);
    let session = assert_ok!(auth_service.authenticate("testuser", "password123").await);
    
    // Test valid session
    let result = auth_service.validate_session(session.session_id).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), user_id);
    
    // Test invalid session ID
    let fake_session_id = Uuid::new_v4();
    let result = auth_service.validate_session(fake_session_id).await;
    assert_eq!(result, Err(AuthError::InvalidSession));
}

#[tokio::test]
async fn test_session_expiration() {
    let auth_service = AuthService::new();
    
    // Create user
    let user_id = assert_ok!(auth_service.create_user("testuser", "test@example.com", "password123").await);
    
    // Create short-lived session
    let session = Session::new(user_id, Duration::from_millis(50));
    let session_id = session.session_id;
    
    {
        let mut sessions = auth_service.sessions.write().await;
        sessions.insert(session_id, session);
    }
    
    // Session should be valid initially
    let result = auth_service.validate_session(session_id).await;
    assert!(result.is_ok());
    
    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Session should now be expired
    let result = auth_service.validate_session(session_id).await;
    assert_eq!(result, Err(AuthError::SessionExpired));
}

#[tokio::test]
async fn test_logout() {
    let auth_service = AuthService::new();
    
    // Create user and authenticate
    let user_id = assert_ok!(auth_service.create_user("testuser", "test@example.com", "password123").await);
    let session = assert_ok!(auth_service.authenticate("testuser", "password123").await);
    
    // Validate session exists
    let result = auth_service.validate_session(session.session_id).await;
    assert!(result.is_ok());
    
    // Logout
    let result = auth_service.logout(session.session_id).await;
    assert!(result.is_ok());
    
    // Session should no longer be valid
    let result = auth_service.validate_session(session.session_id).await;
    assert_eq!(result, Err(AuthError::InvalidSession));
}

#[tokio::test]
async fn test_session_cleanup() {
    let auth_service = AuthService::new();
    
    // Create user
    let user_id = assert_ok!(auth_service.create_user("testuser", "test@example.com", "password123").await);
    
    // Create several sessions with different expiration times
    let short_session = Session::new(user_id, Duration::from_millis(50));
    let long_session = Session::new(user_id, Duration::from_secs(3600));
    
    {
        let mut sessions = auth_service.sessions.write().await;
        sessions.insert(short_session.session_id, short_session);
        sessions.insert(long_session.session_id, long_session);
    }
    
    // Wait for short session to expire
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Cleanup expired sessions
    let cleaned_up = auth_service.cleanup_expired_sessions().await;
    assert_eq!(cleaned_up, 1);
    
    // Verify long session still exists
    let result = auth_service.validate_session(long_session.session_id).await;
    assert!(result.is_ok());
}