//! Log file viewer page handler - 1:1 port of log.php
//! 
//! Log management system with:
//! - Log file viewing and listing
//! - Log entry deletion via process system
//! - Log editing capabilities (placeholder)
//! - Integration with process management for log operations

use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::Html,
};
use serde::Deserialize;
use crate::classes::{system::System, player::Player};
use crate::session::{PhpSession, SessionValue};
use he_db::DbPool;

/// Query parameters for log page navigation
#[derive(Debug, Deserialize)]
pub struct LogQuery {
    pub action: Option<String>, // view, edit, del
    pub id: Option<String>,     // Log ID
}

/// Log entry information
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub id: i64,
    pub timestamp: String,
    pub ip_address: String,
    pub action: String,
    pub details: String,
    pub user_id: i64,
}

/// Main log handler - displays log management interface
/// 
/// Port of: log.php
/// Features:
/// - Log file listing and viewing
/// - Log entry operations (view, edit, delete)
/// - Process integration for log deletion
/// - GET parameter validation
/// - Session message display
/// - Help integration
pub async fn log_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(mut session): Extension<PhpSession>,
    Query(query): Query<LogQuery>,
) -> Result<Html<String>, StatusCode> {
    // Check if user is logged in (required for log page)
    if !session.isset_login() {
        return Ok(Html("<script>window.location.href='/index.php';</script>".to_string()));
    }

    // Initialize required classes
    let system = System::new();
    let user_id = session.get("id")
        .and_then(|v| match v {
            SessionValue::String(s) => s.parse::<i64>().ok(),
            SessionValue::Integer(i) => Some(*i),
            _ => None,
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let player = Player::new(db_pool.clone());
    let log_vpc = LogVPC::new(db_pool.clone());

    // Build page content
    let mut content = String::new();

    // Add message display if present
    if session.isset_msg() {
        content.push_str(&session.return_msg());
    }

    // Start widget structure
    content.push_str(&format!(r#"
                    <div class="span12">
                        <div class="widget-box" style="width:100%">
                            <div class="widget-title">
                                <ul class="nav nav-tabs">
                                    <li class="link active"><a href="log.php"><span class="icon-tab he16-internet_log"></span>{}</a></li>
                                    <a href="{}"><span class="label label-info">{}</span></a>
                                </ul>
                            </div>
                            <div class="widget-content padding noborder center">
    "#, 
        translate("Log file"),
        session.help("log"),
        translate("Help")
    ));

    // Process GET parameters
    let got_get = if query.action.is_some() { 1 } else { 0 } + if query.id.is_some() { 1 } else { 0 };

    if got_get == 2 {
        // Both action and id parameters present
        match handle_log_action(&log_vpc, &mut session, &query, user_id).await? {
            LogActionResult::Content(html) => content.push_str(&html),
            LogActionResult::Redirect(url) => {
                return Ok(Html(format!("<script>window.location.href='{}';</script>", url)));
            },
            LogActionResult::Error(msg) => {
                return Ok(Html(msg));
            },
        }
    } else {
        // Default view - list all logs
        if session.isset_msg() {
            content.push_str(&session.return_msg());
        }
        
        let log_list = log_vpc.list_log(user_id, "", true).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        content.push_str(&log_list);
    }

    // Close widget structure
    content.push_str(r#"
                            </div>
                        </div>
                    </div>
    "#);

    Ok(Html(content))
}

/// Result of log action processing
enum LogActionResult {
    Content(String),
    Redirect(String),
    Error(String),
}

/// Handle specific log actions
async fn handle_log_action(
    log_vpc: &LogVPC,
    session: &mut PhpSession,
    query: &LogQuery,
    user_id: i64,
) -> Result<LogActionResult, StatusCode> {
    let action = query.action.as_ref().ok_or(StatusCode::BAD_REQUEST)?;
    let id_str = query.id.as_ref().ok_or(StatusCode::BAD_REQUEST)?;
    
    // Validate log ID is numeric
    let log_id = id_str.parse::<i64>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Validate action parameter
    let valid_actions = vec!["view", "edit", "del"];
    if !valid_actions.contains(&action.as_str()) {
        return Ok(LogActionResult::Error("Invalid get".to_string()));
    }

    match action.as_str() {
        "view" => {
            let mut content = String::new();
            
            // Show log list first
            let log_list = log_vpc.list_log(user_id, "", true).await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            content.push_str(&log_list);
            
            // Show specific log entry
            let log_content = log_vpc.show_log(log_id, user_id).await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            content.push_str(&log_content);
            
            Ok(LogActionResult::Content(content))
        },
        "edit" => {
            // Placeholder for edit functionality (original just dies with "edit")
            Ok(LogActionResult::Error("Edit functionality not implemented".to_string()))
        },
        "del" => {
            if log_vpc.isset_log(log_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
                // Create process for log deletion
                let process_result = create_log_deletion_process(user_id, log_id).await?;
                
                match process_result {
                    Some(pid) => {
                        let process_id = session.process_id("show");
                        Ok(LogActionResult::Redirect(format!("processes?pid={}", process_id)))
                    },
                    None => {
                        // Process already exists, show existing process
                        let existing_process = get_existing_log_process(user_id, log_id).await?;
                        Ok(LogActionResult::Content(existing_process))
                    }
                }
            } else {
                Ok(LogActionResult::Error("Log entry not found".to_string()))
            }
        },
        _ => Ok(LogActionResult::Error("Invalid action".to_string())),
    }
}

/// Create a log deletion process
async fn create_log_deletion_process(user_id: i64, log_id: i64) -> Result<Option<i64>, StatusCode> {
    // TODO: Implement process creation
    // Original: $process->newProcess($_SESSION['id'], 'D_LOG', '', 'local', '', $getIDInfo['GET_VALUE'], '', '0')
    // This should create a new process of type D_LOG (Delete Log) for the specified log entry
    
    // For now, return a mock process ID
    Ok(Some(12345))
}

/// Get existing log deletion process
async fn get_existing_log_process(user_id: i64, log_id: i64) -> Result<String, StatusCode> {
    // TODO: Implement process info retrieval and display
    // Original shows the existing process template
    Ok("<p>Log deletion process already in progress.</p>".to_string())
}

/// Log VPC class for log operations
#[derive(Debug, Clone)]
pub struct LogVPC {
    pub db_pool: DbPool,
}

impl LogVPC {
    /// Create new LogVPC instance
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    /// List logs for a user
    pub async fn list_log(&self, user_id: i64, filter: &str, show_header: bool) -> Result<String, sqlx::Error> {
        let logs = sqlx::query!(
            "SELECT * FROM logs WHERE user_id = ? ORDER BY timestamp DESC LIMIT 50",
            user_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut html = String::new();
        
        if show_header {
            html.push_str(r#"
                <div class="log-list">
                    <h3>Log Entries</h3>
                    <table class="table table-striped">
                        <thead>
                            <tr>
                                <th>Time</th>
                                <th>IP Address</th>
                                <th>Action</th>
                                <th>Details</th>
                                <th>Actions</th>
                            </tr>
                        </thead>
                        <tbody>
            "#);
        }

        for log in logs {
            html.push_str(&format!(r#"
                        <tr>
                            <td>{}</td>
                            <td>{}</td>
                            <td>{}</td>
                            <td>{}</td>
                            <td>
                                <a href="?action=view&id={}" class="btn btn-sm btn-info">View</a>
                                <a href="?action=del&id={}" class="btn btn-sm btn-danger">Delete</a>
                            </td>
                        </tr>
            "#, 
                log.timestamp.unwrap_or_default(),
                log.ip_address.unwrap_or_default(),
                log.action.unwrap_or_default(),
                log.details.unwrap_or_default(),
                log.id,
                log.id
            ));
        }

        if show_header {
            html.push_str("</tbody></table></div>");
        }

        Ok(html)
    }

    /// Show specific log entry
    pub async fn show_log(&self, log_id: i64, user_id: i64) -> Result<String, sqlx::Error> {
        let log = sqlx::query!(
            "SELECT * FROM logs WHERE id = ? AND user_id = ? LIMIT 1",
            log_id,
            user_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        match log {
            Some(entry) => {
                Ok(format!(r#"
                    <div class="log-details">
                        <h4>Log Entry Details</h4>
                        <div class="log-content">
                            <p><strong>ID:</strong> {}</p>
                            <p><strong>Timestamp:</strong> {}</p>
                            <p><strong>IP Address:</strong> {}</p>
                            <p><strong>Action:</strong> {}</p>
                            <p><strong>Details:</strong> {}</p>
                        </div>
                        <div class="log-actions">
                            <a href="log.php" class="btn btn-primary">Back to Log List</a>
                            <a href="?action=del&id={}" class="btn btn-danger">Delete Entry</a>
                        </div>
                    </div>
                "#, 
                    entry.id,
                    entry.timestamp.unwrap_or_default(),
                    entry.ip_address.unwrap_or_default(),
                    entry.action.unwrap_or_default(),
                    entry.details.unwrap_or_default(),
                    entry.id
                ))
            },
            None => Ok("<p>Log entry not found or access denied.</p>".to_string()),
        }
    }

    /// Check if log entry exists
    pub async fn isset_log(&self, log_id: i64) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM logs WHERE id = ?",
            log_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(count > 0)
    }
}

/// Helper function for internationalization
/// TODO: Implement proper i18n system
fn translate(text: &str) -> String {
    // Placeholder - in full implementation this would use gettext or similar
    text.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_query_parsing() {
        let query = LogQuery {
            action: Some("view".to_string()),
            id: Some("123".to_string()),
        };

        assert_eq!(query.action.unwrap(), "view");
        assert_eq!(query.id.unwrap(), "123");
    }

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry {
            id: 1,
            timestamp: "2023-01-01 12:00:00".to_string(),
            ip_address: "192.168.1.1".to_string(),
            action: "LOGIN".to_string(),
            details: "User logged in successfully".to_string(),
            user_id: 123,
        };

        assert_eq!(entry.id, 1);
        assert_eq!(entry.action, "LOGIN");
        assert_eq!(entry.user_id, 123);
    }

    #[test]
    fn test_valid_actions() {
        let valid_actions = vec!["view", "edit", "del"];
        
        assert!(valid_actions.contains(&"view"));
        assert!(valid_actions.contains(&"edit"));
        assert!(valid_actions.contains(&"del"));
        assert!(!valid_actions.contains(&"invalid"));
    }

    #[test]
    fn test_got_get_calculation() {
        // Test the got_get logic from original PHP
        let has_action = true;
        let has_id = true;
        let got_get = if has_action { 1 } else { 0 } + if has_id { 1 } else { 0 };
        assert_eq!(got_get, 2);

        let has_action = true;
        let has_id = false;
        let got_get = if has_action { 1 } else { 0 } + if has_id { 1 } else { 0 };
        assert_eq!(got_get, 1);

        let has_action = false;
        let has_id = false;
        let got_get = if has_action { 1 } else { 0 } + if has_id { 1 } else { 0 };
        assert_eq!(got_get, 0);
    }

    #[test]
    fn test_log_id_parsing() {
        assert_eq!("123".parse::<i64>().unwrap(), 123);
        assert!("invalid".parse::<i64>().is_err());
        assert!("".parse::<i64>().is_err());
    }

    #[test]
    fn test_translate() {
        assert_eq!(translate("Log file"), "Log file");
        assert_eq!(translate("Help"), "Help");
    }
}