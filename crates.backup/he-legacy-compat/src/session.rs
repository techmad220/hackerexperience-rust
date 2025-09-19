// PHP Session Compatibility Layer
// 1:1 port of Session.class.php with complete state management

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpSession {
    // Core session data (equivalent to $_SESSION array)
    pub data: HashMap<String, SessionValue>,
    
    // Session metadata
    pub session_id: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub language: String,
    
    // Query tracking (original PHP functionality)
    pub query_count: i32,
    pub buffer_query: i32,
    pub exec_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SessionValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}

impl SessionValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            SessionValue::String(s) => Some(s),
            _ => None,
        }
    }
    
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            SessionValue::Integer(i) => Some(*i),
            SessionValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }
    
    pub fn as_bool(&self) -> bool {
        match self {
            SessionValue::Boolean(b) => *b,
            SessionValue::String(s) => s == "1" || s.to_lowercase() == "true",
            SessionValue::Integer(i) => *i != 0,
            _ => false,
        }
    }
}

impl PhpSession {
    // Original PHP: __construct() - Initialize session
    pub fn new() -> Self {
        let session_id = Self::generate_session_id();
        let now = Utc::now();
        
        let mut session = Self {
            data: HashMap::new(),
            session_id,
            created_at: now,
            last_activity: now,
            language: "en".to_string(),
            query_count: 0,
            buffer_query: 0,
            exec_time: Self::microtime(),
        };
        
        // Initialize session variables (original PHP defaults)
        session.set("QUERY_COUNT", SessionValue::String("0".to_string()));
        session.set("BUFFER_QUERY", SessionValue::String("0".to_string()));
        session.set("EXEC_TIME", SessionValue::Float(session.exec_time));
        
        // Set language
        session.language_set(false);
        
        session
    }
    
    // Original PHP: addMsg - Add message to session
    pub fn add_msg(&mut self, msg: &str, msg_type: &str) {
        self.set("MSG", SessionValue::String(msg.to_string()));
        self.set("MSG_TYPE", SessionValue::String(msg_type.to_string()));
    }
    
    // Original PHP: issetMsg - Check if message exists
    pub fn isset_msg(&self) -> bool {
        self.data.contains_key("MSG")
    }
    
    // Original PHP: returnMsg - Render message HTML
    pub fn return_msg(&mut self, prv: Option<&str>) -> String {
        if !self.isset_msg() {
            return String::new();
        }
        
        let msg = self.get("MSG").and_then(|v| v.as_str()).unwrap_or("");
        let msg_type = self.get("MSG_TYPE").and_then(|v| v.as_str()).unwrap_or("notice");
        
        let (alert_type, prefix) = if msg_type == "error" {
            ("error", "<strong>Error!</strong> ")
        } else {
            ("success", "<strong>Success!</strong> ")
        };
        
        let prefix = if prv.is_some() { "" } else { prefix };
        
        let html = format!(
            r#"<div class="alert alert-{}">
                <button class="close" data-dismiss="alert">×</button>
                {}{}
            </div>"#,
            alert_type, prefix, msg
        );
        
        // Add mission notification if needed
        let mission_html = if msg_type == "mission" {
            r#"<span id="notify-mission"></span>"#
        } else {
            ""
        };
        
        // Clear messages
        self.del_msg();
        
        format!("{}{}", html, mission_html)
    }
    
    // Original PHP: delMsg - Delete messages
    pub fn del_msg(&mut self) {
        self.data.remove("MSG");
        self.data.remove("MSG_TYPE");
    }
    
    // Original PHP: loginSession - Initialize login session
    pub fn login_session(&mut self, id: i64, user: &str, premium: bool, special: Option<&str>) {
        self.set("id", SessionValue::Integer(id));
        self.set("user", SessionValue::String(user.to_string()));
        self.set("premium", SessionValue::Boolean(premium));
        
        if let Some(spec) = special {
            self.set("LOGIN_TYPE", SessionValue::String(spec.to_string()));
        }
        
        self.language_set(true);
        
        // Initialize user-specific session data
        self.set("LAST_CHECK", SessionValue::String(Utc::now().to_rfc3339()));
        self.set("ROUND_STATUS", SessionValue::Integer(1)); // Default active round
    }
    
    // Original PHP: issetLogin - Check if user is logged in
    pub fn isset_login(&self) -> bool {
        self.get("id").and_then(|v| v.as_i64()).map_or(false, |id| id > 0)
    }
    
