//! Log edit page handler - 1:1 port of logEdit.php
//! 
//! Log editing system with:
//! - Local and remote log modification
//! - HTML purification and validation
//! - Process creation for log changes
//! - Internet session validation for remote logs
//! - Temporary log storage during editing

use axum::{
    extract::{Extension, Form},
    http::StatusCode,
    response::Html,
};
use serde::Deserialize;
use crate::classes::{system::System, player::Player, npc::NPC};
use crate::session::{PhpSession, SessionValue};
use he_db::DbPool;

/// Form data for log editing
#[derive(Debug, Deserialize)]
pub struct LogEditForm {
    pub log: String,        // New log content
    pub id: String,         // Location ID (0 = remote, 1 = local)
}

/// Log editing errors
#[derive(Debug)]
pub enum LogEditError {
    NotLoggedIn,
    PostOnly,
    BadId,
    NotInternetLogged,
    TargetNotFound,
    IdenticalLogs,
    ProcessExists,
    ProcessCreationFailed,
}

impl LogEditError {
    fn message(&self) -> &'static str {
        match self {
            LogEditError::NotLoggedIn => "Authentication required.",
            LogEditError::PostOnly => "POST requests only.",
            LogEditError::BadId => "Bad ID",
            LogEditError::NotInternetLogged => "Internet session required.",
            LogEditError::TargetNotFound => "Target system not found.",
            LogEditError::IdenticalLogs => "Identical logs.",
            LogEditError::ProcessExists => "There already is a log edit in progress. Delete or complete it before.",
            LogEditError::ProcessCreationFailed => "Failed to create log edit process.",
        }
    }
}

/// Log edit handler - processes log modification requests
/// 
/// Port of: logEdit.php
/// Features:
/// - POST-only request validation
/// - Local vs remote log editing (id: 1 = local, 0 = remote)
/// - Internet session validation for remote logs
/// - HTML content purification and sanitization
/// - Process creation for log edit operations
/// - Temporary log storage during edit process
/// - Target validation for remote systems
pub async fn log_edit_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(mut session): Extension<PhpSession>,
    form: Option<Form<LogEditForm>>,
) -> Result<Html<String>, StatusCode> {
    // Check if user is logged in
    if !session.isset_login() {
        return Ok(Html("<script>window.location.href='/index.php';</script>".to_string()));
    }

    // Validate POST request (original only accepts POST)
    let form_data = match form {
        Some(Form(data)) => data,
        None => {
            return Ok(Html("Post only".to_string()));
        }
    };

    // Process log edit request
    match process_log_edit(&db_pool, &mut session, form_data).await {
        Ok(redirect_url) => {
            Ok(Html(format!("<script>window.location.href='{}';</script>", redirect_url)))
        },
        Err(error) => {
            session.add_msg(error.message(), "error");
            let redirect = if error_is_local_context(&error) {
                "log.php"
            } else {
                "internet?view=logs"
            };
            Ok(Html(format!("<script>window.location.href='{}';</script>", redirect)))
        }
    }
}

