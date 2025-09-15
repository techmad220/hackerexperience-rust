use axum::{
    extract::{Form, Query, State},
    http::{header, StatusCode, HeaderValue},
    response::{Json, Redirect},
    routing::{get, post},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{sync::RwLock, time::sleep};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use rand_core::OsRng;
use tower::ServiceExt;
use axum::body::Body;
use axum::http::Request;

use crate::common::{TestDb, TestFixtures, MockHttpClient};
use crate::{assert_ok, assert_err};

// ===== AUTHENTICATION MODELS =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: u64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub failed_login_attempts: u32,
    pub locked_until: Option<DateTime<Utc>>,
    pub two_factor_enabled: bool,
    pub two_factor_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: Uuid,
    pub user_id: u64,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String, // Subject (user_id)
    pub username: String,
    pub exp: usize,  // Expiration time
    pub iat: usize,  // Issued at
    pub jti: String, // JWT ID (session_id)
}

// Request/Response structures
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
    pub terms_accepted: bool,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub remember_me: Option<bool>,
    pub totp_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
    pub confirm_new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub reset_token: String,
    pub new_password: String,
    pub confirm_new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct Enable2FARequest {
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct Verify2FARequest {
    pub totp_code: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
    pub errors: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user_id: u64,
    pub username: String,
    pub session_id: String,
    pub jwt_token: String,
    pub expires_at: DateTime<Utc>,
    pub requires_2fa: bool,
}

#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub user_id: u64,
    pub username: String,
    pub email: String,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub two_factor_enabled: bool,
}

// ===== AUTHENTICATION SERVICE =====

pub struct AuthService {
    users: Arc<RwLock<HashMap<u64, User>>>,
    sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
    username_to_id: Arc<RwLock<HashMap<String, u64>>>,
    email_to_id: Arc<RwLock<HashMap<String, u64>>>,
    reset_tokens: Arc<RwLock<HashMap<String, (u64, DateTime<Utc>)>>>,
    jwt_secret: String,
    next_user_id: Arc<RwLock<u64>>,
}

