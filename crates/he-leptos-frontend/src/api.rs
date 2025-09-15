use serde::{Deserialize, Serialize};
use leptos::*;

const API_BASE_URL: &str = "http://127.0.0.1:8080";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: u64,
    pub username: String,
    pub reputation: i32,
    pub money: u64,
    pub bitcoin: f64,
    pub level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hardware {
    pub processor: String,
    pub hard_drive: String,
    pub memory: String,
    pub internet: String,
    pub external_hd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub uptime: String,
    pub running_tasks: u32,
    pub connections: u32,
    pub mission: Option<String>,
    pub clan: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameData {
    pub user: UserProfile,
    pub hardware: Hardware,
    pub system_info: SystemInfo,
}

pub async fn fetch_user_profile() -> Result<UserProfile, String> {
    let response = reqwest::get(&format!("{}/user/profile", API_BASE_URL))
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if response.status().is_success() {
        response.json::<UserProfile>()
            .await
            .map_err(|e| format!("JSON parsing error: {}", e))
    } else {
        Err(format!("API error: {}", response.status()))
    }
}

pub async fn fetch_hardware_info() -> Result<Hardware, String> {
    let response = reqwest::get(&format!("{}/hardware/owned", API_BASE_URL))
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if response.status().is_success() {
        response.json::<Hardware>()
            .await
            .map_err(|e| format!("JSON parsing error: {}", e))
    } else {
        // Return mock data for now if API fails
        Ok(Hardware {
            processor: "AMD K6-2 500MHz".to_string(),
            hard_drive: "10 GB Maxtor".to_string(),
            memory: "128 MB".to_string(),
            internet: "Modem 56K".to_string(),
            external_hd: None,
        })
    }
}

pub async fn fetch_system_info() -> Result<SystemInfo, String> {
    let response = reqwest::get(&format!("{}/user/stats", API_BASE_URL))
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if response.status().is_success() {
        response.json::<SystemInfo>()
            .await
            .map_err(|e| format!("JSON parsing error: {}", e))
    } else {
        // Return mock data for now if API fails
        Ok(SystemInfo {
            uptime: "42 minutes".to_string(),
            running_tasks: 0,
            connections: 0,
            mission: None,
            clan: None,
        })
    }
}

pub async fn fetch_game_data() -> Result<GameData, String> {
    // For now, return mock data but structured like real API responses
    Ok(GameData {
        user: UserProfile {
            id: 1,
            username: "Techmad".to_string(),
            reputation: 0,
            money: 1000,
            bitcoin: 0.0,
            level: 1,
        },
        hardware: fetch_hardware_info().await?,
        system_info: fetch_system_info().await?,
    })
}

pub fn create_api_resource() -> Resource<(), Result<GameData, String>> {
    create_resource(|| (), |_| async move {
        fetch_game_data().await
    })
}