//! Actual hacking gameplay handlers

use actix_web::{web, HttpResponse, HttpMessage, HttpRequest};
use chrono::Utc;
use he_database::{Database, queries::ProcessQueries};
use he_game_world::{GameWorld, NPCServer};
use he_game_mechanics::{GameEngine, GameMechanics};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;
use crate::error::ApiResult;
use crate::middleware::auth::AuthUser;

#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    pub target_ip: String,
}

#[derive(Debug, Serialize)]
pub struct ScanResponse {
    pub success: bool,
    pub server_info: Option<ServerInfo>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub ip_address: String,
    pub hostname: String,
    pub owner: String,
    pub server_type: String,
    pub security_level: i32,
    pub firewall_level: i32,
    pub is_online: bool,
}

/// Scan a target server
pub async fn scan_server(
    state: web::Data<AppState>,
    req: HttpRequest,
    data: web::Json<ScanRequest>,
) -> ApiResult<HttpResponse> {
    let user = req.extensions().get::<AuthUser>().unwrap().clone();
    let world = state.game_world.read().await;

    // Check if server exists
    if let Some(server) = world.get_server(&data.target_ip) {
        // Create scan process (takes time)
        let process = ProcessQueries::create_process_with_duration(
            state.db.pool(),
            user.user_id,
            "scan",
            &format!("pc_{}", user.user_id),
            Some(data.target_ip.clone()),
            30, // 30 seconds to scan
        ).await?;

        // Return server info after scan completes
        let info = ServerInfo {
            ip_address: server.ip_address.clone(),
            hostname: server.hostname.clone(),
            owner: server.owner_name.clone(),
            server_type: format!("{:?}", server.server_type),
            security_level: server.security_level,
            firewall_level: server.firewall_level,
            is_online: server.is_online,
        };

        Ok(HttpResponse::Ok().json(ScanResponse {
            success: true,
            server_info: Some(info),
            message: format!("Scan initiated. Process ID: {}", process.pid),
        }))
    } else {
        Ok(HttpResponse::NotFound().json(ScanResponse {
            success: false,
            server_info: None,
            message: "Server not found or unreachable".to_string(),
        }))
    }
}

#[derive(Debug, Deserialize)]
pub struct HackRequest {
    pub target_ip: String,
    pub crack_method: String, // "password", "exploit", "brute_force"
}

#[derive(Debug, Serialize)]
pub struct HackResponse {
    pub success: bool,
    pub access_granted: bool,
    pub process_id: Option<i64>,
    pub estimated_time: Option<i32>,
    pub message: String,
}

/// Initiate hack on target server
pub async fn hack_server(
    state: web::Data<AppState>,
    req: HttpRequest,
    data: web::Json<HackRequest>,
) -> ApiResult<HttpResponse> {
    let user = req.extensions().get::<AuthUser>().unwrap().clone();
    let mut world = state.game_world.write().await;

    if let Some(server) = world.get_server_mut(&data.target_ip) {
        // Get user's player state
        let player = state.get_player_state(user.user_id).await?;

        // Calculate hack duration and success chance
        let engine = GameEngine::new();
        let target_info = convert_server_to_target(server);

        let duration = engine.calculate_process_duration(&data.crack_method, &player, &target_info);
        let success_rate = engine.calculate_success_rate(&player, &target_info);

        // Create hacking process
        let process = ProcessQueries::create_process_with_duration(
            state.db.pool(),
            user.user_id,
            &data.crack_method,
            &format!("pc_{}", user.user_id),
            Some(data.target_ip.clone()),
            duration,
        ).await?;

        // Check if hack succeeds (random based on success_rate)
        let mut rng = rand::thread_rng();
        let success = rng.gen::<f64>() < success_rate.to_f64().unwrap_or(0.5);

        if success {
            // Add to user's hacked servers list
            server.on_hacked(&format!("player_{}", user.user_id));

            // Award experience and update statistics
            let server_tier = (server.security.firewall_level / 25 + 1).min(5) as u32;
            let is_first_time = !state.has_hacked_server(user.user_id, &data.target_ip).await?;

            // Update progression when hack completes
            state.update_progression_on_hack(
                user.user_id,
                server_tier,
                is_first_time,
                &data.target_ip,
            ).await?;

            // Grant access after process completes
            Ok(HttpResponse::Ok().json(HackResponse {
                success: true,
                access_granted: false, // Will be true when process completes
                process_id: Some(process.pid),
                estimated_time: Some(duration),
                message: format!("Hacking in progress. Estimated time: {} seconds", duration),
            }))
        } else {
            Ok(HttpResponse::Ok().json(HackResponse {
                success: false,
                access_granted: false,
                process_id: Some(process.pid),
                estimated_time: Some(duration),
                message: "Hack attempt will fail - security too strong".to_string(),
            }))
        }
    } else {
        Ok(HttpResponse::NotFound().json(HackResponse {
            success: false,
            access_granted: false,
            process_id: None,
            estimated_time: None,
            message: "Target server not found".to_string(),
        }))
    }
}

