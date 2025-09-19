use anyhow::{anyhow, Result};
//! Input validation module for secure data handling
//!
//! This module provides comprehensive input validation to prevent
//! injection attacks and ensure data integrity.

use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};
use once_cell::sync::Lazy;

// Regex patterns for validation
static USERNAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9_-]{3,32}$").map_err(|e| anyhow::anyhow!("Error: {}", e))?
});

static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").map_err(|e| anyhow::anyhow!("Error: {}", e))?
});

static IP_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$").map_err(|e| anyhow::anyhow!("Error: {}", e))?
});

/// Validates that a string contains no SQL injection attempts
fn validate_no_sql_injection(value: &str) -> Result<(), ValidationError> {
    let sql_keywords = [
        "SELECT", "INSERT", "UPDATE", "DELETE", "DROP", "UNION",
        "EXEC", "EXECUTE", "--", "/*", "*/", "xp_", "sp_", "0x"
    ];

    let uppercase_value = value.to_uppercase();
    for keyword in &sql_keywords {
        if uppercase_value.contains(keyword) {
            return Err(ValidationError::new("potential_sql_injection"));
        }
    }

    // Check for common SQL injection patterns
    if value.contains("'") && (value.contains("=") || value.contains("OR")) {
        return Err(ValidationError::new("sql_injection_pattern"));
    }

    Ok(())
}

/// Validates that a string contains no XSS attempts
fn validate_no_xss(value: &str) -> Result<(), ValidationError> {
    let xss_patterns = [
        "<script", "</script>", "javascript:", "onload=", "onerror=",
        "onclick=", "onmouseover=", "<iframe", "<embed", "<object"
    ];

    let lowercase_value = value.to_lowercase();
    for pattern in &xss_patterns {
        if lowercase_value.contains(pattern) {
            return Err(ValidationError::new("potential_xss"));
        }
    }

    Ok(())
}

/// Validates that a string contains no path traversal attempts
fn validate_no_path_traversal(value: &str) -> Result<(), ValidationError> {
    let traversal_patterns = ["../", "..\\", "%2e%2e", "..%2F", "..%5C"];

    for pattern in &traversal_patterns {
        if value.contains(pattern) {
            return Err(ValidationError::new("path_traversal"));
        }
    }

    Ok(())
}

