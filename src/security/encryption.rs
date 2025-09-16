use actix_web::Error;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key
};
use base64::{engine::general_purpose::STANDARD, Engine};

/// Data encryption handler
pub struct DataEncryption {
    cipher: Aes256Gcm,
}

impl DataEncryption {
    pub fn new() -> Self {
        // In production, load key from secure storage
        let key_bytes = std::env::var("ENCRYPTION_KEY")
            .unwrap_or_else(|_| "0123456789abcdef0123456789abcdef".to_string())
            .as_bytes()[..32]
            .try_into()
            .expect("Invalid key length");

        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        Self { cipher }
    }

    /// Encrypt data
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = self.cipher
            .encrypt(&nonce, data)
            .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Encryption failed: {}", e)))?;

        // Prepend nonce to ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt data
    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, Error> {
        if encrypted_data.len() < 12 {
            return Err(actix_web::error::ErrorBadRequest("Invalid encrypted data"));
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Decryption failed: {}", e)))?;

        Ok(plaintext)
    }

    /// Encrypt string to base64
    pub fn encrypt_string(&self, data: &str) -> Result<String, Error> {
        let encrypted = self.encrypt(data.as_bytes())?;
        Ok(STANDARD.encode(encrypted))
    }

    /// Decrypt base64 string
    pub fn decrypt_string(&self, encrypted: &str) -> Result<String, Error> {
        let decoded = STANDARD.decode(encrypted)
            .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid base64: {}", e)))?;

        let decrypted = self.decrypt(&decoded)?;

        String::from_utf8(decrypted)
            .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Invalid UTF-8: {}", e)))
    }
}