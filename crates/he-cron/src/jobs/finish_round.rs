//! Finish round job
//! 
//! This job handles the process of finishing a game round, including
//! statistics updates and cleanup. Equivalent to the legacy finishRound.php cron job.

use crate::error::{CronError, CronResult};
use crate::utils::{get_software_extension, dot_version, execute_command};
use sqlx::{MySqlPool, Row};
use std::sync::Arc;
use tracing::{info, warn, error};

/// Finish round job implementation
pub struct FinishRoundJob;

impl FinishRoundJob {
    /// Execute the finish round job
    pub async fn execute(db_pool: Arc<MySqlPool>) -> CronResult<()> {
        info!("Starting round finish process");
        
        // Note: The original PHP script has an exit() at the top, indicating this is a sensitive operation
        // In production, this should have proper access controls
        
        // Run external stats and ranking update script
        // Note: This was in the original PHP but commented with concerns about external execution
        // Self::update_stats_and_ranking().await?;
        
        // Perform round cleanup operations
        Self::cleanup_round_data(&db_pool).await?;
        
        // Update round status
        Self::finish_current_round(&db_pool).await?;
        
        info!("Round finish process completed");
        Ok(())
    }
    
    /// Update stats and ranking (external script)
    /// Note: Commented out as the original PHP also had this with security concerns
    #[allow(dead_code)]
    async fn update_stats_and_ranking() -> CronResult<()> {
        info!("Running stats and ranking update script");
        
        // This would execute the external shell script
        // execute_command("/bin/sh", &["../cron2/updateStatsAndRanking.sh"]).await?;
        
        warn!("Stats and ranking update script execution is disabled for security");
        Ok(())
    }
    
    /// Cleanup round data and prepare for next round
    async fn cleanup_round_data(db_pool: &MySqlPool) -> CronResult<()> {
        info!("Performing round cleanup");
        
        // Clear temporary round data
        Self::clear_temporary_data(db_pool).await?;
        
        // Archive important round data
        Self::archive_round_data(db_pool).await?;
        
        // Reset dynamic game state
        Self::reset_game_state(db_pool).await?;
        
        info!("Round cleanup completed");
        Ok(())
    }
    
    /// Clear temporary data that doesn't need to persist between rounds
    async fn clear_temporary_data(db_pool: &MySqlPool) -> CronResult<()> {
        info!("Clearing temporary round data");
        
        // Clear online users
        sqlx::query("DELETE FROM users_online").execute(db_pool).await?;
        
        // Clear active processes
        sqlx::query("DELETE FROM processes").execute(db_pool).await?;
        
        // Clear running software
        sqlx::query("DELETE FROM software_running").execute(db_pool).await?;
        
        // Clear temporary connections
        sqlx::query("DELETE FROM internet_connections WHERE temporary = 1").execute(db_pool).await?;
        
        // Clear active missions
        sqlx::query("DELETE FROM missions WHERE status = 1").execute(db_pool).await?;
        
        info!("Temporary data cleared");
        Ok(())
    }
    
    /// Archive important round data to history tables
    async fn archive_round_data(db_pool: &MySqlPool) -> CronResult<()> {
        info!("Archiving round data");
        
        // Get current round ID
        let current_round = sqlx::query(
            "SELECT id FROM round ORDER BY id DESC LIMIT 1"
        )
        .fetch_optional(db_pool)
        .await?;
        
        if let Some(round_row) = current_round {
            let round_id: i32 = round_row.try_get("id")?;
            
            // Archive user statistics
            Self::archive_user_stats(db_pool, round_id).await?;
            
            // Archive clan statistics  
            Self::archive_clan_stats(db_pool, round_id).await?;
            
            // Archive final server statistics
            Self::archive_server_stats(db_pool, round_id).await?;
            
            info!("Round {} data archived", round_id);
        } else {
            warn!("No current round found to archive");
        }
        
        Ok(())
    }
    
