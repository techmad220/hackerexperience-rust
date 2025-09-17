//! Production-ready authentication with Argon2id and JWT

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// User ID
    pub sub: i64,
    /// Expiration timestamp
    pub exp: usize,
    /// Issued at timestamp
    pub iat: usize,
}

/// Hash a password using Argon2id
pub fn hash_password(plain: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(plain.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

    Ok(password_hash.to_string())
}

/// Verify a password against an Argon2id hash
pub fn verify_password(hashed: &str, plain: &str) -> anyhow::Result<()> {
    let parsed = PasswordHash::new(hashed)
        .map_err(|e| anyhow::anyhow!("Invalid password hash format: {}", e))?;

    Argon2::default()
        .verify_password(plain.as_bytes(), &parsed)
        .map_err(|_| anyhow::anyhow!("Invalid password"))?;

    Ok(())
}

/// Issue a JWT token for a user
pub fn issue_jwt(user_id: i64, secret: &str, ttl_secs: i64) -> anyhow::Result<String> {
    let now = OffsetDateTime::now_utc();
    let exp = (now + Duration::seconds(ttl_secs)).unix_timestamp() as usize;
    let iat = now.unix_timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        exp,
        iat,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes())
    ).map_err(|e| anyhow::anyhow!("Failed to encode JWT: {}", e))?;

    Ok(token)
}

/// Verify and decode a JWT token
pub fn verify_jwt(token: &str, secret: &str) -> anyhow::Result<Claims> {
    let key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::default();

    let token_data = decode::<Claims>(token, &key, &validation)
        .map_err(|e| anyhow::anyhow!("Invalid token: {}", e))?;

    Ok(token_data.claims)
}

/// Extract user from request (for use in handlers)
pub struct AuthedUser {
    pub id: i64,
}

impl AuthedUser {
    /// Extract from JWT in Authorization header
    pub fn from_header(auth_header: Option<&str>, jwt_secret: &str) -> anyhow::Result<Self> {
        let header = auth_header
            .ok_or_else(|| anyhow::anyhow!("Missing Authorization header"))?;

        if !header.starts_with("Bearer ") {
            anyhow::bail!("Invalid Authorization format (expected 'Bearer TOKEN')");
        }

        let token = &header[7..];
        let claims = verify_jwt(token, jwt_secret)?;

        Ok(Self { id: claims.sub })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "super_secret_password_123!";

        // Hash password
        let hash = hash_password(password).unwrap();
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2"));

        // Verify correct password
        assert!(verify_password(&hash, password).is_ok());

        // Reject wrong password
        assert!(verify_password(&hash, "wrong_password").is_err());
    }

    #[test]
    fn test_jwt_lifecycle() {
        let secret = "test_secret_key_123";
        let user_id = 42;

        // Issue token
        let token = issue_jwt(user_id, secret, 3600).unwrap();
        assert!(!token.is_empty());

        // Verify token
        let claims = verify_jwt(&token, secret).unwrap();
        assert_eq!(claims.sub, user_id);

        // Wrong secret should fail
        assert!(verify_jwt(&token, "wrong_secret").is_err());

        // Expired token should fail
        let expired = issue_jwt(user_id, secret, -1).unwrap();
        assert!(verify_jwt(&expired, secret).is_err());
    }

    #[test]
    fn test_auth_header_extraction() {
        let secret = "test_secret";
        let token = issue_jwt(123, secret, 3600).unwrap();

        // Valid Bearer token
        let header = format!("Bearer {}", token);
        let user = AuthedUser::from_header(Some(&header), secret).unwrap();
        assert_eq!(user.id, 123);

        // Missing header
        assert!(AuthedUser::from_header(None, secret).is_err());

        // Wrong format
        assert!(AuthedUser::from_header(Some("Basic xyz"), secret).is_err());

        // Invalid token
        assert!(AuthedUser::from_header(Some("Bearer invalid"), secret).is_err());
    }
}