//! Update premium status job
//! 
//! This job checks for expired premium subscriptions and updates user status.
//! Equivalent to the legacy updatePremium.php cron job.

use crate::error::{CronError, CronResult};
use sqlx::{MySqlPool, Row};
use std::sync::Arc;
use tracing::{info, warn};

/// Update premium job implementation
pub struct UpdatePremiumJob;

impl UpdatePremiumJob {
    /// Execute the update premium job
    pub async fn execute(db_pool: Arc<MySqlPool>) -> CronResult<()> {
        info!("Starting premium status update");
        
        // Find expired premium users
        let expired_users = Self::get_expired_premium_users(&db_pool).await?;
        
        if expired_users.is_empty() {
            info!("No expired premium subscriptions found");
            return Ok(());
        }
        
        info!("Found {} expired premium subscriptions", expired_users.len());
        
        // Update each expired user
        for user_id in expired_users {
            Self::revoke_premium_status(&db_pool, user_id).await?;
        }
        
        info!("Premium status update completed");
        Ok(())
    }
    
    /// Get list of users with expired premium subscriptions
    async fn get_expired_premium_users(db_pool: &MySqlPool) -> CronResult<Vec<i32>> {
        let rows = sqlx::query(
            "SELECT id FROM users_premium WHERE TIMESTAMPDIFF(SECOND, NOW(), premiumUntil) < 0"
        )
        .fetch_all(db_pool)
        .await?;
        
        let mut expired_users = Vec::new();
        for row in rows {
            expired_users.push(row.try_get::<i32, _>("id")?);
        }
        
        Ok(expired_users)
    }
    
    /// Revoke premium status for a user
    async fn revoke_premium_status(db_pool: &MySqlPool, user_id: i32) -> CronResult<()> {
        info!("Revoking premium status for user {}", user_id);
        
        // Update users table
        sqlx::query("UPDATE users SET premium = 0 WHERE id = ?")
            .bind(user_id)
            .execute(db_pool)
            .await?;
        
        // Update profile table
        sqlx::query("UPDATE profile SET premium = 0 WHERE id = ?")
            .bind(user_id)
            .execute(db_pool)
            .await?;
        
        // Remove from premium table
        sqlx::query("DELETE FROM users_premium WHERE id = ?")
            .bind(user_id)
            .execute(db_pool)
            .await?;
        
        info!("Premium status revoked for user {}", user_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Tests would require database setup, so keeping them simple
    #[test]
    fn test_update_premium_job_creation() {
        // Just test that we can create the job
        let _job = UpdatePremiumJob;
    }
}