impl AuthService {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            username_to_id: Arc::new(RwLock::new(HashMap::new())),
            email_to_id: Arc::new(RwLock::new(HashMap::new())),
            reset_tokens: Arc::new(RwLock::new(HashMap::new())),
            jwt_secret: "test_jwt_secret_key_for_testing_only".to_string(),
            next_user_id: Arc::new(RwLock::new(1)),
        }
    }

    pub async fn register_user(&self, request: RegisterRequest) -> Result<u64, Vec<String>> {
        let mut errors = Vec::new();

        // Validation
        if request.username.len() < 3 {
            errors.push("Username must be at least 3 characters".to_string());
        }

        if request.username.len() > 20 {
            errors.push("Username must be at most 20 characters".to_string());
        }

        if !request.email.contains('@') || !request.email.contains('.') {
            errors.push("Valid email address is required".to_string());
        }

        if request.password.len() < 8 {
            errors.push("Password must be at least 8 characters".to_string());
        }

        if request.password != request.confirm_password {
            errors.push("Passwords do not match".to_string());
        }

        if !request.terms_accepted {
            errors.push("You must accept the terms and conditions".to_string());
        }

        // Check for existing username/email
        {
            let username_map = self.username_to_id.read().await;
            if username_map.contains_key(&request.username) {
                errors.push("Username is already taken".to_string());
            }
        }

        {
            let email_map = self.email_to_id.read().await;
            if email_map.contains_key(&request.email) {
                errors.push("Email is already registered".to_string());
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        // Hash password
        let password_hash = self.hash_password(&request.password)?;

        // Create user
        let user_id = {
            let mut next_id = self.next_user_id.write().await;
            let id = *next_id;
            *next_id += 1;
            id
        };

        let user = User {
            user_id,
            username: request.username.clone(),
            email: request.email.clone(),
            password_hash,
            is_active: true,
            is_verified: false,
            created_at: Utc::now(),
            last_login: None,
            failed_login_attempts: 0,
            locked_until: None,
            two_factor_enabled: false,
            two_factor_secret: None,
        };

        // Store user
        {
            let mut users = self.users.write().await;
            users.insert(user_id, user);
        }

        {
            let mut username_map = self.username_to_id.write().await;
            username_map.insert(request.username, user_id);
        }

        {
            let mut email_map = self.email_to_id.write().await;
            email_map.insert(request.email, user_id);
        }

        Ok(user_id)
    }

    pub async fn login(&self, request: LoginRequest, ip_address: String, user_agent: Option<String>) -> Result<LoginResponse, String> {
        // Get user by username
        let user_id = {
            let username_map = self.username_to_id.read().await;
            *username_map.get(&request.username).ok_or("Invalid credentials")?
        };

        let mut user = {
            let mut users = self.users.write().await;
            users.get_mut(&user_id).ok_or("Invalid credentials")?.clone()
        };

        // Check if account is locked
        if let Some(locked_until) = user.locked_until {
            if Utc::now() < locked_until {
                return Err("Account is temporarily locked due to too many failed login attempts".to_string());
            } else {
                // Unlock account
                user.locked_until = None;
                user.failed_login_attempts = 0;
            }
        }

        // Verify password
        if !self.verify_password(&request.password, &user.password_hash).unwrap_or(false) {
            // Increment failed attempts
            user.failed_login_attempts += 1;
            
            // Lock account after 5 failed attempts
            if user.failed_login_attempts >= 5 {
                user.locked_until = Some(Utc::now() + chrono::Duration::minutes(30));
            }

            // Update user
            {
                let mut users = self.users.write().await;
                users.insert(user_id, user);
            }

            return Err("Invalid credentials".to_string());
        }

        // Check 2FA if enabled
        if user.two_factor_enabled {
            if let Some(totp_code) = request.totp_code {
                if !self.verify_totp(&user.two_factor_secret.unwrap_or_default(), &totp_code) {
                    return Err("Invalid 2FA code".to_string());
                }
            } else {
                return Ok(LoginResponse {
                    user_id,
                    username: user.username,
                    session_id: String::new(),
                    jwt_token: String::new(),
                    expires_at: Utc::now(),
                    requires_2fa: true,
                });
            }
        }

        // Reset failed attempts on successful login
        user.failed_login_attempts = 0;
        user.locked_until = None;
        user.last_login = Some(Utc::now());

        // Update user
        {
            let mut users = self.users.write().await;
            users.insert(user_id, user.clone());
        }

        // Create session
        let session_duration = if request.remember_me.unwrap_or(false) {
            Duration::from_secs(30 * 24 * 3600) // 30 days
        } else {
            Duration::from_secs(24 * 3600) // 24 hours
        };

        let session_id = Uuid::new_v4();
        let now = Utc::now();
        let expires_at = now + chrono::Duration::from_std(session_duration).unwrap();

        let session = Session {
            session_id,
            user_id,
            created_at: now,
            expires_at,
            last_accessed: now,
            ip_address,
            user_agent,
            is_active: true,
        };

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id, session);
        }

        // Generate JWT token
        let jwt_token = self.generate_jwt_token(user_id, &user.username, session_id, expires_at)?;

        Ok(LoginResponse {
            user_id,
            username: user.username,
            session_id: session_id.to_string(),
            jwt_token,
            expires_at,
            requires_2fa: false,
        })
    }

    pub async fn logout(&self, session_id: Uuid) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        if let Some(mut session) = sessions.get_mut(&session_id) {
            session.is_active = false;
        }
        sessions.remove(&session_id);
        Ok(())
    }

    pub async fn validate_session(&self, session_id: Uuid) -> Result<u64, String> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(&session_id) {
            if !session.is_active {
                return Err("Session is inactive".to_string());
            }

            if Utc::now() > session.expires_at {
                session.is_active = false;
                sessions.remove(&session_id);
                return Err("Session has expired".to_string());
            }

            // Update last accessed
            session.last_accessed = Utc::now();
            Ok(session.user_id)
        } else {
            Err("Invalid session".to_string())
        }
    }

    pub async fn validate_jwt(&self, token: &str) -> Result<JwtClaims, String> {
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_bytes());
        let validation = Validation::new(Algorithm::HS256);
        
        match decode::<JwtClaims>(token, &decoding_key, &validation) {
            Ok(token_data) => {
                // Check if session is still valid
                let session_id = Uuid::parse_str(&token_data.claims.jti)
                    .map_err(|_| "Invalid session ID in token")?;
                
                self.validate_session(session_id).await?;
                Ok(token_data.claims)
            }
            Err(_) => Err("Invalid or expired token".to_string()),
        }
    }

    pub async fn change_password(&self, user_id: u64, request: ChangePasswordRequest) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if request.new_password.len() < 8 {
            errors.push("New password must be at least 8 characters".to_string());
        }

        if request.new_password != request.confirm_new_password {
            errors.push("New passwords do not match".to_string());
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(&user_id) {
            // Verify current password
            if !self.verify_password(&request.current_password, &user.password_hash).unwrap_or(false) {
                return Err(vec!["Current password is incorrect".to_string()]);
            }

            // Hash new password
            let new_password_hash = self.hash_password(&request.new_password)
                .map_err(|_| vec!["Failed to hash password".to_string()])?;

            user.password_hash = new_password_hash;
            Ok(())
        } else {
            Err(vec!["User not found".to_string()])
        }
    }

    pub async fn initiate_password_reset(&self, email: String) -> Result<String, String> {
        let user_id = {
            let email_map = self.email_to_id.read().await;
            *email_map.get(&email).ok_or("Email not found")?
        };

        let reset_token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + chrono::Duration::hours(1); // 1 hour expiry

        {
            let mut reset_tokens = self.reset_tokens.write().await;
            reset_tokens.insert(reset_token.clone(), (user_id, expires_at));
        }

        // In a real implementation, you would send an email here
        // For testing, we just return the token
        Ok(reset_token)
    }

    pub async fn reset_password(&self, request: ResetPasswordRequest) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if request.new_password.len() < 8 {
            errors.push("Password must be at least 8 characters".to_string());
        }

        if request.new_password != request.confirm_new_password {
            errors.push("Passwords do not match".to_string());
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        // Validate reset token
        let (user_id, expires_at) = {
            let mut reset_tokens = self.reset_tokens.write().await;
            let (user_id, expires_at) = reset_tokens.get(&request.reset_token)
                .ok_or_else(|| vec!["Invalid reset token".to_string()])?
                .clone();

            if Utc::now() > expires_at {
                reset_tokens.remove(&request.reset_token);
                return Err(vec!["Reset token has expired".to_string()]);
            }

            reset_tokens.remove(&request.reset_token);
            (user_id, expires_at)
        };

        // Update password
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(&user_id) {
            let new_password_hash = self.hash_password(&request.new_password)
                .map_err(|_| vec!["Failed to hash password".to_string()])?;

            user.password_hash = new_password_hash;
            user.failed_login_attempts = 0;
            user.locked_until = None;

            Ok(())
        } else {
            Err(vec!["User not found".to_string()])
        }
    }

    pub async fn enable_2fa(&self, user_id: u64, password: String) -> Result<String, String> {
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(&user_id) {
            // Verify password
            if !self.verify_password(&password, &user.password_hash).unwrap_or(false) {
                return Err("Incorrect password".to_string());
            }

            // Generate TOTP secret
            let secret = self.generate_totp_secret();
            user.two_factor_secret = Some(secret.clone());

            // Don't enable yet - user needs to verify first
            Ok(secret)
        } else {
            Err("User not found".to_string())
        }
    }

    pub async fn verify_and_enable_2fa(&self, user_id: u64, totp_code: String) -> Result<(), String> {
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(&user_id) {
            let secret = user.two_factor_secret.as_ref().ok_or("2FA not initialized")?;
            
            if self.verify_totp(secret, &totp_code) {
                user.two_factor_enabled = true;
                Ok(())
            } else {
                Err("Invalid TOTP code".to_string())
            }
        } else {
            Err("User not found".to_string())
        }
    }

    pub async fn disable_2fa(&self, user_id: u64, password: String) -> Result<(), String> {
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(&user_id) {
            // Verify password
            if !self.verify_password(&password, &user.password_hash).unwrap_or(false) {
                return Err("Incorrect password".to_string());
            }

            user.two_factor_enabled = false;
            user.two_factor_secret = None;
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }

    pub async fn get_user_profile(&self, user_id: u64) -> Result<UserProfile, String> {
        let users = self.users.read().await;
        if let Some(user) = users.get(&user_id) {
            Ok(UserProfile {
                user_id,
                username: user.username.clone(),
                email: user.email.clone(),
                is_verified: user.is_verified,
                created_at: user.created_at,
                last_login: user.last_login,
                two_factor_enabled: user.two_factor_enabled,
            })
        } else {
            Err("User not found".to_string())
        }
    }

    pub async fn get_user_sessions(&self, user_id: u64) -> Vec<Session> {
        let sessions = self.sessions.read().await;
        sessions.values()
            .filter(|session| session.user_id == user_id && session.is_active)
            .cloned()
            .collect()
    }

    pub async fn revoke_all_sessions(&self, user_id: u64, except_session: Option<Uuid>) -> usize {
        let mut sessions = self.sessions.write().await;
        let mut revoked = 0;

        let sessions_to_revoke: Vec<Uuid> = sessions.values()
            .filter(|session| {
                session.user_id == user_id && 
                session.is_active && 
                Some(session.session_id) != except_session
            })
            .map(|session| session.session_id)
            .collect();

        for session_id in sessions_to_revoke {
            if let Some(session) = sessions.get_mut(&session_id) {
                session.is_active = false;
                revoked += 1;
            }
        }

        revoked
    }

    // Helper methods
    fn hash_password(&self, password: &str) -> Result<String, Vec<String>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        match argon2.hash_password(password.as_bytes(), &salt) {
            Ok(hash) => Ok(hash.to_string()),
            Err(_) => Err(vec!["Failed to hash password".to_string()]),
        }
    }

    fn verify_password(&self, password: &str, hash: &str) -> Result<bool, String> {
        let parsed_hash = PasswordHash::new(hash).map_err(|_| "Invalid password hash")?;
        let argon2 = Argon2::default();
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

    fn generate_jwt_token(&self, user_id: u64, username: &str, session_id: Uuid, expires_at: DateTime<Utc>) -> Result<String, String> {
        let claims = JwtClaims {
            sub: user_id.to_string(),
            username: username.to_string(),
            exp: expires_at.timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
            jti: session_id.to_string(),
        };

        let encoding_key = EncodingKey::from_secret(self.jwt_secret.as_bytes());
        encode(&Header::default(), &claims, &encoding_key)
            .map_err(|_| "Failed to generate token".to_string())
    }

    fn generate_totp_secret(&self) -> String {
        // Generate a random 32-byte secret for TOTP
        let secret: [u8; 32] = rand::random();
        base32::encode(base32::Alphabet::RFC4648 { padding: false }, &secret)
    }

    fn verify_totp(&self, secret: &str, code: &str) -> bool {
        // Simple TOTP verification for testing
        // In production, use a proper TOTP library like `totp-rs`
        let time_step = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() / 30;
        
        // For testing, accept codes that are 6 digits and start with '1'
        code.len() == 6 && code.chars().all(|c| c.is_ascii_digit()) && code.starts_with('1')
    }
}

