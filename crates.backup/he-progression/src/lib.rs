//! Player Progression System
//! Handles leveling, experience, skills, achievements, and unlockables

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub mod leveling;
pub mod skills;
pub mod achievements;
pub mod unlockables;
pub mod reputation;

pub use leveling::*;
pub use skills::*;
pub use achievements::*;
pub use unlockables::*;
pub use reputation::*;

/// Complete player progression profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProgression {
    pub player_id: Uuid,
    pub username: String,
    pub level_info: LevelInfo,
    pub skill_tree: SkillTree,
    pub achievements: AchievementProgress,
    pub unlockables: UnlockableContent,
    pub reputation: ReputationSystem,
    pub statistics: PlayerStatistics,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

/// Player statistics for tracking progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStatistics {
    pub total_playtime: u64, // seconds
    pub servers_hacked: u32,
    pub money_earned: i64,
    pub money_spent: i64,
    pub files_downloaded: u32,
    pub files_deleted: u32,
    pub viruses_uploaded: u32,
    pub processes_completed: u32,
    pub missions_completed: u32,
    pub pvp_wins: u32,
    pub pvp_losses: u32,
    pub clan_contributions: i64,
}

impl PlayerProgression {
    /// Create new player progression
    pub fn new(player_id: Uuid, username: String) -> Self {
        Self {
            player_id,
            username,
            level_info: LevelInfo::new(),
            skill_tree: SkillTree::new(),
            achievements: AchievementProgress::new(),
            unlockables: UnlockableContent::new(),
            reputation: ReputationSystem::new(),
            statistics: PlayerStatistics::default(),
            created_at: Utc::now(),
            last_updated: Utc::now(),
        }
    }

    /// Add experience and check for level ups
    pub fn add_experience(&mut self, amount: u32, source: ExperienceSource) -> Vec<ProgressionEvent> {
        let mut events = Vec::new();

        let old_level = self.level_info.level;
        let level_ups = self.level_info.add_experience(amount);

        // Track experience gain
        events.push(ProgressionEvent::ExperienceGained {
            amount,
            source,
            new_total: self.level_info.total_experience,
        });

        // Handle level ups
        for new_level in (old_level + 1)..=self.level_info.level {
            events.push(ProgressionEvent::LevelUp {
                new_level,
                rewards: self.get_level_rewards(new_level),
            });

            // Grant skill points
            let skill_points = self.calculate_skill_points(new_level);
            self.skill_tree.add_skill_points(skill_points);

            // Check for unlocks
            let unlocks = self.unlockables.check_level_unlocks(new_level);
            for unlock in unlocks {
                events.push(ProgressionEvent::ContentUnlocked(unlock));
            }
        }

        // Check achievements
        let achievement_events = self.achievements.check_achievements(&self.statistics, &self.level_info);
        events.extend(achievement_events);

        self.last_updated = Utc::now();
        events
    }

    /// Calculate skill points awarded for a level
    fn calculate_skill_points(&self, level: u32) -> u32 {
        match level {
            1..=10 => 1,
            11..=25 => 2,
            26..=50 => 3,
            51..=75 => 4,
            _ => 5,
        }
    }

    /// Get rewards for reaching a level
    fn get_level_rewards(&self, level: u32) -> LevelRewards {
        LevelRewards {
            money: 1000 * level as i64,
            items: match level {
                5 => vec!["Basic Firewall v2.0".to_string()],
                10 => vec!["Advanced Cracker v1.0".to_string()],
                20 => vec!["Elite Scanner v1.0".to_string()],
                30 => vec!["Quantum Processor Upgrade".to_string()],
                50 => vec!["AI Assistant Module".to_string()],
                _ => vec![],
            },
            titles: match level {
                10 => vec!["Script Kiddie".to_string()],
                25 => vec!["Hacker".to_string()],
                50 => vec!["Elite Hacker".to_string()],
                75 => vec!["Master Hacker".to_string()],
                100 => vec!["Legend".to_string()],
                _ => vec![],
            },
        }
    }

    /// Update statistics
    pub fn update_stats<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut PlayerStatistics)
    {
        updater(&mut self.statistics);
        self.last_updated = Utc::now();
    }

    /// Get current progression percentage to next level
    pub fn get_level_progress(&self) -> f32 {
        self.level_info.get_progress_percentage()
    }

    /// Get total progression score
    pub fn get_progression_score(&self) -> u64 {
        let level_score = self.level_info.level as u64 * 1000;
        let achievement_score = self.achievements.get_total_points() as u64;
        let skill_score = self.skill_tree.get_total_invested_points() as u64 * 100;
        let reputation_score = self.reputation.get_total_reputation() as u64;

        level_score + achievement_score + skill_score + reputation_score
    }
}

/// Events that occur during progression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgressionEvent {
    ExperienceGained {
        amount: u32,
        source: ExperienceSource,
        new_total: u64,
    },
    LevelUp {
        new_level: u32,
        rewards: LevelRewards,
    },
    SkillUnlocked(String),
    AchievementEarned(Achievement),
    ContentUnlocked(UnlockedContent),
    ReputationChanged {
        faction: String,
        change: i32,
        new_value: i32,
    },
    TitleEarned(String),
}

/// Source of experience points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperienceSource {
    ServerHack { difficulty: u32 },
    MissionComplete { mission_id: String },
    ProcessComplete { process_type: String },
    PvpVictory { opponent_level: u32 },
    DailyBonus,
    Achievement { name: String },
    Discovery { item: String },
}

/// Rewards for leveling up
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelRewards {
    pub money: i64,
    pub items: Vec<String>,
    pub titles: Vec<String>,
}

impl Default for PlayerStatistics {
    fn default() -> Self {
        Self {
            total_playtime: 0,
            servers_hacked: 0,
            money_earned: 0,
            money_spent: 0,
            files_downloaded: 0,
            files_deleted: 0,
            viruses_uploaded: 0,
            processes_completed: 0,
            missions_completed: 0,
            pvp_wins: 0,
            pvp_losses: 0,
            clan_contributions: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progression_creation() {
        let player_id = Uuid::new_v4();
        let progression = PlayerProgression::new(player_id, "TestPlayer".to_string());

        assert_eq!(progression.player_id, player_id);
        assert_eq!(progression.username, "TestPlayer");
        assert_eq!(progression.level_info.level, 1);
    }

    #[test]
    fn test_experience_and_leveling() {
        let mut progression = PlayerProgression::new(Uuid::new_v4(), "TestPlayer".to_string());

        // Add experience
        let events = progression.add_experience(
            500,
            ExperienceSource::ServerHack { difficulty: 3 }
        );

        assert!(!events.is_empty());
        assert!(progression.level_info.total_experience >= 500);
    }
}