//! Password hashing and validation using Argon2id

use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Params, Version, Algorithm
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use zeroize::Zeroize;
use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc, Duration};

/// Password configuration with security best practices
#[derive(Debug, Clone)]
pub struct PasswordConfig {
    /// Minimum password length
    pub min_length: usize,
    /// Maximum password length
    pub max_length: usize,
    /// Require uppercase letters
    pub require_uppercase: bool,
    /// Require lowercase letters
    pub require_lowercase: bool,
    /// Require numbers
    pub require_numbers: bool,
    /// Require special characters
    pub require_special: bool,
    /// Password history count (prevent reuse)
    pub password_history_count: usize,
    /// Password expiration days (0 = never expires)
    pub expiration_days: u32,
    /// Argon2id memory cost (in KiB)
    pub argon2_memory_cost: u32,
    /// Argon2id time cost (iterations)
    pub argon2_time_cost: u32,
    /// Argon2id parallelism
    pub argon2_parallelism: u32,
    /// Check against common passwords list
    pub check_common_passwords: bool,
    /// Enforce password complexity score
    pub min_complexity_score: u8,
    /// Maximum login attempts before temporary lockout
    pub max_attempts: u32,
    /// Lockout duration in seconds
    pub lockout_duration: u64,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            min_length: 12,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special: true,
            password_history_count: 5,
            expiration_days: 90,
            argon2_memory_cost: 65536,  // 64 MiB
            argon2_time_cost: 3,
            argon2_parallelism: 4,
            check_common_passwords: true,
            min_complexity_score: 3,
            max_attempts: 5,
            lockout_duration: 900, // 15 minutes
        }
    }
}

/// Password strength levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PasswordStrength {
    VeryWeak = 0,
    Weak = 1,
    Fair = 2,
    Strong = 3,
    VeryStrong = 4,
}

/// Password validation result
#[derive(Debug, Clone)]
pub struct PasswordValidation {
    pub is_valid: bool,
    pub strength: PasswordStrength,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub entropy: f64,
    pub score: u8,
}

/// Password history entry
#[derive(Debug, Clone)]
struct PasswordHistoryEntry {
    hash: String,
    created_at: DateTime<Utc>,
    expired_at: Option<DateTime<Utc>>,
}

/// Failed login attempt tracking
#[derive(Debug, Clone)]
struct LoginAttempt {
    count: u32,
    last_attempt: DateTime<Utc>,
    locked_until: Option<DateTime<Utc>>,
}

/// Password manager with Argon2id
pub struct PasswordManager {
    config: PasswordConfig,
    argon2: Argon2<'static>,
    common_passwords: Arc<RwLock<HashSet<String>>>,
    password_history: Arc<RwLock<HashMap<String, Vec<PasswordHistoryEntry>>>>,
    login_attempts: Arc<RwLock<HashMap<String, LoginAttempt>>>,
}

use std::collections::HashSet;

impl PasswordManager {
    /// Create a new password manager
    pub fn new(config: PasswordConfig) -> Self {
        // Configure Argon2id with production parameters
        let params = Params::new(
            config.argon2_memory_cost,
            config.argon2_time_cost,
            config.argon2_parallelism,
            None
        ).expect("Invalid Argon2 parameters");

        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            params
        );

        let mut manager = Self {
            config,
            argon2,
            common_passwords: Arc::new(RwLock::new(HashSet::new())),
            password_history: Arc::new(RwLock::new(HashMap::new())),
            login_attempts: Arc::new(RwLock::new(HashMap::new())),
        };

        // Load common passwords list
        manager.load_common_passwords();

        // Start cleanup task for expired attempts
        manager.start_cleanup_task();

