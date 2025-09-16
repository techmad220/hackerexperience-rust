//! Multi-Factor Authentication (MFA) implementation with TOTP and backup codes

use anyhow::{anyhow, Result};
use base32::{Alphabet, encode as base32_encode};
use hmac::{Hmac, Mac};
use qrcode::{QrCode, render::svg};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use sqlx::{PgPool, postgres::PgPoolOptions};

/// MFA configuration
#[derive(Debug, Clone)]
pub struct MfaConfig {
    /// Enable MFA system
    pub enabled: bool,
    /// TOTP issuer name
    pub issuer: String,
    /// TOTP secret length
    pub secret_length: usize,
    /// TOTP time step in seconds
    pub time_step: u64,
    /// Number of TOTP digits
    pub totp_digits: usize,
    /// Allow TOTP time drift (steps before/after)
    pub time_drift_steps: u64,
    /// Number of backup codes to generate
    pub backup_codes_count: usize,
    /// Backup code length
    pub backup_code_length: usize,
    /// Require MFA for admin roles
    pub require_for_admin: bool,
    /// Allow SMS as MFA method
    pub allow_sms: bool,
    /// Allow email as MFA method
    pub allow_email: bool,
    /// Allow authenticator app
    pub allow_authenticator: bool,
    /// Allow hardware keys (WebAuthn)
    pub allow_hardware_keys: bool,
    /// MFA session timeout in seconds
    pub session_timeout: u64,
    /// Database URL for persistence
    pub database_url: Option<String>,
}

impl Default for MfaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            issuer: "HackerExperience".to_string(),
            secret_length: 32,
            time_step: 30,
            totp_digits: 6,
            time_drift_steps: 1,
            backup_codes_count: 10,
            backup_code_length: 8,
            require_for_admin: true,
            allow_sms: true,
            allow_email: true,
            allow_authenticator: true,
            allow_hardware_keys: false,
            session_timeout: 300, // 5 minutes
            database_url: None,
        }
    }
}

/// MFA methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MfaMethod {
    Authenticator,
    Sms,
    Email,
    BackupCode,
    HardwareKey,
}

impl MfaMethod {
    pub fn as_str(&self) -> &str {
        match self {
            MfaMethod::Authenticator => "authenticator",
            MfaMethod::Sms => "sms",
            MfaMethod::Email => "email",
            MfaMethod::BackupCode => "backup_code",
            MfaMethod::HardwareKey => "hardware_key",
        }
    }
}

/// MFA setup status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSetup {
    pub user_id: Uuid,
    pub method: MfaMethod,
    pub secret: Option<String>,
    pub backup_codes: Vec<String>,
    pub qr_code: Option<String>,
    pub provisioning_uri: Option<String>,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

/// MFA verification attempt
#[derive(Debug, Clone)]
struct MfaAttempt {
    user_id: Uuid,
    method: MfaMethod,
    attempts: u32,
    last_attempt: DateTime<Utc>,
    locked_until: Option<DateTime<Utc>>,
}

/// Active MFA session
#[derive(Debug, Clone)]
struct MfaSession {
    user_id: Uuid,
    session_id: String,
    method: MfaMethod,
    challenge: Option<String>,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    verified: bool,
}

/// MFA manager
pub struct MfaManager {
    config: MfaConfig,
    db_pool: Option<PgPool>,
    user_setups: Arc<RwLock<HashMap<Uuid, Vec<MfaSetup>>>>,
    active_sessions: Arc<RwLock<HashMap<String, MfaSession>>>,
    verification_attempts: Arc<RwLock<HashMap<Uuid, MfaAttempt>>>,
    used_backup_codes: Arc<RwLock<HashMap<Uuid, Vec<String>>>>,
}

impl MfaManager {
    /// Create a new MFA manager
    pub fn new(config: MfaConfig) -> Result<Self> {
        let manager = Self {
            config,
            db_pool: None,
            user_setups: Arc::new(RwLock::new(HashMap::new())),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            verification_attempts: Arc::new(RwLock::new(HashMap::new())),
            used_backup_codes: Arc::new(RwLock::new(HashMap::new())),
        };

        // Start cleanup task
        manager.start_cleanup_task();

        Ok(manager)
    }

    /// Create with database connection
    pub async fn with_database(config: MfaConfig, database_url: &str) -> Result<Self> {
        let db_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        let mut manager = Self::new(config)?;
        manager.db_pool = Some(db_pool);
        manager.initialize_database().await?;
        manager.load_from_database().await?;

        Ok(manager)
    }

