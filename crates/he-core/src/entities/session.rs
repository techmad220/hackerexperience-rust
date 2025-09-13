use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::{UserId, HeResult};

// Mapping from PHP Session.class.php
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub user_id: Option<UserId>,
    pub language: String,
    pub query_count: i32,
    pub buffer_query: i32,
    pub exec_time: f64,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub is_active: bool,
}

impl Session {
    pub fn new(session_id: String, ip_address: String) -> Self {
        let now = Utc::now();
        Self {
            session_id,
            user_id: None,
            language: "en_US".to_string(), // Default language
            query_count: 0,
            buffer_query: 0,
            exec_time: 0.0,
            created_at: now,
            last_activity: now,
            ip_address,
            user_agent: None,
            is_active: true,
        }
    }
    
    pub fn authenticate(&mut self, user_id: UserId) {
        self.user_id = Some(user_id);
        self.update_activity();
    }
    
    pub fn logout(&mut self) {
        self.user_id = None;
        self.is_active = false;
    }
    
    pub fn update_activity(&mut self) {
        self.last_activity = Utc::now();
    }
    
    pub fn increment_query_count(&mut self) {
        self.query_count += 1;
    }
    
    pub fn is_authenticated(&self) -> bool {
        self.user_id.is_some()
    }
    
    pub fn is_expired(&self, timeout_seconds: i64) -> bool {
        let now = Utc::now();
        (now.timestamp() - self.last_activity.timestamp()) > timeout_seconds
    }
    
    pub fn set_language(&mut self, language: String) {
        self.language = language;
    }
}

// Session message system - for flash messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub message: String,
    pub message_type: MessageType,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Notice,
    Error,
    Success,
    Warning,
}

impl SessionMessage {
    pub fn new(message: String, message_type: MessageType) -> Self {
        Self {
            message,
            message_type,
            created_at: Utc::now(),
        }
    }
    
    pub fn notice(message: String) -> Self {
        Self::new(message, MessageType::Notice)
    }
    
    pub fn error(message: String) -> Self {
        Self::new(message, MessageType::Error)
    }
    
    pub fn success(message: String) -> Self {
        Self::new(message, MessageType::Success)
    }
    
    pub fn warning(message: String) -> Self {
        Self::new(message, MessageType::Warning)
    }
}

// Session store for managing active sessions
#[derive(Debug, Clone)]
pub struct SessionStore {
    sessions: std::collections::HashMap<String, Session>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: std::collections::HashMap::new(),
        }
    }
    
    pub fn create_session(&mut self, ip_address: String) -> String {
        let session_id = uuid::Uuid::new_v4().to_string();
        let session = Session::new(session_id.clone(), ip_address);
        self.sessions.insert(session_id.clone(), session);
        session_id
    }
    
    pub fn get_session(&self, session_id: &str) -> Option<&Session> {
        self.sessions.get(session_id)
    }
    
    pub fn get_session_mut(&mut self, session_id: &str) -> Option<&mut Session> {
        self.sessions.get_mut(session_id)
    }
    
    pub fn destroy_session(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
    }
    
    pub fn cleanup_expired(&mut self, timeout_seconds: i64) {
        self.sessions.retain(|_, session| !session.is_expired(timeout_seconds));
    }
    
    pub fn get_user_sessions(&self, user_id: UserId) -> Vec<&Session> {
        self.sessions.values()
            .filter(|session| session.user_id == Some(user_id))
            .collect()
    }
    
    pub fn logout_user(&mut self, user_id: UserId) {
        for session in self.sessions.values_mut() {
            if session.user_id == Some(user_id) {
                session.logout();
            }
        }
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}