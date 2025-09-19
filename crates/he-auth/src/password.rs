//! Password management with Argon2id
//!
//! Implements secure password hashing and verification using Argon2id,
//! which provides both resistance to side-channel attacks (from Argon2i)
//! and resistance to GPU-based attacks (from Argon2d).

use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2, Argon2id, Algorithm, Params, Version
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, warn};
use zeroize::Zeroize;

/// Password configuration
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
    /// Argon2 memory cost (in KiB)
    pub argon2_memory_cost: u32,
    /// Argon2 time cost (iterations)
    pub argon2_time_cost: u32,
    /// Argon2 parallelism factor
    pub argon2_parallelism: u32,
    /// Salt length for Argon2
    pub salt_length: usize,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            min_length: 8,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special: false,
            // Argon2id recommended parameters for interactive use
            argon2_memory_cost: 65536,  // 64 MiB
            argon2_time_cost: 3,         // 3 iterations
            argon2_parallelism: 4,       // 4 parallel threads
            salt_length: 32,             // 32 bytes salt
        }
    }
}

/// Password strength levels
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PasswordStrength {
    VeryWeak,
    Weak,
    Fair,
    Strong,
    VeryStrong,
}

/// Password manager for hashing and verification
#[derive(Debug, Clone)]
pub struct PasswordManager {
    config: PasswordConfig,
    argon2: Argon2<'static>,
}

impl PasswordManager {
    /// Create a new password manager
    pub fn new(config: PasswordConfig) -> Self {
        // Create Argon2id instance with custom parameters
        let params = Params::new(
            config.argon2_memory_cost,
            config.argon2_time_cost,
            config.argon2_parallelism,
            None
        ).unwrap_or_else(|_| Params::default());

        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            params,
        );

        Self { config, argon2 }
    }

    /// Hash a password using Argon2id
    pub async fn hash_password(&self, password: &str) -> Result<String> {
        // Validate password
        self.validate_password(password)?;

        // Generate a random salt
        let salt = SaltString::generate(&mut OsRng);

        // Hash the password
        let password_hash = self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Failed to hash password: {}", e))?;

        debug!("Password hashed successfully with Argon2id");
        Ok(password_hash.to_string())
    }

    /// Verify a password against an Argon2id hash
    pub async fn verify_password_argon2id(&self, password: &str, hash: &str) -> Result<bool> {
        // Parse the password hash
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| anyhow!("Invalid password hash format: {}", e))?;

        // Verify the password
        match self.argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => {
                debug!("Password verified successfully");
                Ok(true)
            }
            Err(_) => {
                debug!("Password verification failed");
                Ok(false)
            }
        }
    }

    /// Legacy verify method (for compatibility)
    pub async fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        self.verify_password_argon2id(password, hash).await
    }

    /// Validate password against configured rules
    pub fn validate_password(&self, password: &str) -> Result<()> {
        // Check length
        if password.len() < self.config.min_length {
            return Err(anyhow!(
                "Password must be at least {} characters long",
                self.config.min_length
            ));
        }

        if password.len() > self.config.max_length {
            return Err(anyhow!(
                "Password must be no more than {} characters long",
                self.config.max_length
            ));
        }

        // Check character requirements
        if self.config.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(anyhow!("Password must contain at least one uppercase letter"));
        }

        if self.config.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(anyhow!("Password must contain at least one lowercase letter"));
        }

        if self.config.require_numbers && !password.chars().any(|c| c.is_numeric()) {
            return Err(anyhow!("Password must contain at least one number"));
        }

        if self.config.require_special {
            let has_special = password.chars().any(|c| {
                !c.is_alphanumeric() && !c.is_whitespace()
            });
            if !has_special {
                return Err(anyhow!("Password must contain at least one special character"));
            }
        }

        Ok(())
    }

    /// Check password strength
    pub fn check_strength(&self, password: &str) -> PasswordStrength {
        let mut score = 0;

        // Length scoring
        if password.len() >= 8 {
            score += 1;
        }
        if password.len() >= 12 {
            score += 1;
        }
        if password.len() >= 16 {
            score += 1;
        }

        // Character diversity scoring
        if password.chars().any(|c| c.is_lowercase()) {
            score += 1;
        }
        if password.chars().any(|c| c.is_uppercase()) {
            score += 1;
        }
        if password.chars().any(|c| c.is_numeric()) {
            score += 1;
        }
        if password.chars().any(|c| !c.is_alphanumeric()) {
            score += 2;
        }

        // Entropy estimation
        let unique_chars = password.chars().collect::<std::collections::HashSet<_>>().len();
        if unique_chars >= password.len() / 2 {
            score += 1;
        }

        match score {
            0..=2 => PasswordStrength::VeryWeak,
            3..=4 => PasswordStrength::Weak,
            5..=6 => PasswordStrength::Fair,
            7..=8 => PasswordStrength::Strong,
            _ => PasswordStrength::VeryStrong,
        }
    }

    /// Generate a secure random password
    pub fn generate_password(&self, length: usize) -> String {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let charset: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+-=[]{}|;:,.<>?"
            .chars()
            .collect();

        let mut password = String::with_capacity(length);

        // Ensure at least one of each required type
        if self.config.require_uppercase {
            password.push(rng.gen_range('A'..='Z'));
        }
        if self.config.require_lowercase {
            password.push(rng.gen_range('a'..='z'));
        }
        if self.config.require_numbers {
            password.push(rng.gen_range('0'..='9'));
        }
        if self.config.require_special {
            let specials = "!@#$%^&*()_+-=[]{}|;:,.<>?";
            password.push(specials.chars().nth(rng.gen_range(0..specials.len())).map_err(|e| anyhow::anyhow!("Error: {}", e))?);
        }

        // Fill the rest randomly
        while password.len() < length {
            password.push(charset[rng.gen_range(0..charset.len())]);
        }

        // Shuffle the password
        let mut chars: Vec<char> = password.chars().collect();
        use rand::seq::SliceRandom;
        chars.shuffle(&mut rng);

        chars.into_iter().collect()
    }

    /// Check if a password needs rehashing (e.g., if parameters changed)
    pub fn needs_rehash(&self, hash: &str) -> bool {
        match PasswordHash::new(hash) {
            Ok(parsed) => {
                // Check if parameters match current configuration
                if let Some(params) = parsed.params() {
                    params.m_cost() != self.config.argon2_memory_cost ||
                    params.t_cost() != self.config.argon2_time_cost ||
                    params.p_cost() != self.config.argon2_parallelism
                } else {
                    true // No params found, needs rehash
                }
            }
            Err(_) => true, // Invalid hash, needs rehash
        }
    }
}

