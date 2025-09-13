use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use chrono::{DateTime, Utc, NaiveDateTime};

use crate::error::Result;

/// Represents a mail message
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Mail {
    pub id: i32,
    pub from_id: i32,
    pub to_id: i32,
    pub mail_type: i32,
    pub subject: String,
    pub text: String,
    pub date_sent: NaiveDateTime,
    pub is_read: bool,
    pub is_deleted: bool,
}

/// Mail with sender/recipient information
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MailWithInfo {
    pub id: i32,
    pub from_id: i32,
    pub to_id: i32,
    pub mail_type: i32,
    pub subject: String,
    pub text: String,
    pub date_sent: NaiveDateTime,
    pub is_read: bool,
    pub is_deleted: bool,
    pub sender_name: String,
    pub recipient_name: String,
}

/// Mail history information for special mails (FBI, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MailHistory {
    pub mail_id: i32,
    pub info_date: Option<NaiveDateTime>,
    pub info1: Option<i32>,
}

/// Mail statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailStats {
    pub total_received: i32,
    pub unread_count: i32,
    pub total_sent: i32,
}

/// Mail types for system messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MailType {
    User = 0,
    FBI = 1,
    Safenet = 2,
    FBIWarning = 3,
    EvilCorp = -1,
    FBI2 = -2,
    Safenet2 = -3,
    SocialClan = -4,
    ClanNews = -5,
    Social = -6,
    BadgeAdvisor = -7,
}

/// Mail sender types for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MailSender {
    User(String),
    Unknown,
    NumatakaCorp,
    FBI,
    Safenet,
    SocialClan,
    ClanNews,
    Social,
    BadgeAdvisor,
}

#[async_trait]
pub trait MailRepository {
    /// Send a new mail
    async fn send_mail(&self, from_id: i32, to_id: i32, subject: &str, text: &str, mail_type: i32) -> Result<i32>;
    
    /// Get mail by ID
    async fn get_mail(&self, mail_id: i32) -> Result<Option<Mail>>;
    
    /// Get mail with sender/recipient info
    async fn get_mail_with_info(&self, mail_id: i32) -> Result<Option<MailWithInfo>>;
    
    /// Check if mail exists
    async fn mail_exists(&self, mail_id: i32) -> Result<bool>;
    
    /// Check if user can access mail
    async fn can_access_mail(&self, mail_id: i32, user_id: i32) -> Result<bool>;
    
    /// Mark mail as read
    async fn mark_as_read(&self, mail_id: i32) -> Result<()>;
    
    /// Mark mail as deleted
    async fn delete_mail(&self, mail_id: i32) -> Result<()>;
    
    /// Check if mail is deleted
    async fn is_deleted(&self, mail_id: i32) -> Result<bool>;
    
    /// Check if mail is unread
    async fn is_unread(&self, mail_id: i32) -> Result<bool>;
    
    /// Get mail sender ID
    async fn get_mail_from(&self, mail_id: i32) -> Result<Option<i32>>;
    
    /// Get mail recipient ID
    async fn get_mail_to(&self, mail_id: i32) -> Result<Option<i32>>;
    
    /// Get mail title/subject
    async fn get_mail_title(&self, mail_id: i32) -> Result<Option<String>>;
    
    /// Count user's mails
    async fn count_mails(&self, user_id: i32) -> Result<i32>;
    
    /// Count user's sent mails
    async fn count_sent_mails(&self, user_id: i32) -> Result<i32>;
    
    /// Count user's unread mails
    async fn count_unread_mails(&self, user_id: i32) -> Result<i32>;
    
    /// List user's received mails
    async fn list_received_mails(&self, user_id: i32, limit: i32, offset: i32) -> Result<Vec<MailWithInfo>>;
    
    /// List user's sent mails
    async fn list_sent_mails(&self, user_id: i32, limit: i32, offset: i32) -> Result<Vec<MailWithInfo>>;
    
