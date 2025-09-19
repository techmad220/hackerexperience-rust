//! # HackerExperience Authentication & Authorization
//!
//! Comprehensive authentication and authorization system supporting:
//! - JWT token management
//! - Session management
//! - Role-based access control (RBAC)
//! - Rate limiting
//! - Multi-factor authentication (MFA)
//! - OAuth integration

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub mod jwt;
pub mod session;

#[cfg(test)]
mod tests;
pub mod rbac;
pub mod rate_limit;
pub mod mfa;
pub mod oauth;
pub mod password;
pub mod middleware;

// Re-export main types
pub use jwt::{JwtManager, JwtClaims, JwtConfig};
pub use session::{SessionManager, SessionData, SessionConfig};
pub use rbac::{RoleManager, Permission, Role, AccessControl};
pub use rate_limit::{RateLimiter, RateLimit, RateLimitConfig};
pub use mfa::{MfaManager, MfaMethod, MfaConfig};
pub use oauth::{OAuthProvider, OAuthConfig, OAuthManager};
pub use password::{PasswordManager, PasswordConfig, PasswordStrength};
pub use middleware::{AuthMiddleware, RequireAuth, RequireRole};

/// Main authentication service
#[derive(Debug, Clone)]
pub struct AuthService {
    jwt_manager: Arc<JwtManager>,
    session_manager: Arc<SessionManager>,
    role_manager: Arc<RoleManager>,
    rate_limiter: Arc<RateLimiter>,
    mfa_manager: Arc<MfaManager>,
    password_manager: Arc<PasswordManager>,
    config: AuthConfig,
}

/// Authentication configuration
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt: JwtConfig,
    pub session: SessionConfig,
    pub rate_limit: RateLimitConfig,
    pub mfa: MfaConfig,
    pub password: PasswordConfig,
    pub enable_registration: bool,
    pub require_email_verification: bool,
    pub max_login_attempts: u32,
    pub lockout_duration: std::time::Duration,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt: JwtConfig::default(),
            session: SessionConfig::default(),
            rate_limit: RateLimitConfig::default(),
            mfa: MfaConfig::default(),
            password: PasswordConfig::default(),
            enable_registration: true,
            require_email_verification: false,
            max_login_attempts: 5,
            lockout_duration: std::time::Duration::from_secs(900), // 15 minutes
        }
    }
}