    /// Initialize database tables
    async fn initialize_database(&self) -> Result<()> {
        if let Some(pool) = &self.db_pool {
            sqlx::query(
                r#"
                CREATE TABLE IF NOT EXISTS mfa_setups (
                    user_id UUID NOT NULL,
                    method VARCHAR(50) NOT NULL,
                    secret TEXT,
                    backup_codes TEXT[],
                    phone_number VARCHAR(20),
                    email VARCHAR(255),
                    verified BOOLEAN DEFAULT FALSE,
                    created_at TIMESTAMPTZ NOT NULL,
                    last_used TIMESTAMPTZ,
                    PRIMARY KEY (user_id, method)
                );

                CREATE TABLE IF NOT EXISTS mfa_used_backup_codes (
                    user_id UUID NOT NULL,
                    code VARCHAR(50) NOT NULL,
                    used_at TIMESTAMPTZ NOT NULL,
                    PRIMARY KEY (user_id, code)
                );

                CREATE TABLE IF NOT EXISTS mfa_sessions (
                    session_id VARCHAR(255) PRIMARY KEY,
                    user_id UUID NOT NULL,
                    method VARCHAR(50) NOT NULL,
                    challenge TEXT,
                    created_at TIMESTAMPTZ NOT NULL,
                    expires_at TIMESTAMPTZ NOT NULL,
                    verified BOOLEAN DEFAULT FALSE
                );

                CREATE INDEX IF NOT EXISTS idx_mfa_setups_user ON mfa_setups(user_id);
                CREATE INDEX IF NOT EXISTS idx_mfa_sessions_user ON mfa_sessions(user_id);
                CREATE INDEX IF NOT EXISTS idx_mfa_sessions_expires ON mfa_sessions(expires_at);
                "#
            )
            .execute(pool)
            .await?;

            info!("MFA database tables initialized");
        }
        Ok(())
    }

    /// Load MFA setups from database
    async fn load_from_database(&self) -> Result<()> {
        if let Some(pool) = &self.db_pool {
            let setups = sqlx::query(
                r#"
                SELECT user_id, method, secret, backup_codes, verified, created_at, last_used
                FROM mfa_setups
                WHERE verified = TRUE
                "#
            )
            .fetch_all(pool)
            .await?;

            let mut user_setups = self.user_setups.write().await;
            for setup in setups {
                let mfa_setup = MfaSetup {
                    user_id: setup.user_id,
                    method: match setup.method.as_str() {
                        "authenticator" => MfaMethod::Authenticator,
                        "sms" => MfaMethod::Sms,
                        "email" => MfaMethod::Email,
                        "backup_code" => MfaMethod::BackupCode,
                        "hardware_key" => MfaMethod::HardwareKey,
                        _ => continue,
                    },
                    secret: setup.secret,
                    backup_codes: setup.backup_codes.unwrap_or_default(),
                    qr_code: None,
                    provisioning_uri: None,
                    verified: setup.verified,
                    created_at: setup.created_at,
                    last_used: setup.last_used,
                };

                user_setups
                    .entry(setup.user_id)
                    .or_insert_with(Vec::new)
                    .push(mfa_setup);
            }

            // Load used backup codes
            let used_codes = sqlx::query(
                r#"
                SELECT user_id, code
                FROM mfa_used_backup_codes
                "#
            )
            .fetch_all(pool)
            .await?;

            let mut used_backup_codes = self.used_backup_codes.write().await;
            for code_entry in used_codes {
                used_backup_codes
                    .entry(code_entry.user_id)
                    .or_insert_with(Vec::new)
                    .push(code_entry.code);
            }

            info!("Loaded MFA setups for {} users", user_setups.len());
        }
        Ok(())
    }

