//! Publishable event system for real-time event publishing

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{Event, EventResult};

/// Publishable event data for real-time publishing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishableEvent {
    pub event_id: Uuid,
    pub event_type: String,
    pub channel: PublishChannel,
    pub data: HashMap<String, serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Publishing channels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublishChannel {
    Global,
    Server(Uuid),
    Entity(Uuid),
    Chat(Uuid),
}

impl PublishableEvent {
    /// Create a new publishable event
    pub fn new(event_type: impl Into<String>, channel: PublishChannel) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: event_type.into(),
            channel,
            data: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Add data to the publishable event
    pub fn with_data(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.data.insert(key.into(), json_value);
        }
        self
    }
}

/// Trait for events that can be published
#[async_trait]
pub trait Publishable: Event {
    /// Generate publishable events
    async fn generate_publishable_events(&self) -> EventResult<Vec<PublishableEvent>>;
}

/// Flow handler for publishable events
pub struct PublishableFlow;

impl PublishableFlow {
    /// Process publishable events
    pub async fn process_events(events: Vec<PublishableEvent>) -> EventResult<()> {
        for event in events {
            tracing::info!(
                "Publishing event: type={}, channel={:?}",
                event.event_type,
                event.channel
            );
            // In a real implementation, this would publish via WebSocket or message queue
        }
        Ok(())
    }

    /// Process publishable event
    pub async fn process_event<T: Publishable>(event: &T) -> EventResult<()> {
        let publishable_events = event.generate_publishable_events().await?;
        Self::process_events(publishable_events).await
    }
}