impl AuthService {
    /// Create a new authentication service
    pub async fn new(config: AuthConfig) -> Result<Self> {
        let jwt_manager = Arc::new(JwtManager::new(config.jwt.clone())?);
        let session_manager = Arc::new(SessionManager::new(config.session.clone()).await?);
        let role_manager = Arc::new(RoleManager::new().await?);
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limit.clone()));
        let mfa_manager = Arc::new(MfaManager::new(config.mfa.clone())?);
        let password_manager = Arc::new(PasswordManager::new(config.password.clone()));

        // Initialize default roles for HackerExperience
        Self::setup_default_roles(&role_manager).await?;

        Ok(Self {
            jwt_manager,
            session_manager,
            role_manager,
            rate_limiter,
            mfa_manager,
            password_manager,
            config,
        })
    }

    /// Set up default roles for the game
    async fn setup_default_roles(role_manager: &RoleManager) -> Result<()> {
        // Player role - basic game access
        let player_permissions = vec![
            Permission::new("game:play", "Basic game access"),
            Permission::new("profile:view", "View own profile"),
            Permission::new("profile:edit", "Edit own profile"),
            Permission::new("chat:send", "Send chat messages"),
            Permission::new("mission:start", "Start missions"),
            Permission::new("process:start", "Start processes"),
        ];
        let player_role = Role::new("player", "Standard Player", player_permissions);
        role_manager.create_role(player_role).await?;

        // Premium player role - extended access
        let premium_permissions = vec![
            Permission::new("game:play", "Basic game access"),
            Permission::new("profile:view", "View own profile"),
            Permission::new("profile:edit", "Edit own profile"),
            Permission::new("chat:send", "Send chat messages"),
            Permission::new("mission:start", "Start missions"),
            Permission::new("process:start", "Start processes"),
            Permission::new("premium:features", "Access premium features"),
            Permission::new("stats:detailed", "View detailed statistics"),
        ];
        let premium_role = Role::new("premium_player", "Premium Player", premium_permissions);
        role_manager.create_role(premium_role).await?;

        // Moderator role - moderation capabilities
        let moderator_permissions = vec![
            Permission::new("game:play", "Basic game access"),
            Permission::new("chat:moderate", "Moderate chat"),
            Permission::new("player:warn", "Warn players"),
            Permission::new("player:mute", "Mute players"),
            Permission::new("reports:view", "View reports"),
            Permission::new("reports:resolve", "Resolve reports"),
        ];
        let moderator_role = Role::new("moderator", "Moderator", moderator_permissions);
        role_manager.create_role(moderator_role).await?;

        // Admin role - full access
        let admin_permissions = vec![
            Permission::new("*", "Full system access"),
        ];
        let admin_role = Role::new("admin", "Administrator", admin_permissions);
        role_manager.create_role(admin_role).await?;

        info!("Default roles initialized");
        Ok(())
    }

    /// Authenticate user with email/password using Argon2id
    pub async fn authenticate(
        &self,
        email: &str,
        password: &str,
        client_ip: Option<String>,
        pool: &sqlx::PgPool,
    ) -> Result<AuthenticationResult> {
        // Check rate limiting
        if let Some(ip) = &client_ip {
            if !self.rate_limiter.check_login_rate(ip).await {
                return Ok(AuthenticationResult::RateLimited);
            }
        }

        debug!("Attempting authentication for user: {}", email);

        // Get user from database
        let user = sqlx::query!(
            r#"
            SELECT u.id, u.email, u.pwd as password_hash, u.active, u.email_verified,
                   COALESCE(array_agg(r.name) FILTER (WHERE r.name IS NOT NULL), '{}') as "roles!"
            FROM users u
            LEFT JOIN user_roles ur ON u.id = ur.user_id
            LEFT JOIN roles r ON ur.role_id = r.id
            WHERE u.email = $1
            GROUP BY u.id, u.email, u.pwd, u.active, u.email_verified
            "#,
            email
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        let user = match user {
            Some(u) => u,
            None => {
                // Record failed attempt even if user doesn't exist (prevent enumeration)
                if let Some(ip) = client_ip {
                    self.rate_limiter.record_failed_login(&ip).await;
                }
                return Ok(AuthenticationResult::InvalidCredentials);
            }
        };

        // Check if account is active
        if !user.active.unwrap_or(true) {
            return Ok(AuthenticationResult::AccountLocked);
        }

        // Check email verification if required
        if self.config.require_email_verification && !user.email_verified.unwrap_or(false) {
            return Ok(AuthenticationResult::EmailNotVerified);
        }

        // Verify password using Argon2id
        let password_valid = self.password_manager.verify_password_argon2id(
            password,
            &user.password_hash
        ).await?;

        if !password_valid {
            // Record failed login attempt
            if let Some(ip) = &client_ip {
                self.rate_limiter.record_failed_login(&ip).await;
            }

            // Update failed login count in database
            sqlx::query!(
                "UPDATE users SET failed_login_attempts = failed_login_attempts + 1 WHERE id = $1",
                user.id
            )
            .execute(pool)
            .await?;

            return Ok(AuthenticationResult::InvalidCredentials);
        }

        // Reset failed login attempts on successful login
        sqlx::query!(
            "UPDATE users SET failed_login_attempts = 0, last_login = NOW() WHERE id = $1",
            user.id
        )
        .execute(pool)
        .await?;

        let user_id = Uuid::parse_str(&user.id.to_string()).unwrap_or_else(|_| Uuid::new_v4());
        let user_roles = user.roles;

        // Check if MFA is required
        if self.mfa_manager.is_mfa_required(&user_id).await? {
            return Ok(AuthenticationResult::MfaRequired { user_id });
        }

        // Create session and JWT token
        let session_data = SessionData {
            user_id,
            email: email.to_string(),
            roles: user_roles.clone(),
            login_time: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            ip_address: client_ip,
            metadata: HashMap::new(),
        };

        let session_id = self.session_manager.create_session(session_data).await?;
        
        let jwt_claims = JwtClaims {
            user_id,
            email: email.to_string(),
            roles: user_roles,
            session_id: Some(session_id),
            exp: (chrono::Utc::now() + chrono::Duration::seconds(self.config.jwt.expiration_seconds as i64)).timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
        };

        let token = self.jwt_manager.generate_token(&jwt_claims)?;

        info!("User {} authenticated successfully", email);
        Ok(AuthenticationResult::Success {
            token,
            user_id,
            session_id,
        })
    }

    /// Validate JWT token and get user info
    pub async fn validate_token(&self, token: &str) -> Result<Option<ValidatedUser>> {
        let claims = match self.jwt_manager.validate_token(token) {
            Ok(claims) => claims,
            Err(_) => return Ok(None),
        };

        // Check if session is still valid (if session_id is present)
        if let Some(session_id) = claims.session_id {
            if !self.session_manager.is_session_valid(&session_id).await? {
                return Ok(None);
            }
        }

        Ok(Some(ValidatedUser {
            user_id: claims.user_id,
            email: claims.email,
            roles: claims.roles,
            session_id: claims.session_id,
        }))
    }

    /// Refresh JWT token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<Option<String>> {
        // TODO: Implement refresh token logic
        // This would validate the refresh token and generate a new access token
        warn!("Refresh token functionality not yet implemented");
        Ok(None)
    }

    /// Logout user (invalidate session)
    pub async fn logout(&self, session_id: &str) -> Result<()> {
        self.session_manager.invalidate_session(session_id).await?;
        info!("User logged out, session invalidated: {}", session_id);
        Ok(())
    }

    /// Check if user has permission
    pub async fn check_permission(&self, user_id: &Uuid, permission: &str) -> Result<bool> {
        let user_roles = self.get_user_roles(user_id).await?;
        
        for role_name in user_roles {
            if self.role_manager.role_has_permission(&role_name, permission).await? {
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    /// Get user roles (would typically query database)
    async fn get_user_roles(&self, _user_id: &Uuid) -> Result<Vec<String>> {
        // TODO: Implement actual database query
        Ok(vec!["player".to_string()])
    }

    /// Start MFA process
    pub async fn start_mfa(&self, user_id: &Uuid, method: MfaMethod) -> Result<String> {
        self.mfa_manager.start_mfa_process(user_id, method).await
    }

    /// Verify MFA token
    pub async fn verify_mfa(&self, user_id: &Uuid, token: &str) -> Result<bool> {
        self.mfa_manager.verify_mfa_token(user_id, token).await
    }

    /// Get authentication service statistics
    pub async fn get_stats(&self) -> AuthStats {
        AuthStats {
            active_sessions: self.session_manager.get_active_session_count().await,
            total_users: 0, // TODO: Query from database
            failed_login_attempts: self.rate_limiter.get_failed_attempts_count().await,
            mfa_enabled_users: self.mfa_manager.get_mfa_enabled_count().await,
        }
    }
}

/// Authentication result
#[derive(Debug, Clone)]
pub enum AuthenticationResult {
    Success {
        token: String,
        user_id: Uuid,
        session_id: String,
    },
    InvalidCredentials,
    MfaRequired { user_id: Uuid },
    RateLimited,
    AccountLocked,
    EmailNotVerified,
}

/// Validated user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedUser {
    pub user_id: Uuid,
    pub email: String,
    pub roles: Vec<String>,
    pub session_id: Option<String>,
}

/// Authentication statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStats {
    pub active_sessions: usize,
    pub total_users: u64,
    pub failed_login_attempts: u64,
    pub mfa_enabled_users: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_auth_service_creation() {
        let config = AuthConfig::default();
        let auth_service = AuthService::new(config).await;
        assert!(auth_service.is_ok());
    }

    #[test]
    fn test_auth_config_default() {
        let config = AuthConfig::default();
        assert!(config.enable_registration);
        assert_eq!(config.max_login_attempts, 5);
        assert_eq!(config.lockout_duration, std::time::Duration::from_secs(900));
    }
}
