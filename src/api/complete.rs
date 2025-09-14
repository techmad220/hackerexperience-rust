use axum::{
    extract::{Path, Query, State, WebSocketUpgrade, ws::{Message, WebSocket}},
    http::StatusCode,
    response::{Json, Response},
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::CorsLayer;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::{Result, anyhow};

use crate::database::queries::*;

// Application state
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub websocket_tx: broadcast::Sender<WebSocketMessage>,
    pub active_connections: Arc<RwLock<HashMap<i64, WebSocketConnection>>>,
}

#[derive(Clone)]
pub struct WebSocketConnection {
    pub user_id: i64,
    pub session_id: String,
    pub connected_at: DateTime<Utc>,
}

// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub user_id: Option<i64>,
    pub message_type: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

// API Response types
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            error: None,
        }
    }
    
    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            message: None,
            error: Some(message.to_string()),
        }
    }
}

// Request/Response DTOs
#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub user: UserProfile,
    pub token: String,
}

#[derive(Serialize)]
pub struct UserProfile {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub money: i64,
    pub bitcoin: f64,
    pub experience: i64,
    pub reputation: f64,
    pub total_cpu: i64,
    pub total_ram: i64,
    pub total_hdd: i64,
    pub total_net: i64,
    pub clan_id: Option<i64>,
    pub is_online: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct StartProcessRequest {
    pub action: String,
    pub target_ip: String,
    pub software_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct ConnectServerRequest {
    pub ip: String,
}

#[derive(Deserialize)]
pub struct PurchaseRequest {
    pub software_id: Option<i64>,
    pub hardware_type: Option<String>,
    pub spec_value: Option<i64>,
    pub price: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateClanRequest {
    pub name: String,
    pub tag: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct ChatMessageRequest {
    pub message: String,
}

// Create the complete API router
pub fn create_api_router() -> Router<AppState> {
    Router::new()
        // Authentication endpoints
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(get_current_user))
        .route("/auth/register", post(register))
        
        // User endpoints
        .route("/user/profile", get(get_user_profile))
        .route("/user/update", put(update_user_profile))
        .route("/user/stats", get(get_user_stats))
        
        // Server endpoints
        .route("/servers/available", get(get_available_servers))
        .route("/servers/owned", get(get_owned_servers))
        .route("/servers/connect", post(connect_to_server))
        .route("/servers/:id", get(get_server_details))
        .route("/servers/:id/files", get(get_server_files))
        .route("/servers/:id/logs", get(get_server_logs))
        
        // Software endpoints
        .route("/software/installed", get(get_installed_software))
        .route("/software/:id/start", post(start_software))
        .route("/software/:id/stop", post(stop_software))
        .route("/software/:id/uninstall", post(uninstall_software))
        
        // Process endpoints
        .route("/processes/start", post(start_process))
        .route("/processes/active", get(get_active_processes))
        .route("/processes/:id", get(get_process_details))
        .route("/processes/:id/kill", post(kill_process))
        .route("/processes/kill-all", post(kill_all_processes))
        
        // Hardware endpoints
        .route("/hardware/owned", get(get_owned_hardware))
        .route("/hardware/upgrade", post(upgrade_hardware))
        
        // File endpoints
        .route("/files/list", get(get_file_list))
        .route("/files/create", post(create_file))
        .route("/files/:id/delete", delete(delete_file))
        .route("/files/:id/download", get(download_file))
        
        // Log endpoints
        .route("/logs/recent", get(get_recent_logs))
        .route("/logs/clear", post(clear_logs))
        
        // Mission endpoints
        .route("/missions/active", get(get_active_missions))
        .route("/missions/:id/complete", post(complete_mission))
        .route("/missions/:id/abandon", post(abandon_mission))
        
        // Clan endpoints
        .route("/clan/info", get(get_clan_info))
        .route("/clan/create", post(create_clan))
        .route("/clan/join", post(join_clan))
        .route("/clan/leave", post(leave_clan))
        .route("/clan/members", get(get_clan_members))
        
        // Ranking endpoints
        .route("/rankings/top", get(get_top_rankings))
        .route("/rankings/clans", get(get_clan_rankings))
        
        // Store endpoints
        .route("/store/software", get(get_software_store))
        .route("/store/hardware", get(get_hardware_store))
        .route("/store/purchase-software", post(purchase_software))
        .route("/store/purchase-hardware", post(purchase_hardware))
        
        // Network/Internet endpoints
        .route("/network/scan", post(scan_network))
        .route("/network/trace", post(trace_route))
        
        // Chat/Communication endpoints
        .route("/chat/history", get(get_chat_history))
        .route("/chat/send", post(send_chat_message))
        
        // Game mechanics endpoints
        .route("/game/tick", post(game_tick))
        .route("/game/stats", get(get_game_stats))
        
        // Admin endpoints (protected)
        .route("/admin/users", get(admin_get_users))
        .route("/admin/servers", get(admin_get_servers))
        .route("/admin/processes", get(admin_get_processes))
        
        // WebSocket endpoint
        .route("/ws", get(websocket_handler))
        
        // Add CORS support
        .layer(CorsLayer::permissive())
}

// Authentication handlers
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, StatusCode> {
    match authenticate_user(&state.db, &request.username, &request.password).await {
        Ok(user) => {
            let token = generate_jwt_token(user.id).await;
            
            // Update user online status
            let _ = UserQueries::update_user_online_status(&state.db, user.id, true).await;
            
            let response = LoginResponse {
                user: map_user_to_profile(user),
                token,
            };
            
            Ok(Json(ApiResponse::success(response)))
        }
        Err(_) => Ok(Json(ApiResponse::error("Invalid credentials"))),
    }
}

pub async fn logout(State(state): State<AppState>) -> Json<ApiResponse<()>> {
    // Extract user from JWT token (implementation depends on auth middleware)
    // For now, returning success
    Json(ApiResponse::success(()))
}

pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<UserProfile>>, StatusCode> {
    // Validate request
    if request.username.len() < 3 || request.password.len() < 6 {
        return Ok(Json(ApiResponse::error("Invalid username or password")));
    }
    
    // Check if username exists
    match UserQueries::get_user_by_username(&state.db, &request.username).await {
        Ok(Some(_)) => Ok(Json(ApiResponse::error("Username already exists"))),
        Ok(None) => {
            // Hash password
            let password_hash = hash_password(&request.password).await;
            
            // Create user
            match UserQueries::create_user(&state.db, &request.username, &request.email, &password_hash).await {
                Ok(user) => {
                    // Create initial hardware
                    let _ = create_initial_hardware(&state.db, user.id).await;
                    
                    Ok(Json(ApiResponse::success(map_user_to_profile(user))))
                }
                Err(_) => Ok(Json(ApiResponse::error("Failed to create user"))),
            }
        }
        Err(_) => Ok(Json(ApiResponse::error("Database error"))),
    }
}

pub async fn get_current_user(State(state): State<AppState>) -> Json<ApiResponse<UserProfile>> {
    // Extract user ID from JWT token (implementation depends on auth middleware)
    let user_id = 1; // Placeholder
    
    match UserQueries::get_user_by_id(&state.db, user_id).await {
        Ok(Some(user)) => Json(ApiResponse::success(map_user_to_profile(user))),
        Ok(None) => Json(ApiResponse::error("User not found")),
        Err(_) => Json(ApiResponse::error("Database error")),
    }
}

// User handlers
pub async fn get_user_profile(State(state): State<AppState>) -> Json<ApiResponse<UserProfile>> {
    let user_id = 1; // Extract from JWT
    
    match UserQueries::get_user_by_id(&state.db, user_id).await {
        Ok(Some(user)) => Json(ApiResponse::success(map_user_to_profile(user))),
        Ok(None) => Json(ApiResponse::error("User not found")),
        Err(_) => Json(ApiResponse::error("Database error")),
    }
}

pub async fn update_user_profile(
    State(state): State<AppState>,
    Json(request): Json<UpdateUserRequest>,
) -> Json<ApiResponse<()>> {
    let user_id = 1; // Extract from JWT
    
    // Update user profile logic here
    Json(ApiResponse::success(()))
}

pub async fn get_user_stats(State(state): State<AppState>) -> Json<ApiResponse<UserStats>> {
    let user_id = 1; // Extract from JWT
    
    // Collect comprehensive user stats
    let stats = UserStats {
        total_hacks: 0, // Query from database
        successful_hacks: 0,
        money_earned: 0,
        experience: 0,
        reputation: 0.0,
        rank: 0,
        uptime: 0,
        servers_owned: 0,
        processes_completed: 0,
    };
    
    Json(ApiResponse::success(stats))
}

// Server handlers
pub async fn get_available_servers(State(state): State<AppState>) -> Json<ApiResponse<Vec<DatabaseServer>>> {
    match ServerQueries::get_hackable_servers(&state.db, 50).await {
        Ok(servers) => Json(ApiResponse::success(servers)),
        Err(_) => Json(ApiResponse::error("Failed to load servers")),
    }
}

pub async fn get_owned_servers(State(state): State<AppState>) -> Json<ApiResponse<Vec<DatabaseServer>>> {
    let user_id = 1; // Extract from JWT
    
    match ServerQueries::get_servers_by_owner(&state.db, user_id).await {
        Ok(servers) => Json(ApiResponse::success(servers)),
        Err(_) => Json(ApiResponse::error("Failed to load servers")),
    }
}

pub async fn connect_to_server(
    State(state): State<AppState>,
    Json(request): Json<ConnectServerRequest>,
) -> Json<ApiResponse<DatabaseServer>> {
    match ServerQueries::get_server_by_ip(&state.db, &request.ip).await {
        Ok(Some(server)) => {
            // Check if user has access to this server
            // This would involve checking hacking status, passwords, etc.
            Json(ApiResponse::success(server))
        }
        Ok(None) => Json(ApiResponse::error("Server not found")),
        Err(_) => Json(ApiResponse::error("Connection failed")),
    }
}

pub async fn get_server_details(
    State(state): State<AppState>,
    Path(server_id): Path<i64>,
) -> Json<ApiResponse<ServerDetails>> {
    match ServerQueries::get_server_by_id(&state.db, server_id).await {
        Ok(Some(server)) => {
            // Get additional server details
            let files = FileQueries::get_files_by_server(&state.db, server_id).await.unwrap_or_default();
            let software = SoftwareQueries::get_installed_software(&state.db, server_id).await.unwrap_or_default();
            
            let details = ServerDetails {
                server,
                files,
                installed_software: software,
                is_hackable: true, // Determine based on security measures
                estimated_hack_time: 300, // Calculate based on security vs user skill
            };
            
            Json(ApiResponse::success(details))
        }
        Ok(None) => Json(ApiResponse::error("Server not found")),
        Err(_) => Json(ApiResponse::error("Database error")),
    }
}

pub async fn get_server_files(
    State(state): State<AppState>,
    Path(server_id): Path<i64>,
) -> Json<ApiResponse<Vec<DatabaseFile>>> {
    match FileQueries::get_files_by_server(&state.db, server_id).await {
        Ok(files) => Json(ApiResponse::success(files)),
        Err(_) => Json(ApiResponse::error("Failed to load files")),
    }
}

pub async fn get_server_logs(
    State(state): State<AppState>,
    Path(server_id): Path<i64>,
) -> Json<ApiResponse<Vec<DatabaseLog>>> {
    match LogQueries::get_logs_by_server(&state.db, server_id).await {
        Ok(logs) => Json(ApiResponse::success(logs)),
        Err(_) => Json(ApiResponse::error("Failed to load logs")),
    }
}

// Software handlers
pub async fn get_installed_software(State(state): State<AppState>) -> Json<ApiResponse<Vec<InstalledSoftwareInfo>>> {
    let user_id = 1; // Extract from JWT
    
    // Get user's main server (PC)
    match get_user_main_server(&state.db, user_id).await {
        Ok(server_id) => {
            match SoftwareQueries::get_installed_software(&state.db, server_id).await {
                Ok(software_list) => {
                    let mut software_info = Vec::new();
                    
                    for installed in software_list {
                        if let Ok(Some(software)) = SoftwareQueries::get_software_by_id(&state.db, installed.software_id).await {
                            software_info.push(InstalledSoftwareInfo {
                                id: installed.id,
                                software_id: software.id,
                                name: software.name,
                                software_type: software.software_type,
                                version: installed.version,
                                is_running: installed.is_running,
                                installed_at: installed.installed_at,
                            });
                        }
                    }
                    
                    Json(ApiResponse::success(software_info))
                }
                Err(_) => Json(ApiResponse::error("Failed to load software")),
            }
        }
        Err(_) => Json(ApiResponse::error("User server not found")),
    }
}

pub async fn start_software(
    State(state): State<AppState>,
    Path(software_id): Path<i64>,
) -> Json<ApiResponse<()>> {
    match SoftwareQueries::start_software(&state.db, software_id).await {
        Ok(()) => Json(ApiResponse::success(())),
        Err(_) => Json(ApiResponse::error("Failed to start software")),
    }
}

pub async fn stop_software(
    State(state): State<AppState>,
    Path(software_id): Path<i64>,
) -> Json<ApiResponse<()>> {
    match SoftwareQueries::stop_software(&state.db, software_id).await {
        Ok(()) => Json(ApiResponse::success(())),
        Err(_) => Json(ApiResponse::error("Failed to stop software")),
    }
}

pub async fn uninstall_software(
    State(state): State<AppState>,
    Path(software_id): Path<i64>,
) -> Json<ApiResponse<()>> {
    match SoftwareQueries::uninstall_software(&state.db, software_id).await {
        Ok(()) => Json(ApiResponse::success(())),
        Err(_) => Json(ApiResponse::error("Failed to uninstall software")),
    }
}

// Process handlers
pub async fn start_process(
    State(state): State<AppState>,
    Json(request): Json<StartProcessRequest>,
) -> Json<ApiResponse<DatabaseProcess>> {
    let user_id = 1; // Extract from JWT
    
    // Calculate process parameters based on action and software
    let (duration, cpu_usage, net_usage) = calculate_process_parameters(
        &request.action,
        request.software_id,
        &state.db,
        user_id,
    ).await;
    
    // Get user's IP (their main PC)
    let source_ip = get_user_ip(&state.db, user_id).await.unwrap_or_else(|_| "127.0.0.1".to_string());
    
    match ProcessQueries::create_process(
        &state.db,
        user_id,
        None, // victim_id determined later
        &source_ip,
        &request.target_ip,
        &request.action,
        request.software_id,
        duration,
        cpu_usage,
        net_usage,
    ).await {
        Ok(mut process) => {
            // Start the process
            if let Ok(()) = ProcessQueries::start_process(&state.db, process.id).await {
                process.status = "running".to_string();
                
                // Broadcast process start
                let _ = state.websocket_tx.send(WebSocketMessage {
                    user_id: Some(user_id),
                    message_type: "process_start".to_string(),
                    data: serde_json::to_value(&process).unwrap_or_default(),
                    timestamp: Utc::now(),
                });
                
                Json(ApiResponse::success(process))
            } else {
                Json(ApiResponse::error("Failed to start process"))
            }
        }
        Err(_) => Json(ApiResponse::error("Failed to create process")),
    }
}

pub async fn get_active_processes(State(state): State<AppState>) -> Json<ApiResponse<Vec<DatabaseProcess>>> {
    let user_id = 1; // Extract from JWT
    
    match ProcessQueries::get_processes_by_creator(&state.db, user_id).await {
        Ok(processes) => {
            let active_processes: Vec<DatabaseProcess> = processes
                .into_iter()
                .filter(|p| p.status == "running" || p.status == "pending")
                .collect();
            
            Json(ApiResponse::success(active_processes))
        }
        Err(_) => Json(ApiResponse::error("Failed to load processes")),
    }
}

pub async fn get_process_details(
    State(state): State<AppState>,
    Path(process_id): Path<i64>,
) -> Json<ApiResponse<ProcessDetails>> {
    match ProcessQueries::get_process_by_id(&state.db, process_id).await {
        Ok(Some(process)) => {
            let software = if let Some(software_id) = process.software_id {
                SoftwareQueries::get_software_by_id(&state.db, software_id).await.ok().flatten()
            } else {
                None
            };
            
            let details = ProcessDetails {
                process,
                software,
            };
            
            Json(ApiResponse::success(details))
        }
        Ok(None) => Json(ApiResponse::error("Process not found")),
        Err(_) => Json(ApiResponse::error("Database error")),
    }
}

pub async fn kill_process(
    State(state): State<AppState>,
    Path(process_id): Path<i64>,
) -> Json<ApiResponse<()>> {
    let user_id = 1; // Extract from JWT
    
    // Verify user owns this process
    match ProcessQueries::get_process_by_id(&state.db, process_id).await {
        Ok(Some(process)) if process.creator_id == user_id => {
            match ProcessQueries::cancel_process(&state.db, process_id).await {
                Ok(()) => {
                    // Broadcast process kill
                    let _ = state.websocket_tx.send(WebSocketMessage {
                        user_id: Some(user_id),
                        message_type: "process_killed".to_string(),
                        data: serde_json::json!({"process_id": process_id}),
                        timestamp: Utc::now(),
                    });
                    
                    Json(ApiResponse::success(()))
                }
                Err(_) => Json(ApiResponse::error("Failed to kill process")),
            }
        }
        Ok(Some(_)) => Json(ApiResponse::error("Access denied")),
        Ok(None) => Json(ApiResponse::error("Process not found")),
        Err(_) => Json(ApiResponse::error("Database error")),
    }
}

pub async fn kill_all_processes(State(state): State<AppState>) -> Json<ApiResponse<()>> {
    let user_id = 1; // Extract from JWT
    
    match ProcessQueries::get_processes_by_creator(&state.db, user_id).await {
        Ok(processes) => {
            let mut killed_count = 0;
            
            for process in processes {
                if process.status == "running" || process.status == "pending" {
                    if let Ok(()) = ProcessQueries::cancel_process(&state.db, process.id).await {
                        killed_count += 1;
                    }
                }
            }
            
            // Broadcast mass process kill
            let _ = state.websocket_tx.send(WebSocketMessage {
                user_id: Some(user_id),
                message_type: "all_processes_killed".to_string(),
                data: serde_json::json!({"count": killed_count}),
                timestamp: Utc::now(),
            });
            
            Json(ApiResponse::success(()))
        }
        Err(_) => Json(ApiResponse::error("Failed to kill processes")),
    }
}

// Hardware handlers
pub async fn get_owned_hardware(State(state): State<AppState>) -> Json<ApiResponse<Vec<DatabaseHardware>>> {
    let user_id = 1; // Extract from JWT
    
    match HardwareQueries::get_hardware_by_owner(&state.db, user_id).await {
        Ok(hardware) => Json(ApiResponse::success(hardware)),
        Err(_) => Json(ApiResponse::error("Failed to load hardware")),
    }
}

pub async fn upgrade_hardware(
    State(state): State<AppState>,
    Json(request): Json<UpgradeHardwareRequest>,
) -> Json<ApiResponse<()>> {
    let user_id = 1; // Extract from JWT
    
    // Implement hardware upgrade logic
    Json(ApiResponse::success(()))
}

// File handlers
pub async fn get_file_list(State(state): State<AppState>) -> Json<ApiResponse<Vec<DatabaseFile>>> {
    let user_id = 1; // Extract from JWT
    
    match get_user_main_server(&state.db, user_id).await {
        Ok(server_id) => {
            match FileQueries::get_files_by_server(&state.db, server_id).await {
                Ok(files) => Json(ApiResponse::success(files)),
                Err(_) => Json(ApiResponse::error("Failed to load files")),
            }
        }
        Err(_) => Json(ApiResponse::error("User server not found")),
    }
}

pub async fn create_file(
    State(state): State<AppState>,
    Json(request): Json<CreateFileRequest>,
) -> Json<ApiResponse<DatabaseFile>> {
    let user_id = 1; // Extract from JWT
    
    match get_user_main_server(&state.db, user_id).await {
        Ok(server_id) => {
            match FileQueries::create_file(
                &state.db,
                server_id,
                &request.name,
                &request.file_type,
                request.size,
                &request.path,
                request.is_hidden.unwrap_or(false),
            ).await {
                Ok(file) => Json(ApiResponse::success(file)),
                Err(_) => Json(ApiResponse::error("Failed to create file")),
            }
        }
        Err(_) => Json(ApiResponse::error("User server not found")),
    }
}

pub async fn delete_file(
    State(state): State<AppState>,
    Path(file_id): Path<i64>,
) -> Json<ApiResponse<()>> {
    match FileQueries::delete_file(&state.db, file_id).await {
        Ok(()) => Json(ApiResponse::success(())),
        Err(_) => Json(ApiResponse::error("Failed to delete file")),
    }
}

pub async fn download_file(
    State(state): State<AppState>,
    Path(file_id): Path<i64>,
) -> Json<ApiResponse<FileContent>> {
    // Implement file download logic
    // This would typically return file content or a download URL
    Json(ApiResponse::error("Not implemented"))
}

// Log handlers
pub async fn get_recent_logs(State(state): State<AppState>) -> Json<ApiResponse<Vec<DatabaseLog>>> {
    let user_id = 1; // Extract from JWT
    
    match get_user_main_server(&state.db, user_id).await {
        Ok(server_id) => {
            match LogQueries::get_logs_by_server(&state.db, server_id).await {
                Ok(logs) => Json(ApiResponse::success(logs)),
                Err(_) => Json(ApiResponse::error("Failed to load logs")),
            }
        }
        Err(_) => Json(ApiResponse::error("User server not found")),
    }
}

pub async fn clear_logs(State(state): State<AppState>) -> Json<ApiResponse<()>> {
    let user_id = 1; // Extract from JWT
    
    match get_user_main_server(&state.db, user_id).await {
        Ok(server_id) => {
            match LogQueries::clear_server_logs(&state.db, server_id).await {
                Ok(_) => Json(ApiResponse::success(())),
                Err(_) => Json(ApiResponse::error("Failed to clear logs")),
            }
        }
        Err(_) => Json(ApiResponse::error("User server not found")),
    }
}

// Mission handlers
pub async fn get_active_missions(State(state): State<AppState>) -> Json<ApiResponse<Vec<DatabaseMission>>> {
    let user_id = 1; // Extract from JWT
    
    match MissionQueries::get_active_missions(&state.db, user_id).await {
        Ok(missions) => Json(ApiResponse::success(missions)),
        Err(_) => Json(ApiResponse::error("Failed to load missions")),
    }
}

pub async fn complete_mission(
    State(state): State<AppState>,
    Path(mission_id): Path<i64>,
) -> Json<ApiResponse<MissionResult>> {
    let user_id = 1; // Extract from JWT
    
    // Verify user owns this mission and complete it
    match MissionQueries::complete_mission(&state.db, mission_id).await {
        Ok(()) => {
            // Award rewards (implementation needed)
            let result = MissionResult {
                success: true,
                reward_money: 1000,
                reward_experience: 100,
                message: "Mission completed successfully!".to_string(),
            };
            
            Json(ApiResponse::success(result))
        }
        Err(_) => Json(ApiResponse::error("Failed to complete mission")),
    }
}

pub async fn abandon_mission(
    State(state): State<AppState>,
    Path(mission_id): Path<i64>,
) -> Json<ApiResponse<()>> {
    match MissionQueries::fail_mission(&state.db, mission_id).await {
        Ok(()) => Json(ApiResponse::success(())),
        Err(_) => Json(ApiResponse::error("Failed to abandon mission")),
    }
}

// Clan handlers
pub async fn get_clan_info(State(state): State<AppState>) -> Json<ApiResponse<ClanInfo>> {
    let user_id = 1; // Extract from JWT
    
    match UserQueries::get_user_by_id(&state.db, user_id).await {
        Ok(Some(user)) if user.clan_id.is_some() => {
            let clan_id = user.clan_id.unwrap();
            
            match ClanQueries::get_clan_by_id(&state.db, clan_id).await {
                Ok(Some(clan)) => {
                    let members = ClanQueries::get_clan_members(&state.db, clan_id).await.unwrap_or_default();
                    
                    let clan_info = ClanInfo {
                        clan,
                        members,
                    };
                    
                    Json(ApiResponse::success(clan_info))
                }
                Ok(None) => Json(ApiResponse::error("Clan not found")),
                Err(_) => Json(ApiResponse::error("Database error")),
            }
        }
        Ok(Some(_)) => Json(ApiResponse::error("You are not in a clan")),
        Ok(None) => Json(ApiResponse::error("User not found")),
        Err(_) => Json(ApiResponse::error("Database error")),
    }
}

pub async fn create_clan(
    State(state): State<AppState>,
    Json(request): Json<CreateClanRequest>,
) -> Json<ApiResponse<DatabaseClan>> {
    let user_id = 1; // Extract from JWT
    
    // Check if user is already in a clan
    match UserQueries::get_user_by_id(&state.db, user_id).await {
        Ok(Some(user)) if user.clan_id.is_none() => {
            match ClanQueries::create_clan(&state.db, &request.name, &request.tag, &request.description, user_id).await {
                Ok(clan) => {
                    // Add user to the clan
                    let _ = UserQueries::join_clan(&state.db, user_id, clan.id).await;
                    
                    Json(ApiResponse::success(clan))
                }
                Err(_) => Json(ApiResponse::error("Failed to create clan")),
            }
        }
        Ok(Some(_)) => Json(ApiResponse::error("You are already in a clan")),
        Ok(None) => Json(ApiResponse::error("User not found")),
        Err(_) => Json(ApiResponse::error("Database error")),
    }
}

pub async fn join_clan(
    State(state): State<AppState>,
    Json(request): Json<JoinClanRequest>,
) -> Json<ApiResponse<()>> {
    let user_id = 1; // Extract from JWT
    
    match UserQueries::join_clan(&state.db, user_id, request.clan_id).await {
        Ok(()) => {
            // Update clan member count
            let _ = ClanQueries::update_member_count(&state.db, request.clan_id).await;
            
            Json(ApiResponse::success(()))
        }
        Err(_) => Json(ApiResponse::error("Failed to join clan")),
    }
}

pub async fn leave_clan(State(state): State<AppState>) -> Json<ApiResponse<()>> {
    let user_id = 1; // Extract from JWT
    
    match UserQueries::get_user_by_id(&state.db, user_id).await {
        Ok(Some(user)) if user.clan_id.is_some() => {
            let clan_id = user.clan_id.unwrap();
            
            match UserQueries::leave_clan(&state.db, user_id).await {
                Ok(()) => {
                    // Update clan member count
                    let _ = ClanQueries::update_member_count(&state.db, clan_id).await;
                    
                    Json(ApiResponse::success(()))
                }
                Err(_) => Json(ApiResponse::error("Failed to leave clan")),
            }
        }
        Ok(Some(_)) => Json(ApiResponse::error("You are not in a clan")),
        Ok(None) => Json(ApiResponse::error("User not found")),
        Err(_) => Json(ApiResponse::error("Database error")),
    }
}

pub async fn get_clan_members(State(state): State<AppState>) -> Json<ApiResponse<Vec<DatabaseUser>>> {
    let user_id = 1; // Extract from JWT
    
    match UserQueries::get_user_by_id(&state.db, user_id).await {
        Ok(Some(user)) if user.clan_id.is_some() => {
            let clan_id = user.clan_id.unwrap();
            
            match ClanQueries::get_clan_members(&state.db, clan_id).await {
                Ok(members) => Json(ApiResponse::success(members)),
                Err(_) => Json(ApiResponse::error("Failed to load clan members")),
            }
        }
        Ok(Some(_)) => Json(ApiResponse::error("You are not in a clan")),
        Ok(None) => Json(ApiResponse::error("User not found")),
        Err(_) => Json(ApiResponse::error("Database error")),
    }
}

// Ranking handlers
pub async fn get_top_rankings(State(state): State<AppState>) -> Json<ApiResponse<Vec<DatabaseUser>>> {
    match UserQueries::get_user_ranking(&state.db, 100).await {
        Ok(users) => Json(ApiResponse::success(users)),
        Err(_) => Json(ApiResponse::error("Failed to load rankings")),
    }
}

pub async fn get_clan_rankings(State(state): State<AppState>) -> Json<ApiResponse<Vec<DatabaseClan>>> {
    match ClanQueries::get_top_clans(&state.db, 50).await {
        Ok(clans) => Json(ApiResponse::success(clans)),
        Err(_) => Json(ApiResponse::error("Failed to load clan rankings")),
    }
}

// Store handlers
pub async fn get_software_store(State(state): State<AppState>) -> Json<ApiResponse<Vec<DatabaseSoftware>>> {
    match SoftwareQueries::get_public_software(&state.db).await {
        Ok(software) => Json(ApiResponse::success(software)),
        Err(_) => Json(ApiResponse::error("Failed to load software store")),
    }
}

pub async fn get_hardware_store(State(state): State<AppState>) -> Json<ApiResponse<Vec<HardwareStoreItem>>> {
    // Generate hardware store items
    let hardware_items = generate_hardware_store_items();
    Json(ApiResponse::success(hardware_items))
}

pub async fn purchase_software(
    State(state): State<AppState>,
    Json(request): Json<PurchaseRequest>,
) -> Json<ApiResponse<()>> {
    let user_id = 1; // Extract from JWT
    
    if let Some(software_id) = request.software_id {
        // Get software details
        match SoftwareQueries::get_software_by_id(&state.db, software_id).await {
            Ok(Some(software)) => {
                // Check if user has enough money
                match UserQueries::get_user_by_id(&state.db, user_id).await {
                    Ok(Some(user)) if user.money >= software.price => {
                        // Deduct money and install software
                        match TransactionQueries::purchase_software(&state.db, user_id, software_id, software.price).await {
                            Ok(()) => Json(ApiResponse::success(())),
                            Err(_) => Json(ApiResponse::error("Transaction failed")),
                        }
                    }
                    Ok(Some(_)) => Json(ApiResponse::error("Insufficient funds")),
                    Ok(None) => Json(ApiResponse::error("User not found")),
                    Err(_) => Json(ApiResponse::error("Database error")),
                }
            }
            Ok(None) => Json(ApiResponse::error("Software not found")),
            Err(_) => Json(ApiResponse::error("Database error")),
        }
    } else {
        Json(ApiResponse::error("Invalid purchase request"))
    }
}

pub async fn purchase_hardware(
    State(state): State<AppState>,
    Json(request): Json<PurchaseRequest>,
) -> Json<ApiResponse<()>> {
    let user_id = 1; // Extract from JWT
    
    if let (Some(hardware_type), Some(spec_value), Some(price)) = 
        (&request.hardware_type, request.spec_value, request.price) {
        
        match TransactionQueries::purchase_hardware(&state.db, user_id, hardware_type, spec_value, price).await {
            Ok(_) => Json(ApiResponse::success(())),
            Err(_) => Json(ApiResponse::error("Purchase failed")),
        }
    } else {
        Json(ApiResponse::error("Invalid purchase request"))
    }
}

// Network handlers
pub async fn scan_network(State(state): State<AppState>) -> Json<ApiResponse<Vec<NetworkScanResult>>> {
    let user_id = 1; // Extract from JWT
    
    // Implement network scanning logic
    let scan_results = vec![
        NetworkScanResult {
            ip: "192.168.1.1".to_string(),
            hostname: Some("router.local".to_string()),
            open_ports: vec![80, 443, 22],
            os: Some("Linux".to_string()),
            response_time: 15,
        },
        NetworkScanResult {
            ip: "192.168.1.100".to_string(),
            hostname: Some("target-server".to_string()),
            open_ports: vec![80, 21, 22, 3389],
            os: Some("Windows".to_string()),
            response_time: 25,
        },
    ];
    
    Json(ApiResponse::success(scan_results))
}

pub async fn trace_route(
    State(state): State<AppState>,
    Json(request): Json<TraceRouteRequest>,
) -> Json<ApiResponse<Vec<TraceHop>>> {
    // Implement traceroute logic
    let hops = vec![
        TraceHop {
            hop_number: 1,
            ip: "192.168.1.1".to_string(),
            hostname: Some("router.local".to_string()),
            response_time: 1,
        },
        TraceHop {
            hop_number: 2,
            ip: "10.0.0.1".to_string(),
            hostname: Some("gateway.isp.com".to_string()),
            response_time: 15,
        },
        TraceHop {
            hop_number: 3,
            ip: request.target_ip.clone(),
            hostname: None,
            response_time: 45,
        },
    ];
    
    Json(ApiResponse::success(hops))
}

// Chat handlers
pub async fn get_chat_history(
    State(state): State<AppState>,
    Query(params): Query<ChatHistoryParams>,
) -> Json<ApiResponse<Vec<ChatMessage>>> {
    // Implement chat history loading
    // This would typically load from a chat_messages table
    let messages = vec![];
    Json(ApiResponse::success(messages))
}

pub async fn send_chat_message(
    State(state): State<AppState>,
    Json(request): Json<ChatMessageRequest>,
) -> Json<ApiResponse<()>> {
    let user_id = 1; // Extract from JWT
    
    // Get username
    match UserQueries::get_user_by_id(&state.db, user_id).await {
        Ok(Some(user)) => {
            // Broadcast message to all connected users
            let message = WebSocketMessage {
                user_id: None, // Broadcast to all
                message_type: "chat_message".to_string(),
                data: serde_json::json!({
                    "sender": user.username,
                    "message": request.message,
                    "timestamp": Utc::now()
                }),
                timestamp: Utc::now(),
            };
            
            let _ = state.websocket_tx.send(message);
            
            Json(ApiResponse::success(()))
        }
        Ok(None) => Json(ApiResponse::error("User not found")),
        Err(_) => Json(ApiResponse::error("Database error")),
    }
}

// Game mechanics handlers
pub async fn game_tick(State(state): State<AppState>) -> Json<ApiResponse<GameTickResult>> {
    // Update all active processes
    let active_processes = ProcessQueries::get_active_processes(&state.db).await.unwrap_or_default();
    
    let mut completed_processes = Vec::new();
    let mut updated_processes = Vec::new();
    
    for mut process in active_processes {
        if process.time_left > 0 {
            process.time_left = (process.time_left - 1).max(0);
            process.progress = 1.0 - (process.time_left as f64 / process.duration as f64);
            
            if process.time_left == 0 {
                // Process completed
                let _ = ProcessQueries::complete_process(&state.db, process.id).await;
                completed_processes.push(process.clone());
                
                // Handle process completion effects
                handle_process_completion(&state, &process).await;
            } else {
                // Process still running, update progress
                let _ = ProcessQueries::update_process_progress(&state.db, process.id, process.progress, process.time_left).await;
                updated_processes.push(process);
            }
        }
    }
    
    let result = GameTickResult {
        completed_processes: completed_processes.len() as i32,
        updated_processes: updated_processes.len() as i32,
        timestamp: Utc::now(),
    };
    
    Json(ApiResponse::success(result))
}

pub async fn get_game_stats(State(state): State<AppState>) -> Json<ApiResponse<GameStats>> {
    // Collect global game statistics
    let stats = GameStats {
        total_users: 0, // Query from database
        online_users: 0,
        total_servers: 0,
        active_processes: 0,
        total_clans: 0,
        uptime_seconds: 0,
    };
    
    Json(ApiResponse::success(stats))
}

// Admin handlers (would require admin authentication)
pub async fn admin_get_users(State(state): State<AppState>) -> Json<ApiResponse<Vec<DatabaseUser>>> {
    // Return all users for admin panel
    match UserQueries::get_user_ranking(&state.db, 1000).await {
        Ok(users) => Json(ApiResponse::success(users)),
        Err(_) => Json(ApiResponse::error("Failed to load users")),
    }
}

pub async fn admin_get_servers(State(state): State<AppState>) -> Json<ApiResponse<Vec<DatabaseServer>>> {
    // Return all servers for admin panel
    Json(ApiResponse::error("Not implemented"))
}

pub async fn admin_get_processes(State(state): State<AppState>) -> Json<ApiResponse<Vec<DatabaseProcess>>> {
    match ProcessQueries::get_active_processes(&state.db).await {
        Ok(processes) => Json(ApiResponse::success(processes)),
        Err(_) => Json(ApiResponse::error("Failed to load processes")),
    }
}

// WebSocket handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: WebSocket, state: AppState) {
    let user_id = 1; // Extract from authentication
    
    // Register connection
    let session_id = Uuid::new_v4().to_string();
    let connection = WebSocketConnection {
        user_id,
        session_id: session_id.clone(),
        connected_at: Utc::now(),
    };
    
    state.active_connections.write().await.insert(user_id, connection);
    
    // Subscribe to broadcasts
    let mut rx = state.websocket_tx.subscribe();
    
    // Handle WebSocket messages
    let (mut sender, mut receiver) = socket.split();
    
    // Listen for broadcasts and send to client
    tokio::spawn(async move {
        while let Ok(message) = rx.recv().await {
            // Send message to user if it's for them or broadcast
            if message.user_id.is_none() || message.user_id == Some(user_id) {
                let msg_text = serde_json::to_string(&message).unwrap_or_default();
                if sender.send(Message::Text(msg_text)).await.is_err() {
                    break;
                }
            }
        }
    });
    
    // Handle incoming messages from client
    while let Some(msg) = receiver.next().await {
        if let Ok(Message::Text(text)) = msg {
            if let Ok(client_message) = serde_json::from_str::<ClientMessage>(&text) {
                handle_client_message(&state, user_id, client_message).await;
            }
        }
    }
    
    // Remove connection when done
    state.active_connections.write().await.remove(&user_id);
}

// Helper functions
async fn handle_client_message(state: &AppState, user_id: i64, message: ClientMessage) {
    match message.message_type.as_str() {
        "ping" => {
            let response = WebSocketMessage {
                user_id: Some(user_id),
                message_type: "pong".to_string(),
                data: serde_json::json!({"timestamp": Utc::now()}),
                timestamp: Utc::now(),
            };
            let _ = state.websocket_tx.send(response);
        }
        "chat" => {
            if let Ok(chat_data) = serde_json::from_value::<ChatMessageRequest>(message.data) {
                // Broadcast chat message to all users
                if let Ok(Some(user)) = UserQueries::get_user_by_id(&state.db, user_id).await {
                    let chat_message = WebSocketMessage {
                        user_id: None, // Broadcast
                        message_type: "chat_message".to_string(),
                        data: serde_json::json!({
                            "sender": user.username,
                            "message": chat_data.message,
                            "timestamp": Utc::now()
                        }),
                        timestamp: Utc::now(),
                    };
                    let _ = state.websocket_tx.send(chat_message);
                }
            }
        }
        _ => {
            // Handle other message types
        }
    }
}

async fn handle_process_completion(state: &AppState, process: &DatabaseProcess) {
    // Implement process completion logic based on action type
    match process.action.as_str() {
        "hack" => {
            // Handle hack completion
            let success = calculate_hack_success(process);
            
            let message = WebSocketMessage {
                user_id: Some(process.creator_id),
                message_type: "process_complete".to_string(),
                data: serde_json::json!({
                    "process_id": process.id,
                    "action": process.action,
                    "success": success,
                    "results": generate_hack_results(process, success)
                }),
                timestamp: Utc::now(),
            };
            
            let _ = state.websocket_tx.send(message);
        }
        "download" => {
            // Handle file download completion
            let message = WebSocketMessage {
                user_id: Some(process.creator_id),
                message_type: "process_complete".to_string(),
                data: serde_json::json!({
                    "process_id": process.id,
                    "action": process.action,
                    "success": true,
                    "results": {
                        "file_downloaded": true,
                        "file_name": "data.txt"
                    }
                }),
                timestamp: Utc::now(),
            };
            
            let _ = state.websocket_tx.send(message);
        }
        _ => {
            // Handle other process types
        }
    }
}

// Utility and helper functions
async fn authenticate_user(db: &PgPool, username: &str, password: &str) -> Result<DatabaseUser> {
    match UserQueries::get_user_by_username(db, username).await? {
        Some(user) => {
            if verify_password(password, &user.password_hash).await {
                Ok(user)
            } else {
                Err(anyhow!("Invalid password"))
            }
        }
        None => Err(anyhow!("User not found")),
    }
}

async fn generate_jwt_token(user_id: i64) -> String {
    // Implement JWT token generation
    format!("jwt_token_for_user_{}", user_id)
}

async fn hash_password(password: &str) -> String {
    // Implement password hashing (bcrypt)
    format!("$2b$12$hashed_{}", password)
}

async fn verify_password(password: &str, hash: &str) -> bool {
    // Implement password verification
    hash.contains(password) // Simplified for example
}

fn map_user_to_profile(user: DatabaseUser) -> UserProfile {
    UserProfile {
        id: user.id,
        username: user.username,
        email: user.email,
        money: user.money,
        bitcoin: user.bitcoin,
        experience: user.experience,
        reputation: user.reputation,
        total_cpu: user.total_cpu,
        total_ram: user.total_ram,
        total_hdd: user.total_hdd,
        total_net: user.total_net,
        clan_id: user.clan_id,
        is_online: user.is_online,
        created_at: user.created_at,
    }
}

async fn get_user_main_server(db: &PgPool, user_id: i64) -> Result<i64> {
    // Get user's main PC/server
    match ServerQueries::get_servers_by_owner(db, user_id).await? {
        servers if !servers.is_empty() => Ok(servers[0].id),
        _ => Err(anyhow!("No server found for user")),
    }
}

async fn get_user_ip(db: &PgPool, user_id: i64) -> Result<String> {
    match get_user_main_server(db, user_id).await {
        Ok(server_id) => {
            match ServerQueries::get_server_by_id(db, server_id).await? {
                Some(server) => Ok(server.ip),
                None => Err(anyhow!("Server not found")),
            }
        }
        Err(e) => Err(e),
    }
}

async fn calculate_process_parameters(
    action: &str,
    software_id: Option<i64>,
    db: &PgPool,
    user_id: i64,
) -> (i32, i32, i32) {
    // Calculate duration, CPU usage, and NET usage based on action and software
    match action {
        "hack" => (300, 80, 60), // 5 minutes, 80% CPU, 60% NET
        "download" => (120, 20, 90), // 2 minutes, 20% CPU, 90% NET
        "upload" => (180, 30, 70), // 3 minutes, 30% CPU, 70% NET
        "scan" => (60, 50, 80), // 1 minute, 50% CPU, 80% NET
        _ => (180, 50, 50), // Default: 3 minutes, 50% CPU, 50% NET
    }
}

fn calculate_hack_success(process: &DatabaseProcess) -> bool {
    // Implement hack success calculation based on various factors
    // This is simplified - real implementation would consider:
    // - Target security level
    // - User skill/software level
    // - Random factors
    true // Simplified for example
}

fn generate_hack_results(process: &DatabaseProcess, success: bool) -> serde_json::Value {
    if success {
        serde_json::json!({
            "success": true,
            "money_gained": 5000,
            "experience_gained": 100,
            "files_found": ["database.sql", "passwords.txt"],
            "access_gained": true
        })
    } else {
        serde_json::json!({
            "success": false,
            "reason": "Firewall blocked the attack",
            "detected": true
        })
    }
}

async fn create_initial_hardware(db: &PgPool, user_id: i64) -> Result<()> {
    // Create basic starting hardware for new users
    let hardware_types = vec![
        ("cpu", 1000),
        ("ram", 512),
        ("hdd", 2048),
        ("net", 10),
    ];
    
    for (hw_type, spec_value) in hardware_types {
        let _ = HardwareQueries::create_hardware(db, user_id, hw_type, spec_value, false, true).await;
    }
    
    Ok(())
}

fn generate_hardware_store_items() -> Vec<HardwareStoreItem> {
    vec![
        HardwareStoreItem {
            hardware_type: "cpu".to_string(),
            spec_value: 2000,
            price: 5000,
            name: "CPU 2GHz".to_string(),
        },
        HardwareStoreItem {
            hardware_type: "ram".to_string(),
            spec_value: 1024,
            price: 3000,
            name: "RAM 1GB".to_string(),
        },
        HardwareStoreItem {
            hardware_type: "hdd".to_string(),
            spec_value: 10240,
            price: 4000,
            name: "HDD 10GB".to_string(),
        },
        HardwareStoreItem {
            hardware_type: "net".to_string(),
            spec_value: 100,
            price: 8000,
            name: "Network 100Mbps".to_string(),
        },
    ]
}

// Additional DTOs and types
#[derive(Serialize)]
pub struct ServerDetails {
    pub server: DatabaseServer,
    pub files: Vec<DatabaseFile>,
    pub installed_software: Vec<DatabaseInstalledSoftware>,
    pub is_hackable: bool,
    pub estimated_hack_time: i32,
}

#[derive(Serialize)]
pub struct ProcessDetails {
    pub process: DatabaseProcess,
    pub software: Option<DatabaseSoftware>,
}

#[derive(Serialize)]
pub struct InstalledSoftwareInfo {
    pub id: i64,
    pub software_id: i64,
    pub name: String,
    pub software_type: String,
    pub version: i32,
    pub is_running: bool,
    pub installed_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct UserStats {
    pub total_hacks: i32,
    pub successful_hacks: i32,
    pub money_earned: i64,
    pub experience: i64,
    pub reputation: f64,
    pub rank: i32,
    pub uptime: i64,
    pub servers_owned: i32,
    pub processes_completed: i32,
}

#[derive(Serialize)]
pub struct ClanInfo {
    pub clan: DatabaseClan,
    pub members: Vec<DatabaseUser>,
}

#[derive(Serialize)]
pub struct MissionResult {
    pub success: bool,
    pub reward_money: i64,
    pub reward_experience: i64,
    pub message: String,
}

#[derive(Serialize)]
pub struct HardwareStoreItem {
    pub hardware_type: String,
    pub spec_value: i64,
    pub price: i64,
    pub name: String,
}

#[derive(Serialize)]
pub struct NetworkScanResult {
    pub ip: String,
    pub hostname: Option<String>,
    pub open_ports: Vec<u16>,
    pub os: Option<String>,
    pub response_time: u32,
}

#[derive(Serialize)]
pub struct TraceHop {
    pub hop_number: i32,
    pub ip: String,
    pub hostname: Option<String>,
    pub response_time: u32,
}

#[derive(Serialize)]
pub struct ChatMessage {
    pub id: i64,
    pub sender: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct GameTickResult {
    pub completed_processes: i32,
    pub updated_processes: i32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct GameStats {
    pub total_users: i32,
    pub online_users: i32,
    pub total_servers: i32,
    pub active_processes: i32,
    pub total_clans: i32,
    pub uptime_seconds: i64,
}

#[derive(Serialize)]
pub struct FileContent {
    pub name: String,
    pub content: String,
    pub size: i64,
    pub file_type: String,
}

// Additional request DTOs
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub password: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateFileRequest {
    pub name: String,
    pub file_type: String,
    pub size: i64,
    pub path: String,
    pub is_hidden: Option<bool>,
}

#[derive(Deserialize)]
pub struct JoinClanRequest {
    pub clan_id: i64,
}

#[derive(Deserialize)]
pub struct TraceRouteRequest {
    pub target_ip: String,
}

#[derive(Deserialize)]
pub struct ChatHistoryParams {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Deserialize)]
pub struct UpgradeHardwareRequest {
    pub hardware_id: i64,
    pub new_spec_value: i64,
}

#[derive(Deserialize)]
pub struct ClientMessage {
    pub message_type: String,
    pub data: serde_json::Value,
}

// Re-export for main application
pub use create_api_router;