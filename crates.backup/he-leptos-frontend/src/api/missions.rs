//! Missions API client

use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub id: i64,
    pub mission_type: String,
    pub status: String,
    pub reward_money: i64,
    pub reward_xp: i32,
    pub progress: i32,
    pub total_steps: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MissionsResponse {
    pub success: bool,
    pub missions: Vec<Mission>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressUpdate {
    pub progress: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MissionActionResponse {
    pub success: bool,
    pub message: String,
}

/// Get all missions for the user
pub async fn get_missions() -> Result<Vec<Mission>, String> {
    let client = reqwest::Client::new();
    let token = get_auth_token();

    match client
        .get(&format!("{}/api/missions", get_api_url()))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<MissionsResponse>().await {
                    Ok(data) => {
                        if data.success {
                            Ok(data.missions)
                        } else {
                            Err("Failed to get missions".to_string())
                        }
                    }
                    Err(e) => Err(e.to_string()),
                }
            } else {
                Err(format!("Failed to get missions: {}", response.status()))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Accept a mission
pub async fn accept_mission(mission_id: i64) -> Result<bool, String> {
    let client = reqwest::Client::new();
    let token = get_auth_token();

    match client
        .post(&format!("{}/api/missions/{}/accept", get_api_url(), mission_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<MissionActionResponse>().await {
                    Ok(data) => Ok(data.success),
                    Err(e) => Err(e.to_string()),
                }
            } else {
                Err(format!("Failed to accept mission: {}", response.status()))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Update mission progress
pub async fn update_mission_progress(mission_id: i64, progress: i32) -> Result<bool, String> {
    let client = reqwest::Client::new();
    let token = get_auth_token();

    match client
        .put(&format!("{}/api/missions/{}/progress", get_api_url(), mission_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&ProgressUpdate { progress })
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<MissionActionResponse>().await {
                    Ok(data) => Ok(data.success),
                    Err(e) => Err(e.to_string()),
                }
            } else {
                Err(format!("Failed to update progress: {}", response.status()))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Get available missions from the game world
pub async fn get_available_missions() -> Result<Vec<MissionTemplate>, String> {
    let client = reqwest::Client::new();
    let token = get_auth_token();

    match client
        .get(&format!("{}/api/missions/available", get_api_url()))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                response.json().await.map_err(|e| e.to_string())
            } else {
                Err(format!("Failed to get available missions: {}", response.status()))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MissionTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub mission_type: String,
    pub difficulty: String,
    pub reward_money: i64,
    pub reward_xp: i32,
    pub reward_items: Vec<String>,
    pub requirements: Vec<String>,
    pub objectives: Vec<String>,
}

// Helper functions
fn get_api_url() -> String {
    std::env::var("API_URL").unwrap_or_else(|_| "http://localhost:3000".to_string())
}

fn get_auth_token() -> String {
    // Get from local storage or session
    "mock_token".to_string()
}