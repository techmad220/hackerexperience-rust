//! DEFCON job for detecting and managing clan wars
//! 
//! This job detects if clan wars should start based on attack patterns.
//! Equivalent to the legacy defcon.php and defcon2.php cron jobs.

use crate::error::{CronError, CronResult};
use sqlx::{MySqlPool, Row};
use std::sync::Arc;
use std::collections::HashMap;
use tracing::{info, warn, error};

/// DEFCON job implementation
pub struct DefconJob;

#[derive(Debug, Clone)]
struct AttackInfo {
    id: i32,
    attacker_id: i32,
    attacker_clan_id: i32,
    victim_id: i32,
    victim_clan_id: i32,
    attack_date: chrono::NaiveDateTime,
    clan_server: i32,
}

#[derive(Debug)]
struct WarCandidate {
    clan_id1: i32,
    clan_id2: i32,
}

impl DefconJob {
    /// Execute the DEFCON job
    pub async fn execute(db_pool: Arc<MySqlPool>) -> CronResult<()> {
        info!("Starting DEFCON analysis");
        
        // Clean up old attack records (older than 3 days)
        Self::cleanup_old_attacks(&db_pool).await?;
        
        // Get all current attacks
        let attacks = Self::get_current_attacks(&db_pool).await?;
        
        if attacks.is_empty() {
            info!("No attacks found for DEFCON analysis");
            return Ok(());
        }
        
        info!("Analyzing {} attacks for potential wars", attacks.len());
        
        // Analyze attacks for war patterns
        let war_candidates = Self::analyze_war_patterns(&attacks)?;
        
        // Process each war candidate
        for candidate in war_candidates {
            Self::process_war_candidate(&db_pool, &candidate).await?;
        }
        
        info!("DEFCON analysis completed");
        Ok(())
    }
    
    /// Clean up attack records older than 3 days
    async fn cleanup_old_attacks(db_pool: &MySqlPool) -> CronResult<()> {
        let deleted = sqlx::query(
            "DELETE FROM clan_defcon WHERE TIMESTAMPDIFF(DAY, attackDate, NOW()) >= 3"
        )
        .execute(db_pool)
        .await?
        .rows_affected();
        
        if deleted > 0 {
            info!("Cleaned up {} old attack records", deleted);
        }
        
        Ok(())
    }
    
    /// Get all current attack records
    async fn get_current_attacks(db_pool: &MySqlPool) -> CronResult<Vec<AttackInfo>> {
        let rows = sqlx::query(
            "SELECT id, attackerID, attackerClanID, victimID, victimClanID, attackDate, clanServer
             FROM clan_defcon ORDER BY attackDate ASC"
        )
        .fetch_all(db_pool)
        .await?;
        
        let mut attacks = Vec::new();
        for row in rows {
            attacks.push(AttackInfo {
                id: row.try_get("id")?,
                attacker_id: row.try_get("attackerID")?,
                attacker_clan_id: row.try_get("attackerClanID")?,
                victim_id: row.try_get("victimID")?,
                victim_clan_id: row.try_get("victimClanID")?,
                attack_date: row.try_get("attackDate")?,
                clan_server: row.try_get("clanServer")?,
            });
        }
        
        Ok(attacks)
    }
    
    /// Analyze attack patterns to detect potential wars
    fn analyze_war_patterns(attacks: &[AttackInfo]) -> CronResult<Vec<WarCandidate>> {
        let mut clan_interactions: HashMap<(i32, i32), Vec<&AttackInfo>> = HashMap::new();
        
        // Group attacks by clan pairs
        for attack in attacks {
            let key = if attack.attacker_clan_id < attack.victim_clan_id {
                (attack.attacker_clan_id, attack.victim_clan_id)
            } else {
                (attack.victim_clan_id, attack.attacker_clan_id)
            };
            
            clan_interactions.entry(key).or_insert_with(Vec::new).push(attack);
        }
        
        let mut war_candidates = Vec::new();
        
        // Analyze each clan pair
        for ((clan1, clan2), clan_attacks) in clan_interactions {
            if Self::should_start_war(clan1, clan2, &clan_attacks)? {
                war_candidates.push(WarCandidate {
                    clan_id1: clan1,
                    clan_id2: clan2,
                });
            }
        }
        
        Ok(war_candidates)
    }
    
