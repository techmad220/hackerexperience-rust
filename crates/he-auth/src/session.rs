//! Session management

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Session timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum number of active sessions per user
    pub max_sessions_per_user: usize,
    /// Clean up expired sessions interval
    pub cleanup_interval_seconds: u64,
    /// Store sessions in memory or database
    pub storage_type: SessionStorageType,
    /// Session cookie settings
    pub cookie_config: SessionCookieConfig,
}

/// Session storage type
#[derive(Debug, Clone)]
pub enum SessionStorageType {
    Memory,
    Database,
    Redis,
}

/// Session cookie configuration
#[derive(Debug, Clone)]
pub struct SessionCookieConfig {
    pub name: String,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: SameSite,
    pub domain: Option<String>,
    pub path: String,
}

#[derive(Debug, Clone)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 3600, // 1 hour
            max_sessions_per_user: 5,
            cleanup_interval_seconds: 300, // 5 minutes
            storage_type: SessionStorageType::Memory,
            cookie_config: SessionCookieConfig {
                name: "he_session".to_string(),
                secure: true,
                http_only: true,
                same_site: SameSite::Strict,
                domain: None,
                path: "/".to_string(),
            },
        }
    }
}

/// Session data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// User ID associated with this session
    pub user_id: Uuid,
    /// User email
    pub email: String,
    /// User roles
    pub roles: Vec<String>,
    /// When the session was created
    pub login_time: chrono::DateTime<chrono::Utc>,
    /// Last activity timestamp
    pub last_activity: chrono::DateTime<chrono::Utc>,
    /// Client IP address
    pub ip_address: Option<String>,
    /// Additional session metadata
    pub metadata: HashMap<String, String>,
}

impl SessionData {
    /// Check if session has expired
    pub fn is_expired(&self, timeout_seconds: u64) -> bool {
        let timeout_duration = chrono::Duration::seconds(timeout_seconds as i64);
        chrono::Utc::now() - self.last_activity > timeout_duration
    }

    /// Update last activity timestamp
    pub fn update_activity(&mut self) {
        self.last_activity = chrono::Utc::now();
    }

    /// Get session duration
    pub fn duration(&self) -> chrono::Duration {
        chrono::Utc::now() - self.login_time
    }
}

/// Session manager
#[derive(Debug)]
pub struct SessionManager {
    config: SessionConfig,
    sessions: Arc<RwLock<HashMap<String, SessionData>>>,
    user_sessions: Arc<RwLock<HashMap<Uuid, Vec<String>>>>,
}

impl SessionManager {
    /// Create a new session manager
    pub async fn new(config: SessionConfig) -> Result<Self> {
        let manager = Self {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            user_sessions: Arc::new(RwLock::new(HashMap::new())),
        };

        // Start cleanup task
        manager.start_cleanup_task().await;

        Ok(manager)
    }

    /// Create a new session
    pub async fn create_session(&self, mut session_data: SessionData) -> Result<String> {
        let session_id = Uuid::new_v4().to_string();
        session_data.last_activity = chrono::Utc::now();

        // Check if user has too many active sessions
        self.enforce_session_limit(&session_data.user_id).await?;

        // Store session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.clone(), session_data.clone());
        }

        // Update user session mapping
        {
            let mut user_sessions = self.user_sessions.write().await;
            user_sessions
                .entry(session_data.user_id)
                .or_insert_with(Vec::new)
                .push(session_id.clone());
        }

