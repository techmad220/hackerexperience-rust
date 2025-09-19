use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use chrono::{DateTime, Utc};

use crate::error::Result;

/// Player ranking information
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlayerRanking {
    pub user_id: i32,
    pub login: String,
    pub reputation: i64,
    pub research_exp: i64,
    pub clan_id: Option<i32>,
    pub clan_name: Option<String>,
    pub position: i64,
    pub total_exp: i64,
    pub mail_sent: i32,
    pub warez_sent: f64,
    pub money_earned: i64,
    pub btc_earned: f64,
    pub mission_count: i32,
    pub virus_count: i32,
    pub hack_count: i32,
    pub research_count: i32,
    pub ddos_count: i32,
}

/// Research statistics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ResearchStats {
    pub total_points: i64,
    pub hack_points: i64,
    pub ddos_points: i64,
    pub mission_points: i64,
    pub research_points: i64,
    pub last_research: Option<DateTime<Utc>>,
}

/// Experience statistics for different activities
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExperienceStats {
    pub hack_exp: i64,
    pub ddos_exp: i64,
    pub mission_exp: i64,
    pub research_exp: i64,
    pub collect_exp: i64,
    pub total_exp: i64,
}

/// Ranking modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RankingMode {
    Reputation = 1,
    Research = 2,
}

/// Experience action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperienceAction {
    Hack { time: i32, logs_removed: i32, money: Option<i32> },
    Ddos { time: i32, servers: i32 },
    Mission { mission_type: i32, prize: i32 },
    Research { points: i32 },
    Collect { category: String, value: f64 },
}

#[async_trait]
pub trait RankingRepository {
    /// Get player ranking position
    async fn get_player_ranking(&self, user_id: i32, mode: RankingMode) -> Result<i64>;
    
    /// Get player's total experience
    async fn get_total_experience(&self, user_id: i32, mode: RankingMode) -> Result<i64>;
    
    /// Get detailed experience breakdown
    async fn get_experience_stats(&self, user_id: i32) -> Result<ExperienceStats>;
    
    /// Get top rankings with pagination
    async fn get_top_rankings(&self, mode: RankingMode, limit: i32, offset: i32) -> Result<Vec<PlayerRanking>>;
    
    /// Add experience for various actions
    async fn add_experience(&self, user_id: i32, action: ExperienceAction) -> Result<()>;
    
    /// Get research statistics
    async fn get_research_stats(&self, user_id: i32) -> Result<ResearchStats>;
    
    /// Update collect statistics
    async fn update_collect_stats(&self, warez_sent: f64, mail_sent: i32, money_earned: i64, btc_earned: f64) -> Result<()>;
    
    /// Get current collect/virus statistics
    async fn get_collect_stats(&self, user_id: i32) -> Result<(f64, i32, i64, f64)>;
    
    /// Reset collect statistics
    async fn reset_collect_stats(&self, user_id: i32) -> Result<()>;
    
    /// Calculate research level from experience points
    fn calculate_research_level(exp: i64) -> i32;
    
    /// Calculate reputation level from experience points
    fn calculate_reputation_level(exp: i64) -> i32;
    
    /// Get total players count
    async fn get_total_players(&self, mode: RankingMode) -> Result<i64>;
    
    /// Get player count in specific rank range
    async fn get_rank_range_count(&self, user_id: i32, mode: RankingMode) -> Result<i64>;
    
    /// Check if ranking needs update
    async fn needs_ranking_update(&self, user_id: i32) -> Result<bool>;
    
    /// Force ranking recalculation
    async fn recalculate_rankings(&self, mode: RankingMode) -> Result<()>;
}

pub struct RankingService {
    db: PgPool,
}

