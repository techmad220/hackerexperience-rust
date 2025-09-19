use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};
use regex::Regex;
use once_cell::sync::Lazy;

static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
});

static USERNAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9_-]{3,20}$").unwrap()
});

/// Input validation for user registration
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct RegisterInput {
    #[validate(length(min = 3, max = 20), regex = "USERNAME_REGEX")]
    pub username: String,

    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8, max = 128), custom = "validate_password_strength")]
    pub password: String,
}

/// Input validation for login
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct LoginInput {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

/// Input validation for process creation
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct ProcessInput {
    #[validate(range(min = 1, max = 1000000))]
    pub target_id: i64,

    #[validate(length(min = 1, max = 50))]
    pub process_type: String,

    #[validate(range(min = 1, max = 100))]
    pub priority: Option<u8>,
}

/// Input validation for clan operations
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct ClanInput {
    #[validate(length(min = 3, max = 30))]
    pub name: String,

    #[validate(length(max = 500))]
    pub description: Option<String>,
}

/// Input validation for transfer operations
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct TransferInput {
    #[validate(range(min = 1))]
    pub recipient_id: i64,

    #[validate(range(min = 1, max = 1000000000))]
    pub amount: i64,

    #[validate(length(max = 200))]
    pub memo: Option<String>,
}

/// Custom password validation
fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_uppercase || !has_lowercase || !has_digit {
        return Err(ValidationError::new("weak_password"));
    }

    if password.len() >= 12 && !has_special {
        return Err(ValidationError::new("missing_special_char"));
    }

    Ok(())
}

/// Sanitize HTML input to prevent XSS
pub fn sanitize_html(input: &str) -> String {
    ammonia::clean(input)
}

/// Validate and sanitize file paths
pub fn validate_path(path: &str) -> Result<String, ValidationError> {
    if path.contains("..") || path.contains("~") {
        return Err(ValidationError::new("invalid_path"));
    }

    Ok(path.to_string())
}

/// Rate limiting check
pub struct RateLimiter {
    attempts: std::sync::RwLock<std::collections::HashMap<String, Vec<std::time::Instant>>>,
    max_attempts: usize,
    window: std::time::Duration,
}

impl RateLimiter {
    pub fn new(max_attempts: usize, window_seconds: u64) -> Self {
        Self {
            attempts: std::sync::RwLock::new(std::collections::HashMap::new()),
            max_attempts,
            window: std::time::Duration::from_secs(window_seconds),
        }
    }

    pub fn check(&self, key: &str) -> bool {
        let mut attempts = self.attempts.write().unwrap();
        let now = std::time::Instant::now();

        let entry = attempts.entry(key.to_string()).or_insert_with(Vec::new);

        // Remove old attempts outside the window
        entry.retain(|&attempt| now.duration_since(attempt) < self.window);

        if entry.len() >= self.max_attempts {
            false
        } else {
            entry.push(now);
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let input = LoginInput {
            email: "test@example.com".to_string(),
            password: "SecurePass123!".to_string(),
        };
        assert!(input.validate().is_ok());

        let invalid = LoginInput {
            email: "invalid-email".to_string(),
            password: "SecurePass123!".to_string(),
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_password_strength() {
        assert!(validate_password_strength("weak").is_err());
        assert!(validate_password_strength("NoDigits!").is_err());
        assert!(validate_password_strength("SecurePass123").is_ok());
        assert!(validate_password_strength("VerySecurePass123!").is_ok());
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(3, 60);
        let key = "test_user";

        assert!(limiter.check(key));
        assert!(limiter.check(key));
        assert!(limiter.check(key));
        assert!(!limiter.check(key)); // Should fail on 4th attempt
    }
}