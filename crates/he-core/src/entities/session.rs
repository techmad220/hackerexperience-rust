use anyhow::{anyhow, Result};
//! Session entity - User session management and state tracking
//! 
//! This module provides the Session struct and methods for managing user sessions,
//! flash messages, language settings, experience points, and various game state sessions.

use sqlx::{Pool, Postgres, Row};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::error::HeResult;

/// Represents a user session in the Hacker Experience game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session ID
    pub session_id: String,
    /// User ID (if logged in)
    pub user_id: Option<i32>,
    /// Username (if logged in)
    pub username: Option<String>,
    /// Whether user is premium
    pub is_premium: bool,
    /// Language setting
    pub language: String,
    /// Query count for this session
    pub query_count: i32,
    /// Buffer query count
    pub buffer_query: i32,
    /// Execution time tracking
    pub exec_time: f64,
    /// Session creation time
    pub created_at: DateTime<Utc>,
    /// Last activity time
    pub last_activity: DateTime<Utc>,
    /// Client IP address
    pub ip_address: String,
    /// User agent string
    pub user_agent: Option<String>,
    /// Whether session is active
    pub is_active: bool,
    /// Special login flags (Facebook, etc.)
    pub special_login: Option<String>,
    /// Current flash message
    pub message: Option<SessionMessage>,
    /// Database connection pool
    #[serde(skip)]
    pub db_pool: Option<Pool<Postgres>>,
}

/// Flash message for user notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub message: String,
    pub message_type: MessageType,
    pub created_at: DateTime<Utc>,
}

/// Types of flash messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Notice,
    Error,
    Success,
    Warning,
    Mission,
}

/// Process session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSession {
    pub process_id: i32,
}

/// Log session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSession {
    pub log_id: i32,
    pub is_local: bool,
    pub victim_id: Option<i32>,
    pub is_npc: bool,
}

/// Internet session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternetSession {
    pub current_ip: String,
    pub logged_in_ip: Option<i64>,
    pub current_page: Option<String>,
    pub chmod: Option<String>,
    pub logged_user: Option<i32>,
}

/// Hacking session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HackingSession {
    pub hacked: bool,
    pub method: String,
}

/// Bank session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankSession {
    pub bank_id: i32,
    pub bank_account: String,
    pub bank_ip: String,
}

/// Wallet session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSession {
    pub wallet_address: String,
}

/// Mission session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionSession {
    pub mission_id: i32,
    pub mission_type: i32,
}

/// Certificate session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertSession {
    pub cert_level: i32,
}

/// Skill session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSession {
    pub skills: HashMap<String, i32>,
}

impl Session {
    /// Creates a new Session instance
    pub fn new(session_id: String, ip_address: String, db_pool: Pool<Postgres>) -> Self {
        let now = Utc::now();
        Self {
            session_id,
            user_id: None,
            username: None,
            is_premium: false,
            language: Self::language_set_default(&ip_address),
            query_count: 0,
            buffer_query: 0,
            exec_time: now.timestamp_millis() as f64 / 1000.0,
            created_at: now,
            last_activity: now,
            ip_address,
            user_agent: None,
            is_active: true,
            special_login: None,
            message: None,
            db_pool: Some(db_pool),
        }
    }

    /// Adds a flash message to the session
    /// 
    /// # Arguments
    /// * `msg` - Message text
    /// * `msg_type` - Message type (notice, error, etc.)
    pub fn add_msg(&mut self, msg: &str, msg_type: &str) {
        let message_type = match msg_type {
            "error" => MessageType::Error,
            "success" => MessageType::Success,
            "warning" => MessageType::Warning,
            "mission" => MessageType::Mission,
            _ => MessageType::Notice,
        };

        self.message = Some(SessionMessage {
            message: msg.to_string(),
            message_type,
            created_at: Utc::now(),
        });
    }

    /// Checks if there's a flash message
    /// 
    /// # Returns
    /// True if message exists, false otherwise
    pub fn isset_msg(&self) -> bool {
        self.message.is_some()
    }

