//! Mail page handler - 1:1 port of mail.php
//! 
//! Complex mail system with multiple views:
//! - Inbox (default)
//! - Outbox (sent messages)
//! - New message composition
//! - Current message view
//! Handles mail navigation, message validation, and state management.

use axum::{
    extract::{Extension, Form, Query},
    http::StatusCode,
    response::Html,
};
use serde::Deserialize;
use std::collections::HashMap;
use crate::classes::{mail::Mail, system::System, player::Player};
use crate::session::{PhpSession, SessionValue};
use he_db::DbPool;

/// Query parameters for mail page navigation
#[derive(Debug, Deserialize)]
pub struct MailQuery {
    pub action: Option<String>, // new, outbox
    pub id: Option<String>,     // Mail ID
    pub show: Option<String>,   // Show parameter
}

/// Form data for mail operations
#[derive(Debug, Deserialize)]
pub struct MailForm {
    #[serde(flatten)]
    pub data: HashMap<String, String>,
}

/// Mail page navigation state
#[derive(Debug, Clone)]
pub struct MailNavigation {
    pub current: String,
    pub inbox: String,
    pub sent: String,
    pub new: String,
    pub isset_current: bool,
    pub current_link: String,
}

impl Default for MailNavigation {
    fn default() -> Self {
        Self {
            current: String::new(),
            inbox: " active".to_string(),
            sent: String::new(),
            new: String::new(),
            isset_current: false,
            current_link: String::new(),
        }
    }
}

