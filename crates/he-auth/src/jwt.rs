//! JWT token management

use anyhow::{anyhow, Result};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error};
use uuid::Uuid;

/// JWT configuration
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// Secret key for JWT signing
    pub secret: String,
    /// Token expiration time in seconds
    pub expiration_seconds: u64,
    /// JWT algorithm to use
    pub algorithm: Algorithm,
    /// Token issuer
    pub issuer: Option<String>,
    /// Token audience
    pub audience: Option<String>,
    /// Allow token refresh
    pub allow_refresh: bool,
    /// Refresh token expiration in seconds
    pub refresh_expiration_seconds: u64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        // Load JWT secret from environment variable, panic if not set in production
        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
            // Only allow default in development/test environments
            if cfg!(debug_assertions) || cfg!(test) {
                eprintln!("WARNING: JWT_SECRET not set, using insecure default. This is only acceptable in development!");
                // Generate a random secret for development
                "INSECURE_DEV_SECRET_DO_NOT_USE_IN_PRODUCTION_MUST_BE_32_CHARS".to_string()
            } else {
                // Panic in production if JWT_SECRET is not set
                panic!("JWT_SECRET environment variable must be set in production!");
            }
        });

        Self {
            secret,
            expiration_seconds: 3600, // 1 hour
            algorithm: Algorithm::HS256,
            issuer: Some("HackerExperience".to_string()),
            audience: Some("HackerExperience-Users".to_string()),
            allow_refresh: true,
            refresh_expiration_seconds: 86400 * 7, // 1 week
        }
    }
}

impl JwtConfig {
    /// Create JWT configuration from environment variables
    pub fn from_env() -> Self {
        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
            if cfg!(debug_assertions) || cfg!(test) {
                eprintln!("WARNING: JWT_SECRET not set, using insecure default");
                "INSECURE_DEV_SECRET_DO_NOT_USE_IN_PRODUCTION_MUST_BE_32_CHARS".to_string()
            } else {
                panic!("JWT_SECRET environment variable is required in production");
            }
        });

        let expiration_seconds = std::env::var("JWT_EXPIRATION_SECONDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3600);

        let refresh_expiration_seconds = std::env::var("JWT_REFRESH_EXPIRATION_SECONDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(86400 * 7);

        let issuer = std::env::var("JWT_ISSUER")
            .ok()
            .or_else(|| Some("HackerExperience".to_string()));

        let audience = std::env::var("JWT_AUDIENCE")
            .ok()
            .or_else(|| Some("HackerExperience-Users".to_string()));

        Self {
            secret,
            expiration_seconds,
            algorithm: Algorithm::HS256,
            issuer,
            audience,
            allow_refresh: true,
            refresh_expiration_seconds,
        }
    }
}

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// User ID
    pub user_id: Uuid,
    /// User email
    pub email: String,
    /// User roles
    pub roles: Vec<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Expiration time (as UTC timestamp)
    pub exp: usize,
    /// Issued at (as UTC timestamp)
    pub iat: usize,
    /// Issuer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    /// Audience
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    /// Subject (user identifier)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    /// JWT ID (unique identifier for the token)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
}

/// JWT refresh claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshClaims {
    /// User ID
    pub user_id: Uuid,
    /// Token type (always "refresh")
    pub token_type: String,
    /// Expiration time (as UTC timestamp)
    pub exp: usize,
    /// Issued at (as UTC timestamp)
    pub iat: usize,
    /// JWT ID (unique identifier for the refresh token)
    pub jti: String,
}

