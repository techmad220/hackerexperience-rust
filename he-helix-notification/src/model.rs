//! Notification model types and structures

pub mod code;
pub mod notification;

pub use code::{NotificationCode, CodeRegistry};
pub use notification::{
    Notification, NotificationClass, ServerNotification, 
    ChatNotification, EntityNotification
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Base notification structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseNotification {
    pub notification_id: Uuid,
    pub account_id: Uuid,
    pub class: NotificationClass,
    pub code: String,
    pub data: HashMap<String, serde_json::Value>,
    pub is_read: bool,
    pub creation_time: DateTime<Utc>,
}

impl BaseNotification {
    /// Create a new notification
    pub fn new(
        account_id: Uuid,
        class: NotificationClass,
        code: impl Into<String>,
        data: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            notification_id: Uuid::new_v4(),
            account_id,
            class,
            code: code.into(),
            data,
            is_read: false,
            creation_time: Utc::now(),
        }
    }

    /// Mark notification as read
    pub fn mark_read(&mut self) {
        self.is_read = true;
    }

    /// Check if notification is read
    pub fn is_read(&self) -> bool {
        self.is_read
    }

    /// Get notification age
    pub fn age(&self) -> chrono::Duration {
        Utc::now() - self.creation_time
    }
}

/// Notification creation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotificationParams {
    pub account_id: Uuid,
    pub class: NotificationClass,
    pub code: String,
    pub data: HashMap<String, serde_json::Value>,
    pub target_id: Option<Uuid>, // server_id, chat_id, entity_id depending on class
}

/// Notification query parameters
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotificationQuery {
    pub account_id: Option<Uuid>,
    pub class: Option<NotificationClass>,
    pub code: Option<String>,
    pub is_read: Option<bool>,
    pub target_id: Option<Uuid>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl NotificationQuery {
    /// Create query for account notifications
    pub fn for_account(account_id: Uuid) -> Self {
        Self {
            account_id: Some(account_id),
            ..Default::default()
        }
    }

    /// Add class filter
    pub fn with_class(mut self, class: NotificationClass) -> Self {
        self.class = Some(class);
        self
    }

    /// Add unread filter
    pub fn unread_only(mut self) -> Self {
        self.is_read = Some(false);
        self
    }

    /// Add limit
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_creation() {
        let account_id = Uuid::new_v4();
        let data = HashMap::new();
        
        let notification = BaseNotification::new(
            account_id,
            NotificationClass::Server,
            "connection_lost",
            data,
        );

        assert_eq!(notification.account_id, account_id);
        assert_eq!(notification.class, NotificationClass::Server);
        assert_eq!(notification.code, "connection_lost");
        assert!(!notification.is_read());
        assert!(!notification.notification_id.is_nil());
    }

    #[test]
    fn test_notification_read() {
        let account_id = Uuid::new_v4();
        let mut notification = BaseNotification::new(
            account_id,
            NotificationClass::Chat,
            "new_message",
            HashMap::new(),
        );

        assert!(!notification.is_read());
        notification.mark_read();
        assert!(notification.is_read());
    }

    #[test]
    fn test_notification_query_builder() {
        let account_id = Uuid::new_v4();
        
        let query = NotificationQuery::for_account(account_id)
            .with_class(NotificationClass::Server)
            .unread_only()
            .limit(10);

        assert_eq!(query.account_id, Some(account_id));
        assert_eq!(query.class, Some(NotificationClass::Server));
        assert_eq!(query.is_read, Some(false));
        assert_eq!(query.limit, Some(10));
    }
}