impl RankingService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
    
    /// Calculate experience gain for hacking
    fn calculate_hack_experience(time: i32, logs_removed: i32, money: Option<i32>) -> i64 {
        let base = (time as f64 * 1.5) as i64;
        let log_bonus = logs_removed as i64 * 10;
        let money_bonus = money.map(|m| m as i64 / 100).unwrap_or(0);
        
        base + log_bonus + money_bonus
    }
    
    /// Calculate experience gain for DDoS
    fn calculate_ddos_experience(time: i32, servers: i32) -> i64 {
        let base = (time as f64 * 2.0) as i64;
        let server_bonus = servers as i64 * 50;
        
        base + server_bonus
    }
    
    /// Calculate experience gain for missions
    fn calculate_mission_experience(mission_type: i32, prize: i32) -> i64 {
        let type_multiplier = match mission_type {
            1 => 1.0, // Delete software
            2 => 1.2, // Steal software
            3 => 0.8, // Check bank
            4 => 1.5, // Transfer money
            5 => 2.0, // DDoS
            _ => 1.0,
        };
        
        ((prize as f64 * type_multiplier) / 10.0) as i64
    }
    
    /// Calculate research points gain
    fn calculate_research_experience(points: i32) -> i64 {
        points as i64 * 100
    }
}

#[async_trait]
impl RankingRepository for RankingService {
    /// Get player ranking position
    async fn get_player_ranking(&self, user_id: i32, mode: RankingMode) -> Result<i64> {
        let field = match mode {
            RankingMode::Reputation => "reputation",
            RankingMode::Research => "research_exp",
        };
        
        let query = format!(
            "SELECT COUNT(*) + 1 as position
             FROM users_stats us1
             INNER JOIN users u ON u.id = us1.uid
             WHERE us1.{} > (
                 SELECT {}
                 FROM users_stats us2
                 WHERE us2.uid = $1
             ) AND u.isNPC = false",
            field, field
        );
        
        let position = sqlx::query_scalar(&query)
            .bind(user_id)
            .fetch_one(&self.db)
            .await?;
        
        Ok(position.unwrap_or(1))
    }
    
    /// Get player's total experience
    async fn get_total_experience(&self, user_id: i32, mode: RankingMode) -> Result<i64> {
        let exp = match mode {
            RankingMode::Reputation => {
                sqlx::query_scalar!(
                    "SELECT reputation FROM users_stats WHERE uid = $1",
                    user_id
                )
                .fetch_one(&self.db)
                .await?
            }
            RankingMode::Research => {
                sqlx::query_scalar!(
                    "SELECT research_exp FROM users_stats WHERE uid = $1", 
                    user_id
                )
                .fetch_one(&self.db)
                .await?
            }
        };
        
        Ok(exp.unwrap_or(0))
    }
    
