//! Restore NPC software job
//! 
//! This job restores NPC software according to the software_original table.
//! Equivalent to the legacy restoreSoftware.php cron job.

use crate::error::{CronError, CronResult};
use sqlx::{MySqlPool, Row};
use std::sync::Arc;
use tracing::{info, warn, error};

/// Restore software job implementation
pub struct RestoreSoftwareJob;

#[derive(Debug, sqlx::FromRow)]
struct OriginalSoftware {
    id: i32,
    npc_id: i32,
    soft_name: String,
    soft_version: i32,
    soft_ram: i32,
    soft_size: i32,
    soft_type: i32,
}

impl RestoreSoftwareJob {
    /// Execute the restore software job
    pub async fn execute(db_pool: Arc<MySqlPool>) -> CronResult<()> {
        info!("Starting NPC software restoration");
        
        let mut restored_count = 0;
        let mut skipped_count = 0;
        
        // Get all original software entries
        let original_software = Self::get_original_software(&db_pool).await?;
        
        info!("Found {} original software entries to process", original_software.len());
        
        for software in original_software {
            match Self::restore_software_if_missing(&db_pool, &software).await {
                Ok(restored) => {
                    if restored {
                        restored_count += 1;
                        info!("Restored software: {} v{} for NPC {}", 
                              software.soft_name, software.soft_version, software.npc_id);
                    } else {
                        skipped_count += 1;
                    }
                }
                Err(e) => {
                    error!("Failed to restore software {} for NPC {}: {}", 
                           software.soft_name, software.npc_id, e);
                    return Err(e);
                }
            }
        }
        
        info!("NPC software restoration completed: {} restored, {} skipped", 
              restored_count, skipped_count);
        Ok(())
    }
    
    /// Get all original software entries from the database
    async fn get_original_software(db_pool: &MySqlPool) -> CronResult<Vec<OriginalSoftware>> {
        let rows = sqlx::query_as::<_, OriginalSoftware>(
            "SELECT id, npcID as npc_id, softName as soft_name, softVersion as soft_version, 
                    softRam as soft_ram, softSize as soft_size, softType as soft_type 
             FROM software_original"
        )
        .fetch_all(db_pool)
        .await?;
        
        Ok(rows)
    }
    
    /// Check if software exists for the NPC, and restore it if missing
    async fn restore_software_if_missing(
        db_pool: &MySqlPool, 
        original: &OriginalSoftware
    ) -> CronResult<bool> {
        // Check if software already exists
        let existing_count = sqlx::query(
            "SELECT id FROM software 
             WHERE userID = ? AND isNPC = 1 AND softName = ? AND softVersion = ?"
        )
        .bind(original.npc_id)
        .bind(&original.soft_name)
        .bind(original.soft_version)
        .fetch_all(db_pool)
        .await?
        .len();
        
        if existing_count > 0 {
            // Software already exists, skip restoration
            return Ok(false);
        }
        
        // Software doesn't exist, restore it
        Self::insert_software(db_pool, original).await?;
        Ok(true)
    }
    
    /// Insert the missing software into the database
    async fn insert_software(db_pool: &MySqlPool, original: &OriginalSoftware) -> CronResult<()> {
        sqlx::query(
            "INSERT INTO software 
             (id, softHidden, softHiddenWith, softLastEdit, softName, softSize,
              softType, softVersion, userID, isNPC, softRam) 
             VALUES (NULL, 0, 0, NOW(), ?, ?, ?, ?, ?, 1, ?)"
        )
        .bind(&original.soft_name)
        .bind(original.soft_size)
        .bind(original.soft_type)
        .bind(original.soft_version)
        .bind(original.npc_id)
        .bind(original.soft_ram)
        .execute(db_pool)
        .await?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_original_software_structure() {
        let software = OriginalSoftware {
            id: 1,
            npc_id: 123,
            soft_name: "TestSoft".to_string(),
            soft_version: 15,
            soft_ram: 64,
            soft_size: 512,
            soft_type: 1,
        };
        
        assert_eq!(software.npc_id, 123);
        assert_eq!(software.soft_name, "TestSoft");
        assert_eq!(software.soft_version, 15);
    }
}