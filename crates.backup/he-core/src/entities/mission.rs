use async_trait::async_trait;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use chrono::{DateTime, Utc};

use crate::error::Result;

/// Mission types available in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionType {
    DeleteSoftware = 1,
    StealSoftware = 2,
    CheckBankStatus = 3,
    TransferMoney = 4,
    DestroyServer = 5,
    /// Special story missions (doom-related)
    ExploitMission = 50,
    LaunchDoom = 51,
    HackNsa = 52,
    ReceiveDoom = 53,
    ReceiveCracker = 54,
    /// Tutorial missions
    TutorialMission = 80,
}

/// Mission status values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionStatus {
    Available = 1,
    Accepted = 2,
    Completed = 3,
    Failed = 4,
}

/// Represents a mission in the game
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Mission {
    pub id: i32,
    pub mission_type: i32,
    pub status: i32,
    pub hirer: i64,          // IP address as long
    pub victim: i64,         // IP address as long  
    pub info: Option<String>,
    pub info2: Option<String>,
    pub new_info: Option<String>,
    pub new_info2: Option<String>,
    pub prize: i32,
    pub user_id: Option<i32>,
    pub level: i32,
    pub date_generated: DateTime<Utc>,
}

/// Mission with hirer and victim names for display
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MissionWithNames {
    pub id: i32,
    pub status: i32,
    pub info: Option<String>,
    pub info2: Option<String>,
    pub new_info: Option<String>,
    pub new_info2: Option<String>,
    pub mission_type: i32,
    pub hirer: i64,
    pub prize: i32,
    pub victim: i64,
    pub user_id: Option<i32>,
    pub level: i32,
    pub date_generated: DateTime<Utc>,
    pub hirer_name: String,
    pub victim_name: String,
}

/// Mission seed for text generation randomization
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MissionSeed {
    pub mission_id: i32,
    pub greeting: i32,
    pub intro: i32,
    pub victim_call: i32,
    pub payment: i32,
    pub victim_location: i32,
    pub warning: i32,
    pub action: i32,
}

/// Mission history record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MissionHistory {
    pub id: i32,
    pub mission_type: i32,
    pub hirer: i64,
    pub mission_end: DateTime<Utc>,
    pub prize: i32,
    pub user_id: i32,
    pub completed: bool,
}

/// Mission statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionStats {
    pub total: i32,
    pub completed: i32,
    pub aborted: i32,
    pub ratio: String,
    pub reward: i32,
}

#[async_trait]
pub trait MissionRepository {
    /// Check if mission exists
    async fn mission_exists(&self, mission_id: i32) -> Result<bool>;
    
    /// Get mission by ID
    async fn get_mission(&self, mission_id: i32) -> Result<Option<Mission>>;
    
    /// Get mission with hirer/victim names
    async fn get_mission_with_names(&self, mission_id: i32) -> Result<Option<MissionWithNames>>;
    
    /// Check mission status
    async fn get_mission_status(&self, mission_id: i32) -> Result<Option<i32>>;
    
    /// Get mission type
    async fn get_mission_type(&self, mission_id: i32) -> Result<Option<i32>>;
    
    /// Get mission info fields
    async fn get_mission_info(&self, mission_id: i32) -> Result<Option<String>>;
    async fn get_mission_info2(&self, mission_id: i32) -> Result<Option<String>>;
    async fn get_mission_new_info(&self, mission_id: i32) -> Result<Option<String>>;
    
    /// Get mission victim and hirer IPs
    async fn get_mission_victim(&self, mission_id: i32) -> Result<Option<i64>>;
    async fn get_mission_hirer(&self, mission_id: i32) -> Result<Option<i64>>;
    async fn get_mission_prize(&self, mission_id: i32) -> Result<Option<i32>>;
    
    /// Accept a mission
    async fn accept_mission(&self, user_id: i32, mission_id: i32) -> Result<()>;
    
    /// Complete a mission
    async fn complete_mission(&self, mission_id: i32) -> Result<()>;
    
    /// Abort a mission
    async fn abort_mission(&self, mission_id: i32) -> Result<()>;
    
    /// Finalize mission (mark as completed and pay reward)
    async fn finalize_mission(&self, mission_id: i32, bank_account: i32) -> Result<()>;
    
    /// Check if player is on a mission
    async fn player_on_mission(&self, user_id: i32) -> Result<bool>;
    
