//! Progression API client

use leptos::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::api_request;

/// Player progression data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProgression {
    pub player_id: Uuid,
    pub level_info: LevelInfo,
    pub skill_tree: SkillTree,
    pub achievements: AchievementProgress,
    pub unlockables: UnlockableContent,
    pub reputation: ReputationSystem,
    pub statistics: PlayerStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelInfo {
    pub level: u32,
    pub current_experience: u64,
    pub total_experience: u64,
    pub experience_to_next: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTree {
    pub skill_points_available: u32,
    pub skill_points_spent: u32,
    pub unlocked_skills: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementProgress {
    pub unlocked_achievements: Vec<String>,
    pub achievement_points: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockableContent {
    pub unlocked_items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationSystem {
    pub faction_standings: Vec<FactionStanding>,
    pub total_reputation: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionStanding {
    pub faction_id: String,
    pub faction_name: String,
    pub current_reputation: i32,
    pub reputation_level: String,
    pub rank: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStatistics {
    pub servers_hacked: u32,
    pub missions_completed: u32,
    pub money_earned: i64,
    pub pvp_wins: u32,
    pub pvp_losses: u32,
    pub time_played_seconds: u64,
}

/// Get player progression
pub async fn get_progression() -> Result<PlayerProgression, ServerFnError> {
    api_request::<(), PlayerProgression>("GET", "/api/progression", None).await
}

/// Add experience
#[derive(Serialize)]
pub struct AddExperienceRequest {
    pub amount: u32,
    pub source: String,
}

#[derive(Deserialize)]
pub struct AddExperienceResponse {
    pub experience_gained: u32,
    pub level_ups: u32,
    pub new_level: u32,
}

pub async fn add_experience(amount: u32, source: String) -> Result<AddExperienceResponse, ServerFnError> {
    let request = AddExperienceRequest { amount, source };
    api_request("POST", "/api/progression/experience", Some(request)).await
}

/// Invest skill points
#[derive(Serialize)]
pub struct InvestSkillRequest {
    pub skill_id: String,
    pub points: u32,
}

pub async fn invest_skill(skill_id: String, points: u32) -> Result<SkillTree, ServerFnError> {
    let request = InvestSkillRequest { skill_id, points };
    api_request("POST", "/api/progression/skills/invest", Some(request)).await
}

/// Reset skills
pub async fn reset_skills() -> Result<SkillTree, ServerFnError> {
    api_request::<(), SkillTree>("POST", "/api/progression/skills/reset", None).await
}

/// Get achievements
pub async fn get_achievements() -> Result<AchievementProgress, ServerFnError> {
    api_request::<(), AchievementProgress>("GET", "/api/progression/achievements", None).await
}

/// Get unlockables
pub async fn get_unlockables() -> Result<UnlockableContent, ServerFnError> {
    api_request::<(), UnlockableContent>("GET", "/api/progression/unlockables", None).await
}

/// Get reputation
pub async fn get_reputation() -> Result<ReputationSystem, ServerFnError> {
    api_request::<(), ReputationSystem>("GET", "/api/progression/reputation", None).await
}

/// Get statistics
pub async fn get_statistics() -> Result<PlayerStatistics, ServerFnError> {
    api_request::<(), PlayerStatistics>("GET", "/api/progression/statistics", None).await
}

/// Complete action (for tracking)
#[derive(Serialize)]
pub struct CompleteActionRequest {
    pub action_type: String,
    pub details: serde_json::Value,
}

pub async fn complete_action(action_type: String) -> Result<serde_json::Value, ServerFnError> {
    let request = CompleteActionRequest {
        action_type,
        details: serde_json::json!({}),
    };
    api_request("POST", "/api/progression/action", Some(request)).await
}

/// Get leaderboard
#[derive(Serialize)]
pub struct LeaderboardRequest {
    pub board_type: String,
    pub limit: Option<usize>,
}

#[derive(Deserialize, Clone)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub player_id: Uuid,
    pub player_name: String,
    pub value: i64,
}

pub async fn get_leaderboard(board_type: String) -> Result<Vec<LeaderboardEntry>, ServerFnError> {
    api_request::<(), Vec<LeaderboardEntry>>(
        "GET",
        &format!("/api/progression/leaderboard?board_type={}", board_type),
        None,
    ).await
}