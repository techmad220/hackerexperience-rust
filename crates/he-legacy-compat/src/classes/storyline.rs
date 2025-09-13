// STORYLINE.CLASS.PHP PORT - Mission system, SafeNet, FBI, and round management
// Original: Complex storyline system with law enforcement mechanics

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use sqlx::MySqlPool;
use serde::{Deserialize, Serialize};
use he_core::*;
use he_db::*;
use crate::classes::system::System;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeNetEntry {
    pub ip: u32,
    pub reason: i32,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub on_fbi: bool,
    pub info: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FbiEntry {
    pub ip: u32,
    pub reason: i32,
    pub bounty: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub info: String,
    pub incidents: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRound {
    pub id: i32,
    pub name: String,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub status: i32, // 0 = inactive, 1 = active, 2 = ended
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundStats {
    pub round_id: i32,
    pub total_players: i32,
    pub total_hacks: i32,
    pub total_money_transferred: i64,
    pub total_viruses: i32,
    pub clans_created: i32,
}

pub struct Storyline {
    db_pool: MySqlPool,
    fbi_ip: Option<u32>,
    safenet_ip: Option<u32>,
}

impl Storyline {
    pub fn new(db_pool: MySqlPool) -> Self {
        Self {
            db_pool,
            fbi_ip: None,
            safenet_ip: None,
        }
    }
    
    // Original PHP: safenet_list - Display SafeNet entries with masked IPs
    pub async fn safenet_list(&self) -> Result<String, StorylineError> {
        let entries = sqlx::query_as::<_, SafeNetEntry>(
            "SELECT ip, reason, start_time, end_time, on_fbi, info FROM safeNet ORDER BY reason, start_time ASC"
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(StorylineError::DatabaseError)?;
        
        if entries.is_empty() {
            return Ok("No IPs harming internet".to_string());
        }
        
        let mut output = String::new();
        for entry in entries {
            let reason_text = match entry.reason {
                1 => "DDoS",
                2 => "Doom",
                3 => "Illegal Transfer",
                4 => "Delete file",
                _ => "Unknown",
            };
            
            let real_ip = System::long_to_ip(entry.ip).unwrap_or_else(|_| "0.0.0.0".to_string());
            let masked_ip = Self::mask_ip(&real_ip);
            
            output.push_str(&format!("{} - reason: {}<br/>", masked_ip, reason_text));
        }
        
        Ok(output)
    }
    
    // Original PHP: safenet_onFBI - Check if SafeNet entry is on FBI list
    pub async fn safenet_on_fbi(&self, ip: u32, reason: i32) -> Result<bool, StorylineError> {
        let on_fbi = sqlx::query_scalar::<_, Option<bool>>(
            "SELECT on_fbi FROM safeNet WHERE ip = ? AND reason = ? LIMIT 1"
        )
        .bind(ip)
        .bind(reason)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(StorylineError::DatabaseError)?
        .flatten();
        
        Ok(on_fbi.unwrap_or(false))
    }
    
    // Original PHP: safenet_add - Add IP to SafeNet
    pub async fn safenet_add(&self, ip: u32, reason: i32, info: Option<String>) -> Result<(), StorylineError> {
        if self.safenet_exists(ip, reason).await? {
            return Ok(()); // Already exists
        }
        
        let info_text = info.unwrap_or_else(|| {
            match reason {
                1 => "DDoS attack detected",
                2 => "Doom virus deployment",
                3 => "Illegal money transfer",
                4 => "Unauthorized file deletion",
                _ => "Unknown violation",
            }.to_string()
        });
        
        // Calculate end time based on reason (different violations have different durations)
        let hours_duration = match reason {
            1 => 24,  // DDoS - 24 hours
            2 => 72,  // Doom - 72 hours
            3 => 48,  // Illegal Transfer - 48 hours
            4 => 12,  // Delete file - 12 hours
            _ => 24,  // Default - 24 hours
        };
        
        sqlx::query(
            "INSERT INTO safeNet (ip, reason, start_time, end_time, on_fbi, info) 
             VALUES (?, ?, NOW(), DATE_ADD(NOW(), INTERVAL ? HOUR), FALSE, ?)"
        )
        .bind(ip)
        .bind(reason)
        .bind(hours_duration)
        .bind(&info_text)
        .execute(&self.db_pool)
        .await
        .map_err(StorylineError::DatabaseError)?;
        
        // Check if this should also go to FBI
        if self.should_escalate_to_fbi(ip, reason).await? {
            self.fbi_add(ip, reason, Some(info_text)).await?;
        }
        
        Ok(())
    }
    
    // Original PHP: safenet_isset - Check if IP exists in SafeNet
    pub async fn safenet_exists(&self, ip: u32, reason: i32) -> Result<bool, StorylineError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM safeNet WHERE ip = ? AND reason = ? AND end_time > NOW()"
        )
        .bind(ip)
        .bind(reason)
        .fetch_one(&self.db_pool)
        .await
        .map_err(StorylineError::DatabaseError)?;
        
        Ok(count > 0)
    }
    
    // Original PHP: safenet_until - Get time until SafeNet entry expires
    pub async fn safenet_until(&self, ip: u32) -> Result<Option<DateTime<Utc>>, StorylineError> {
        let end_time = sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
            "SELECT MIN(end_time) FROM safeNet WHERE ip = ? AND end_time > NOW()"
        )
        .bind(ip)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(StorylineError::DatabaseError)?
        .flatten();
        
        Ok(end_time)
    }
    
    // Original PHP: safenet_monitorTransfers - Monitor money transfers for illegal activity
    pub async fn safenet_monitor_transfers(&self, amount: i64, user_ip: u32) -> Result<(), StorylineError> {
        // Check for suspicious transfer amounts (original logic preserved)
        let is_suspicious = amount > 1000000 || amount < 0; // Large amounts or negative (hack attempts)
        
        if is_suspicious {
            let info = format!("Suspicious transfer of ${}", amount);
            self.safenet_add(user_ip, 3, Some(info)).await?;
        }
        
        Ok(())
    }
    
    // Original PHP: fbi_add - Add IP to FBI wanted list
    pub async fn fbi_add(&self, ip: u32, reason: i32, info: Option<String>) -> Result<(), StorylineError> {
        if self.fbi_exists(ip, Some(reason)).await? {
            // Update existing entry
            return self.fbi_update(ip, reason, info).await;
        }
        
        let info_text = info.unwrap_or_else(|| "Criminal activity detected".to_string());
        let bounty = self.fbi_calculate_bounty(reason, &info_text);
        let duration_hours = self.fbi_calculate_time(reason, &info_text, 0);
        
        sqlx::query(
            "INSERT INTO fbi (ip, reason, bounty, start_time, end_time, info, incidents) 
             VALUES (?, ?, ?, NOW(), DATE_ADD(NOW(), INTERVAL ? HOUR), ?, 1)"
        )
        .bind(ip)
        .bind(reason)
        .bind(bounty)
        .bind(duration_hours)
        .bind(&info_text)
        .execute(&self.db_pool)
        .await
        .map_err(StorylineError::DatabaseError)?;
        
        // Update SafeNet to mark as on FBI
        sqlx::query("UPDATE safeNet SET on_fbi = TRUE WHERE ip = ?")
            .bind(ip)
            .execute(&self.db_pool)
            .await
            .map_err(StorylineError::DatabaseError)?;
        
        Ok(())
    }
    
    // Original PHP: fbi_exists - Check if IP is on FBI list
    pub async fn fbi_exists(&self, ip: u32, reason: Option<i32>) -> Result<bool, StorylineError> {
        let count = if let Some(r) = reason {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM fbi WHERE ip = ? AND reason = ? AND end_time > NOW()"
            )
            .bind(ip)
            .bind(r)
            .fetch_one(&self.db_pool)
            .await
            .map_err(StorylineError::DatabaseError)?
        } else {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM fbi WHERE ip = ? AND end_time > NOW()"
            )
            .bind(ip)
            .fetch_one(&self.db_pool)
            .await
            .map_err(StorylineError::DatabaseError)?
        };
        
        Ok(count > 0)
    }
    
    // Original PHP: fbi_getBounty - Get bounty for IP
    pub async fn fbi_get_bounty(&self, ip: u32) -> Result<i64, StorylineError> {
        let bounty = sqlx::query_scalar::<_, Option<i64>>(
            "SELECT SUM(bounty) FROM fbi WHERE ip = ? AND end_time > NOW()"
        )
        .bind(ip)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(StorylineError::DatabaseError)?
        .flatten();
        
        Ok(bounty.unwrap_or(0))
    }
    
    // Original PHP: fbi_payBounty - Pay bounty to bounty hunter
    pub async fn fbi_pay_bounty(&self, ip: u32, hunter_id: i64) -> Result<i64, StorylineError> {
        let total_bounty = self.fbi_get_bounty(ip).await?;
        
        if total_bounty > 0 {
            // TODO: Add money to hunter's account
            // finances.add_money(total_bounty, hunter_account);
            
            // Remove from FBI list
            sqlx::query("DELETE FROM fbi WHERE ip = ? AND end_time > NOW()")
                .bind(ip)
                .execute(&self.db_pool)
                .await
                .map_err(StorylineError::DatabaseError)?;
            
            // Update SafeNet
            sqlx::query("UPDATE safeNet SET on_fbi = FALSE WHERE ip = ?")
                .bind(ip)
                .execute(&self.db_pool)
                .await
                .map_err(StorylineError::DatabaseError)?;
        }
        
        Ok(total_bounty)
    }
    
    // Original PHP: fbi_list - Get FBI wanted list
    pub async fn fbi_list(&self) -> Result<Vec<FbiEntry>, StorylineError> {
        let entries = sqlx::query_as::<_, FbiEntry>(
            "SELECT ip, reason, bounty, start_time, end_time, info, incidents 
             FROM fbi 
             WHERE end_time > NOW() 
             ORDER BY bounty DESC"
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(StorylineError::DatabaseError)?;
        
        Ok(entries)
    }
    
    // Original PHP: round_status - Get current round status
    pub async fn round_status(&self) -> Result<i32, StorylineError> {
        let status = sqlx::query_scalar::<_, Option<i32>>(
            "SELECT status FROM game_rounds WHERE status = 1 ORDER BY start_date DESC LIMIT 1"
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(StorylineError::DatabaseError)?
        .flatten();
        
        Ok(status.unwrap_or(0))
    }
    
    // Original PHP: round_current - Get current round
    pub async fn round_current(&self) -> Result<Option<GameRound>, StorylineError> {
        let round = sqlx::query_as::<_, GameRound>(
            "SELECT id, name, start_date, end_date, status, description 
             FROM game_rounds 
             WHERE status = 1 
             ORDER BY start_date DESC 
             LIMIT 1"
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(StorylineError::DatabaseError)?;
        
        Ok(round)
    }
    
    // Original PHP: round_getAll - Get all rounds
    pub async fn round_get_all(&self) -> Result<Vec<GameRound>, StorylineError> {
        let rounds = sqlx::query_as::<_, GameRound>(
            "SELECT id, name, start_date, end_date, status, description 
             FROM game_rounds 
             ORDER BY start_date DESC"
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(StorylineError::DatabaseError)?;
        
        Ok(rounds)
    }
    
    // Original PHP: round_stats - Get round statistics
    pub async fn round_stats(&self, round_id: i32) -> Result<RoundStats, StorylineError> {
        let stats = sqlx::query_as::<_, RoundStats>(
            "SELECT round_id, total_players, total_hacks, total_money_transferred, total_viruses, clans_created
             FROM round_statistics 
             WHERE round_id = ?"
        )
        .bind(round_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(StorylineError::DatabaseError)?;
        
        // If no stats exist, calculate them
        let stats = if let Some(s) = stats {
            s
        } else {
            self.calculate_round_stats(round_id).await?
        };
        
        Ok(stats)
    }
    
    // Helper methods
    async fn should_escalate_to_fbi(&self, ip: u32, reason: i32) -> Result<bool, StorylineError> {
        // Check incident count for this IP
        let incident_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM safeNet WHERE ip = ?"
        )
        .bind(ip)
        .fetch_one(&self.db_pool)
        .await
        .map_err(StorylineError::DatabaseError)?;
        
        // Escalate to FBI if multiple incidents or severe violation
        Ok(incident_count >= 3 || reason == 2) // 3+ incidents or Doom virus
    }
    
    async fn fbi_update(&self, ip: u32, reason: i32, info: Option<String>) -> Result<(), StorylineError> {
        let info_text = info.unwrap_or_else(|| "Updated criminal activity".to_string());
        let additional_bounty = self.fbi_calculate_bounty(reason, &info_text) / 2; // Half bounty for updates
        
        sqlx::query(
            "UPDATE fbi 
             SET bounty = bounty + ?, info = ?, incidents = incidents + 1, end_time = DATE_ADD(NOW(), INTERVAL 48 HOUR)
             WHERE ip = ? AND reason = ?"
        )
        .bind(additional_bounty)
        .bind(&info_text)
        .bind(ip)
        .bind(reason)
        .execute(&self.db_pool)
        .await
        .map_err(StorylineError::DatabaseError)?;
        
        Ok(())
    }
    
    fn fbi_calculate_bounty(&self, reason: i32, info: &str) -> i64 {
        let base_bounty = match reason {
            1 => 50000,   // DDoS
            2 => 200000,  // Doom virus
            3 => 100000,  // Illegal transfer
            4 => 25000,   // File deletion
            _ => 10000,   // Default
        };
        
        // Increase bounty based on info severity (simplified)
        let multiplier = if info.contains("massive") || info.contains("critical") {
            2.0
        } else if info.contains("serious") || info.contains("major") {
            1.5
        } else {
            1.0
        };
        
        (base_bounty as f64 * multiplier) as i64
    }
    
    fn fbi_calculate_time(&self, reason: i32, _info: &str, reincident: i32) -> i32 {
        let base_hours = match reason {
            1 => 48,   // DDoS - 48 hours
            2 => 168,  // Doom - 1 week
            3 => 72,   // Illegal transfer - 72 hours
            4 => 24,   // File deletion - 24 hours
            _ => 48,   // Default
        };
        
        // Increase time for repeat offenders
        base_hours + (reincident * 24)
    }
    
    fn mask_ip(ip: &str) -> String {
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() != 4 {
            return ip.to_string();
        }
        
        format!("{}.{}.X.X", parts[0], parts[1])
    }
    
    async fn calculate_round_stats(&self, round_id: i32) -> Result<RoundStats, StorylineError> {
        // This would calculate stats from various tables
        // For now, return default stats
        Ok(RoundStats {
            round_id,
            total_players: 0,
            total_hacks: 0,
            total_money_transferred: 0,
            total_viruses: 0,
            clans_created: 0,
        })
    }
}

// Implement FromRow for database types
impl sqlx::FromRow<'_, sqlx::mysql::MySqlRow> for SafeNetEntry {
    fn from_row(row: &'_ sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(SafeNetEntry {
            ip: row.try_get("ip")?,
            reason: row.try_get("reason")?,
            start_time: row.try_get("start_time")?,
            end_time: row.try_get("end_time")?,
            on_fbi: row.try_get("on_fbi")?,
            info: row.try_get("info")?,
        })
    }
}