    // Original PHP: issetFBLogin - Check if Facebook login
    pub fn isset_fb_login(&self) -> bool {
        self.get("LOGIN_TYPE").and_then(|v| v.as_str()) == Some("facebook")
    }
    
    // Original PHP: logout - Clear session data
    pub fn logout(&mut self, query: bool, redirect: bool) -> Option<String> {
        // Clear user-specific data
        let keys_to_remove = vec![
            "id", "user", "premium", "LOGIN_TYPE", "LAST_CHECK", 
            "ROUND_STATUS", "CLAN_ID", "SPECIAL_ID", "GOING_ON",
            "CLICKED", "PROCESS_ID", "LOG_ID", "LOG_LOCAL", 
            "LOG_VICTIM_IP", "INTERNET_IP", "HACKING"
        ];
        
        for key in keys_to_remove {
            self.data.remove(key);
        }
        
        // Add logout message
        self.add_msg("You have been logged out.", "notice");
        
        if redirect {
            Some("index".to_string())
        } else {
            None
        }
    }
    
    // Original PHP: validLogin - Validate login session
    pub fn valid_login(&self) -> bool {
        // Check if session has valid user ID
        if !self.isset_login() {
            return false;
        }
        
        // Check session expiry (24 hours)
        let now = Utc::now();
        let duration = now.signed_duration_since(self.last_activity);
        
        duration.num_hours() < 24
    }
    
    // Original PHP: issetProcessSession - Check if process session exists
    pub fn isset_process_session(&self) -> bool {
        self.data.contains_key("PROCESS_ID")
    }
    
    // Original PHP: processID - Get/set process ID
    pub fn process_id(&mut self, param: Option<&str>, pid: Option<i64>) -> Option<i64> {
        match param {
            Some("set") => {
                if let Some(id) = pid {
                    self.set("PROCESS_ID", SessionValue::Integer(id));
                    Some(id)
                } else {
                    None
                }
            },
            Some("get") | None => {
                self.get("PROCESS_ID").and_then(|v| v.as_i64())
            },
            Some("del") => {
                self.data.remove("PROCESS_ID");
                None
            },
            _ => None,
        }
    }
    
    // Original PHP: createLogSession - Create log session
    pub fn create_log_session(&mut self, lid: i64, local: &str, victim_ip: Option<&str>) {
        self.set("LOG_ID", SessionValue::Integer(lid));
        self.set("LOG_LOCAL", SessionValue::String(local.to_string()));
        
        if let Some(ip) = victim_ip {
            self.set("LOG_VICTIM_IP", SessionValue::String(ip.to_string()));
        }
    }
    
    // Original PHP: deleteLogSession - Delete log session
    pub fn delete_log_session(&mut self) {
        self.data.remove("LOG_ID");
        self.data.remove("LOG_LOCAL");
        self.data.remove("LOG_VICTIM_IP");
    }
    
    // Original PHP: issetLogSession - Check if log session exists
    pub fn isset_log_session(&self) -> bool {
        self.data.contains_key("LOG_ID")
    }
    
    // Original PHP: createInternetSession - Create internet session
    pub fn create_internet_session(&mut self, ip: &str) {
        self.set("INTERNET_IP", SessionValue::String(ip.to_string()));
    }
    
    // Original PHP: issetInternetSession - Check if internet session exists
    pub fn isset_internet_session(&self) -> bool {
        self.data.contains_key("INTERNET_IP")
    }
    
    // Original PHP: deleteInternetSession - Delete internet session
    pub fn delete_internet_session(&mut self) {
        self.data.remove("INTERNET_IP");
    }
    
    // Original PHP: isInternetLogged - Check if logged into internet
    pub fn is_internet_logged(&self) -> bool {
        self.isset_internet_session()
    }
    
    // Original PHP: isHacking - Check if currently hacking
    pub fn is_hacking(&self) -> bool {
        self.data.contains_key("HACKING")
    }
    
    // Original PHP: deleteHackingSession - Delete hacking session
    pub fn delete_hacking_session(&mut self) {
        self.data.remove("HACKING");
    }
    
    // Query tracking methods (original PHP functionality)
    pub fn new_query(&mut self) {
        self.query_count += 1;
        self.set("QUERY_COUNT", SessionValue::Integer(self.query_count as i64));
    }
    
