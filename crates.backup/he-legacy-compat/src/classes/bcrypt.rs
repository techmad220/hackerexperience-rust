// BCRYPT.CLASS.PHP PORT - Password hashing using BCrypt
// Original: Simple wrapper around PHP's crypt function with Blowfish
// Rust version uses modern bcrypt library with same security guarantees

use bcrypt::{hash, verify, DEFAULT_COST};

pub struct BCrypt {
    rounds: u32,
}

impl BCrypt {
    // Original PHP: __construct($rounds = 13)
    pub fn new(rounds: Option<u32>) -> Result<Self, BCryptError> {
        let rounds = rounds.unwrap_or(13);
        
        // Validate rounds (bcrypt supports 4-31, but 13 was the original default)
        if rounds < 4 || rounds > 31 {
            return Err(BCryptError::InvalidRounds(rounds));
        }
        
        Ok(Self { rounds })
    }
    
    // Create with default rounds (13, same as original PHP)
    pub fn default() -> Self {
        Self::new(Some(13)).unwrap() // 13 is always valid
    }
    
    // Original PHP: hash($input)
    pub fn hash(&self, input: &str) -> Result<String, BCryptError> {
        // Use the bcrypt crate which is secure and well-maintained
        hash(input, self.rounds)
            .map_err(|e| BCryptError::HashingFailed(e.to_string()))
    }
    
    // Original PHP: verify($input, $existingHash)
    pub fn verify(&self, input: &str, existing_hash: &str) -> Result<bool, BCryptError> {
        verify(input, existing_hash)
            .map_err(|e| BCryptError::VerificationFailed(e.to_string()))
    }
    
    // Utility method to check if a hash looks valid
    pub fn is_valid_hash(hash: &str) -> bool {
        // BCrypt hashes start with $2a$, $2b$, $2x$, or $2y$
        // and are 60 characters long
        hash.len() == 60 && (
            hash.starts_with("$2a$") ||
            hash.starts_with("$2b$") ||
            hash.starts_with("$2x$") ||
            hash.starts_with("$2y$")
        )
    }
    
    // Get the cost/rounds from an existing hash
    pub fn get_cost_from_hash(hash: &str) -> Option<u32> {
        if hash.len() >= 7 && hash.starts_with("$2") {
            // Extract cost from positions 4-5 (e.g., "$2a$13$...")
            if let Ok(cost) = hash[4..6].parse::<u32>() {
                return Some(cost);
            }
        }
        None
    }
    
    // Check if a hash needs rehashing (if cost has changed)
    pub fn needs_rehash(&self, hash: &str) -> bool {
        if let Some(hash_cost) = Self::get_cost_from_hash(hash) {
            hash_cost != self.rounds
        } else {
            true // Invalid hash format, needs rehashing
        }
    }
}

// Implement Default trait for convenient usage
impl Default for BCrypt {
    fn default() -> Self {
        Self::default()
    }
}

// Create a module-level function for quick hashing (similar to original usage pattern)
pub fn hash_password(password: &str) -> Result<String, BCryptError> {
    let bcrypt = BCrypt::default();
    bcrypt.hash(password)
}

// Create a module-level function for quick verification
pub fn verify_password(password: &str, hash: &str) -> Result<bool, BCryptError> {
    let bcrypt = BCrypt::default();
    bcrypt.verify(password, hash)
}

#[derive(Debug)]
pub enum BCryptError {
    InvalidRounds(u32),
    HashingFailed(String),
    VerificationFailed(String),
}

impl std::fmt::Display for BCryptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BCryptError::InvalidRounds(rounds) => {
                write!(f, "Invalid rounds: {}. Must be between 4 and 31.", rounds)
            },
            BCryptError::HashingFailed(msg) => {
                write!(f, "Password hashing failed: {}", msg)
            },
            BCryptError::VerificationFailed(msg) => {
                write!(f, "Password verification failed: {}", msg)
            },
        }
    }
}

impl std::error::Error for BCryptError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_and_verify() {
        let bcrypt = BCrypt::default();
        let password = "test_password_123";
        
        let hash = bcrypt.hash(password).unwrap();
        assert!(BCrypt::is_valid_hash(&hash));
        
        let is_valid = bcrypt.verify(password, &hash).unwrap();
        assert!(is_valid);
        
        let is_invalid = bcrypt.verify("wrong_password", &hash).unwrap();
        assert!(!is_invalid);
    }
    
    #[test]
    fn test_different_rounds() {
        let bcrypt_10 = BCrypt::new(Some(10)).unwrap();
        let bcrypt_12 = BCrypt::new(Some(12)).unwrap();
        
        let password = "test_password";
        let hash_10 = bcrypt_10.hash(password).unwrap();
        let hash_12 = bcrypt_12.hash(password).unwrap();
        
        // Both should verify correctly
        assert!(bcrypt_10.verify(password, &hash_10).unwrap());
        assert!(bcrypt_12.verify(password, &hash_12).unwrap());
        
        // Cross-verification should also work (bcrypt handles cost automatically)
        assert!(bcrypt_10.verify(password, &hash_12).unwrap());
        assert!(bcrypt_12.verify(password, &hash_10).unwrap());
    }
    
    #[test]
    fn test_cost_extraction() {
        let bcrypt = BCrypt::new(Some(13)).unwrap();
        let password = "test";
        let hash = bcrypt.hash(password).unwrap();
        
        let extracted_cost = BCrypt::get_cost_from_hash(&hash);
        assert_eq!(extracted_cost, Some(13));
    }
    
    #[test]
    fn test_needs_rehash() {
        let bcrypt_10 = BCrypt::new(Some(10)).unwrap();
        let bcrypt_12 = BCrypt::new(Some(12)).unwrap();
        
        let password = "test";
        let hash_10 = bcrypt_10.hash(password).unwrap();
        
        // Same cost should not need rehash
        assert!(!bcrypt_10.needs_rehash(&hash_10));
        
        // Different cost should need rehash
        assert!(bcrypt_12.needs_rehash(&hash_10));
    }
    
    #[test]
    fn test_module_functions() {
        let password = "module_test_password";
        
        let hash = hash_password(password).unwrap();
        assert!(BCrypt::is_valid_hash(&hash));
        
        let is_valid = verify_password(password, &hash).unwrap();
        assert!(is_valid);
    }
    
    #[test]
    fn test_invalid_rounds() {
        assert!(BCrypt::new(Some(3)).is_err()); // Too low
        assert!(BCrypt::new(Some(32)).is_err()); // Too high
        assert!(BCrypt::new(Some(10)).is_ok()); // Valid
    }
}