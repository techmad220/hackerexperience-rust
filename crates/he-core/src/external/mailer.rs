use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MailerError {
    #[error("SMTP connection failed: {0}")]
    SmtpConnectionFailed(String),
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Invalid email address: {0}")]
    InvalidEmail(String),
    #[error("Message send failed: {0}")]
    SendFailed(String),
    #[error("Attachment error: {0}")]
    AttachmentError(String),
    #[error("Template error: {0}")]
    TemplateError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailerConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub use_tls: bool,
    pub use_ssl: bool,
    pub from_email: String,
    pub from_name: String,
    pub reply_to: Option<String>,
    pub charset: String,
    pub timeout: u32,
    pub debug_level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAddress {
    pub email: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAttachment {
    pub filename: String,
    pub content: Vec<u8>,
    pub mime_type: String,
    pub disposition: AttachmentDisposition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttachmentDisposition {
    Attachment,
    Inline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailPriority {
    Low = 5,
    Normal = 3,
    High = 1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailFormat {
    Text,
    Html,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub subject: String,
    pub body: String,
    pub alt_body: Option<String>,
    pub format: EmailFormat,
    pub priority: EmailPriority,
    pub to: Vec<EmailAddress>,
    pub cc: Vec<EmailAddress>,
    pub bcc: Vec<EmailAddress>,
    pub attachments: Vec<EmailAttachment>,
    pub headers: HashMap<String, String>,
    pub message_id: Option<String>,
    pub date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Pending,
    Sent,
    Failed,
    Bounced,
    Delivered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryResult {
    pub status: DeliveryStatus,
    pub message_id: String,
    pub sent_at: DateTime<Utc>,
    pub error_message: Option<String>,
    pub recipients_accepted: Vec<String>,
    pub recipients_rejected: Vec<String>,
}

pub struct PHPMailer {
    config: MailerConfig,
    connected: bool,
    last_error: Option<String>,
    debug_output: Vec<String>,
    sent_count: u32,
    failed_count: u32,
}

impl Default for MailerConfig {
    fn default() -> Self {
        Self {
            smtp_host: "localhost".to_string(),
            smtp_port: 587,
            smtp_username: String::new(),
            smtp_password: String::new(),
            use_tls: true,
            use_ssl: false,
            from_email: "noreply@example.com".to_string(),
            from_name: "System".to_string(),
            reply_to: None,
            charset: "UTF-8".to_string(),
            timeout: 30,
            debug_level: 0,
        }
    }
}

impl Default for EmailMessage {
    fn default() -> Self {
        Self {
            subject: String::new(),
            body: String::new(),
            alt_body: None,
            format: EmailFormat::Html,
            priority: EmailPriority::Normal,
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            attachments: Vec::new(),
            headers: HashMap::new(),
            message_id: None,
            date: None,
        }
    }
}

impl PHPMailer {
    pub fn new(config: MailerConfig) -> Self {
        Self {
            config,
            connected: false,
            last_error: None,
            debug_output: Vec::new(),
            sent_count: 0,
            failed_count: 0,
        }
    }

    pub fn with_default() -> Self {
        Self::new(MailerConfig::default())
    }

    pub fn connect(&mut self) -> Result<(), MailerError> {
        if self.connected {
            return Ok(());
        }

        // Simulate SMTP connection
        if self.config.smtp_host.is_empty() {
            return Err(MailerError::ConfigError("SMTP host not configured".to_string()));
        }

        if self.config.smtp_username.is_empty() || self.config.smtp_password.is_empty() {
            return Err(MailerError::AuthenticationFailed("Missing credentials".to_string()));
        }

        self.connected = true;
        self.add_debug("Connected to SMTP server");
        Ok(())
    }

    pub fn disconnect(&mut self) {
        self.connected = false;
        self.add_debug("Disconnected from SMTP server");
    }

    pub fn send_message(&mut self, message: &EmailMessage) -> Result<DeliveryResult, MailerError> {
        if !self.connected {
            self.connect()?;
        }

        self.validate_message(message)?;

        // Simulate sending
        let message_id = format!("msg_{}", chrono::Utc::now().timestamp());
        let sent_at = chrono::Utc::now();

        // Check for delivery simulation
        let mut recipients_accepted = Vec::new();
        let mut recipients_rejected = Vec::new();

        for addr in &message.to {
            if self.is_valid_email(&addr.email) {
                recipients_accepted.push(addr.email.clone());
            } else {
                recipients_rejected.push(addr.email.clone());
            }
        }

        let status = if recipients_rejected.is_empty() {
            DeliveryStatus::Sent
        } else if recipients_accepted.is_empty() {
            DeliveryStatus::Failed
        } else {
            DeliveryStatus::Sent // Partial success
        };

        if status == DeliveryStatus::Sent {
            self.sent_count += 1;
        } else {
            self.failed_count += 1;
        }

        self.add_debug(&format!("Message sent with ID: {}", message_id));

        Ok(DeliveryResult {
            status,
            message_id,
            sent_at,
            error_message: None,
            recipients_accepted,
            recipients_rejected,
        })
    }

    pub fn send_simple_email(
        &mut self,
        to: &str,
        subject: &str,
        body: &str,
        is_html: bool,
    ) -> Result<DeliveryResult, MailerError> {
        let mut message = EmailMessage::default();
        message.subject = subject.to_string();
        message.body = body.to_string();
        message.format = if is_html { EmailFormat::Html } else { EmailFormat::Text };
        message.to.push(EmailAddress {
            email: to.to_string(),
            name: None,
        });

        self.send_message(&message)
    }

    pub fn add_attachment(
        &self,
        message: &mut EmailMessage,
        filename: &str,
        content: Vec<u8>,
        mime_type: &str,
    ) -> Result<(), MailerError> {
        if filename.is_empty() {
            return Err(MailerError::AttachmentError("Filename cannot be empty".to_string()));
        }

        message.attachments.push(EmailAttachment {
            filename: filename.to_string(),
            content,
            mime_type: mime_type.to_string(),
            disposition: AttachmentDisposition::Attachment,
        });

        Ok(())
    }

    pub fn set_template_vars(&self, template: &str, vars: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        for (key, value) in vars {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }

    pub fn validate_email(&self, email: &str) -> bool {
        self.is_valid_email(email)
    }

    pub fn get_stats(&self) -> (u32, u32) {
        (self.sent_count, self.failed_count)
    }

    pub fn get_last_error(&self) -> Option<&String> {
        self.last_error.as_ref()
    }

    pub fn get_debug_output(&self) -> &Vec<String> {
        &self.debug_output
    }

    pub fn clear_debug(&mut self) {
        self.debug_output.clear();
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    // Private helper methods
    fn validate_message(&self, message: &EmailMessage) -> Result<(), MailerError> {
        if message.subject.is_empty() {
            return Err(MailerError::SendFailed("Subject cannot be empty".to_string()));
        }

        if message.body.is_empty() {
            return Err(MailerError::SendFailed("Body cannot be empty".to_string()));
        }

        if message.to.is_empty() {
            return Err(MailerError::SendFailed("No recipients specified".to_string()));
        }

        for addr in &message.to {
            if !self.is_valid_email(&addr.email) {
                return Err(MailerError::InvalidEmail(addr.email.clone()));
            }
        }

        Ok(())
    }

    fn is_valid_email(&self, email: &str) -> bool {
        // Basic email validation
        email.contains('@') && email.len() > 3 && !email.starts_with('@') && !email.ends_with('@')
    }

    fn add_debug(&mut self, message: &str) {
        if self.config.debug_level > 0 {
            self.debug_output.push(format!("[{}] {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), message));
        }
    }
}

impl Drop for PHPMailer {
    fn drop(&mut self) {
        if self.connected {
            self.disconnect();
        }
    }
}

// Helper functions for common email operations
pub fn create_email_address(email: &str, name: Option<&str>) -> EmailAddress {
    EmailAddress {
        email: email.to_string(),
        name: name.map(|n| n.to_string()),
    }
}

pub fn create_html_email(
    to: &str,
    subject: &str,
    html_body: &str,
    text_body: Option<&str>,
) -> EmailMessage {
    let mut message = EmailMessage::default();
    message.subject = subject.to_string();
    message.body = html_body.to_string();
    message.alt_body = text_body.map(|s| s.to_string());
    message.format = EmailFormat::Html;
    message.to.push(create_email_address(to, None));
    message
}

pub fn create_text_email(to: &str, subject: &str, body: &str) -> EmailMessage {
    let mut message = EmailMessage::default();
    message.subject = subject.to_string();
    message.body = body.to_string();
    message.format = EmailFormat::Text;
    message.to.push(create_email_address(to, None));
    message
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let mailer = PHPMailer::with_default();
        
        assert!(mailer.validate_email("test@example.com"));
        assert!(mailer.validate_email("user.name@domain.co.uk"));
        assert!(!mailer.validate_email("invalid"));
        assert!(!mailer.validate_email("@example.com"));
        assert!(!mailer.validate_email("test@"));
    }

    #[test]
    fn test_template_vars() {
        let mailer = PHPMailer::with_default();
        let template = "Hello {{name}}, your code is {{code}}";
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "John".to_string());
        vars.insert("code".to_string(), "12345".to_string());
        
        let result = mailer.set_template_vars(template, &vars);
        assert_eq!(result, "Hello John, your code is 12345");
    }

    #[test]
    fn test_create_simple_message() {
        let message = create_text_email("test@example.com", "Test Subject", "Test Body");
        
        assert_eq!(message.subject, "Test Subject");
        assert_eq!(message.body, "Test Body");
        assert_eq!(message.to.len(), 1);
        assert_eq!(message.to[0].email, "test@example.com");
        assert!(matches!(message.format, EmailFormat::Text));
    }
}