    /// Get detailed experience breakdown
    async fn get_experience_stats(&self, user_id: i32) -> Result<ExperienceStats> {
        let stats = sqlx::query_as!(
            ExperienceStats,
            "SELECT hack_exp, ddos_exp, mission_exp, research_exp, collect_exp,
                    (hack_exp + ddos_exp + mission_exp + research_exp + collect_exp) as total_exp
             FROM users_stats WHERE uid = $1",
            user_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(stats.unwrap_or(ExperienceStats {
            hack_exp: 0,
            ddos_exp: 0,
            mission_exp: 0,
            research_exp: 0,
            collect_exp: 0,
            total_exp: 0,
        }))
    }
    
    /// Get top rankings with pagination
    async fn get_top_rankings(&self, mode: RankingMode, limit: i32, offset: i32) -> Result<Vec<PlayerRanking>> {
        let order_field = match mode {
            RankingMode::Reputation => "us.reputation",
            RankingMode::Research => "us.research_exp",
        };
        
        let query = format!(
            "SELECT u.id as user_id, u.login, us.reputation, us.research_exp,
                    cm.clan_id, c.name as clan_name,
                    ROW_NUMBER() OVER (ORDER BY {} DESC) as position,
                    (us.hack_exp + us.ddos_exp + us.mission_exp + us.research_exp + us.collect_exp) as total_exp,
                    us.mail_sent, us.warez_sent, us.money_earned, us.btc_earned,
                    us.mission_count, us.virus_count, us.hack_count, us.research_count, us.ddos_count
             FROM users u
             INNER JOIN users_stats us ON u.id = us.uid
             LEFT JOIN clan_members cm ON u.id = cm.user_id
             LEFT JOIN clans c ON cm.clan_id = c.id
             WHERE u.isNPC = false
             ORDER BY {} DESC
             LIMIT $1 OFFSET $2",
            order_field, order_field
        );
        
        let rankings = sqlx::query_as(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?;
        
        Ok(rankings)
    }
    
    /// Add experience for various actions
    async fn add_experience(&self, user_id: i32, action: ExperienceAction) -> Result<()> {
        match action {
            ExperienceAction::Hack { time, logs_removed, money } => {
                let exp = Self::calculate_hack_experience(time, logs_removed, money);
                
                sqlx::query!(
                    "UPDATE users_stats 
                     SET hack_exp = hack_exp + $2, reputation = reputation + $2, hack_count = hack_count + 1
                     WHERE uid = $1",
                    user_id,
                    exp
                )
                .execute(&self.db)
                .await?;
            }
            
            ExperienceAction::Ddos { time, servers } => {
                let exp = Self::calculate_ddos_experience(time, servers);
                
                sqlx::query!(
                    "UPDATE users_stats 
                     SET ddos_exp = ddos_exp + $2, reputation = reputation + $2, ddos_count = ddos_count + 1
                     WHERE uid = $1",
                    user_id,
                    exp
                )
                .execute(&self.db)
                .await?;
            }
            
            ExperienceAction::Mission { mission_type, prize } => {
                let exp = Self::calculate_mission_experience(mission_type, prize);
                
                sqlx::query!(
                    "UPDATE users_stats 
                     SET mission_exp = mission_exp + $2, reputation = reputation + $2, mission_count = mission_count + 1
                     WHERE uid = $1",
                    user_id,
                    exp
                )
                .execute(&self.db)
                .await?;
            }
            
            ExperienceAction::Research { points } => {
                let exp = Self::calculate_research_experience(points);
                
                sqlx::query!(
                    "UPDATE users_stats 
                     SET research_exp = research_exp + $2, research_count = research_count + 1, last_research = NOW()
                     WHERE uid = $1",
                    user_id,
                    exp
                )
                .execute(&self.db)
                .await?;
            }
            
            ExperienceAction::Collect { category, value } => {
                let exp = (value * 10.0) as i64;
                
                sqlx::query!(
                    "UPDATE users_stats 
                     SET collect_exp = collect_exp + $2, reputation = reputation + $2
                     WHERE uid = $1",
                    user_id,
                    exp
                )
                .execute(&self.db)
                .await?;
                
                // Update specific collect stats
                match category.as_str() {
                    "cash" => {
                        sqlx::query!(
                            "UPDATE users_stats SET money_earned = money_earned + $2 WHERE uid = $1",
                            user_id,
                            value as i64
                        )
                        .execute(&self.db)
                        .await?;
                    }
                    "btc" => {
                        sqlx::query!(
                            "UPDATE users_stats SET btc_earned = btc_earned + $2 WHERE uid = $1",
                            user_id,
                            value
                        )
                        .execute(&self.db)
                        .await?;
                    }
                    _ => {}
                }
            }
        }
        
        Ok(())
    }
    
    /// Get research statistics
    async fn get_research_stats(&self, user_id: i32) -> Result<ResearchStats> {
        let stats = sqlx::query_as!(
            ResearchStats,
            "SELECT 
                (hack_exp + ddos_exp + mission_exp + research_exp + collect_exp) as total_points,
                hack_exp as hack_points,
                ddos_exp as ddos_points, 
                mission_exp as mission_points,
                research_exp as research_points,
                last_research
             FROM users_stats 
             WHERE uid = $1",
            user_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(stats.unwrap_or(ResearchStats {
            total_points: 0,
            hack_points: 0,
            ddos_points: 0,
            mission_points: 0,
            research_points: 0,
            last_research: None,
        }))
    }
    
    /// Update collect statistics
    async fn update_collect_stats(&self, warez_sent: f64, mail_sent: i32, money_earned: i64, btc_earned: f64) -> Result<()> {
        // This would be called from session context with current user
        // Simplified implementation for global stats
        sqlx::query!(
            "UPDATE collect_stats 
             SET warez_sent = warez_sent + $1, mail_sent = mail_sent + $2, 
                 money_earned = money_earned + $3, btc_earned = btc_earned + $4",
            warez_sent,
            mail_sent,
            money_earned,
            btc_earned
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Get current collect/virus statistics
    async fn get_collect_stats(&self, user_id: i32) -> Result<(f64, i32, i64, f64)> {
        let stats = sqlx::query!(
            "SELECT warez_sent, mail_sent, money_earned, btc_earned
             FROM users_stats WHERE uid = $1",
            user_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        match stats {
            Some(s) => Ok((s.warez_sent, s.mail_sent, s.money_earned, s.btc_earned)),
            None => Ok((0.0, 0, 0, 0.0)),
        }
    }
    
    /// Reset collect statistics
    async fn reset_collect_stats(&self, user_id: i32) -> Result<()> {
        sqlx::query!(
            "UPDATE users_stats 
             SET warez_sent = 0, mail_sent = 0, money_earned = 0, btc_earned = 0
             WHERE uid = $1",
            user_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Calculate research level from experience points
    fn calculate_research_level(exp: i64) -> i32 {
        // Research levels are more linear
        match exp {
            0..=999 => 1,
            1000..=2999 => 2,
            3000..=5999 => 3,
            6000..=9999 => 4,
            10000..=14999 => 5,
            15000..=20999 => 6,
            21000..=27999 => 7,
            28000..=35999 => 8,
            36000..=44999 => 9,
            45000.. => 10,
        }
    }
    
    /// Calculate reputation level from experience points
    fn calculate_reputation_level(exp: i64) -> i32 {
        // Exponential scaling for reputation levels
        match exp {
            0..=99 => 1,
            100..=499 => 2,
            500..=1199 => 3,
            1200..=2399 => 4,
            2400..=4199 => 5,
            4200..=6799 => 6,
            6800..=10399 => 7,
            10400..=15199 => 8,
            15200..=21399 => 9,
            21400.. => 10,
        }
    }
    
    /// Get total players count
    async fn get_total_players(&self, mode: RankingMode) -> Result<i64> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE isNPC = false"
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0))
    }
    
    /// Get player count in specific rank range
    async fn get_rank_range_count(&self, user_id: i32, mode: RankingMode) -> Result<i64> {
        let user_rank = self.get_player_ranking(user_id, mode).await?;
        let start_range = ((user_rank - 1) / 50) * 50 + 1;
        let end_range = start_range + 49;
        
        let total = self.get_total_players(mode).await?;
        
        Ok(std::cmp::min(end_range, total) - start_range + 1)
    }
    
    /// Check if ranking needs update
    async fn needs_ranking_update(&self, user_id: i32) -> Result<bool> {
        let last_update = sqlx::query_scalar!(
            "SELECT last_ranking_update FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        match last_update.flatten() {
            Some(last) => {
                let now = chrono::Utc::now();
                let diff = now.signed_duration_since(last);
                Ok(diff.num_hours() >= 1) // Update every hour
            }
            None => Ok(true),
        }
    }
    
    /// Force ranking recalculation
    async fn recalculate_rankings(&self, mode: RankingMode) -> Result<()> {
        // This would be a heavy operation to recalculate all rankings
        // Simplified implementation
        sqlx::query!(
            "UPDATE users SET last_ranking_update = NOW() WHERE isNPC = false"
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
}