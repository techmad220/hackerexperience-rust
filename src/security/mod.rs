use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Duration, Utc};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use regex::Regex;
use lazy_static::lazy_static;

pub mod rate_limiter;
pub mod csrf;
pub mod input_validator;
pub mod encryption;

use rate_limiter::RateLimiter;
use csrf::CsrfProtection;
use input_validator::InputValidator;
use encryption::DataEncryption;

/// Security configuration
#[derive(Clone)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub refresh_token_expiration: i64,
    pub max_login_attempts: u32,
    pub lockout_duration: i64,
    pub password_min_length: usize,
    pub require_2fa: bool,
    pub session_timeout: i64,
    pub csrf_token_length: usize,
    pub rate_limit_requests: u32,
    pub rate_limit_window: i64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "change_me_in_production".to_string()),
            jwt_expiration: 3600, // 1 hour
            refresh_token_expiration: 86400 * 7, // 7 days
            max_login_attempts: 5,
            lockout_duration: 900, // 15 minutes
            password_min_length: 12,
            require_2fa: false,
            session_timeout: 1800, // 30 minutes
            csrf_token_length: 32,
            rate_limit_requests: 100,
            rate_limit_window: 60, // 1 minute
        }
    }
}

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub username: String,
    pub roles: Vec<String>,
    pub exp: usize,
    pub iat: usize,
    pub session_id: String,
    pub ip_address: Option<String>,
}

/// Security manager
pub struct SecurityManager {
    config: SecurityConfig,
    rate_limiter: Arc<RateLimiter>,
    csrf_protection: Arc<CsrfProtection>,
    input_validator: Arc<InputValidator>,
    encryption: Arc<DataEncryption>,
    failed_attempts: Arc<RwLock<HashMap<String, (u32, DateTime<Utc>)>>>,
    active_sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
}

#[derive(Clone, Debug)]
pub struct SessionInfo {
    pub user_id: String,
    pub username: String,
    pub ip_address: String,
    pub user_agent: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub is_2fa_verified: bool,
}

impl SecurityManager {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config: config.clone(),
            rate_limiter: Arc::new(RateLimiter::new(
                config.rate_limit_requests,
                config.rate_limit_window,
            )),
            csrf_protection: Arc::new(CsrfProtection::new(config.csrf_token_length)),
            input_validator: Arc::new(InputValidator::new()),
            encryption: Arc::new(DataEncryption::new()),
            failed_attempts: Arc::new(RwLock::new(HashMap::new())),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Hash password using Argon2
    pub async fn hash_password(&self, password: &str) -> Result<String, Error> {
        // Validate password strength
        self.validate_password_strength(password)?;

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?
            .to_string();

        Ok(password_hash)
    }

    /// Verify password
    pub async fn verify_password(&self, password: &str, hash: &str) -> Result<bool, Error> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

        let argon2 = Argon2::default();
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

    /// Validate password strength
    fn validate_password_strength(&self, password: &str) -> Result<(), Error> {
        if password.len() < self.config.password_min_length {
            return Err(actix_web::error::ErrorBadRequest(
                format!("Password must be at least {} characters", self.config.password_min_length)
            ));
        }

        lazy_static! {
            static ref UPPERCASE: Regex = Regex::new(r"[A-Z]").unwrap();
            static ref LOWERCASE: Regex = Regex::new(r"[a-z]").unwrap();
            static ref DIGIT: Regex = Regex::new(r"\d").unwrap();
            static ref SPECIAL: Regex = Regex::new(r"[!@#$%^&*(),.?\":{}|<>]").unwrap();
        }

        let mut strength = 0;
        if UPPERCASE.is_match(password) { strength += 1; }
        if LOWERCASE.is_match(password) { strength += 1; }
        if DIGIT.is_match(password) { strength += 1; }
        if SPECIAL.is_match(password) { strength += 1; }

        if strength < 3 {
            return Err(actix_web::error::ErrorBadRequest(
                "Password must contain at least 3 of: uppercase, lowercase, digit, special character"
            ));
        }

        // Check for common passwords
        if self.is_common_password(password) {
            return Err(actix_web::error::ErrorBadRequest(
                "Password is too common. Please choose a stronger password"
            ));
        }

        Ok(())
    }

