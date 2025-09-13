//! Doom virus updater job
//! 
//! This job monitors doom viruses and triggers round finish when needed.
//! Equivalent to the legacy doomUpdater.php cron job.

use crate::error::{CronError, CronResult};
use crate::jobs::finish_round;
use sqlx::{MySqlPool, Row};
use std::sync::Arc;
use tracing::{info, warn, error};
use tokio::time::{sleep, Duration};

/// Doom updater job implementation
pub struct DoomUpdaterJob;

#[derive(Debug)]
struct DoomVirus {
    doom_id: i32,
    creator_id: i32,
    clan_id: Option<i32>,
    time_left: i64,
}

impl DoomUpdaterJob {
    /// Execute the doom updater job
    pub async fn execute(db_pool: Arc<MySqlPool>) -> CronResult<()> {
        info!("Starting doom virus check");
        
        // Check if any doom viruses exist
        if !Self::doom_viruses_exist(&db_pool).await? {
            info!("No doom viruses found");
            return Ok(());
        }
        
        // Start recursive checking
        Self::recursive_doom_check(&db_pool).await?;
        
        info!("Doom virus check completed");
        Ok(())
    }
    
    /// Check if any doom viruses exist in the system
    async fn doom_viruses_exist(db_pool: &MySqlPool) -> CronResult<bool> {
        let count = sqlx::query(
            "SELECT COUNT(*) as count FROM virus_doom"
        )
        .fetch_one(db_pool)
        .await?;
        
        Ok(count.try_get::<i64, _>("count")? > 0)
    }
    
    /// Recursively check doom viruses and handle timeouts
    async fn recursive_doom_check(db_pool: &MySqlPool) -> CronResult<()> {
        loop {
            let active_dooms = Self::get_active_doom_viruses(&db_pool).await?;
            
            if active_dooms.is_empty() {
                info!("No active doom viruses found");
                break;
            }
            
            // Check each doom virus
            for doom in &active_dooms {
                if doom.time_left < 0 {
                    // Doom virus has expired - finish the round
                    info!("Doom virus {} has expired, finishing round", doom.doom_id);
                    
                    Self::mark_doom_finished(&db_pool, doom.doom_id).await?;
                    
                    // Call finish round job
                    finish_round::FinishRoundJob::execute(Arc::clone(&db_pool)).await?;
                    
                    // TODO: Disconnect all users (would require WebSocket/session management)
                    warn!("User disconnection not implemented - requires session management");
                    
                    return Ok(());
                } else if doom.time_left < 60 {
                    // Less than 60 seconds left - wait and check again
                    let wait_time = (doom.time_left + 1).max(1) as u64;
                    info!("Doom virus {} expires in {} seconds, waiting...", doom.doom_id, doom.time_left);
                    
                    sleep(Duration::from_secs(wait_time)).await;
                    continue;
                }
            }
            
            // If we reach here, no doom viruses are expiring soon
            break;
        }
        
        Ok(())
    }
    
    /// Get all active doom viruses with time remaining
    async fn get_active_doom_viruses(db_pool: &MySqlPool) -> CronResult<Vec<DoomVirus>> {
        let rows = sqlx::query(
            "SELECT doomID, creatorID, clanID, TIMESTAMPDIFF(SECOND, NOW(), doomDate) AS timeLeft 
             FROM virus_doom 
             WHERE status = 1 
             ORDER BY releaseDate ASC"
        )
        .fetch_all(db_pool)
        .await?;
        
        let mut dooms = Vec::new();
        for row in rows {
            dooms.push(DoomVirus {
                doom_id: row.try_get("doomID")?,
                creator_id: row.try_get("creatorID")?,
                clan_id: row.try_get("clanID").ok(),
                time_left: row.try_get("timeLeft")?,
            });
        }
        
        Ok(dooms)
    }
    
    /// Mark a doom virus as finished
    async fn mark_doom_finished(db_pool: &MySqlPool, doom_id: i32) -> CronResult<()> {
        sqlx::query(
            "UPDATE virus_doom SET status = 3 WHERE doomID = ?"
        )
        .bind(doom_id)
        .execute(db_pool)
        .await?;
        
        info!("Marked doom virus {} as finished", doom_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_doom_virus_structure() {
        let doom = DoomVirus {
            doom_id: 1,
            creator_id: 123,
            clan_id: Some(456),
            time_left: 3600,
        };
        
        assert_eq!(doom.doom_id, 1);
        assert_eq!(doom.creator_id, 123);
        assert_eq!(doom.clan_id, Some(456));
        assert_eq!(doom.time_left, 3600);
    }
}