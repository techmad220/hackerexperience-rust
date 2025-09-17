//! Field-level encryption for sensitive data at rest

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key
};
use chacha20poly1305::ChaCha20Poly1305;
use argon2::{Argon2, PasswordHasher, PasswordVerifier, PasswordHash};
use argon2::password_hash::{rand_core::RngCore, SaltString};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Result, anyhow};

/// Encrypted field wrapper for database storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedField {
    pub algorithm: String,        // "AES256-GCM" or "ChaCha20-Poly1305"
    pub ciphertext: String,       // Base64 encoded
    pub nonce: String,            // Base64 encoded
    pub key_id: String,           // Key rotation tracking
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Key management for field encryption
pub struct FieldEncryption {
    keys: HashMap<String, Vec<u8>>,
    active_key_id: String,
    algorithm: EncryptionAlgorithm,
}

#[derive(Debug, Clone)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

impl FieldEncryption {
    /// Create new encryption handler with master key
    pub fn new(master_key: &[u8], algorithm: EncryptionAlgorithm) -> Result<Self> {
        // Derive encryption key from master key using Argon2
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let mut derived_key = vec![0u8; 32];
        argon2.hash_password_into(master_key, salt.as_bytes(), &mut derived_key)
            .map_err(|e| anyhow!("Key derivation failed: {}", e))?;

        let mut keys = HashMap::new();
        let key_id = format!("key_v1_{}", chrono::Utc::now().timestamp());
        keys.insert(key_id.clone(), derived_key);

        Ok(Self {
            keys,
            active_key_id: key_id,
            algorithm,
        })
    }

    /// Encrypt a string value
    pub fn encrypt(&self, plaintext: &str) -> Result<EncryptedField> {
        let key = self.keys.get(&self.active_key_id)
            .ok_or_else(|| anyhow!("Active key not found"))?;

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);

        let ciphertext = match &self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
                let nonce = Nonce::from_slice(&nonce_bytes);
                cipher.encrypt(nonce, plaintext.as_bytes())
                    .map_err(|e| anyhow!("Encryption failed: {}", e))?
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                let cipher = ChaCha20Poly1305::new(key.as_slice().into());
                let nonce = chacha20poly1305::Nonce::from_slice(&nonce_bytes);
                cipher.encrypt(nonce, plaintext.as_bytes())
                    .map_err(|e| anyhow!("Encryption failed: {}", e))?
            }
        };

        Ok(EncryptedField {
            algorithm: match &self.algorithm {
                EncryptionAlgorithm::Aes256Gcm => "AES256-GCM".to_string(),
                EncryptionAlgorithm::ChaCha20Poly1305 => "ChaCha20-Poly1305".to_string(),
            },
            ciphertext: general_purpose::STANDARD.encode(&ciphertext),
            nonce: general_purpose::STANDARD.encode(&nonce_bytes),
            key_id: self.active_key_id.clone(),
            created_at: chrono::Utc::now(),
        })
    }

    /// Decrypt an encrypted field
    pub fn decrypt(&self, field: &EncryptedField) -> Result<String> {
        let key = self.keys.get(&field.key_id)
            .ok_or_else(|| anyhow!("Key {} not found", field.key_id))?;

        let ciphertext = general_purpose::STANDARD.decode(&field.ciphertext)
            .map_err(|e| anyhow!("Invalid ciphertext: {}", e))?;

        let nonce_bytes = general_purpose::STANDARD.decode(&field.nonce)
            .map_err(|e| anyhow!("Invalid nonce: {}", e))?;

        let plaintext = match field.algorithm.as_str() {
            "AES256-GCM" => {
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
                let nonce = Nonce::from_slice(&nonce_bytes);
                cipher.decrypt(nonce, ciphertext.as_slice())
                    .map_err(|e| anyhow!("Decryption failed: {}", e))?
            }
            "ChaCha20-Poly1305" => {
                let cipher = ChaCha20Poly1305::new(key.as_slice().into());
                let nonce = chacha20poly1305::Nonce::from_slice(&nonce_bytes);
                cipher.decrypt(nonce, ciphertext.as_slice())
                    .map_err(|e| anyhow!("Decryption failed: {}", e))?
            }
            _ => return Err(anyhow!("Unknown algorithm: {}", field.algorithm))
        };

        String::from_utf8(plaintext)
            .map_err(|e| anyhow!("Invalid UTF-8: {}", e))
    }

    /// Rotate to a new encryption key
    pub fn rotate_key(&mut self, new_master_key: &[u8]) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let mut derived_key = vec![0u8; 32];
        argon2.hash_password_into(new_master_key, salt.as_bytes(), &mut derived_key)
            .map_err(|e| anyhow!("Key derivation failed: {}", e))?;

        let new_key_id = format!("key_v1_{}", chrono::Utc::now().timestamp());
        self.keys.insert(new_key_id.clone(), derived_key);
        self.active_key_id = new_key_id.clone();

        Ok(new_key_id)
    }

    /// Re-encrypt field with current key (for key rotation)
    pub fn reencrypt(&self, field: &EncryptedField) -> Result<EncryptedField> {
        let plaintext = self.decrypt(field)?;
        self.encrypt(&plaintext)
    }
}