    /// Gets and clears the flash message
    /// 
    /// # Returns
    /// The flash message if it exists
    pub fn return_msg(&mut self, prefix: Option<&str>) -> Option<String> {
        if let Some(msg) = &self.message {
            let (msg_type, prefix_msg) = match msg.message_type {
                MessageType::Error => ("error", "Error!"),
                MessageType::Success => ("success", "Success!"),
                MessageType::Warning => ("warning", "Warning!"),
                MessageType::Mission => ("info", "Mission"),
                MessageType::Notice => ("info", "Notice"),
            };

            let prefix_text = if prefix.is_some() {
                ""
            } else {
                &format!("<strong>{}</strong> ", prefix_msg)
            };

            let html = format!(
                r#"<div class="alert alert-{}">
                    <button class="close" data-dismiss="alert">×</button>
                    {}{}
                </div>"#,
                msg_type, prefix_text, msg.message
            );

            self.message = None;
            Some(html)
        } else {
            None
        }
    }

    /// Deletes the flash message
    pub fn del_msg(&mut self) {
        self.message = None;
    }

    /// Creates a login session
    /// 
    /// # Arguments
    /// * `id` - User ID
    /// * `user` - Username
    /// * `premium` - Whether user is premium
    /// * `special` - Special login type (facebook, etc.)
    pub fn login_session(&mut self, id: i32, user: &str, premium: bool, special: Option<&str>) {
        self.user_id = Some(id);
        self.username = Some(user.to_string());
        self.is_premium = premium;
        self.language_set(true);

        if let Some(special_type) = special {
            self.special_login = Some(special_type.to_string());
        }

        self.update_activity();
    }

    /// Checks if user is logged in
    /// 
    /// # Returns
    /// True if user is logged in, false otherwise
    pub fn isset_login(&self) -> bool {
        self.user_id.is_some()
    }

    /// Checks if user logged in with Facebook
    /// 
    /// # Returns
    /// True if Facebook login, false otherwise
    pub fn isset_fb_login(&self) -> bool {
        self.special_login.as_ref().map_or(false, |s| s == "facebook")
    }

    /// Logs out the user
    /// 
    /// # Arguments
    /// * `clear_db` - Whether to clear database entries
    /// * `redirect` - Whether to redirect after logout
    pub async fn logout(&mut self, clear_db: bool, redirect: bool) -> HeResult<()> {
        if clear_db && self.user_id.is_some() {
            let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
            let user_id = self.user_id.map_err(|e| anyhow::anyhow!("Error: {}", e))?;

            // Remove from online users
            sqlx::query("DELETE FROM users_online WHERE id = $1")
                .bind(user_id)
                .execute(db)
                .await?;

            // Remove from expiring users
            sqlx::query("DELETE FROM users_expire WHERE userID = $1")
                .bind(user_id)
                .execute(db)
                .await?;
        }

        // Clear session data
        self.user_id = None;
        self.username = None;
        self.is_premium = false;
        self.is_active = false;
        self.special_login = None;

        Ok(())
    }

    /// Validates if the login is still valid
    /// 
    /// # Returns
    /// True if login is valid, false otherwise
    pub async fn valid_login(&self) -> HeResult<bool> {
        if self.user_id.is_none() {
            return Ok(false);
        }

        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let user_id = self.user_id.map_err(|e| anyhow::anyhow!("Error: {}", e))?;

        let row = sqlx::query("SELECT COUNT(*) AS t FROM users_online WHERE id = $1 LIMIT 1")
            .bind(user_id)
            .fetch_one(db)
            .await?;

        let total: i64 = row.get("t");
        Ok(total == 1)
    }

    /// Increments the query count
    pub fn new_query(&mut self) {
        self.query_count += 1;
    }

    /// Checks if query count is set
    /// 
    /// # Returns
    /// True if query count exists
    pub fn isset_query_count(&self) -> bool {
        true // Always true in our implementation
    }