/// JWT manager for token operations
#[derive(Debug)]
pub struct JwtManager {
    config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtManager {
    /// Create a new JWT manager
    pub fn new(config: JwtConfig) -> Result<Self> {
        if config.secret.len() < 32 {
            return Err(anyhow!("JWT secret must be at least 32 characters long"));
        }

        let encoding_key = EncodingKey::from_secret(config.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());

        let mut validation = Validation::new(config.algorithm);
        validation.set_issuer(&[config.issuer.as_ref().unwrap_or(&"HackerExperience".to_string())]);
        validation.set_audience(&[config.audience.as_ref().unwrap_or(&"HackerExperience-Users".to_string())]);

        Ok(Self {
            config,
            encoding_key,
            decoding_key,
            validation,
        })
    }

    /// Generate a new access token
    pub fn generate_token(&self, claims: &JwtClaims) -> Result<String> {
        let mut token_claims = claims.clone();
        
        // Set standard claims
        token_claims.iss = self.config.issuer.clone();
        token_claims.aud = self.config.audience.clone();
        token_claims.sub = Some(token_claims.user_id.to_string());
        token_claims.jti = Some(Uuid::new_v4().to_string());

        let header = Header::new(self.config.algorithm);
        
        let token = encode(&header, &token_claims, &self.encoding_key)
            .map_err(|e| anyhow!("Failed to encode JWT token: {}", e))?;

        debug!("Generated JWT token for user: {}", claims.user_id);
        Ok(token)
    }

    /// Generate a refresh token
    pub fn generate_refresh_token(&self, user_id: Uuid) -> Result<String> {
        if !self.config.allow_refresh {
            return Err(anyhow!("Refresh tokens are disabled"));
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        let claims = RefreshClaims {
            user_id,
            token_type: "refresh".to_string(),
            exp: now + self.config.refresh_expiration_seconds as usize,
            iat: now,
            jti: Uuid::new_v4().to_string(),
        };

        let header = Header::new(self.config.algorithm);
        
        let token = encode(&header, &claims, &self.encoding_key)
            .map_err(|e| anyhow!("Failed to encode refresh token: {}", e))?;

        debug!("Generated refresh token for user: {}", user_id);
        Ok(token)
    }

    /// Validate and decode a JWT token
    pub fn validate_token(&self, token: &str) -> Result<JwtClaims> {
        let token_data = decode::<JwtClaims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| {
                error!("Failed to decode JWT token: {}", e);
                anyhow!("Invalid token: {}", e)
            })?;

        let claims = token_data.claims;

        // Additional validation
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        if claims.exp < now {
            return Err(anyhow!("Token has expired"));
        }

        debug!("Validated JWT token for user: {}", claims.user_id);
        Ok(claims)
    }

    /// Validate and decode a refresh token
    pub fn validate_refresh_token(&self, token: &str) -> Result<RefreshClaims> {
        if !self.config.allow_refresh {
            return Err(anyhow!("Refresh tokens are disabled"));
        }

        // Use minimal validation for refresh tokens
        let mut validation = Validation::new(self.config.algorithm);
        validation.validate_exp = true;
        validation.validate_nbf = false;
        validation.set_issuer(&[]);
        validation.set_audience(&[]);

        let token_data = decode::<RefreshClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| anyhow!("Invalid refresh token: {}", e))?;

        let claims = token_data.claims;

        if claims.token_type != "refresh" {
            return Err(anyhow!("Invalid token type"));
        }

        debug!("Validated refresh token for user: {}", claims.user_id);
        Ok(claims)
    }

    /// Create a new access token from a refresh token
    pub fn refresh_access_token(&self, refresh_token: &str) -> Result<(String, JwtClaims)> {
        let refresh_claims = self.validate_refresh_token(refresh_token)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        // TODO: Get user roles and other info from database
        let access_claims = JwtClaims {
            user_id: refresh_claims.user_id,
            email: "placeholder@example.com".to_string(), // Would get from database
            roles: vec!["player".to_string()], // Would get from database
            session_id: None,
            exp: now + self.config.expiration_seconds as usize,
            iat: now,
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            sub: Some(refresh_claims.user_id.to_string()),
            jti: Some(Uuid::new_v4().to_string()),
        };

        let access_token = self.generate_token(&access_claims)?;

        debug!("Refreshed access token for user: {}", refresh_claims.user_id);
        Ok((access_token, access_claims))
    }

    /// Extract claims from token without validation (for debugging)
    pub fn decode_token_unsafe(&self, token: &str) -> Result<JwtClaims> {
        let mut validation = Validation::new(self.config.algorithm);
        validation.validate_exp = false;
        validation.set_issuer(&[]);
        validation.set_audience(&[]);

        let token_data = decode::<JwtClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| anyhow!("Failed to decode token: {}", e))?;

