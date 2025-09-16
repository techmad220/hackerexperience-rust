use actix_web::Error;
use regex::Regex;
use lazy_static::lazy_static;
use ammonia::clean;

lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(
        r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
    ).unwrap();

    static ref USERNAME_REGEX: Regex = Regex::new(
        r"^[a-zA-Z0-9_]{3,20}$"
    ).unwrap();

    static ref IP_REGEX: Regex = Regex::new(
        r"^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}$"
    ).unwrap();

    static ref SQL_INJECTION_REGEX: Regex = Regex::new(
        r"(?i)(union|select|insert|update|delete|drop|create|alter|exec|execute|script|javascript|onclick|onload)"
    ).unwrap();

    static ref PATH_TRAVERSAL_REGEX: Regex = Regex::new(
        r"(\.\./|\.\.\\|%2e%2e%2f|%2e%2e/|\.\.%2f|%2e%2e%5c)"
    ).unwrap();
}

/// Input validator for security
pub struct InputValidator;

impl InputValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validate email format
    pub fn validate_email(&self, email: &str) -> Result<(), Error> {
        if !EMAIL_REGEX.is_match(email) {
            return Err(actix_web::error::ErrorBadRequest("Invalid email format"));
        }
        Ok(())
    }

    /// Validate username format
    pub fn validate_username(&self, username: &str) -> Result<(), Error> {
        if !USERNAME_REGEX.is_match(username) {
            return Err(actix_web::error::ErrorBadRequest(
                "Username must be 3-20 characters, alphanumeric and underscore only"
            ));
        }
        Ok(())
    }

    /// Validate IP address format
    pub fn validate_ip(&self, ip: &str) -> Result<(), Error> {
        if !IP_REGEX.is_match(ip) {
            return Err(actix_web::error::ErrorBadRequest("Invalid IP address format"));
        }
        Ok(())
    }

    /// Check for SQL injection patterns
    pub fn check_sql_injection(&self, input: &str) -> Result<(), Error> {
        if SQL_INJECTION_REGEX.is_match(input) {
            return Err(actix_web::error::ErrorBadRequest(
                "Input contains potentially malicious patterns"
            ));
        }
        Ok(())
    }

    /// Check for path traversal attempts
    pub fn check_path_traversal(&self, path: &str) -> Result<(), Error> {
        if PATH_TRAVERSAL_REGEX.is_match(path) {
            return Err(actix_web::error::ErrorBadRequest(
                "Path contains invalid characters"
            ));
        }
        Ok(())
    }

    /// Sanitize HTML input to prevent XSS
    pub fn sanitize_html(&self, input: &str) -> String {
        clean(input)
    }

    /// Sanitize filename
    pub fn sanitize_filename(&self, filename: &str) -> String {
        filename
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
            .collect::<String>()
            .chars()
            .take(255) // Max filename length
            .collect()
    }

    /// Validate and sanitize JSON input
    pub fn validate_json(&self, json_str: &str) -> Result<serde_json::Value, Error> {
        serde_json::from_str(json_str)
            .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid JSON: {}", e)))
    }

    /// Validate integer range
    pub fn validate_int_range(&self, value: i64, min: i64, max: i64) -> Result<(), Error> {
        if value < min || value > max {
            return Err(actix_web::error::ErrorBadRequest(
                format!("Value must be between {} and {}", min, max)
            ));
        }
        Ok(())
    }

    /// Validate string length
    pub fn validate_string_length(&self, s: &str, min: usize, max: usize) -> Result<(), Error> {
        let len = s.len();
        if len < min || len > max {
            return Err(actix_web::error::ErrorBadRequest(
                format!("String length must be between {} and {} characters", min, max)
            ));
        }
        Ok(())
    }
}