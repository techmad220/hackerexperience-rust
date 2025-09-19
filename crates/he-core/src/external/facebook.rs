use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FacebookError {
    #[error("API error: {0}")]
    Api(String),
    #[error("Authentication error: {0}")]
    Auth(String),
    #[error("Invalid access token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Network error: {0}")]
    Network(String),
    #[error("JSON parsing error: {0}")]
    JsonParsing(#[from] serde_json::Error),
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Invalid app configuration")]
    InvalidAppConfig,
    #[error("User not found")]
    UserNotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacebookConfig {
    pub app_id: String,
    pub app_secret: String,
    pub api_version: String,
    pub redirect_uri: String,
    pub permissions: Vec<String>,
    pub enable_file_upload: bool,
    pub trust_forwarded_headers: bool,
    pub allow_signed_request: bool,
}

impl Default for FacebookConfig {
    fn default() -> Self {
        Self {
            app_id: String::new(),
            app_secret: String::new(),
            api_version: "v18.0".to_string(),
            redirect_uri: String::new(),
            permissions: vec![
                "public_profile".to_string(),
                "email".to_string(),
            ],
            enable_file_upload: false,
            trust_forwarded_headers: false,
            allow_signed_request: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacebookUser {
    pub id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub picture: Option<FacebookPicture>,
    pub locale: Option<String>,
    pub timezone: Option<i32>,
    pub verified: Option<bool>,
    pub link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacebookPicture {
    pub data: FacebookPictureData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacebookPictureData {
    pub height: u32,
    pub width: u32,
    pub url: String,
    pub is_silhouette: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    pub token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub expires_at: Option<DateTime<Utc>>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacebookApiResponse<T> {
    pub data: Option<T>,
    pub error: Option<FacebookApiError>,
    pub paging: Option<FacebookPaging>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacebookApiError {
    pub message: String,
    pub r#type: String,
    pub code: u32,
    pub error_subcode: Option<u32>,
    pub fbtrace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacebookPaging {
    pub cursors: Option<FacebookCursors>,
    pub previous: Option<String>,
    pub next: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacebookCursors {
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginUrlParams {
    pub redirect_uri: Option<String>,
    pub scope: Option<Vec<String>>,
    pub display: Option<String>,
    pub state: Option<String>,
    pub response_type: Option<String>,
}

/// Facebook SDK integration ported from PHP Facebook class
/// Provides OAuth authentication, API calls, and user data retrieval
pub struct Facebook {
    config: FacebookConfig,
    access_token: Option<AccessToken>,
    user_id: Option<String>,
    signed_request: Option<String>,
    state: Option<String>,
}

impl Facebook {
    /// Create new Facebook instance with configuration
    pub fn new(config: FacebookConfig) -> Result<Self, FacebookError> {
        if config.app_id.is_empty() || config.app_secret.is_empty() {
            return Err(FacebookError::InvalidAppConfig);
        }

        Ok(Self {
            config,
            access_token: None,
            user_id: None,
            signed_request: None,
            state: None,
        })
    }

    /// Create Facebook instance with app credentials
    pub fn with_app_credentials(app_id: String, app_secret: String) -> Result<Self, FacebookError> {
        let config = FacebookConfig {
            app_id,
            app_secret,
            ..Default::default()
        };
        Self::new(config)
    }

    /// Set access token
    pub fn set_access_token(&mut self, token: AccessToken) {
        self.access_token = Some(token);
    }

    /// Get current access token
    pub fn get_access_token(&self) -> Option<&AccessToken> {
        self.access_token.as_ref()
    }

    /// Get login URL for OAuth flow
    pub fn get_login_url(&self, params: Option<LoginUrlParams>) -> String {
        let params = params.unwrap_or_default();
        
        let redirect_uri = params.redirect_uri
            .unwrap_or_else(|| self.config.redirect_uri.clone());
        
        let scope = params.scope
            .unwrap_or_else(|| self.config.permissions.clone())
            .join(",");

        let state = params.state
            .unwrap_or_else(|| self.generate_state());

        let mut url_params = vec![
            ("client_id", self.config.app_id.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("scope", scope.as_str()),
            ("response_type", params.response_type.as_deref().unwrap_or("code")),
            ("state", state.as_str()),
        ];

        if let Some(display) = params.display.as_deref() {
            url_params.push(("display", display));
        }

        let query_string = url_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        format!(
            "https://www.facebook.com/{}/dialog/oauth?{}",
            self.config.api_version,
            query_string
        )
    }

    /// Get logout URL
    pub fn get_logout_url(&self, next_url: Option<String>) -> String {
        let next = next_url.unwrap_or_else(|| self.config.redirect_uri.clone());
        let access_token = self.access_token
            .as_ref()
            .map(|t| t.token.as_str())
            .unwrap_or("");

        format!(
            "https://www.facebook.com/logout.php?next={}&access_token={}",
            urlencoding::encode(&next),
            urlencoding::encode(access_token)
        )
    }

    /// Exchange authorization code for access token
    pub fn get_access_token_from_code(&mut self, code: &str, redirect_uri: Option<String>) -> Result<AccessToken, FacebookError> {
        if code.is_empty() {
            return Err(FacebookError::Auth("Empty authorization code".to_string()));
        }

        let redirect_uri = redirect_uri
            .unwrap_or_else(|| self.config.redirect_uri.clone());

        let params = [
            ("client_id", self.config.app_id.as_str()),
            ("client_secret", self.config.app_secret.as_str()),
            ("code", code),
            ("redirect_uri", redirect_uri.as_str()),
        ];

        let url = format!(
            "https://graph.facebook.com/{}/oauth/access_token",
            self.config.api_version
        );

        // In real implementation, make HTTP request
        // For now, simulate response
        let mock_response = r#"{
            "access_token": "mock_access_token_123",
            "token_type": "bearer",
            "expires_in": 3600
        }"#;

        let response: serde_json::Value = serde_json::from_str(mock_response)?;
        
        if let Some(error) = response.get("error") {
            return Err(FacebookError::Api(error.to_string()));
        }

        let access_token = AccessToken {
            token: response["access_token"].as_str().unwrap_or("").to_string(),
            token_type: response["token_type"].as_str().unwrap_or("bearer").to_string(),
            expires_in: response["expires_in"].as_u64(),
            expires_at: response["expires_in"].as_u64().map(|exp| {
                Utc::now() + chrono::Duration::seconds(exp as i64)
            }),
            scope: None,
        };

        self.access_token = Some(access_token.clone());
        Ok(access_token)
    }

    /// Get current user information
    pub fn get_user(&mut self) -> Result<Option<FacebookUser>, FacebookError> {
        let user_info = self.api("/me", "GET", Some(&[
            ("fields", "id,name,email,first_name,last_name,picture,locale,timezone,verified,link")
        ]))?;

        if let Some(user_data) = user_info.data {
            let user: FacebookUser = serde_json::from_value(user_data)?;
            self.user_id = Some(user.id.clone());
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    /// Get user by ID
    pub fn get_user_by_id(&self, user_id: &str, fields: Option<&[&str]>) -> Result<Option<FacebookUser>, FacebookError> {
        let fields_str = fields
            .map(|f| f.join(","))
            .unwrap_or_else(|| "id,name,email".to_string());

        let user_info = self.api(
            &format!("/{}", user_id),
            "GET",
            Some(&[("fields", &fields_str)])
        )?;

        if let Some(user_data) = user_info.data {
            let user: FacebookUser = serde_json::from_value(user_data)?;
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    /// Make API call to Facebook Graph API
    pub fn api(&self, path: &str, method: &str, params: Option<&[(&str, &str)]>) -> Result<FacebookApiResponse<serde_json::Value>, FacebookError> {
        // Validate access token
        let access_token = self.access_token
            .as_ref()
            .ok_or(FacebookError::InvalidToken)?;

        // Check token expiration
        if let Some(expires_at) = access_token.expires_at {
            if expires_at <= Utc::now() {
                return Err(FacebookError::TokenExpired);
            }
        }

        let url = format!("https://graph.facebook.com{}", path);
        
        // In real implementation, make actual HTTP request
        // For now, simulate API response
        let mock_response = match path {
            "/me" => {
                r#"{
                    "data": {
                        "id": "12345678901234567",
                        "name": "John Doe",
                        "email": "john.doe@example.com",
                        "first_name": "John",
                        "last_name": "Doe",
                        "picture": {
                            "data": {
                                "height": 50,
                                "width": 50,
                                "url": "https://platform-lookaside.fbsbx.com/platform/profilepic/?asid=12345678901234567&height=50&width=50",
                                "is_silhouette": false
                            }
                        },
                        "verified": true
                    }
                }"#
            }
            _ => {
                r#"{
                    "data": {}
                }"#
            }
        };

        let response: FacebookApiResponse<serde_json::Value> = serde_json::from_str(mock_response)?;
        
        if let Some(error) = &response.error {
            return Err(FacebookError::Api(format!("API Error {}: {}", error.code, error.message)));
        }

        Ok(response)
    }

    /// Get application access token
    pub fn get_app_access_token(&self) -> String {
        format!("{}|{}", self.config.app_id, self.config.app_secret)
    }

    /// Validate access token
    pub fn validate_access_token(&self, token: &str) -> Result<bool, FacebookError> {
        let url = format!(
            "https://graph.facebook.com/{}/debug_token",
            self.config.api_version
        );

        // In real implementation, make HTTP request to validate token
        // For now, simulate validation
        if token.is_empty() || token == "invalid_token" {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    /// Extend short-lived token to long-lived token
    pub fn extend_access_token(&mut self) -> Result<AccessToken, FacebookError> {
        let current_token = self.access_token
            .as_ref()
            .ok_or(FacebookError::InvalidToken)?;

        let params = [
            ("grant_type", "fb_exchange_token"),
            ("client_id", self.config.app_id.as_str()),
            ("client_secret", self.config.app_secret.as_str()),
            ("fb_exchange_token", current_token.token.as_str()),
        ];

        // In real implementation, make HTTP request
        // For now, simulate extended token
        let extended_token = AccessToken {
            token: format!("extended_{}", current_token.token),
            token_type: "bearer".to_string(),
            expires_in: Some(5184000), // 60 days
            expires_at: Some(Utc::now() + chrono::Duration::days(60)),
            scope: current_token.scope.clone(),
        };

        self.access_token = Some(extended_token.clone());
        Ok(extended_token)
    }

    /// Parse signed request
    pub fn parse_signed_request(&self, signed_request: &str) -> Result<serde_json::Value, FacebookError> {
        if signed_request.is_empty() {
            return Err(FacebookError::Auth("Empty signed request".to_string()));
        }

        let parts: Vec<&str> = signed_request.split('.').collect();
        if parts.len() != 2 {
            return Err(FacebookError::Auth("Invalid signed request format".to_string()));
        }

        // In real implementation, decode base64 and verify signature
        // For now, simulate parsed data
        let mock_data = serde_json::json!({
            "algorithm": "HMAC-SHA256",
            "issued_at": Utc::now().timestamp(),
            "user_id": "12345678901234567",
            "oauth_token": "mock_oauth_token"
        });

        Ok(mock_data)
    }

    /// Get user permissions
    pub fn get_user_permissions(&self, user_id: Option<&str>) -> Result<Vec<String>, FacebookError> {
        let uid = user_id.unwrap_or("me");
        let response = self.api(&format!("/{}/permissions", uid), "GET", None)?;

        // Parse permissions from response
        // For now, return mock permissions
        Ok(vec![
            "public_profile".to_string(),
            "email".to_string(),
        ])
    }

    /// Post to user's timeline (requires publish_actions permission)
    pub fn post_to_timeline(&self, message: &str, link: Option<&str>) -> Result<String, FacebookError> {
        let mut params = vec![("message", message)];
        
        if let Some(url) = link {
            params.push(("link", url));
        }

        let response = self.api("/me/feed", "POST", Some(&params))?;
        
        // Extract post ID from response
        if let Some(data) = response.data {
            if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                Ok(id.to_string())
            } else {
                Err(FacebookError::Api("No post ID in response".to_string()))
            }
        } else {
            Err(FacebookError::Api("Empty response".to_string()))
        }
    }

    /// Get user's friends (limited in newer API versions)
    pub fn get_friends(&self, user_id: Option<&str>) -> Result<Vec<FacebookUser>, FacebookError> {
        let uid = user_id.unwrap_or("me");
        let response = self.api(&format!("/{}/friends", uid), "GET", None)?;

        // In newer API versions, this only returns friends who also use the app
        // For now, return empty list as this is the typical case
        Ok(vec![])
    }

    /// Helper methods
    fn generate_state(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow::anyhow!("Error: {}", e))?
            .as_nanos();
        
        format!("fb_state_{}", timestamp)
    }

    /// Get app information
    pub fn get_app_info(&self) -> Result<serde_json::Value, FacebookError> {
        let app_token = self.get_app_access_token();
        
        // In real implementation, make API call with app token
        // For now, return mock app info
        let mock_info = serde_json::json!({
            "id": self.config.app_id,
            "name": "Hacker Experience",
            "category": "Games",
            "description": "A realistic hacking simulation game",
            "link": "https://hackerexperience.com"
        });

        Ok(mock_info)
    }

    /// Delete user data (GDPR compliance)
    pub fn delete_user_data(&self, user_id: &str) -> Result<bool, FacebookError> {
        // In real implementation, this would make an API call to delete user data
        // This is required for GDPR compliance
        Ok(true)
    }
}

impl Default for LoginUrlParams {
    fn default() -> Self {
        Self {
            redirect_uri: None,
            scope: None,
            display: None,
            state: None,
            response_type: Some("code".to_string()),
        }
    }
}

/// Utility functions for Facebook integration
pub mod utils {
    use super::*;

    /// Extract user ID from Facebook user URL
    pub fn extract_user_id_from_url(url: &str) -> Option<String> {
        // Extract numeric user ID from Facebook profile URL
        if let Some(start) = url.find("facebook.com/") {
            let rest = &url[start + 13..];
            if let Some(end) = rest.find('/') {
                Some(rest[..end].to_string())
            } else {
                Some(rest.to_string())
            }
        } else {
            None
        }
    }

    /// Format Facebook profile picture URL with specific size
    pub fn format_picture_url(user_id: &str, width: u32, height: u32) -> String {
        format!(
            "https://graph.facebook.com/{}/picture?width={}&height={}",
            user_id, width, height
        )
    }

    /// Check if permission is granted
    pub fn has_permission(permissions: &[String], permission: &str) -> bool {
        permissions.iter().any(|p| p == permission)
    }

    /// Validate Facebook user ID format
    pub fn is_valid_user_id(user_id: &str) -> bool {
        // Facebook user IDs are typically 15-17 digit numbers
        user_id.chars().all(|c| c.is_numeric()) && user_id.len() >= 15 && user_id.len() <= 17
    }
}

// Simple URL encoding for parameters
mod urlencoding {
    pub fn encode(input: &str) -> String {
        // Simple URL encoding implementation
        input
            .chars()
            .map(|c| match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                ' ' => "+".to_string(),
                _ => format!("%{:02X}", c as u8),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_facebook_creation() {
        let config = FacebookConfig {
            app_id: "test_app_id".to_string(),
            app_secret: "test_app_secret".to_string(),
            ..Default::default()
        };

        let facebook = Facebook::new(config);
        assert!(facebook.is_ok());
    }

    #[test]
    fn test_facebook_invalid_config() {
        let config = FacebookConfig {
            app_id: "".to_string(),
            app_secret: "".to_string(),
            ..Default::default()
        };

        let facebook = Facebook::new(config);
        assert!(facebook.is_err());
        assert!(matches!(facebook.unwrap_err(), FacebookError::InvalidAppConfig));
    }

    #[test]
    fn test_with_app_credentials() {
        let facebook = Facebook::with_app_credentials(
            "test_app_id".to_string(),
            "test_app_secret".to_string(),
        );
        assert!(facebook.is_ok());
    }

    #[test]
    fn test_generate_state() {
        let config = FacebookConfig {
            app_id: "test_app_id".to_string(),
            app_secret: "test_app_secret".to_string(),
            ..Default::default()
        };
        let facebook = Facebook::new(config).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        let state1 = facebook.generate_state();
        let state2 = facebook.generate_state();
        
        assert_ne!(state1, state2);
        assert!(state1.starts_with("fb_state_"));
    }

    #[test]
    fn test_get_login_url() {
        let config = FacebookConfig {
            app_id: "123456789".to_string(),
            app_secret: "test_secret".to_string(),
            redirect_uri: "https://example.com/callback".to_string(),
            ..Default::default()
        };
        let facebook = Facebook::new(config).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        let login_url = facebook.get_login_url(None);
        
        assert!(login_url.contains("client_id=123456789"));
        assert!(login_url.contains("redirect_uri="));
        assert!(login_url.contains("response_type=code"));
        assert!(login_url.contains("facebook.com"));
    }

    #[test]
    fn test_get_app_access_token() {
        let config = FacebookConfig {
            app_id: "123456789".to_string(),
            app_secret: "secret123".to_string(),
            ..Default::default()
        };
        let facebook = Facebook::new(config).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        let app_token = facebook.get_app_access_token();
        assert_eq!(app_token, "123456789|secret123");
    }

    #[test]
    fn test_access_token_serialization() {
        let token = AccessToken {
            token: "test_token".to_string(),
            token_type: "bearer".to_string(),
            expires_in: Some(3600),
            expires_at: Some(Utc::now()),
            scope: Some("public_profile,email".to_string()),
        };

        let json = serde_json::to_string(&token).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let deserialized: AccessToken = serde_json::from_str(&json).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        assert_eq!(token.token, deserialized.token);
        assert_eq!(token.token_type, deserialized.token_type);
        assert_eq!(token.expires_in, deserialized.expires_in);
    }

    #[test]
    fn test_utils_extract_user_id() {
        let url1 = "https://www.facebook.com/12345678901234567";
        let url2 = "https://facebook.com/john.doe/";
        
        assert_eq!(utils::extract_user_id_from_url(url1), Some("12345678901234567".to_string()));
        assert_eq!(utils::extract_user_id_from_url(url2), Some("john.doe".to_string()));
        assert_eq!(utils::extract_user_id_from_url("invalid"), None);
    }

    #[test]
    fn test_utils_format_picture_url() {
        let url = utils::format_picture_url("12345678901234567", 100, 100);
        assert_eq!(url, "https://graph.facebook.com/12345678901234567/picture?width=100&height=100");
    }

    #[test]
    fn test_utils_has_permission() {
        let permissions = vec!["public_profile".to_string(), "email".to_string()];
        
        assert!(utils::has_permission(&permissions, "email"));
        assert!(!utils::has_permission(&permissions, "publish_actions"));
    }

    #[test]
    fn test_utils_is_valid_user_id() {
        assert!(utils::is_valid_user_id("12345678901234567"));
        assert!(!utils::is_valid_user_id("123")); // Too short
        assert!(!utils::is_valid_user_id("123456789012345678")); // Too long
        assert!(!utils::is_valid_user_id("12345678901234abc")); // Contains letters
    }

    #[test]
    fn test_validate_access_token() {
        let config = FacebookConfig {
            app_id: "test_app_id".to_string(),
            app_secret: "test_app_secret".to_string(),
            ..Default::default()
        };
        let facebook = Facebook::new(config).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        assert!(facebook.validate_access_token("valid_token").map_err(|e| anyhow::anyhow!("Error: {}", e))?);
        assert!(!facebook.validate_access_token("invalid_token").map_err(|e| anyhow::anyhow!("Error: {}", e))?);
        assert!(!facebook.validate_access_token("").map_err(|e| anyhow::anyhow!("Error: {}", e))?);
    }
}