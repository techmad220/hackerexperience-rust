//! Refresh token storage and management

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Refresh token database model
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: i64,
    pub token_hash: String,
    pub jti: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

pub struct RefreshTokenQueries;

impl RefreshTokenQueries {
    /// Store a new refresh token
    pub async fn create(
        pool: &PgPool,
        user_id: i64,
        token_hash: &str,
        jti: &str,
        expires_at: DateTime<Utc>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<RefreshToken> {
        let token = sqlx::query_as!(
            RefreshToken,
            r#"
            INSERT INTO refresh_tokens (
                id, user_id, token_hash, jti, expires_at,
                created_at, revoked, ip_address, user_agent
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            Uuid::new_v4(),
            user_id,
            token_hash,
            jti,
            expires_at,
            Utc::now(),
            false,
            ip_address,
            user_agent
        )
        .fetch_one(pool)
        .await?;

        Ok(token)
    }

    /// Find a refresh token by JTI
    pub async fn find_by_jti(pool: &PgPool, jti: &str) -> Result<Option<RefreshToken>> {
        let token = sqlx::query_as!(
            RefreshToken,
            r#"
            SELECT * FROM refresh_tokens
            WHERE jti = $1 AND revoked = false AND expires_at > NOW()
            "#,
            jti
        )
        .fetch_optional(pool)
        .await?;

        Ok(token)
    }

    /// Revoke a refresh token
    pub async fn revoke(pool: &PgPool, jti: &str) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE refresh_tokens
            SET revoked = true, revoked_at = NOW()
            WHERE jti = $1 AND revoked = false
            "#,
            jti
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Revoke all tokens for a user
    pub async fn revoke_all_for_user(pool: &PgPool, user_id: i64) -> Result<u64> {
        let result = sqlx::query!(
            r#"
            UPDATE refresh_tokens
            SET revoked = true, revoked_at = NOW()
            WHERE user_id = $1 AND revoked = false
            "#,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Clean up expired tokens
    pub async fn cleanup_expired(pool: &PgPool) -> Result<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM refresh_tokens
            WHERE expires_at < NOW() OR (revoked = true AND revoked_at < NOW() - INTERVAL '7 days')
            "#
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Get active token count for a user
    pub async fn get_active_count(pool: &PgPool, user_id: i64) -> Result<i64> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM refresh_tokens
            WHERE user_id = $1 AND revoked = false AND expires_at > NOW()
            "#,
            user_id
        )
        .fetch_one(pool)
        .await?;

        Ok(count)
    }
}