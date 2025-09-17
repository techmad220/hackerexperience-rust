//! Progression API handlers

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use he_progression::{
    PlayerProgression, LevelInfo, SkillTree, AchievementProgress,
    UnlockableContent, ReputationSystem, ProgressionEvent,
    PlayerStatistics, SkillError,
};

use crate::{error::ApiResult, auth::Claims};

/// Get player progression data
pub async fn get_progression(
    claims: Claims,
) -> ApiResult<HttpResponse> {
    // TODO: Load from database
    let progression = PlayerProgression::new(claims.sub.parse::<Uuid>().unwrap());

    Ok(HttpResponse::Ok().json(&progression))
}

/// Add experience points
#[derive(Debug, Deserialize)]
pub struct AddExperienceRequest {
    pub amount: u32,
    pub source: String,
}

#[derive(Debug, Serialize)]
pub struct AddExperienceResponse {
    pub experience_gained: u32,
    pub level_ups: u32,
    pub new_level: u32,
    pub events: Vec<ProgressionEvent>,
}

pub async fn add_experience(
    claims: Claims,
    payload: web::Json<AddExperienceRequest>,
) -> ApiResult<HttpResponse> {
    // TODO: Load from database
    let mut progression = PlayerProgression::new(claims.sub.parse::<Uuid>().unwrap());

    let events = progression.gain_experience(payload.amount);

    let response = AddExperienceResponse {
        experience_gained: payload.amount,
        level_ups: events.iter().filter(|e| matches!(e, ProgressionEvent::LevelUp(_))).count() as u32,
        new_level: progression.level_info.level,
        events,
    };

    // TODO: Save to database

    Ok(HttpResponse::Ok().json(&response))
}

/// Invest skill points
#[derive(Debug, Deserialize)]
pub struct InvestSkillRequest {
    pub skill_id: String,
    pub points: u32,
}

pub async fn invest_skill(
    claims: Claims,
    payload: web::Json<InvestSkillRequest>,
) -> ApiResult<HttpResponse> {
    // TODO: Load from database
    let mut progression = PlayerProgression::new(claims.sub.parse::<Uuid>().unwrap());

    match progression.skill_tree.invest_skill(&payload.skill_id, payload.points) {
        Ok(()) => {
            // TODO: Save to database
            Ok(HttpResponse::Ok().json(&progression.skill_tree))
        }
        Err(e) => {
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": e.to_string()
            })))
        }
    }
}

/// Reset skill tree
pub async fn reset_skills(
    claims: Claims,
) -> ApiResult<HttpResponse> {
    // TODO: Load from database
    let mut progression = PlayerProgression::new(claims.sub.parse::<Uuid>().unwrap());

    // TODO: Check if player has reset item or enough money
    progression.skill_tree.reset_skills();

    // TODO: Save to database

    Ok(HttpResponse::Ok().json(&progression.skill_tree))
}

/// Get achievements
pub async fn get_achievements(
    claims: Claims,
) -> ApiResult<HttpResponse> {
    // TODO: Load from database
    let progression = PlayerProgression::new(claims.sub.parse::<Uuid>().unwrap());

    Ok(HttpResponse::Ok().json(&progression.achievements))
}

/// Get unlockables
pub async fn get_unlockables(
    claims: Claims,
) -> ApiResult<HttpResponse> {
    // TODO: Load from database
    let progression = PlayerProgression::new(claims.sub.parse::<Uuid>().unwrap());

    Ok(HttpResponse::Ok().json(&progression.unlockables))
}

/// Get reputation
pub async fn get_reputation(
    claims: Claims,
) -> ApiResult<HttpResponse> {
    // TODO: Load from database
    let progression = PlayerProgression::new(claims.sub.parse::<Uuid>().unwrap());

    Ok(HttpResponse::Ok().json(&progression.reputation))
}

/// Modify faction reputation
#[derive(Debug, Deserialize)]
pub struct ModifyReputationRequest {
    pub faction_id: String,
    pub amount: i32,
}

pub async fn modify_reputation(
    claims: Claims,
    payload: web::Json<ModifyReputationRequest>,
) -> ApiResult<HttpResponse> {
    // TODO: Load from database
    let mut progression = PlayerProgression::new(claims.sub.parse::<Uuid>().unwrap());

    let change = progression.reputation.modify_reputation(&payload.faction_id, payload.amount);

    // TODO: Save to database

    Ok(HttpResponse::Ok().json(&change))
}

/// Get player statistics
pub async fn get_statistics(
    claims: Claims,
) -> ApiResult<HttpResponse> {
    // TODO: Load from database
    let progression = PlayerProgression::new(claims.sub.parse::<Uuid>().unwrap());

    Ok(HttpResponse::Ok().json(&progression.statistics))
}

/// Complete an action (for statistics tracking)
#[derive(Debug, Deserialize)]
pub struct CompleteActionRequest {
    pub action_type: String,
    pub details: serde_json::Value,
}

pub async fn complete_action(
    claims: Claims,
    payload: web::Json<CompleteActionRequest>,
) -> ApiResult<HttpResponse> {
    // TODO: Load from database
    let mut progression = PlayerProgression::new(claims.sub.parse::<Uuid>().unwrap());

    // Update statistics based on action
    match payload.action_type.as_str() {
        "server_hack" => {
            progression.statistics.servers_hacked += 1;
            progression.statistics.total_hacks += 1;
        }
        "mission_complete" => {
            progression.statistics.missions_completed += 1;
        }
        "virus_upload" => {
            progression.statistics.viruses_uploaded += 1;
        }
        "file_download" => {
            progression.statistics.files_downloaded += 1;
        }
        "pvp_win" => {
            progression.statistics.pvp_wins += 1;
            progression.statistics.pvp_matches += 1;
        }
        "pvp_loss" => {
            progression.statistics.pvp_losses += 1;
            progression.statistics.pvp_matches += 1;
        }
        _ => {}
    }

    // Check for new achievements
    let events = progression.check_achievements();

    // TODO: Save to database

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "events": events,
        "statistics": progression.statistics
    })))
}

/// Get leaderboard
#[derive(Debug, Deserialize)]
pub struct LeaderboardRequest {
    pub board_type: String, // "level", "reputation", "pvp", "achievements"
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub player_id: Uuid,
    pub player_name: String,
    pub value: i64,
}

pub async fn get_leaderboard(
    query: web::Query<LeaderboardRequest>,
) -> ApiResult<HttpResponse> {
    let limit = query.limit.unwrap_or(100);

    // TODO: Query from database
    // For now, return mock data
    let entries: Vec<LeaderboardEntry> = vec![];

    Ok(HttpResponse::Ok().json(&entries))
}