    /// Setup TOTP authenticator for user
    pub async fn setup_authenticator(&self, user_id: &Uuid, email: &str) -> Result<MfaSetup> {
        if !self.config.allow_authenticator {
            return Err(anyhow!("Authenticator MFA method is not allowed"));
        }

        // Generate secret
        let secret = self.generate_totp_secret();
        let secret_base32 = base32_encode(
            Alphabet::RFC4648 { padding: false },
            secret.as_bytes()
        );

        // Generate backup codes
        let backup_codes = self.generate_backup_codes();

        // Create provisioning URI
        let provisioning_uri = format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}&digits={}&period={}",
            self.config.issuer,
            email,
            secret_base32,
            self.config.issuer,
            self.config.totp_digits,
            self.config.time_step
        );

        // Generate QR code
        let qr_code = self.generate_qr_code(&provisioning_uri)?;

        let setup = MfaSetup {
            user_id: *user_id,
            method: MfaMethod::Authenticator,
            secret: Some(secret.clone()),
            backup_codes: backup_codes.clone(),
            qr_code: Some(qr_code),
            provisioning_uri: Some(provisioning_uri),
            verified: false,
            created_at: Utc::now(),
            last_used: None,
        };

        // Store temporarily (not verified yet)
        let mut user_setups = self.user_setups.write().await;
        user_setups
            .entry(*user_id)
            .or_insert_with(Vec::new)
            .retain(|s| s.method != MfaMethod::Authenticator || s.verified);
        user_setups.get_mut(user_id).unwrap().push(setup.clone());

        info!("Authenticator setup initiated for user {}", user_id);
        Ok(setup)
    }

    /// Verify TOTP setup
    pub async fn verify_authenticator_setup(&self, user_id: &Uuid, code: &str) -> Result<bool> {
        let mut user_setups = self.user_setups.write().await;

        if let Some(setups) = user_setups.get_mut(user_id) {
            for setup in setups.iter_mut() {
                if setup.method == MfaMethod::Authenticator && !setup.verified {
                    if let Some(secret) = &setup.secret {
                        if self.verify_totp_code(secret, code)? {
                            setup.verified = true;

                            // Save to database
                            if let Some(pool) = &self.db_pool {
                                sqlx::query(
                                    r#"
                                    INSERT INTO mfa_setups (user_id, method, secret, backup_codes, verified, created_at)
                                    VALUES ($1, $2, $3, $4, $5, $6)
                                    ON CONFLICT (user_id, method)
                                    DO UPDATE SET
                                        secret = $3,
                                        backup_codes = $4,
                                        verified = $5
                                    "#
                                )
                                .bind(user_id)
                                .bind("authenticator")
                                .bind(secret)
                                .bind(&setup.backup_codes[..])
                                .bind(true)
                                .bind(setup.created_at)
                                .execute(pool)
                                .await?;
                            }

                            info!("Authenticator verified for user {}", user_id);
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    /// Setup SMS MFA
    pub async fn setup_sms(&self, user_id: &Uuid, phone_number: &str) -> Result<MfaSetup> {
        if !self.config.allow_sms {
            return Err(anyhow!("SMS MFA method is not allowed"));
        }

        // Validate phone number format
        if !self.validate_phone_number(phone_number) {
            return Err(anyhow!("Invalid phone number format"));
        }

        let backup_codes = self.generate_backup_codes();

        let setup = MfaSetup {
            user_id: *user_id,
            method: MfaMethod::Sms,
            secret: Some(phone_number.to_string()),
            backup_codes,
            qr_code: None,
            provisioning_uri: None,
            verified: false,
            created_at: Utc::now(),
            last_used: None,
        };

        // Send verification code via SMS
        let verification_code = self.generate_verification_code();
        self.send_sms_code(phone_number, &verification_code).await?;

        // Store setup and verification code
        let mut user_setups = self.user_setups.write().await;
        user_setups
            .entry(*user_id)
            .or_insert_with(Vec::new)
            .push(setup.clone());

        // Create verification session
        let session_id = Uuid::new_v4().to_string();
        let mut sessions = self.active_sessions.write().await;
        sessions.insert(session_id.clone(), MfaSession {
            user_id: *user_id,
            session_id: session_id.clone(),
            method: MfaMethod::Sms,
            challenge: Some(verification_code),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::seconds(self.config.session_timeout as i64),
            verified: false,
        });

        info!("SMS MFA setup initiated for user {}", user_id);
        Ok(setup)
    }

    /// Setup Email MFA
    pub async fn setup_email(&self, user_id: &Uuid, email: &str) -> Result<MfaSetup> {
        if !self.config.allow_email {
            return Err(anyhow!("Email MFA method is not allowed"));
        }

        let backup_codes = self.generate_backup_codes();

        let setup = MfaSetup {
            user_id: *user_id,
            method: MfaMethod::Email,
            secret: Some(email.to_string()),
            backup_codes,
            qr_code: None,
            provisioning_uri: None,
            verified: false,
            created_at: Utc::now(),
            last_used: None,
        };

        // Send verification code via email
        let verification_code = self.generate_verification_code();
        self.send_email_code(email, &verification_code).await?;

        // Store setup
        let mut user_setups = self.user_setups.write().await;
        user_setups
            .entry(*user_id)
            .or_insert_with(Vec::new)
            .push(setup.clone());

        // Create verification session
        let session_id = Uuid::new_v4().to_string();
        let mut sessions = self.active_sessions.write().await;
        sessions.insert(session_id.clone(), MfaSession {
            user_id: *user_id,
            session_id: session_id.clone(),
            method: MfaMethod::Email,
            challenge: Some(verification_code),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::seconds(self.config.session_timeout as i64),
            verified: false,
        });

        info!("Email MFA setup initiated for user {}", user_id);
        Ok(setup)
    }

    /// Start MFA verification process
    pub async fn start_mfa_process(&self, user_id: &Uuid, method: MfaMethod) -> Result<String> {
        // Check if user has MFA setup
        let user_setups = self.user_setups.read().await;
        let setups = user_setups.get(user_id)
            .ok_or_else(|| anyhow!("No MFA setup found for user"))?;

        let setup = setups.iter()
            .find(|s| s.method == method && s.verified)
            .ok_or_else(|| anyhow!("MFA method not setup or not verified"))?;

        let session_id = Uuid::new_v4().to_string();
        let mut challenge = None;

        // Generate and send challenge based on method
        match method {
            MfaMethod::Sms => {
                if let Some(phone) = &setup.secret {
                    let code = self.generate_verification_code();
                    self.send_sms_code(phone, &code).await?;
                    challenge = Some(code);
                }
            }
            MfaMethod::Email => {
                if let Some(email) = &setup.secret {
                    let code = self.generate_verification_code();
                    self.send_email_code(email, &code).await?;
                    challenge = Some(code);
                }
            }
            _ => {}
        }

        // Create MFA session
        let mut sessions = self.active_sessions.write().await;
        sessions.insert(session_id.clone(), MfaSession {
            user_id: *user_id,
            session_id: session_id.clone(),
            method,
            challenge,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::seconds(self.config.session_timeout as i64),
            verified: false,
        });

        info!("MFA process started for user {} with method {:?}", user_id, method);
        Ok(session_id)
    }

    /// Verify MFA token
    pub async fn verify_mfa_token(&self, user_id: &Uuid, token: &str) -> Result<bool> {
        // Check for rate limiting
        if !self.check_verification_attempts(user_id).await {
            return Err(anyhow!("Too many failed attempts. Please try again later"));
        }

        let user_setups = self.user_setups.read().await;
        let setups = user_setups.get(user_id)
            .ok_or_else(|| anyhow!("No MFA setup found for user"))?;

        // Try TOTP first
        for setup in setups {
            if setup.method == MfaMethod::Authenticator && setup.verified {
                if let Some(secret) = &setup.secret {
                    if self.verify_totp_code(secret, token)? {
                        self.update_last_used(user_id, MfaMethod::Authenticator).await?;
                        self.reset_verification_attempts(user_id).await;
                        return Ok(true);
                    }
                }
            }
        }

        // Try backup codes
        if self.verify_backup_code(user_id, token).await? {
            self.update_last_used(user_id, MfaMethod::BackupCode).await?;
            self.reset_verification_attempts(user_id).await;
            return Ok(true);
        }

        // Try active session challenges
        let sessions = self.active_sessions.read().await;
        for session in sessions.values() {
            if session.user_id == *user_id && !session.verified {
                if let Some(challenge) = &session.challenge {
                    if challenge == token && session.expires_at > Utc::now() {
                        drop(sessions);
                        let mut sessions = self.active_sessions.write().await;
                        if let Some(mut session) = sessions.get_mut(&session.session_id) {
                            session.verified = true;
                        }
                        self.update_last_used(user_id, session.method).await?;
                        self.reset_verification_attempts(user_id).await;
                        return Ok(true);
                    }
                }
            }
        }

        // Record failed attempt
        self.record_failed_attempt(user_id).await;
        Ok(false)
    }

    /// Generate TOTP secret
    fn generate_totp_secret(&self) -> String {
        let mut rng = thread_rng();
        let secret: String = (0..self.config.secret_length)
            .map(|_| {
                let idx = rng.gen_range(0..32);
                "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567".chars().nth(idx).unwrap()
            })
            .collect();
        secret
    }

    /// Generate backup codes
    fn generate_backup_codes(&self) -> Vec<String> {
        let mut rng = thread_rng();
        let mut codes = Vec::new();

        for _ in 0..self.config.backup_codes_count {
            let code: String = (0..self.config.backup_code_length)
                .map(|_| {
                    let idx = rng.gen_range(0..36);
                    "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().nth(idx).unwrap()
                })
                .collect();
            codes.push(code);
        }

        codes
    }

    /// Generate verification code
    fn generate_verification_code(&self) -> String {
        let mut rng = thread_rng();
        (0..6)
            .map(|_| rng.gen_range(0..10).to_string())
            .collect()
    }

    /// Generate QR code
    fn generate_qr_code(&self, data: &str) -> Result<String> {
        let code = QrCode::new(data)?;
        let svg = code.render::<svg::Color>()
            .min_dimensions(200, 200)
            .build();
        Ok(svg)
    }

    /// Verify TOTP code
    fn verify_totp_code(&self, secret: &str, code: &str) -> Result<bool> {
        let decoded_secret = base32::decode(
            Alphabet::RFC4648 { padding: false },
            secret
        ).ok_or_else(|| anyhow!("Invalid secret"))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        // Check current and adjacent time windows
        for i in 0..=(self.config.time_drift_steps * 2) {
            let time_offset = i as i64 - self.config.time_drift_steps as i64;
            let counter = (now / self.config.time_step) as i64 + time_offset;

            if self.generate_totp_code(&decoded_secret, counter as u64)? == code {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Generate TOTP code
    fn generate_totp_code(&self, secret: &[u8], counter: u64) -> Result<String> {
        let counter_bytes = counter.to_be_bytes();

        let mut mac = Hmac::<Sha1>::new_from_slice(secret)
            .map_err(|e| anyhow!("Invalid key: {}", e))?;
        mac.update(&counter_bytes);
        let result = mac.finalize();
        let code_bytes = result.into_bytes();

        let offset = (code_bytes[code_bytes.len() - 1] & 0x0f) as usize;
        let code = u32::from_be_bytes([
            code_bytes[offset] & 0x7f,
            code_bytes[offset + 1],
            code_bytes[offset + 2],
            code_bytes[offset + 3],
        ]);

        let code = code % 10u32.pow(self.config.totp_digits as u32);
        Ok(format!("{:0width$}", code, width = self.config.totp_digits))
    }

    /// Verify backup code
    async fn verify_backup_code(&self, user_id: &Uuid, code: &str) -> Result<bool> {
        // Check if code was already used
        {
            let used_codes = self.used_backup_codes.read().await;
            if let Some(user_codes) = used_codes.get(user_id) {
                if user_codes.contains(&code.to_string()) {
                    return Ok(false);
                }
            }
        }

        // Check if code is valid
        let user_setups = self.user_setups.read().await;
        if let Some(setups) = user_setups.get(user_id) {
            for setup in setups {
                if setup.backup_codes.contains(&code.to_string()) {
                    // Mark code as used
                    let mut used_codes = self.used_backup_codes.write().await;
                    used_codes
                        .entry(*user_id)
                        .or_insert_with(Vec::new)
                        .push(code.to_string());

                    // Save to database
                    if let Some(pool) = &self.db_pool {
                        sqlx::query(
                            r#"
                            INSERT INTO mfa_used_backup_codes (user_id, code, used_at)
                            VALUES ($1, $2, $3)
                            "#
                        )
                        .bind(user_id)
                        .bind(code)
                        .bind(Utc::now())
                        .execute(pool)
                        .await?;
                    }

                    info!("Backup code used for user {}", user_id);
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Check if user has MFA required
    pub async fn is_mfa_required(&self, user_id: &Uuid) -> Result<bool> {
        if !self.config.enabled {
            return Ok(false);
        }

        let user_setups = self.user_setups.read().await;
        if let Some(setups) = user_setups.get(user_id) {
            Ok(setups.iter().any(|s| s.verified))
        } else {
            Ok(false)
        }
    }

    /// Get user's MFA methods
    pub async fn get_user_mfa_methods(&self, user_id: &Uuid) -> Vec<MfaMethod> {
        let user_setups = self.user_setups.read().await;
        if let Some(setups) = user_setups.get(user_id) {
            setups.iter()
                .filter(|s| s.verified)
                .map(|s| s.method)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Disable MFA method
    pub async fn disable_mfa_method(&self, user_id: &Uuid, method: MfaMethod) -> Result<()> {
        let mut user_setups = self.user_setups.write().await;
        if let Some(setups) = user_setups.get_mut(user_id) {
            setups.retain(|s| s.method != method);

            // Update database
            if let Some(pool) = &self.db_pool {
                sqlx::query(
                    r#"
                    DELETE FROM mfa_setups
                    WHERE user_id = $1 AND method = $2
                    "#
                )
                .bind(user_id)
                .bind(method.as_str())
                .execute(pool)
                .await?;
            }

            info!("Disabled MFA method {:?} for user {}", method, user_id);
        }

        Ok(())
    }

    /// Regenerate backup codes
    pub async fn regenerate_backup_codes(&self, user_id: &Uuid) -> Result<Vec<String>> {
        let new_codes = self.generate_backup_codes();

        let mut user_setups = self.user_setups.write().await;
        if let Some(setups) = user_setups.get_mut(user_id) {
            for setup in setups {
                setup.backup_codes = new_codes.clone();
            }

            // Update database
            if let Some(pool) = &self.db_pool {
                for setup in setups {
                    sqlx::query(
                        r#"
                        UPDATE mfa_setups
                        SET backup_codes = $3
                        WHERE user_id = $1 AND method = $2
                        "#
                    )
                    .bind(user_id)
                    .bind(setup.method.as_str())
                    .bind(&new_codes[..])
                    .execute(pool)
                    .await?;
                }
            }

            // Clear used backup codes
            let mut used_codes = self.used_backup_codes.write().await;
            used_codes.remove(user_id);

            if let Some(pool) = &self.db_pool {
                sqlx::query(
                    r#"
                    DELETE FROM mfa_used_backup_codes
                    WHERE user_id = $1
                    "#
                )
                .bind(user_id)
                .execute(pool)
                .await?;
            }

            info!("Regenerated backup codes for user {}", user_id);
            Ok(new_codes)
        } else {
            Err(anyhow!("No MFA setup found for user"))
        }
    }

    /// Check verification attempts
    async fn check_verification_attempts(&self, user_id: &Uuid) -> bool {
        let attempts = self.verification_attempts.read().await;
        if let Some(attempt) = attempts.get(user_id) {
            if let Some(locked_until) = attempt.locked_until {
                return Utc::now() > locked_until;
            }
            return attempt.attempts < 5;
        }
        true
    }

    /// Record failed verification attempt
    async fn record_failed_attempt(&self, user_id: &Uuid) {
        let mut attempts = self.verification_attempts.write().await;
        let attempt = attempts.entry(*user_id).or_insert_with(|| MfaAttempt {
            user_id: *user_id,
            method: MfaMethod::Authenticator,
            attempts: 0,
            last_attempt: Utc::now(),
            locked_until: None,
        });

        attempt.attempts += 1;
        attempt.last_attempt = Utc::now();

        if attempt.attempts >= 5 {
            attempt.locked_until = Some(Utc::now() + Duration::minutes(15));
            warn!("MFA verification locked for user {} due to failed attempts", user_id);
        }
    }

    /// Reset verification attempts
    async fn reset_verification_attempts(&self, user_id: &Uuid) {
        let mut attempts = self.verification_attempts.write().await;
        attempts.remove(user_id);
    }

    /// Update last used timestamp
    async fn update_last_used(&self, user_id: &Uuid, method: MfaMethod) -> Result<()> {
        let mut user_setups = self.user_setups.write().await;
        if let Some(setups) = user_setups.get_mut(user_id) {
            for setup in setups {
                if setup.method == method {
                    setup.last_used = Some(Utc::now());

                    // Update database
                    if let Some(pool) = &self.db_pool {
                        sqlx::query(
                            r#"
                            UPDATE mfa_setups
                            SET last_used = $3
                            WHERE user_id = $1 AND method = $2
                            "#
                        )
                        .bind(user_id)
                        .bind(method.as_str())
                        .bind(Utc::now())
                        .execute(pool)
                        .await?;
                    }
                    break;
                }
            }
        }
        Ok(())
    }

    /// Validate phone number format
    fn validate_phone_number(&self, phone: &str) -> bool {
        // Basic validation - should be enhanced for production
        phone.len() >= 10 && phone.chars().all(|c| c.is_numeric() || c == '+' || c == '-')
    }

    /// Send SMS code (placeholder - integrate with SMS provider)
    async fn send_sms_code(&self, phone: &str, code: &str) -> Result<()> {
        // In production, integrate with Twilio, AWS SNS, etc.
        info!("SMS code {} would be sent to {}", code, phone);
        Ok(())
    }

    /// Send email code (placeholder - integrate with email provider)
    async fn send_email_code(&self, email: &str, code: &str) -> Result<()> {
        // In production, integrate with SendGrid, AWS SES, etc.
        info!("Email code {} would be sent to {}", code, email);
        Ok(())
    }

    /// Get MFA enabled count
    pub async fn get_mfa_enabled_count(&self) -> u64 {
        let user_setups = self.user_setups.read().await;
        user_setups.len() as u64
    }

    /// Start cleanup task
    fn start_cleanup_task(&self) {
        let sessions = self.active_sessions.clone();
        let attempts = self.verification_attempts.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

            loop {
                interval.tick().await;

                // Clean up expired sessions
                {
                    let mut sessions_guard = sessions.write().await;
                    let now = Utc::now();
                    sessions_guard.retain(|_, session| session.expires_at > now);
                }

                // Clean up old verification attempts
                {
                    let mut attempts_guard = attempts.write().await;
                    let cutoff = Utc::now() - Duration::hours(1);
                    attempts_guard.retain(|_, attempt| attempt.last_attempt > cutoff);
                }

                debug!("MFA cleanup completed");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_totp_generation_and_verification() {
        let config = MfaConfig::default();
        let manager = MfaManager::new(config).unwrap();

        let secret = manager.generate_totp_secret();
        let decoded = base32::decode(
            Alphabet::RFC4648 { padding: false },
            &secret
        ).unwrap();

        let counter = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() / 30;

        let code = manager.generate_totp_code(&decoded, counter).unwrap();
        assert_eq!(code.len(), 6);

        // Verify the code
        let verified = manager.verify_totp_code(&secret, &code).unwrap();
        assert!(verified);

        // Wrong code should fail
        let verified = manager.verify_totp_code(&secret, "000000").unwrap();
        assert!(!verified);
    }

    #[tokio::test]
    async fn test_backup_codes_generation() {
        let config = MfaConfig::default();
        let manager = MfaManager::new(config).unwrap();

        let codes = manager.generate_backup_codes();
        assert_eq!(codes.len(), 10);

        for code in &codes {
            assert_eq!(code.len(), 8);
            assert!(code.chars().all(|c| c.is_alphanumeric()));
        }

        // Codes should be unique
        let unique_codes: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(unique_codes.len(), codes.len());
    }

    #[tokio::test]
    async fn test_authenticator_setup() {
        let config = MfaConfig::default();
        let manager = MfaManager::new(config).unwrap();

        let user_id = Uuid::new_v4();
        let email = "test@example.com";

        let setup = manager.setup_authenticator(&user_id, email).await.unwrap();
        assert_eq!(setup.method, MfaMethod::Authenticator);
        assert!(!setup.verified);
        assert!(setup.secret.is_some());
        assert!(setup.provisioning_uri.is_some());
        assert!(setup.qr_code.is_some());
        assert_eq!(setup.backup_codes.len(), 10);
    }

    #[tokio::test]
    async fn test_mfa_required_check() {
        let config = MfaConfig::default();
        let manager = MfaManager::new(config).unwrap();

        let user_id = Uuid::new_v4();

        // Should not be required initially
        assert!(!manager.is_mfa_required(&user_id).await.unwrap());

        // Setup but don't verify
        manager.setup_authenticator(&user_id, "test@example.com").await.unwrap();
        assert!(!manager.is_mfa_required(&user_id).await.unwrap());

        // After verification it should be required
        // (Would need to verify setup first in real scenario)
    }

    #[tokio::test]
    async fn test_verification_attempts_limiting() {
        let config = MfaConfig::default();
        let manager = MfaManager::new(config).unwrap();

        let user_id = Uuid::new_v4();

        // Should allow initially
        assert!(manager.check_verification_attempts(&user_id).await);

        // Record multiple failures
        for _ in 0..5 {
            manager.record_failed_attempt(&user_id).await;
        }

        // Should be locked after 5 attempts
        assert!(!manager.check_verification_attempts(&user_id).await);

        // Reset should allow again
        manager.reset_verification_attempts(&user_id).await;
        assert!(manager.check_verification_attempts(&user_id).await);
    }
}