        manager
    }

    /// Hash a password using Argon2id
    pub async fn hash_password(&self, password: &str) -> Result<String> {
        // Validate password first
        let validation = self.validate_password(password).await;
        if !validation.is_valid {
            return Err(anyhow!("Password validation failed: {:?}", validation.errors));
        }

        // Generate salt
        let salt = SaltString::generate(&mut OsRng);

        // Hash password with Argon2id
        let password_hash = self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Failed to hash password: {}", e))?
            .to_string();

        debug!("Password hashed successfully with Argon2id");
        Ok(password_hash)
    }

    /// Verify a password against a hash
    pub async fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| anyhow!("Invalid password hash format: {}", e))?;

        match self.argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => {
                debug!("Password verified successfully");
                Ok(true)
            }
            Err(_) => {
                debug!("Password verification failed");
                Ok(false)
            }
        }
    }

    /// Validate password against policy
    pub async fn validate_password(&self, password: &str) -> PasswordValidation {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check length
        if password.len() < self.config.min_length {
            errors.push(format!("Password must be at least {} characters", self.config.min_length));
        }
        if password.len() > self.config.max_length {
            errors.push(format!("Password must be at most {} characters", self.config.max_length));
        }

        // Check character requirements
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_numeric());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        if self.config.require_uppercase && !has_uppercase {
            errors.push("Password must contain at least one uppercase letter".to_string());
        }
        if self.config.require_lowercase && !has_lowercase {
            errors.push("Password must contain at least one lowercase letter".to_string());
        }
        if self.config.require_numbers && !has_digit {
            errors.push("Password must contain at least one number".to_string());
        }
        if self.config.require_special && !has_special {
            errors.push("Password must contain at least one special character".to_string());
        }

        // Check against common passwords
        if self.config.check_common_passwords {
            let common_passwords = self.common_passwords.read().await;
            if common_passwords.contains(&password.to_lowercase()) {
                errors.push("Password is too common and easily guessable".to_string());
            }
        }

        // Check for repeated characters
        if has_excessive_repetition(password) {
            warnings.push("Password contains excessive character repetition".to_string());
        }

        // Check for sequential characters
        if has_sequential_chars(password) {
            warnings.push("Password contains sequential characters".to_string());
        }

        // Calculate entropy and strength
        let entropy = calculate_entropy(password);
        let strength = calculate_strength(password, &errors, entropy);
        let score = strength as u8;

        // Check minimum complexity score
        if score < self.config.min_complexity_score {
            errors.push(format!(
                "Password complexity score {} is below minimum required {}",
                score, self.config.min_complexity_score
            ));
        }

        PasswordValidation {
            is_valid: errors.is_empty(),
            strength,
            errors,
            warnings,
            entropy,
            score,
        }
    }

    /// Check if password was recently used
    pub async fn check_password_history(&self, user_id: &str, password: &str) -> Result<bool> {
        if self.config.password_history_count == 0 {
            return Ok(false);
        }

        let history = self.password_history.read().await;
        if let Some(user_history) = history.get(user_id) {
            // Check against recent password hashes
            for entry in user_history.iter().take(self.config.password_history_count) {
                if self.verify_password(password, &entry.hash).await? {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Add password to user's history
    pub async fn add_to_history(&self, user_id: &str, hash: String) -> Result<()> {
        let mut history = self.password_history.write().await;
        let user_history = history.entry(user_id.to_string()).or_insert_with(Vec::new);

        user_history.insert(0, PasswordHistoryEntry {
            hash,
            created_at: Utc::now(),
            expired_at: None,
        });

        // Keep only configured number of entries
        if user_history.len() > self.config.password_history_count {
            user_history.truncate(self.config.password_history_count);
        }

        Ok(())
    }

    /// Check if password has expired
    pub async fn is_password_expired(&self, created_at: DateTime<Utc>) -> bool {
        if self.config.expiration_days == 0 {
            return false;
        }

        let expiration_duration = Duration::days(self.config.expiration_days as i64);
        Utc::now() > created_at + expiration_duration
    }

    /// Record login attempt
    pub async fn record_login_attempt(&self, identifier: &str, success: bool) -> Result<()> {
        let mut attempts = self.login_attempts.write().await;
        let attempt = attempts.entry(identifier.to_string()).or_insert_with(|| LoginAttempt {
            count: 0,
            last_attempt: Utc::now(),
            locked_until: None,
        });

        if success {
            // Reset on successful login
            attempt.count = 0;
            attempt.locked_until = None;
        } else {
            attempt.count += 1;
            attempt.last_attempt = Utc::now();

            // Check if should lock account
            if attempt.count >= self.config.max_attempts {
                let lockout_duration = Duration::seconds(self.config.lockout_duration as i64);
                attempt.locked_until = Some(Utc::now() + lockout_duration);
                warn!("Account {} locked due to {} failed attempts", identifier, attempt.count);
            }
        }

        Ok(())
    }

    /// Check if account is locked
    pub async fn is_account_locked(&self, identifier: &str) -> bool {
        let attempts = self.login_attempts.read().await;
        if let Some(attempt) = attempts.get(identifier) {
            if let Some(locked_until) = attempt.locked_until {
                return Utc::now() < locked_until;
            }
        }
        false
    }

    /// Get remaining lockout time
    pub async fn get_lockout_remaining(&self, identifier: &str) -> Option<Duration> {
        let attempts = self.login_attempts.read().await;
        if let Some(attempt) = attempts.get(identifier) {
            if let Some(locked_until) = attempt.locked_until {
                if Utc::now() < locked_until {
                    return Some(locked_until - Utc::now());
                }
            }
        }
        None
    }

    /// Generate secure random password
    pub fn generate_password(&self, length: usize) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let charset = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+-=[]{}|;:,.<>?";
        let password: String = (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..charset.len());
                charset.chars().nth(idx).unwrap()
            })
            .collect();

        password
    }

    /// Load common passwords list
    fn load_common_passwords(&self) {
        let common_passwords_list = vec![
            "password", "123456", "password123", "admin", "letmein",
            "qwerty", "abc123", "monkey", "1234567890", "123456789",
            "welcome", "password1", "p@ssword", "123qwe", "123321",
            "1234567", "123123", "1234", "12345", "iloveyou",
            "admin123", "root", "toor", "pass", "test", "guest",
            "master", "dragon", "baseball", "football", "letmein123",
            "welcome123", "admin@123", "root123", "pass123", "test123",
            "qwerty123", "zxcvbnm", "qazwsx", "123qweasd", "123456a",
            "123456789a", "qwertyuiop", "mypassword", "password12",
            "hello123", "winter2023", "summer2023", "spring2023",
            "password2023", "password2024", "hackerexperience", "hexperience"
        ];

        let common_passwords = self.common_passwords.clone();
        tokio::spawn(async move {
            let mut passwords = common_passwords.write().await;
            for pwd in common_passwords_list {
                passwords.insert(pwd.to_string());
            }
            info!("Loaded {} common passwords", passwords.len());
        });
    }

    /// Start cleanup task for expired lockouts
    fn start_cleanup_task(&self) {
        let attempts = self.login_attempts.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

            loop {
                interval.tick().await;

                let mut attempts_guard = attempts.write().await;
                let now = Utc::now();

                // Remove expired lockouts
                attempts_guard.retain(|_, attempt| {
                    if let Some(locked_until) = attempt.locked_until {
                        locked_until > now
                    } else {
                        // Keep attempts for 1 hour for rate limiting
                        now - attempt.last_attempt < Duration::hours(1)
                    }
                });
            }
        });
    }

    /// Securely update password
    pub async fn update_password(
        &self,
        user_id: &str,
        current_password: &str,
        new_password: &str,
        current_hash: &str,
    ) -> Result<String> {
        // Verify current password
        if !self.verify_password(current_password, current_hash).await? {
            return Err(anyhow!("Current password is incorrect"));
        }

        // Check if new password is same as current
        if current_password == new_password {
            return Err(anyhow!("New password must be different from current password"));
        }

        // Check password history
        if self.check_password_history(user_id, new_password).await? {
            return Err(anyhow!(
                "This password was recently used. Please choose a different password"
            ));
        }

        // Validate and hash new password
        let new_hash = self.hash_password(new_password).await?;

        // Add to history
        self.add_to_history(user_id, new_hash.clone()).await?;

        Ok(new_hash)
    }

    /// Get password policy for client display
    pub fn get_password_policy(&self) -> PasswordPolicy {
        PasswordPolicy {
            min_length: self.config.min_length,
            max_length: self.config.max_length,
            require_uppercase: self.config.require_uppercase,
            require_lowercase: self.config.require_lowercase,
            require_numbers: self.config.require_numbers,
            require_special: self.config.require_special,
            min_complexity_score: self.config.min_complexity_score,
            expiration_days: self.config.expiration_days,
        }
    }
}

