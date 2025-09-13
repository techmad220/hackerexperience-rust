//! Mission system mechanics - difficulty scaling, reward calculations, prerequisite checking

use crate::config::MissionConfig;
use crate::types::*;
use crate::{GameMechanicsError, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MissionMechanics {
    config: Arc<MissionConfig>,
}

impl MissionMechanics {
    pub fn new(config: Arc<crate::config::GameConfig>) -> Result<Self> {
        Ok(Self {
            config: Arc::new(config.mission.clone()),
        })
    }

    pub fn validate(&self) -> Result<()> {
        self.config.validate().map_err(|e| GameMechanicsError::Configuration(e))
    }

    /// Calculate mission difficulty based on player level and mission type
    pub fn calculate_mission_difficulty(
        &self,
        base_difficulty: u32,
        player_level: u32,
        mission_type: MissionType,
    ) -> Result<MissionDifficulty> {
        let level_scaling = (player_level as f64 * self.config.difficulty_progression).max(1.0);
        let type_modifier = match mission_type {
            MissionType::Tutorial => 0.5,
            MissionType::Simple => 1.0,
            MissionType::Complex => 1.5,
            MissionType::Expert => 2.0,
            MissionType::Legendary => 3.0,
        };
        
        let final_difficulty = (base_difficulty as f64 * level_scaling * type_modifier) as u32;
        
        Ok(MissionDifficulty {
            numeric_value: final_difficulty,
            scaling_factors: DifficultyScaling {
                base_difficulty,
                level_scaling,
                type_modifier,
            },
        })
    }

    /// Calculate mission rewards
    pub fn calculate_mission_rewards(
        &self,
        difficulty: &MissionDifficulty,
        player_level: u32,
        bonus_multiplier: f64,
    ) -> Result<MissionRewards> {
        let base_exp = 100u32;
        let base_money = 500i64;
        
        let difficulty_multiplier = 1.0 + (difficulty.numeric_value as f64 / 50.0);
        let level_multiplier = 1.0 + (player_level as f64 * 0.1);
        
        let final_multiplier = difficulty_multiplier * level_multiplier * bonus_multiplier * self.config.reward_scaling;
        
        let experience = (base_exp as f64 * final_multiplier) as u32;
        let money = Money::new((base_money as f64 * final_multiplier) as i64);
        
        Ok(MissionRewards {
            experience,
            money,
            reputation: (difficulty.numeric_value / 10).max(1),
            special_items: Vec::new(), // Would be populated based on mission type
        })
    }

    /// Check if player meets mission prerequisites
    pub fn check_prerequisites(
        &self,
        mission: &Mission,
        player_level: u32,
        player_skills: &std::collections::HashMap<crate::experience::SkillType, SkillLevel>,
        completed_missions: &[MissionId],
    ) -> Result<PrerequisiteResult> {
        if !self.config.prerequisite_strict {
            return Ok(PrerequisiteResult {
                requirements_met: true,
                missing_requirements: Vec::new(),
            });
        }
        
        let mut missing_requirements = Vec::new();
        
        // Check level requirement
        if player_level < mission.min_level {
            missing_requirements.push(format!(
                "Level {} required (current: {})",
                mission.min_level, player_level
            ));
        }
        
        // Check skill requirements
        for (skill_type, required_level) in &mission.required_skills {
            if let Some(current_level) = player_skills.get(skill_type) {
                if current_level.value() < required_level.value() {
                    missing_requirements.push(format!(
                        "{:?} level {} required (current: {})",
                        skill_type, required_level.value(), current_level.value()
                    ));
                }
            } else {
                missing_requirements.push(format!(
                    "{:?} level {} required (not trained)",
                    skill_type, required_level.value()
                ));
            }
        }
        
        // Check prerequisite missions
        for prereq_id in &mission.prerequisite_missions {
            if !completed_missions.contains(prereq_id) {
                missing_requirements.push(format!("Mission {} must be completed first", prereq_id));
            }
        }
        
        Ok(PrerequisiteResult {
            requirements_met: missing_requirements.is_empty(),
            missing_requirements,
        })
    }
}

pub type MissionId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionType {
    Tutorial,
    Simple,
    Complex,
    Expert,
    Legendary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionDifficulty {
    pub numeric_value: u32,
    pub scaling_factors: DifficultyScaling,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyScaling {
    pub base_difficulty: u32,
    pub level_scaling: f64,
    pub type_modifier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionRewards {
    pub experience: u32,
    pub money: Money,
    pub reputation: u32,
    pub special_items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub id: MissionId,
    pub name: String,
    pub description: String,
    pub mission_type: MissionType,
    pub min_level: u32,
    pub required_skills: std::collections::HashMap<crate::experience::SkillType, SkillLevel>,
    pub prerequisite_missions: Vec<MissionId>,
    pub base_difficulty: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrerequisiteResult {
    pub requirements_met: bool,
    pub missing_requirements: Vec<String>,
}