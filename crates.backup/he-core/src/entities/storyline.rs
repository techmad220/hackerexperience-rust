use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use chrono::{DateTime, Utc};

use crate::error::Result;

/// SafeNet tracking entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SafeNetEntry {
    pub ip: i64,
    pub reason: i32,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub count: i32,
    pub on_fbi: bool,
}

/// FBI wanted entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FbiEntry {
    pub ip: i64,
    pub reason: i32,
    pub bounty: i32,
    pub date_add: DateTime<Utc>,
    pub date_end: DateTime<Utc>,
}

/// Round information
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Round {
    pub id: i32,
    pub name: String,
    pub start_date: DateTime<Utc>,
    pub status: i32,
}

/// Round statistics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RoundStats {
    pub active_users: i32,
    pub ddos_count: i32,
    pub hack_count: i32,
    pub research_count: i32,
}

/// Doom virus information
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DoomVirus {
    pub doom_id: i32,
    pub doom_ip: i64,
    pub creator_id: i32,
    pub clan_id: i32,
    pub release_date: DateTime<Utc>,
    pub doom_date: DateTime<Utc>,
    pub status: i32,
}

#[async_trait]
pub trait StorylineRepository {
    // SafeNet operations
    async fn safenet_add(&self, ip: i64, reason: i32, info: Option<i32>) -> Result<()>;
    async fn safenet_update(&self, ip: i64, reason: i32, info: Option<i32>) -> Result<()>;
    async fn safenet_exists(&self, ip: i64, reason: i32) -> Result<bool>;
    async fn safenet_on_fbi(&self, ip: i64, reason: i32) -> Result<bool>;
    async fn safenet_get_ip(&self) -> Result<i64>;
    async fn safenet_until(&self, ip: i64) -> Result<Option<DateTime<Utc>>>;
    async fn safenet_list(&self) -> Result<Vec<SafeNetEntry>>;
    async fn safenet_monitor_transfers(&self, amount: i32, user_ip: i64) -> Result<()>;
    
    // FBI operations
    async fn fbi_add(&self, ip: i64, reason: i32, info: Option<i32>) -> Result<()>;
    async fn fbi_update(&self, ip: i64, reason: i32, info: i32) -> Result<()>;
    async fn fbi_exists(&self, ip: i64, reason: Option<i32>) -> Result<bool>;
    async fn fbi_get_bounty(&self, ip: i64) -> Result<i32>;
    async fn fbi_until(&self, ip: i64) -> Result<Option<DateTime<Utc>>>;
    async fn fbi_get_ip(&self) -> Result<i64>;
    async fn fbi_list(&self) -> Result<Vec<FbiEntry>>;
    async fn fbi_pay_bounty(&self, ip: i64) -> Result<()>;
    
    // Round operations
    async fn round_current(&self) -> Result<i32>;
    async fn round_status(&self) -> Result<i32>;
    async fn round_get_all(&self) -> Result<Vec<Round>>;
    async fn round_stats(&self, round_id: i32) -> Result<Option<RoundStats>>;
    async fn round_time_to_start(&self) -> Result<String>;
    
    // Doom operations
    async fn doom_stats(&self, round_id: i32) -> Result<Vec<DoomVirus>>;
    async fn doom_total_services(&self) -> Result<i32>;
    async fn doom_list_services(&self) -> Result<Vec<DoomVirus>>;
    
    // NSA operations
    async fn nsa_get_id(&self) -> Result<i32>;
    async fn nsa_get_ip(&self) -> Result<i64>;
    async fn nsa_have_doom(&self) -> Result<bool>;
    async fn nsa_install_doom(&self) -> Result<()>;
    async fn nsa_get_doom_id(&self) -> Result<i32>;
    
    // Special NPC operations
    async fn md_get_id(&self) -> Result<i32>;
    async fn md_get_ip(&self) -> Result<i64>;
    async fn evilcorp_get_id(&self) -> Result<i32>;
    async fn evilcorp_get_ip(&self) -> Result<i64>;
    async fn evilcorp_get_name(&self) -> Result<String>;
    
    // Tutorial operations
    async fn tutorial_create_victims(&self) -> Result<Vec<i64>>;
    async fn tutorial_start(&self, user_id: i32) -> Result<()>;
    async fn tutorial_set_expire_date(&self, ip1: i64, ip2: i64) -> Result<()>;
}