/// User registration input validation
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct RegisterInput {
    #[validate(length(min = 3, max = 32), regex = "USERNAME_REGEX", custom = "validate_no_sql_injection")]
    pub username: String,

    #[validate(email, custom = "validate_no_sql_injection")]
    pub email: String,

    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

/// User login input validation
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct LoginInput {
    #[validate(custom = "validate_no_sql_injection", custom = "validate_no_xss")]
    pub username_or_email: String,

    #[validate(length(min = 1, max = 128))]
    pub password: String,
}

/// Process creation input validation
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct CreateProcessInput {
    #[validate(length(min = 1, max = 50), custom = "validate_no_sql_injection")]
    pub process_type: String,

    #[validate(custom = "validate_no_sql_injection", custom = "validate_no_path_traversal")]
    pub target: Option<String>,

    #[validate(range(min = 1, max = 3600))]
    pub duration_seconds: Option<i32>,
}

/// Bank transfer input validation
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct BankTransferInput {
    #[validate(length(min = 10, max = 34), custom = "validate_no_sql_injection")]
    pub from_account: String,

    #[validate(length(min = 10, max = 34), custom = "validate_no_sql_injection")]
    pub to_account: String,

    #[validate(range(min = 1.0, max = 1_000_000_000.0))]
    pub amount: f64,

    #[validate(length(max = 500), custom = "validate_no_xss")]
    pub memo: Option<String>,
}

/// Chat message input validation
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct ChatMessageInput {
    #[validate(length(min = 1, max = 1000), custom = "validate_no_xss", custom = "validate_no_sql_injection")]
    pub message: String,

    #[validate(length(min = 1, max = 50), custom = "validate_no_sql_injection")]
    pub channel: String,
}

/// IP address validation
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct IPInput {
    #[validate(regex = "IP_REGEX", custom = "validate_no_sql_injection")]
    pub ip_address: String,
}

/// Search query validation
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct SearchInput {
    #[validate(length(min = 1, max = 100), custom = "validate_no_sql_injection", custom = "validate_no_xss")]
    pub query: String,

    #[validate(range(min = 1, max = 100))]
    pub limit: Option<u32>,

    #[validate(range(min = 0))]
    pub offset: Option<u32>,
}

/// File upload validation
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct FileUploadInput {
    #[validate(length(min = 1, max = 255), custom = "validate_no_path_traversal", custom = "validate_no_xss")]
    pub filename: String,

    #[validate(range(min = 1, max = 10_485_760))] // 10MB max
    pub size: usize,

    #[validate(custom = "validate_mime_type")]
    pub mime_type: String,
}

/// Validates MIME type for file uploads
fn validate_mime_type(mime_type: &str) -> Result<(), ValidationError> {
    let allowed_types = [
        "text/plain",
        "text/csv",
        "application/json",
        "image/png",
        "image/jpeg",
        "image/gif",
        "application/pdf",
    ];

    if !allowed_types.contains(&mime_type) {
        return Err(ValidationError::new("invalid_mime_type"));
    }

    Ok(())
}

/// Hardware upgrade input validation
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct HardwareUpgradeInput {
    #[validate(length(min = 1, max = 50), custom = "validate_no_sql_injection")]
    pub component: String,

    #[validate(range(min = 1, max = 100))]
    pub level: u32,

    #[validate(range(min = 0.0))]
    pub cost: Option<f64>,
}

/// Mission action input validation
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct MissionActionInput {
    #[validate(range(min = 1))]
    pub mission_id: i64,

    #[validate(length(min = 1, max = 50), custom = "validate_no_sql_injection")]
    pub action: String,

    #[validate(custom = "validate_no_xss", custom = "validate_no_sql_injection")]
    pub data: Option<String>,
}

/// Sanitizes user input by removing potentially dangerous characters
pub fn sanitize_input(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.' | '@'))
        .take(1000) // Limit length
        .collect()
}

/// Sanitizes HTML content to prevent XSS
pub fn sanitize_html(html: &str) -> String {
    ammonia::clean(html)
}

/// Validation middleware for API endpoints
pub struct ValidationMiddleware;

impl ValidationMiddleware {
    /// Validates any input that implements the Validate trait
    pub fn validate<T: Validate>(input: &T) -> Result<(), ValidationError> {
        input.validate()?;
        Ok(())
    }

    /// Validates and sanitizes string input
    pub fn validate_string(input: &str, max_length: usize) -> Result<String, ValidationError> {
        if input.is_empty() {
            return Err(ValidationError::new("empty_input"));
        }

        if input.len() > max_length {
            return Err(ValidationError::new("input_too_long"));
        }

        validate_no_sql_injection(input)?;
        validate_no_xss(input)?;
        validate_no_path_traversal(input)?;

        Ok(sanitize_input(input))
    }

    /// Validates numeric input
    pub fn validate_number<T: PartialOrd>(value: T, min: T, max: T) -> Result<T, ValidationError> {
        if value < min || value > max {
            return Err(ValidationError::new("out_of_range"));
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_injection_detection() {
        assert!(validate_no_sql_injection("normal input").is_ok());
        assert!(validate_no_sql_injection("SELECT * FROM users").is_err());
        assert!(validate_no_sql_injection("1' OR '1'='1").is_err());
        assert!(validate_no_sql_injection("DROP TABLE users--").is_err());
    }

    #[test]
    fn test_xss_detection() {
        assert!(validate_no_xss("normal text").is_ok());
        assert!(validate_no_xss("<script>alert('XSS')</script>").is_err());
        assert!(validate_no_xss("javascript:alert(1)").is_err());
        assert!(validate_no_xss("<img src=x onerror=alert(1)>").is_err());
    }

    #[test]
    fn test_path_traversal_detection() {
        assert!(validate_no_path_traversal("normal/path").is_ok());
        assert!(validate_no_path_traversal("../../../etc/passwd").is_err());
        assert!(validate_no_path_traversal("..\\windows\\system32").is_err());
    }

    #[test]
    fn test_input_sanitization() {
        assert_eq!(sanitize_input("hello world"), "hello world");
        assert_eq!(sanitize_input("test@email.com"), "test@email.com");
        assert_eq!(sanitize_input("DROP TABLE; users"), " users");
        assert_eq!(sanitize_input("<script>alert()</script>"), "scriptalertscript");
    }

    #[test]
    fn test_registration_validation() {
        let valid_input = RegisterInput {
            username: "validuser123".to_string(),
            email: "test@example.com".to_string(),
            password: "SecurePass123!".to_string(),
        };
        assert!(valid_input.validate().is_ok());

        let invalid_input = RegisterInput {
            username: "a".to_string(), // Too short
            email: "not-an-email".to_string(),
            password: "123".to_string(), // Too short
        };
        assert!(invalid_input.validate().is_err());
    }
}