    /// Get mail statistics
    async fn get_mail_stats(&self, user_id: i32) -> Result<MailStats>;
    
    /// Get mail history info (for FBI, bounty info, etc.)
    async fn get_mail_history(&self, mail_id: i32) -> Result<Option<MailHistory>>;
    
    /// Create mail history entry
    async fn create_mail_history(&self, mail_id: i32, info_date: Option<NaiveDateTime>, info1: Option<i32>) -> Result<()>;
    
    /// Send welcome mail to new user
    async fn send_welcome_mail(&self, user_id: i32, user_login: &str) -> Result<()>;
    
    /// Send system mail (FBI, SafeNet, etc.)
    async fn send_system_mail(&self, to_id: i32, subject: &str, text: &str, sender_type: MailType, history_info: Option<(NaiveDateTime, i32)>) -> Result<i32>;
    
    /// Get sender display name
    fn get_sender_name(&self, from_id: i32) -> Result<MailSender>;
    
    /// Reply to mail
    async fn reply_to_mail(&self, original_mail_id: i32, from_id: i32, subject: &str, text: &str) -> Result<i32>;
}

pub struct MailService {
    db: PgPool,
}

impl MailService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl MailRepository for MailService {
    /// Send a new mail
    async fn send_mail(&self, from_id: i32, to_id: i32, subject: &str, text: &str, mail_type: i32) -> Result<i32> {
        let mail_id = sqlx::query_scalar!(
            "INSERT INTO mails (from_id, to_id, mail_type, subject, text, date_sent, is_read, is_deleted)
             VALUES ($1, $2, $3, $4, $5, NOW(), false, false)
             RETURNING id",
            from_id,
            to_id,
            mail_type,
            subject,
            text
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(mail_id)
    }
    
    /// Get mail by ID
    async fn get_mail(&self, mail_id: i32) -> Result<Option<Mail>> {
        let mail = sqlx::query_as!(
            Mail,
            "SELECT id, from_id, to_id, mail_type, subject, text, date_sent, is_read, is_deleted
             FROM mails 
             WHERE id = $1",
            mail_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(mail)
    }
    
    /// Get mail with sender/recipient info
    async fn get_mail_with_info(&self, mail_id: i32) -> Result<Option<MailWithInfo>> {
        let mail = sqlx::query_as!(
            MailWithInfo,
            "SELECT m.id, m.from_id, m.to_id, m.mail_type, m.subject, m.text, m.date_sent, m.is_read, m.is_deleted,
                    COALESCE(u_from.login, 'System') as sender_name,
                    COALESCE(u_to.login, 'System') as recipient_name
             FROM mails m
             LEFT JOIN users u_from ON m.from_id = u_from.id AND m.from_id > 0
             LEFT JOIN users u_to ON m.to_id = u_to.id AND m.to_id > 0
             WHERE m.id = $1",
            mail_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(mail)
    }
    
    /// Check if mail exists
    async fn mail_exists(&self, mail_id: i32) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM mails WHERE id = $1",
            mail_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) > 0)
    }
    
