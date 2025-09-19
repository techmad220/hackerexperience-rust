// SYSTEM.CLASS.PHP PORT - Core utility functions and validation
// Original: Core utilities for error handling, validation, and parameter checking

use std::collections::HashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetResult {
    pub isset_get: bool,
    pub get_name: String,
    pub get_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericResult {
    pub is_numeric: bool,
    pub get_value: Option<i32>,
}

pub struct System {
    // Future: Add database and session references if needed
}

impl System {
    pub fn new() -> Self {
        Self {}
    }
    
    // Original PHP: issetGet - Check if GET parameter exists and is not empty
    pub fn isset_get(params: &HashMap<String, String>, get: &str) -> bool {
        params.get(get).map_or(false, |v| !v.is_empty())
    }
    
    // Original PHP: switchGet - Check if GET parameter matches any of the provided values
    pub fn switch_get(params: &HashMap<String, String>, get: &str, allowed_values: &[&str]) -> GetResult {
        if let Some(value) = params.get(get) {
            if allowed_values.contains(&value.as_str()) {
                return GetResult {
                    isset_get: true,
                    get_name: get.to_string(),
                    get_value: value.clone(),
                };
            }
        }
        
        GetResult {
            isset_get: false,
            get_name: String::new(),
            get_value: String::new(),
        }
    }
    
    // Original PHP: verifyNumericGet - Check if GET parameter is numeric
    pub fn verify_numeric_get(params: &HashMap<String, String>, get: &str) -> NumericResult {
        if Self::isset_get(params, get) {
            if let Some(value_str) = params.get(get) {
                if let Ok(value) = value_str.parse::<i32>() {
                    if value != 0 {
                        return NumericResult {
                            is_numeric: true,
                            get_value: Some(value),
                        };
                    }
                }
            }
        }
        
        NumericResult {
            is_numeric: false,
            get_value: None,
        }
    }
    
    // Original PHP: verifyStringGet - Check if GET parameter is valid string
    pub fn verify_string_get(params: &HashMap<String, String>, get: &str) -> GetResult {
        if Self::isset_get(params, get) {
            if let Some(value) = params.get(get) {
                if !value.is_empty() && value.len() <= 50 { // reasonable limit
                    return GetResult {
                        isset_get: true,
                        get_name: get.to_string(),
                        get_value: value.clone(),
                    };
                }
            }
        }
        
        GetResult {
            isset_get: false,
            get_name: String::new(),
            get_value: String::new(),
        }
    }
    
    // Original PHP: handleError - Get error message for error codes
    pub fn handle_error(error: &str) -> String {
        match error {
            "" => String::new(),
            "INVALID_GET" => "Invalid page".to_string(),
            "INVALID_ID" => "Invalid ID".to_string(),
            "WRONG_PASS" => "Ops! Wrong password.".to_string(),
            "NO_PERMISSION" => "You do not have permission to perform this.".to_string(),
            "NO_INSTALLER" => "Can't find doom installer.".to_string(),
            "NO_COLLECTOR" => "You do not have a virus collector.".to_string(),
            "DOWNGRADE" => "Downgrade hardware is not possible.".to_string(),
            "NOT_LISTED" => "This IP is not listed on your hacked database.".to_string(),
            "INEXISTENT_SERVER" => "This server does not exist.".to_string(),
            "INEXISTENT_IP" => "This IP does not exist.".to_string(),
            "INVALID_IP" => "This IP is invalid.".to_string(),
            "BAD_XHD" => "Your external HD does not support this software.".to_string(),
            "BAD_MONEY" => "You do not have enough money to complete this action.".to_string(),
            "NO_LICENSE" => "You don't have license to use this software.".to_string(),
            "HARDWARE_OFFLINE" => "This hardware is offline.".to_string(),
            "SOFTWARE_RUNNING" => "This software is already running.".to_string(),
            "PROCESS_LIMIT" => "You have reached the maximum number of processes.".to_string(),
            "ALREADY_INSTALLED" => "This software is already installed.".to_string(),
            "MISSING_DEPENDENCY" => "Missing software dependency.".to_string(),
            "INSUFFICIENT_STORAGE" => "Insufficient storage space.".to_string(),
            "NETWORK_ERROR" => "Network connection error.".to_string(),
            "SERVER_OVERLOADED" => "Server is currently overloaded.".to_string(),
            "MAINTENANCE_MODE" => "Server is in maintenance mode.".to_string(),
            "RATE_LIMITED" => "Rate limit exceeded. Please try again later.".to_string(),
            "INVALID_TOKEN" => "Invalid security token.".to_string(),
            "SESSION_EXPIRED" => "Your session has expired.".to_string(),
            "BANNED_USER" => "Your account has been suspended.".to_string(),
            _ => format!("Unknown error: {}", error),
        }
    }
    
    // Original PHP: validate - Comprehensive validation function
    pub fn validate(var: &str, validation_type: &str) -> bool {
        match validation_type {
            "ip" | "IP" => {
                // IPv4 validation
                Self::validate_ipv4(var)
            },
            "hintip" => {
                // XXX.XXX.XXX.XXX format (with X as wildcards)
                let re = Regex::new(r"^[0-9.xX]{7,15}$").unwrap();
                re.is_match(var)
            },
            "user" | "username" => {
                // Username validation: alphanumeric + ._- (1-15 chars)
                let re = Regex::new(r"^[a-zA-Z0-9_.-]{1,15}$").unwrap();
                re.is_match(var)
            },
            "soft" | "software" => {
                // Software name validation
                let re = Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9_ -]{1,}$").unwrap();
                re.is_match(var)
            },
            "subject" => {
                // Subject line validation (with international characters)
                let re = Regex::new(r"^[a-zA-Z0-9áÁÀàóÓêÊõíÍúÚçÇñã][a-zA-Z0-9çÇáÁÀàóÓêÊõíÍúÚñã.,_$!?()'\" -]{1,}$").unwrap();
                re.is_match(var)
            },
            "text" => {
                // General text validation (with special characters)
                let re = Regex::new(r"^[a-zA-Z0-9áÁÀàóÓêÊõíÍúÚñãçÇ][a-zA-Z0-9çÇáÁÀàóÓêÊõíÍúÚñã.,_$!()'\"@#%*+={}<> -?!]{1,}$").unwrap();
                re.is_match(var)
            },
            "email" => {
                // Email validation
                Self::validate_email(var)
            },
            "clan_name" => {
                // Clan name validation
                let re = Regex::new(r"^[a-zA-Z0-9áÁÀàóÓêÊõíçÇÍúÚñã][a-zA-Z0-9áÁÀàóÓêÊçÇõíÍúÚñã_.! -]{1,}$").unwrap();
                re.is_match(var)
            },
            "clan_tag" => {
                // Clan tag validation (usually 2-6 characters)
                let re = Regex::new(r"^[a-zA-Z0-9]{2,6}$").unwrap();
                re.is_match(var)
            },
            "password" => {
                // Password validation (minimum 6 characters)
                var.len() >= 6 && var.len() <= 128
            },
            "numeric" => {
                // Numeric validation
                var.parse::<i64>().is_ok()
            },
            "alphanumeric" => {
                // Alphanumeric validation
                let re = Regex::new(r"^[a-zA-Z0-9]+$").unwrap();
                re.is_match(var)
            },
            "hex" => {
                // Hexadecimal validation
                let re = Regex::new(r"^[0-9a-fA-F]+$").unwrap();
                re.is_match(var)
            },
            "domain" => {
                // Domain name validation
                let re = Regex::new(r"^[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap();
                re.is_match(var)
            },
            "port" => {
                // Port number validation (1-65535)
                if let Ok(port) = var.parse::<u16>() {
                    port > 0
                } else {
                    false
                }
            },
            _ => false,
        }
    }
    
    // Helper function for IPv4 validation
    fn validate_ipv4(ip: &str) -> bool {
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() != 4 {
            return false;
        }
        
        for part in parts {
            if let Ok(num) = part.parse::<u8>() {
                // Valid range 0-255
                continue;
            } else {
                return false;
            }
        }
        
        true
    }
    
    // Helper function for email validation
    fn validate_email(email: &str) -> bool {
        let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        re.is_match(email) && email.len() <= 254 // RFC 5321 limit
    }
    
    // Original PHP: changeHTML - Generate JavaScript for DOM manipulation
    pub fn change_html(id: &str, content: &str) -> String {
        // In the original PHP, this would output JavaScript directly
        // In Rust, we return the JavaScript as a string for the frontend to use
        format!(
            r#"<script>document.getElementById("{}").innerHTML="{}";</script>"#,
            Self::escape_js(id),
            Self::escape_js(content)
        )
    }
    
    // Utility functions for string manipulation and security
    pub fn escape_js(input: &str) -> String {
        input
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }
    
    pub fn sanitize_filename(filename: &str) -> String {
        // Remove dangerous characters from filenames
        let re = Regex::new(r"[^a-zA-Z0-9._-]").unwrap();
        re.replace_all(filename, "_").to_string()
    }
    
    pub fn generate_token() -> String {
        // Generate secure random token for CSRF protection
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        (0..32)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
    
    pub fn hash_password(password: &str) -> Result<String, SystemError> {
        bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| SystemError::HashingError(e.to_string()))
    }
    
    pub fn verify_password(password: &str, hash: &str) -> Result<bool, SystemError> {
        bcrypt::verify(password, hash)
            .map_err(|e| SystemError::HashingError(e.to_string()))
    }
    
    // Rate limiting utilities
    pub fn check_rate_limit(key: &str, max_requests: u32, window_seconds: u32) -> bool {
        // TODO: Implement with Redis or in-memory store
        // For now, always allow (placeholder)
        true
    }
    
    // IP address utilities
    pub fn get_client_ip(headers: &std::collections::HashMap<String, String>) -> String {
        // Check common headers for client IP
        if let Some(forwarded) = headers.get("x-forwarded-for") {
            if let Some(ip) = forwarded.split(',').next() {
                return ip.trim().to_string();
            }
        }
        
        if let Some(real_ip) = headers.get("x-real-ip") {
            return real_ip.clone();
        }
        
        // Fallback to direct connection IP
        headers.get("remote-addr").cloned().unwrap_or_else(|| "unknown".to_string())
    }
    
    // Convert long IP to dotted notation (like PHP's long2ip)
    pub fn long_to_ip(ip_long: u32) -> String {
        format!(
            "{}.{}.{}.{}",
            (ip_long >> 24) & 0xFF,
            (ip_long >> 16) & 0xFF,
            (ip_long >> 8) & 0xFF,
            ip_long & 0xFF
        )
    }
    
    // Convert dotted notation to long (like PHP's ip2long)
    pub fn ip_to_long(ip: &str) -> Result<u32, SystemError> {
        if !Self::validate_ipv4(ip) {
            return Err(SystemError::InvalidIP(ip.to_string()));
        }
        
        let parts: Vec<u32> = ip
            .split('.')
            .map(|s| s.parse().unwrap())
            .collect();
            
        Ok((parts[0] << 24) + (parts[1] << 16) + (parts[2] << 8) + parts[3])
    }
}

impl Default for System {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum SystemError {
    ValidationError(String),
    HashingError(String),
    InvalidIP(String),
}

impl std::fmt::Display for SystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            SystemError::HashingError(msg) => write!(f, "Hashing error: {}", msg),
            SystemError::InvalidIP(ip) => write!(f, "Invalid IP address: {}", ip),
        }
    }
}

impl std::error::Error for SystemError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_ip() {
        assert!(System::validate("192.168.1.1", "ip"));
        assert!(System::validate("127.0.0.1", "ip"));
        assert!(!System::validate("999.999.999.999", "ip"));
        assert!(!System::validate("invalid", "ip"));
    }
    
    #[test]
    fn test_validate_username() {
        assert!(System::validate("user123", "username"));
        assert!(System::validate("test_user", "username"));
        assert!(System::validate("a", "username"));
        assert!(!System::validate("", "username"));
        assert!(!System::validate("verylongusernamethatexceedslimit", "username"));
    }
    
    #[test]
    fn test_validate_email() {
        assert!(System::validate("test@example.com", "email"));
        assert!(System::validate("user.name+tag@domain.co.uk", "email"));
        assert!(!System::validate("invalid-email", "email"));
        assert!(!System::validate("@example.com", "email"));
    }
    
    #[test]
    fn test_ip_conversion() {
        let ip = "192.168.1.1";
        let long = System::ip_to_long(ip).unwrap();
        let converted_back = System::long_to_ip(long);
        assert_eq!(ip, converted_back);
    }
}