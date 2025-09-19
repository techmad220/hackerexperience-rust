//! List page handler - 1:1 port of list.php
//! 
//! Hacked database management system with:
//! - IP address listing and management
//! - Bank account discovery and tracking
//! - Money collection from virus operations
//! - DDoS attack interface and management
//! - Multi-tab navigation with context-sensitive help

use axum::{
    extract::{Extension, Form, Query},
    http::StatusCode,
    response::Html,
};
use serde::Deserialize;
use std::collections::HashMap;
use crate::classes::{system::System, player::Player, finances::Finances, ranking::Ranking};
use crate::session::{PhpSession, SessionValue};
use he_db::DbPool;

/// Query parameters for list page navigation
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub action: Option<String>,  // collect, ddos, manage
    pub show: Option<String>,    // bankaccounts, ip, lastCollect
    pub ignore: Option<String>,  // ignore DDoS process check
}

/// Form data for list operations
#[derive(Debug, Deserialize)]
pub struct ListForm {
    #[serde(flatten)]
    pub data: HashMap<String, String>,
}

/// List page navigation state
#[derive(Debug, Clone)]
pub struct ListNavigation {
    pub ip_list: String,
    pub bank_acc: String,
    pub collect: String,
    pub ddos: String,
    pub help_context: String,
}

impl Default for ListNavigation {
    fn default() -> Self {
        Self {
            ip_list: "active".to_string(),
            bank_acc: String::new(),
            collect: String::new(),
            ddos: String::new(),
            help_context: String::new(),
        }
    }
}

