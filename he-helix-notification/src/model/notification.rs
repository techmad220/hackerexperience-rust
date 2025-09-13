//! Notification types and class definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Notification classes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationClass {
    Server,
    Chat,
    Entity,
}

impl NotificationClass {
    /// Get all valid notification classes
    pub fn all() -> &'static [NotificationClass] {
        &[NotificationClass::Server, NotificationClass::Chat, NotificationClass::Entity]
    }

    /// Get class as string
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationClass::Server => "server",
            NotificationClass::Chat => "chat",
            NotificationClass::Entity => "entity",
        }
    }
}

/// Generic notification trait
pub trait Notification {
    fn notification_id(&self) -> Uuid;
    fn account_id(&self) -> Uuid;
    fn class(&self) -> NotificationClass;
    fn code(&self) -> &str;
    fn data(&self) -> &HashMap<String, serde_json::Value>;
    fn is_read(&self) -> bool;
    fn creation_time(&self) -> DateTime<Utc>;
    
    fn mark_read(&mut self);
    fn render_data(&self) -> HashMap<String, serde_json::Value>;
}

/// Server notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerNotification {
    pub notification_id: Uuid,
    pub account_id: Uuid,
    pub server_id: Uuid,
    pub code: String,
    pub data: HashMap<String, serde_json::Value>,
    pub is_read: bool,
    pub creation_time: DateTime<Utc>,
}

impl ServerNotification {
    /// Create a new server notification
    pub fn new(
        account_id: Uuid,
        server_id: Uuid,
        code: impl Into<String>,
        data: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            notification_id: Uuid::new_v4(),
            account_id,
            server_id,
            code: code.into(),
            data,
            is_read: false,
            creation_time: Utc::now(),
        }
    }
}

impl Notification for ServerNotification {
    fn notification_id(&self) -> Uuid { self.notification_id }
    fn account_id(&self) -> Uuid { self.account_id }
    fn class(&self) -> NotificationClass { NotificationClass::Server }
    fn code(&self) -> &str { &self.code }
    fn data(&self) -> &HashMap<String, serde_json::Value> { &self.data }
    fn is_read(&self) -> bool { self.is_read }
    fn creation_time(&self) -> DateTime<Utc> { self.creation_time }
    
    fn mark_read(&mut self) {
        self.is_read = true;
    }
    
    fn render_data(&self) -> HashMap<String, serde_json::Value> {
        let mut rendered = self.data.clone();
        rendered.insert("server_id".to_string(), serde_json::to_value(&self.server_id).unwrap());
        rendered.insert("notification_type".to_string(), "server".into());
        rendered
    }
}

/// Chat notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatNotification {
    pub notification_id: Uuid,
    pub account_id: Uuid,
    pub chat_id: Uuid,
    pub code: String,
    pub data: HashMap<String, serde_json::Value>,
    pub is_read: bool,
    pub creation_time: DateTime<Utc>,
}

impl ChatNotification {
    /// Create a new chat notification
    pub fn new(
        account_id: Uuid,
        chat_id: Uuid,
        code: impl Into<String>,
        data: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            notification_id: Uuid::new_v4(),
            account_id,
            chat_id,
            code: code.into(),
            data,
            is_read: false,
            creation_time: Utc::now(),
        }
    }
}

impl Notification for ChatNotification {
    fn notification_id(&self) -> Uuid { self.notification_id }
    fn account_id(&self) -> Uuid { self.account_id }
    fn class(&self) -> NotificationClass { NotificationClass::Chat }
    fn code(&self) -> &str { &self.code }
    fn data(&self) -> &HashMap<String, serde_json::Value> { &self.data }
    fn is_read(&self) -> bool { self.is_read }
    fn creation_time(&self) -> DateTime<Utc> { self.creation_time }
    
    fn mark_read(&mut self) {
        self.is_read = true;
    }
    
    fn render_data(&self) -> HashMap<String, serde_json::Value> {
        let mut rendered = self.data.clone();
        rendered.insert("chat_id".to_string(), serde_json::to_value(&self.chat_id).unwrap());
        rendered.insert("notification_type".to_string(), "chat".into());
        rendered
    }
}

/// Entity notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityNotification {
    pub notification_id: Uuid,
    pub account_id: Uuid,
    pub entity_id: Uuid,
    pub code: String,
    pub data: HashMap<String, serde_json::Value>,
    pub is_read: bool,
    pub creation_time: DateTime<Utc>,
}

impl EntityNotification {
    /// Create a new entity notification
    pub fn new(
        account_id: Uuid,
        entity_id: Uuid,
        code: impl Into<String>,
        data: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            notification_id: Uuid::new_v4(),
            account_id,
            entity_id,
            code: code.into(),
            data,
            is_read: false,
            creation_time: Utc::now(),
        }
    }
}

impl Notification for EntityNotification {
    fn notification_id(&self) -> Uuid { self.notification_id }
    fn account_id(&self) -> Uuid { self.account_id }
    fn class(&self) -> NotificationClass { NotificationClass::Entity }
    fn code(&self) -> &str { &self.code }
    fn data(&self) -> &HashMap<String, serde_json::Value> { &self.data }
    fn is_read(&self) -> bool { self.is_read }
    fn creation_time(&self) -> DateTime<Utc> { self.creation_time }
    
    fn mark_read(&mut self) {
        self.is_read = true;
    }
    
    fn render_data(&self) -> HashMap<String, serde_json::Value> {
        let mut rendered = self.data.clone();
        rendered.insert("entity_id".to_string(), serde_json::to_value(&self.entity_id).unwrap());
        rendered.insert("notification_type".to_string(), "entity".into());
        rendered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_class_serialization() {
        let server_class = NotificationClass::Server;
        let serialized = serde_json::to_string(&server_class).unwrap();
        assert_eq!(serialized, "\"server\"");
        
        let deserialized: NotificationClass = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, NotificationClass::Server);
    }

    #[test]
    fn test_server_notification() {
        let account_id = Uuid::new_v4();
        let server_id = Uuid::new_v4();
        let mut data = HashMap::new();
        data.insert("message".to_string(), "Connection established".into());
        
        let mut notification = ServerNotification::new(
            account_id, server_id, "connection_established", data
        );
        
        assert_eq!(notification.account_id(), account_id);
        assert_eq!(notification.class(), NotificationClass::Server);
        assert_eq!(notification.code(), "connection_established");
        assert!(!notification.is_read());
        
        notification.mark_read();
        assert!(notification.is_read());
        
        let rendered = notification.render_data();
        assert!(rendered.contains_key("server_id"));
        assert!(rendered.contains_key("notification_type"));
    }
}