// ===== HTTP HANDLERS =====

async fn register_handler(
    State(auth_service): State<Arc<AuthService>>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<AuthResponse<u64>>, StatusCode> {
    match auth_service.register_user(request).await {
        Ok(user_id) => Ok(Json(AuthResponse {
            success: true,
            message: "User registered successfully".to_string(),
            data: Some(user_id),
            errors: None,
        })),
        Err(errors) => Ok(Json(AuthResponse {
            success: false,
            message: "Registration failed".to_string(),
            data: None,
            errors: Some(errors),
        })),
    }
}

async fn login_handler(
    State(auth_service): State<Arc<AuthService>>,
    headers: axum::http::HeaderMap,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse<LoginResponse>>, StatusCode> {
    let ip_address = "127.0.0.1".to_string(); // In production, extract from headers
    let user_agent = headers.get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    match auth_service.login(request, ip_address, user_agent).await {
        Ok(response) => Ok(Json(AuthResponse {
            success: true,
            message: if response.requires_2fa {
                "2FA code required".to_string()
            } else {
                "Login successful".to_string()
            },
            data: Some(response),
            errors: None,
        })),
        Err(error) => Ok(Json(AuthResponse {
            success: false,
            message: error,
            data: None,
            errors: None,
        })),
    }
}

async fn logout_handler(
    State(auth_service): State<Arc<AuthService>>,
    Extension(session_id): Extension<Uuid>,
) -> Result<Json<AuthResponse<()>>, StatusCode> {
    match auth_service.logout(session_id).await {
        Ok(_) => Ok(Json(AuthResponse {
            success: true,
            message: "Logged out successfully".to_string(),
            data: None,
            errors: None,
        })),
        Err(error) => Ok(Json(AuthResponse {
            success: false,
            message: error,
            data: None,
            errors: None,
        })),
    }
}

async fn profile_handler(
    State(auth_service): State<Arc<AuthService>>,
    Extension(user_id): Extension<u64>,
) -> Result<Json<AuthResponse<UserProfile>>, StatusCode> {
    match auth_service.get_user_profile(user_id).await {
        Ok(profile) => Ok(Json(AuthResponse {
            success: true,
            message: "Profile retrieved".to_string(),
            data: Some(profile),
            errors: None,
        })),
        Err(error) => Ok(Json(AuthResponse {
            success: false,
            message: error,
            data: None,
            errors: None,
        })),
    }
}

async fn change_password_handler(
    State(auth_service): State<Arc<AuthService>>,
    Extension(user_id): Extension<u64>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<Json<AuthResponse<()>>, StatusCode> {
    match auth_service.change_password(user_id, request).await {
        Ok(_) => Ok(Json(AuthResponse {
            success: true,
            message: "Password changed successfully".to_string(),
            data: None,
            errors: None,
        })),
        Err(errors) => Ok(Json(AuthResponse {
            success: false,
            message: "Password change failed".to_string(),
            data: None,
            errors: Some(errors),
        })),
    }
}

async fn forgot_password_handler(
    State(auth_service): State<Arc<AuthService>>,
    Json(request): Json<ForgotPasswordRequest>,
) -> Result<Json<AuthResponse<String>>, StatusCode> {
    match auth_service.initiate_password_reset(request.email).await {
        Ok(reset_token) => Ok(Json(AuthResponse {
            success: true,
            message: "Password reset email sent".to_string(),
            data: Some(reset_token), // In production, don't return token
            errors: None,
        })),
        Err(error) => Ok(Json(AuthResponse {
            success: false,
            message: error,
            data: None,
            errors: None,
        })),
    }
}

async fn reset_password_handler(
    State(auth_service): State<Arc<AuthService>>,
    Json(request): Json<ResetPasswordRequest>,
) -> Result<Json<AuthResponse<()>>, StatusCode> {
    match auth_service.reset_password(request).await {
        Ok(_) => Ok(Json(AuthResponse {
            success: true,
            message: "Password reset successfully".to_string(),
            data: None,
            errors: None,
        })),
        Err(errors) => Ok(Json(AuthResponse {
            success: false,
            message: "Password reset failed".to_string(),
            data: None,
            errors: Some(errors),
        })),
    }
}

async fn enable_2fa_handler(
    State(auth_service): State<Arc<AuthService>>,
    Extension(user_id): Extension<u64>,
    Json(request): Json<Enable2FARequest>,
) -> Result<Json<AuthResponse<String>>, StatusCode> {
    match auth_service.enable_2fa(user_id, request.password).await {
        Ok(secret) => Ok(Json(AuthResponse {
            success: true,
            message: "2FA setup initiated".to_string(),
            data: Some(secret),
            errors: None,
        })),
        Err(error) => Ok(Json(AuthResponse {
            success: false,
            message: error,
            data: None,
            errors: None,
        })),
    }
}

async fn verify_2fa_handler(
    State(auth_service): State<Arc<AuthService>>,
    Extension(user_id): Extension<u64>,
    Json(request): Json<Verify2FARequest>,
) -> Result<Json<AuthResponse<()>>, StatusCode> {
    match auth_service.verify_and_enable_2fa(user_id, request.totp_code).await {
        Ok(_) => Ok(Json(AuthResponse {
            success: true,
            message: "2FA enabled successfully".to_string(),
            data: None,
            errors: None,
        })),
        Err(error) => Ok(Json(AuthResponse {
            success: false,
            message: error,
            data: None,
            errors: None,
        })),
    }
}

// Authentication middleware
#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for Extension<u64>
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Extract JWT token from Authorization header
        let auth_header = parts.headers.get("Authorization")
            .and_then(|header| header.to_str().ok())
            .and_then(|header| header.strip_prefix("Bearer "))
            .ok_or(StatusCode::UNAUTHORIZED)?;

        // For testing, we'll simulate JWT validation
        // In real implementation, decode and validate JWT
        if auth_header.starts_with("valid_jwt_") {
            let user_id = auth_header.strip_prefix("valid_jwt_")
                .and_then(|s| s.parse::<u64>().ok())
                .ok_or(StatusCode::UNAUTHORIZED)?;
            
            Ok(Extension(user_id))
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for Extension<Uuid>
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Extract session ID from headers or cookies
        // For testing, we'll use a simple header
        let session_header = parts.headers.get("X-Session-ID")
            .and_then(|header| header.to_str().ok())
            .and_then(|header| Uuid::parse_str(header).ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;
        
        Ok(Extension(session_header))
    }
}

// Create test application
fn create_auth_app(auth_service: Arc<AuthService>) -> Router {
    Router::new()
        .route("/auth/register", post(register_handler))
        .route("/auth/login", post(login_handler))
        .route("/auth/logout", post(logout_handler))
        .route("/auth/profile", get(profile_handler))
        .route("/auth/change-password", post(change_password_handler))
        .route("/auth/forgot-password", post(forgot_password_handler))
        .route("/auth/reset-password", post(reset_password_handler))
        .route("/auth/2fa/enable", post(enable_2fa_handler))
        .route("/auth/2fa/verify", post(verify_2fa_handler))
        .with_state(auth_service)
}

// Helper functions for testing
fn create_json_request(method: axum::http::Method, uri: &str, body: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap()
}

fn create_authenticated_request(method: axum::http::Method, uri: &str, user_id: u64, body: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer valid_jwt_{}", user_id))
        .body(Body::from(body))
        .unwrap()
}

// ===== AUTHENTICATION INTEGRATION TESTS =====

#[tokio::test]
async fn test_user_registration_flow() {
    let auth_service = Arc::new(AuthService::new());
    let app = create_auth_app(auth_service);

    // Test successful registration
    let register_data = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123",
        "confirm_password": "password123",
        "terms_accepted": true
    });

    let request = create_json_request(
        axum::http::Method::POST, 
        "/auth/register", 
        &register_data.to_string()
    );

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let auth_response: AuthResponse<u64> = serde_json::from_slice(&body).unwrap();
    
    assert!(auth_response.success);
    assert!(auth_response.data.is_some());
    assert_eq!(auth_response.data.unwrap(), 1); // First user ID

    // Test registration with validation errors
    let invalid_data = json!({
        "username": "x", // Too short
        "email": "invalid-email", // Invalid format
        "password": "123", // Too short
        "confirm_password": "456", // Doesn't match
        "terms_accepted": false // Not accepted
    });

    let request = create_json_request(
        axum::http::Method::POST, 
        "/auth/register", 
        &invalid_data.to_string()
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let auth_response: AuthResponse<u64> = serde_json::from_slice(&body).unwrap();
    
    assert!(!auth_response.success);
    assert!(auth_response.errors.is_some());
    let errors = auth_response.errors.unwrap();
    assert!(errors.len() >= 4); // Multiple validation errors
}

#[tokio::test]
async fn test_login_flow() {
    let auth_service = Arc::new(AuthService::new());
    let app = create_auth_app(auth_service.clone());

    // Register a user first
    let register_result = auth_service.register_user(RegisterRequest {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        confirm_password: "password123".to_string(),
        terms_accepted: true,
    }).await;
    assert!(register_result.is_ok());

    // Test successful login
    let login_data = json!({
        "username": "testuser",
        "password": "password123",
        "remember_me": false
    });

    let request = create_json_request(
        axum::http::Method::POST, 
        "/auth/login", 
        &login_data.to_string()
    );

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let auth_response: AuthResponse<LoginResponse> = serde_json::from_slice(&body).unwrap();
    
    assert!(auth_response.success);
    let login_response = auth_response.data.unwrap();
    assert_eq!(login_response.user_id, 1);
    assert_eq!(login_response.username, "testuser");
    assert!(!login_response.jwt_token.is_empty());
    assert!(!login_response.requires_2fa);

    // Test login with wrong password
    let wrong_login = json!({
        "username": "testuser",
        "password": "wrongpassword"
    });

    let request = create_json_request(
        axum::http::Method::POST, 
        "/auth/login", 
        &wrong_login.to_string()
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let auth_response: AuthResponse<LoginResponse> = serde_json::from_slice(&body).unwrap();
    
    assert!(!auth_response.success);
    assert_eq!(auth_response.message, "Invalid credentials");
}

#[tokio::test]
async fn test_account_lockout_mechanism() {
    let auth_service = Arc::new(AuthService::new());

    // Register a user
    let register_result = auth_service.register_user(RegisterRequest {
        username: "locktest".to_string(),
        email: "locktest@example.com".to_string(),
        password: "password123".to_string(),
        confirm_password: "password123".to_string(),
        terms_accepted: true,
    }).await;
    assert!(register_result.is_ok());

    // Attempt multiple failed logins
    for i in 0..5 {
        let result = auth_service.login(LoginRequest {
            username: "locktest".to_string(),
            password: "wrongpassword".to_string(),
            remember_me: None,
            totp_code: None,
        }, "127.0.0.1".to_string(), None).await;
        
        assert!(result.is_err());
        if i < 4 {
            assert_eq!(result.unwrap_err(), "Invalid credentials");
        }
    }

    // 6th attempt should indicate account is locked
    let result = auth_service.login(LoginRequest {
        username: "locktest".to_string(),
        password: "wrongpassword".to_string(),
        remember_me: None,
        totp_code: None,
    }, "127.0.0.1".to_string(), None).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("locked"));

    // Even correct password should fail when locked
    let result = auth_service.login(LoginRequest {
        username: "locktest".to_string(),
        password: "password123".to_string(),
        remember_me: None,
        totp_code: None,
    }, "127.0.0.1".to_string(), None).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("locked"));
}

#[tokio::test]
async fn test_password_change_flow() {
    let auth_service = Arc::new(AuthService::new());
    let app = create_auth_app(auth_service.clone());

    // Register and login user
    let user_id = assert_ok!(auth_service.register_user(RegisterRequest {
        username: "changetest".to_string(),
        email: "change@example.com".to_string(),
        password: "oldpassword123".to_string(),
        confirm_password: "oldpassword123".to_string(),
        terms_accepted: true,
    }).await);

    // Test successful password change
    let change_data = json!({
        "current_password": "oldpassword123",
        "new_password": "newpassword456",
        "confirm_new_password": "newpassword456"
    });

    let request = create_authenticated_request(
        axum::http::Method::POST, 
        "/auth/change-password", 
        user_id,
        &change_data.to_string()
    );

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let auth_response: AuthResponse<()> = serde_json::from_slice(&body).unwrap();
    
    assert!(auth_response.success);

    // Verify old password no longer works
    let old_login_result = auth_service.login(LoginRequest {
        username: "changetest".to_string(),
        password: "oldpassword123".to_string(),
        remember_me: None,
        totp_code: None,
    }, "127.0.0.1".to_string(), None).await;
    assert!(old_login_result.is_err());

    // Verify new password works
    let new_login_result = auth_service.login(LoginRequest {
        username: "changetest".to_string(),
        password: "newpassword456".to_string(),
        remember_me: None,
        totp_code: None,
    }, "127.0.0.1".to_string(), None).await;
    assert!(new_login_result.is_ok());

    // Test password change with wrong current password
    let wrong_change = json!({
        "current_password": "wrongcurrent",
        "new_password": "anothernew123",
        "confirm_new_password": "anothernew123"
    });

    let request = create_authenticated_request(
        axum::http::Method::POST, 
        "/auth/change-password", 
        user_id,
        &wrong_change.to_string()
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let auth_response: AuthResponse<()> = serde_json::from_slice(&body).unwrap();
    
    assert!(!auth_response.success);
    assert!(auth_response.errors.is_some());
}

#[tokio::test]
async fn test_password_reset_flow() {
    let auth_service = Arc::new(AuthService::new());
    let app = create_auth_app(auth_service.clone());

    // Register a user
    let user_id = assert_ok!(auth_service.register_user(RegisterRequest {
        username: "resettest".to_string(),
        email: "reset@example.com".to_string(),
        password: "originalpass123".to_string(),
        confirm_password: "originalpass123".to_string(),
        terms_accepted: true,
    }).await);

    // Test forgot password
    let forgot_data = json!({
        "email": "reset@example.com"
    });

    let request = create_json_request(
        axum::http::Method::POST, 
        "/auth/forgot-password", 
        &forgot_data.to_string()
    );

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let auth_response: AuthResponse<String> = serde_json::from_slice(&body).unwrap();
    
    assert!(auth_response.success);
    let reset_token = auth_response.data.unwrap();

    // Test password reset with token
    let reset_data = json!({
        "reset_token": reset_token,
        "new_password": "resetpass456",
        "confirm_new_password": "resetpass456"
    });

    let request = create_json_request(
        axum::http::Method::POST, 
        "/auth/reset-password", 
        &reset_data.to_string()
    );

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let auth_response: AuthResponse<()> = serde_json::from_slice(&body).unwrap();
    
    assert!(auth_response.success);

    // Verify old password doesn't work
    let old_login_result = auth_service.login(LoginRequest {
        username: "resettest".to_string(),
        password: "originalpass123".to_string(),
        remember_me: None,
        totp_code: None,
    }, "127.0.0.1".to_string(), None).await;
    assert!(old_login_result.is_err());

    // Verify new password works
    let new_login_result = auth_service.login(LoginRequest {
        username: "resettest".to_string(),
        password: "resetpass456".to_string(),
        remember_me: None,
        totp_code: None,
    }, "127.0.0.1".to_string(), None).await;
    assert!(new_login_result.is_ok());

    // Test reset with invalid token
    let invalid_reset = json!({
        "reset_token": "invalid-token",
        "new_password": "newpass789",
        "confirm_new_password": "newpass789"
    });

    let request = create_json_request(
        axum::http::Method::POST, 
        "/auth/reset-password", 
        &invalid_reset.to_string()
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let auth_response: AuthResponse<()> = serde_json::from_slice(&body).unwrap();
    
    assert!(!auth_response.success);
    assert!(auth_response.errors.is_some());
}

#[tokio::test]
async fn test_two_factor_authentication_flow() {
    let auth_service = Arc::new(AuthService::new());
    let app = create_auth_app(auth_service.clone());

    // Register a user
    let user_id = assert_ok!(auth_service.register_user(RegisterRequest {
        username: "2fatest".to_string(),
        email: "2fa@example.com".to_string(),
        password: "password123".to_string(),
        confirm_password: "password123".to_string(),
        terms_accepted: true,
    }).await);

    // Test 2FA setup
    let setup_data = json!({
        "password": "password123"
    });

    let request = create_authenticated_request(
        axum::http::Method::POST, 
        "/auth/2fa/enable", 
        user_id,
        &setup_data.to_string()
    );

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let auth_response: AuthResponse<String> = serde_json::from_slice(&body).unwrap();
    
    assert!(auth_response.success);
    let totp_secret = auth_response.data.unwrap();
    assert!(!totp_secret.is_empty());

    // Test 2FA verification
    let verify_data = json!({
        "totp_code": "123456" // Valid test code (starts with '1')
    });

    let request = create_authenticated_request(
        axum::http::Method::POST, 
        "/auth/2fa/verify", 
        user_id,
        &verify_data.to_string()
    );

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let auth_response: AuthResponse<()> = serde_json::from_slice(&body).unwrap();
    
    assert!(auth_response.success);

    // Test login with 2FA now required
    let login_without_2fa = LoginRequest {
        username: "2fatest".to_string(),
        password: "password123".to_string(),
        remember_me: None,
        totp_code: None,
    };

    let login_result = auth_service.login(login_without_2fa, "127.0.0.1".to_string(), None).await;
    assert!(login_result.is_ok());
    let login_response = login_result.unwrap();
    assert!(login_response.requires_2fa);

    // Test login with 2FA code
    let login_with_2fa = LoginRequest {
        username: "2fatest".to_string(),
        password: "password123".to_string(),
        remember_me: None,
        totp_code: Some("123456".to_string()), // Valid test code
    };

    let login_result = auth_service.login(login_with_2fa, "127.0.0.1".to_string(), None).await;
    assert!(login_result.is_ok());
    let login_response = login_result.unwrap();
    assert!(!login_response.requires_2fa);
    assert!(!login_response.jwt_token.is_empty());
}

#[tokio::test]
async fn test_session_management() {
    let auth_service = Arc::new(AuthService::new());

    // Register and login user
    let user_id = assert_ok!(auth_service.register_user(RegisterRequest {
        username: "sessiontest".to_string(),
        email: "session@example.com".to_string(),
        password: "password123".to_string(),
        confirm_password: "password123".to_string(),
        terms_accepted: true,
    }).await);

    // Create multiple sessions
    let login1 = auth_service.login(LoginRequest {
        username: "sessiontest".to_string(),
        password: "password123".to_string(),
        remember_me: Some(false),
        totp_code: None,
    }, "127.0.0.1".to_string(), Some("Browser1".to_string())).await;
    assert!(login1.is_ok());
    let session1 = Uuid::parse_str(&login1.unwrap().session_id).unwrap();

    let login2 = auth_service.login(LoginRequest {
        username: "sessiontest".to_string(),
        password: "password123".to_string(),
        remember_me: Some(true), // Remember me session
        totp_code: None,
    }, "192.168.1.100".to_string(), Some("Mobile".to_string())).await;
    assert!(login2.is_ok());
    let session2 = Uuid::parse_str(&login2.unwrap().session_id).unwrap();

    // Check user has multiple sessions
    let sessions = auth_service.get_user_sessions(user_id).await;
    assert_eq!(sessions.len(), 2);

    // Validate sessions
    let validation1 = auth_service.validate_session(session1).await;
    assert!(validation1.is_ok());
    assert_eq!(validation1.unwrap(), user_id);

    let validation2 = auth_service.validate_session(session2).await;
    assert!(validation2.is_ok());
    assert_eq!(validation2.unwrap(), user_id);

    // Logout one session
    assert_ok!(auth_service.logout(session1).await);

    // First session should be invalid
    let validation1 = auth_service.validate_session(session1).await;
    assert!(validation1.is_err());

    // Second session should still be valid
    let validation2 = auth_service.validate_session(session2).await;
    assert!(validation2.is_ok());

    // Revoke all sessions except current
    let revoked = auth_service.revoke_all_sessions(user_id, Some(session2)).await;
    assert_eq!(revoked, 0); // Only one active session, which we excluded

    // Revoke all sessions
    let revoked = auth_service.revoke_all_sessions(user_id, None).await;
    assert_eq!(revoked, 1);
}

#[tokio::test]
async fn test_concurrent_authentication_operations() {
    let auth_service = Arc::new(AuthService::new());

    // Register multiple users concurrently
    let register_futures: Vec<_> = (0..10).map(|i| {
        let auth_service = auth_service.clone();
        async move {
            auth_service.register_user(RegisterRequest {
                username: format!("user{}", i),
                email: format!("user{}@example.com", i),
                password: "password123".to_string(),
                confirm_password: "password123".to_string(),
                terms_accepted: true,
            }).await
        }
    }).collect();

    let register_results = futures::future::join_all(register_futures).await;

    // All registrations should succeed
    for result in register_results {
        assert!(result.is_ok());
    }

    // Login with all users concurrently
    let login_futures: Vec<_> = (0..10).map(|i| {
        let auth_service = auth_service.clone();
        async move {
            auth_service.login(LoginRequest {
                username: format!("user{}", i),
                password: "password123".to_string(),
                remember_me: None,
                totp_code: None,
            }, "127.0.0.1".to_string(), None).await
        }
    }).collect();

    let login_results = futures::future::join_all(login_futures).await;

    // All logins should succeed
    for result in login_results {
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_authentication_security_edge_cases() {
    let auth_service = Arc::new(AuthService::new());

    // Test registration with malicious input
    let malicious_register = RegisterRequest {
        username: "<script>alert('xss')</script>".to_string(),
        email: "'; DROP TABLE users; --".to_string(),
        password: "password123".to_string(),
        confirm_password: "password123".to_string(),
        terms_accepted: true,
    };

    let result = auth_service.register_user(malicious_register).await;
    // Should fail validation due to invalid email format
    assert!(result.is_err());

    // Test SQL injection attempts in login
    let sql_injection_login = LoginRequest {
        username: "admin' OR '1'='1".to_string(),
        password: "password".to_string(),
        remember_me: None,
        totp_code: None,
    };

    let result = auth_service.login(sql_injection_login, "127.0.0.1".to_string(), None).await;
    assert!(result.is_err()); // Should fail because user doesn't exist

    // Test extremely long inputs
    let long_username = "a".repeat(1000);
    let long_password = "b".repeat(1000);

    let long_input_register = RegisterRequest {
        username: long_username.clone(),
        email: "long@example.com".to_string(),
        password: long_password.clone(),
        confirm_password: long_password,
        terms_accepted: true,
    };

    let result = auth_service.register_user(long_input_register).await;
    assert!(result.is_err()); // Should fail validation
}

#[tokio::test]
async fn test_authentication_performance() {
    let auth_service = Arc::new(AuthService::new());

    // Register a user
    let user_id = assert_ok!(auth_service.register_user(RegisterRequest {
        username: "perftest".to_string(),
        email: "perf@example.com".to_string(),
        password: "password123".to_string(),
        confirm_password: "password123".to_string(),
        terms_accepted: true,
    }).await);

    let start_time = std::time::Instant::now();
    let num_operations = 100;

    // Perform many login operations
    for _ in 0..num_operations {
        let login_result = auth_service.login(LoginRequest {
            username: "perftest".to_string(),
            password: "password123".to_string(),
            remember_me: None,
            totp_code: None,
        }, "127.0.0.1".to_string(), None).await;
        assert!(login_result.is_ok());
    }

    let duration = start_time.elapsed();
    let ops_per_sec = num_operations as f64 / duration.as_secs_f64();

    println!("Authentication performance: {} operations in {:?} ({:.2} ops/sec)", 
        num_operations, duration, ops_per_sec);

    // Performance assertion: should handle at least 10 logins per second
    assert!(ops_per_sec >= 10.0, "Authentication too slow: {:.2} ops/sec", ops_per_sec);
}