    /// Archive user statistics
    async fn archive_user_stats(db_pool: &MySqlPool, round_id: i32) -> CronResult<()> {
        // Create user stats history entries
        sqlx::query(
            "INSERT INTO users_stats_history 
             (user_id, round_id, money_earned, money_spent, hack_count, ddos_count, 
              spam_sent, warez_sent, bitcoin_sent, time_playing, created_at)
             SELECT uid, ?, moneyEarned, moneySpent, hackCount, ddosCount, 
                    spamSent, warezSent, bitcoinSent, timePlaying, NOW()
             FROM users_stats"
        )
        .bind(round_id)
        .execute(db_pool)
        .await?;
        
        info!("User statistics archived for round {}", round_id);
        Ok(())
    }
    
    /// Archive clan statistics
    async fn archive_clan_stats(db_pool: &MySqlPool, round_id: i32) -> CronResult<()> {
        // Create clan stats history entries
        sqlx::query(
            "INSERT INTO clan_stats_history 
             (clan_id, round_id, wars_won, wars_lost, members_count, power, created_at)
             SELECT c.clanID, ?, cs.won, cs.lost, 
                    (SELECT COUNT(*) FROM clan_users cu WHERE cu.clanID = c.clanID), 
                    c.power, NOW()
             FROM clan c
             LEFT JOIN clan_stats cs ON c.clanID = cs.cid"
        )
        .bind(round_id)
        .execute(db_pool)
        .await?;
        
        info!("Clan statistics archived for round {}", round_id);
        Ok(())
    }
    
    /// Archive server statistics
    async fn archive_server_stats(db_pool: &MySqlPool, round_id: i32) -> CronResult<()> {
        // Archive final round statistics
        sqlx::query(
            "INSERT INTO round_stats_history 
             SELECT *, ? as round_id, NOW() as archived_at
             FROM round_stats 
             ORDER BY id DESC LIMIT 1"
        )
        .bind(round_id)
        .execute(db_pool)
        .await?;
        
        info!("Server statistics archived for round {}", round_id);
        Ok(())
    }
    
    /// Reset game state for new round
    async fn reset_game_state(db_pool: &MySqlPool) -> CronResult<()> {
        info!("Resetting game state for new round");
        
        // Reset user statistics
        sqlx::query(
            "UPDATE users_stats SET 
             moneyEarned = 0, moneySpent = 0, hackCount = 0, ddosCount = 0,
             spamSent = 0, warezSent = 0, bitcoinSent = 0, timePlaying = 0"
        ).execute(db_pool).await?;
        
        // Reset clan war statistics
        sqlx::query("UPDATE clan_stats SET won = 0, lost = 0").execute(db_pool).await?;
        
        // Clear clan wars
        sqlx::query("DELETE FROM clan_war").execute(db_pool).await?;
        
        // Clear DEFCON data
        sqlx::query("DELETE FROM clan_defcon").execute(db_pool).await?;
        
        // Reset clan power to base values
        sqlx::query("UPDATE clan SET power = 100").execute(db_pool).await?;
        
        // Clear virus infections
        sqlx::query("DELETE FROM virus WHERE temporary = 1").execute(db_pool).await?;
        
        info!("Game state reset completed");
        Ok(())
    }
    
    /// Finish the current round and prepare for the next
    async fn finish_current_round(db_pool: &MySqlPool) -> CronResult<()> {
        info!("Finishing current round");
        
        // Mark current round as finished
        sqlx::query(
            "UPDATE round SET status = 0, endDate = NOW() 
             ORDER BY id DESC LIMIT 1"
        ).execute(db_pool).await?;
        
        // Create new round
        sqlx::query(
            "INSERT INTO round (id, startDate, endDate, status) 
             VALUES (NULL, NOW(), NULL, 1)"
        ).execute(db_pool).await?;
        
        let new_round_id = sqlx::query("SELECT LAST_INSERT_ID() as id")
            .fetch_one(db_pool)
            .await?
            .try_get::<u64, _>("id")? as i32;
        
        // Initialize new round statistics
        sqlx::query(
            "INSERT INTO round_stats 
             (round_id, totalUsers, activeUsers, onlineUsers, created_at)
             VALUES (?, 0, 0, 0, NOW())"
        )
        .bind(new_round_id)
        .execute(db_pool)
        .await?;
        
        info!("New round {} started", new_round_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_finish_round_job_creation() {
        let _job = FinishRoundJob;
        // Basic test - actual functionality requires database
    }
}