    /// Get player's current mission ID
    async fn get_player_mission_id(&self, user_id: i32) -> Result<Option<i32>>;
    
    /// Restore mission session after login
    async fn restore_mission_session(&self, user_id: i32) -> Result<Option<(i32, i32)>>;
    
    /// Get player mission level based on software
    async fn get_player_mission_level(&self, user_id: i32) -> Result<i32>;
    
    /// Get mission level
    async fn get_mission_level(&self, mission_id: i32) -> Result<Option<i32>>;
    
    /// List available missions for player
    async fn list_available_missions(&self, user_id: i32, level: i32) -> Result<Vec<MissionWithNames>>;
    
    /// Count available missions
    async fn count_available_missions(&self, level: i32) -> Result<i32>;
    
    /// Update mission info fields
    async fn update_mission_info(&self, mission_id: i32, new_info: &str) -> Result<()>;
    async fn update_mission_info2(&self, mission_id: i32, new_info2: &str) -> Result<()>;
    
    /// Mission seed management
    async fn seed_exists(&self, mission_id: i32) -> Result<bool>;
    async fn generate_seed(&self, mission_id: i32, mission_type: i32) -> Result<()>;
    async fn get_seed(&self, mission_id: i32) -> Result<Option<MissionSeed>>;
    
    /// Mission history
    async fn record_mission(&self, mission_id: i32) -> Result<()>;
    async fn get_mission_stats(&self, user_id: i32) -> Result<MissionStats>;
    async fn count_completed_missions(&self, user_id: i32) -> Result<i32>;
    
    /// Tutorial and special missions
    async fn passed_tutorial(&self, user_id: i32) -> Result<bool>;
    async fn create_tutorial_mission(&self, user_id: i32, mission_type: i32, hirer: i64, victim: Vec<i64>) -> Result<()>;
    async fn tutorial_update(&self, mission_id: i32, new_type: i32) -> Result<()>;
    
    /// Doom missions
    async fn has_doom_mission(&self, user_id: i32) -> Result<bool>;
    async fn create_doom_mission(&self, user_id: i32, mission_type: i32, info: Option<i32>) -> Result<()>;
    async fn delete_mission(&self, mission_id: i32) -> Result<()>;
}

pub struct MissionService {
    db: PgPool,
}

impl MissionService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
    
    /// Generate random seed values for mission text generation
    fn generate_random_seeds(mission_type: i32) -> [i32; 7] {
        let mut rng = rand::thread_rng();
        let mut seeds = [0; 7];
        
        // Default limits for different text parts
        let limits = match mission_type {
            1 | 2 => [3, 3, 4, 3, 3, 3, 3], // Delete/steal software
            3 => [3, 3, 2, 3, 2, 3, 0],     // Bank status check
            4 => [3, 3, 2, 3, 2, 3, 0],     // Money transfer
            5 => [3, 3, 2, 3, 3, 3, 0],     // DDoS
            _ => [3, 3, 4, 3, 3, 3, 0],     // Default
        };
        
        for (i, &limit) in limits.iter().enumerate() {
            if limit > 0 {
                seeds[i] = rng.gen_range(1..=limit);
            }
        }
        
        seeds
    }
    
    /// Get mission difficulty text
    fn get_mission_difficulty(mission_type: i32) -> &'static str {
        match mission_type {
            1 => "Very Easy",
            2 => "Easy", 
            3 => "Medium",
            4 => "Hard",
            5 => "Very Hard",
            _ => "Unknown",
        }
    }
    
    /// Get mission type description
    fn get_mission_text(mission_type: i32) -> &'static str {
        match mission_type {
            1 => "Delete software",
            2 => "Steal software", 
            3 => "Check bank status",
            4 => "Transfer money",
            5 => "Destroy server",
            50 | 51 => "Exploit",
            80..=84 => "Tutorial Mission",
            _ => "INVALID",
        }
    }
}