#[derive(Debug, Deserialize)]
pub struct ServerActionRequest {
    pub target_ip: String,
    pub action: String, // "download", "upload", "delete", "logs", "transfer"
    pub parameter: Option<String>, // filename, amount, etc.
}

#[derive(Debug, Serialize)]
pub struct ServerActionResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub message: String,
}

/// Perform action on hacked server
pub async fn server_action(
    state: web::Data<AppState>,
    req: HttpRequest,
    data: web::Json<ServerActionRequest>,
) -> ApiResult<HttpResponse> {
    let user = req.extensions().get::<AuthUser>().unwrap().clone();
    let mut world = state.game_world.write().await;

    // Check if user has access to this server
    if !state.user_has_access(user.user_id, &data.target_ip).await? {
        return Ok(HttpResponse::Forbidden().json(ServerActionResponse {
            success: false,
            data: None,
            message: "Access denied. Hack the server first.".to_string(),
        }));
    }

    if let Some(server) = world.get_server_mut(&data.target_ip) {
        match data.action.as_str() {
            "download" => {
                if let Some(filename) = &data.parameter {
                    if let Some(file) = server.download_file(filename, &format!("player_{}", user.user_id)) {
                        // Add file to user's storage
                        state.add_file_to_user(user.user_id, file.clone()).await?;

                        Ok(HttpResponse::Ok().json(ServerActionResponse {
                            success: true,
                            data: Some(serde_json::to_value(file)?),
                            message: format!("Downloaded {}", filename),
                        }))
                    } else {
                        Ok(HttpResponse::NotFound().json(ServerActionResponse {
                            success: false,
                            data: None,
                            message: "File not found".to_string(),
                        }))
                    }
                } else {
                    // List files
                    let files: Vec<String> = server.files.iter()
                        .filter(|f| !f.is_hidden) // Don't show hidden files initially
                        .map(|f| f.name.clone())
                        .collect();

                    Ok(HttpResponse::Ok().json(ServerActionResponse {
                        success: true,
                        data: Some(serde_json::to_value(files)?),
                        message: "File list retrieved".to_string(),
                    }))
                }
            },
            "delete" => {
                if let Some(filename) = &data.parameter {
                    if server.delete_file(filename, &format!("player_{}", user.user_id)) {
                        Ok(HttpResponse::Ok().json(ServerActionResponse {
                            success: true,
                            data: None,
                            message: format!("Deleted {}", filename),
                        }))
                    } else {
                        Ok(HttpResponse::NotFound().json(ServerActionResponse {
                            success: false,
                            data: None,
                            message: "File not found".to_string(),
                        }))
                    }
                } else {
                    Ok(HttpResponse::BadRequest().json(ServerActionResponse {
                        success: false,
                        data: None,
                        message: "Filename required".to_string(),
                    }))
                }
            },
            "logs" => {
                if data.parameter.as_deref() == Some("delete") {
                    // Delete all logs
                    server.logs.clear();
                    Ok(HttpResponse::Ok().json(ServerActionResponse {
                        success: true,
                        data: None,
                        message: "Logs deleted".to_string(),
                    }))
                } else {
                    // Show logs
                    let logs: Vec<String> = server.logs.iter()
                        .map(|l| format!("[{}] {} from {}", l.timestamp, l.action, l.ip_address))
                        .collect();

                    Ok(HttpResponse::Ok().json(ServerActionResponse {
                        success: true,
                        data: Some(serde_json::to_value(logs)?),
                        message: "Logs retrieved".to_string(),
                    }))
                }
            },
            "transfer" => {
                if let Some(amount_str) = &data.parameter {
                    if let Ok(amount) = amount_str.parse::<i64>() {
                        if server.transfer_money(amount, &format!("player_{}", user.user_id)) {
                            // Add money to user's account
                            state.add_money_to_user(user.user_id, amount).await?;

                            Ok(HttpResponse::Ok().json(ServerActionResponse {
                                success: true,
                                data: Some(serde_json::json!({ "transferred": amount })),
                                message: format!("Transferred ${}", amount),
                            }))
                        } else {
                            Ok(HttpResponse::BadRequest().json(ServerActionResponse {
                                success: false,
                                data: None,
                                message: "Insufficient funds on server".to_string(),
                            }))
                        }
                    } else {
                        Ok(HttpResponse::BadRequest().json(ServerActionResponse {
                            success: false,
                            data: None,
                            message: "Invalid amount".to_string(),
                        }))
                    }
                } else {
                    // Show available money
                    Ok(HttpResponse::Ok().json(ServerActionResponse {
                        success: true,
                        data: Some(serde_json::json!({ "available": server.money_available })),
                        message: format!("${} available", server.money_available),
                    }))
                }
            },
            _ => {
                Ok(HttpResponse::BadRequest().json(ServerActionResponse {
                    success: false,
                    data: None,
                    message: "Unknown action".to_string(),
                }))
            }
        }
    } else {
        Ok(HttpResponse::NotFound().json(ServerActionResponse {
            success: false,
            data: None,
            message: "Server not found".to_string(),
        }))
    }
}

