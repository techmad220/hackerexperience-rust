use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::net::Ipv4Addr;

use crate::error::Result;

/// Internet navigation session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternetSession {
    pub current_ip: i64,
    pub is_logged_in: bool,
    pub connection_time: chrono::NaiveDateTime,
    pub can_upload: bool,
    pub is_cracking: bool,
    pub connected_server: Option<ServerInfo>,
}

/// Server information for internet navigation
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ServerInfo {
    pub id: i32,
    pub ip: i64,
    pub user_id: i32,
    pub server_type: String, // "NPC" or "USER"
    pub login: Option<String>,
    pub password: Option<String>,
    pub is_down: bool,
    pub is_protected: bool,
}

/// Navigation result for internet browsing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationResult {
    pub success: bool,
    pub error_message: Option<String>,
    pub server_info: Option<ServerInfo>,
    pub available_actions: Vec<String>,
    pub webpage_content: Option<String>,
}

/// File system information
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FileSystemEntry {
    pub id: i32,
    pub name: String,
    pub entry_type: String, // "file" or "folder"
    pub size: i32,
    pub owner_id: i32,
    pub permissions: String,
    pub last_modified: chrono::NaiveDateTime,
    pub is_hidden: bool,
}

/// Login attempt result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResult {
    pub success: bool,
    pub message: String,
    pub session_created: bool,
    pub detected: bool,
}

#[async_trait]
pub trait InternetRepository {
    /// Navigate to IP address
    async fn navigate_to(&self, ip: i64, user_id: i32) -> Result<NavigationResult>;
    
    /// Attempt to login to server
    async fn attempt_login(&self, ip: i64, username: &str, password: &str, user_id: i32) -> Result<LoginResult>;
    
    /// Check if server exists
    async fn server_exists(&self, ip: i64) -> Result<bool>;
    
    /// Get server information
    async fn get_server_info(&self, ip: i64) -> Result<Option<ServerInfo>>;
    
    /// Check if server is down
    async fn is_server_down(&self, ip: i64) -> Result<bool>;
    
    /// Create internet session
    async fn create_session(&self, user_id: i32, ip: i64) -> Result<()>;
    
    /// Delete internet session
    async fn delete_session(&self, user_id: i32) -> Result<()>;
    
    /// Get current session
    async fn get_session(&self, user_id: i32) -> Result<Option<InternetSession>>;
    
    /// Check if user is logged into any server
    async fn is_logged_in(&self, user_id: i32) -> Result<bool>;
    
    /// Get current connected IP
    async fn get_connected_ip(&self, user_id: i32) -> Result<Option<i64>>;
    
    /// List files in current directory
    async fn list_files(&self, ip: i64, directory_id: Option<i32>) -> Result<Vec<FileSystemEntry>>;
    
    /// Download file from server
    async fn download_file(&self, ip: i64, file_id: i32, user_id: i32) -> Result<bool>;
    
    /// Upload file to server
    async fn upload_file(&self, ip: i64, file_name: &str, file_data: &[u8], user_id: i32) -> Result<bool>;
    
    /// Delete file from server
    async fn delete_file(&self, ip: i64, file_id: i32, user_id: i32) -> Result<bool>;
    
    /// Get server webpage content
    async fn get_webpage(&self, ip: i64) -> Result<Option<String>>;
    
    /// Check login credentials
    async fn verify_credentials(&self, ip: i64, username: &str, password: &str) -> Result<bool>;
    
    /// Convert between IP formats
    fn long_to_ip(ip_long: i64) -> Ipv4Addr;
    fn ip_to_long(ip: Ipv4Addr) -> i64;
}

pub struct InternetService {
    db: PgPool,
}

impl InternetService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl InternetRepository for InternetService {
    /// Navigate to IP address
    async fn navigate_to(&self, ip: i64, user_id: i32) -> Result<NavigationResult> {
        if !self.server_exists(ip).await? {
            return Ok(NavigationResult {
                success: false,
                error_message: Some("404 - Page not found. This IP does not exist.".to_string()),
                server_info: None,
                available_actions: vec![],
                webpage_content: None,
            });
        }
        
        if self.is_server_down(ip).await? {
            return Ok(NavigationResult {
                success: false,
                error_message: Some("Server is currently down.".to_string()),
                server_info: None,
                available_actions: vec![],
                webpage_content: None,
            });
        }
        
        let server_info = self.get_server_info(ip).await?;
        let webpage = self.get_webpage(ip).await?;
        
        // Create session
        self.create_session(user_id, ip).await?;
        
        let mut actions = vec!["crack".to_string(), "whois".to_string()];
        if self.is_logged_in(user_id).await? {
            actions.extend(vec!["files".to_string(), "upload".to_string(), "download".to_string()]);
        }
        
        Ok(NavigationResult {
            success: true,
            error_message: None,
            server_info,
            available_actions: actions,
            webpage_content: webpage,
        })
    }
    
