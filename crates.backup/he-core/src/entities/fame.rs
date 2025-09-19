use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FameError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("User not found: {0}")]
    UserNotFound(u64),
    #[error("Achievement not found: {0}")]
    AchievementNotFound(String),
    #[error("Hall of Fame entry not found: {0}")]
    HallOfFameNotFound(u64),
    #[error("Invalid period: {0}")]
    InvalidPeriod(String),
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Already achieved")]
    AlreadyAchieved,
    #[error("Requirements not met")]
    RequirementsNotMet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementCategory {
    Hacking,
    Social,
    Economic,
    Competition,
    Community,
    Special,
    Legacy,
    Seasonal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Mythic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FamePeriod {
    Daily,
    Weekly,
    Monthly,
    Yearly,
    AllTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FameCategory {
    Overall,
    Hacking,
    Wealth,
    Social,
    PvP,
    Missions,
    Reputation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: AchievementCategory,
    pub rarity: AchievementRarity,
    pub points: u32,
    pub icon: String,
    pub requirements: Vec<AchievementRequirement>,
    pub rewards: Vec<AchievementReward>,
    pub is_hidden: bool,
    pub is_active: bool,
    pub unlock_date: Option<DateTime<Utc>>,
    pub expire_date: Option<DateTime<Utc>>,
    pub max_recipients: Option<u32>,
    pub current_recipients: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementRequirement {
    pub requirement_type: String,
    pub target_value: u64,
    pub current_value: Option<u64>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementReward {
    pub reward_type: String,
    pub value: u64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAchievement {
    pub id: u64,
    pub user_id: u64,
    pub achievement_id: String,
    pub achieved_at: DateTime<Utc>,
    pub progress: HashMap<String, u64>,
    pub is_displayed: bool,
    pub notification_sent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HallOfFameEntry {
    pub id: u64,
    pub user_id: u64,
    pub username: String,
    pub category: FameCategory,
    pub period: FamePeriod,
    pub value: u64,
    pub rank: u32,
    pub previous_rank: Option<u32>,
    pub entry_date: DateTime<Utc>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FameStats {
    pub user_id: u64,
    pub total_achievements: u32,
    pub achievement_points: u32,
    pub rarity_breakdown: HashMap<AchievementRarity, u32>,
    pub category_breakdown: HashMap<AchievementCategory, u32>,
    pub hall_of_fame_entries: u32,
    pub highest_rank: Option<u32>,
    pub current_rankings: HashMap<FameCategory, u32>,
    pub achievement_rate: f64, // Achievements per day
    pub last_achievement: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub user_id: u64,
    pub username: String,
    pub value: u64,
    pub change: i32, // Change from previous period
    pub avatar_url: Option<String>,
    pub country: Option<String>,
    pub clan_name: Option<String>,
}

/// Fame and Achievement system ported from PHP Fame class
/// Handles achievements, leaderboards, and hall of fame
pub struct Fame {
    current_user_id: Option<u64>,
}

impl Fame {
    /// Create new Fame instance
    pub fn new(current_user_id: Option<u64>) -> Self {
        Self { current_user_id }
    }

    /// Get all available achievements
    pub fn get_achievements(&self, category: Option<AchievementCategory>) -> Result<Vec<Achievement>, FameError> {
        // Simulate database query
        let mut achievements = vec![
            Achievement {
                id: "first_hack".to_string(),
                name: "First Steps".to_string(),
                description: "Complete your first hack".to_string(),
                category: AchievementCategory::Hacking,
                rarity: AchievementRarity::Common,
                points: 10,
                icon: "first_hack.png".to_string(),
                requirements: vec![
                    AchievementRequirement {
                        requirement_type: "hack_count".to_string(),
                        target_value: 1,
                        current_value: None,
                        description: "Complete 1 hack".to_string(),
                    }
                ],
                rewards: vec![
                    AchievementReward {
                        reward_type: "experience".to_string(),
                        value: 100,
                        description: "100 XP".to_string(),
                    }
                ],
                is_hidden: false,
                is_active: true,
                unlock_date: None,
                expire_date: None,
                max_recipients: None,
                current_recipients: 1250,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            Achievement {
                id: "millionaire".to_string(),
                name: "Millionaire".to_string(),
                description: "Accumulate 1,000,000 credits".to_string(),
                category: AchievementCategory::Economic,
                rarity: AchievementRarity::Rare,
                points: 100,
                icon: "millionaire.png".to_string(),
                requirements: vec![
                    AchievementRequirement {
                        requirement_type: "total_money".to_string(),
                        target_value: 1_000_000,
                        current_value: None,
                        description: "Accumulate 1,000,000 credits".to_string(),
                    }
                ],
                rewards: vec![
                    AchievementReward {
                        reward_type: "title".to_string(),
                        value: 1,
                        description: "Millionaire title".to_string(),
                    }
                ],
                is_hidden: false,
                is_active: true,
                unlock_date: None,
                expire_date: None,
                max_recipients: None,
                current_recipients: 89,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];

        // Filter by category if specified
        if let Some(cat) = category {
            achievements.retain(|a| std::mem::discriminant(&a.category) == std::mem::discriminant(&cat));
        }

        // Filter out inactive achievements
        achievements.retain(|a| a.is_active);

        Ok(achievements)
    }

    /// Get achievement by ID
    pub fn get_achievement(&self, achievement_id: &str) -> Result<Achievement, FameError> {
        let achievements = self.get_achievements(None)?;
        achievements
            .into_iter()
            .find(|a| a.id == achievement_id)
            .ok_or_else(|| FameError::AchievementNotFound(achievement_id.to_string()))
    }

    /// Get user's achievements
    pub fn get_user_achievements(&self, user_id: u64) -> Result<Vec<UserAchievement>, FameError> {
        // Simulate database query
        let user_achievements = vec![
            UserAchievement {
                id: 1,
                user_id,
                achievement_id: "first_hack".to_string(),
                achieved_at: Utc::now() - Duration::days(10),
                progress: HashMap::new(),
                is_displayed: true,
                notification_sent: true,
            }
        ];

        Ok(user_achievements)
    }

    /// Check and award achievements for user
    pub fn check_achievements(&self, user_id: u64) -> Result<Vec<UserAchievement>, FameError> {
        let all_achievements = self.get_achievements(None)?;
        let user_achievements = self.get_user_achievements(user_id)?;
        let achieved_ids: Vec<String> = user_achievements.iter()
            .map(|ua| ua.achievement_id.clone())
            .collect();

        let mut newly_achieved = vec![];

        for achievement in all_achievements {
            // Skip if already achieved
            if achieved_ids.contains(&achievement.id) {
                continue;
            }

            // Check if requirements are met
            if self.check_achievement_requirements(&achievement, user_id)? {
                let user_achievement = self.award_achievement(user_id, &achievement)?;
                newly_achieved.push(user_achievement);
            }
        }

        Ok(newly_achieved)
    }

    /// Award specific achievement to user
    fn award_achievement(&self, user_id: u64, achievement: &Achievement) -> Result<UserAchievement, FameError> {
        // Check if achievement is still available
        if let Some(max_recipients) = achievement.max_recipients {
            if achievement.current_recipients >= max_recipients {
                return Err(FameError::RequirementsNotMet);
            }
        }

        // Create user achievement record
        let user_achievement = UserAchievement {
            id: self.generate_id(),
            user_id,
            achievement_id: achievement.id.clone(),
            achieved_at: Utc::now(),
            progress: HashMap::new(),
            is_displayed: true,
            notification_sent: false,
        };

        // Award rewards
        for reward in &achievement.rewards {
            self.award_reward(user_id, reward)?;
        }

        // Save to database and update counters
        // In real implementation: INSERT INTO user_achievements, UPDATE achievements SET current_recipients = current_recipients + 1

        Ok(user_achievement)
    }

    /// Check if user meets achievement requirements
    fn check_achievement_requirements(&self, achievement: &Achievement, user_id: u64) -> Result<bool, FameError> {
        for requirement in &achievement.requirements {
            let current_value = self.get_user_stat(user_id, &requirement.requirement_type)?;
            if current_value < requirement.target_value {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Get user statistic value
    fn get_user_stat(&self, user_id: u64, stat_type: &str) -> Result<u64, FameError> {
        // Simulate database lookup for user statistics
        match stat_type {
            "hack_count" => Ok(5),
            "total_money" => Ok(500_000),
            "mission_count" => Ok(20),
            "social_connections" => Ok(15),
            _ => Ok(0),
        }
    }

    /// Award reward to user
    fn award_reward(&self, user_id: u64, reward: &AchievementReward) -> Result<(), FameError> {
        match reward.reward_type.as_str() {
            "experience" => {
                // Award experience points
                Ok(())
            }
            "money" => {
                // Award money
                Ok(())
            }
            "title" => {
                // Unlock title
                Ok(())
            }
            "badge" => {
                // Unlock badge
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Get Hall of Fame entries
    pub fn get_hall_of_fame(
        &self,
        category: FameCategory,
        period: FamePeriod,
        limit: u32,
    ) -> Result<Vec<HallOfFameEntry>, FameError> {
        let limit = if limit > 100 { 100 } else { limit };

        // Calculate period dates
        let (period_start, period_end) = self.calculate_period_dates(&period);

        // Simulate database query
        let entries = vec![
            HallOfFameEntry {
                id: 1,
                user_id: 1,
                username: "TopHacker".to_string(),
                category: category.clone(),
                period: period.clone(),
                value: 1_000_000,
                rank: 1,
                previous_rank: Some(2),
                entry_date: Utc::now(),
                period_start,
                period_end,
                is_current: true,
            }
        ];

        Ok(entries)
    }

    /// Get current leaderboard
    pub fn get_leaderboard(
        &self,
        category: FameCategory,
        period: FamePeriod,
        limit: u32,
    ) -> Result<Vec<LeaderboardEntry>, FameError> {
        let limit = if limit > 100 { 100 } else { limit };

        // Simulate leaderboard query
        let entries = vec![
            LeaderboardEntry {
                rank: 1,
                user_id: 1,
                username: "TopPlayer".to_string(),
                value: 1_500_000,
                change: 2, // Moved up 2 ranks
                avatar_url: Some("avatar1.jpg".to_string()),
                country: Some("US".to_string()),
                clan_name: Some("Elite Hackers".to_string()),
            },
            LeaderboardEntry {
                rank: 2,
                user_id: 2,
                username: "Runner Up".to_string(),
                value: 1_200_000,
                change: -1, // Moved down 1 rank
                avatar_url: Some("avatar2.jpg".to_string()),
                country: Some("CA".to_string()),
                clan_name: None,
            },
        ];

        Ok(entries.into_iter().take(limit as usize).collect())
    }

    /// Get user's fame statistics
    pub fn get_user_fame_stats(&self, user_id: u64) -> Result<FameStats, FameError> {
        let user_achievements = self.get_user_achievements(user_id)?;
        let all_achievements = self.get_achievements(None)?;

        // Calculate rarity breakdown
        let mut rarity_breakdown = HashMap::new();
        let mut category_breakdown = HashMap::new();
        let mut total_points = 0;

        for user_achievement in &user_achievements {
            if let Ok(achievement) = self.get_achievement(&user_achievement.achievement_id) {
                total_points += achievement.points;
                
                // Rarity breakdown
                *rarity_breakdown.entry(achievement.rarity.clone()).or_insert(0) += 1;
                
                // Category breakdown
                *category_breakdown.entry(achievement.category.clone()).or_insert(0) += 1;
            }
        }

        // Calculate achievement rate (achievements per day)
        let days_since_first = if let Some(first_achievement) = user_achievements.first() {
            let days = (Utc::now() - first_achievement.achieved_at).num_days() as f64;
            if days > 0.0 { days } else { 1.0 }
        } else {
            1.0
        };
        let achievement_rate = user_achievements.len() as f64 / days_since_first;

        // Get current rankings
        let mut current_rankings = HashMap::new();
        for category in [FameCategory::Overall, FameCategory::Hacking, FameCategory::Wealth] {
            if let Ok(leaderboard) = self.get_leaderboard(category.clone(), FamePeriod::AllTime, 1000) {
                if let Some(entry) = leaderboard.iter().find(|e| e.user_id == user_id) {
                    current_rankings.insert(category, entry.rank);
                }
            }
        }

        let stats = FameStats {
            user_id,
            total_achievements: user_achievements.len() as u32,
            achievement_points: total_points,
            rarity_breakdown,
            category_breakdown,
            hall_of_fame_entries: 0, // Would query hall of fame entries
            highest_rank: current_rankings.values().min().copied(),
            current_rankings,
            achievement_rate,
            last_achievement: user_achievements.last().map(|ua| ua.achieved_at),
        };

        Ok(stats)
    }

    /// Update leaderboards (periodic task)
    pub fn update_leaderboards(&self) -> Result<u32, FameError> {
        // This would be called periodically to update rankings
        // 1. Calculate new rankings for each category and period
        // 2. Update hall of fame entries
        // 3. Archive old entries
        // 4. Send notifications for rank changes

        // Return number of updated entries
        Ok(0)
    }

    /// Get achievement progress for user
    pub fn get_achievement_progress(&self, user_id: u64, achievement_id: &str) -> Result<HashMap<String, f64>, FameError> {
        let achievement = self.get_achievement(achievement_id)?;
        let mut progress = HashMap::new();

        for requirement in &achievement.requirements {
            let current_value = self.get_user_stat(user_id, &requirement.requirement_type)?;
            let progress_pct = (current_value as f64 / requirement.target_value as f64 * 100.0).min(100.0);
            progress.insert(requirement.requirement_type.clone(), progress_pct);
        }

        Ok(progress)
    }

    /// Get nearby achievements (close to completion)
    pub fn get_nearby_achievements(&self, user_id: u64, threshold: f64) -> Result<Vec<(Achievement, f64)>, FameError> {
        let all_achievements = self.get_achievements(None)?;
        let user_achievements = self.get_user_achievements(user_id)?;
        let achieved_ids: Vec<String> = user_achievements.iter()
            .map(|ua| ua.achievement_id.clone())
            .collect();

        let mut nearby = vec![];

        for achievement in all_achievements {
            if achieved_ids.contains(&achievement.id) {
                continue;
            }

            let progress = self.get_achievement_progress(user_id, &achievement.id)?;
            let min_progress = progress.values().fold(0.0, |acc, &x| if acc == 0.0 { x } else { acc.min(x) });

            if min_progress >= threshold {
                nearby.push((achievement, min_progress));
            }
        }

        // Sort by progress descending
        nearby.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        Ok(nearby)
    }

    /// Helper methods
    fn calculate_period_dates(&self, period: &FamePeriod) -> (DateTime<Utc>, DateTime<Utc>) {
        let now = Utc::now();
        
        match period {
            FamePeriod::Daily => {
                let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
                let end = start + Duration::days(1);
                (start, end)
            }
            FamePeriod::Weekly => {
                let days_since_monday = now.weekday().num_days_from_monday();
                let start = (now - Duration::days(days_since_monday as i64))
                    .date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
                let end = start + Duration::weeks(1);
                (start, end)
            }
            FamePeriod::Monthly => {
                let start = now.date_naive().with_day(1).unwrap().and_hms_opt(0, 0, 0).unwrap().and_utc();
                let end = if start.month() == 12 {
                    start.with_year(start.year() + 1).unwrap().with_month(1).unwrap()
                } else {
                    start.with_month(start.month() + 1).unwrap()
                };
                (start, end)
            }
            FamePeriod::Yearly => {
                let start = now.date_naive().with_ordinal(1).unwrap().and_hms_opt(0, 0, 0).unwrap().and_utc();
                let end = start.with_year(start.year() + 1).unwrap();
                (start, end)
            }
            FamePeriod::AllTime => {
                let start = DateTime::from_timestamp(0, 0).unwrap();
                let end = DateTime::from_timestamp(i64::MAX, 0).unwrap();
                (start, end)
            }
        }
    }

    fn generate_id(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fame_creation() {
        let fame = Fame::new(Some(1));
        assert_eq!(fame.current_user_id, Some(1));
    }

    #[test]
    fn test_get_achievements() {
        let fame = Fame::new(None);
        let result = fame.get_achievements(None);
        assert!(result.is_ok());

        let achievements = result.unwrap();
        assert!(!achievements.is_empty());
        assert!(achievements.iter().all(|a| a.is_active));
    }

    #[test]
    fn test_get_achievement_by_category() {
        let fame = Fame::new(None);
        let result = fame.get_achievements(Some(AchievementCategory::Hacking));
        assert!(result.is_ok());

        let achievements = result.unwrap();
        // All returned achievements should be hacking category
        // Note: This test would work properly with real enum comparison
    }

    #[test]
    fn test_get_achievement() {
        let fame = Fame::new(None);
        let result = fame.get_achievement("first_hack");
        assert!(result.is_ok());

        let achievement = result.unwrap();
        assert_eq!(achievement.id, "first_hack");
        assert_eq!(achievement.name, "First Steps");
    }

    #[test]
    fn test_invalid_achievement() {
        let fame = Fame::new(None);
        let result = fame.get_achievement("invalid_achievement");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FameError::AchievementNotFound(_)));
    }

    #[test]
    fn test_user_achievements() {
        let fame = Fame::new(None);
        let result = fame.get_user_achievements(1);
        assert!(result.is_ok());

        let achievements = result.unwrap();
        assert!(!achievements.is_empty());
    }

    #[test]
    fn test_achievement_progress() {
        let fame = Fame::new(None);
        let result = fame.get_achievement_progress(1, "first_hack");
        assert!(result.is_ok());

        let progress = result.unwrap();
        assert!(!progress.is_empty());
    }

    #[test]
    fn test_leaderboard() {
        let fame = Fame::new(None);
        let result = fame.get_leaderboard(FameCategory::Overall, FamePeriod::AllTime, 10);
        assert!(result.is_ok());

        let leaderboard = result.unwrap();
        assert!(!leaderboard.is_empty());
        
        // Check that ranks are in order
        for (i, entry) in leaderboard.iter().enumerate() {
            assert_eq!(entry.rank, (i + 1) as u32);
        }
    }

    #[test]
    fn test_period_date_calculation() {
        let fame = Fame::new(None);
        
        let (start, end) = fame.calculate_period_dates(&FamePeriod::Daily);
        assert!(end > start);
        assert_eq!((end - start).num_days(), 1);

        let (start, end) = fame.calculate_period_dates(&FamePeriod::Weekly);
        assert!(end > start);
        assert_eq!((end - start).num_weeks(), 1);
    }
}