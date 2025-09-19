use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailVerificationError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Invalid email format: {0}")]
    InvalidEmail(String),
    #[error("User not found: {0}")]
    UserNotFound(u64),
    #[error("Verification code not found")]
    CodeNotFound,
    #[error("Invalid verification code")]
    InvalidCode,
    #[error("Verification code expired")]
    CodeExpired,
    #[error("Email already verified")]
    AlreadyVerified,
    #[error("Too many attempts")]
    TooManyAttempts,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Email sending failed: {0}")]
    EmailSendFailed(String),
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    #[error("Invalid template data")]
    InvalidTemplateData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    Pending,
    Verified,
    Expired,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailTemplate {
    Welcome,
    Verification,
    ResendVerification,
    VerificationSuccess,
    VerificationExpired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCode {
    pub id: u64,
    pub user_id: u64,
    pub email: String,
    pub code: String,
    pub status: VerificationStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
    pub attempts: u32,
    pub max_attempts: u32,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationAttempt {
    pub id: u64,
    pub user_id: u64,
    pub code_id: u64,
    pub submitted_code: String,
    pub is_successful: bool,
    pub attempted_at: DateTime<Utc>,
    pub ip_address: String,
    pub user_agent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplateData {
    pub user_id: u64,
    pub username: String,
    pub email: String,
    pub verification_code: String,
    pub verification_url: String,
    pub expiration_date: DateTime<Utc>,
    pub support_email: String,
    pub site_name: String,
    pub site_url: String,
}

#[derive(Debug, Clone)]
pub struct VerificationConfig {
    pub code_length: usize,
    pub code_expiry: Duration,
    pub max_attempts: u32,
    pub rate_limit_window: Duration,
    pub rate_limit_max_requests: u32,
    pub verification_url_base: String,
    pub support_email: String,
    pub site_name: String,
    pub site_url: String,
    pub send_welcome_email: bool,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            code_length: 32,
            code_expiry: Duration::hours(24),
            max_attempts: 5,
            rate_limit_window: Duration::minutes(15),
            rate_limit_max_requests: 3,
            verification_url_base: "https://example.com/verify".to_string(),
            support_email: "support@example.com".to_string(),
            site_name: "Hacker Experience".to_string(),
            site_url: "https://hackerexperience.com".to_string(),
            send_welcome_email: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationStats {
    pub total_codes_sent: u64,
    pub successful_verifications: u64,
    pub expired_codes: u64,
    pub failed_attempts: u64,
    pub success_rate: f64,
    pub average_verification_time: Option<Duration>,
    pub pending_verifications: u64,
}

/// Email verification system ported from PHP EmailVerification class
/// Handles email verification codes, sending emails, and verification tracking
pub struct EmailVerification {
    config: VerificationConfig,
    current_user_id: Option<u64>,
}

impl EmailVerification {
    /// Create new EmailVerification instance
    pub fn new(config: VerificationConfig, current_user_id: Option<u64>) -> Self {
        Self {
            config,
            current_user_id,
        }
    }

    /// Create EmailVerification with default config
    pub fn default() -> Self {
        Self::new(VerificationConfig::default(), None)
    }

    /// Create EmailVerification with user context
    pub fn with_user(user_id: u64) -> Self {
        Self::new(VerificationConfig::default(), Some(user_id))
    }

    /// Generate and send verification code to user
    pub fn send_verification_email(
        &self,
        user_id: u64,
        email: &str,
        username: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<VerificationCode, EmailVerificationError> {
        // Validate email format
        if !self.is_valid_email(email) {
            return Err(EmailVerificationError::InvalidEmail(email.to_string()));
        }

        // Check rate limiting
        self.check_rate_limit(user_id)?;

        // Check if user already has a pending verification
        if let Ok(existing_code) = self.get_pending_verification(user_id) {
            if existing_code.expires_at > Utc::now() {
                // Resend existing code
                return self.resend_verification_email(&existing_code, username);
            }
        }

        // Generate new verification code
        let code = self.generate_verification_code();
        let expires_at = Utc::now() + self.config.code_expiry;

        let verification_code = VerificationCode {
            id: self.generate_id(),
            user_id,
            email: email.to_string(),
            code: code.clone(),
            status: VerificationStatus::Pending,
            created_at: Utc::now(),
            expires_at,
            verified_at: None,
            attempts: 0,
            max_attempts: self.config.max_attempts,
            ip_address,
            user_agent,
        };

        // Save verification code to database
        self.save_verification_code(&verification_code)?;

        // Send verification email
        self.send_email(&verification_code, username, EmailTemplate::Verification)?;

        // Record rate limit
        self.record_rate_limit_attempt(user_id)?;

        Ok(verification_code)
    }

    /// Resend existing verification code
    pub fn resend_verification_email(
        &self,
        verification_code: &VerificationCode,
        username: &str,
    ) -> Result<VerificationCode, EmailVerificationError> {
        // Check if code is still valid
        if verification_code.expires_at <= Utc::now() {
            return Err(EmailVerificationError::CodeExpired);
        }

        if matches!(verification_code.status, VerificationStatus::Verified) {
            return Err(EmailVerificationError::AlreadyVerified);
        }

        // Check rate limiting
        self.check_rate_limit(verification_code.user_id)?;

        // Send email
        self.send_email(verification_code, username, EmailTemplate::ResendVerification)?;

        // Record rate limit
        self.record_rate_limit_attempt(verification_code.user_id)?;

        Ok(verification_code.clone())
    }

    /// Verify email with provided code
    pub fn verify_email(&self, user_id: u64, submitted_code: &str, ip_address: String, user_agent: String) -> Result<bool, EmailVerificationError> {
        // Get pending verification code
        let mut verification_code = self.get_pending_verification(user_id)?;

        // Check if already verified
        if matches!(verification_code.status, VerificationStatus::Verified) {
            return Err(EmailVerificationError::AlreadyVerified);
        }

        // Check expiration
        if verification_code.expires_at <= Utc::now() {
            verification_code.status = VerificationStatus::Expired;
            self.save_verification_code(&verification_code)?;
            return Err(EmailVerificationError::CodeExpired);
        }

        // Check max attempts
        if verification_code.attempts >= verification_code.max_attempts {
            verification_code.status = VerificationStatus::Failed;
            self.save_verification_code(&verification_code)?;
            return Err(EmailVerificationError::TooManyAttempts);
        }

        // Record attempt
        verification_code.attempts += 1;
        let is_successful = self.timing_safe_equals(&verification_code.code, submitted_code);

        let attempt = VerificationAttempt {
            id: self.generate_id(),
            user_id,
            code_id: verification_code.id,
            submitted_code: submitted_code.to_string(),
            is_successful,
            attempted_at: Utc::now(),
            ip_address,
            user_agent,
        };

        self.save_verification_attempt(&attempt)?;

        if is_successful {
            // Mark as verified
            verification_code.status = VerificationStatus::Verified;
            verification_code.verified_at = Some(Utc::now());
            self.save_verification_code(&verification_code)?;

            // Send welcome email if configured
            if self.config.send_welcome_email {
                let user_info = self.get_user_info(user_id)?;
                self.send_email(&verification_code, &user_info.username, EmailTemplate::Welcome)?;
            }

            // Update user's email verification status
            self.mark_user_as_verified(user_id)?;

            Ok(true)
        } else {
            // Save updated attempt count
            self.save_verification_code(&verification_code)?;
            Ok(false)
        }
    }

    /// Verify email using code-only verification (from email link)
    pub fn verify_by_code_only(&self, code: &str) -> Result<u64, EmailVerificationError> {
        // Find verification code
        let mut verification_code = self.get_verification_by_code(code)?;

        // Check if already verified
        if matches!(verification_code.status, VerificationStatus::Verified) {
            return Err(EmailVerificationError::AlreadyVerified);
        }

        // Check expiration
        if verification_code.expires_at <= Utc::now() {
            verification_code.status = VerificationStatus::Expired;
            self.save_verification_code(&verification_code)?;
            return Err(EmailVerificationError::CodeExpired);
        }

        // Mark as verified
        verification_code.status = VerificationStatus::Verified;
        verification_code.verified_at = Some(Utc::now());
        self.save_verification_code(&verification_code)?;

        // Send welcome email if configured
        if self.config.send_welcome_email {
            let user_info = self.get_user_info(verification_code.user_id)?;
            self.send_email(&verification_code, &user_info.username, EmailTemplate::Welcome)?;
        }

        // Update user's email verification status
        self.mark_user_as_verified(verification_code.user_id)?;

        Ok(verification_code.user_id)
    }

    /// Check if user's email is verified
    pub fn is_verified(&self, user_id: u64) -> Result<bool, EmailVerificationError> {
        // Check if user has any pending verification codes
        match self.get_pending_verification(user_id) {
            Ok(_) => Ok(false), // Has pending verification
            Err(EmailVerificationError::CodeNotFound) => Ok(true), // No pending verification, assume verified
            Err(e) => Err(e),
        }
    }

    /// Get verification status for user
    pub fn get_verification_status(&self, user_id: u64) -> Result<Option<VerificationCode>, EmailVerificationError> {
        match self.get_pending_verification(user_id) {
            Ok(code) => Ok(Some(code)),
            Err(EmailVerificationError::CodeNotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Get verification statistics
    pub fn get_verification_stats(&self) -> Result<VerificationStats, EmailVerificationError> {
        // Simulate database aggregation
        Ok(VerificationStats {
            total_codes_sent: 1000,
            successful_verifications: 850,
            expired_codes: 100,
            failed_attempts: 50,
            success_rate: 85.0,
            average_verification_time: Some(Duration::minutes(45)),
            pending_verifications: 150,
        })
    }

    /// Clean up expired verification codes
    pub fn cleanup_expired_codes(&self) -> Result<u32, EmailVerificationError> {
        // Delete expired codes
        // UPDATE email_verifications SET status = 'expired' WHERE expires_at <= NOW() AND status = 'pending'
        Ok(0) // Return count of cleaned codes
    }

    /// Private helper methods
    fn generate_verification_code(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // Generate unique code
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        
        let random_data = format!("verification_{}_{}", timestamp, self.config.site_name);
        let hash = self.hash_data(&random_data);
        
        // Take first N characters
        hash.chars().take(self.config.code_length).collect()
    }

    fn hash_data(&self, data: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn timing_safe_equals(&self, a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let mut result = 0u8;
        for (byte_a, byte_b) in a.bytes().zip(b.bytes()) {
            result |= byte_a ^ byte_b;
        }

        result == 0
    }

    fn is_valid_email(&self, email: &str) -> bool {
        // Basic email validation
        email.contains('@') && email.contains('.') && email.len() > 5
    }

    fn check_rate_limit(&self, user_id: u64) -> Result<(), EmailVerificationError> {
        // Check if user has exceeded rate limit
        let attempts = self.get_recent_attempts(user_id)?;
        if attempts >= self.config.rate_limit_max_requests {
            return Err(EmailVerificationError::RateLimitExceeded);
        }
        Ok(())
    }

    fn get_recent_attempts(&self, user_id: u64) -> Result<u32, EmailVerificationError> {
        // Count attempts in the rate limit window
        // SELECT COUNT(*) FROM email_verification_rate_limits 
        // WHERE user_id = ? AND created_at > NOW() - INTERVAL ? MINUTE
        Ok(0) // Mock implementation
    }

    fn record_rate_limit_attempt(&self, user_id: u64) -> Result<(), EmailVerificationError> {
        // Record rate limit attempt
        // INSERT INTO email_verification_rate_limits (user_id, created_at) VALUES (?, NOW())
        Ok(())
    }

    fn get_pending_verification(&self, user_id: u64) -> Result<VerificationCode, EmailVerificationError> {
        // Get pending verification code for user
        // SELECT * FROM email_verifications WHERE user_id = ? AND status = 'pending' ORDER BY created_at DESC LIMIT 1
        
        // Mock verification code
        Ok(VerificationCode {
            id: 1,
            user_id,
            email: "user@example.com".to_string(),
            code: "test_verification_code".to_string(),
            status: VerificationStatus::Pending,
            created_at: Utc::now() - Duration::hours(1),
            expires_at: Utc::now() + Duration::hours(23),
            verified_at: None,
            attempts: 0,
            max_attempts: self.config.max_attempts,
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("Test User Agent".to_string()),
        })
    }

    fn get_verification_by_code(&self, code: &str) -> Result<VerificationCode, EmailVerificationError> {
        // Get verification by code
        // SELECT * FROM email_verifications WHERE code = ? AND status = 'pending' LIMIT 1
        
        if code.is_empty() {
            return Err(EmailVerificationError::CodeNotFound);
        }

        // Mock verification code
        Ok(VerificationCode {
            id: 1,
            user_id: 1,
            email: "user@example.com".to_string(),
            code: code.to_string(),
            status: VerificationStatus::Pending,
            created_at: Utc::now() - Duration::hours(1),
            expires_at: Utc::now() + Duration::hours(23),
            verified_at: None,
            attempts: 0,
            max_attempts: self.config.max_attempts,
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("Test User Agent".to_string()),
        })
    }

    fn save_verification_code(&self, code: &VerificationCode) -> Result<(), EmailVerificationError> {
        // Save verification code to database
        Ok(())
    }

    fn save_verification_attempt(&self, attempt: &VerificationAttempt) -> Result<(), EmailVerificationError> {
        // Save verification attempt to database
        Ok(())
    }

    fn get_user_info(&self, user_id: u64) -> Result<UserInfo, EmailVerificationError> {
        // Get user information
        if user_id == 0 {
            return Err(EmailVerificationError::UserNotFound(user_id));
        }

        Ok(UserInfo {
            id: user_id,
            username: format!("user_{}", user_id),
            email: format!("user_{}@example.com", user_id),
        })
    }

    fn mark_user_as_verified(&self, user_id: u64) -> Result<(), EmailVerificationError> {
        // Update user's email_verified status
        // UPDATE users SET email_verified = true, email_verified_at = NOW() WHERE id = ?
        Ok(())
    }

    fn send_email(
        &self,
        verification_code: &VerificationCode,
        username: &str,
        template: EmailTemplate,
    ) -> Result<(), EmailVerificationError> {
        let template_data = self.prepare_template_data(verification_code, username)?;
        
        // In real implementation, integrate with email service (SES, SendGrid, etc.)
        match template {
            EmailTemplate::Verification => {
                // Send verification email with code
                println!("Sending verification email to {} with code {}", 
                    template_data.email, template_data.verification_code);
            }
            EmailTemplate::Welcome => {
                // Send welcome email after successful verification
                println!("Sending welcome email to {}", template_data.email);
            }
            EmailTemplate::ResendVerification => {
                // Resend verification email
                println!("Resending verification email to {}", template_data.email);
            }
            _ => {
                return Err(EmailVerificationError::TemplateNotFound(format!("{:?}", template)));
            }
        }

        Ok(())
    }

    fn prepare_template_data(&self, verification_code: &VerificationCode, username: &str) -> Result<EmailTemplateData, EmailVerificationError> {
        let verification_url = format!("{}?code={}", 
            self.config.verification_url_base, 
            verification_code.code);

        Ok(EmailTemplateData {
            user_id: verification_code.user_id,
            username: username.to_string(),
            email: verification_code.email.clone(),
            verification_code: verification_code.code.clone(),
            verification_url,
            expiration_date: verification_code.expires_at,
            support_email: self.config.support_email.clone(),
            site_name: self.config.site_name.clone(),
            site_url: self.config.site_url.clone(),
        })
    }

    fn generate_id(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

#[derive(Debug, Clone)]
struct UserInfo {
    id: u64,
    username: String,
    email: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_verification_creation() {
        let verification = EmailVerification::default();
        assert_eq!(verification.config.code_length, 32);
        assert_eq!(verification.config.max_attempts, 5);
    }

    #[test]
    fn test_email_verification_with_user() {
        let verification = EmailVerification::with_user(123);
        assert_eq!(verification.current_user_id, Some(123));
    }

    #[test]
    fn test_generate_verification_code() {
        let verification = EmailVerification::default();
        let code1 = verification.generate_verification_code();
        let code2 = verification.generate_verification_code();
        
        assert_ne!(code1, code2);
        assert_eq!(code1.len(), verification.config.code_length);
        assert_eq!(code2.len(), verification.config.code_length);
    }

    #[test]
    fn test_email_validation() {
        let verification = EmailVerification::default();
        
        assert!(verification.is_valid_email("user@example.com"));
        assert!(verification.is_valid_email("test.user+tag@domain.co.uk"));
        assert!(!verification.is_valid_email("invalid"));
        assert!(!verification.is_valid_email("@example.com"));
        assert!(!verification.is_valid_email("user@"));
        assert!(!verification.is_valid_email(""));
    }

    #[test]
    fn test_timing_safe_equals() {
        let verification = EmailVerification::default();
        
        assert!(verification.timing_safe_equals("hello", "hello"));
        assert!(!verification.timing_safe_equals("hello", "world"));
        assert!(!verification.timing_safe_equals("hello", "hello_longer"));
        assert!(!verification.timing_safe_equals("longer", "short"));
    }

    #[test]
    fn test_verification_code_structure() {
        let code = VerificationCode {
            id: 1,
            user_id: 123,
            email: "test@example.com".to_string(),
            code: "test_code".to_string(),
            status: VerificationStatus::Pending,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
            verified_at: None,
            attempts: 0,
            max_attempts: 5,
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("Test Agent".to_string()),
        };

        assert_eq!(code.user_id, 123);
        assert_eq!(code.email, "test@example.com");
        assert!(matches!(code.status, VerificationStatus::Pending));
        assert!(code.verified_at.is_none());
    }

    #[test]
    fn test_template_data_preparation() {
        let verification = EmailVerification::default();
        let code = VerificationCode {
            id: 1,
            user_id: 123,
            email: "test@example.com".to_string(),
            code: "test_verification_code".to_string(),
            status: VerificationStatus::Pending,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
            verified_at: None,
            attempts: 0,
            max_attempts: 5,
            ip_address: None,
            user_agent: None,
        };

        let template_data = verification.prepare_template_data(&code, "testuser").unwrap();
        
        assert_eq!(template_data.user_id, 123);
        assert_eq!(template_data.username, "testuser");
        assert_eq!(template_data.email, "test@example.com");
        assert_eq!(template_data.verification_code, "test_verification_code");
        assert!(template_data.verification_url.contains("test_verification_code"));
    }

    #[test]
    fn test_send_verification_email() {
        let verification = EmailVerification::default();
        
        let result = verification.send_verification_email(
            123,
            "test@example.com",
            "testuser",
            Some("127.0.0.1".to_string()),
            Some("Test Agent".to_string()),
        );

        assert!(result.is_ok());
        let code = result.unwrap();
        assert_eq!(code.user_id, 123);
        assert_eq!(code.email, "test@example.com");
        assert!(matches!(code.status, VerificationStatus::Pending));
    }

    #[test]
    fn test_invalid_email_format() {
        let verification = EmailVerification::default();
        
        let result = verification.send_verification_email(
            123,
            "invalid_email",
            "testuser",
            None,
            None,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EmailVerificationError::InvalidEmail(_)));
    }

    #[test]
    fn test_verification_stats() {
        let verification = EmailVerification::default();
        let stats = verification.get_verification_stats().unwrap();
        
        assert_eq!(stats.total_codes_sent, 1000);
        assert_eq!(stats.successful_verifications, 850);
        assert_eq!(stats.success_rate, 85.0);
        assert!(stats.average_verification_time.is_some());
    }

    #[test]
    fn test_verify_by_code_only() {
        let verification = EmailVerification::default();
        
        let result = verification.verify_by_code_only("valid_code");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // Returns user_id

        let result = verification.verify_by_code_only("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EmailVerificationError::CodeNotFound));
    }
}