pub struct StorylineService {
    db: PgPool,
}

impl StorylineService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
    
    /// Calculate time duration for SafeNet/FBI tracking
    fn calculate_time_duration(reason: i32, info: Option<i32>) -> i32 {
        match reason {
            1 => info.map(|i| (i as f32 * 0.8) as i32).unwrap_or(0), // DDoS
            2 => 604800, // Doom (1 week)
            3 => {
                // Transfer
                let amount = info.unwrap_or(0) as i32;
                std::cmp::max((amount as f32 * 0.01) as i32, 3600)
            }
            4 => 1000, // Delete file
            _ => 0,
        }
    }
    
    /// Calculate bounty for FBI wanted list
    fn calculate_bounty(reason: i32, info: Option<i32>) -> i32 {
        match reason {
            1 => info.map(|i| i / 100).unwrap_or(0), // DDoS
            2 => 100000, // Doom
            3 => {
                // Transfer
                let amount = info.unwrap_or(0);
                std::cmp::min((amount as f32 * 0.10) as i32, 100000)
            }
            4 => 1000, // Delete file
            _ => 0,
        }
    }
}

#[async_trait]
impl StorylineRepository for StorylineService {
    /// Add IP to SafeNet tracking
    async fn safenet_add(&self, ip: i64, reason: i32, info: Option<i32>) -> Result<()> {
        if !self.safenet_exists(ip, reason).await? {
            let duration = Self::calculate_time_duration(reason, info);
            
            sqlx::query!(
                "INSERT INTO safenet (ip, reason, start_time, end_time, count, on_fbi)
                 VALUES ($1, $2, NOW(), NOW() + INTERVAL '$3 seconds', 1, false)",
                ip,
                reason,
                duration
            )
            .execute(&self.db)
            .await?;
            
            // TODO: Send mail notification
        } else {
            self.safenet_update(ip, reason, info).await?;
        }
        
        Ok(())
    }
    