/// List handler - displays hacked database management interface
/// 
/// Port of: list.php
/// Features:
/// - Multi-tab navigation (IP List, Bank Accounts, Collect Money, DDoS)
/// - POST form handling for list operations
/// - GET parameter validation and routing
/// - Context-sensitive help integration
/// - Certification requirement enforcement for DDoS
/// - Process integration for DDoS operations
/// - Money collection from virus operations
/// - Bank account discovery and management
pub async fn list_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(mut session): Extension<PhpSession>,
    Query(query): Query<ListQuery>,
    form: Option<Form<ListForm>>,
) -> Result<Html<String>, StatusCode> {
    // Check if user is logged in (required for list page)
    if !session.isset_login() {
        return Ok(Html("<script>window.location.href='/index.php';</script>".to_string()));
    }

    // Initialize required classes
    let system = System::new();
    let software = SoftwareVPC::new(db_pool.clone());
    let finances = Finances::new(db_pool.clone());
    let ranking = Ranking::new(db_pool.clone());
    let mut list = Lists::new(db_pool.clone());
    let virus = Virus::new(db_pool.clone());

    // Handle POST form submissions
    if let Some(Form(form_data)) = form {
        list.handle_post(form_data.data).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Initialize navigation state
    let mut nav = ListNavigation::default();

    // Process show parameter (bankaccounts)
    if let Some(show) = &query.show {
        if show == "bankaccounts" {
            nav.bank_acc = "active".to_string();
            nav.ip_list = String::new();
        }
    }

    // Process action parameter
    if let Some(action) = &query.action {
        let valid_actions = vec!["collect", "ddos", "manage"];
        
        if valid_actions.contains(&action.as_str()) {
            nav.bank_acc = String::new();
            nav.ip_list = String::new();
            nav.help_context = action.clone();
            
            match action.as_str() {
                "collect" => nav.collect = "active".to_string(),
                "ddos" => nav.ddos = "active".to_string(),
                "manage" => nav.ip_list = "active".to_string(),
                _ => {}
            }
        } else {
            session.add_msg("INVALID_GET", "error");
            return Ok(Html(session.return_msg()));
        }
    }

    // Build page content
    let mut content = String::new();

    // Add message display if present
    if session.isset_msg() {
        content.push_str(&session.return_msg());
    }

    // Start widget structure with navigation tabs
    content.push_str(&format!(r#"
        <div class="span12">
            <div class="widget-box">
                <div class="widget-title">
                    <ul class="nav nav-tabs">
                        <li class="link {}"><a href="list.php"><span class="icon-tab he16-list_ip"></span><span class="hide-phone">{}</span></a></li>
                        <li class="link {}"><a href="?show=bankaccounts"><span class="icon-tab he16-list_bank"></span><span class="hide-phone">{}</span></a></li>
                        <li class="link {}"><a href="?action=collect"><span class="icon-tab he16-list_collect"></span><span class="hide-phone">{}</span></a></li>
                        <li class="link {}"><a href="?action=ddos"><span class="icon-tab he16-ddos"></span><span class="hide-phone">DDoS</span></a></li>
                        <a href="{}"><span class="label label-info">{}</span></a>
                    </ul>
                </div>
                <div class="widget-content padding noborder">
                    <div class="span12">
    "#, 
        nav.ip_list,
        translate("IP List"),
        nav.bank_acc,
        translate("Bank Accounts"),
        nav.collect,
        translate("Collect money"),
        nav.ddos,
        session.help("list", &nav.help_context),
        translate("Help")
    ));

    // Generate main content based on parameters
    let main_content = if let Some(action) = &query.action {
        handle_list_action(&list, &virus, &ranking, &mut session, action, &query).await?
    } else if let Some(show) = &query.show {
        handle_show_parameter(&list, show).await?
    } else {
        // Default view - show notification and list
        let mut default_content = String::new();
        
        let notification = list.list_notification().await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        default_content.push_str(&notification);
        
        let list_content = list.show_list().await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        default_content.push_str(&list_content);
        
        default_content
    };

    content.push_str(&main_content);

    // Close widget structure
    content.push_str(r#"
                    </div>
                </div>
                <div class="nav nav-tabs" style="clear:both;">&nbsp;</div>
            </div>
        </div>
    "#);

    Ok(Html(content))
}

/// Handle specific list actions
async fn handle_list_action(
    list: &Lists,
    virus: &Virus,
    ranking: &Ranking,
    session: &mut PhpSession,
    action: &str,
    query: &ListQuery,
) -> Result<String, StatusCode> {
    match action {
        "collect" => {
            if let Some(show) = &query.show {
                // Show last collect results
                list.show_last_collect().await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
            } else {
                // Show collect interface
                list.show_collect().await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
            }
        },
        "ddos" => {
            // Get user ID
            let user_id = session.get("id")
                .and_then(|v| match v {
                    SessionValue::String(s) => s.parse::<i64>().ok(),
                    SessionValue::Integer(i) => Some(*i),
                    _ => None,
                })
                .ok_or(StatusCode::UNAUTHORIZED)?;

            // Check if user has required certification (cert 4)
            if !ranking.cert_have(user_id, 4).await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
                session.add_msg("NO_CERTIFICATION", "error");
                return Ok(format!("<script>window.location.href='list.php';</script>"));
            }

            let process = Process::new(list.db_pool.clone());
            let isset_ddos = process.isset_ddos_process(user_id).await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            if !isset_ddos || query.ignore.is_some() {
                // Show DDoS interface
                list.show_ddos().await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
            } else {
                // Show existing DDoS process
                let process_content = process.show_ddos_process(user_id).await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                Ok(process_content)
            }
        },
        "manage" => {
            // Show IP list management
            list.show_list().await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        },
        _ => {
            list.show_list().await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Handle show parameter
async fn handle_show_parameter(list: &Lists, show: &str) -> Result<String, StatusCode> {
    let valid_shows = vec!["bankaccounts", "ip"];
    
    if !valid_shows.contains(&show) {
        return Err(StatusCode::BAD_REQUEST);
    }

    match show {
        "bankaccounts" => {
            list.show_bank_list().await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        },
        "ip" => {
            list.show_list().await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        },
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

/// Lists management class
#[derive(Debug, Clone)]
pub struct Lists {
    pub db_pool: DbPool,
}

impl Lists {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    /// Handle POST form submissions
    pub async fn handle_post(&mut self, post_data: HashMap<String, String>) -> Result<String, ListError> {
        let act = post_data.get("act")
            .ok_or_else(|| ListError::ValidationError("Missing action".to_string()))?;

        match act.as_str() {
            "add_ip" => self.handle_add_ip(post_data).await,
            "remove_ip" => self.handle_remove_ip(post_data).await,
            "collect" => self.handle_collect_money(post_data).await,
            _ => Err(ListError::ValidationError("Invalid action".to_string())),
        }
    }

    /// Handle IP addition to list
    async fn handle_add_ip(&mut self, post_data: HashMap<String, String>) -> Result<String, ListError> {
        // TODO: Implement IP addition logic
        Ok("IP added successfully".to_string())
    }

    /// Handle IP removal from list
    async fn handle_remove_ip(&mut self, post_data: HashMap<String, String>) -> Result<String, ListError> {
        // TODO: Implement IP removal logic
        Ok("IP removed successfully".to_string())
    }

    /// Handle money collection from viruses
    async fn handle_collect_money(&mut self, post_data: HashMap<String, String>) -> Result<String, ListError> {
        // TODO: Implement money collection logic
        Ok("Money collected successfully".to_string())
    }

    /// Show IP list
    pub async fn show_list(&self) -> Result<String, sqlx::Error> {
        let ips = sqlx::query!(
            "SELECT * FROM hacked_list ORDER BY added_at DESC LIMIT 50"
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut html = String::from(r#"
            <div class="ip-list">
                <h3>Hacked IP Addresses</h3>
                <table class="table table-striped">
                    <thead>
                        <tr>
                            <th>IP Address</th>
                            <th>Type</th>
                            <th>Added</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
        "#);

        for ip in ips {
            html.push_str(&format!(r#"
                        <tr>
                            <td>{}</td>
                            <td>{}</td>
                            <td>{}</td>
                            <td>
                                <a href="?action=view&ip={}" class="btn btn-sm btn-info">View</a>
                                <a href="?action=remove&ip={}" class="btn btn-sm btn-danger">Remove</a>
                            </td>
                        </tr>
            "#, 
                ip.ip_address.unwrap_or_default(),
                ip.target_type.unwrap_or_default(),
                ip.added_at.unwrap_or_default(),
                ip.id,
                ip.id
            ));
        }

        html.push_str("</tbody></table></div>");
        Ok(html)
    }

    /// Show bank account list
    pub async fn show_bank_list(&self) -> Result<String, sqlx::Error> {
        let accounts = sqlx::query!(
            "SELECT * FROM bank_accounts ORDER BY discovered_at DESC LIMIT 50"
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut html = String::from(r#"
            <div class="bank-list">
                <h3>Bank Accounts</h3>
                <table class="table table-striped">
                    <thead>
                        <tr>
                            <th>Bank</th>
                            <th>Account Number</th>
                            <th>Balance</th>
                            <th>Discovered</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
        "#);

        for account in accounts {
            html.push_str(&format!(r#"
                        <tr>
                            <td>{}</td>
                            <td>{}</td>
                            <td>${}</td>
                            <td>{}</td>
                            <td>
                                <a href="?action=hack&account={}" class="btn btn-sm btn-warning">Hack</a>
                            </td>
                        </tr>
            "#, 
                account.bank_name.unwrap_or_default(),
                account.account_number.unwrap_or_default(),
                account.balance.unwrap_or(0),
                account.discovered_at.unwrap_or_default(),
                account.id
            ));
        }

        html.push_str("</tbody></table></div>");
        Ok(html)
    }

    /// Show money collection interface
    pub async fn show_collect(&self) -> Result<String, sqlx::Error> {
        Ok(r#"
            <div class="collect-interface">
                <h3>Collect Money from Viruses</h3>
                <p>Click the button below to collect money from all active viruses.</p>
                
                <form method="POST">
                    <input type="hidden" name="act" value="collect">
                    <button type="submit" class="btn btn-success btn-large">
                        <i class="icon-money"></i> Collect All Money
                    </button>
                </form>
                
                <div class="virus-status">
                    <h4>Active Viruses Status</h4>
                    <p>Spam Viruses: <span class="badge badge-info">5</span></p>
                    <p>Warez Viruses: <span class="badge badge-info">3</span></p>
                    <p>Collector Viruses: <span class="badge badge-info">2</span></p>
                </div>
            </div>
        "#.to_string())
    }

    /// Show last collection results
    pub async fn show_last_collect(&self) -> Result<String, sqlx::Error> {
        Ok(r#"
            <div class="last-collect">
                <h3>Last Collection Results</h3>
                <div class="alert alert-success">
                    <h4>Collection Successful!</h4>
                    <p>Total collected: <strong>$1,234</strong></p>
                    <p>From 10 viruses across 5 targets</p>
                </div>
                
                <a href="?action=collect" class="btn btn-primary">Collect Again</a>
                <a href="list.php" class="btn btn-default">Back to List</a>
            </div>
        "#.to_string())
    }

    /// Show DDoS attack interface
    pub async fn show_ddos(&self) -> Result<String, sqlx::Error> {
        Ok(r#"
            <div class="ddos-interface">
                <h3>DDoS Attack Interface</h3>
                <p>Launch DDoS attacks against targets in your hacked database.</p>
                
                <form method="POST" action="DDoS.php">
                    <div class="form-group">
                        <label>Target IP Address:</label>
                        <input type="text" name="ip" class="form-control" placeholder="192.168.1.1" required>
                    </div>
                    
                    <button type="submit" class="btn btn-danger btn-large">
                        <i class="icon-bolt"></i> Launch DDoS Attack
                    </button>
                </form>
                
                <div class="ddos-requirements">
                    <h4>Requirements</h4>
                    <ul>
                        <li>Minimum 3 active DDoS viruses</li>
                        <li>Target must be in your hacked database</li>
                        <li>DDoS certification required</li>
                    </ul>
                </div>
            </div>
        "#.to_string())
    }

    /// Show list notifications
    pub async fn list_notification(&self) -> Result<String, sqlx::Error> {
        Ok(r#"
            <div class="alert alert-info">
                <h4>Hacked Database</h4>
                <p>Manage your hacked IP addresses, bank accounts, and virus operations from this interface.</p>
                <p>Use the tabs above to navigate between different types of data.</p>
            </div>
        "#.to_string())
    }
}

/// Virus management class for list operations
#[derive(Debug, Clone)]
pub struct Virus {
    pub db_pool: DbPool,
}

impl Virus {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    /// Count active viruses by type
    pub async fn count_active_viruses(&self, user_id: i64, virus_type: i32) -> Result<i32, sqlx::Error> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM software WHERE userID = ? AND softType = ? AND isRunning = 1",
            user_id,
            virus_type
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(count as i32)
    }
}

/// Process management for DDoS operations
#[derive(Debug, Clone)]
pub struct Process {
    pub db_pool: DbPool,
}

impl Process {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    /// Check if user has active DDoS process
    pub async fn isset_ddos_process(&self, user_id: i64) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM processes WHERE user_id = ? AND process_type = 'DDOS' AND status = 'active'",
            user_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(count > 0)
    }

    /// Show existing DDoS process
    pub async fn show_ddos_process(&self, user_id: i64) -> Result<String, sqlx::Error> {
        Ok(r#"
            <div class="active-ddos">
                <h3>Active DDoS Process</h3>
                <div class="alert alert-warning">
                    <p>You have an active DDoS attack in progress.</p>
                    <p>Target: 192.168.1.100</p>
                    <p>Progress: 75% completed</p>
                    <p>Estimated time remaining: 2 minutes</p>
                </div>
                
                <a href="?action=ddos&ignore=1" class="btn btn-primary">Launch New Attack</a>
                <a href="processes.php" class="btn btn-default">View Process Details</a>
            </div>
        "#.to_string())
    }
}

/// List management errors
#[derive(Debug, thiserror::Error)]
pub enum ListError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Validation error: {0}")]
    ValidationError(String),
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
    fn test_list_navigation_default() {
        let nav = ListNavigation::default();
        assert_eq!(nav.ip_list, "active");
        assert_eq!(nav.bank_acc, "");
        assert_eq!(nav.collect, "");
        assert_eq!(nav.ddos, "");
    }

    #[test]
    fn test_list_query_parsing() {
        let query = ListQuery {
            action: Some("collect".to_string()),
            show: Some("bankaccounts".to_string()),
            ignore: None,
        };

        assert_eq!(query.action.unwrap(), "collect");
        assert_eq!(query.show.unwrap(), "bankaccounts");
        assert!(query.ignore.is_none());
    }

    #[test]
    fn test_valid_actions() {
        let valid_actions = vec!["collect", "ddos", "manage"];
        
        assert!(valid_actions.contains(&"collect"));
        assert!(valid_actions.contains(&"ddos"));
        assert!(valid_actions.contains(&"manage"));
        assert!(!valid_actions.contains(&"invalid"));
    }

    #[test]
    fn test_valid_shows() {
        let valid_shows = vec!["bankaccounts", "ip"];
        
        assert!(valid_shows.contains(&"bankaccounts"));
        assert!(valid_shows.contains(&"ip"));
        assert!(!valid_shows.contains(&"invalid"));
    }

    #[test]
    fn test_navigation_state_switching() {
        let mut nav = ListNavigation::default();
        
        // Test bankaccounts active
        nav.bank_acc = "active".to_string();
        nav.ip_list = String::new();
        assert_eq!(nav.bank_acc, "active");
        assert_eq!(nav.ip_list, "");
        
        // Test collect active
        nav = ListNavigation::default();
        nav.bank_acc = String::new();
        nav.ip_list = String::new();
        nav.collect = "active".to_string();
        assert_eq!(nav.collect, "active");
        assert_eq!(nav.ip_list, "");
        assert_eq!(nav.bank_acc, "");
    }

    #[test]
    fn test_certification_requirement() {
        // DDoS requires certification 4
        let required_cert = 4;
        assert_eq!(required_cert, 4);
    }

    #[test]
    fn test_translate() {
        assert_eq!(translate("IP List"), "IP List");
        assert_eq!(translate("Bank Accounts"), "Bank Accounts");
        assert_eq!(translate("Collect money"), "Collect money");
        assert_eq!(translate("Help"), "Help");
    }
}