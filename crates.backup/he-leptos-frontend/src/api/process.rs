//! Process API client

use leptos::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    pub pid: i64,
    pub user_id: i64,
    pub process_type: String,
    pub state: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub source_server: String,
    pub target_server: Option<String>,
    pub priority: i32,
    pub cpu_usage: i32,
    pub ram_usage: i32,
    pub completion_percentage: f32,
    pub estimated_time_remaining: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateProcessRequest {
    pub process_type: String,
    pub target_server: Option<String>,
    pub priority: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProcessResponse {
    pub success: bool,
    pub process: Option<Process>,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProcessListResponse {
    pub processes: Vec<Process>,
}

/// Get all processes for the user
pub async fn get_processes() -> Result<Vec<Process>, String> {
    let client = reqwest::Client::new();
    let token = get_auth_token();

    match client
        .get(&format!("{}/api/process/list", get_api_url()))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<ProcessListResponse>().await {
                    Ok(data) => Ok(data.processes),
                    Err(e) => Err(e.to_string()),
                }
            } else {
                Err(format!("Failed to get processes: {}", response.status()))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Create a new process
pub async fn create_process(
    process_type: String,
    target_server: Option<String>,
    priority: i32,
) -> Result<Process, String> {
    let client = reqwest::Client::new();
    let token = get_auth_token();

    match client
        .post(&format!("{}/api/process/create", get_api_url()))
        .header("Authorization", format!("Bearer {}", token))
        .json(&CreateProcessRequest {
            process_type,
            target_server,
            priority,
        })
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<CreateProcessResponse>().await {
                    Ok(data) => {
                        if let Some(process) = data.process {
                            Ok(process)
                        } else {
                            Err(data.message)
                        }
                    }
                    Err(e) => Err(e.to_string()),
                }
            } else {
                Err(format!("Failed to create process: {}", response.status()))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Cancel a process
pub async fn cancel_process(pid: i64) -> Result<bool, String> {
    let client = reqwest::Client::new();
    let token = get_auth_token();

    match client
        .post(&format!("{}/api/process/{}/cancel", get_api_url(), pid))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                Ok(true)
            } else {
                Err(format!("Failed to cancel process: {}", response.status()))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Pause/resume a process
pub async fn toggle_process(pid: i64) -> Result<bool, String> {
    let client = reqwest::Client::new();
    let token = get_auth_token();

    match client
        .post(&format!("{}/api/process/{}/toggle", get_api_url(), pid))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                Ok(true)
            } else {
                Err(format!("Failed to toggle process: {}", response.status()))
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