    /// Determine if a war should start between two clans
    fn should_start_war(clan1: i32, clan2: i32, attacks: &[&AttackInfo]) -> CronResult<bool> {
        let mut clan1_attacks = Vec::new();
        let mut clan2_attacks = Vec::new();
        
        // Separate attacks by attacking clan
        for attack in attacks {
            if attack.attacker_clan_id == clan1 {
                clan1_attacks.push(*attack);
            } else if attack.attacker_clan_id == clan2 {
                clan2_attacks.push(*attack);
            }
        }
        
        // Check various war conditions
        
        // Condition 1: Member of clan1 attacked at least 2 members of clan2 
        // AND at least 1 member of clan2 countered
        let clan1_victim_count = Self::count_unique_victims(&clan1_attacks);
        let clan2_retaliation = !clan2_attacks.is_empty();
        
        if clan1_victim_count >= 2 && clan2_retaliation {
            info!("War condition met: {} attacked {} members of {}, {} retaliated", 
                  clan1, clan1_victim_count, clan2, clan2);
            return Ok(true);
        }
        
        // Condition 2: 2+ members of clan1 attacked 1+ member of clan2 
        // AND at least 1 member of clan2 countered
        let clan1_attacker_count = Self::count_unique_attackers(&clan1_attacks);
        if clan1_attacker_count >= 2 && clan2_retaliation {
            info!("War condition met: {} attackers from {} vs {}, {} retaliated", 
                  clan1_attacker_count, clan1, clan2, clan2);
            return Ok(true);
        }
        
        // Condition 3: Server attacks
        let clan1_server_attacks = clan1_attacks.iter().any(|a| a.clan_server == 1);
        if clan1_server_attacks && clan2_retaliation {
            info!("War condition met: {} attacked {}'s server, {} retaliated", 
                  clan1, clan2, clan2);
            return Ok(true);
        }
        
        // Same conditions but with clans reversed
        let clan2_victim_count = Self::count_unique_victims(&clan2_attacks);
        let clan1_retaliation = !clan1_attacks.is_empty();
        
        if clan2_victim_count >= 2 && clan1_retaliation {
            info!("War condition met: {} attacked {} members of {}, {} retaliated", 
                  clan2, clan2_victim_count, clan1, clan1);
            return Ok(true);
        }
        
        let clan2_attacker_count = Self::count_unique_attackers(&clan2_attacks);
        if clan2_attacker_count >= 2 && clan1_retaliation {
            info!("War condition met: {} attackers from {} vs {}, {} retaliated", 
                  clan2_attacker_count, clan2, clan1, clan1);
            return Ok(true);
        }
        
        let clan2_server_attacks = clan2_attacks.iter().any(|a| a.clan_server == 1);
        if clan2_server_attacks && clan1_retaliation {
            info!("War condition met: {} attacked {}'s server, {} retaliated", 
                  clan2, clan1, clan1);
            return Ok(true);
        }
        
        Ok(false)
    }
    
    /// Count unique victims in attacks
    fn count_unique_victims(attacks: &[&AttackInfo]) -> usize {
        let mut victims = std::collections::HashSet::new();
        for attack in attacks {
            victims.insert(attack.victim_id);
        }
        victims.len()
    }
    
    /// Count unique attackers in attacks
    fn count_unique_attackers(attacks: &[&AttackInfo]) -> usize {
        let mut attackers = std::collections::HashSet::new();
        for attack in attacks {
            attackers.insert(attack.attacker_id);
        }
        attackers.len()
    }
    