#[async_trait]
impl MissionRepository for MissionService {
    /// Check if mission exists
    async fn mission_exists(&self, mission_id: i32) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM missions WHERE id = $1",
            mission_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) > 0)
    }
    
    /// Get mission by ID
    async fn get_mission(&self, mission_id: i32) -> Result<Option<Mission>> {
        let mission = sqlx::query_as!(
            Mission,
            "SELECT id, mission_type, status, hirer, victim, info, info2, new_info, new_info2, 
                    prize, user_id, level, date_generated
             FROM missions 
             WHERE id = $1",
            mission_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(mission)
    }
    
    /// Get mission with hirer/victim names
    async fn get_mission_with_names(&self, mission_id: i32) -> Result<Option<MissionWithNames>> {
        let mission = sqlx::query_as!(
            MissionWithNames,
            "SELECT m.id, m.status, m.info, m.info2, m.new_info, m.new_info2, m.mission_type,
                    m.hirer, m.prize, m.victim, m.user_id, m.level, m.date_generated,
                    h_info.name as hirer_name, v_info.name as victim_name
             FROM missions m
             INNER JOIN npc npc_hirer ON npc_hirer.npc_ip = m.hirer
             INNER JOIN npc_info_en h_info ON h_info.npc_id = npc_hirer.id
             INNER JOIN npc npc_victim ON npc_victim.npc_ip = m.victim  
             INNER JOIN npc_info_en v_info ON v_info.npc_id = npc_victim.id
             WHERE m.id = $1",
            mission_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(mission)
    }
    
    /// Check mission status
    async fn get_mission_status(&self, mission_id: i32) -> Result<Option<i32>> {
        let status = sqlx::query_scalar!(
            "SELECT status FROM missions WHERE id = $1",
            mission_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(status)
    }
    
    /// Get mission type
    async fn get_mission_type(&self, mission_id: i32) -> Result<Option<i32>> {
        let mission_type = sqlx::query_scalar!(
            "SELECT mission_type FROM missions WHERE id = $1",
            mission_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(mission_type)
    }
    
    /// Get mission info fields
    async fn get_mission_info(&self, mission_id: i32) -> Result<Option<String>> {
        let info = sqlx::query_scalar!(
            "SELECT info FROM missions WHERE id = $1",
            mission_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(info.flatten())
    }
    
    async fn get_mission_info2(&self, mission_id: i32) -> Result<Option<String>> {
        let info2 = sqlx::query_scalar!(
            "SELECT info2 FROM missions WHERE id = $1", 
            mission_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(info2.flatten())
    }
    
    async fn get_mission_new_info(&self, mission_id: i32) -> Result<Option<String>> {
        let new_info = sqlx::query_scalar!(
            "SELECT new_info FROM missions WHERE id = $1",
            mission_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(new_info.flatten())
    }
    
    /// Get mission victim and hirer IPs
    async fn get_mission_victim(&self, mission_id: i32) -> Result<Option<i64>> {
        let victim = sqlx::query_scalar!(
            "SELECT victim FROM missions WHERE id = $1",
            mission_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(victim)
    }
    
    async fn get_mission_hirer(&self, mission_id: i32) -> Result<Option<i64>> {
        let hirer = sqlx::query_scalar!(
            "SELECT hirer FROM missions WHERE id = $1",
            mission_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(hirer)
    }
    
    async fn get_mission_prize(&self, mission_id: i32) -> Result<Option<i32>> {
        let prize = sqlx::query_scalar!(
            "SELECT prize FROM missions WHERE id = $1",
            mission_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(prize)
    }
    
    /// Accept a mission
    async fn accept_mission(&self, user_id: i32, mission_id: i32) -> Result<()> {
        sqlx::query!(
            "UPDATE missions SET status = $1, user_id = $2 WHERE id = $3",
            MissionStatus::Accepted as i32,
            user_id,
            mission_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Complete a mission
    async fn complete_mission(&self, mission_id: i32) -> Result<()> {
        sqlx::query!(
            "UPDATE missions SET status = $1 WHERE id = $2",
            MissionStatus::Completed as i32,
            mission_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Abort a mission
    async fn abort_mission(&self, mission_id: i32) -> Result<()> {
        sqlx::query!(
            "UPDATE missions SET status = $1 WHERE id = $2",
            MissionStatus::Failed as i32,
            mission_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Finalize mission (mark as completed and pay reward)
    async fn finalize_mission(&self, mission_id: i32, bank_account: i32) -> Result<()> {
        // Record the mission in history
        self.record_mission(mission_id).await?;
        
        // Delete the mission
        self.delete_mission(mission_id).await?;
        
        // Delete mission seed
        sqlx::query!(
            "DELETE FROM missions_seed WHERE mission_id = $1",
            mission_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Check if player is on a mission
    async fn player_on_mission(&self, user_id: i32) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM missions WHERE user_id = $1 AND (status = $2 OR status = $3)",
            user_id,
            MissionStatus::Accepted as i32,
            MissionStatus::Completed as i32
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) > 0)
    }
    
    /// Get player's current mission ID
    async fn get_player_mission_id(&self, user_id: i32) -> Result<Option<i32>> {
        let mission_id = sqlx::query_scalar!(
            "SELECT id FROM missions WHERE user_id = $1 AND (status = $2 OR status = $3) LIMIT 1",
            user_id,
            MissionStatus::Accepted as i32,
            MissionStatus::Completed as i32
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(mission_id)
    }
    
    /// Restore mission session after login
    async fn restore_mission_session(&self, user_id: i32) -> Result<Option<(i32, i32)>> {
        let mission = sqlx::query!(
            "SELECT id, mission_type FROM missions WHERE user_id = $1 AND (status = $2 OR status = $3)",
            user_id,
            MissionStatus::Accepted as i32,
            MissionStatus::Completed as i32
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(mission.map(|m| (m.id, m.mission_type)))
    }
    
    /// Get player mission level based on software
    async fn get_player_mission_level(&self, user_id: i32) -> Result<i32> {
        // Check for best cracker software to determine mission level
        let best_cracker = sqlx::query_scalar!(
            "SELECT MAX(soft_version) FROM software 
             WHERE user_id = $1 AND soft_type = 1 AND is_npc = false",
            user_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        match best_cracker.flatten() {
            Some(version) if version >= 60 => Ok(3),
            Some(version) if version >= 30 => Ok(2),
            Some(_) => Ok(1),
            None => Ok(1),
        }
    }
    
    /// Get mission level
    async fn get_mission_level(&self, mission_id: i32) -> Result<Option<i32>> {
        let level = sqlx::query_scalar!(
            "SELECT level FROM missions WHERE id = $1",
            mission_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(level)
    }
    
    /// List available missions for player
    async fn list_available_missions(&self, user_id: i32, level: i32) -> Result<Vec<MissionWithNames>> {
        let missions = sqlx::query_as!(
            MissionWithNames,
            "SELECT m.id, m.status, m.info, m.info2, m.new_info, m.new_info2, m.mission_type,
                    m.hirer, m.prize, m.victim, m.user_id, m.level, m.date_generated,
                    h_info.name as hirer_name, v_info.name as victim_name
             FROM missions m
             INNER JOIN npc npc_hirer ON npc_hirer.npc_ip = m.hirer
             INNER JOIN npc_info_en h_info ON h_info.npc_id = npc_hirer.id
             INNER JOIN npc npc_victim ON npc_victim.npc_ip = m.victim
             INNER JOIN npc_info_en v_info ON v_info.npc_id = npc_victim.id
             WHERE m.status = $1 AND m.level = $2
             ORDER BY m.prize DESC",
            MissionStatus::Available as i32,
            level
        )
        .fetch_all(&self.db)
        .await?;
        
        Ok(missions)
    }
    
    /// Count available missions
    async fn count_available_missions(&self, level: i32) -> Result<i32> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM missions WHERE status = $1 AND level = $2",
            MissionStatus::Available as i32,
            level
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) as i32)
    }
    
    /// Update mission info fields
    async fn update_mission_info(&self, mission_id: i32, new_info: &str) -> Result<()> {
        sqlx::query!(
            "UPDATE missions SET new_info = $1 WHERE id = $2",
            new_info,
            mission_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    async fn update_mission_info2(&self, mission_id: i32, new_info2: &str) -> Result<()> {
        sqlx::query!(
            "UPDATE missions SET info2 = $1 WHERE id = $2",
            new_info2,
            mission_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Mission seed management
    async fn seed_exists(&self, mission_id: i32) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM missions_seed WHERE mission_id = $1",
            mission_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) > 0)
    }
    
    async fn generate_seed(&self, mission_id: i32, mission_type: i32) -> Result<()> {
        let seeds = Self::generate_random_seeds(mission_type);
        
        sqlx::query!(
            "INSERT INTO missions_seed (mission_id, greeting, intro, victim_call, payment, victim_location, warning, action)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            mission_id,
            seeds[0],
            seeds[1], 
            seeds[2],
            seeds[3],
            seeds[4],
            seeds[5],
            seeds[6]
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    async fn get_seed(&self, mission_id: i32) -> Result<Option<MissionSeed>> {
        let seed = sqlx::query_as!(
            MissionSeed,
            "SELECT mission_id, greeting, intro, victim_call, payment, victim_location, warning, action
             FROM missions_seed WHERE mission_id = $1",
            mission_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(seed)
    }
    
    /// Mission history
    async fn record_mission(&self, mission_id: i32) -> Result<()> {
        let mission = sqlx::query!(
            "SELECT mission_type, hirer, prize, user_id FROM missions WHERE id = $1",
            mission_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        if let Some(m) = mission {
            sqlx::query!(
                "INSERT INTO missions_history (id, mission_type, hirer, mission_end, prize, user_id, completed)
                 VALUES ($1, $2, $3, NOW(), $4, $5, true)",
                mission_id,
                m.mission_type,
                m.hirer,
                m.prize,
                m.user_id
            )
            .execute(&self.db)
            .await?;
            
            // Update user stats
            if let Some(user_id) = m.user_id {
                sqlx::query!(
                    "UPDATE users_stats SET mission_count = mission_count + 1 WHERE uid = $1",
                    user_id
                )
                .execute(&self.db)
                .await?;
            }
        }
        
        Ok(())
    }
    
    async fn get_mission_stats(&self, user_id: i32) -> Result<MissionStats> {
        let stats = sqlx::query!(
            "SELECT COUNT(*) as total, 
                    SUM(CASE WHEN completed THEN 1 ELSE 0 END) as completed,
                    SUM(CASE WHEN completed THEN prize ELSE 0 END) as reward
             FROM missions_history WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        match stats {
            Some(s) => {
                let total = s.total.unwrap_or(0) as i32;
                let completed = s.completed.unwrap_or(0) as i32; 
                let aborted = total - completed;
                let reward = s.reward.unwrap_or(0) as i32;
                
                let ratio = if total > 0 {
                    format!("{}%", (completed as f32 / total as f32 * 100.0).round() as i32)
                } else {
                    "".to_string()
                };
                
                Ok(MissionStats {
                    total,
                    completed,
                    aborted,
                    ratio,
                    reward,
                })
            }
            None => Ok(MissionStats {
                total: 0,
                completed: 0,
                aborted: 0,
                ratio: "".to_string(),
                reward: 0,
            })
        }
    }
    
    async fn count_completed_missions(&self, user_id: i32) -> Result<i32> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM missions_history WHERE user_id = $1 AND completed = true",
            user_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) as i32)
    }
    
    /// Tutorial and special missions
    async fn passed_tutorial(&self, user_id: i32) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM missions_history WHERE user_id = $1 AND mission_type = 84",
            user_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) > 0)
    }
    
    async fn create_tutorial_mission(&self, user_id: i32, mission_type: i32, hirer: i64, victim: Vec<i64>) -> Result<()> {
        let victim_ip = victim.get(0).copied().unwrap_or(0);
        let victim_info = victim.get(1).map(|v| v.to_string()).unwrap_or_default();
        
        sqlx::query!(
            "INSERT INTO missions (mission_type, status, hirer, victim, info, new_info, prize, user_id)
             VALUES ($1, $2, $3, $4, $5, '', 500, $6)",
            mission_type,
            MissionStatus::Accepted as i32,
            hirer,
            victim_ip,
            victim_info,
            user_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    async fn tutorial_update(&self, mission_id: i32, new_type: i32) -> Result<()> {
        sqlx::query!(
            "UPDATE missions SET mission_type = $1 WHERE id = $2",
            new_type,
            mission_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Doom missions
    async fn has_doom_mission(&self, user_id: i32) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM missions 
             WHERE (mission_type BETWEEN 50 AND 54) AND user_id = $1",
            user_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) > 0)
    }
    
    async fn create_doom_mission(&self, user_id: i32, mission_type: i32, info: Option<i32>) -> Result<()> {
        // These would need storyline service to get proper IPs
        let hirer = 0; // evilcorp IP
        let victim = 0; // NSA IP  
        let prize = 1000000000;
        
        sqlx::query!(
            "INSERT INTO missions (mission_type, status, hirer, victim, info, new_info, prize, user_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            mission_type,
            MissionStatus::Accepted as i32,
            hirer,
            victim,
            info.map(|i| i.to_string()),
            info.map(|i| i.to_string()),
            prize,
            user_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    async fn delete_mission(&self, mission_id: i32) -> Result<()> {
        sqlx::query!(
            "DELETE FROM missions WHERE id = $1",
            mission_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
}