    /// Check if password is in common passwords list
    fn is_common_password(&self, password: &str) -> bool {
        let common_passwords = vec![
            "password123", "admin123", "letmein", "qwerty123",
            "welcome123", "monkey123", "dragon123", "master123"
        ];

        let lower_password = password.to_lowercase();
        common_passwords.iter().any(|&p| lower_password.contains(p))
    }

    /// Generate JWT token
    pub async fn generate_jwt(
        &self,
        user_id: &str,
        username: &str,
        roles: Vec<String>,
        ip_address: Option<String>
    ) -> Result<String, Error> {
        let now = Utc::now();
        let session_id = uuid::Uuid::new_v4().to_string();

        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            roles,
            exp: (now + Duration::seconds(self.config.jwt_expiration)).timestamp() as usize,
            iat: now.timestamp() as usize,
            session_id: session_id.clone(),
            ip_address: ip_address.clone(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

        // Store session info
        if let Some(ip) = ip_address {
            let session_info = SessionInfo {
                user_id: user_id.to_string(),
                username: username.to_string(),
                ip_address: ip,
                user_agent: String::new(),
                created_at: now,
                last_activity: now,
                is_2fa_verified: false,
            };

            let mut sessions = self.active_sessions.write().await;
            sessions.insert(session_id, session_info);
        }

        Ok(token)
    }

    /// Validate JWT token
    pub async fn validate_jwt(&self, token: &str) -> Result<Claims, Error> {
        let validation = Validation::default();

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            &validation,
        )
        .map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

        // Check if session is still active
        let sessions = self.active_sessions.read().await;
        if let Some(session) = sessions.get(&token_data.claims.session_id) {
            // Check session timeout
            let now = Utc::now();
            if (now - session.last_activity).num_seconds() > self.config.session_timeout {
                return Err(actix_web::error::ErrorUnauthorized("Session expired"));
            }

            // Validate IP address if present
            if let Some(ref token_ip) = token_data.claims.ip_address {
                if &session.ip_address != token_ip {
                    return Err(actix_web::error::ErrorUnauthorized("IP address mismatch"));
                }
            }
        } else {
            return Err(actix_web::error::ErrorUnauthorized("Invalid session"));
        }

        Ok(token_data.claims)
    }

    /// Check login attempts
    pub async fn check_login_attempts(&self, identifier: &str) -> Result<(), Error> {
        let mut attempts = self.failed_attempts.write().await;
        let now = Utc::now();

        if let Some((count, first_attempt)) = attempts.get(identifier) {
            // Check if lockout period has passed
            if (now - *first_attempt).num_seconds() < self.config.lockout_duration {
                if *count >= self.config.max_login_attempts {
                    return Err(actix_web::error::ErrorTooManyRequests(
                        format!("Account locked. Try again in {} minutes",
                            (self.config.lockout_duration - (now - *first_attempt).num_seconds()) / 60)
                    ));
                }
            } else {
                // Reset attempts after lockout period
                attempts.remove(identifier);
            }
        }

        Ok(())
    }

    /// Record failed login attempt
    pub async fn record_failed_attempt(&self, identifier: &str) {
        let mut attempts = self.failed_attempts.write().await;
        let now = Utc::now();

        let (count, first_attempt) = attempts
            .entry(identifier.to_string())
            .or_insert((0, now));

        if (now - *first_attempt).num_seconds() > self.config.lockout_duration {
            // Reset counter after lockout period
            *count = 1;
            *first_attempt = now;
        } else {
            *count += 1;
        }
    }

    /// Clear failed attempts
    pub async fn clear_failed_attempts(&self, identifier: &str) {
        let mut attempts = self.failed_attempts.write().await;
        attempts.remove(identifier);
    }

