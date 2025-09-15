use actix::prelude::*;
use std::time::Duration;
use tokio::time::timeout;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::common::TestFixtures;
use crate::{assert_ok, assert_err};

// Mock actor implementations for testing

#[derive(Message)]
#[rtype(result = "Result<String, ActorError>")]
pub struct TestMessage {
    pub content: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct TestNotification {
    pub user_id: u64,
    pub message: String,
}

#[derive(Message)] 
#[rtype(result = "Result<ProcessStatus, ActorError>")]
pub struct StartProcess {
    pub process_type: String,
    pub target: String,
    pub duration: Duration,
}

#[derive(Message)]
#[rtype(result = "Result<(), ActorError>")]
pub struct StopProcess {
    pub process_id: u64,
}

#[derive(Debug)]
pub enum ActorError {
    ProcessNotFound,
    InvalidInput,
    DatabaseError,
    NetworkError,
}

#[derive(Debug, PartialEq)]
pub struct ProcessStatus {
    pub id: u64,
    pub status: String,
    pub progress: f32,
    pub remaining_time: Duration,
}

// Entity Actor - handles game entity management
pub struct EntityActor {
    pub entity_id: u64,
    pub entity_data: HashMap<String, Value>,
}

impl EntityActor {
    pub fn new(entity_id: u64) -> Self {
        Self {
            entity_id,
            entity_data: HashMap::new(),
        }
    }

    pub fn update_component(&mut self, component: &str, value: Value) {
        self.entity_data.insert(component.to_string(), value);
    }

    pub fn get_component(&self, component: &str) -> Option<&Value> {
        self.entity_data.get(component)
    }
}

impl Actor for EntityActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("EntityActor {} started", self.entity_id);
    }
}

impl Handler<TestMessage> for EntityActor {
    type Result = Result<String, ActorError>;

    fn handle(&mut self, msg: TestMessage, _ctx: &mut Self::Context) -> Self::Result {
        if msg.content.is_empty() {
            return Err(ActorError::InvalidInput);
        }
        Ok(format!("Entity {} processed: {}", self.entity_id, msg.content))
    }
}

// Process Actor - handles game processes (cracking, uploading, etc.)
pub struct ProcessActor {
    pub process_id: u64,
    pub process_type: String,
    pub status: String,
    pub progress: f32,
    pub remaining_time: Duration,
}

impl ProcessActor {
    pub fn new(process_id: u64, process_type: String, duration: Duration) -> Self {
        Self {
            process_id,
            process_type,
            status: "running".to_string(),
            progress: 0.0,
            remaining_time: duration,
        }
    }

    pub fn update_progress(&mut self, progress: f32) {
        self.progress = progress.min(100.0).max(0.0);
        if self.progress >= 100.0 {
            self.status = "completed".to_string();
            self.remaining_time = Duration::from_secs(0);
        }
    }
}

impl Actor for ProcessActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("ProcessActor {} started for type: {}", self.process_id, self.process_type);
        
        // Simulate process progression
        ctx.run_interval(Duration::from_millis(100), |act, _ctx| {
            if act.status == "running" {
                act.progress += 1.0;
                act.update_progress(act.progress);
            }
        });
    }
}

impl Handler<StartProcess> for ProcessActor {
    type Result = Result<ProcessStatus, ActorError>;

    fn handle(&mut self, msg: StartProcess, _ctx: &mut Self::Context) -> Self::Result {
        if msg.process_type.is_empty() {
            return Err(ActorError::InvalidInput);
        }

        self.process_type = msg.process_type;
        self.remaining_time = msg.duration;
        self.status = "running".to_string();
        self.progress = 0.0;

        Ok(ProcessStatus {
            id: self.process_id,
            status: self.status.clone(),
            progress: self.progress,
            remaining_time: self.remaining_time,
        })
    }
}

impl Handler<StopProcess> for ProcessActor {
    type Result = Result<(), ActorError>;

    fn handle(&mut self, msg: StopProcess, _ctx: &mut Self::Context) -> Self::Result {
        if msg.process_id != self.process_id {
            return Err(ActorError::ProcessNotFound);
        }

        self.status = "stopped".to_string();
        self.progress = 0.0;
        Ok(())
    }
}

// Notification Actor - handles user notifications
pub struct NotificationActor {
    pub notifications: Vec<(u64, String)>,
}

impl NotificationActor {
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
        }
    }
}

impl Actor for NotificationActor {
    type Context = Context<Self>;
}

impl Handler<TestNotification> for NotificationActor {
    type Result = ();

    fn handle(&mut self, msg: TestNotification, _ctx: &mut Self::Context) -> Self::Result {
        self.notifications.push((msg.user_id, msg.message));
        println!("Notification sent to user {}: {}", msg.user_id, msg.message);
    }
}

// Network Actor - handles network connections and communications  
pub struct NetworkActor {
    pub connections: HashMap<String, String>,
    pub connection_count: usize,
}

