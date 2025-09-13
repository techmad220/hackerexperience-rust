//! Listenable event system for real-time event listening

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{Event, EventResult};

/// Real-time event data for listeners
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenableEvent {
    pub event_id: Uuid,
    pub event_type: String,
    pub target_type: ListenTargetType,
    pub target_id: Uuid,
    pub data: HashMap<String, serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Types of listening targets
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListenTargetType {
    Server,
    Entity,
    Channel,
    Global,
}

impl ListenableEvent {
    /// Create a new listenable event
    pub fn new(
        event_type: impl Into<String>,
        target_type: ListenTargetType,
        target_id: Uuid,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: event_type.into(),
            target_type,
            target_id,
            data: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Add data to the listenable event
    pub fn with_data(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.data.insert(key.into(), json_value);
        }
        self
    }
}

/// Trait for events that can be listened to in real-time
#[async_trait]
pub trait Listenable: Event {
    /// Generate listenable events for real-time dispatch
    async fn generate_listenable_events(&self) -> EventResult<Vec<ListenableEvent>>;
}

/// Event listener management
pub struct EventListener {
    sender: broadcast::Sender<ListenableEvent>,
    _receiver: broadcast::Receiver<ListenableEvent>,
}

impl EventListener {
    /// Create a new event listener with channel capacity
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = broadcast::channel(capacity);
        Self {
            sender,
            _receiver: receiver,
        }
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<ListenableEvent> {
        self.sender.subscribe()
    }

    /// Send an event to all listeners
    pub async fn send_event(&self, event: ListenableEvent) -> EventResult<()> {
        match self.sender.send(event) {
            Ok(listener_count) => {
                tracing::debug!("Event sent to {} listeners", listener_count);
                Ok(())
            }
            Err(_) => {
                tracing::warn!("No listeners available for event");
                Ok(()) // Not an error if no listeners
            }
        }
    }

    /// Get listener count
    pub fn listener_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

/// Flow handler for listenable events
pub struct ListenableFlow {
    listener: EventListener,
}

impl ListenableFlow {
    /// Create a new listenable flow
    pub fn new(channel_capacity: usize) -> Self {
        Self {
            listener: EventListener::new(channel_capacity),
        }
    }

    /// Process listenable events and send them to listeners
    pub async fn process_events(&self, events: Vec<ListenableEvent>) -> EventResult<()> {
        for event in events {
            self.listener.send_event(event).await?;
        }
        Ok(())
    }

    /// Process listenable event and dispatch to listeners
    pub async fn process_event<T: Listenable>(&self, event: &T) -> EventResult<()> {
        let listenable_events = event.generate_listenable_events().await?;
        self.process_events(listenable_events).await
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<ListenableEvent> {
        self.listener.subscribe()
    }

    /// Get current listener count
    pub fn listener_count(&self) -> usize {
        self.listener.listener_count()
    }
}

impl Default for ListenableFlow {
    fn default() -> Self {
        Self::new(1000) // Default channel capacity
    }
}

/// Utility functions for listenable event generation
pub mod utils {
    use super::*;

    /// Create a server event for listeners
    pub fn server_event(
        server_id: Uuid,
        event_type: impl Into<String>,
        message: impl Into<String>,
    ) -> ListenableEvent {
        ListenableEvent::new(event_type, ListenTargetType::Server, server_id)
            .with_data("message", message.into())
    }

    /// Create an entity event for listeners
    pub fn entity_event(
        entity_id: Uuid,
        event_type: impl Into<String>,
        description: impl Into<String>,
    ) -> ListenableEvent {
        ListenableEvent::new(event_type, ListenTargetType::Entity, entity_id)
            .with_data("description", description.into())
    }

    /// Create a global event for all listeners
    pub fn global_event(
        event_type: impl Into<String>,
        announcement: impl Into<String>,
    ) -> ListenableEvent {
        // Use a nil UUID for global events
        let global_id = Uuid::nil();
        ListenableEvent::new(event_type, ListenTargetType::Global, global_id)
            .with_data("announcement", announcement.into())
    }

    /// Create a process update event
    pub fn process_update_event(
        server_id: Uuid,
        process_id: Uuid,
        process_type: impl Into<String>,
        progress: f32,
    ) -> ListenableEvent {
        ListenableEvent::new("process_update", ListenTargetType::Server, server_id)
            .with_data("process_id", process_id.to_string())
            .with_data("process_type", process_type.into())
            .with_data("progress", progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[test]
    fn test_listenable_event_creation() {
        let server_id = Uuid::new_v4();
        let event = ListenableEvent::new("connection_established", ListenTargetType::Server, server_id)
            .with_data("source", "test");

        assert_eq!(event.event_type, "connection_established");
        assert_eq!(event.target_id, server_id);
        assert!(matches!(event.target_type, ListenTargetType::Server));
        assert!(event.data.contains_key("source"));
    }

    #[tokio::test]
    async fn test_event_listener() {
        let listener = EventListener::new(10);
        let mut receiver = listener.subscribe();

        let event = ListenableEvent::new("test_event", ListenTargetType::Global, Uuid::nil());
        
        // Send event
        listener.send_event(event.clone()).await.unwrap();

        // Receive event
        let received = timeout(Duration::from_millis(100), receiver.recv()).await;
        assert!(received.is_ok());
        
        let received_event = received.unwrap().unwrap();
        assert_eq!(received_event.event_type, event.event_type);
    }

    #[tokio::test]
    async fn test_listenable_flow() {
        let flow = ListenableFlow::new(10);
        let mut receiver = flow.subscribe();

        let events = vec![
            ListenableEvent::new("test1", ListenTargetType::Global, Uuid::nil()),
            ListenableEvent::new("test2", ListenTargetType::Global, Uuid::nil()),
        ];

        // Process events
        flow.process_events(events).await.unwrap();

        // Should receive both events
        let event1 = timeout(Duration::from_millis(100), receiver.recv()).await;
        let event2 = timeout(Duration::from_millis(100), receiver.recv()).await;
        
        assert!(event1.is_ok());
        assert!(event2.is_ok());
    }

    #[test]
    fn test_listenable_utils() {
        let server_id = Uuid::new_v4();
        let event = utils::server_event(server_id, "connection_lost", "Server disconnected");

        assert_eq!(event.event_type, "connection_lost");
        assert_eq!(event.target_id, server_id);
        assert!(event.data.contains_key("message"));

        let global_event = utils::global_event("maintenance", "Server maintenance starting");
        assert!(matches!(global_event.target_type, ListenTargetType::Global));
        assert!(global_event.data.contains_key("announcement"));
    }
}