    /// Check if user can access mail
    async fn can_access_mail(&self, mail_id: i32, user_id: i32) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM mails WHERE id = $1 AND (from_id = $2 OR to_id = $2)",
            mail_id,
            user_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) > 0)
    }
    
    /// Mark mail as read
    async fn mark_as_read(&self, mail_id: i32) -> Result<()> {
        sqlx::query!(
            "UPDATE mails SET is_read = true WHERE id = $1 AND is_read = false",
            mail_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Mark mail as deleted
    async fn delete_mail(&self, mail_id: i32) -> Result<()> {
        sqlx::query!(
            "UPDATE mails SET is_deleted = true WHERE id = $1",
            mail_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Check if mail is deleted
    async fn is_deleted(&self, mail_id: i32) -> Result<bool> {
        let is_deleted = sqlx::query_scalar!(
            "SELECT is_deleted FROM mails WHERE id = $1",
            mail_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(is_deleted.unwrap_or(true))
    }
    
    /// Check if mail is unread
    async fn is_unread(&self, mail_id: i32) -> Result<bool> {
        let is_read = sqlx::query_scalar!(
            "SELECT is_read FROM mails WHERE id = $1 AND is_deleted = false",
            mail_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(!is_read.unwrap_or(true))
    }
    
    /// Get mail sender ID
    async fn get_mail_from(&self, mail_id: i32) -> Result<Option<i32>> {
        let from_id = sqlx::query_scalar!(
            "SELECT from_id FROM mails WHERE id = $1",
            mail_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(from_id)
    }
    
    /// Get mail recipient ID
    async fn get_mail_to(&self, mail_id: i32) -> Result<Option<i32>> {
        let to_id = sqlx::query_scalar!(
            "SELECT to_id FROM mails WHERE id = $1",
            mail_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(to_id)
    }
    
    /// Get mail title/subject
    async fn get_mail_title(&self, mail_id: i32) -> Result<Option<String>> {
        let subject = sqlx::query_scalar!(
            "SELECT subject FROM mails WHERE id = $1 AND is_deleted = false",
            mail_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(subject)
    }
    
    /// Count user's mails
    async fn count_mails(&self, user_id: i32) -> Result<i32> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM mails WHERE to_id = $1 AND is_deleted = false",
            user_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) as i32)
    }
    
    /// Count user's sent mails
    async fn count_sent_mails(&self, user_id: i32) -> Result<i32> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM mails WHERE from_id = $1",
            user_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) as i32)
    }
    
    /// Count user's unread mails
    async fn count_unread_mails(&self, user_id: i32) -> Result<i32> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM mails WHERE to_id = $1 AND is_read = false AND is_deleted = false",
            user_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) as i32)
    }
    
    /// List user's received mails
    async fn list_received_mails(&self, user_id: i32, limit: i32, offset: i32) -> Result<Vec<MailWithInfo>> {
        let mails = sqlx::query_as!(
            MailWithInfo,
            "SELECT m.id, m.from_id, m.to_id, m.mail_type, m.subject, m.text, m.date_sent, m.is_read, m.is_deleted,
                    COALESCE(u_from.login, 'System') as sender_name,
                    COALESCE(u_to.login, 'System') as recipient_name
             FROM mails m
             LEFT JOIN users u_from ON m.from_id = u_from.id AND m.from_id > 0
             LEFT JOIN users u_to ON m.to_id = u_to.id AND m.to_id > 0
             WHERE m.to_id = $1 AND m.is_deleted = false
             ORDER BY m.date_sent DESC
             LIMIT $2 OFFSET $3",
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await?;
        
        Ok(mails)
    }
    
    /// List user's sent mails
    async fn list_sent_mails(&self, user_id: i32, limit: i32, offset: i32) -> Result<Vec<MailWithInfo>> {
        let mails = sqlx::query_as!(
            MailWithInfo,
            "SELECT m.id, m.from_id, m.to_id, m.mail_type, m.subject, m.text, m.date_sent, m.is_read, m.is_deleted,
                    COALESCE(u_from.login, 'System') as sender_name,
                    COALESCE(u_to.login, 'System') as recipient_name
             FROM mails m
             LEFT JOIN users u_from ON m.from_id = u_from.id AND m.from_id > 0
             LEFT JOIN users u_to ON m.to_id = u_to.id AND m.to_id > 0
             WHERE m.from_id = $1
             ORDER BY m.date_sent DESC
             LIMIT $2 OFFSET $3",
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await?;
        
        Ok(mails)
    }
    
    /// Get mail statistics
    async fn get_mail_stats(&self, user_id: i32) -> Result<MailStats> {
        let total_received = self.count_mails(user_id).await?;
        let unread_count = self.count_unread_mails(user_id).await?;
        let total_sent = self.count_sent_mails(user_id).await?;
        
        Ok(MailStats {
            total_received,
            unread_count,
            total_sent,
        })
    }
    
    /// Get mail history info (for FBI, bounty info, etc.)
    async fn get_mail_history(&self, mail_id: i32) -> Result<Option<MailHistory>> {
        let history = sqlx::query_as!(
            MailHistory,
            "SELECT mail_id, info_date, info1 FROM mails_history WHERE mail_id = $1 LIMIT 1",
            mail_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(history)
    }
    
    /// Create mail history entry
    async fn create_mail_history(&self, mail_id: i32, info_date: Option<NaiveDateTime>, info1: Option<i32>) -> Result<()> {
        sqlx::query!(
            "INSERT INTO mails_history (mail_id, info_date, info1) VALUES ($1, $2, $3)",
            mail_id,
            info_date,
            info1
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Send welcome mail to new user
    async fn send_welcome_mail(&self, user_id: i32, user_login: &str) -> Result<()> {
        let subject = "Welcome to Hacker Experience!";
        let text = format!(
            "Greetings, {}!\n\nThanks for trying out Hacker Experience, we are very excited to have you on board. \
             Congratulations on completing the tutorial. It wasn't that hard, right?\n\n\
             There are a lot more things to do in the game, if you get stuck don't hesitate in talking to us.\n\n\
             The fastest way of getting help is posting at our community board. We will happily guide you through the game. \
             You can also find a great resource information on our Wiki page.\n\n\
             Replying to this mail is another option. I'd be thrilled to talk to you :)\n\n\
             (By the way, I just sent you 1.337 BTC. You can see it on the Finances page. Enjoy!)\n\n\
             Pardon the pun, but we hope you have a great experience here!!\n\n\
             Happy hacking!\nRenato.",
            user_login
        );
        
        self.send_mail(1, user_id, subject, &text, 1).await?;
        
        Ok(())
    }
    
    /// Send system mail (FBI, SafeNet, etc.)
    async fn send_system_mail(&self, to_id: i32, subject: &str, text: &str, sender_type: MailType, history_info: Option<(NaiveDateTime, i32)>) -> Result<i32> {
        let mail_id = self.send_mail(sender_type as i32, to_id, subject, text, sender_type as i32).await?;
        
        // Create history entry if provided
        if let Some((date, info1)) = history_info {
            self.create_mail_history(mail_id, Some(date), Some(info1)).await?;
        }
        
        Ok(mail_id)
    }
    
    /// Get sender display name
    fn get_sender_name(&self, from_id: i32) -> Result<MailSender> {
        match from_id {
            id if id > 0 => {
                // Would need to fetch from database, simplified for now
                Ok(MailSender::User("Player".to_string()))
            }
            0 => Ok(MailSender::Unknown),
            -1 => Ok(MailSender::NumatakaCorp),
            -2 => Ok(MailSender::FBI),
            -3 => Ok(MailSender::Safenet),
            -4 => Ok(MailSender::SocialClan),
            -5 => Ok(MailSender::ClanNews),
            -6 => Ok(MailSender::Social),
            -7 => Ok(MailSender::BadgeAdvisor),
            _ => Ok(MailSender::Unknown),
        }
    }
    
    /// Reply to mail
    async fn reply_to_mail(&self, original_mail_id: i32, from_id: i32, subject: &str, text: &str) -> Result<i32> {
        let original_sender = self.get_mail_from(original_mail_id).await?;
        
        match original_sender {
            Some(to_id) if to_id > 0 => {
                let reply_subject = if subject.starts_with("Re: ") {
                    subject.to_string()
                } else {
                    format!("Re: {}", subject)
                };
                
                self.send_mail(from_id, to_id, &reply_subject, text, 0).await
            }
            _ => Err(crate::error::Error::InvalidOperation("Cannot reply to system message".to_string())),
        }
    }
}