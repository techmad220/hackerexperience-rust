use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BcryptError {
    #[error("Hashing error: {0}")]
    Hash(#[from] bcrypt::BcryptError),
    #[error("Invalid cost parameter: {0}")]
    InvalidCost(u32),
    #[error("Verification failed")]
    VerificationFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BcryptConfig {
    pub cost: u32,
    pub prefix: String,
}

impl Default for BcryptConfig {
    fn default() -> Self {
        Self {
            cost: DEFAULT_COST,
            prefix: "$2y$".to_string(),
        }
    }
}

/// BCrypt password hashing utility ported from PHP BCrypt class
/// Provides secure password hashing and verification
pub struct BCrypt {
    config: BcryptConfig,
}

impl BCrypt {
    /// Create new BCrypt instance with configuration
    pub fn new(config: BcryptConfig) -> Result<Self, BcryptError> {
        // Validate cost parameter (should be between 4 and 31)
        if config.cost < 4 || config.cost > 31 {
            return Err(BcryptError::InvalidCost(config.cost));
        }

        Ok(Self { config })
    }

    /// Create BCrypt instance with default configuration
    pub fn default() -> Self {
        Self {
            config: BcryptConfig::default(),
        }
    }

    /// Create BCrypt instance with custom cost
    pub fn with_cost(cost: u32) -> Result<Self, BcryptError> {
        let config = BcryptConfig {
            cost,
            ..Default::default()
        };
        Self::new(config)
    }

    /// Hash a password using BCrypt
    pub fn hash_password(&self, password: &str) -> Result<String, BcryptError> {
        let hashed = hash(password, self.config.cost)?;
        Ok(hashed)
    }

    /// Verify a password against a hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, BcryptError> {
        match verify(password, hash) {
            Ok(is_valid) => Ok(is_valid),
            Err(e) => Err(BcryptError::Hash(e)),
        }
    }

    /// Check if a hash needs rehashing (due to cost change)
    pub fn needs_rehash(&self, hash: &str) -> bool {
        // Extract cost from hash string
        if let Some(cost) = self.extract_cost_from_hash(hash) {
            cost != self.config.cost
        } else {
            true // Invalid hash format, needs rehashing
        }
    }

    /// Extract cost parameter from hash string
    fn extract_cost_from_hash(&self, hash: &str) -> Option<u32> {
        // BCrypt hash format: $2y$10$saltsaltsa...
        let parts: Vec<&str> = hash.split('$').collect();
        if parts.len() >= 3 {
            parts[2].parse().ok()
        } else {
            None
        }
    }

    /// Generate a random salt (for manual salt handling if needed)
    pub fn generate_salt(&self) -> Result<String, BcryptError> {
        use rand::{distributions::Alphanumeric, Rng};
        
        let salt: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(22)
            .map(char::from)
            .collect();
            
        Ok(salt)
    }

    /// Hash password with custom salt (mainly for legacy compatibility)
    pub fn hash_with_salt(&self, password: &str, salt: &str) -> Result<String, BcryptError> {
        // For BCrypt, we typically don't use custom salts as they're generated automatically
        // This method is provided for legacy compatibility
        self.hash_password(password)
    }

    /// Timing-safe string comparison
    pub fn timing_safe_equals(&self, a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let mut result = 0u8;
        for (byte_a, byte_b) in a.bytes().zip(b.bytes()) {
            result |= byte_a ^ byte_b;
        }

        result == 0
    }

    /// Get current cost setting
    pub fn get_cost(&self) -> u32 {
        self.config.cost
    }

    /// Set new cost (creates new instance)
    pub fn with_new_cost(&self, cost: u32) -> Result<Self, BcryptError> {
        let mut new_config = self.config.clone();
        new_config.cost = cost;
        Self::new(new_config)
    }

    /// Check if hash is valid BCrypt format
    pub fn is_valid_hash(&self, hash: &str) -> bool {
        // Basic BCrypt hash format validation
        let bcrypt_regex = regex::Regex::new(r"^\$2[ayb]\$\d{2}\$[A-Za-z0-9./]{53}$");
        match bcrypt_regex {
            Ok(regex) => regex.is_match(hash),
            Err(_) => false,
        }
    }

    /// Calculate time to hash (for performance testing)
    pub fn time_hash(&self, password: &str) -> Result<(String, std::time::Duration), BcryptError> {
        let start = std::time::Instant::now();
        let hash = self.hash_password(password)?;
        let duration = start.elapsed();
        Ok((hash, duration))
    }

    /// Benchmark different cost values
    pub fn benchmark_cost(password: &str, max_cost: u32) -> Result<Vec<(u32, std::time::Duration)>, BcryptError> {
        let mut results = vec![];
        
        for cost in 4..=max_cost {
            let bcrypt = Self::with_cost(cost)?;
            let (_, duration) = bcrypt.time_hash(password)?;
            results.push((cost, duration));
        }
        
        Ok(results)
    }

    /// Legacy hash method for compatibility with old PHP code
    pub fn legacy_hash(&self, password: &str, rounds: Option<u32>) -> Result<String, BcryptError> {
        let cost = rounds.unwrap_or(self.config.cost);
        let bcrypt = Self::with_cost(cost)?;
        bcrypt.hash_password(password)
    }

    /// Legacy verify method for compatibility with old PHP code
    pub fn legacy_verify(&self, password: &str, hash: &str) -> bool {
        self.verify_password(password, hash).unwrap_or(false)
    }
}

/// Utility functions for password strength validation
impl BCrypt {
    /// Check password strength
    pub fn check_password_strength(&self, password: &str) -> PasswordStrength {
        let mut score = 0;
        let mut feedback = vec![];

        // Length check
        if password.len() >= 8 {
            score += 1;
        } else {
            feedback.push("Password should be at least 8 characters long".to_string());
        }

        if password.len() >= 12 {
            score += 1;
        }

        // Character variety checks
        if password.chars().any(|c| c.is_lowercase()) {
            score += 1;
        } else {
            feedback.push("Password should contain lowercase letters".to_string());
        }

        if password.chars().any(|c| c.is_uppercase()) {
            score += 1;
        } else {
            feedback.push("Password should contain uppercase letters".to_string());
        }

        if password.chars().any(|c| c.is_numeric()) {
            score += 1;
        } else {
            feedback.push("Password should contain numbers".to_string());
        }

        if password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)) {
            score += 1;
        } else {
            feedback.push("Password should contain special characters".to_string());
        }

        // Common password check
        if !self.is_common_password(password) {
            score += 1;
        } else {
            feedback.push("Password is too common".to_string());
        }

        let strength = match score {
            0..=2 => PasswordStrengthLevel::Weak,
            3..=4 => PasswordStrengthLevel::Fair,
            5..=6 => PasswordStrengthLevel::Good,
            _ => PasswordStrengthLevel::Strong,
        };

        PasswordStrength {
            level: strength,
            score,
            feedback,
        }
    }

    /// Check if password is in common passwords list (simplified)
    fn is_common_password(&self, password: &str) -> bool {
        let common_passwords = [
            "password", "123456", "123456789", "12345678", "12345",
            "1234567", "admin", "password123", "qwerty", "abc123",
            "letmein", "monkey", "111111", "password1", "1234567890",
        ];

        common_passwords.contains(&password.to_lowercase().as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PasswordStrengthLevel {
    Weak,
    Fair,
    Good,
    Strong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordStrength {
    pub level: PasswordStrengthLevel,
    pub score: u32,
    pub feedback: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bcrypt_creation() {
        let bcrypt = BCrypt::default();
        assert_eq!(bcrypt.get_cost(), DEFAULT_COST);
    }

    #[test]
    fn test_password_hashing() {
        let bcrypt = BCrypt::default();
        let password = "test_password_123";
        
        let hash = bcrypt.hash_password(password).unwrap();
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$2"));
    }

    #[test]
    fn test_password_verification() {
        let bcrypt = BCrypt::default();
        let password = "test_password_123";
        
        let hash = bcrypt.hash_password(password).unwrap();
        assert!(bcrypt.verify_password(password, &hash).unwrap());
        assert!(!bcrypt.verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_needs_rehash() {
        let bcrypt = BCrypt::with_cost(10).unwrap();
        let password = "test_password_123";
        
        let hash = bcrypt.hash_password(password).unwrap();
        assert!(!bcrypt.needs_rehash(&hash));
        
        let bcrypt_higher_cost = BCrypt::with_cost(12).unwrap();
        assert!(bcrypt_higher_cost.needs_rehash(&hash));
    }

    #[test]
    fn test_timing_safe_equals() {
        let bcrypt = BCrypt::default();
        
        assert!(bcrypt.timing_safe_equals("hello", "hello"));
        assert!(!bcrypt.timing_safe_equals("hello", "world"));
        assert!(!bcrypt.timing_safe_equals("hello", "hello_longer"));
    }

    #[test]
    fn test_password_strength() {
        let bcrypt = BCrypt::default();
        
        let weak = bcrypt.check_password_strength("123");
        assert!(matches!(weak.level, PasswordStrengthLevel::Weak));
        
        let strong = bcrypt.check_password_strength("MyStr0ng!P@ssw0rd");
        assert!(matches!(strong.level, PasswordStrengthLevel::Strong));
    }

    #[test]
    fn test_hash_validation() {
        let bcrypt = BCrypt::default();
        let password = "test_password_123";
        
        let hash = bcrypt.hash_password(password).unwrap();
        assert!(bcrypt.is_valid_hash(&hash));
        assert!(!bcrypt.is_valid_hash("invalid_hash"));
    }

    #[test]
    fn test_cost_validation() {
        assert!(BCrypt::with_cost(3).is_err()); // Too low
        assert!(BCrypt::with_cost(32).is_err()); // Too high
        assert!(BCrypt::with_cost(10).is_ok()); // Valid
    }
}