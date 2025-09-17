//! Achievement System - Track and reward player accomplishments

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::{PlayerStatistics, LevelInfo, ProgressionEvent};

/// Achievement progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementProgress {
    pub unlocked_achievements: HashSet<String>,
    pub achievement_points: u32,
    pub progress_tracking: HashMap<String, AchievementTracker>,
    pub categories: AchievementCategories,
    pub completed_at: HashMap<String, DateTime<Utc>>,
}

/// Achievement categories for organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementCategories {
    pub hacking: Vec<Achievement>,
    pub progression: Vec<Achievement>,
    pub collection: Vec<Achievement>,
    pub exploration: Vec<Achievement>,
    pub social: Vec<Achievement>,
    pub mastery: Vec<Achievement>,
    pub special: Vec<Achievement>,
}

/// Individual achievement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub points: u32,
    pub rarity: AchievementRarity,
    pub requirements: AchievementRequirement,
    pub rewards: AchievementRewards,
    pub hidden: bool,
}

/// Achievement rarity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementRarity {
    Common,      // 80%+ of players
    Uncommon,    // 50-80% of players
    Rare,        // 20-50% of players
    Epic,        // 5-20% of players
    Legendary,   // <5% of players
}

/// Requirements to unlock an achievement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementRequirement {
    ServersHacked(u32),
    MoneyEarned(i64),
    LevelReached(u32),
    MissionsCompleted(u32),
    FilesCollected(u32),
    VirusesUploaded(u32),
    PvpWins(u32),
    ConsecutiveDailyLogins(u32),
    SpecificAction(String),
    MultiRequirement(Vec<AchievementRequirement>),
    Custom(String),
}

/// Rewards for completing achievements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementRewards {
    pub experience: u32,
    pub money: i64,
    pub items: Vec<String>,
    pub titles: Vec<String>,
    pub cosmetics: Vec<String>,
}

/// Tracks progress for an achievement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementTracker {
    pub current_progress: u32,
    pub target_progress: u32,
    pub percentage: f32,
}

impl AchievementProgress {
    /// Create new achievement progress
    pub fn new() -> Self {
        Self {
            unlocked_achievements: HashSet::new(),
            achievement_points: 0,
            progress_tracking: HashMap::new(),
            categories: AchievementCategories::default(),
            completed_at: HashMap::new(),
        }
    }

    /// Check for new achievements based on current stats
    pub fn check_achievements(&mut self, stats: &PlayerStatistics, level_info: &LevelInfo) -> Vec<ProgressionEvent> {
        let mut events = Vec::new();

        // Check all achievements
        let all_achievements = self.get_all_achievements();

        for achievement in all_achievements {
            if !self.unlocked_achievements.contains(&achievement.id) {
                if self.check_requirement(&achievement.requirements, stats, level_info) {
                    self.unlock_achievement(achievement.clone());
                    events.push(ProgressionEvent::AchievementEarned(achievement));
                }
            }
        }

        events
    }

    /// Check if a requirement is met
    fn check_requirement(&self, req: &AchievementRequirement, stats: &PlayerStatistics, level_info: &LevelInfo) -> bool {
        match req {
            AchievementRequirement::ServersHacked(required) => stats.servers_hacked >= *required,
            AchievementRequirement::MoneyEarned(required) => stats.money_earned >= *required,
            AchievementRequirement::LevelReached(required) => level_info.level >= *required,
            AchievementRequirement::MissionsCompleted(required) => stats.missions_completed >= *required,
            AchievementRequirement::FilesCollected(required) => stats.files_downloaded >= *required,
            AchievementRequirement::VirusesUploaded(required) => stats.viruses_uploaded >= *required,
            AchievementRequirement::PvpWins(required) => stats.pvp_wins >= *required,
            AchievementRequirement::MultiRequirement(reqs) => {
                reqs.iter().all(|r| self.check_requirement(r, stats, level_info))
            }
            _ => false,
        }
    }

    /// Unlock an achievement
    fn unlock_achievement(&mut self, achievement: Achievement) {
        self.unlocked_achievements.insert(achievement.id.clone());
        self.achievement_points += achievement.points;
        self.completed_at.insert(achievement.id.clone(), Utc::now());
    }

    /// Get total achievement points
    pub fn get_total_points(&self) -> u32 {
        self.achievement_points
    }

