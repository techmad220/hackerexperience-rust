use anyhow::{anyhow, Result};
//! Generate missions job
//! 
//! This job generates new missions for players to complete.
//! Equivalent to the legacy generateMissions.php cron job.

use crate::error::{CronError, CronResult};
use crate::utils::{rand_string_default, generate_bank_account, generate_short_bank_account};
use sqlx::{MySqlPool, Row};
use std::sync::Arc;
use tracing::{info, warn, error};
use rand::Rng;

/// Generate missions job implementation
pub struct GenerateMissionsJob;

#[derive(Debug)]
struct NpcInfo {
    id: i32,
    ip: String,
}

impl GenerateMissionsJob {
    /// Execute the generate missions job
    pub async fn execute(db_pool: Arc<MySqlPool>) -> CronResult<()> {
        info!("Starting mission generation");
        
        // Check if round is active
        if !Self::is_round_active(&db_pool).await? {
            info!("Round is not active, skipping mission generation");
            return Ok(());
        }
        
        // Clear completed missions
        Self::clear_completed_missions(&db_pool).await?;
        
        // Generate missions for each level
        Self::generate_missions_for_level(&db_pool, 1, 50).await?;
        Self::generate_missions_for_level(&db_pool, 2, 30).await?;
        Self::generate_missions_for_level(&db_pool, 3, 25).await?;
        
        info!("Mission generation completed");
        Ok(())
    }
    
    /// Check if the current round is active
    async fn is_round_active(db_pool: &MySqlPool) -> CronResult<bool> {
        let status = sqlx::query(
            "SELECT status FROM round ORDER BY id DESC LIMIT 1"
        )
        .fetch_optional(db_pool)
        .await?;
        
        if let Some(row) = status {
            let status: i32 = row.try_get("status")?;
            Ok(status == 1)
        } else {
            Ok(false)
        }
    }
    
    /// Clear completed missions
    async fn clear_completed_missions(db_pool: &MySqlPool) -> CronResult<()> {
        sqlx::query("DELETE FROM missions WHERE status = 1")
            .execute(db_pool)
            .await?;
        
        info!("Cleared completed missions");
        Ok(())
    }
    
    /// Generate missions for a specific level
    async fn generate_missions_for_level(
        db_pool: &MySqlPool, 
        level: i32, 
        target_total: i32
    ) -> CronResult<()> {
        info!("Generating missions for level {}, target: {}", level, target_total);
        
        let (npc_type, multiplier) = match level {
            1 => (71, 1.0),
            2 => (72, 1.1),
            3 => (73, 1.2),
            _ => return Err(CronError::Runtime("Invalid mission level".to_string())),
        };
        
        // Get current mission count for this level
        let current_count = Self::get_current_mission_count(db_pool, level).await?;
        
        if current_count >= target_total {
            info!("Level {} already has enough missions ({}/{})", level, current_count, target_total);
            return Ok(());
        }
        
        // Get available NPCs for this level
        let npcs = Self::get_npcs_for_level(db_pool, npc_type).await?;
        if npcs.is_empty() {
            return Err(CronError::Runtime(format!("No NPCs found for level {}", level)));
        }
        
        let missions_to_generate = target_total - current_count;
        info!("Generating {} missions for level {}", missions_to_generate, level);
        
        for i in 0..missions_to_generate {
            if let Err(e) = Self::generate_single_mission(db_pool, level, &npcs, multiplier, i).await {
                error!("Failed to generate mission {} for level {}: {}", i, level, e);
            }
        }
        
        info!("Generated missions for level {}", level);
        Ok(())
    }
    
    /// Get current mission count for a level
    async fn get_current_mission_count(db_pool: &MySqlPool, level: i32) -> CronResult<i32> {
        let count = sqlx::query(
            "SELECT COUNT(*) as count FROM missions WHERE status = 1 AND level = ?"
        )
        .bind(level)
        .fetch_one(db_pool)
        .await?;
        
        Ok(count.try_get::<i64, _>("count")? as i32)
    }
    
    /// Get NPCs available for a mission level
    async fn get_npcs_for_level(db_pool: &MySqlPool, npc_type: i32) -> CronResult<Vec<NpcInfo>> {
        let rows = sqlx::query(
            "SELECT id, npcIP as ip FROM npc WHERE npcType = ?"
        )
        .bind(npc_type)
        .fetch_all(db_pool)
        .await?;
        
        let mut npcs = Vec::new();
        for row in rows {
            npcs.push(NpcInfo {
                id: row.try_get("id")?,
                ip: row.try_get("ip")?,
            });
        }
        
        Ok(npcs)
    }
    