/// Main mail handler - displays mail interface with dynamic navigation
/// 
/// Port of: mail.php
/// Features:
/// - Multi-tab navigation (Inbox, Outbox, New, Current)
/// - POST form handling for mail operations
/// - Complex session state management for current message
/// - Mail validation and permissions checking
/// - Dynamic tab state based on URL parameters and session
/// - Error handling for invalid mail IDs and permissions
pub async fn mail_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(mut session): Extension<PhpSession>,
    Query(query): Query<MailQuery>,
    form: Option<Form<MailForm>>,
) -> Result<Html<String>, StatusCode> {
    // Check if user is logged in (required for mail page)
    if !session.isset_login() {
        return Ok(Html("<script>window.location.href='/index.php';</script>".to_string()));
    }

    // Get user ID from session
    let user_id = match session.get("id") {
        Some(SessionValue::Integer(id)) => *id,
        Some(SessionValue::String(id_str)) => {
            id_str.parse::<i64>().unwrap_or(0)
        },
        _ => {
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Initialize required classes
    let system = System::new();
    let mut mail = Mail::new(db_pool.clone());
    let _player = Player::new(db_pool.clone());

    // Handle POST form submissions
    if let Some(Form(form_data)) = form {
        mail.handle_post(form_data.data).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Initialize navigation state
    let mut nav = MailNavigation::default();
    let mut invalid = false;
    let mut mid: Option<i64> = None;

    // Handle current message state from session
    if let Some(SessionValue::Integer(cur_mail)) = session.get("CUR_MAIL") {
        if !system.isset_get("show") && std::env::var("PHP_SELF").unwrap_or_default() != "/mail.php" {
            nav.current = " active".to_string();
            nav.inbox = String::new();
            nav.isset_current = true;
        }
    }

    // Process GET parameters for navigation
    if let Some(action) = &query.action {
        if query.id.is_none() {
            nav.inbox = String::new();
            
            match action.as_str() {
                "new" => {
                    nav.new = " active".to_string();
                    nav.current = String::new();
                },
                "outbox" => {
                    nav.sent = " active".to_string();
                },
                _ => {
                    // Invalid action will be handled in content generation
                }
            }
        }
    } else if let Some(id_str) = &query.id {
        // Handle specific mail ID
        if let Ok(mail_id) = id_str.parse::<i64>() {
            nav.current_link = format!("?id={}", mail_id);
            mid = Some(mail_id);
        } else {
            nav.current_link = "mail.php".to_string();
            invalid = true;
        }
        
        nav.current = " active".to_string();
        nav.inbox = String::new();
        nav.isset_current = true;
    }

    // Handle persistent current mail from session
    if let Some(SessionValue::Integer(cur_mail)) = session.get("CUR_MAIL") {
        if query.id.is_none() {
            nav.current_link = format!("?id={}", cur_mail);
            nav.isset_current = true;
            mid = Some(*cur_mail);
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
                    <li class="link{}"><a href="mail.php">{}</a></li>
                    <li class="link{}"><a href="?action=outbox">{}</a></li>
                    <li class="link{}"><a href="?action=new">{}</a></li>
    "#, 
        nav.inbox, 
        translate("Inbox"),
        nav.sent, 
        translate("Outbox"),
        nav.new, 
        translate("New message")
    ));

    // Add current message tab if active
    if nav.isset_current {
        content.push_str(&format!(r#"
                    <li class="link{}"><a href="{}" id="mail-title">{}</a></li>
        "#, 
            nav.current, 
            nav.current_link, 
            translate("Current Message")
        ));
    }

    // Close navigation and start content area
    content.push_str(r#"
                    <li class="link" style="float: right;"><span class="icol32-help"></span></li>
                </ul>
            </div>
            <div class="widget-content padding noborder">
    "#);

    // Generate main content based on current view
    let mut error = String::new();

    if let Some(action) = &query.action {
        match action.as_str() {
            "new" => {
                let new_mail_content = mail.show_send_mail().await
                    .unwrap_or_else(|_| "Error loading new mail form".to_string());
                content.push_str(&new_mail_content);
            },
            "outbox" => {
                let sent_mails_content = mail.list_sent_mails().await
                    .unwrap_or_else(|_| "Error loading sent mails".to_string());
                content.push_str(&sent_mails_content);
            },
            _ => {
                error = "Invalid action".to_string();
            }
        }
    } else if let Some(mail_id) = mid {
        if !invalid {
            // Handle specific mail viewing with permission checks
            match mail.isset_mail(mail_id).await {
                Ok(true) => {
                    let mut valid = false;
                    
                    if !mail.is_deleted(mail_id).await.unwrap_or(true) {
                        // Mail not deleted - check ownership
                        if let Ok(Some(mail_info)) = mail.return_mail_info(mail_id).await {
                            if mail_info.to_user_id == user_id && !mail_info.is_deleted {
                                valid = true;
                            } else if mail_info.from_user_id == user_id {
                                valid = true;
                            }
                        }
                    } else {
                        // Mail is deleted - check if user is sender
                        if let Ok(Some(from_id)) = mail.return_mail_from(mail_id).await {
                            if from_id == user_id {
                                valid = true;
                            }
                        }
                    }
                    
                    if valid {
                        let mail_content = mail.show_mail(mail_id).await
                            .unwrap_or_else(|_| "Error loading mail content".to_string());
                        content.push_str(&mail_content);
                    } else {
                        error = "This email does not exists.".to_string();
                    }
                },
                _ => {
                    error = "This email does not exists.".to_string();
                }
            }
        } else {
            error = "Invalid mail ID.".to_string();
        }
    } else {
        // Default view - list inbox mails
        let mail_list = mail.list_mails().await
            .unwrap_or_else(|_| "Error loading mail list".to_string());
        content.push_str(&mail_list);
    }

    // Handle errors
    if !error.is_empty() {
        let error_content = system.handle_error(&error, "mail.php");
        content.push_str(&error_content);
    }

    // Handle special closing for new/current views
    let nbsp = if !nav.new.is_empty() || !nav.current.is_empty() {
        content.push_str("</div>");
        if !nav.current.is_empty() { "&nbsp;" } else { "" }
    } else {
        ""
    };

    // Close widget structure
    content.push_str(&format!(r#"
        </div>
        <div style="clear: both;" class="nav nav-tabs">{}</div>
    </div>
    "#, nbsp));

    Ok(Html(content))
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
    fn test_mail_navigation_default() {
        let nav = MailNavigation::default();
        assert_eq!(nav.inbox, " active");
        assert_eq!(nav.sent, "");
        assert_eq!(nav.new, "");
        assert_eq!(nav.current, "");
        assert!(!nav.isset_current);
    }

    #[test]
    fn test_mail_id_parsing() {
        assert_eq!("123".parse::<i64>().unwrap(), 123);
        assert!("invalid".parse::<i64>().is_err());
    }

    #[test]
    fn test_translate() {
        assert_eq!(translate("Inbox"), "Inbox");
        assert_eq!(translate("Outbox"), "Outbox");
        assert_eq!(translate("New message"), "New message");
    }
}