    /// Process a war candidate and potentially start a war
    async fn process_war_candidate(db_pool: &MySqlPool, candidate: &WarCandidate) -> CronResult<()> {
        info!("Processing war candidate: {} vs {}", candidate.clan_id1, candidate.clan_id2);
        
        // Check if war already exists
        let existing_war = sqlx::query(
            "SELECT endDate FROM clan_war 
             WHERE (clanID1 = ? AND clanID2 = ?) OR (clanID1 = ? AND clanID2 = ?) 
             LIMIT 1"
        )
        .bind(candidate.clan_id1)
        .bind(candidate.clan_id2)
        .bind(candidate.clan_id2)
        .bind(candidate.clan_id1)
        .fetch_optional(db_pool)
        .await?;
        
        if let Some(_) = existing_war {
            // War already exists, extend it by 1 day
            sqlx::query(
                "UPDATE clan_war 
                 SET endDate = DATE_ADD(endDate, INTERVAL 1 DAY) 
                 WHERE (clanID1 = ? AND clanID2 = ?) OR (clanID1 = ? AND clanID2 = ?) 
                 LIMIT 1"
            )
            .bind(candidate.clan_id1)
            .bind(candidate.clan_id2)
            .bind(candidate.clan_id2)
            .bind(candidate.clan_id1)
            .execute(db_pool)
            .await?;
            
            info!("Extended existing war between {} and {}", candidate.clan_id1, candidate.clan_id2);
        } else {
            // Create new war
            Self::create_new_war(db_pool, candidate).await?;
        }
        
        Ok(())
    }
    
    /// Create a new clan war
    async fn create_new_war(db_pool: &MySqlPool, candidate: &WarCandidate) -> CronResult<()> {
        // Calculate initial scores based on existing DDoS attacks
        let (score1, score2) = Self::calculate_initial_scores(db_pool, candidate).await?;
        
        let duration_days = 2; // War duration in days
        
        sqlx::query(
            "INSERT INTO clan_war (clanID1, clanID2, startDate, endDate, score1, score2, bounty)
             VALUES (?, ?, NOW(), DATE_ADD(NOW(), INTERVAL ? DAY), ?, ?, ?)"
        )
        .bind(candidate.clan_id1)
        .bind(candidate.clan_id2)
        .bind(duration_days)
        .bind(score1)
        .bind(score2)
        .bind(Self::calculate_bounty(score1, score2))
        .execute(db_pool)
        .await?;
        
        info!("Created new war between {} and {} (scores: {}-{})", 
              candidate.clan_id1, candidate.clan_id2, score1, score2);
        
        Ok(())
    }
    
    /// Calculate initial war scores based on DDoS attacks
    async fn calculate_initial_scores(
        db_pool: &MySqlPool, 
        candidate: &WarCandidate
    ) -> CronResult<(i64, i64)> {
        let ddos_data = sqlx::query(
            "SELECT clan_ddos.attackerClan, round_ddos.power
             FROM clan_ddos
             INNER JOIN round_ddos ON round_ddos.id = clan_ddos.ddosID
             WHERE (attackerClan = ? AND victimClan = ?) OR (attackerClan = ? AND victimClan = ?)"
        )
        .bind(candidate.clan_id1)
        .bind(candidate.clan_id2)
        .bind(candidate.clan_id2)
        .bind(candidate.clan_id1)
        .fetch_all(db_pool)
        .await?;
        
        let mut score1 = 0i64;
        let mut score2 = 0i64;
        
        for row in ddos_data {
            let attacker_clan: i32 = row.try_get("attackerClan")?;
            let power: i64 = row.try_get("power")?;
            
            if attacker_clan == candidate.clan_id1 {
                score1 += power;
            } else {
                score2 += power;
            }
        }
        
        Ok((score1, score2))
    }
    
    /// Calculate war bounty based on scores
    fn calculate_bounty(score1: i64, score2: i64) -> i64 {
        let total_score = score1 + score2;
        // Base bounty calculation - could be made configurable
        std::cmp::max(1000, total_score / 10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[test]
    fn test_war_candidate() {
        let candidate = WarCandidate {
            clan_id1: 1,
            clan_id2: 2,
        };
        
        assert_eq!(candidate.clan_id1, 1);
        assert_eq!(candidate.clan_id2, 2);
    }
    
    #[test]
    fn test_bounty_calculation() {
        assert_eq!(DefconJob::calculate_bounty(0, 0), 1000);
        assert_eq!(DefconJob::calculate_bounty(5000, 3000), 1000);
        assert_eq!(DefconJob::calculate_bounty(15000, 5000), 2000);
    }
}