impl NetworkActor {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            connection_count: 0,
        }
    }

    pub fn add_connection(&mut self, ip: &str, connection_type: &str) {
        self.connections.insert(ip.to_string(), connection_type.to_string());
        self.connection_count += 1;
    }

    pub fn remove_connection(&mut self, ip: &str) -> bool {
        if self.connections.remove(ip).is_some() {
            self.connection_count -= 1;
            true
        } else {
            false
        }
    }
}

impl Actor for NetworkActor {
    type Context = Context<Self>;
}

// Tests

#[actix::test]
async fn test_entity_actor_creation() {
    let entity_id = 123;
    let mut entity = EntityActor::new(entity_id);
    
    assert_eq!(entity.entity_id, entity_id);
    assert!(entity.entity_data.is_empty());

    // Test component management
    entity.update_component("health", json!(100));
    entity.update_component("position", json!({"x": 10, "y": 20}));

    assert_eq!(entity.get_component("health"), Some(&json!(100)));
    assert_eq!(entity.get_component("position"), Some(&json!({"x": 10, "y": 20})));
    assert_eq!(entity.get_component("nonexistent"), None);
}

#[actix::test]
async fn test_entity_actor_message_handling() {
    let entity = EntityActor::new(456).start();
    
    // Test valid message
    let msg = TestMessage {
        content: "Hello, entity!".to_string(),
    };
    
    let result = entity.send(msg).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.is_ok());
    assert!(response.unwrap().contains("Entity 456 processed: Hello, entity!"));

    // Test invalid message (empty content)
    let msg = TestMessage {
        content: String::new(),
    };
    
    let result = entity.send(msg).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(matches!(response, Err(ActorError::InvalidInput)));
}

#[actix::test]
async fn test_process_actor_lifecycle() {
    let process_id = 789;
    let duration = Duration::from_millis(500);
    let mut process = ProcessActor::new(process_id, "cracker".to_string(), duration);
    
    assert_eq!(process.process_id, process_id);
    assert_eq!(process.process_type, "cracker");
    assert_eq!(process.status, "running");
    assert_eq!(process.progress, 0.0);

    // Test progress updates
    process.update_progress(50.0);
    assert_eq!(process.progress, 50.0);
    assert_eq!(process.status, "running");

    // Test completion
    process.update_progress(100.0);
    assert_eq!(process.progress, 100.0);
    assert_eq!(process.status, "completed");
    assert_eq!(process.remaining_time, Duration::from_secs(0));

    // Test progress bounds
    process.update_progress(150.0);
    assert_eq!(process.progress, 100.0);

    let mut process2 = ProcessActor::new(999, "test".to_string(), duration);
    process2.update_progress(-10.0);
    assert_eq!(process2.progress, 0.0);
}

#[actix::test]
async fn test_process_actor_start_process() {
    let process_actor = ProcessActor::new(1, "old_type".to_string(), Duration::from_secs(1)).start();
    
    // Test starting a new process
    let msg = StartProcess {
        process_type: "file_download".to_string(),
        target: "192.168.1.100".to_string(),
        duration: Duration::from_secs(10),
    };
    
    let result = process_actor.send(msg).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.is_ok());
    
    let status = response.unwrap();
    assert_eq!(status.id, 1);
    assert_eq!(status.status, "running");
    assert_eq!(status.progress, 0.0);
    assert_eq!(status.remaining_time, Duration::from_secs(10));

    // Test invalid process type
    let msg = StartProcess {
        process_type: String::new(),
        target: "192.168.1.100".to_string(), 
        duration: Duration::from_secs(10),
    };
    
    let result = process_actor.send(msg).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(matches!(response, Err(ActorError::InvalidInput)));
}

#[actix::test]
async fn test_process_actor_stop_process() {
    let process_id = 555;
    let process_actor = ProcessActor::new(process_id, "test".to_string(), Duration::from_secs(1)).start();
    
    // Test stopping the correct process
    let msg = StopProcess { process_id };
    let result = process_actor.send(msg).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_ok());

    // Test stopping wrong process ID
    let msg = StopProcess { process_id: 999 };
    let result = process_actor.send(msg).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(matches!(response, Err(ActorError::ProcessNotFound)));
}

#[actix::test]
async fn test_notification_actor() {
    let notification_actor = NotificationActor::new().start();
    
    // Test sending notification
    let msg = TestNotification {
        user_id: 123,
        message: "Process completed successfully!".to_string(),
    };
    
    let result = timeout(Duration::from_secs(1), notification_actor.send(msg)).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_ok());

    // Test multiple notifications
    for i in 0..5 {
        let msg = TestNotification {
            user_id: i,
            message: format!("Notification {}", i),
        };
        let result = notification_actor.send(msg).await;
        assert!(result.is_ok());
    }
}

