//! Process management handlers

use actix_web::{web, HttpResponse, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use he_database::queries::ProcessQueries;

#[derive(Serialize)]
pub struct ProcessResponse {
    pub success: bool,
    pub processes: Vec<ProcessInfo>,
}

#[derive(Serialize)]
pub struct ProcessInfo {
    pub pid: i64,
    pub process_type: String,
    pub pc_id: String,
    pub target_pc_id: Option<String>,
    pub start_time: String,
    pub end_time: String,
    pub priority: i32,
}

#[derive(Deserialize)]
pub struct CreateProcessRequest {
    pub process_type: String,
    pub target_pc_id: Option<String>,
}

pub async fn list_processes(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> HttpResponse {
    // Get user from token
    let user_id = match extract_user_id(&state, &req).await {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "success": false,
                "message": "Unauthorized"
            }));
        }
    };

    // Get user processes
    match ProcessQueries::get_user_processes(&state.db.pool, user_id).await {
        Ok(processes) => {
            let process_list: Vec<ProcessInfo> = processes.into_iter().map(|p| ProcessInfo {
                pid: p.pid,
                process_type: p.process_type,
                pc_id: p.pc_id,
                target_pc_id: p.target_pc_id,
                start_time: p.start_time.to_string(),
                end_time: p.end_time.to_string(),
                priority: p.priority,
            }).collect();

            HttpResponse::Ok().json(ProcessResponse {
                success: true,
                processes: process_list,
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("Failed to get processes: {}", e)
            }))
        }
    }
}

pub async fn create_process(
    state: web::Data<AppState>,
    req: HttpRequest,
    data: web::Json<CreateProcessRequest>,
) -> HttpResponse {
    let user_id = match extract_user_id(&state, &req).await {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "success": false,
                "message": "Unauthorized"
            }));
        }
    };

    // Get user's hardware specs to calculate process duration
    let hardware = match he_database::queries::HardwareQueries::get_user_hardware(&state.db.pool, user_id).await {
        Ok(hw) => hw,
        Err(_) => {
            // Use default hardware if not found
            he_database::models::Hardware {
                hardware_id: 0,
                user_id,
                cpu_mhz: 1000,
                ram_mb: 1024,
                hdd_mb: 10240,
                net_mbps: 10,
                gpu_cores: 1,
                total_slots: 5,
                used_slots: 1,
            }
        }
    };

    // Create player state for game mechanics
    let player_state = he_game_mechanics::PlayerState {
        user_id: user_id as i32,
        level: 1, // Get from user profile
        experience: 0,
        money: 0,
        reputation: std::collections::HashMap::new(),
        hardware_specs: he_game_mechanics::HardwareSpecs {
            cpu: hardware.cpu_mhz,
            ram: hardware.ram_mb,
            hdd: hardware.hdd_mb,
            net: hardware.net_mbps,
            security_level: 50,
            performance_rating: 100,
        },
        software_installed: Vec::new(),
        active_processes: Vec::new(),
        clan_membership: None,
        last_updated: chrono::Utc::now(),
    };

    // Create target info
    let target_info = he_game_mechanics::TargetInfo {
        ip_address: data.target_pc_id.clone().unwrap_or_else(|| "127.0.0.1".to_string()),
        target_type: "server".to_string(),
        difficulty_level: 50,
        security_rating: 50,
        reward_money: 1000,
        defense_systems: Vec::new(),
    };

    // Use game mechanics to calculate process time
    let game_engine = he_game_mechanics::GameEngine::new();
    let process_type = he_game_mechanics::process::ProcessType::from_str(&data.process_type);
    let duration = he_game_mechanics::process::calculate_duration(
        &data.process_type,
        &player_state,
        &target_info,
        &game_engine.config().process,
    );

    // Calculate resource usage
    let resource_usage = he_game_mechanics::process::calculate_resource_usage(
        &data.process_type,
        &target_info,
        &game_engine.config().process,
    );

    // Create process in database with calculated duration
    let pc_id = format!("pc_{}", user_id);
    let end_time = chrono::Utc::now() + chrono::Duration::seconds(duration as i64);

    match ProcessQueries::create_process_with_duration(
        &state.db.pool,
        user_id,
        &data.process_type,
        &pc_id,
        data.target_pc_id.clone(),
        duration,
    ).await {
        Ok(process) => {
            // Send WebSocket event about process start
            if let Some(ws_manager) = &state.ws_manager {
                let event = he_websocket::EventBuilder::process_started(
                    process.pid,
                    data.process_type.clone(),
                    duration as u64,
                );
                ws_manager.send_to_user(user_id, event.to_server_message());
            }

            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "pid": process.pid,
                "duration": duration,
                "cpu_usage": resource_usage.cpu_usage,
                "ram_usage": resource_usage.ram_usage,
                "net_usage": resource_usage.net_usage,
                "message": format!("Process {} started (ETA: {} seconds)", data.process_type, duration)
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("Failed to create process: {}", e)
            }))
        }
    }
}

pub async fn cancel_process(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<i64>,
) -> HttpResponse {
    let user_id = match extract_user_id(&state, &req).await {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "success": false,
                "message": "Unauthorized"
            }));
        }
    };

    let pid = path.into_inner();

    match ProcessQueries::cancel_process(&state.db.pool, pid, user_id).await {
        Ok(true) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": format!("Process {} cancelled", pid)
            }))
        }
        Ok(false) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "Process not found"
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("Failed to cancel process: {}", e)
            }))
        }
    }
}

// Helper function to extract user ID from JWT token
async fn extract_user_id(state: &web::Data<AppState>, req: &HttpRequest) -> Option<i64> {
    let token = req.headers()
        .get("Authorization")?
        .to_str().ok()?
        .strip_prefix("Bearer ")?;

    let validated = state.auth.validate_token(token).await.ok()??;

    // Convert UUID to i64 (simplified - in production you'd have proper mapping)
    Some(validated.user_id.as_u128() as i64)
}