/// Helper functions for common encryption operations

/// Encrypt sensitive user data fields
pub fn encrypt_field(value: &str, key: &[u8]) -> Result<String> {
    let encryptor = FieldEncryption::new(key, EncryptionAlgorithm::Aes256Gcm)?;
    let encrypted = encryptor.encrypt(value)?;
    Ok(serde_json::to_string(&encrypted)?)
}

/// Decrypt sensitive user data fields
pub fn decrypt_field(encrypted_json: &str, key: &[u8]) -> Result<String> {
    let field: EncryptedField = serde_json::from_str(encrypted_json)?;
    let encryptor = FieldEncryption::new(key, EncryptionAlgorithm::Aes256Gcm)?;
    encryptor.decrypt(&field)
}

/// Transparent encryption layer for database operations
pub struct TransparentEncryption {
    encryptor: FieldEncryption,
    encrypted_columns: HashMap<String, Vec<String>>, // table -> columns
}

impl TransparentEncryption {
    pub fn new(master_key: &[u8]) -> Result<Self> {
        let encryptor = FieldEncryption::new(master_key, EncryptionAlgorithm::Aes256Gcm)?;

        let mut encrypted_columns = HashMap::new();

        // Define which columns should be encrypted
        encrypted_columns.insert("users".to_string(), vec![
            "email".to_string(),
            "real_name".to_string(),
            "phone".to_string(),
        ]);

        encrypted_columns.insert("bank_accounts".to_string(), vec![
            "account_number".to_string(),
            "routing_number".to_string(),
        ]);

        encrypted_columns.insert("messages".to_string(), vec![
            "content".to_string(),
        ]);

        encrypted_columns.insert("logs".to_string(), vec![
            "ip_address".to_string(),
            "user_agent".to_string(),
        ]);

        Ok(Self {
            encryptor,
            encrypted_columns,
        })
    }

    /// Check if a column should be encrypted
    pub fn should_encrypt(&self, table: &str, column: &str) -> bool {
        self.encrypted_columns
            .get(table)
            .map(|cols| cols.contains(&column.to_string()))
            .unwrap_or(false)
    }

    /// Encrypt value if column requires encryption
    pub fn maybe_encrypt(&self, table: &str, column: &str, value: &str) -> Result<String> {
        if self.should_encrypt(table, column) {
            let encrypted = self.encryptor.encrypt(value)?;
            Ok(serde_json::to_string(&encrypted)?)
        } else {
            Ok(value.to_string())
        }
    }

    /// Decrypt value if column is encrypted
    pub fn maybe_decrypt(&self, table: &str, column: &str, value: &str) -> Result<String> {
        if self.should_encrypt(table, column) {
            let field: EncryptedField = serde_json::from_str(value)?;
            self.encryptor.decrypt(&field)
        } else {
            Ok(value.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_encryption() {
        let master_key = b"test_master_key_32_bytes_long!!!";
        let encryptor = FieldEncryption::new(master_key, EncryptionAlgorithm::Aes256Gcm).unwrap();

        let plaintext = "sensitive data";
        let encrypted = encryptor.encrypt(plaintext).unwrap();

        assert_ne!(encrypted.ciphertext, plaintext);
        assert!(!encrypted.nonce.is_empty());

        let decrypted = encryptor.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_transparent_encryption() {
        let master_key = b"test_master_key_32_bytes_long!!!";
        let transparent = TransparentEncryption::new(master_key).unwrap();

        // Should encrypt email field
        assert!(transparent.should_encrypt("users", "email"));
        let encrypted = transparent.maybe_encrypt("users", "email", "user@example.com").unwrap();
        assert!(encrypted.contains("ciphertext"));

        // Should not encrypt username field
        assert!(!transparent.should_encrypt("users", "username"));
        let not_encrypted = transparent.maybe_encrypt("users", "username", "john_doe").unwrap();
        assert_eq!(not_encrypted, "john_doe");
    }
}