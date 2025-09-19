use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RememberMeError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("User not found: {0}")]
    UserNotFound(u64),
    #[error("Cookie not found")]
    CookieNotFound,
    #[error("Invalid cookie format")]
    InvalidCookieFormat,
    #[error("Signature verification failed")]
    SignatureVerificationFailed,
    #[error("Token tampering detected")]
    TokenTampering,
    #[error("Crypto error: {0}")]
    Crypto(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RememberMeToken {
    pub user_id: u64,
    pub token: String,
    pub signature: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RememberMeConfig {
    pub token_lifetime: Duration,
    pub max_tokens_per_user: u32,
    pub require_https: bool,
    pub secure_cookies: bool,
    pub http_only: bool,
    pub same_site: SameSitePolicy,
    pub domain: Option<String>,
    pub path: String,
    pub cookie_name: String,
    pub private_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SameSitePolicy {
    Strict,
    Lax,
    None,
}

impl Default for RememberMeConfig {
    fn default() -> Self {
        Self {
            token_lifetime: Duration::days(30),
            max_tokens_per_user: 5,
            require_https: true,
            secure_cookies: true,
            http_only: true,
            same_site: SameSitePolicy::Lax,
            domain: None,
            path: "/".to_string(),
            cookie_name: "remember_token".to_string(),
            private_key: "default_key_change_in_production".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    pub user_id: u64,
    pub username: String,
    pub token: RememberMeToken,
    pub is_new_session: bool,
    pub last_seen: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieData {
    pub user: u64,
    pub token: String,
    pub signature: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub expires: DateTime<Utc>,
}

/// Remember Me authentication system ported from PHP RememberMe class
/// Provides secure persistent login functionality with token rotation
pub struct RememberMe {
    config: RememberMeConfig,
}

impl RememberMe {
    /// Create new RememberMe instance with configuration
    pub fn new(config: RememberMeConfig) -> Self {
        Self { config }
    }

    /// Create RememberMe instance with default configuration
    pub fn default() -> Self {
        Self::new(RememberMeConfig::default())
    }

    /// Create RememberMe instance with custom private key
    pub fn with_private_key(private_key: String) -> Self {
        let config = RememberMeConfig {
            private_key,
            ..Default::default()
        };
        Self::new(config)
    }

    /// Authenticate user from remember me cookie
    pub fn authenticate(&self, cookie_value: Option<String>) -> Result<Option<AuthResult>, RememberMeError> {
        let cookie_value = match cookie_value {
            Some(value) if !value.is_empty() => value,
            _ => return Ok(None), // No cookie, not an error
        };

        // Decode and validate cookie
        let cookie_data = self.decode_cookie(&cookie_value)?;

        // Verify signature
        if !self.verify_signature(&cookie_data)? {
            return Err(RememberMeError::SignatureVerificationFailed);
        }

        // Check expiration
        if cookie_data.expires <= Utc::now() {
            return Err(RememberMeError::TokenExpired);
        }

        // Verify token in database
        let stored_token = self.get_stored_token(cookie_data.user, &cookie_data.token)?;
        if stored_token.token != cookie_data.token {
            return Err(RememberMeError::TokenTampering);
        }

        // Get user information
        let user_info = self.get_user_info(cookie_data.user)?;

        // Rotate token for security
        let new_token = self.rotate_token(cookie_data.user, &stored_token)?;

        Ok(Some(AuthResult {
            user_id: cookie_data.user,
            username: user_info.username,
            token: new_token,
            is_new_session: false,
            last_seen: stored_token.created_at.into(),
        }))
    }

    /// Create remember me token for user
    pub fn remember(
        &self,
        user_id: u64,
        create_cookie: bool,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(RememberMeToken, Option<String>), RememberMeError> {
        // Clean up old tokens for this user
        self.cleanup_user_tokens(user_id)?;

        // Generate new token
        let token_string = self.generate_token()?;
        let expires_at = Utc::now() + self.config.token_lifetime;

        let token = RememberMeToken {
            user_id,
            token: token_string.clone(),
            signature: String::new(), // Will be set by create_cookie_data
            created_at: Utc::now(),
            expires_at,
            ip_address,
            user_agent,
        };

        // Store token in database
        self.store_token(&token)?;

        // Create cookie if requested
        let cookie_value = if create_cookie {
            Some(self.create_cookie_data(&token)?)
        } else {
            None
        };

        Ok((token, cookie_value))
    }

    /// Revoke remember me token
    pub fn forget(&self, user_id: u64, token: Option<String>) -> Result<(), RememberMeError> {
        match token {
            Some(token_string) => {
                // Revoke specific token
                self.revoke_token(user_id, &token_string)?;
            }
            None => {
                // Revoke all tokens for user
                self.revoke_all_user_tokens(user_id)?;
            }
        }

        Ok(())
    }

    /// Validate remember me token without authentication
    pub fn validate_token(&self, cookie_value: &str) -> Result<bool, RememberMeError> {
        let cookie_data = self.decode_cookie(cookie_value)?;
        
        // Basic validation
        if cookie_data.expires <= Utc::now() {
            return Ok(false);
        }

        if !self.verify_signature(&cookie_data)? {
            return Ok(false);
        }

        // Check if token exists in database
        match self.get_stored_token(cookie_data.user, &cookie_data.token) {
            Ok(_) => Ok(true),
            Err(RememberMeError::InvalidToken) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Get all active tokens for user
    pub fn get_user_tokens(&self, user_id: u64) -> Result<Vec<RememberMeToken>, RememberMeError> {
        // Simulate database query
        // SELECT * FROM remember_tokens WHERE user_id = ? AND expires_at > NOW() ORDER BY created_at DESC
        Ok(vec![])
    }

    /// Revoke all tokens for user (useful for logout from all devices)
    pub fn revoke_all_user_tokens(&self, user_id: u64) -> Result<u32, RememberMeError> {
        // Simulate database operation
        // DELETE FROM remember_tokens WHERE user_id = ?
        Ok(0) // Return count of revoked tokens
    }

    /// Clean up expired tokens
    pub fn cleanup_expired_tokens(&self) -> Result<u32, RememberMeError> {
        // Simulate database cleanup
        // DELETE FROM remember_tokens WHERE expires_at <= NOW()
        Ok(0) // Return count of cleaned tokens
    }

    /// Get token statistics
    pub fn get_token_stats(&self) -> Result<TokenStats, RememberMeError> {
        Ok(TokenStats {
            total_active_tokens: 0,
            expired_tokens: 0,
            tokens_last_24h: 0,
            unique_users_with_tokens: 0,
            average_token_age: Duration::zero(),
        })
    }

    /// Private helper methods
    fn decode_cookie(&self, cookie_value: &str) -> Result<CookieData, RememberMeError> {
        // Base64 decode and JSON parse
        let decoded = base64::decode(cookie_value)
            .map_err(|_| RememberMeError::InvalidCookieFormat)?;
        
        let cookie_data: CookieData = serde_json::from_slice(&decoded)?;
        
        Ok(cookie_data)
    }

    fn create_cookie_data(&self, token: &RememberMeToken) -> Result<String, RememberMeError> {
        let cookie_data = CookieData {
            user: token.user_id,
            token: token.token.clone(),
            signature: String::new(), // Will be calculated below
            expires: token.expires_at,
        };

        // Calculate signature
        let signature = self.calculate_signature(&cookie_data)?;
        let cookie_data = CookieData {
            signature,
            ..cookie_data
        };

        // Encode to JSON and base64
        let json = serde_json::to_string(&cookie_data)?;
        let encoded = base64::encode(json.as_bytes());

        Ok(encoded)
    }

    fn verify_signature(&self, cookie_data: &CookieData) -> Result<bool, RememberMeError> {
        let expected_signature = self.calculate_signature(cookie_data)?;
        Ok(self.timing_safe_equals(&cookie_data.signature, &expected_signature))
    }

    fn calculate_signature(&self, cookie_data: &CookieData) -> Result<String, RememberMeError> {
        let data = format!("{}{}{}",
            cookie_data.user,
            cookie_data.token,
            cookie_data.expires.timestamp()
        );
        
        self.hmac_sign(&data)
    }

    fn hmac_sign(&self, data: &str) -> Result<String, RememberMeError> {
        // Add random salt for additional security
        let salt = self.generate_salt()?;
        let signed_data = format!("{}{}{}", salt, data, self.config.private_key);
        
        // Simple hash (in production, use proper HMAC)
        let hash = self.hash_data(&signed_data)?;
        
        Ok(format!("{}{}", salt, hash))
    }

    fn generate_token(&self) -> Result<String, RememberMeError> {
        self.generate_random_string(64)
    }

    fn generate_salt(&self) -> Result<String, RememberMeError> {
        self.generate_random_string(8)
    }

    fn generate_random_string(&self, length: usize) -> Result<String, RememberMeError> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // Simple random string generation (use proper crypto in production)
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow::anyhow!("Error: {}", e))?
            .as_nanos();
        
        let hash = self.hash_data(&format!("{}{}", timestamp, self.config.private_key))?;
        Ok(hash.chars().take(length).collect())
    }

    fn hash_data(&self, data: &str) -> Result<String, RememberMeError> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        Ok(format!("{:x}", hasher.finish()))
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

    fn get_stored_token(&self, user_id: u64, token: &str) -> Result<RememberMeToken, RememberMeError> {
        // Simulate database lookup
        // SELECT * FROM remember_tokens WHERE user_id = ? AND token = ? AND expires_at > NOW()
        
        if user_id == 0 || token.is_empty() {
            return Err(RememberMeError::InvalidToken);
        }

        // Mock token for testing
        Ok(RememberMeToken {
            user_id,
            token: token.to_string(),
            signature: String::new(),
            created_at: Utc::now() - Duration::hours(1),
            expires_at: Utc::now() + Duration::days(29),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("Test User Agent".to_string()),
        })
    }

    fn get_user_info(&self, user_id: u64) -> Result<UserInfo, RememberMeError> {
        // Simulate database lookup
        if user_id == 0 {
            return Err(RememberMeError::UserNotFound(user_id));
        }

        Ok(UserInfo {
            id: user_id,
            username: format!("user_{}", user_id),
            email: format!("user_{}@example.com", user_id),
            is_active: true,
        })
    }

    fn store_token(&self, token: &RememberMeToken) -> Result<(), RememberMeError> {
        // Simulate database storage
        // INSERT INTO remember_tokens (user_id, token, expires_at, created_at, ip_address, user_agent)
        // VALUES (?, ?, ?, ?, ?, ?)
        Ok(())
    }

    fn rotate_token(&self, user_id: u64, old_token: &RememberMeToken) -> Result<RememberMeToken, RememberMeError> {
        // Generate new token
        let new_token_string = self.generate_token()?;
        let new_token = RememberMeToken {
            user_id,
            token: new_token_string,
            signature: String::new(),
            created_at: Utc::now(),
            expires_at: Utc::now() + self.config.token_lifetime,
            ip_address: old_token.ip_address.clone(),
            user_agent: old_token.user_agent.clone(),
        };

        // Store new token
        self.store_token(&new_token)?;

        // Remove old token
        self.revoke_token(user_id, &old_token.token)?;

        Ok(new_token)
    }

    fn revoke_token(&self, user_id: u64, token: &str) -> Result<(), RememberMeError> {
        // Simulate database operation
        // DELETE FROM remember_tokens WHERE user_id = ? AND token = ?
        Ok(())
    }

    fn cleanup_user_tokens(&self, user_id: u64) -> Result<(), RememberMeError> {
        // Remove expired tokens and limit active tokens per user
        // 1. DELETE FROM remember_tokens WHERE user_id = ? AND expires_at <= NOW()
        // 2. DELETE FROM remember_tokens WHERE user_id = ? ORDER BY created_at ASC LIMIT ?
        //    (Keep only the most recent tokens up to max_tokens_per_user)
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct UserInfo {
    id: u64,
    username: String,
    email: String,
    is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStats {
    pub total_active_tokens: u64,
    pub expired_tokens: u64,
    pub tokens_last_24h: u64,
    pub unique_users_with_tokens: u64,
    pub average_token_age: Duration,
}

/// Cookie builder for HTTP responses
pub struct CookieBuilder {
    config: RememberMeConfig,
}

impl CookieBuilder {
    pub fn new(config: RememberMeConfig) -> Self {
        Self { config }
    }

    pub fn build_cookie_header(&self, cookie_value: &str) -> String {
        let mut parts = vec![
            format!("{}={}", self.config.cookie_name, cookie_value),
            format!("Max-Age={}", self.config.token_lifetime.num_seconds()),
            format!("Path={}", self.config.path),
        ];

        if let Some(domain) = &self.config.domain {
            parts.push(format!("Domain={}", domain));
        }

        if self.config.secure_cookies {
            parts.push("Secure".to_string());
        }

        if self.config.http_only {
            parts.push("HttpOnly".to_string());
        }

        match self.config.same_site {
            SameSitePolicy::Strict => parts.push("SameSite=Strict".to_string()),
            SameSitePolicy::Lax => parts.push("SameSite=Lax".to_string()),
            SameSitePolicy::None => parts.push("SameSite=None".to_string()),
        }

        parts.join("; ")
    }

    pub fn build_delete_cookie_header(&self) -> String {
        format!(
            "{}=; Max-Age=0; Path={}; Expires=Thu, 01 Jan 1970 00:00:00 GMT",
            self.config.cookie_name,
            self.config.path
        )
    }
}

// Re-export base64 for convenience (in real implementation, would use proper crate)
mod base64 {
    pub fn encode(data: &[u8]) -> String {
        // Simple base64 implementation for testing
        format!("base64_{}", data.len())
    }

    pub fn decode(data: &str) -> Result<Vec<u8>, String> {
        if data.starts_with("base64_") {
            let size: usize = data[7..].parse().map_err(|_| "Invalid format")?;
            Ok(vec![0u8; size])
        } else {
            Err("Invalid format".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remember_me_creation() {
        let remember_me = RememberMe::default();
        assert_eq!(remember_me.config.cookie_name, "remember_token");
        assert_eq!(remember_me.config.token_lifetime, Duration::days(30));
    }

    #[test]
    fn test_remember_me_with_key() {
        let remember_me = RememberMe::with_private_key("test_key".to_string());
        assert_eq!(remember_me.config.private_key, "test_key");
    }

    #[test]
    fn test_generate_token() {
        let remember_me = RememberMe::default();
        let token1 = remember_me.generate_token().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let token2 = remember_me.generate_token().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        assert_ne!(token1, token2);
        assert_eq!(token1.len(), 64);
    }

    #[test]
    fn test_timing_safe_equals() {
        let remember_me = RememberMe::default();
        
        assert!(remember_me.timing_safe_equals("hello", "hello"));
        assert!(!remember_me.timing_safe_equals("hello", "world"));
        assert!(!remember_me.timing_safe_equals("hello", "hello_longer"));
        assert!(!remember_me.timing_safe_equals("longer", "short"));
    }

    #[test]
    fn test_cookie_data_serialization() {
        let cookie_data = CookieData {
            user: 123,
            token: "test_token".to_string(),
            signature: "test_signature".to_string(),
            expires: Utc::now(),
        };

        let json = serde_json::to_string(&cookie_data).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let deserialized: CookieData = serde_json::from_str(&json).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        assert_eq!(cookie_data.user, deserialized.user);
        assert_eq!(cookie_data.token, deserialized.token);
        assert_eq!(cookie_data.signature, deserialized.signature);
    }

    #[test]
    fn test_cookie_builder() {
        let config = RememberMeConfig::default();
        let builder = CookieBuilder::new(config);
        
        let header = builder.build_cookie_header("test_value");
        assert!(header.contains("remember_token=test_value"));
        assert!(header.contains("HttpOnly"));
        assert!(header.contains("Secure"));
        assert!(header.contains("SameSite=Lax"));

        let delete_header = builder.build_delete_cookie_header();
        assert!(delete_header.contains("Max-Age=0"));
        assert!(delete_header.contains("Expires=Thu, 01 Jan 1970"));
    }

    #[test]
    fn test_remember_token_creation() {
        let remember_me = RememberMe::default();
        let result = remember_me.remember(
            123,
            true,
            Some("127.0.0.1".to_string()),
            Some("Test Agent".to_string()),
        );

        assert!(result.is_ok());
        let (token, cookie) = result.map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        assert_eq!(token.user_id, 123);
        assert!(!token.token.is_empty());
        assert!(cookie.is_some());
    }

    #[test]
    fn test_validate_token_invalid_format() {
        let remember_me = RememberMe::default();
        let result = remember_me.validate_token("invalid_cookie");
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RememberMeError::InvalidCookieFormat));
    }

    #[test]
    fn test_authenticate_no_cookie() {
        let remember_me = RememberMe::default();
        let result = remember_me.authenticate(None).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        assert!(result.is_none());
    }

    #[test]
    fn test_forget_user_tokens() {
        let remember_me = RememberMe::default();
        
        // Should not error even if no tokens exist
        let result = remember_me.forget(123, None);
        assert!(result.is_ok());

        let result = remember_me.forget(123, Some("test_token".to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_cleanup_expired_tokens() {
        let remember_me = RememberMe::default();
        let result = remember_me.cleanup_expired_tokens();
        
        assert!(result.is_ok());
        let count = result.map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert_eq!(count, 0); // Mock implementation returns 0
    }
}