        Ok(token_data.claims)
    }

    /// Get token expiration time from claims
    pub fn get_token_expiration(&self, token: &str) -> Result<SystemTime> {
        let claims = self.decode_token_unsafe(token)?;
        let exp_time = UNIX_EPOCH + std::time::Duration::from_secs(claims.exp as u64);
        Ok(exp_time)
    }

    /// Check if token is expired
    pub fn is_token_expired(&self, token: &str) -> bool {
        match self.get_token_expiration(token) {
            Ok(exp_time) => exp_time < SystemTime::now(),
            Err(_) => true,
        }
    }

    /// Get remaining token lifetime
    pub fn get_token_lifetime(&self, token: &str) -> Result<std::time::Duration> {
        let exp_time = self.get_token_expiration(token)?;
        let now = SystemTime::now();

        if exp_time > now {
            Ok(exp_time.duration_since(now)?)
        } else {
            Ok(std::time::Duration::from_secs(0))
        }
    }
}

/// Token pair (access + refresh tokens)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

impl JwtManager {
    /// Generate both access and refresh tokens
    pub fn generate_token_pair(&self, claims: &JwtClaims) -> Result<TokenPair> {
        let access_token = self.generate_token(claims)?;
        let refresh_token = self.generate_refresh_token(claims.user_id)?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.expiration_seconds,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    fn create_test_jwt_manager() -> JwtManager {
        let config = JwtConfig {
            secret: "test-secret-key-that-is-long-enough-for-security".to_string(),
            expiration_seconds: 1,
            ..Default::default()
        };
        JwtManager::new(config).unwrap()
    }

    fn create_test_claims() -> JwtClaims {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        JwtClaims {
            user_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            roles: vec!["player".to_string()],
            session_id: Some(Uuid::new_v4().to_string()),
            exp: now + 3600,
            iat: now,
            iss: None,
            aud: None,
            sub: None,
            jti: None,
        }
    }

    #[test]
    fn test_jwt_manager_creation() {
        let config = JwtConfig::default();
        let result = JwtManager::new(config);
        assert!(result.is_err()); // Should fail due to short secret

        let config = JwtConfig {
            secret: "a-very-long-secret-key-for-testing-purposes".to_string(),
            ..Default::default()
        };
        let result = JwtManager::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_token_generation_and_validation() {
        let jwt_manager = create_test_jwt_manager();
        let claims = create_test_claims();

        let token = jwt_manager.generate_token(&claims).unwrap();
        assert!(!token.is_empty());

        let validated_claims = jwt_manager.validate_token(&token).unwrap();
        assert_eq!(validated_claims.user_id, claims.user_id);
        assert_eq!(validated_claims.email, claims.email);
        assert_eq!(validated_claims.roles, claims.roles);
    }

    #[test]
    fn test_token_expiration() {
        let jwt_manager = create_test_jwt_manager();
        let claims = create_test_claims();

        let token = jwt_manager.generate_token(&claims).unwrap();
        assert!(!jwt_manager.is_token_expired(&token));

        // Wait for token to expire
        thread::sleep(Duration::from_secs(2));
        assert!(jwt_manager.is_token_expired(&token));
    }

    #[test]
    fn test_refresh_token() {
        let config = JwtConfig {
            secret: "a-very-long-secret-key-for-testing-purposes".to_string(),
            allow_refresh: true,
            ..Default::default()
        };
        let jwt_manager = JwtManager::new(config).unwrap();

        let user_id = Uuid::new_v4();
        let refresh_token = jwt_manager.generate_refresh_token(user_id).unwrap();
        assert!(!refresh_token.is_empty());

        let refresh_claims = jwt_manager.validate_refresh_token(&refresh_token).unwrap();
        assert_eq!(refresh_claims.user_id, user_id);
        assert_eq!(refresh_claims.token_type, "refresh");
    }

    #[test]
    fn test_token_pair_generation() {
        let config = JwtConfig {
            secret: "a-very-long-secret-key-for-testing-purposes".to_string(),
            allow_refresh: true,
            ..Default::default()
        };
        let jwt_manager = JwtManager::new(config).unwrap();
        let claims = create_test_claims();

        let token_pair = jwt_manager.generate_token_pair(&claims).unwrap();
        assert!(!token_pair.access_token.is_empty());
        assert!(!token_pair.refresh_token.is_empty());
        assert_eq!(token_pair.token_type, "Bearer");
    }
}