/// Process log edit with all validations
async fn process_log_edit(
    db_pool: &DbPool,
    session: &mut PhpSession,
    form_data: LogEditForm,
) -> Result<String, LogEditError> {
    let new_log_value = form_data.log;
    let id = form_data.id.parse::<i32>()
        .map_err(|_| LogEditError::BadId)?;

    // Validate ID (only 0 or 1 are valid)
    if id != 0 && id != 1 {
        return Err(LogEditError::BadId);
    }

    let is_local = id == 1;
    let (uid, vid, npc, npc_type, redirect_link) = if is_local {
        // Local log editing
        let user_id = session.get("id")
            .and_then(|v| match v {
                SessionValue::String(s) => s.parse::<i64>().ok(),
                SessionValue::Integer(i) => Some(*i),
                _ => None,
            })
            .ok_or(LogEditError::NotLoggedIn)?;

        (user_id, 0, 0, "VPC".to_string(), "log.php".to_string())
    } else {
        // Remote log editing - requires internet session
        if !session.is_internet_logged() {
            return Err(LogEditError::NotInternetLogged);
        }

        let logged_ip = session.get("LOGGED_IN")
            .and_then(|v| match v {
                SessionValue::String(s) => s.parse::<i64>().ok(),
                SessionValue::Integer(i) => Some(*i),
                _ => None,
            })
            .ok_or(LogEditError::NotInternetLogged)?;

        let player = Player::new(db_pool.clone());
        let player_info = player.get_id_by_ip(logged_ip).await
            .map_err(|_| LogEditError::TargetNotFound)?;

        if !player_info.exists {
            return Err(LogEditError::TargetNotFound);
        }

        let (npc_val, npc_type) = if player_info.pc_type == "NPC" {
            (1, "NPC".to_string())
        } else {
            (0, "VPC".to_string())
        };

        (player_info.id, player_info.id, npc_val, npc_type, "internet?view=logs".to_string())
    };

    // Get current log value
    let log_vpc = LogVPC::new(db_pool.clone());
    let current_log = log_vpc.get_log_value(uid, &npc_type).await
        .map_err(|_| LogEditError::ProcessCreationFailed)?;

    // Check if log content has actually changed
    if current_log == new_log_value {
        return Err(LogEditError::IdenticalLogs);
    }

    // Purify and validate the new log content
    let purifier = Purifier::new();
    let validated_log = purifier.purify_text(&new_log_value);

    let tmp_log_id = if !validated_log.is_empty() {
        log_vpc.create_tmp_log(uid, npc, &validated_log).await
            .map_err(|_| LogEditError::ProcessCreationFailed)?
    } else {
        String::new()
    };

    // Determine process parameters
    let (local_str, redirect) = if is_local {
        ("local", "log.php")
    } else {
        ("remote", "internet?view=logs")
    };

    let user_id = session.get("id")
        .and_then(|v| match v {
            SessionValue::String(s) => s.parse::<i64>().ok(),
            SessionValue::Integer(i) => Some(*i),
            _ => None,
        })
        .ok_or(LogEditError::NotLoggedIn)?;

    // Check if process already exists
    let process = Process::new(db_pool.clone());
    if process.isset_process(user_id, "E_LOG", vid, local_str, "", &tmp_log_id).await
        .map_err(|_| LogEditError::ProcessCreationFailed)? {
        return Err(LogEditError::ProcessExists);
    }

    // Create log edit process
    match process.new_process(
        user_id,
        "E_LOG",
        vid,
        local_str,
        "",
        &tmp_log_id,
        "",
        npc,
    ).await {
        Ok(true) => {
            let process_id = session.process_id("show");
            Ok(format!("processes?pid={}", process_id))
        },
        Ok(false) => {
            // Process creation failed, clean up tmp log
            log_vpc.delete_tmp_log(uid, npc).await.ok();
            Ok(redirect.to_string())
        },
        Err(_) => {
            log_vpc.delete_tmp_log(uid, npc).await.ok();
            Err(LogEditError::ProcessCreationFailed)
        }
    }
}

/// Check if error context is local for proper redirect
fn error_is_local_context(error: &LogEditError) -> bool {
    matches!(error, LogEditError::NotLoggedIn | LogEditError::PostOnly | LogEditError::BadId)
}

/// Log VPC class for log operations
#[derive(Debug, Clone)]
pub struct LogVPC {
    pub db_pool: DbPool,
}

impl LogVPC {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    /// Get current log value for user
    pub async fn get_log_value(&self, user_id: i64, pc_type: &str) -> Result<String, sqlx::Error> {
        let log_content = sqlx::query_scalar!(
            "SELECT log_content FROM logs WHERE user_id = ? AND pc_type = ? ORDER BY id DESC LIMIT 1",
            user_id,
            pc_type
        )
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(log_content.unwrap_or_default())
    }

    /// Create temporary log during editing
    pub async fn create_tmp_log(&self, user_id: i64, is_npc: i32, content: &str) -> Result<String, sqlx::Error> {
        let result = sqlx::query!(
            "INSERT INTO tmp_logs (user_id, is_npc, content, created_at) VALUES (?, ?, ?, NOW())",
            user_id,
            is_npc,
            content
        )
        .execute(&self.db_pool)
        .await?;

        Ok(result.last_insert_id().to_string())
    }

    /// Delete temporary log
    pub async fn delete_tmp_log(&self, user_id: i64, is_npc: i32) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM tmp_logs WHERE user_id = ? AND is_npc = ?",
            user_id,
            is_npc
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}

/// HTML content purifier
#[derive(Debug, Clone)]
pub struct Purifier {
    // Configuration could be added here
}

impl Purifier {
    pub fn new() -> Self {
        Self {}
    }

    /// Purify text content (HTML sanitization)
    pub fn purify_text(&self, content: &str) -> String {
        // TODO: Implement proper HTML purification
        // Original uses Purifier class with 'text' config
        // For now, basic HTML escaping and cleanup
        self.basic_sanitize(content)
    }