    /// Update session activity
    pub async fn update_session_activity(&self, session_id: &str) {
        let mut sessions = self.active_sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_activity = Utc::now();
        }
    }

    /// Invalidate session
    pub async fn invalidate_session(&self, session_id: &str) {
        let mut sessions = self.active_sessions.write().await;
        sessions.remove(session_id);
    }

    /// Clean expired sessions
    pub async fn clean_expired_sessions(&self) {
        let mut sessions = self.active_sessions.write().await;
        let now = Utc::now();

        sessions.retain(|_, session| {
            (now - session.last_activity).num_seconds() < self.config.session_timeout
        });
    }

    /// Sanitize input to prevent XSS
    pub fn sanitize_input(&self, input: &str) -> String {
        self.input_validator.sanitize_html(input)
    }

    /// Validate email format
    pub fn validate_email(&self, email: &str) -> Result<(), Error> {
        self.input_validator.validate_email(email)
    }

    /// Generate CSRF token
    pub async fn generate_csrf_token(&self) -> String {
        self.csrf_protection.generate_token()
    }

    /// Validate CSRF token
    pub async fn validate_csrf_token(&self, token: &str, session_id: &str) -> Result<(), Error> {
        self.csrf_protection.validate_token(token, session_id)
    }

    /// Check rate limit
    pub async fn check_rate_limit(&self, identifier: &str) -> Result<(), Error> {
        self.rate_limiter.check_limit(identifier).await
    }

    /// Encrypt sensitive data
    pub async fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        self.encryption.encrypt(data)
    }

    /// Decrypt sensitive data
    pub async fn decrypt_data(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, Error> {
        self.encryption.decrypt(encrypted_data)
    }
}

/// Middleware for JWT authentication
pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let config = req
        .app_data::<SecurityConfig>()
        .cloned()
        .unwrap_or_default();

    let security_manager = SecurityManager::new(config);

    match security_manager.validate_jwt(credentials.token()).await {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            Ok(req)
        }
        Err(e) => Err((e, req)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_password_hashing() {
        let config = SecurityConfig::default();
        let security = SecurityManager::new(config);

        let password = "SecureP@ssw0rd123!";
        let hash = security.hash_password(password).await.unwrap();

        assert!(security.verify_password(password, &hash).await.unwrap());
        assert!(!security.verify_password("WrongPassword", &hash).await.unwrap());
    }

    #[tokio::test]
    async fn test_password_strength_validation() {
        let config = SecurityConfig::default();
        let security = SecurityManager::new(config);

        // Weak passwords should fail
        assert!(security.hash_password("weak").await.is_err());
        assert!(security.hash_password("password123").await.is_err());

        // Strong password should pass
        assert!(security.hash_password("Str0ng!P@ssw0rd#2024").await.is_ok());
    }

    #[tokio::test]
    async fn test_jwt_generation_and_validation() {
        let config = SecurityConfig::default();
        let security = SecurityManager::new(config);

        let token = security.generate_jwt(
            "user123",
            "testuser",
            vec!["user".to_string()],
            Some("127.0.0.1".to_string())
        ).await.unwrap();

        let claims = security.validate_jwt(&token).await.unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.username, "testuser");
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let config = SecurityConfig {
            rate_limit_requests: 3,
            rate_limit_window: 1,
            ..Default::default()
        };
        let security = SecurityManager::new(config);

        let identifier = "test_user";

        // First 3 requests should pass
        for _ in 0..3 {
            assert!(security.check_rate_limit(identifier).await.is_ok());
        }

        // 4th request should be rate limited
        assert!(security.check_rate_limit(identifier).await.is_err());
    }

    #[tokio::test]
    async fn test_login_attempt_tracking() {
        let config = SecurityConfig {
            max_login_attempts: 3,
            lockout_duration: 60,
            ..Default::default()
        };
        let security = SecurityManager::new(config);

        let identifier = "test@example.com";

        // Check initial state
        assert!(security.check_login_attempts(identifier).await.is_ok());

        // Record failed attempts
        for _ in 0..2 {
            security.record_failed_attempt(identifier).await;
            assert!(security.check_login_attempts(identifier).await.is_ok());
        }

        // 3rd failed attempt should trigger lockout
        security.record_failed_attempt(identifier).await;
        assert!(security.check_login_attempts(identifier).await.is_err());

        // Clear attempts should reset
        security.clear_failed_attempts(identifier).await;
        assert!(security.check_login_attempts(identifier).await.is_ok());
    }
}