    /// Returns the current query count
    /// 
    /// # Returns
    /// Query count as string
    pub fn return_query_count(&self) -> String {
        self.query_count.to_string()
    }

    /// Adds experience points to a user
    /// 
    /// # Arguments
    /// * `action` - Action type that grants experience
    /// * `info` - Additional information for experience calculation
    /// * `uid` - User ID (optional, uses session user if None)
    pub async fn exp_add(&self, action: &str, info: Vec<String>, uid: Option<i32>) -> HeResult<()> {
        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let user_id = uid.or(self.user_id).unwrap_or(0);

        let total_to_add = self.exp_get_amount(action, &info);

        sqlx::query("UPDATE users_stats SET exp = exp + $1 WHERE uid = $2")
            .bind(total_to_add)
            .bind(user_id)
            .execute(db)
            .await?;

        Ok(())
    }

    /// Calculates experience amount for an action
    /// 
    /// # Arguments
    /// * `action` - Action type
    /// * `info` - Additional information
    /// 
    /// # Returns
    /// Experience points to award
    fn exp_get_amount(&self, action: &str, info: &[String]) -> i32 {
        match action {
            "LOGIN" => 1, // Note: This was marked as a bad idea in original code
            "EDIT_LOG" => {
                if info.get(0).map_or(false, |s| s == "1") {
                    1 // local
                } else {
                    3 // remote
                }
            },
            "REMOTE_LOGIN" => {
                if let Some(login_type) = info.get(0) {
                    match login_type.as_str() {
                        "download" => 1,
                        "bank" => 25,
                        "clan" => 3,
                        _ => {
                            // Calculate based on version difference
                            let att_version = info.get(2).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
                            let def_version = info.get(3).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
                            let diff = att_version - def_version;
                            let relative = if diff == 0 { 15 } else { 15 - diff };
                            relative.max(5)
                        }
                    }
                } else {
                    5
                }
            },
            "HACK" => {
                let base = 50;
                let bonus = info.get(1).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
                base + bonus.round() as i32
            },
            "CERT" => 100,
            "MISSION" => {
                let base = 50;
                let multiplier = info.get(0).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
                base + (multiplier * 0.05).round() as i32
            },
            "TRANSFER" => {
                let base = 200;
                let amount = info.get(0).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
                base + (amount / 15.0).round() as i32
            },
            "RESET" => {
                if info.get(0).map_or(false, |s| s == "ip") {
                    50
                } else {
                    15
                }
            },
            "RESEARCH" => {
                let base = 35;
                let bonus = info.get(1).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0).round() as i32;
                let version = info.get(0).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
                let final_bonus = if version <= 4 { bonus * 2 } else { bonus };
                base + final_bonus
            },
            "BUY" => {
                if let Some(buy_type) = info.get(0) {
                    match buy_type.as_str() {
                        "license" => {
                            let base = 25;
                            let bonus = info.get(1).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0).round() as i32;
                            base + bonus
                        },
                        "pc" => {
                            let pc_type = info.get(1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(1);
                            match pc_type {
                                2 => 100,
                                3 => 500,
                                _ => 500 * pc_type,
                            }
                        },
                        "xhd" => {
                            let count = info.get(1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(1);
                            100 * count
                        },
                        _ => {
                            let base = 5;
                            let price = info.get(1).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
                            base + (price / 10.0).round() as i32
                        }
                    }
                } else {
                    5
                }
            },
            "ACTIONS" => {
                let action_type = info.get(0).unwrap_or(&String::new());
                let price = info.get(1).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
                let mut bonus = (price / 10.0).round() as i32;
                
                if action_type == "net" {
                    bonus *= 2;
                }
                
                bonus.clamp(1, 15)
            },
            "AV" => {
                let location = info.get(0).unwrap_or(&String::new());
                let count = info.get(1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(1);
                if location == "local" {
                    10 * count
                } else {
                    15 * count
                }
            },
            "DOOM" => 10000,
            "DDOS" => {
                let power = info.get(0).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
                let total_damage = info.get(1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
                let seized_before = info.get(2).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
                
                if total_damage == 0 && seized_before == 1 {
                    100
                } else {
                    250 + (power * 0.01).round() as i32
                }
            },
            "COLLECT" => {
                let collect_type = info.get(0).unwrap_or(&String::new());
                let amount = info.get(1).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
                if collect_type == "btc" {
                    (amount * 600.0) as i32
                } else {
                    (amount * 0.1).round() as i32
                }
            },
            "PUZZLE" => 25,
            "NMAP" => {
                let count = info.get(0).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
                25 + 25 * count
            },
            "ANALYZE" => 15,
            _ => 0,
        }
    }

    /// Sets the language for the session
    /// 
    /// # Arguments
    /// * `set_session` - Whether to store in session
    pub fn language_set(&mut self, set_session: bool) {
        if set_session {
            self.language = Self::language_set_default(&self.ip_address);
        }
    }

    /// Determines default language based on host or IP
    /// 
    /// # Arguments
    /// * `ip_address` - Client IP address
    /// 
    /// # Returns
    /// Language code
    fn language_set_default(ip_address: &str) -> String {
        // TODO: Implement proper language detection based on domain/IP
        // For now, default to English
        "en_US".to_string()
    }

    /// Gets the current language setting
    /// 
    /// # Returns
    /// Language code
    pub fn language_get(&self) -> &str {
        &self.language
    }

    /// Generates help URLs for different pages
    /// 
    /// # Arguments
    /// * `page` - Page name
    /// * `info` - Additional info (optional)
    /// 
    /// # Returns
    /// Help URL
    pub fn help(&self, page: &str, info: Option<&str>) -> String {
        let lang = if self.language.starts_with("pt") { "pt" } else { "en" };
        let base_url = format!("https://wiki.hackerexperience.com/{}:", lang);

        match page {
            "clan" => format!("{}clans", base_url),
            "missions" => {
                let ext = if info == Some("level") { "#mission_level" } else { "" };
                format!("{}missions{}", base_url, ext)
            },
            "hardware" => format!("{}hardware", base_url),
            "log" => format!("{}log", base_url),
            "university" => format!("{}university", base_url),
            "finances" => format!("{}finances", base_url),
            "list" => {
                match info {
                    Some("ddos") => format!("{}ddos", base_url),
                    Some("collect") => format!("{}hacked_database", base_url),
                    _ => format!("{}hacked_database", base_url),
                }
            },
            "task" => format!("{}processes", base_url),
            "software" => {
                let ext = if info == Some("external") { "#external_hard_drive" } else { "" };
                format!("{}softwares{}", base_url, ext)
            },
            "internet" => {
                let ext = if info == Some("hack") { "hacking" } else { "internet" };
                format!("{}{}", base_url, ext)
            },
            _ => format!("{}help", base_url),
        }
    }

    /// Updates the last activity timestamp
    pub fn update_activity(&mut self) {
        self.last_activity = Utc::now();
    }

    /// Checks if session is expired
    /// 
    /// # Arguments
    /// * `timeout_seconds` - Session timeout in seconds
    /// 
    /// # Returns
    /// True if session is expired
    pub fn is_expired(&self, timeout_seconds: i64) -> bool {
        let now = Utc::now();
        (now.timestamp() - self.last_activity.timestamp()) > timeout_seconds
    }

    // TODO: Implement session management methods for:
    // - Process sessions (isset_process_session, process_id, etc.)
    // - Log sessions (create_log_session, delete_log_session, etc.)
    // - Internet sessions (create_internet_session, is_internet_logged, etc.)
    // - Bank sessions (create_bank_session, isset_bank_session, etc.)
    // - Mission sessions (isset_mission_session, mission_id, etc.)
    // - Wallet sessions
    // - Certificate and skill sessions
    // 
    // These would likely be handled by separate session state managers
    // in a more modern Rust architecture.
}