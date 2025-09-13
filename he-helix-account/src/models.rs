use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

use he_core::id::{AccountId, EntityId};
use he_entity::{Entity, EntitySpecialization, EntityType};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Validate)]
pub struct Account {
    pub account_id: AccountId,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 3, max = 16))]
    pub username: String,
    pub display_name: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub confirmed: bool,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Account {
    /// Checks if the provided password matches the account's password
    /// This function is safe against timing attacks
    pub fn check_password(&self, password: &str) -> bool {
        bcrypt::verify(password, &self.password).unwrap_or(false)
    }

    /// Converts an EntityId to AccountId
    pub fn cast_from_entity(entity_id: EntityId) -> AccountId {
        entity_id
    }
}

impl EntitySpecialization for Account {
    fn get_entity_id(&self) -> EntityId {
        self.account_id
    }

    fn get_entity_type(&self) -> EntityType {
        EntityType::Account
    }
}

#[derive(Debug, Clone, Validate)]
pub struct CreateAccountParams {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 3, max = 16))]
    pub username: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Clone, Validate)]
pub struct UpdateAccountParams {
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 8))]
    pub password: Option<String>,
    pub confirmed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email_or_username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub account: PublicAccount,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

/// Public representation of an account (without sensitive information)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicAccount {
    pub account_id: AccountId,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub confirmed: bool,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Account> for PublicAccount {
    fn from(account: Account) -> Self {
        Self {
            account_id: account.account_id,
            email: account.email,
            username: account.username,
            display_name: account.display_name,
            confirmed: account.confirmed,
            inserted_at: account.inserted_at,
            updated_at: account.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccountToken {
    pub token_id: uuid::Uuid,
    pub account_id: AccountId,
    pub token_hash: String,
    pub token_type: TokenType,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub inserted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "token_type", rename_all = "lowercase")]
pub enum TokenType {
    Auth,
    PasswordReset,
    EmailConfirmation,
}

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("Account not found")]
    NotFound,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Account already exists")]
    AlreadyExists,
    #[error("Token expired")]
    TokenExpired,
    #[error("Token already used")]
    TokenUsed,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),
    #[error("Password hashing error: {0}")]
    PasswordHash(#[from] bcrypt::BcryptError),
}