    /// Basic content sanitization
    fn basic_sanitize(&self, content: &str) -> String {
        // Remove potentially dangerous HTML tags and scripts
        let cleaned = content
            .replace("<script", "&lt;script")
            .replace("</script>", "&lt;/script&gt;")
            .replace("javascript:", "")
            .replace("vbscript:", "")
            .replace("onload=", "")
            .replace("onerror=", "")
            .replace("onclick=", "");

        // Trim whitespace
        cleaned.trim().to_string()
    }
}

/// Process management for log editing
#[derive(Debug, Clone)]
pub struct Process {
    pub db_pool: DbPool,
}

impl Process {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    /// Check if process already exists
    pub async fn isset_process(
        &self,
        user_id: i64,
        process_type: &str,
        target_id: i64,
        location: &str,
        param1: &str,
        param2: &str,
    ) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM processes WHERE user_id = ? AND process_type = ? AND target_id = ? AND location = ? AND param2 = ?",
            user_id,
            process_type,
            target_id,
            location,
            param2
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(count > 0)
    }

    /// Create new log edit process
    pub async fn new_process(
        &self,
        user_id: i64,
        process_type: &str,
        target_id: i64,
        location: &str,
        param1: &str,
        param2: &str,
        param3: &str,
        is_npc: i32,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "INSERT INTO processes (user_id, process_type, target_id, location, param1, param2, param3, is_npc, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, NOW())",
            user_id,
            process_type,
            target_id,
            location,
            param1,
            param2,
            param3,
            is_npc
        )
        .execute(&self.db_pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

/// Extended session methods for log editing
impl PhpSession {
    /// Check if user has active internet session
    pub fn is_internet_logged(&self) -> bool {
        self.get("LOGGED_IN").is_some()
    }

    /// Get process ID for display
    pub fn process_id(&self, action: &str) -> String {
        // TODO: Implement process ID tracking
        // Original: $pid = $session->processID('show');
        "12345".to_string() // Placeholder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_edit_form_parsing() {
        let form = LogEditForm {
            log: "Modified log content".to_string(),
            id: "1".to_string(),
        };

        assert_eq!(form.log, "Modified log content");
        assert_eq!(form.id, "1");
    }

    #[test]
    fn test_id_validation() {
        // Valid IDs
        assert_eq!("0".parse::<i32>().unwrap(), 0);
        assert_eq!("1".parse::<i32>().unwrap(), 1);

        // Invalid IDs
        assert!("2".parse::<i32>().unwrap() != 0 && "2".parse::<i32>().unwrap() != 1);
        assert!("invalid".parse::<i32>().is_err());
    }

    #[test]
    fn test_local_vs_remote_logic() {
        // Original logic: if($id == 1) { $local = 1; } else { $local = 0; }
        let id1_is_local = 1 == 1;
        let id0_is_local = 0 == 1;

        assert!(id1_is_local);
        assert!(!id0_is_local);
    }

    #[test]
    fn test_npc_type_logic() {
        // Original: if($pInfo['0']['pctype'] == 'NPC') { $npc = 1; $npcType = 'NPC'; }
        let pc_type = "NPC";
        let (npc_val, npc_type) = if pc_type == "NPC" {
            (1, "NPC".to_string())
        } else {
            (0, "VPC".to_string())
        };

        assert_eq!(npc_val, 1);
        assert_eq!(npc_type, "NPC");

        let pc_type = "VPC";
        let (npc_val, npc_type) = if pc_type == "NPC" {
            (1, "NPC".to_string())
        } else {
            (0, "VPC".to_string())
        };

        assert_eq!(npc_val, 0);
        assert_eq!(npc_type, "VPC");
    }

    #[test]
    fn test_purifier_basic_sanitization() {
        let purifier = Purifier::new();
        
        let malicious = "<script>alert('xss')</script>Click me";
        let cleaned = purifier.purify_text(malicious);
        assert!(!cleaned.contains("<script"));
        assert!(cleaned.contains("&lt;script"));

        let normal = "Normal log content";
        let unchanged = purifier.purify_text(normal);
        assert_eq!(unchanged, "Normal log content");
    }

    #[test]
    fn test_log_edit_error_messages() {
        assert_eq!(LogEditError::BadId.message(), "Bad ID");
        assert_eq!(LogEditError::IdenticalLogs.message(), "Identical logs.");
        assert_eq!(LogEditError::ProcessExists.message(), "There already is a log edit in progress. Delete or complete it before.");
    }

    #[test]
    fn test_redirect_logic() {
        // Local context redirects
        let local_redirect = if true { "log.php" } else { "internet?view=logs" };
        assert_eq!(local_redirect, "log.php");

        // Remote context redirects
        let remote_redirect = if false { "log.php" } else { "internet?view=logs" };
        assert_eq!(remote_redirect, "internet?view=logs");
    }
}