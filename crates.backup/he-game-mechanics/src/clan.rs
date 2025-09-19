//! Clan system mechanics - reputation formulas, warfare mechanics, contribution tracking

use crate::config::ClanConfig;
use crate::formulas::Formulas;
use crate::types::*;
use crate::{GameMechanicsError, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ClanMechanics {
    config: Arc<ClanConfig>,
}

impl ClanMechanics {
    pub fn new(config: Arc<crate::config::GameConfig>) -> Result<Self> {
        Ok(Self {
            config: Arc::new(config.clan.clone()),
        })
    }

    pub fn validate(&self) -> Result<()> {
        self.config.validate().map_err(|e| GameMechanicsError::Configuration(e))
    }

    /// Calculate clan reputation based on member activities
    pub fn calculate_clan_reputation(
        &self,
        base_reputation: i32,
        member_contributions: &[(u32, f64)], // (contribution, weight)
        war_victories: u32,
        war_defeats: u32,
        penalties: i32,
    ) -> Result<ClanReputation> {
        let reputation_value = Formulas::clan_reputation(
            base_reputation,
            member_contributions,
            war_victories,
            war_defeats,
            penalties,
        )?;
        
        let reputation_level = match reputation_value {
            0..=100 => ReputationLevel::Unknown,
            101..=300 => ReputationLevel::Emerging,
            301..=600 => ReputationLevel::Established,
            601..=1000 => ReputationLevel::Renowned,
            1001..=1500 => ReputationLevel::Elite,
            _ => ReputationLevel::Legendary,
        };
        
        Ok(ClanReputation {
            level: reputation_level,
            numeric_value: reputation_value,
            member_contribution_score: member_contributions.iter().map(|(c, w)| *c as f64 * w).sum::<f64>() as i32,
            war_performance_score: (war_victories as i32 * 10) - (war_defeats as i32 * 5),
            penalties_applied: penalties,
        })
    }

    /// Calculate warfare effectiveness
    pub fn calculate_warfare_effectiveness(
        &self,
        attacking_clan_power: u32,
        defending_clan_power: u32,
        strategy_bonus: f64,
        coordination_factor: f64,
    ) -> Result<WarfareResult> {
        let power_ratio = attacking_clan_power as f64 / defending_clan_power.max(1) as f64;
        let attack_bonus = strategy_bonus * coordination_factor * self.config.warfare_multiplier;
        
        let base_success_chance = 0.5 * power_ratio * attack_bonus;
        let success_probability = Probability::new(base_success_chance.min(0.95))?;
        
        let damage_dealt = if rand::random::<f64>() < success_probability.value() {
            Formulas::combat_damage(
                100, // Base damage
                (attacking_clan_power / 10) as u8,
                (defending_clan_power / 10) as u8,
                attack_bonus,
                1.0, // Defender equipment (neutral)
                rand::random::<f64>() * 0.4 + 0.8, // 80-120% variance
            )? as f64
        } else {
            0.0
        };
        
        Ok(WarfareResult {
            success: damage_dealt > 0.0,
            damage_dealt,
            success_probability,
            power_ratio,
            bonuses_applied: attack_bonus,
        })
    }

    /// Calculate member contribution value
    pub fn calculate_member_contribution(
        &self,
        activities: &[ClanActivity],
        member_rank: ClanRank,
        time_in_clan: chrono::Duration,
    ) -> Result<MemberContribution> {
        let mut total_contribution = 0.0;
        
        for activity in activities {
            let base_value = match activity.activity_type {
                ClanActivityType::Recruitment => 50.0,
                ClanActivityType::Training => 30.0,
                ClanActivityType::Warfare => 100.0,
                ClanActivityType::Research => 75.0,
                ClanActivityType::Defense => 80.0,
            };
            
            let quality_multiplier = activity.quality_score;
            let contribution_value = base_value * quality_multiplier * self.config.contribution_weight;
            total_contribution += contribution_value;
        }
        
        // Rank multiplier
        let rank_multiplier = match member_rank {
            ClanRank::Recruit => 1.0,
            ClanRank::Member => 1.2,
            ClanRank::Veteran => 1.5,
            ClanRank::Officer => 2.0,
            ClanRank::Leader => 2.5,
        };
        
        // Loyalty bonus based on time in clan
        let loyalty_bonus = (time_in_clan.num_days() as f64 / 365.0 * 0.1).min(0.5); // Max 50% bonus after 5 years
        
        let final_contribution = total_contribution * rank_multiplier * (1.0 + loyalty_bonus);
        
        Ok(MemberContribution {
            total_value: final_contribution,
            activity_count: activities.len(),
            rank_multiplier,
            loyalty_bonus,
            recent_activity_score: activities.iter()
                .filter(|a| chrono::Utc::now().signed_duration_since(a.timestamp).num_days() < 30)
                .map(|a| a.quality_score)
                .sum::<f64>(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReputationLevel {
    Unknown,
    Emerging,
    Established,
    Renowned,
    Elite,
    Legendary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanReputation {
    pub level: ReputationLevel,
    pub numeric_value: i32,
    pub member_contribution_score: i32,
    pub war_performance_score: i32,
    pub penalties_applied: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarfareResult {
    pub success: bool,
    pub damage_dealt: f64,
    pub success_probability: Probability,
    pub power_ratio: f64,
    pub bonuses_applied: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClanRank {
    Recruit,
    Member,
    Veteran,
    Officer,
    Leader,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClanActivityType {
    Recruitment,
    Training,
    Warfare,
    Research,
    Defense,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanActivity {
    pub activity_type: ClanActivityType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub quality_score: f64,
    pub participants: Vec<PlayerId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberContribution {
    pub total_value: f64,
    pub activity_count: usize,
    pub rank_multiplier: f64,
    pub loyalty_bonus: f64,
    pub recent_activity_score: f64,
}