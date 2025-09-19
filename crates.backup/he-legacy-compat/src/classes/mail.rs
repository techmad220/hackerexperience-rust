//! Mail class - 1:1 port of Mail.class.php
//! 
//! Complete in-game mail system handling:
//! - Mail composition and sending
//! - Mail reading and management
//! - Inbox and outbox functionality
//! - Mail validation and permissions
//! - Reply system

use std::collections::HashMap;
use sqlx::Row;
use serde::{Serialize, Deserialize};
use crate::classes::{player::Player, system::System};
use crate::session::{PhpSession, SessionValue};
use he_db::DbPool;
use chrono::{DateTime, Utc};

/// Mail information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailInfo {
    pub id: i64,
    pub from_user_id: i64,
    pub to_user_id: i64,
    pub subject: String,
    pub content: String,
    pub sent_date: DateTime<Utc>,
    pub is_read: bool,
    pub is_deleted: bool,
    pub reply_to: Option<i64>,
}

/// Mail system errors
#[derive(Debug)]
pub enum MailError {
    DatabaseError(sqlx::Error),
    ValidationError(String),
    PermissionError(String),
    NotFound(String),
}

impl std::fmt::Display for MailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MailError::DatabaseError(e) => write!(f, "Database error: {}", e),
            MailError::ValidationError(e) => write!(f, "Validation error: {}", e),
            MailError::PermissionError(e) => write!(f, "Permission error: {}", e),
            MailError::NotFound(e) => write!(f, "Not found: {}", e),
        }
    }
}

impl std::error::Error for MailError {}

impl From<sqlx::Error> for MailError {
    fn from(error: sqlx::Error) -> Self {
        MailError::DatabaseError(error)
    }
}

/// Mail class - handles all mail-related operations
pub struct Mail {
    db_pool: DbPool,
    player: Player,
    mail_info: Option<MailInfo>,
}

