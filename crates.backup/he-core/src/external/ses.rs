use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SesError {
    #[error("AWS SES API error: {0}")]
    ApiError(String),
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
    #[error("Invalid email address: {0}")]
    InvalidEmail(String),
    #[error("Message send failed: {0}")]
    SendFailed(String),
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),
    #[error("Template error: {0}")]
    TemplateError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Verification required: {0}")]
    VerificationRequired(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SesConfig {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub region: String,
    pub endpoint: Option<String>,
    pub use_ssl: bool,
    pub timeout: u32,
    pub max_send_rate: f64,
    pub daily_send_quota: u64,
    pub verified_domains: Vec<String>,
    pub verified_emails: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SesDestination {
    pub to_addresses: Vec<String>,
    pub cc_addresses: Vec<String>,
    pub bcc_addresses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SesMessage {
    pub subject: SesContent,
    pub body: SesBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SesContent {
    pub data: String,
    pub charset: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SesBody {
    pub text: Option<SesContent>,
    pub html: Option<SesContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SesTemplate {
    pub template_name: String,
    pub template_data: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SesRawMessage {
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SesMessageStatus {
    Sent,
    Bounce,
    Complaint,
    Delivery,
    Reject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SesSendResult {
    pub message_id: String,
    pub sent_at: DateTime<Utc>,
    pub status: SesMessageStatus,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SesQuotaInfo {
    pub max_24_hour_send: u64,
    pub max_send_rate: f64,
    pub sent_last_24_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SesVerificationStatus {
    pub identity: String,
    pub verification_status: String,
    pub verification_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SesBounceNotification {
    pub message_id: String,
    pub bounce_type: String,
    pub bounce_subtype: String,
    pub timestamp: DateTime<Utc>,
    pub bounced_recipients: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SesComplaintNotification {
    pub message_id: String,
    pub complaint_type: String,
    pub timestamp: DateTime<Utc>,
    pub complained_recipients: Vec<String>,
}

pub struct SES {
    config: SesConfig,
    authenticated: bool,
    send_count: u64,
    last_error: Option<String>,
    rate_limit_remaining: f64,
    quota_remaining: u64,
}

impl Default for SesConfig {
    fn default() -> Self {
        Self {
            access_key_id: String::new(),
            secret_access_key: String::new(),
            region: "us-east-1".to_string(),
            endpoint: None,
            use_ssl: true,
            timeout: 30,
            max_send_rate: 1.0,
            daily_send_quota: 200,
            verified_domains: Vec::new(),
            verified_emails: Vec::new(),
        }
    }
}

impl SES {
    pub fn new(config: SesConfig) -> Self {
        Self {
            config,
            authenticated: false,
            send_count: 0,
            last_error: None,
            rate_limit_remaining: 1.0,
            quota_remaining: 200,
        }
    }

    pub fn with_default() -> Self {
        Self::new(SesConfig::default())
    }

    pub fn authenticate(&mut self) -> Result<(), SesError> {
        if self.config.access_key_id.is_empty() || self.config.secret_access_key.is_empty() {
            return Err(SesError::AuthenticationError("Missing AWS credentials".to_string()));
        }

        // Simulate authentication
        self.authenticated = true;
        self.rate_limit_remaining = self.config.max_send_rate;
        self.quota_remaining = self.config.daily_send_quota;
        Ok(())
    }

    pub fn send_email(
        &mut self,
        source: &str,
        destination: &SesDestination,
        message: &SesMessage,
    ) -> Result<SesSendResult, SesError> {
        if !self.authenticated {
            self.authenticate()?;
        }

        self.check_rate_limit()?;
        self.check_quota()?;
        self.validate_source(source)?;
        self.validate_destinations(destination)?;

        // Simulate sending
        let message_id = format!("ses_{}", chrono::Utc::now().timestamp_nanos());
        let sent_at = chrono::Utc::now();

        self.send_count += 1;
        self.rate_limit_remaining -= 1.0;
        self.quota_remaining -= 1;

        Ok(SesSendResult {
            message_id,
            sent_at,
            status: SesMessageStatus::Sent,
            error_message: None,
        })
    }

    pub fn send_templated_email(
        &mut self,
        source: &str,
        destination: &SesDestination,
        template: &SesTemplate,
    ) -> Result<SesSendResult, SesError> {
        if !self.authenticated {
            self.authenticate()?;
        }

        self.check_rate_limit()?;
        self.check_quota()?;
        self.validate_source(source)?;
        self.validate_destinations(destination)?;

        if template.template_name.is_empty() {
            return Err(SesError::TemplateError("Template name cannot be empty".to_string()));
        }

        // Simulate template sending
        let message_id = format!("ses_template_{}", chrono::Utc::now().timestamp_nanos());
        let sent_at = chrono::Utc::now();

        self.send_count += 1;
        self.rate_limit_remaining -= 1.0;
        self.quota_remaining -= 1;

        Ok(SesSendResult {
            message_id,
            sent_at,
            status: SesMessageStatus::Sent,
            error_message: None,
        })
    }

    pub fn send_raw_email(
        &mut self,
        source: &str,
        destinations: &[String],
        raw_message: &SesRawMessage,
    ) -> Result<SesSendResult, SesError> {
        if !self.authenticated {
            self.authenticate()?;
        }

        self.check_rate_limit()?;
        self.check_quota()?;
        self.validate_source(source)?;

        if raw_message.data.is_empty() {
            return Err(SesError::SendFailed("Raw message data cannot be empty".to_string()));
        }

        // Simulate raw sending
        let message_id = format!("ses_raw_{}", chrono::Utc::now().timestamp_nanos());
        let sent_at = chrono::Utc::now();

        self.send_count += 1;
        self.rate_limit_remaining -= 1.0;
        self.quota_remaining -= 1;

        Ok(SesSendResult {
            message_id,
            sent_at,
            status: SesMessageStatus::Sent,
            error_message: None,
        })
    }

    pub fn get_send_quota(&self) -> Result<SesQuotaInfo, SesError> {
        if !self.authenticated {
            return Err(SesError::AuthenticationError("Not authenticated".to_string()));
        }

        Ok(SesQuotaInfo {
            max_24_hour_send: self.config.daily_send_quota,
            max_send_rate: self.config.max_send_rate,
            sent_last_24_hours: self.send_count,
        })
    }

    pub fn verify_email_identity(&mut self, email: &str) -> Result<SesVerificationStatus, SesError> {
        if !self.authenticated {
            self.authenticate()?;
        }

        if !self.is_valid_email(email) {
            return Err(SesError::InvalidEmail(email.to_string()));
        }

        // Simulate verification process
        let verification_token = format!("token_{}", chrono::Utc::now().timestamp());
        
        Ok(SesVerificationStatus {
            identity: email.to_string(),
            verification_status: "Pending".to_string(),
            verification_token: Some(verification_token),
        })
    }

    pub fn verify_domain_identity(&mut self, domain: &str) -> Result<SesVerificationStatus, SesError> {
        if !self.authenticated {
            self.authenticate()?;
        }

        if domain.is_empty() || !domain.contains('.') {
            return Err(SesError::ConfigError("Invalid domain format".to_string()));
        }

        // Simulate domain verification
        let verification_token = format!("domain_token_{}", chrono::Utc::now().timestamp());
        
        Ok(SesVerificationStatus {
            identity: domain.to_string(),
            verification_status: "Pending".to_string(),
            verification_token: Some(verification_token),
        })
    }

    pub fn get_identity_verification_attributes(&self, identities: &[String]) -> Result<HashMap<String, SesVerificationStatus>, SesError> {
        if !self.authenticated {
            return Err(SesError::AuthenticationError("Not authenticated".to_string()));
        }

        let mut results = HashMap::new();
        
        for identity in identities {
            let status = if self.config.verified_emails.contains(identity) || 
                           self.config.verified_domains.iter().any(|d| identity.ends_with(d)) {
                "Success"
            } else {
                "Pending"
            };

            results.insert(identity.clone(), SesVerificationStatus {
                identity: identity.clone(),
                verification_status: status.to_string(),
                verification_token: None,
            });
        }

        Ok(results)
    }

    pub fn put_configuration_set(&mut self, configuration_set_name: &str) -> Result<(), SesError> {
        if !self.authenticated {
            self.authenticate()?;
        }

        if configuration_set_name.is_empty() {
            return Err(SesError::ConfigError("Configuration set name cannot be empty".to_string()));
        }

        // Simulate configuration set creation
        Ok(())
    }

    pub fn create_template(&mut self, template_name: &str, subject: &str, html_part: &str, text_part: Option<&str>) -> Result<(), SesError> {
        if !self.authenticated {
            self.authenticate()?;
        }

        if template_name.is_empty() {
            return Err(SesError::TemplateError("Template name cannot be empty".to_string()));
        }

        if subject.is_empty() {
            return Err(SesError::TemplateError("Subject cannot be empty".to_string()));
        }

        // Simulate template creation
        Ok(())
    }

    pub fn update_template(&mut self, template_name: &str, subject: &str, html_part: &str, text_part: Option<&str>) -> Result<(), SesError> {
        if !self.authenticated {
            self.authenticate()?;
        }

        if template_name.is_empty() {
            return Err(SesError::TemplateError("Template name cannot be empty".to_string()));
        }

        // Simulate template update
        Ok(())
    }

    pub fn delete_template(&mut self, template_name: &str) -> Result<(), SesError> {
        if !self.authenticated {
            self.authenticate()?;
        }

        if template_name.is_empty() {
            return Err(SesError::TemplateError("Template name cannot be empty".to_string()));
        }

        // Simulate template deletion
        Ok(())
    }

    pub fn get_send_statistics(&self) -> Result<HashMap<String, u64>, SesError> {
        if !self.authenticated {
            return Err(SesError::AuthenticationError("Not authenticated".to_string()));
        }

        let mut stats = HashMap::new();
        stats.insert("Send".to_string(), self.send_count);
        stats.insert("Bounce".to_string(), 0);
        stats.insert("Complaint".to_string(), 0);
        stats.insert("Delivery".to_string(), self.send_count);
        stats.insert("Reject".to_string(), 0);

        Ok(stats)
    }

    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }

    pub fn get_last_error(&self) -> Option<&String> {
        self.last_error.as_ref()
    }

    pub fn get_send_count(&self) -> u64 {
        self.send_count
    }

    pub fn get_rate_limit_remaining(&self) -> f64 {
        self.rate_limit_remaining
    }

    pub fn get_quota_remaining(&self) -> u64 {
        self.quota_remaining
    }

    // Private helper methods
    fn check_rate_limit(&self) -> Result<(), SesError> {
        if self.rate_limit_remaining <= 0.0 {
            return Err(SesError::RateLimitExceeded("Send rate limit exceeded".to_string()));
        }
        Ok(())
    }

    fn check_quota(&self) -> Result<(), SesError> {
        if self.quota_remaining == 0 {
            return Err(SesError::QuotaExceeded("Daily send quota exceeded".to_string()));
        }
        Ok(())
    }

    fn validate_source(&self, source: &str) -> Result<(), SesError> {
        if !self.is_valid_email(source) {
            return Err(SesError::InvalidEmail(format!("Invalid source email: {}", source)));
        }

        // Check if source is verified
        if !self.config.verified_emails.contains(&source.to_string()) {
            let domain = source.split('@').nth(1).unwrap_or("");
            if !self.config.verified_domains.contains(&domain.to_string()) {
                return Err(SesError::VerificationRequired(format!("Source email or domain not verified: {}", source)));
            }
        }

        Ok(())
    }

    fn validate_destinations(&self, destination: &SesDestination) -> Result<(), SesError> {
        if destination.to_addresses.is_empty() && destination.cc_addresses.is_empty() && destination.bcc_addresses.is_empty() {
            return Err(SesError::SendFailed("No destination addresses specified".to_string()));
        }

        for email in &destination.to_addresses {
            if !self.is_valid_email(email) {
                return Err(SesError::InvalidEmail(email.clone()));
            }
        }

        for email in &destination.cc_addresses {
            if !self.is_valid_email(email) {
                return Err(SesError::InvalidEmail(email.clone()));
            }
        }

        for email in &destination.bcc_addresses {
            if !self.is_valid_email(email) {
                return Err(SesError::InvalidEmail(email.clone()));
            }
        }

        Ok(())
    }

    fn is_valid_email(&self, email: &str) -> bool {
        // Basic email validation
        email.contains('@') && email.len() > 3 && !email.starts_with('@') && !email.ends_with('@')
    }
}

// Helper functions for creating SES structures
pub fn create_destination(to: Vec<String>, cc: Option<Vec<String>>, bcc: Option<Vec<String>>) -> SesDestination {
    SesDestination {
        to_addresses: to,
        cc_addresses: cc.unwrap_or_default(),
        bcc_addresses: bcc.unwrap_or_default(),
    }
}

pub fn create_content(data: &str, charset: Option<&str>) -> SesContent {
    SesContent {
        data: data.to_string(),
        charset: charset.unwrap_or("UTF-8").to_string(),
    }
}

pub fn create_message(subject: &str, html_body: Option<&str>, text_body: Option<&str>) -> SesMessage {
    SesMessage {
        subject: create_content(subject, None),
        body: SesBody {
            html: html_body.map(|body| create_content(body, None)),
            text: text_body.map(|body| create_content(body, None)),
        },
    }
}

pub fn create_template(template_name: &str, template_data: HashMap<String, String>) -> SesTemplate {
    SesTemplate {
        template_name: template_name.to_string(),
        template_data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let ses = SES::with_default();
        
        assert!(ses.is_valid_email("test@example.com"));
        assert!(ses.is_valid_email("user.name@domain.co.uk"));
        assert!(!ses.is_valid_email("invalid"));
        assert!(!ses.is_valid_email("@example.com"));
        assert!(!ses.is_valid_email("test@"));
    }

    #[test]
    fn test_create_destination() {
        let dest = create_destination(
            vec!["to@example.com".to_string()],
            Some(vec!["cc@example.com".to_string()]),
            None
        );
        
        assert_eq!(dest.to_addresses.len(), 1);
        assert_eq!(dest.cc_addresses.len(), 1);
        assert_eq!(dest.bcc_addresses.len(), 0);
    }

    #[test]
    fn test_create_message() {
        let message = create_message(
            "Test Subject",
            Some("<h1>HTML Body</h1>"),
            Some("Text Body")
        );
        
        assert_eq!(message.subject.data, "Test Subject");
        assert!(message.body.html.is_some());
        assert!(message.body.text.is_some());
    }

    #[test]
    fn test_authentication_required() {
        let mut ses = SES::with_default();
        let dest = create_destination(vec!["test@example.com".to_string()], None, None);
        let message = create_message("Test", Some("Body"), None);
        
        let result = ses.send_email("from@example.com", &dest, &message);
        assert!(result.is_err());
    }
}