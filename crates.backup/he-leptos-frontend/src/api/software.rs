//! Software API client

use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Software {
    pub id: i64,
    pub name: String,
    pub version: String,
    pub software_type: String,
    pub size: i32, // MB
    pub cpu_usage: i32,
    pub ram_usage: i32,
    pub effectiveness: i32,
    pub is_hidden: bool,
    pub is_installed: bool,
    pub location: String, // "local" or "external"
}

#[derive(Debug, Clone, Deserialize)]
pub struct SoftwareListResponse {
    pub local_software: Vec<Software>,
    pub external_software: Vec<Software>,
    pub local_used: i64,  // MB
    pub local_total: i64, // MB
    pub external_used: i64,
    pub external_total: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SoftwareActionRequest {
    pub software_id: i64,
    pub action: String, // "run", "install", "hide", "delete", "move"
    pub target: Option<String>, // For upload/move actions
}

#[derive(Debug, Clone, Deserialize)]
pub struct SoftwareActionResponse {
    pub success: bool,
    pub message: String,
    pub process_id: Option<i64>,
}

/// Get all software for the user
pub async fn get_software() -> Result<SoftwareListResponse, String> {
    let client = reqwest::Client::new();
    let token = get_auth_token();

    match client
        .get(&format!("{}/api/software/list", get_api_url()))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                response.json().await.map_err(|e| e.to_string())
            } else {
                Err(format!("Failed to get software: {}", response.status()))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Perform action on software
pub async fn software_action(
    software_id: i64,
    action: String,
    target: Option<String>,
) -> Result<SoftwareActionResponse, String> {
    let client = reqwest::Client::new();
    let token = get_auth_token();

    match client
        .post(&format!("{}/api/software/action", get_api_url()))
        .header("Authorization", format!("Bearer {}", token))
        .json(&SoftwareActionRequest {
            software_id,
            action,
            target,
        })
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

/// Download software from market/server
pub async fn download_software(
    software_name: String,
    version: String,
) -> Result<Software, String> {
    let client = reqwest::Client::new();
    let token = get_auth_token();

    match client
        .post(&format!("{}/api/software/download", get_api_url()))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "name": software_name,
            "version": version,
        }))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                response.json().await.map_err(|e| e.to_string())
            } else {
                Err(format!("Download failed: {}", response.status()))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Research new software version
pub async fn research_software(
    software_type: String,
    target_version: String,
) -> Result<i64, String> {
    let client = reqwest::Client::new();
    let token = get_auth_token();

    match client
        .post(&format!("{}/api/software/research", get_api_url()))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "software_type": software_type,
            "target_version": target_version,
        }))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        if let Some(process_id) = data.get("process_id").and_then(|v| v.as_i64()) {
                            Ok(process_id)
                        } else {
                            Err("No process ID returned".to_string())
                        }
                    }
                    Err(e) => Err(e.to_string()),
                }
            } else {
                Err(format!("Research failed: {}", response.status()))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

// Helper functions
fn get_api_url() -> String {
    std::env::var("API_URL").unwrap_or_else(|_| "http://localhost:3000".to_string())
}

fn get_auth_token() -> String {
    // Get from local storage or session
    "mock_token".to_string()
}