impl Mail {
    /// Create new Mail instance
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            player: Player::new(db_pool.clone()),
            db_pool,
            mail_info: None,
        }
    }

    /// Handle POST data for mail operations
    /// 
    /// Port of: handlePost() method
    /// Actions:
    /// - new: Send new mail
    /// - reply: Reply to existing mail
    /// - delete: Delete mail
    /// - read: Mark as read
    pub async fn handle_post(&mut self, post_data: HashMap<String, String>) -> Result<String, MailError> {
        let system = System::new();
        
        let act = post_data.get("act")
            .ok_or_else(|| MailError::ValidationError("Missing action".to_string()))?;

        match act.as_str() {
            "new" => self.handle_new_mail(post_data).await,
            "reply" => self.handle_reply_mail(post_data).await,
            "delete" => self.handle_delete_mail(post_data).await,
            "read" => self.handle_read_mail(post_data).await,
            _ => Err(MailError::ValidationError("Invalid action".to_string())),
        }
    }

    /// Handle new mail sending
    async fn handle_new_mail(&mut self, post_data: HashMap<String, String>) -> Result<String, MailError> {
        let to = post_data.get("to")
            .ok_or_else(|| MailError::ValidationError("Missing recipient".to_string()))?;
        
        let subject = post_data.get("subject")
            .ok_or_else(|| MailError::ValidationError("Missing subject".to_string()))?;
            
        let content = post_data.get("content")
            .ok_or_else(|| MailError::ValidationError("Missing content".to_string()))?;

        // Determine if recipient is user ID or username
        let recipient_id = if to.chars().all(char::is_numeric) {
            // Numeric - treat as user ID
            let user_id: i64 = to.parse()
                .map_err(|_| MailError::ValidationError("Invalid user ID".to_string()))?;
                
            if !self.player.verify_id(user_id).await? {
                return Err(MailError::ValidationError(format!("User ID {} does not exist", user_id)));
            }
            user_id
        } else {
            // String - treat as username
            let system = System::new();
            if !system.validate(to, "username") {
                return Err(MailError::ValidationError(format!("Invalid username: {}", to)));
            }
            
            if !self.player.isset_user(to).await? {
                return Err(MailError::ValidationError(format!("Username {} does not exist", to)));
            }
            
            self.player.get_id_by_user(to).await?
                .ok_or_else(|| MailError::ValidationError(format!("Could not find user ID for username: {}", to)))?
        };

        // TODO: Get sender ID from session - for now using placeholder
        let sender_id = 1i64; // This should come from session
        
        if recipient_id == sender_id {
            return Err(MailError::ValidationError("You cannot send mail to yourself".to_string()));
        }

        // Insert mail into database
        self.send_mail(sender_id, recipient_id, subject, content, None).await?;

        Ok("Mail sent successfully".to_string())
    }

    /// Handle mail reply
    async fn handle_reply_mail(&mut self, post_data: HashMap<String, String>) -> Result<String, MailError> {
        let mid = post_data.get("mid")
            .ok_or_else(|| MailError::ValidationError("Missing mail ID".to_string()))?;
            
        let mail_id: i64 = mid.parse()
            .map_err(|_| MailError::ValidationError("Invalid mail ID".to_string()))?;

        if !self.isset_mail(mail_id).await? {
            return Err(MailError::ValidationError("Mail does not exist".to_string()));
        }

        let subject = post_data.get("subject")
            .ok_or_else(|| MailError::ValidationError("Missing subject".to_string()))?;
            
        let content = post_data.get("content")
            .ok_or_else(|| MailError::ValidationError("Missing content".to_string()))?;

        // Get original mail info
        let original_mail = self.return_mail_info(mail_id).await?
            .ok_or_else(|| MailError::NotFound("Original mail not found".to_string()))?;

        // TODO: Get sender ID from session
        let sender_id = 1i64;

        // Send reply
        self.send_mail(sender_id, original_mail.from_user_id, subject, content, Some(mail_id)).await?;

        Ok("Reply sent successfully".to_string())
    }

    /// Handle mail deletion
    async fn handle_delete_mail(&mut self, post_data: HashMap<String, String>) -> Result<String, MailError> {
        let mid = post_data.get("mid")
            .ok_or_else(|| MailError::ValidationError("Missing mail ID".to_string()))?;
            
        let mail_id: i64 = mid.parse()
            .map_err(|_| MailError::ValidationError("Invalid mail ID".to_string()))?;

        // TODO: Verify user owns this mail
        self.delete_mail(mail_id).await?;

        Ok("Mail deleted successfully".to_string())
    }

    /// Handle marking mail as read
    async fn handle_read_mail(&mut self, post_data: HashMap<String, String>) -> Result<String, MailError> {
        let mid = post_data.get("mid")
            .ok_or_else(|| MailError::ValidationError("Missing mail ID".to_string()))?;
            
        let mail_id: i64 = mid.parse()
            .map_err(|_| MailError::ValidationError("Invalid mail ID".to_string()))?;

        self.mark_as_read(mail_id).await?;

        Ok("Mail marked as read".to_string())
    }

    /// Send a mail
    async fn send_mail(
        &self, 
        from_user_id: i64, 
        to_user_id: i64, 
        subject: &str, 
        content: &str, 
        reply_to: Option<i64>
    ) -> Result<i64, MailError> {
        let mail_id = sqlx::query!(
            "INSERT INTO mail (from_user_id, to_user_id, subject, content, reply_to, sent_date, is_read, is_deleted) VALUES (?, ?, ?, ?, ?, NOW(), 0, 0)",
            from_user_id,
            to_user_id,
            subject,
            content,
            reply_to
        )
        .execute(&self.db_pool)
        .await?
        .last_insert_id() as i64;

        Ok(mail_id)
    }

    /// Check if mail exists
    /// 
    /// Port of: issetMail() method
    pub async fn isset_mail(&self, mail_id: i64) -> Result<bool, MailError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM mail WHERE id = ?"
        )
        .bind(mail_id)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(count > 0)
    }

    /// Check if mail is deleted
    /// 
    /// Port of: isDeleted() method
    pub async fn is_deleted(&self, mail_id: i64) -> Result<bool, MailError> {
        let is_deleted = sqlx::query_scalar::<_, bool>(
            "SELECT is_deleted FROM mail WHERE id = ?"
        )
        .bind(mail_id)
        .fetch_optional(&self.db_pool)
        .await?
        .unwrap_or(true);

        Ok(is_deleted)
    }

    /// Get mail information
    /// 
    /// Port of: returnMailInfo() method
    pub async fn return_mail_info(&self, mail_id: i64) -> Result<Option<MailInfo>, MailError> {
        let row = sqlx::query(
            "SELECT id, from_user_id, to_user_id, subject, content, sent_date, is_read, is_deleted, reply_to FROM mail WHERE id = ?"
        )
        .bind(mail_id)
        .fetch_optional(&self.db_pool)
        .await?;

        match row {
            Some(row) => {
                let mail_info = MailInfo {
                    id: row.get("id"),
                    from_user_id: row.get("from_user_id"),
                    to_user_id: row.get("to_user_id"),
                    subject: row.get("subject"),
                    content: row.get("content"),
                    sent_date: row.get("sent_date"),
                    is_read: row.get("is_read"),
                    is_deleted: row.get("is_deleted"),
                    reply_to: row.get("reply_to"),
                };
                Ok(Some(mail_info))
            },
            None => Ok(None),
        }
    }

    /// Get mail sender ID
    /// 
    /// Port of: returnMailFrom() method  
    pub async fn return_mail_from(&self, mail_id: i64) -> Result<Option<i64>, MailError> {
        let from_user_id = sqlx::query_scalar::<_, i64>(
            "SELECT from_user_id FROM mail WHERE id = ?"
        )
        .bind(mail_id)
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(from_user_id)
    }

    /// Display inbox mails
    /// 
    /// Port of: listMails() method
    pub async fn list_mails(&self) -> Result<String, MailError> {
        // TODO: Get user ID from session
        let user_id = 1i64;

        let mails = sqlx::query!(
            "SELECT m.id, m.subject, m.sent_date, m.is_read, u.login as sender_name 
             FROM mail m 
             JOIN users u ON m.from_user_id = u.id 
             WHERE m.to_user_id = ? AND m.is_deleted = 0 
             ORDER BY m.sent_date DESC",
            user_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut html = String::from("<div class='mail-list'>");
        
        for mail in mails {
            let read_class = if mail.is_read { "read" } else { "unread" };
            html.push_str(&format!(
                r#"<div class="mail-item {}">
                    <a href="?id={}">
                        <span class="sender">{}</span>
                        <span class="subject">{}</span>
                        <span class="date">{}</span>
                    </a>
                </div>"#,
                read_class,
                mail.id,
                html_escape::encode_text(&mail.sender_name),
                html_escape::encode_text(&mail.subject),
                mail.sent_date.format("%Y-%m-%d %H:%M")
            ));
        }
        
        html.push_str("</div>");
        Ok(html)
    }

    /// Display sent mails
    /// 
    /// Port of: listSentMails() method
    pub async fn list_sent_mails(&self) -> Result<String, MailError> {
        // TODO: Get user ID from session
        let user_id = 1i64;

        let mails = sqlx::query!(
            "SELECT m.id, m.subject, m.sent_date, u.login as recipient_name 
             FROM mail m 
             JOIN users u ON m.to_user_id = u.id 
             WHERE m.from_user_id = ? 
             ORDER BY m.sent_date DESC",
            user_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut html = String::from("<div class='sent-mail-list'>");
        
        for mail in mails {
            html.push_str(&format!(
                r#"<div class="mail-item">
                    <a href="?id={}">
                        <span class="recipient">{}</span>
                        <span class="subject">{}</span>
                        <span class="date">{}</span>
                    </a>
                </div>"#,
                mail.id,
                html_escape::encode_text(&mail.recipient_name),
                html_escape::encode_text(&mail.subject),
                mail.sent_date.format("%Y-%m-%d %H:%M")
            ));
        }
        
        html.push_str("</div>");
        Ok(html)
    }

    /// Show specific mail
    /// 
    /// Port of: showMail() method
    pub async fn show_mail(&self, mail_id: i64) -> Result<String, MailError> {
        let mail = self.return_mail_info(mail_id).await?
            .ok_or_else(|| MailError::NotFound("Mail not found".to_string()))?;

        // Mark as read if it's unread
        if !mail.is_read {
            self.mark_as_read(mail_id).await?;
        }

        // Get sender name
        let sender_name = self.player.get_username_by_id(mail.from_user_id).await?
            .unwrap_or_else(|| "Unknown".to_string());

        let html = format!(
            r#"<div class="mail-content">
                <div class="mail-header">
                    <h3>{}</h3>
                    <p>From: {} | Date: {}</p>
                </div>
                <div class="mail-body">
                    {}
                </div>
                <div class="mail-actions">
                    <a href="?action=new&reply={}" class="btn btn-primary">Reply</a>
                    <form method="POST" style="display: inline;">
                        <input type="hidden" name="act" value="delete">
                        <input type="hidden" name="mid" value="{}">
                        <button type="submit" class="btn btn-danger">Delete</button>
                    </form>
                </div>
            </div>"#,
            html_escape::encode_text(&mail.subject),
            html_escape::encode_text(&sender_name),
            mail.sent_date.format("%Y-%m-%d %H:%M:%S"),
            html_escape::encode_text(&mail.content),
            mail_id,
            mail_id
        );

        Ok(html)
    }

    /// Show new mail composition form
    /// 
    /// Port of: show_sendMail() method
    pub async fn show_send_mail(&self) -> Result<String, MailError> {
        let html = r#"
        <div class="new-mail-form">
            <form method="POST">
                <input type="hidden" name="act" value="new">
                <div class="form-group">
                    <label>To (username or ID):</label>
                    <input type="text" name="to" class="form-control" required>
                </div>
                <div class="form-group">
                    <label>Subject:</label>
                    <input type="text" name="subject" class="form-control" required>
                </div>
                <div class="form-group">
                    <label>Message:</label>
                    <textarea name="content" class="form-control" rows="10" required></textarea>
                </div>
                <button type="submit" class="btn btn-primary">Send Mail</button>
            </form>
        </div>
        "#;

        Ok(html.to_string())
    }

    /// Mark mail as read
    async fn mark_as_read(&self, mail_id: i64) -> Result<(), MailError> {
        sqlx::query!("UPDATE mail SET is_read = 1 WHERE id = ?", mail_id)
            .execute(&self.db_pool)
            .await?;

        Ok(())
    }

    /// Delete mail
    async fn delete_mail(&self, mail_id: i64) -> Result<(), MailError> {
        sqlx::query!("UPDATE mail SET is_deleted = 1 WHERE id = ?", mail_id)
            .execute(&self.db_pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mail_info_creation() {
        let mail_info = MailInfo {
            id: 1,
            from_user_id: 100,
            to_user_id: 200,
            subject: "Test Subject".to_string(),
            content: "Test Content".to_string(),
            sent_date: Utc::now(),
            is_read: false,
            is_deleted: false,
            reply_to: None,
        };

        assert_eq!(mail_info.id, 1);
        assert_eq!(mail_info.from_user_id, 100);
        assert_eq!(mail_info.to_user_id, 200);
        assert!(!mail_info.is_read);
        assert!(!mail_info.is_deleted);
    }
}