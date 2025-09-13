//! Notification events

use he_events::{Event, EventResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::model::NotificationClass;

/// Notification added event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAddedEvent {
    pub notification_id: Uuid,
    pub account_id: Uuid,
    pub class: NotificationClass,
    pub code: String,
    pub data: HashMap<String, serde_json::Value>,
}

impl NotificationAddedEvent {
    pub fn new(
        notification_id: Uuid,
        account_id: Uuid,
        class: NotificationClass,
        code: String,
        data: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            notification_id,
            account_id,
            class,
            code,
            data,
        }
    }
}

impl Event for NotificationAddedEvent {
    fn name(&self) -> &'static str {
        "notification_added"
    }

    fn version(&self) -> u32 {
        1
    }
}

/// Notification read event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationReadEvent {
    pub notification_id: Uuid,
    pub account_id: Uuid,
    pub class: NotificationClass,
}

impl Event for NotificationReadEvent {
    fn name(&self) -> &'static str {
        "notification_read"
    }

    fn version(&self) -> u32 {
        1
    }
}

/// Event handler for notifications
#[async_trait::async_trait]
pub trait NotificationEventHandler {
    async fn handle_notification_added(&self, event: &NotificationAddedEvent) -> EventResult<()>;
    async fn handle_notification_read(&self, event: &NotificationReadEvent) -> EventResult<()>;
}