        debug!("Created session {} for user {}", session_id, session_data.user_id);
        Ok(session_id)
    }

    /// Get session data
    pub async fn get_session(&self, session_id: &str) -> Option<SessionData> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// Update session activity
    pub async fn update_session_activity(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.update_activity();
            debug!("Updated activity for session {}", session_id);
        }
        Ok(())
    }

    /// Check if session is valid (exists and not expired)
    pub async fn is_session_valid(&self, session_id: &str) -> Result<bool> {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(session_id) {
            Ok(!session.is_expired(self.config.timeout_seconds))
        } else {
            Ok(false)
        }
    }

    /// Invalidate a specific session
    pub async fn invalidate_session(&self, session_id: &str) -> Result<()> {
        let session_data = {
            let mut sessions = self.sessions.write().await;
            sessions.remove(session_id)
        };

        if let Some(session) = session_data {
            // Remove from user session mapping
            let mut user_sessions = self.user_sessions.write().await;
            if let Some(user_session_list) = user_sessions.get_mut(&session.user_id) {
                user_session_list.retain(|id| id != session_id);
                if user_session_list.is_empty() {
                    user_sessions.remove(&session.user_id);
                }
            }

            debug!("Invalidated session {} for user {}", session_id, session.user_id);
        }

        Ok(())
    }

    /// Invalidate all sessions for a user
    pub async fn invalidate_user_sessions(&self, user_id: &Uuid) -> Result<usize> {
        let session_ids = {
            let mut user_sessions = self.user_sessions.write().await;
            user_sessions.remove(user_id).unwrap_or_default()
        };

        let mut invalidated_count = 0;
        {
            let mut sessions = self.sessions.write().await;
            for session_id in &session_ids {
                if sessions.remove(session_id).is_some() {
                    invalidated_count += 1;
                }
            }
        }

        info!("Invalidated {} sessions for user {}", invalidated_count, user_id);
        Ok(invalidated_count)
    }

    /// Get active sessions for a user
    pub async fn get_user_sessions(&self, user_id: &Uuid) -> Vec<SessionData> {
        let session_ids = {
            let user_sessions = self.user_sessions.read().await;
            user_sessions.get(user_id).cloned().unwrap_or_default()
        };

        let sessions = self.sessions.read().await;
        session_ids
            .iter()
            .filter_map(|id| sessions.get(id).cloned())
            .collect()
    }

    /// Get total number of active sessions
    pub async fn get_active_session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }

    /// Get session statistics
    pub async fn get_session_stats(&self) -> SessionStats {
        let sessions = self.sessions.read().await;
        let user_sessions = self.user_sessions.read().await;

        let now = chrono::Utc::now();
        let mut expired_count = 0;
        let mut total_duration = chrono::Duration::zero();

        for session in sessions.values() {
            if session.is_expired(self.config.timeout_seconds) {
                expired_count += 1;
            }
            total_duration = total_duration + session.duration();
        }

        let avg_duration = if !sessions.is_empty() {
            total_duration / sessions.len() as i32
        } else {
            chrono::Duration::zero()
        };

        SessionStats {
            total_sessions: sessions.len(),
            active_sessions: sessions.len() - expired_count,
            expired_sessions: expired_count,
            unique_users: user_sessions.len(),
            average_session_duration: avg_duration,
            timestamp: now,
        }
    }

    /// Enforce session limit per user
    async fn enforce_session_limit(&self, user_id: &Uuid) -> Result<()> {
        let session_ids = {
            let user_sessions = self.user_sessions.read().await;
            user_sessions.get(user_id).cloned().unwrap_or_default()
        };

        if session_ids.len() >= self.config.max_sessions_per_user {
            // Remove oldest session
            if let Some(oldest_session_id) = session_ids.first() {
                self.invalidate_session(oldest_session_id).await?;
                debug!("Removed oldest session for user {} due to limit", user_id);
            }
        }

        Ok(())
    }

    /// Start background cleanup task
    async fn start_cleanup_task(&self) {
        let sessions = self.sessions.clone();
        let user_sessions = self.user_sessions.clone();
        let timeout_seconds = self.config.timeout_seconds;
        let cleanup_interval = std::time::Duration::from_secs(self.config.cleanup_interval_seconds);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);

            loop {
                interval.tick().await;

                let mut expired_sessions = Vec::new();
                let mut user_session_updates = HashMap::new();

                // Find expired sessions
                {
                    let sessions_guard = sessions.read().await;
                    for (session_id, session_data) in sessions_guard.iter() {
                        if session_data.is_expired(timeout_seconds) {
                            expired_sessions.push(session_id.clone());
                            user_session_updates
                                .entry(session_data.user_id)
                                .or_insert_with(Vec::new)
                                .push(session_id.clone());
                        }
                    }
                }

                if !expired_sessions.is_empty() {
                    // Remove expired sessions
                    {
                        let mut sessions_guard = sessions.write().await;
                        for session_id in &expired_sessions {
                            sessions_guard.remove(session_id);
                        }
                    }

                    // Update user session mappings
                    {
                        let mut user_sessions_guard = user_sessions.write().await;
                        for (user_id, expired_session_ids) in user_session_updates {
                            if let Some(user_session_list) = user_sessions_guard.get_mut(&user_id) {
                                user_session_list.retain(|id| !expired_session_ids.contains(id));
                                if user_session_list.is_empty() {
                                    user_sessions_guard.remove(&user_id);
                                }
                            }
                        }
                    }

                    debug!("Cleaned up {} expired sessions", expired_sessions.len());
                }
            }
        });
    }
}

/// Session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub expired_sessions: usize,
    pub unique_users: usize,
    pub average_session_duration: chrono::Duration,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Session cookie builder
pub struct SessionCookie {
    config: SessionCookieConfig,
}

impl SessionCookie {
    /// Create a new session cookie builder
    pub fn new(config: SessionCookieConfig) -> Self {
        Self { config }
    }