    /// Generate a single mission
    async fn generate_single_mission(
        db_pool: &MySqlPool,
        level: i32,
        npcs: &[NpcInfo],
        multiplier: f64,
        iteration: i32,
    ) -> CronResult<()> {
        let mut rng = rand::thread_rng();
        let npc_count = npcs.len();
        
        // Select hirer (contractor)
        let hirer_idx = if iteration < npc_count as i32 {
            iteration as usize
        } else {
            rng.gen_range(0..npc_count)
        };
        let hirer = &npcs[hirer_idx];
        
        // Select victim (different from hirer)
        let victim_idx = loop {
            let idx = rng.gen_range(0..npc_count);
            if idx != hirer_idx {
                break idx;
            }
        };
        let victim = &npcs[victim_idx];
        
        // Determine mission type (1-5 with specific probabilities)
        let mission_type = Self::determine_mission_type()?;
        
        // Calculate base prize
        let mut prize = Self::calculate_base_prize(mission_type)?;
        
        // Apply level multiplier
        if level == 2 {
            prize = (prize as f64 * 1.25) as i32;
        } else if level == 3 {
            prize = (prize as f64 * 1.5) as i32;
        }
        
        prize = (prize as f64 * multiplier) as i32;
        
        // Generate mission-specific data
        let (info, new_info, info2, new_info2, final_victim) = 
            Self::generate_mission_data(db_pool, mission_type, level, victim, &npcs).await?;
        
        // Insert the mission
        sqlx::query(
            "INSERT INTO missions 
             (id, type, status, hirer, victim, info, newInfo, info2, newInfo2, prize, level) 
             VALUES (NULL, ?, 1, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(mission_type)
        .bind(&hirer.ip)
        .bind(&final_victim)
        .bind(&info)
        .bind(&new_info)
        .bind(&info2)
        .bind(&new_info2)
        .bind(prize)
        .bind(level)
        .execute(db_pool)
        .await?;
        
        Ok(())
    }
    
    /// Determine mission type based on probabilities
    fn determine_mission_type() -> CronResult<i32> {
        let mut rng = rand::thread_rng();
        let roll = rng.gen_range(1..=100);
        
        Ok(match roll {
            1..=30 => 1,    // 30% - Steal software
            31..=60 => 2,   // 30% - Delete software
            61..=80 => 3,   // 20% - Steal money
            81..=90 => 4,   // 10% - Transfer money
            _ => 5,         // 10% - DDoS
        })
    }
    
    /// Calculate base prize for mission type
    fn calculate_base_prize(mission_type: i32) -> CronResult<i32> {
        let mut rng = rand::thread_rng();
        
        Ok(match mission_type {
            1 => rng.gen_range(150..=350),      // Steal software
            2 => rng.gen_range(250..=450),      // Delete software
            3 => rng.gen_range(500..=750),      // Steal money
            4 => rng.gen_range(1000..=1500),    // Transfer money
            5 => rng.gen_range(3000..=5000),    // DDoS
            _ => return Err(CronError::Runtime("Invalid mission type".to_string())),
        })
    }
    
    /// Generate mission-specific data based on type
    async fn generate_mission_data(
        db_pool: &MySqlPool,
        mission_type: i32,
        level: i32,
        victim: &NpcInfo,
        npcs: &[NpcInfo],
    ) -> CronResult<(String, String, String, String, String)> {
        let mut rng = rand::thread_rng();
        
        match mission_type {
            1 | 2 => {
                // Software-related missions - get software ID
                let software_id = Self::get_random_software_for_npc(db_pool, victim.id).await?;
                Ok((software_id.to_string(), String::new(), String::new(), String::new(), victim.ip.clone()))
            },
            3 | 4 => {
                // Money-related missions - create bank accounts
                let (bank_info, victim_ip) = Self::create_bank_mission_data(db_pool, level, npcs).await?;
                
                if mission_type == 4 {
                    // Transfer mission - create second bank account
                    let transfer_bank = generate_short_bank_account();
                    let transfer_npc_idx = rng.gen_range(0..npcs.len());
                    let transfer_npc = &npcs[transfer_npc_idx];
                    let cash = rng.gen_range(0..=2500);
                    
                    // Create transfer bank account
                    sqlx::query(
                        "INSERT INTO bankAccounts 
                         (id, bankAcc, bankPass, bankID, bankUser, cash, dateCreated) 
                         VALUES (NULL, ?, ?, ?, 0, ?, NOW())"
                    )
                    .bind(transfer_bank.to_string())
                    .bind(rand_string_default(6))
                    .bind(transfer_npc.id)
                    .bind(cash)
                    .execute(db_pool)
                    .await?;
                    
                    Ok((
                        bank_info.0, 
                        bank_info.1, 
                        transfer_bank.to_string(), 
                        transfer_npc.ip.clone(),
                        victim_ip
                    ))
                } else {
                    Ok((bank_info.0, bank_info.1, String::new(), String::new(), victim_ip))
                }
            },
            5 => {
                // DDoS mission - find suitable target
                let ddos_victim = Self::find_ddos_target(db_pool, &victim.ip).await?;
                Ok((String::new(), String::new(), String::new(), String::new(), ddos_victim))
            },
            _ => Err(CronError::Runtime("Invalid mission type".to_string())),
        }
    }
    
    /// Get random software for an NPC
    async fn get_random_software_for_npc(db_pool: &MySqlPool, npc_id: i32) -> CronResult<i32> {
        let software_list = sqlx::query(
            "SELECT id FROM software_original WHERE npcID = ? AND softType < 7"
        )
        .bind(npc_id)
        .fetch_all(db_pool)
        .await?;
        
        if software_list.is_empty() {
            return Err(CronError::Runtime("No software found for NPC".to_string()));
        }
        
        let mut rng = rand::thread_rng();
        let selected = &software_list[rng.gen_range(0..software_list.len())];
        Ok(selected.try_get("id")?)
    }
    
    /// Create bank account data for money missions
    async fn create_bank_mission_data(
        db_pool: &MySqlPool,
        level: i32,
        npcs: &[NpcInfo],
    ) -> CronResult<((String, String), String)> {
        let mut rng = rand::thread_rng();
        
        // Get bank NPCs based on level
        let bank_query = match level {
            1 => "SELECT npc.id, npc.npcIP FROM npc LEFT JOIN npc_key ON npc_key.npcID = npc.id WHERE npc_key.key = 'BANK/1' OR npc_key.key = 'BANK/2'",
            2 => "SELECT npc.id, npc.npcIP FROM npc LEFT JOIN npc_key ON npc_key.npcID = npc.id WHERE npc_key.key IN ('BANK/1', 'BANK/2', 'BANK/3', 'BANK/4')",
            _ => "SELECT npc.id, npc.npcIP FROM npc WHERE npcType = 1",
        };
        
        let bank_npcs = sqlx::query(bank_query)
            .fetch_all(db_pool)
            .await?;
        
        if bank_npcs.is_empty() {
            return Err(CronError::Runtime("No bank NPCs found".to_string()));
        }
        
        let selected_bank = &bank_npcs[rng.gen_range(0..bank_npcs.len())];
        let bank_id: i32 = selected_bank.try_get("id")?;
        let bank_ip: String = selected_bank.try_get("npcIP")?;
        
        // Generate unique bank account number
        let bank_acc = loop {
            let acc = generate_bank_account();
            let exists = sqlx::query(
                "SELECT id FROM bankAccounts WHERE bankAcc = ?"
            )
            .bind(acc.to_string())
            .fetch_optional(db_pool)
            .await?;
            
            if exists.is_none() {
                break acc;
            }
        };
        
        let cash = rng.gen_range(100..=1400);
        
        // Create bank account
        sqlx::query(
            "INSERT INTO bankAccounts 
             (id, bankAcc, bankPass, bankID, bankUser, cash, dateCreated) 
             VALUES (NULL, ?, ?, ?, 0, ?, NOW())"
        )
        .bind(bank_acc.to_string())
        .bind(rand_string_default(6))
        .bind(bank_id)
        .bind(cash)
        .execute(db_pool)
        .await?;
        
        Ok(((bank_acc.to_string(), cash.to_string()), bank_ip))
    }
    
    /// Find suitable DDoS target
    async fn find_ddos_target(db_pool: &MySqlPool, hirer_ip: &str) -> CronResult<String> {
        let target = sqlx::query(
            "SELECT npc.npcIP
             FROM hardware
             INNER JOIN npc ON hardware.userID = npc.id
             WHERE (hardware.cpu != 500 OR hardware.hdd != 1000 OR 
                    hardware.ram != 256 OR hardware.net != 1) AND
                   hardware.isNPC = 1 AND npc.npcType = 4 AND npc.npcIP != ?
             LIMIT 1"
        )
        .bind(hirer_ip)
        .fetch_optional(db_pool)
        .await?;
        
        if let Some(row) = target {
            Ok(row.try_get("npcIP")?)
        } else {
            // Fallback to hirer IP if no suitable target found
            Ok(hirer_ip.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mission_type_determination() {
        // Test that mission type is always between 1 and 5
        for _ in 0..100 {
            let mission_type = GenerateMissionsJob::determine_mission_type().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
            assert!(mission_type >= 1 && mission_type <= 5);
        }
    }
    
    #[test]
    fn test_base_prize_calculation() {
        for mission_type in 1..=5 {
            let prize = GenerateMissionsJob::calculate_base_prize(mission_type).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
            assert!(prize > 0);
            
            match mission_type {
                1 => assert!(prize >= 150 && prize <= 350),
                2 => assert!(prize >= 250 && prize <= 450),
                3 => assert!(prize >= 500 && prize <= 750),
                4 => assert!(prize >= 1000 && prize <= 1500),
                5 => assert!(prize >= 3000 && prize <= 5000),
                _ => panic!("Invalid mission type"),
            }
        }
    }
}