/// Secure password wrapper that zeroizes on drop
pub struct SecurePassword {
    inner: String,
}

impl SecurePassword {
    pub fn new(password: String) -> Self {
        Self { inner: password }
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl Drop for SecurePassword {
    fn drop(&mut self) {
        self.inner.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_password_hashing_and_verification() {
        let manager = PasswordManager::new(PasswordConfig::default());

        let password = "SecurePassword123!";
        let hash = manager.hash_password(password).await.map_err(|e| anyhow::anyhow!("Error: {}", e))?;

        assert!(manager.verify_password_argon2id(password, &hash).await.map_err(|e| anyhow::anyhow!("Error: {}", e))?);
        assert!(!manager.verify_password_argon2id("WrongPassword", &hash).await.map_err(|e| anyhow::anyhow!("Error: {}", e))?);
    }

    #[test]
    fn test_password_validation() {
        let manager = PasswordManager::new(PasswordConfig::default());

        assert!(manager.validate_password("Short").is_err());
        assert!(manager.validate_password("nouppercase123").is_err());
        assert!(manager.validate_password("NOLOWERCASE123").is_err());
        assert!(manager.validate_password("NoNumbers!").is_err());
        assert!(manager.validate_password("ValidPass123").is_ok());
    }

    #[test]
    fn test_password_strength() {
        let manager = PasswordManager::new(PasswordConfig::default());

        assert_eq!(manager.check_strength("weak"), PasswordStrength::VeryWeak);
        assert_eq!(manager.check_strength("Weak123"), PasswordStrength::Weak);
        assert_eq!(manager.check_strength("Better123"), PasswordStrength::Fair);
        assert_eq!(manager.check_strength("VeryGood123!"), PasswordStrength::Strong);
        assert_eq!(manager.check_strength("ExcellentP@ssw0rd!123"), PasswordStrength::VeryStrong);
    }

    #[test]
    fn test_password_generation() {
        let manager = PasswordManager::new(PasswordConfig::default());

        let password = manager.generate_password(16);
        assert_eq!(password.len(), 16);
        assert!(manager.validate_password(&password).is_ok());
    }

    #[test]
    fn test_needs_rehash() {
        let manager = PasswordManager::new(PasswordConfig::default());

        // Old hash with different parameters
        let old_hash = "$argon2id$v=19$m=4096,t=3,p=1$salt$hash";
        assert!(manager.needs_rehash(old_hash));
    }
}