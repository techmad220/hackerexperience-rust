//! Hacking API client

use leptos::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub struct ScanRequest {
    pub target_ip: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScanResponse {
    pub success: bool,
    pub server_info: Option<ServerInfo>,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerInfo {
    pub ip_address: String,
    pub hostname: String,
    pub owner: String,
    pub server_type: String,
    pub security_level: i32,
    pub firewall_level: i32,
    pub is_online: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct HackRequest {
    pub target_ip: String,
    pub crack_method: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HackResponse {
    pub success: bool,
    pub access_granted: bool,
    pub process_id: Option<i64>,
    pub estimated_time: Option<i32>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerActionRequest {
    pub target_ip: String,
    pub action: String,
    pub parameter: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerActionResponse {
    pub success: bool,
    pub data: Option<Value>,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InternetResponse {
    pub your_ip: String,
    pub known_servers: Vec<KnownServer>,
    pub recent_hacks: Vec<String>,
    pub bounties: Vec<BountyInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KnownServer {
    pub ip: String,
    pub hostname: String,
    pub last_seen: String,
    pub notes: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BountyInfo {
    pub corporation: String,
    pub target_ip: String,
    pub reward: i64,
    pub difficulty: String,
}

/// Scan a server
pub async fn scan_server(target_ip: String) -> Result<ScanResponse, String> {
    let client = reqwest::Client::new();
    let token = get_auth_token();

    match client
        .post(&format!("{}/api/hacking/scan", get_api_url()))
        .header("Authorization", format!("Bearer {}", token))
        .json(&ScanRequest { target_ip })
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                response.json().await.map_err(|e| e.to_string())
            } else {
                Err(format!("Scan failed: {}", response.status()))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Hack a server
pub async fn hack_server(target_ip: String, crack_method: String) -> Result<HackResponse, String> {
    let client = reqwest::Client::new();
    let token = get_auth_token();

    match client
        .post(&format!("{}/api/hacking/hack", get_api_url()))
        .header("Authorization", format!("Bearer {}", token))
        .json(&HackRequest { target_ip, crack_method })
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                response.json().await.map_err(|e| e.to_string())
            } else {
                Err(format!("Hack failed: {}", response.status()))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Perform action on server
pub async fn server_action(
    target_ip: String,
    action: String,
    parameter: Option<String>
) -> Result<ServerActionResponse, String> {
    let client = reqwest::Client::new();
    let token = get_auth_token();

    match client
        .post(&format!("{}/api/hacking/action", get_api_url()))
        .header("Authorization", format!("Bearer {}", token))
        .json(&ServerActionRequest { target_ip, action, parameter })
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                response.json().await.map_err(|e| e.to_string())
            } else {
                Err(format!("Action failed: {}", response.status()))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Get internet view
pub async fn get_internet_view() -> Result<InternetResponse, String> {
    let client = reqwest::Client::new();
    let token = get_auth_token();

    match client
        .get(&format!("{}/api/hacking/internet", get_api_url()))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                response.json().await.map_err(|e| e.to_string())
            } else {
                Err(format!("Failed to get internet view: {}", response.status()))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

// Helper functions
fn get_api_url() -> String {
    // Get from environment or use default
    std::env::var("API_URL").unwrap_or_else(|_| "http://localhost:3000".to_string())
}

fn get_auth_token() -> String {
    // Get from local storage or session
    // For now, return mock token
    "mock_token".to_string()
}