#[derive(Debug, Serialize)]
pub struct InternetResponse {
    pub your_ip: String,
    pub known_servers: Vec<KnownServer>,
    pub recent_hacks: Vec<String>,
    pub bounties: Vec<BountyInfo>,
}

#[derive(Debug, Serialize)]
pub struct KnownServer {
    pub ip: String,
    pub hostname: String,
    pub last_seen: String,
    pub notes: String,
}

#[derive(Debug, Serialize)]
pub struct BountyInfo {
    pub corporation: String,
    pub target_ip: String,
    pub reward: i64,
    pub difficulty: String,
}

/// Get internet/network view for player
pub async fn internet_view(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let user = req.extensions().get::<AuthUser>().unwrap().clone();
    let world = state.game_world.read().await;

    // Get player's discovered servers
    let known_servers = state.get_user_known_servers(user.user_id).await?;

    // Get available bounties
    let bounties: Vec<BountyInfo> = world.corporations.iter()
        .flat_map(|corp| corp.bounties.iter().map(|b| BountyInfo {
            corporation: corp.name.clone(),
            target_ip: b.target_ip.clone(),
            reward: b.reward_money,
            difficulty: match world.get_server(&b.target_ip) {
                Some(s) => format!("Tier {}", s.tier),
                None => "Unknown".to_string(),
            },
        }))
        .take(5)
        .collect();

    // Get recent hacks
    let recent = state.get_user_recent_hacks(user.user_id).await?;

    Ok(HttpResponse::Ok().json(InternetResponse {
        your_ip: format!("player_{}.vpn", user.user_id),
        known_servers,
        recent_hacks: recent,
        bounties,
    }))
}

// Helper functions

fn convert_server_to_target(server: &NPCServer) -> he_game_mechanics::TargetInfo {
    he_game_mechanics::TargetInfo {
        ip_address: server.ip_address.clone(),
        target_type: format!("{:?}", server.server_type),
        difficulty_level: server.tier,
        security_rating: server.security_level,
        reward_money: server.money_available,
        defense_systems: server.running_software.iter()
            .map(|s| he_game_mechanics::DefenseSystem {
                system_type: s.software_type.clone(),
                strength: s.effectiveness,
                detection_rate: rust_decimal::Decimal::from(s.effectiveness) / rust_decimal::Decimal::from(100),
                response_time: 30,
            })
            .collect(),
    }
}

// Extension methods for AppState
impl AppState {
    async fn get_player_state(&self, user_id: i64) -> ApiResult<he_game_mechanics::PlayerState> {
        // Get user data from database and construct PlayerState
        // This is a simplified version
        Ok(he_game_mechanics::PlayerState {
            user_id: user_id as i32,
            level: 5, // Get from DB
            experience: 10000,
            money: 5000,
            reputation: std::collections::HashMap::new(),
            hardware_specs: he_game_mechanics::HardwareSpecs {
                cpu: 2000,
                ram: 4096,
                hdd: 100000,
                net: 100,
                security_level: 50,
                performance_rating: 60,
            },
            software_installed: Vec::new(),
            active_processes: Vec::new(),
            clan_membership: None,
            last_updated: Utc::now(),
        })
    }

    async fn user_has_access(&self, user_id: i64, target_ip: &str) -> ApiResult<bool> {
        // Check if user has hacked this server
        // For now, simplified check
        Ok(true) // TODO: Implement proper access control
    }

    async fn add_file_to_user(&self, user_id: i64, file: he_game_world::ServerFile) -> ApiResult<()> {
        // Add file to user's storage in database
        // TODO: Implement file storage
        Ok(())
    }

    async fn add_money_to_user(&self, user_id: i64, amount: i64) -> ApiResult<()> {
        // Update user's money in database
        // TODO: Implement money update
        Ok(())
    }

    async fn get_user_known_servers(&self, user_id: i64) -> ApiResult<Vec<KnownServer>> {
        // Get list of servers user has discovered
        Ok(vec![
            KnownServer {
                ip: "1.2.3.4".to_string(),
                hostname: "whois.first.org".to_string(),
                last_seen: "Recently".to_string(),
                notes: "Tutorial server".to_string(),
            },
        ])
    }

    async fn get_user_recent_hacks(&self, user_id: i64) -> ApiResult<Vec<String>> {
        // Get user's recent successful hacks
        Ok(vec![
            "10.0.0.1 - Home PC".to_string(),
            "172.16.0.5 - Small Company".to_string(),
        ])
    }
}