impl sqlx::FromRow<'_, sqlx::mysql::MySqlRow> for FbiEntry {
    fn from_row(row: &'_ sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(FbiEntry {
            ip: row.try_get("ip")?,
            reason: row.try_get("reason")?,
            bounty: row.try_get("bounty")?,
            start_time: row.try_get("start_time")?,
            end_time: row.try_get("end_time")?,
            info: row.try_get("info")?,
            incidents: row.try_get("incidents")?,
        })
    }
}

impl sqlx::FromRow<'_, sqlx::mysql::MySqlRow> for GameRound {
    fn from_row(row: &'_ sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(GameRound {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            start_date: row.try_get("start_date")?,
            end_date: row.try_get("end_date")?,
            status: row.try_get("status")?,
            description: row.try_get("description")?,
        })
    }
}

impl sqlx::FromRow<'_, sqlx::mysql::MySqlRow> for RoundStats {
    fn from_row(row: &'_ sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(RoundStats {
            round_id: row.try_get("round_id")?,
            total_players: row.try_get("total_players")?,
            total_hacks: row.try_get("total_hacks")?,
            total_money_transferred: row.try_get("total_money_transferred")?,
            total_viruses: row.try_get("total_viruses")?,
            clans_created: row.try_get("clans_created")?,
        })
    }
}

#[derive(Debug)]
pub enum StorylineError {
    DatabaseError(sqlx::Error),
    InvalidIp(String),
    InvalidReason(i32),
    RoundNotFound(i32),
}

impl std::fmt::Display for StorylineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorylineError::DatabaseError(e) => write!(f, "Database error: {}", e),
            StorylineError::InvalidIp(ip) => write!(f, "Invalid IP address: {}", ip),
            StorylineError::InvalidReason(reason) => write!(f, "Invalid reason code: {}", reason),
            StorylineError::RoundNotFound(id) => write!(f, "Round {} not found", id),
        }
    }
}

impl std::error::Error for StorylineError {}