    /// Update existing SafeNet entry
    async fn safenet_update(&self, ip: i64, reason: i32, info: Option<i32>) -> Result<()> {
        let duration = Self::calculate_time_duration(reason, info);
        
        sqlx::query!(
            "UPDATE safenet 
             SET count = count + 1, end_time = end_time + INTERVAL '$3 seconds'
             WHERE ip = $1 AND reason = $2",
            ip,
            reason,
            duration
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Check if IP is already tracked by SafeNet for specific reason
    async fn safenet_exists(&self, ip: i64, reason: i32) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM safenet WHERE ip = $1 AND reason = $2",
            ip,
            reason
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) > 0)
    }
    
    /// Check if SafeNet entry is on FBI list
    async fn safenet_on_fbi(&self, ip: i64, reason: i32) -> Result<bool> {
        let on_fbi = sqlx::query_scalar!(
            "SELECT on_fbi FROM safenet WHERE ip = $1 AND reason = $2 LIMIT 1",
            ip,
            reason
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(on_fbi.unwrap_or(false))
    }
    
    /// Get SafeNet IP address
    async fn safenet_get_ip(&self) -> Result<i64> {
        let ip = sqlx::query_scalar!(
            "SELECT npc_ip FROM npc WHERE npc_type = 50 LIMIT 1"
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(ip)
    }
    
    /// Get when SafeNet tracking ends for IP
    async fn safenet_until(&self, ip: i64) -> Result<Option<DateTime<Utc>>> {
        let end_time = sqlx::query_scalar!(
            "SELECT end_time FROM safenet WHERE ip = $1 ORDER BY end_time DESC LIMIT 1",
            ip
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(end_time)
    }
    
    /// List all SafeNet entries
    async fn safenet_list(&self) -> Result<Vec<SafeNetEntry>> {
        let entries = sqlx::query_as!(
            SafeNetEntry,
            "SELECT ip, reason, start_time, end_time, count, on_fbi 
             FROM safenet 
             ORDER BY reason, start_time ASC"
        )
        .fetch_all(&self.db)
        .await?;
        
        Ok(entries)
    }
    
    /// Monitor money transfers for suspicious activity
    async fn safenet_monitor_transfers(&self, amount: i32, user_ip: i64) -> Result<()> {
        if amount >= 100000 {
            let div = std::cmp::min(amount / 10000, 50);
            let odds = div as f32 / 100.0;
            
            // Random chance based on amount
            let random = rand::random::<f32>();
            
            if !self.safenet_exists(user_ip, 3).await? {
                if odds >= random {
                    self.safenet_add(user_ip, 3, Some(amount)).await?;
                }
            } else {
                self.safenet_update(user_ip, 3, Some(amount)).await?;
                
                if !self.fbi_exists(user_ip, Some(3)).await? {
                    if odds >= random {
                        self.fbi_add(user_ip, 3, Some(amount)).await?;
                    }
                } else {
                    self.fbi_update(user_ip, 3, amount).await?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Add IP to FBI wanted list
    async fn fbi_add(&self, ip: i64, reason: i32, info: Option<i32>) -> Result<()> {
        if !self.fbi_exists(ip, Some(reason)).await? {
            // Update SafeNet entry
            sqlx::query!(
                "UPDATE safenet SET on_fbi = true WHERE ip = $1 AND reason = $2",
                ip,
                reason
            )
            .execute(&self.db)
            .await?;
            
            let duration = Self::calculate_time_duration(reason, info);
            let bounty = Self::calculate_bounty(reason, info);
            
            sqlx::query!(
                "INSERT INTO fbi (ip, reason, bounty, date_add, date_end)
                 VALUES ($1, $2, $3, NOW(), NOW() + INTERVAL '$4 seconds')",
                ip,
                reason,
                bounty,
                duration
            )
            .execute(&self.db)
            .await?;
            
            // TODO: Send mail notification
        } else {
            self.fbi_update(ip, reason, info.unwrap_or(0)).await?;
        }
        
        Ok(())
    }
    
    /// Update FBI entry
    async fn fbi_update(&self, ip: i64, reason: i32, info: i32) -> Result<()> {
        let duration = Self::calculate_time_duration(reason, Some(info));
        let bounty = Self::calculate_bounty(reason, Some(info));
        
        sqlx::query!(
            "UPDATE fbi 
             SET date_end = date_end + INTERVAL '$4 seconds', bounty = bounty + $3
             WHERE ip = $1 AND reason = $2",
            ip,
            reason,
            bounty,
            duration
        )
        .execute(&self.db)
        .await?;
        
        // TODO: Send bounty increase mail
        Ok(())
    }
    
    /// Check if IP exists in FBI list
    async fn fbi_exists(&self, ip: i64, reason: Option<i32>) -> Result<bool> {
        let count = match reason {
            Some(r) => {
                sqlx::query_scalar!(
                    "SELECT COUNT(*) FROM fbi WHERE ip = $1 AND reason = $2",
                    ip, r
                ).fetch_one(&self.db).await?
            }
            None => {
                sqlx::query_scalar!(
                    "SELECT COUNT(*) FROM fbi WHERE ip = $1",
                    ip
                ).fetch_one(&self.db).await?
            }
        };
        
        Ok(count.unwrap_or(0) > 0)
    }
    
    /// Get total bounty for IP
    async fn fbi_get_bounty(&self, ip: i64) -> Result<i32> {
        let bounty = sqlx::query_scalar!(
            "SELECT SUM(bounty) FROM fbi WHERE ip = $1",
            ip
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(bounty.flatten().unwrap_or(0) as i32)
    }
    
    /// Get FBI tracking end date
    async fn fbi_until(&self, ip: i64) -> Result<Option<DateTime<Utc>>> {
        let date_end = sqlx::query_scalar!(
            "SELECT date_end FROM fbi WHERE ip = $1 ORDER BY date_end DESC LIMIT 1",
            ip
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(date_end)
    }
    
    /// Get FBI IP address
    async fn fbi_get_ip(&self) -> Result<i64> {
        let ip = sqlx::query_scalar!(
            "SELECT npc_ip FROM npc WHERE npc_type = 51 LIMIT 1"
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(ip)
    }
    
    /// List FBI wanted entries
    async fn fbi_list(&self) -> Result<Vec<FbiEntry>> {
        let entries = sqlx::query_as!(
            FbiEntry,
            "SELECT ip, reason, bounty, date_add, date_end 
             FROM fbi 
             ORDER BY reason, date_add ASC"
        )
        .fetch_all(&self.db)
        .await?;
        
        Ok(entries)
    }
    
    /// Pay bounty and remove from wanted list
    async fn fbi_pay_bounty(&self, ip: i64) -> Result<()> {
        // Delete FBI entries
        sqlx::query!(
            "DELETE FROM fbi WHERE ip = $1",
            ip
        )
        .execute(&self.db)
        .await?;
        
        // Delete SafeNet entries
        sqlx::query!(
            "DELETE FROM safenet WHERE ip = $1",
            ip
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Get current round ID
    async fn round_current(&self) -> Result<i32> {
        let id = sqlx::query_scalar!(
            "SELECT id FROM round ORDER BY id DESC LIMIT 1"
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(id)
    }
    
    /// Get round status
    async fn round_status(&self) -> Result<i32> {
        let status = sqlx::query_scalar!(
            "SELECT status FROM round ORDER BY id DESC LIMIT 1"
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(status)
    }
    
    /// Get all rounds
    async fn round_get_all(&self) -> Result<Vec<Round>> {
        let rounds = sqlx::query_as!(
            Round,
            "SELECT id, name, start_date, status FROM round ORDER BY id DESC"
        )
        .fetch_all(&self.db)
        .await?;
        
        Ok(rounds)
    }
    
    /// Get round statistics
    async fn round_stats(&self, round_id: i32) -> Result<Option<RoundStats>> {
        let stats = sqlx::query_as!(
            RoundStats,
            "SELECT active_users, ddos_count, hack_count, research_count 
             FROM round_stats WHERE id = $1",
            round_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(stats)
    }
    
    /// Get time until round starts
    async fn round_time_to_start(&self) -> Result<String> {
        let diff = sqlx::query_scalar!(
            "SELECT EXTRACT(epoch FROM (start_date - NOW())) FROM round ORDER BY id DESC LIMIT 1"
        )
        .fetch_one(&self.db)
        .await?;
        
        let seconds = diff.unwrap_or(0.0) as i64;
        
        let time_str = if seconds < 60 {
            "ONE MINUTE!!!".to_string()
        } else if seconds < 3600 {
            "less than one hour!".to_string()
        } else {
            let hours = seconds / 3600;
            if hours < 24 {
                format!("{} hours.", hours)
            } else {
                let days = hours / 24;
                if days == 1 {
                    "1 day".to_string()
                } else {
                    format!("{} days", days)
                }
            }
        };
        
        Ok(time_str)
    }
    
    /// Get doom statistics for round
    async fn doom_stats(&self, round_id: i32) -> Result<Vec<DoomVirus>> {
        let doom_entries = sqlx::query_as!(
            DoomVirus,
            "SELECT doom_id, doom_ip, creator_id, clan_id, release_date, doom_date, status
             FROM hist_doom WHERE round = $1",
            round_id
        )
        .fetch_all(&self.db)
        .await?;
        
        Ok(doom_entries)
    }
    
    /// Get total active doom services
    async fn doom_total_services(&self) -> Result<i32> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM virus_doom WHERE status = 1"
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) as i32)
    }
    
    /// List active doom services
    async fn doom_list_services(&self) -> Result<Vec<DoomVirus>> {
        let services = sqlx::query_as!(
            DoomVirus,
            "SELECT doom_id, doom_ip, creator_id, clan_id, release_date, doom_date, status
             FROM virus_doom WHERE status = 1"
        )
        .fetch_all(&self.db)
        .await?;
        
        Ok(services)
    }
    
    /// Get NSA NPC ID
    async fn nsa_get_id(&self) -> Result<i32> {
        let id = sqlx::query_scalar!(
            "SELECT id FROM npc WHERE npc_type = 52 LIMIT 1"
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(id)
    }
    
    /// Get NSA IP address
    async fn nsa_get_ip(&self) -> Result<i64> {
        let ip = sqlx::query_scalar!(
            "SELECT npc_ip FROM npc WHERE npc_type = 52 LIMIT 1"
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(ip)
    }
    
    /// Check if NSA has doom virus
    async fn nsa_have_doom(&self) -> Result<bool> {
        let nsa_id = self.nsa_get_id().await?;
        
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM software 
             WHERE soft_type = 29 AND user_id = $1 AND is_npc = true",
            nsa_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) > 0)
    }
    
    /// Install doom virus on NSA
    async fn nsa_install_doom(&self) -> Result<()> {
        let nsa_id = self.nsa_get_id().await?;
        
        sqlx::query!(
            "INSERT INTO software (user_id, soft_name, soft_version, soft_size, soft_ram, soft_type, soft_last_edit, soft_hidden, is_npc, original_from, licensed_to, is_folder)
             VALUES ($1, 'DooM', 10, 100, 512, 29, NOW(), false, true, 0, 0, false)",
            nsa_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Get NSA doom virus ID
    async fn nsa_get_doom_id(&self) -> Result<i32> {
        let nsa_id = self.nsa_get_id().await?;
        
        let doom_id = sqlx::query_scalar!(
            "SELECT id FROM software 
             WHERE soft_type = 29 AND user_id = $1 AND is_npc = true LIMIT 1",
            nsa_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(doom_id)
    }
    
    /// Get MD (Mission Distributor) NPC ID
    async fn md_get_id(&self) -> Result<i32> {
        let id = sqlx::query_scalar!(
            "SELECT npc_id FROM npc_key WHERE key = 'MD' LIMIT 1"
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(id)
    }
    
    /// Get MD IP address
    async fn md_get_ip(&self) -> Result<i64> {
        let md_id = self.md_get_id().await?;
        
        let ip = sqlx::query_scalar!(
            "SELECT npc_ip FROM npc WHERE id = $1 LIMIT 1",
            md_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(ip)
    }
    
    /// Get EvilCorp NPC ID
    async fn evilcorp_get_id(&self) -> Result<i32> {
        let id = sqlx::query_scalar!(
            "SELECT npc_id FROM npc_key WHERE key = 'EVILCORP' LIMIT 1"
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(id)
    }
    
    /// Get EvilCorp IP address
    async fn evilcorp_get_ip(&self) -> Result<i64> {
        let evilcorp_id = self.evilcorp_get_id().await?;
        
        let ip = sqlx::query_scalar!(
            "SELECT npc_ip FROM npc WHERE id = $1 LIMIT 1",
            evilcorp_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(ip)
    }
    
    /// Get EvilCorp name
    async fn evilcorp_get_name(&self) -> Result<String> {
        let evilcorp_id = self.evilcorp_get_id().await?;
        
        let name = sqlx::query_scalar!(
            "SELECT name FROM npc_info_en WHERE npc_id = $1 LIMIT 1",
            evilcorp_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(name)
    }
    
    /// Create tutorial victim NPCs
    async fn tutorial_create_victims(&self) -> Result<Vec<i64>> {
        // This would create two NPCs and return their IPs
        // Simplified implementation
        Ok(vec![0, 0]) // Placeholder IPs
    }
    
    /// Start tutorial for new player
    async fn tutorial_start(&self, user_id: i32) -> Result<()> {
        // Create tutorial mission
        let hirer = self.md_get_ip().await?;
        let victims = self.tutorial_create_victims().await?;
        
        // Would create mission and send welcome mail
        // Simplified implementation
        
        // Give starter cracker software
        sqlx::query!(
            "INSERT INTO software (user_id, soft_name, soft_version, soft_size, soft_ram, soft_type, soft_last_edit, soft_hidden, soft_hidden_with, is_npc, original_from)
             VALUES ($1, 'cracker', 10, 28, 9, 1, NOW(), false, '', false, '')",
            user_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Set expiration date for tutorial NPCs
    async fn tutorial_set_expire_date(&self, ip1: i64, ip2: i64) -> Result<()> {
        sqlx::query!(
            "INSERT INTO npc_expire (npc_id, expire_date)
             SELECT npc.id, NOW() + INTERVAL '7 days'
             FROM npc 
             WHERE npc_ip IN ($1, $2)",
            ip1,
            ip2
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
}

/// Helper functions for storyline text generation
impl StorylineService {
    /// Get reason text for SafeNet/FBI tracking
    pub fn get_reason_text(reason: i32) -> &'static str {
        match reason {
            1 => "DDoS",
            2 => "Doom",
            3 => "Illegal Transfer",
            4 => "Delete file",
            _ => "Unknown",
        }
    }
    
    /// Format IP address for display (partial masking for SafeNet)
    pub fn format_ip_for_safenet(ip: i64) -> String {
        let ip_str = format!("{}.{}.{}.{}", 
            (ip >> 24) & 0xFF,
            (ip >> 16) & 0xFF,
            (ip >> 8) & 0xFF,
            ip & 0xFF
        );
        
        let parts: Vec<&str> = ip_str.split('.').collect();
        if parts.len() == 4 {
            format!("{}.{}.X.X", parts[0], parts[1])
        } else {
            "X.X.X.X".to_string()
        }
    }
}