    /// Get completion percentage
    pub fn get_completion_percentage(&self) -> f32 {
        let total_achievements = self.get_all_achievements().len();
        if total_achievements == 0 {
            0.0
        } else {
            (self.unlocked_achievements.len() as f32 / total_achievements as f32) * 100.0
        }
    }

    /// Get all achievements
    fn get_all_achievements(&self) -> Vec<Achievement> {
        let mut all = Vec::new();
        all.extend(self.categories.hacking.clone());
        all.extend(self.categories.progression.clone());
        all.extend(self.categories.collection.clone());
        all.extend(self.categories.exploration.clone());
        all.extend(self.categories.social.clone());
        all.extend(self.categories.mastery.clone());
        all.extend(self.categories.special.clone());
        all
    }

    /// Update progress tracking for incremental achievements
    pub fn update_tracker(&mut self, achievement_id: &str, progress: u32, target: u32) {
        self.progress_tracking.insert(
            achievement_id.to_string(),
            AchievementTracker {
                current_progress: progress,
                target_progress: target,
                percentage: (progress as f32 / target as f32) * 100.0,
            },
        );
    }
}

impl Default for AchievementCategories {
    fn default() -> Self {
        Self {
            hacking: vec![
                Achievement {
                    id: "first_hack".to_string(),
                    name: "First Blood".to_string(),
                    description: "Successfully hack your first server".to_string(),
                    icon: "🔓".to_string(),
                    points: 10,
                    rarity: AchievementRarity::Common,
                    requirements: AchievementRequirement::ServersHacked(1),
                    rewards: AchievementRewards {
                        experience: 100,
                        money: 1000,
                        items: vec![],
                        titles: vec!["Novice Hacker".to_string()],
                        cosmetics: vec![],
                    },
                    hidden: false,
                },
                Achievement {
                    id: "hack_100".to_string(),
                    name: "Century Mark".to_string(),
                    description: "Hack 100 servers".to_string(),
                    icon: "💯".to_string(),
                    points: 50,
                    rarity: AchievementRarity::Uncommon,
                    requirements: AchievementRequirement::ServersHacked(100),
                    rewards: AchievementRewards {
                        experience: 5000,
                        money: 50000,
                        items: vec!["Elite Scanner v2.0".to_string()],
                        titles: vec!["Server Hunter".to_string()],
                        cosmetics: vec![],
                    },
                    hidden: false,
                },
                Achievement {
                    id: "hack_elite".to_string(),
                    name: "Elite Access".to_string(),
                    description: "Successfully hack an elite tier server".to_string(),
                    icon: "🎯".to_string(),
                    points: 100,
                    rarity: AchievementRarity::Epic,
                    requirements: AchievementRequirement::Custom("hack_tier_4_server".to_string()),
                    rewards: AchievementRewards {
                        experience: 10000,
                        money: 100000,
                        items: vec!["Quantum Decryptor".to_string()],
                        titles: vec!["Elite Breaker".to_string()],
                        cosmetics: vec!["Golden Terminal".to_string()],
                    },
                    hidden: false,
                },
            ],
            progression: vec![
                Achievement {
                    id: "level_10".to_string(),
                    name: "Double Digits".to_string(),
                    description: "Reach level 10".to_string(),
                    icon: "🔟".to_string(),
                    points: 20,
                    rarity: AchievementRarity::Common,
                    requirements: AchievementRequirement::LevelReached(10),
                    rewards: AchievementRewards {
                        experience: 1000,
                        money: 5000,
                        items: vec![],
                        titles: vec!["Experienced".to_string()],
                        cosmetics: vec![],
                    },
                    hidden: false,
                },
                Achievement {
                    id: "level_50".to_string(),
                    name: "Halfway to Legend".to_string(),
                    description: "Reach level 50".to_string(),
                    icon: "⭐".to_string(),
                    points: 75,
                    rarity: AchievementRarity::Rare,
                    requirements: AchievementRequirement::LevelReached(50),
                    rewards: AchievementRewards {
                        experience: 10000,
                        money: 100000,
                        items: vec!["Skill Reset Token".to_string()],
                        titles: vec!["Veteran".to_string()],
                        cosmetics: vec!["Platinum Badge".to_string()],
                    },
                    hidden: false,
                },
            ],
            collection: vec![
                Achievement {
                    id: "software_collector".to_string(),
                    name: "Software Hoarder".to_string(),
                    description: "Collect 50 different software programs".to_string(),
                    icon: "📦".to_string(),
                    points: 30,
                    rarity: AchievementRarity::Uncommon,
                    requirements: AchievementRequirement::Custom("collect_50_software".to_string()),
                    rewards: AchievementRewards {
                        experience: 2500,
                        money: 25000,
                        items: vec!["Storage Expansion".to_string()],
                        titles: vec!["Collector".to_string()],
                        cosmetics: vec![],
                    },
                    hidden: false,
                },
            ],
            exploration: vec![
                Achievement {
                    id: "discover_hidden".to_string(),
                    name: "Hidden Network".to_string(),
                    description: "Discover a hidden server network".to_string(),
                    icon: "🔍".to_string(),
                    points: 50,
                    rarity: AchievementRarity::Rare,
                    requirements: AchievementRequirement::Custom("find_hidden_network".to_string()),
                    rewards: AchievementRewards {
                        experience: 5000,
                        money: 50000,
                        items: vec!["Network Map".to_string()],
                        titles: vec!["Explorer".to_string()],
                        cosmetics: vec![],
                    },
                    hidden: true,
                },
            ],
            social: vec![
                Achievement {
                    id: "pvp_first_win".to_string(),
                    name: "First Victory".to_string(),
                    description: "Win your first PvP hack".to_string(),
                    icon: "🏆".to_string(),
                    points: 15,
                    rarity: AchievementRarity::Common,
                    requirements: AchievementRequirement::PvpWins(1),
                    rewards: AchievementRewards {
                        experience: 500,
                        money: 2500,
                        items: vec![],
                        titles: vec!["Competitor".to_string()],
                        cosmetics: vec![],
                    },
                    hidden: false,
                },
                Achievement {
                    id: "pvp_champion".to_string(),
                    name: "Undefeated".to_string(),
                    description: "Win 100 PvP hacks".to_string(),
                    icon: "🥇".to_string(),
                    points: 100,
                    rarity: AchievementRarity::Epic,
                    requirements: AchievementRequirement::PvpWins(100),
                    rewards: AchievementRewards {
                        experience: 15000,
                        money: 200000,
                        items: vec!["PvP Shield".to_string()],
                        titles: vec!["Champion".to_string()],
                        cosmetics: vec!["Champion Crown".to_string()],
                    },
                    hidden: false,
                },
            ],
            mastery: vec![
                Achievement {
                    id: "perfect_hack".to_string(),
                    name: "Flawless Execution".to_string(),
                    description: "Complete a hack without triggering any alarms".to_string(),
                    icon: "🤫".to_string(),
                    points: 40,
                    rarity: AchievementRarity::Rare,
                    requirements: AchievementRequirement::Custom("perfect_hack".to_string()),
                    rewards: AchievementRewards {
                        experience: 3000,
                        money: 30000,
                        items: vec!["Stealth Module".to_string()],
                        titles: vec!["Ghost".to_string()],
                        cosmetics: vec![],
                    },
                    hidden: false,
                },
            ],
            special: vec![
                Achievement {
                    id: "mystery_solver".to_string(),
                    name: "The Truth".to_string(),
                    description: "Uncover the mystery of the 13.37.13.37 server".to_string(),
                    icon: "🎭".to_string(),
                    points: 200,
                    rarity: AchievementRarity::Legendary,
                    requirements: AchievementRequirement::Custom("solve_mystery".to_string()),
                    rewards: AchievementRewards {
                        experience: 50000,
                        money: 1000000,
                        items: vec!["Quantum Computer".to_string()],
                        titles: vec!["Truth Seeker".to_string(), "Legend".to_string()],
                        cosmetics: vec!["Legendary Aura".to_string()],
                    },
                    hidden: true,
                },
                Achievement {
                    id: "millionaire".to_string(),
                    name: "Digital Millionaire".to_string(),
                    description: "Earn 1,000,000 credits total".to_string(),
                    icon: "💰".to_string(),
                    points: 75,
                    rarity: AchievementRarity::Rare,
                    requirements: AchievementRequirement::MoneyEarned(1000000),
                    rewards: AchievementRewards {
                        experience: 10000,
                        money: 100000,
                        items: vec!["Golden USB".to_string()],
                        titles: vec!["Rich".to_string()],
                        cosmetics: vec!["Money Rain Effect".to_string()],
                    },
                    hidden: false,
                },
            ],
        }
    }
}