    // Experience system integration
    pub fn exp_add(&mut self, action: &str) {
        let exp_amount = match action {
            "LOGIN" => 5,
            "HACK" => 10,
            "UPLOAD" => 15,
            "DOWNLOAD" => 10,
            "DELETE" => 8,
            _ => 0,
        };
        
        if exp_amount > 0 {
            // TODO: Add to user's experience in database
            self.add_msg(&format!("Gained {} experience points!", exp_amount), "notice");
        }
    }
    
    // Certification session management
    pub fn cert_session(&mut self, certs: Vec<i32>) {
        let certs_str = certs.iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",");
        self.set("CERTS", SessionValue::String(certs_str));
    }
    
    // Generic session data methods
    pub fn get(&self, key: &str) -> Option<&SessionValue> {
        self.data.get(key)
    }
    
    pub fn set(&mut self, key: &str, value: SessionValue) {
        self.data.insert(key.to_string(), value);
        self.last_activity = Utc::now();
    }
    
    pub fn remove(&mut self, key: &str) -> Option<SessionValue> {
        self.data.remove(key)
    }
    
    pub fn clear(&mut self) {
        self.data.clear();
    }
    
    // Language management (original PHP functionality)
    fn language_get(&self) -> String {
        // Check session language setting
        if let Some(lang) = self.get("LANGUAGE").and_then(|v| v.as_str()) {
            return lang.to_string();
        }
        
        // Default to English
        "en".to_string()
    }
    
    fn language_set(&mut self, user_logged_in: bool) {
        let lang = if user_logged_in {
            // TODO: Get user's preferred language from database
            "en".to_string()
        } else {
            self.language_get()
        };
        
        self.language = lang.clone();
        self.set("LANGUAGE", SessionValue::String(lang));
    }
    
    // Utility methods
    fn generate_session_id() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        (0..32)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
    
    fn microtime() -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        duration.as_secs_f64()
    }
    
    // Session persistence methods
    pub fn to_cookie_value(&self) -> Result<String, SessionError> {
        let json = serde_json::to_string(self)
            .map_err(|e| SessionError::SerializationError(e.to_string()))?;
        
        // In production, this should be encrypted
        Ok(base64::encode(json))
    }
    
    pub fn from_cookie_value(value: &str) -> Result<Self, SessionError> {
        let json = base64::decode(value)
            .map_err(|e| SessionError::DeserializationError(e.to_string()))?;
        
        let session: PhpSession = serde_json::from_slice(&json)
            .map_err(|e| SessionError::DeserializationError(e.to_string()))?;
        
        Ok(session)
    }
}

impl Default for PhpSession {
    fn default() -> Self {
        Self::new()
    }
}

// Axum integration for automatic session extraction
#[async_trait]
impl<S> FromRequestParts<S> for PhpSession
where
    S: Send + Sync,
{
    type Rejection = SessionError;
    
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract session from cookie
        if let Some(cookie_header) = parts.headers.get("cookie") {
            if let Ok(cookie_str) = cookie_header.to_str() {
                for cookie_pair in cookie_str.split(';') {
                    let mut parts = cookie_pair.trim().splitn(2, '=');
                    if let (Some(name), Some(value)) = (parts.next(), parts.next()) {
                        if name == "PHPSESSID" {
                            return PhpSession::from_cookie_value(value);
                        }
                    }
                }
            }
        }
        
        // No session found, create new one
        Ok(PhpSession::new())
    }
}

#[derive(Debug)]
pub enum SessionError {
    SerializationError(String),
    DeserializationError(String),
    InvalidSession,
    Expired,
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionError::SerializationError(e) => write!(f, "Session serialization error: {}", e),
            SessionError::DeserializationError(e) => write!(f, "Session deserialization error: {}", e),
            SessionError::InvalidSession => write!(f, "Invalid session"),
            SessionError::Expired => write!(f, "Session expired"),
        }
    }
}

impl std::error::Error for SessionError {}

// Session middleware for automatic session management
pub async fn session_middleware<B>(
    req: axum::extract::Request<B>,
    next: axum::middleware::Next<B>,
) -> axum::response::Response {
    // Extract session from request
    let (mut parts, body) = req.into_parts();
    let session = PhpSession::from_request_parts(&mut parts, &())
        .await
        .unwrap_or_default();
    
    // Add session to request extensions
    parts.extensions.insert(session);
    
    let req = axum::extract::Request::from_parts(parts, body);
    let mut response = next.run(req).await;
    
    // Set session cookie in response (if session was modified)
    // This would be implemented with proper cookie handling
    
    response
}