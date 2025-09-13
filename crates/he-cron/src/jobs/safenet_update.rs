//! SafeNet update job
//! 
//! This job manages the SafeNet system, cleaning up expired records and
//! generating reports. Equivalent to the legacy safenetUpdate.php cron job.

use crate::error::{CronError, CronResult};
use crate::utils::long_to_ip;
use sqlx::{MySqlPool, Row};
use std::sync::Arc;
use tracing::{info, warn, error};

/// SafeNet update job implementation
pub struct SafeNetUpdateJob;

#[derive(Debug)]
struct SafeNetEntry {
    ip: u32,
    reason: i32,
}

impl SafeNetUpdateJob {
    /// Execute the SafeNet update job
    pub async fn execute(db_pool: Arc<MySqlPool>) -> CronResult<()> {
        info!("Starting SafeNet update");
        
        // Clean up expired SafeNet entries
        Self::cleanup_expired_entries(&db_pool).await?;
        
        // Get remaining SafeNet entries
        let entries = Self::get_safenet_entries(&db_pool).await?;
        
        if entries.is_empty() {
            info!("No SafeNet entries to process");
            return Ok(());
        }
        
        info!("Processing {} SafeNet entries", entries.len());
        
        // Generate report
        Self::generate_report(&db_pool, &entries).await?;
        
        info!("SafeNet update completed");
        Ok(())
    }
    
    /// Clean up expired SafeNet entries
    async fn cleanup_expired_entries(db_pool: &MySqlPool) -> CronResult<()> {
        let deleted = sqlx::query(
            "DELETE FROM safeNet WHERE TIMESTAMPDIFF(SECOND, NOW(), endTime) < 0"
        )
        .execute(db_pool)
        .await?
        .rows_affected();
        
        if deleted > 0 {
            info!("Cleaned up {} expired SafeNet entries", deleted);
        }
        
        Ok(())
    }
    
    /// Get current SafeNet entries
    async fn get_safenet_entries(db_pool: &MySqlPool) -> CronResult<Vec<SafeNetEntry>> {
        let rows = sqlx::query(
            "SELECT IP, reason FROM safeNet"
        )
        .fetch_all(db_pool)
        .await?;
        
        let mut entries = Vec::new();
        for row in rows {
            entries.push(SafeNetEntry {
                ip: row.try_get::<u32, _>("IP")?,
                reason: row.try_get::<i32, _>("reason")?,
            });
        }
        
        Ok(entries)
    }
    
    /// Generate SafeNet report and create text file for SafeNet NPC
    async fn generate_report(db_pool: &MySqlPool, entries: &[SafeNetEntry]) -> CronResult<()> {
        // Build report content
        let mut report = String::from("Dolan, send this brief to the FBI please: <br/><br/>");
        
        for entry in entries {
            let ip_str = long_to_ip(entry.ip);
            let reason_str = Self::get_reason_string(entry.reason);
            report.push_str(&format!("IP [{}] caught for {}<br/>", ip_str, reason_str));
        }
        
        // Get SafeNet NPC ID
        let safenet_npc = Self::get_safenet_npc(db_pool).await?;
        
        // Remove old report if exists
        Self::remove_old_report(db_pool, safenet_npc).await?;
        
        // Create new report file
        Self::create_report_file(db_pool, safenet_npc, &report).await?;
        
        info!("Generated SafeNet report with {} entries", entries.len());
        Ok(())
    }
    
    /// Get reason string for SafeNet entry
    fn get_reason_string(reason: i32) -> &'static str {
        match reason {
            1 => "DDoS",
            _ => "Unknown",
        }
    }
    
    /// Get SafeNet NPC ID
    async fn get_safenet_npc(db_pool: &MySqlPool) -> CronResult<i32> {
        let row = sqlx::query(
            "SELECT id FROM npc WHERE npcType = 50 LIMIT 1"
        )
        .fetch_optional(db_pool)
        .await?;
        
        if let Some(row) = row {
            Ok(row.try_get("id")?)
        } else {
            Err(CronError::Runtime("SafeNet NPC not found".to_string()))
        }
    }
    
    /// Remove old SafeNet report files
    async fn remove_old_report(db_pool: &MySqlPool, safenet_npc: i32) -> CronResult<()> {
        // Get IDs of old text files
        let old_files = sqlx::query(
            "SELECT software_texts.id 
             FROM software_texts
             INNER JOIN software ON software.id = software_texts.id
             WHERE software.userID = ? AND software.isNPC = 1 AND softType = 30"
        )
        .bind(safenet_npc)
        .fetch_all(db_pool)
        .await?;
        
        for file_row in old_files {
            let file_id: i32 = file_row.try_get("id")?;
            
            // Delete from software table
            sqlx::query("DELETE FROM software WHERE id = ? LIMIT 1")
                .bind(file_id)
                .execute(db_pool)
                .await?;
            
            // Delete from software_texts table
            sqlx::query("DELETE FROM software_texts WHERE id = ? LIMIT 1")
                .bind(file_id)
                .execute(db_pool)
                .await?;
        }
        
        if !old_files.is_empty() {
            info!("Removed {} old SafeNet report files", old_files.len());
        }
        
        Ok(())
    }
    
    /// Create new SafeNet report file
    async fn create_report_file(db_pool: &MySqlPool, safenet_npc: i32, content: &str) -> CronResult<()> {
        // Insert software entry
        let software_id = sqlx::query(
            "INSERT INTO software 
             (id, userID, softName, softVersion, softSize, softRam, softType, 
              softLastEdit, softHidden, softHiddenWith, isNPC, licensedTo)
             VALUES (NULL, ?, 'Fwd to FBI', 0, 1, 0, 30, NOW(), 0, 0, 1, 0)"
        )
        .bind(safenet_npc)
        .execute(db_pool)
        .await?
        .last_insert_id();
        
        // Insert text content
        sqlx::query(
            "INSERT INTO software_texts (id, userID, isNPC, text, lastEdit) 
             VALUES (?, ?, 1, ?, NOW())"
        )
        .bind(software_id)
        .bind(safenet_npc)
        .bind(content)
        .execute(db_pool)
        .await?;
        
        info!("Created SafeNet report file with ID {}", software_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reason_string() {
        assert_eq!(SafeNetUpdateJob::get_reason_string(1), "DDoS");
        assert_eq!(SafeNetUpdateJob::get_reason_string(999), "Unknown");
    }
    
    #[test]
    fn test_safenet_entry() {
        let entry = SafeNetEntry {
            ip: 3232235777, // 192.168.1.1
            reason: 1,
        };
        
        assert_eq!(entry.ip, 3232235777);
        assert_eq!(entry.reason, 1);
    }
}