/// Password policy for client display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: usize,
    pub max_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special: bool,
    pub min_complexity_score: u8,
    pub expiration_days: u32,
}

/// Calculate password entropy
fn calculate_entropy(password: &str) -> f64 {
    let mut charset_size = 0;
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if has_lowercase {
        charset_size += 26;
    }
    if has_uppercase {
        charset_size += 26;
    }
    if has_digit {
        charset_size += 10;
    }
    if has_special {
        charset_size += 32;
    }

    if charset_size == 0 {
        return 0.0;
    }

    let entropy_per_char = (charset_size as f64).log2();
    entropy_per_char * password.len() as f64
}

/// Calculate password strength
fn calculate_strength(password: &str, errors: &[String], entropy: f64) -> PasswordStrength {
    if !errors.is_empty() {
        return PasswordStrength::VeryWeak;
    }

    match entropy {
        e if e < 30.0 => PasswordStrength::VeryWeak,
        e if e < 40.0 => PasswordStrength::Weak,
        e if e < 50.0 => PasswordStrength::Fair,
        e if e < 60.0 => PasswordStrength::Strong,
        _ => PasswordStrength::VeryStrong,
    }
}

/// Check for excessive character repetition
fn has_excessive_repetition(password: &str) -> bool {
    let chars: Vec<char> = password.chars().collect();
    for i in 0..chars.len().saturating_sub(2) {
        if chars[i] == chars[i + 1] && chars[i] == chars[i + 2] {
            return true;
        }
    }
    false
}