    /// Build cookie header value
    pub fn build_cookie_header(&self, session_id: &str, max_age: Option<u64>) -> String {
        let mut cookie = format!("{}={}", self.config.name, session_id);

        if let Some(max_age) = max_age {
            cookie.push_str(&format!("; Max-Age={}", max_age));
        }

        cookie.push_str(&format!("; Path={}", self.config.path));

        if let Some(domain) = &self.config.domain {
            cookie.push_str(&format!("; Domain={}", domain));
        }

        if self.config.secure {
            cookie.push_str("; Secure");
        }

        if self.config.http_only {
            cookie.push_str("; HttpOnly");
        }

        match self.config.same_site {
            SameSite::Strict => cookie.push_str("; SameSite=Strict"),
            SameSite::Lax => cookie.push_str("; SameSite=Lax"),
            SameSite::None => cookie.push_str("; SameSite=None"),
        }

        cookie
    }

    /// Build cookie deletion header
    pub fn build_delete_cookie_header(&self) -> String {
        format!(
            "{}=; Path={}; Max-Age=0{}{}{}",
            self.config.name,
            self.config.path,
            if self.config.secure { "; Secure" } else { "" },
            if self.config.http_only { "; HttpOnly" } else { "" },
            match self.config.same_site {
                SameSite::Strict => "; SameSite=Strict",
                SameSite::Lax => "; SameSite=Lax",
                SameSite::None => "; SameSite=None",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    async fn create_test_session_manager() -> SessionManager {
        let config = SessionConfig {
            timeout_seconds: 1,
            cleanup_interval_seconds: 1,
            ..Default::default()
        };
        SessionManager::new(config).await.unwrap()
    }

    fn create_test_session_data() -> SessionData {
        SessionData {
            user_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            roles: vec!["player".to_string()],
            login_time: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            ip_address: Some("127.0.0.1".to_string()),
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_session_creation() {
        let manager = create_test_session_manager().await;
        let session_data = create_test_session_data();

        let session_id = manager.create_session(session_data.clone()).await.unwrap();
        assert!(!session_id.is_empty());

        let retrieved = manager.get_session(&session_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().user_id, session_data.user_id);
    }

    #[tokio::test]
    async fn test_session_expiration() {
        let manager = create_test_session_manager().await;
        let session_data = create_test_session_data();

        let session_id = manager.create_session(session_data).await.unwrap();
        
        // Session should be valid initially
        assert!(manager.is_session_valid(&session_id).await.unwrap());

        // Wait for expiration
        sleep(Duration::from_secs(2)).await;

        // Session should be expired
        assert!(!manager.is_session_valid(&session_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_session_invalidation() {
        let manager = create_test_session_manager().await;
        let session_data = create_test_session_data();

        let session_id = manager.create_session(session_data).await.unwrap();
        assert!(manager.get_session(&session_id).await.is_some());

        manager.invalidate_session(&session_id).await.unwrap();
        assert!(manager.get_session(&session_id).await.is_none());
    }

    #[tokio::test]
    async fn test_user_session_limit() {
        let config = SessionConfig {
            max_sessions_per_user: 2,
            ..Default::default()
        };
        let manager = SessionManager::new(config).await.unwrap();

        let user_id = Uuid::new_v4();
        let mut session_data = create_test_session_data();
        session_data.user_id = user_id;

        // Create first session
        let session1 = manager.create_session(session_data.clone()).await.unwrap();
        
        // Create second session
        let session2 = manager.create_session(session_data.clone()).await.unwrap();
        
        // Both sessions should exist
        assert!(manager.get_session(&session1).await.is_some());
        assert!(manager.get_session(&session2).await.is_some());

        // Create third session (should remove first)
        let _session3 = manager.create_session(session_data).await.unwrap();
        
        // First session should be gone
        assert!(manager.get_session(&session1).await.is_none());
    }

    #[test]
    fn test_session_cookie_builder() {
        let config = SessionCookieConfig {
            name: "test_session".to_string(),
            secure: true,
            http_only: true,
            same_site: SameSite::Strict,
            domain: Some("example.com".to_string()),
            path: "/".to_string(),
        };

        let cookie_builder = SessionCookie::new(config);
        let cookie_header = cookie_builder.build_cookie_header("session123", Some(3600));

        assert!(cookie_header.contains("test_session=session123"));
        assert!(cookie_header.contains("Max-Age=3600"));
        assert!(cookie_header.contains("Domain=example.com"));
        assert!(cookie_header.contains("Secure"));
        assert!(cookie_header.contains("HttpOnly"));
        assert!(cookie_header.contains("SameSite=Strict"));
    }
}