    /// Attempt to login to server
    async fn attempt_login(&self, ip: i64, username: &str, password: &str, user_id: i32) -> Result<LoginResult> {
        if self.verify_credentials(ip, username, password).await? {
            // Create logged in session
            sqlx::query!(
                "UPDATE internet_sessions SET is_logged_in = true WHERE user_id = $1 AND current_ip = $2",
                user_id,
                ip
            )
            .execute(&self.db)
            .await?;
            
            Ok(LoginResult {
                success: true,
                message: "Login successful".to_string(),
                session_created: true,
                detected: false,
            })
        } else {
            Ok(LoginResult {
                success: false,
                message: "Invalid credentials".to_string(),
                session_created: false,
                detected: false, // TODO: Implement detection logic
            })
        }
    }
    
    /// Check if server exists
    async fn server_exists(&self, ip: i64) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM 
             (SELECT id FROM users WHERE game_ip = $1 
              UNION 
              SELECT id FROM npc WHERE npc_ip = $1) as servers",
            ip
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) > 0)
    }
    
    /// Get server information
    async fn get_server_info(&self, ip: i64) -> Result<Option<ServerInfo>> {
        // Check if it's a user server first
        let user_server = sqlx::query!(
            "SELECT id, login FROM users WHERE game_ip = $1",
            ip
        )
        .fetch_optional(&self.db)
        .await?;
        
        if let Some(user) = user_server {
            return Ok(Some(ServerInfo {
                id: user.id,
                ip,
                user_id: user.id,
                server_type: "USER".to_string(),
                login: Some(user.login),
                password: None, // Never expose passwords
                is_down: false,
                is_protected: false,
            }));
        }
        
        // Check if it's an NPC server
        let npc_server = sqlx::query!(
            "SELECT npc.id, npc_info.name FROM npc 
             INNER JOIN npc_info_en npc_info ON npc.id = npc_info.npc_id
             WHERE npc.npc_ip = $1",
            ip
        )
        .fetch_optional(&self.db)
        .await?;
        
        if let Some(npc) = npc_server {
            return Ok(Some(ServerInfo {
                id: npc.id,
                ip,
                user_id: npc.id,
                server_type: "NPC".to_string(),
                login: Some(npc.name),
                password: None,
                is_down: false,
                is_protected: false,
            }));
        }
        
        Ok(None)
    }
    
    /// Check if server is down
    async fn is_server_down(&self, ip: i64) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM npc_down 
             INNER JOIN npc ON npc_down.npc_id = npc.id
             WHERE npc.npc_ip = $1",
            ip
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) > 0)
    }
    
    /// Create internet session
    async fn create_session(&self, user_id: i32, ip: i64) -> Result<()> {
        // Delete existing session
        self.delete_session(user_id).await?;
        
        sqlx::query!(
            "INSERT INTO internet_sessions (user_id, current_ip, is_logged_in, connection_time, can_upload, is_cracking)
             VALUES ($1, $2, false, NOW(), false, false)",
            user_id,
            ip
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Delete internet session
    async fn delete_session(&self, user_id: i32) -> Result<()> {
        sqlx::query!(
            "DELETE FROM internet_sessions WHERE user_id = $1",
            user_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Get current session
    async fn get_session(&self, user_id: i32) -> Result<Option<InternetSession>> {
        let session = sqlx::query!(
            "SELECT current_ip, is_logged_in, connection_time, can_upload, is_cracking 
             FROM internet_sessions WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        match session {
            Some(s) => {
                let server_info = if s.current_ip != 0 {
                    self.get_server_info(s.current_ip).await?
                } else {
                    None
                };
                
                Ok(Some(InternetSession {
                    current_ip: s.current_ip,
                    is_logged_in: s.is_logged_in,
                    connection_time: s.connection_time,
                    can_upload: s.can_upload,
                    is_cracking: s.is_cracking,
                    connected_server: server_info,
                }))
            }
            None => Ok(None),
        }
    }
    
    /// Check if user is logged into any server
    async fn is_logged_in(&self, user_id: i32) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM internet_sessions WHERE user_id = $1 AND is_logged_in = true",
            user_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) > 0)
    }
    
    /// Get current connected IP
    async fn get_connected_ip(&self, user_id: i32) -> Result<Option<i64>> {
        let ip = sqlx::query_scalar!(
            "SELECT current_ip FROM internet_sessions WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(ip)
    }
    
    /// List files in current directory
    async fn list_files(&self, ip: i64, directory_id: Option<i32>) -> Result<Vec<FileSystemEntry>> {
        let server_info = self.get_server_info(ip).await?;
        
        if let Some(server) = server_info {
            let files = sqlx::query_as!(
                FileSystemEntry,
                "SELECT id, soft_name as name, 'file' as entry_type, soft_size as size, 
                        user_id as owner_id, 'rwx' as permissions, soft_last_edit as last_modified, soft_hidden as is_hidden
                 FROM software 
                 WHERE user_id = $1 AND (is_npc = $2) AND is_folder = false
                 ORDER BY soft_name",
                server.user_id,
                server.server_type == "NPC"
            )
            .fetch_all(&self.db)
            .await?;
            
            Ok(files)
        } else {
            Ok(vec![])
        }
    }
    
    /// Download file from server
    async fn download_file(&self, ip: i64, file_id: i32, user_id: i32) -> Result<bool> {
        // Check if user is logged in and file exists
        if !self.is_logged_in(user_id).await? {
            return Ok(false);
        }
        
        let file_exists = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM software WHERE id = $1",
            file_id
        )
        .fetch_one(&self.db)
        .await?;
        
        if file_exists.unwrap_or(0) == 0 {
            return Ok(false);
        }
        
        // Copy file to user's system
        sqlx::query!(
            "INSERT INTO software (user_id, soft_name, soft_version, soft_size, soft_ram, soft_type, soft_last_edit, soft_hidden, is_npc, original_from)
             SELECT $1, soft_name, soft_version, soft_size, soft_ram, soft_type, NOW(), false, false, $2
             FROM software WHERE id = $3",
            user_id,
            file_id,
            file_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(true)
    }
    
    /// Upload file to server
    async fn upload_file(&self, ip: i64, file_name: &str, file_data: &[u8], user_id: i32) -> Result<bool> {
        if !self.is_logged_in(user_id).await? {
            return Ok(false);
        }
        
        let server_info = self.get_server_info(ip).await?;
        if let Some(server) = server_info {
            // Create file on target server
            sqlx::query!(
                "INSERT INTO software (user_id, soft_name, soft_version, soft_size, soft_ram, soft_type, soft_last_edit, soft_hidden, is_npc, original_from)
                 VALUES ($1, $2, 10, $3, 0, 99, NOW(), false, $4, $5)",
                server.user_id,
                file_name,
                file_data.len() as i32,
                server.server_type == "NPC",
                user_id
            )
            .execute(&self.db)
            .await?;
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Delete file from server
    async fn delete_file(&self, ip: i64, file_id: i32, user_id: i32) -> Result<bool> {
        if !self.is_logged_in(user_id).await? {
            return Ok(false);
        }
        
        sqlx::query!(
            "DELETE FROM software WHERE id = $1",
            file_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(true)
    }
    
    /// Get server webpage content
    async fn get_webpage(&self, ip: i64) -> Result<Option<String>> {
        let webpage = sqlx::query_scalar!(
            "SELECT npc_info.web FROM npc 
             INNER JOIN npc_info_en npc_info ON npc.id = npc_info.npc_id
             WHERE npc.npc_ip = $1",
            ip
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(webpage.flatten())
    }
    
    /// Check login credentials
    async fn verify_credentials(&self, ip: i64, username: &str, password: &str) -> Result<bool> {
        // Check user credentials
        let user_match = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE game_ip = $1 AND login = $2 AND password = crypt($3, password)",
            ip,
            username,
            password
        )
        .fetch_one(&self.db)
        .await?;
        
        if user_match.unwrap_or(0) > 0 {
            return Ok(true);
        }
        
        // Check NPC credentials
        let npc_match = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM npc WHERE npc_ip = $1 AND npc_pass = $2",
            ip,
            password
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(npc_match.unwrap_or(0) > 0)
    }
    
    /// Convert long integer to IPv4 address
    fn long_to_ip(ip_long: i64) -> Ipv4Addr {
        Ipv4Addr::new(
            ((ip_long >> 24) & 0xFF) as u8,
            ((ip_long >> 16) & 0xFF) as u8,
            ((ip_long >> 8) & 0xFF) as u8,
            (ip_long & 0xFF) as u8,
        )
    }
    
    /// Convert IPv4 address to long integer
    fn ip_to_long(ip: Ipv4Addr) -> i64 {
        let octets = ip.octets();
        ((octets[0] as i64) << 24) +
        ((octets[1] as i64) << 16) +
        ((octets[2] as i64) << 8) +
        (octets[3] as i64)
    }
}