/// Check for sequential characters
fn has_sequential_chars(password: &str) -> bool {
    let sequences = ["abcdefghijklmnopqrstuvwxyz", "0123456789", "qwertyuiop", "asdfghjkl", "zxcvbnm"];
    let lower = password.to_lowercase();

    for seq in sequences {
        for window in seq.as_bytes().windows(3) {
            let seq_str = std::str::from_utf8(window).unwrap();
            if lower.contains(seq_str) {
                return true;
            }
        }
    }
    false
}

/// Secure string that zeros memory on drop
pub struct SecureString {
    inner: String,
}

impl SecureString {
    pub fn new(s: String) -> Self {
        Self { inner: s }
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        self.inner.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_password_hashing() {
        let config = PasswordConfig::default();
        let manager = PasswordManager::new(config);

        let password = "MySecureP@ssw0rd123!";
        let hash = manager.hash_password(password).await.unwrap();

        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2id"));

        // Verify password
        assert!(manager.verify_password(password, &hash).await.unwrap());
        assert!(!manager.verify_password("wrong", &hash).await.unwrap());
    }

    #[tokio::test]
    async fn test_password_validation() {
        let config = PasswordConfig::default();
        let manager = PasswordManager::new(config);

        // Test weak password
        let validation = manager.validate_password("password").await;
        assert!(!validation.is_valid);
        assert!(!validation.errors.is_empty());

        // Test strong password
        let validation = manager.validate_password("MyV3ryS3cur3P@ssw0rd!").await;
        assert!(validation.is_valid);
        assert!(validation.errors.is_empty());
        assert_eq!(validation.strength, PasswordStrength::VeryStrong);
    }

    #[tokio::test]
    async fn test_password_history() {
        let config = PasswordConfig {
            password_history_count: 3,
            ..Default::default()
        };
        let manager = PasswordManager::new(config);

        let user_id = "test_user";
        let passwords = ["OldP@ssw0rd1!", "OldP@ssw0rd2!", "OldP@ssw0rd3!"];

        // Add passwords to history
        for pwd in &passwords {
            let hash = manager.hash_password(pwd).await.unwrap();
            manager.add_to_history(user_id, hash).await.unwrap();
        }

        // Check that old passwords are rejected
        for pwd in &passwords {
            assert!(manager.check_password_history(user_id, pwd).await.unwrap());
        }

        // New password should be allowed
        assert!(!manager.check_password_history(user_id, "NewP@ssw0rd4!").await.unwrap());
    }

    #[tokio::test]
    async fn test_account_lockout() {
        let config = PasswordConfig {
            max_attempts: 3,
            lockout_duration: 2, // 2 seconds for testing
            ..Default::default()
        };
        let manager = PasswordManager::new(config);

        let identifier = "test@example.com";

        // Record failed attempts
        for _ in 0..3 {
            manager.record_login_attempt(identifier, false).await.unwrap();
        }

        // Should be locked
        assert!(manager.is_account_locked(identifier).await);
        assert!(manager.get_lockout_remaining(identifier).await.is_some());

        // Wait for lockout to expire
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        // Should be unlocked
        assert!(!manager.is_account_locked(identifier).await);
    }

    #[test]
    fn test_entropy_calculation() {
        assert!(calculate_entropy("password") < 30.0);
        assert!(calculate_entropy("P@ssw0rd") > 40.0);
        assert!(calculate_entropy("MyV3ryS3cur3P@ssw0rd!") > 60.0);
    }

    #[test]
    fn test_repetition_detection() {
        assert!(has_excessive_repetition("passsword"));
        assert!(has_excessive_repetition("111222333"));
        assert!(!has_excessive_repetition("password"));
    }

    #[test]
    fn test_sequential_detection() {
        assert!(has_sequential_chars("abc123"));
        assert!(has_sequential_chars("qwerty"));
        assert!(has_sequential_chars("789xyz"));
        assert!(!has_sequential_chars("p@ssw0rd"));
    }

    #[test]
    fn test_password_generation() {
        let config = PasswordConfig::default();
        let manager = PasswordManager::new(config);

        let password = manager.generate_password(16);
        assert_eq!(password.len(), 16);

        // Check it contains various character types
        assert!(password.chars().any(|c| c.is_uppercase()));
        assert!(password.chars().any(|c| c.is_lowercase()));
        assert!(password.chars().any(|c| c.is_numeric()));
    }
}