#[actix::test]
async fn test_network_actor() {
    let mut network = NetworkActor::new();
    
    assert_eq!(network.connection_count, 0);
    assert!(network.connections.is_empty());

    // Test adding connections
    network.add_connection("192.168.1.100", "ssh");
    network.add_connection("192.168.1.101", "ftp");
    
    assert_eq!(network.connection_count, 2);
    assert_eq!(network.connections.get("192.168.1.100"), Some(&"ssh".to_string()));
    assert_eq!(network.connections.get("192.168.1.101"), Some(&"ftp".to_string()));

    // Test removing connections
    assert!(network.remove_connection("192.168.1.100"));
    assert_eq!(network.connection_count, 1);
    assert!(!network.connections.contains_key("192.168.1.100"));

    // Test removing non-existent connection
    assert!(!network.remove_connection("192.168.1.999"));
    assert_eq!(network.connection_count, 1);
}

#[actix::test]
async fn test_actor_system_integration() {
    // Test interaction between multiple actors
    let entity_actor = EntityActor::new(1).start();
    let process_actor = ProcessActor::new(1, "cracker".to_string(), Duration::from_secs(5)).start();
    let notification_actor = NotificationActor::new().start();

    // Start a process
    let start_msg = StartProcess {
        process_type: "password_crack".to_string(),
        target: "target_server".to_string(),
        duration: Duration::from_secs(3),
    };

    let process_result = process_actor.send(start_msg).await;
    assert!(process_result.is_ok());
    assert!(process_result.unwrap().is_ok());

    // Send notification about process start
    let notification_msg = TestNotification {
        user_id: 1,
        message: "Password cracking process started".to_string(),
    };

    let notification_result = notification_actor.send(notification_msg).await;
    assert!(notification_result.is_ok());

    // Update entity with process information
    let entity_msg = TestMessage {
        content: "process_started".to_string(),
    };

    let entity_result = entity_actor.send(entity_msg).await;
    assert!(entity_result.is_ok());
    assert!(entity_result.unwrap().is_ok());
}

#[actix::test]
async fn test_actor_failure_recovery() {
    // Test actor behavior under failure conditions
    let process_actor = ProcessActor::new(1, "test".to_string(), Duration::from_secs(1)).start();

    // Send multiple conflicting messages rapidly
    let futures: Vec<_> = (0..10).map(|i| {
        let actor = process_actor.clone();
        async move {
            let msg = StartProcess {
                process_type: format!("process_{}", i),
                target: format!("target_{}", i),
                duration: Duration::from_millis(100 * i),
            };
            actor.send(msg).await
        }
    }).collect();

    let results = futures::future::join_all(futures).await;
    
    // All messages should be processed successfully (actor handles them sequentially)
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
}

#[actix::test]
async fn test_actor_message_timeout() {
    let entity_actor = EntityActor::new(1).start();

    // Test message with timeout
    let msg = TestMessage {
        content: "timeout_test".to_string(),
    };

    let result = timeout(Duration::from_millis(100), entity_actor.send(msg)).await;
    
    // Message should complete within timeout
    assert!(result.is_ok());
    assert!(result.unwrap().is_ok());
}

#[actix::test]
async fn test_concurrent_actor_operations() {
    let num_actors = 5;
    let mut actors = Vec::new();

    // Create multiple entity actors
    for i in 0..num_actors {
        actors.push(EntityActor::new(i as u64).start());
    }

    // Send messages to all actors concurrently
    let futures: Vec<_> = actors.iter().enumerate().map(|(i, actor)| {
        let msg = TestMessage {
            content: format!("message_to_actor_{}", i),
        };
        actor.send(msg)
    }).collect();

    let results = futures::future::join_all(futures).await;

    // All messages should be processed successfully
    for (i, result) in results.into_iter().enumerate() {
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_ok());
        assert!(response.unwrap().contains(&format!("Entity {} processed", i)));
    }
}

// Performance tests
#[actix::test]
async fn test_actor_throughput() {
    let entity_actor = EntityActor::new(1).start();
    let num_messages = 1000;

    let start_time = std::time::Instant::now();

    let futures: Vec<_> = (0..num_messages).map(|i| {
        let actor = entity_actor.clone();
        async move {
            let msg = TestMessage {
                content: format!("message_{}", i),
            };
            actor.send(msg).await
        }
    }).collect();

    let results = futures::future::join_all(futures).await;
    let duration = start_time.elapsed();

    // Verify all messages processed successfully
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }

    println!("Processed {} messages in {:?} ({:.2} msg/sec)", 
        num_messages, duration, num_messages as f64 / duration.as_secs_f64());

    // Performance assertion: should handle at least 1000 messages per second
    assert!(duration.as_secs_f64() < 1.0, "Actor throughput too low: {:?}", duration);
}

// Memory leak detection test
#[actix::test]
async fn test_actor_memory_management() {
    // Create and destroy actors repeatedly to test for memory leaks
    for i in 0..100 {
        let actor = EntityActor::new(i).start();
        
        let msg = TestMessage {
            content: format!("test_{}", i),
        };
        
        let result = actor.send(msg).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());

        // Actor should be automatically cleaned up when going out of scope
    }
}