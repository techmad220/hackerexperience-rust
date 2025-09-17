#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! actix-web = "4"
//! actix-cors = "0.6"
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! tokio = { version = "1", features = ["full"] }
//! uuid = { version = "1.0", features = ["v4", "serde"] }
//! ```

use actix_web::{web, App, HttpResponse, HttpServer, middleware};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// Mock data structures
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ServerInfo {
    ip_address: String,
    hostname: String,
    owner: String,
    server_type: String,
    security_level: i32,
    firewall_level: i32,
    is_online: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Process {
    pid: i64,
    user_id: i64,
    process_type: String,
    state: String,
    started_at: String,
    completed_at: Option<String>,
    source_server: String,
    target_server: Option<String>,
    priority: i32,
    cpu_usage: i32,
    ram_usage: i32,
    completion_percentage: f32,
    estimated_time_remaining: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Mission {
    id: i64,
    mission_type: String,
    status: String,
    reward_money: i64,
    reward_xp: i32,
    progress: i32,
    total_steps: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Software {
    id: i64,
    name: String,
    version: String,
    software_type: String,
    size: i32,
    cpu_usage: i32,
    ram_usage: i32,
    effectiveness: i32,
    is_hidden: bool,
    is_installed: bool,
    location: String,
}

// App State
struct AppState {
    processes: Arc<Mutex<Vec<Process>>>,
    missions: Arc<Mutex<Vec<Mission>>>,
    software: Arc<Mutex<Vec<Software>>>,
}

// Hacking endpoints
async fn scan_server(data: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "server_info": ServerInfo {
            ip_address: "1.2.3.4".to_string(),
            hostname: "whois.first.org".to_string(),
            owner: "First International Bank".to_string(),
            server_type: "Web Server".to_string(),
            security_level: 50,
            firewall_level: 30,
            is_online: true,
        },
        "message": "Scan complete"
    }))
}

async fn hack_server(data: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "access_granted": false,
        "process_id": 12345,
        "estimated_time": 60,
        "message": "Hacking in progress..."
    }))
}

async fn server_action(data: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": {
            "files": ["passwords.txt", "database.db", "secret.key"],
            "available": 50000
        },
        "message": "Action completed"
    }))
}

async fn internet_view(data: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "your_ip": "player_1.vpn",
        "known_servers": [
            {
                "ip": "1.2.3.4",
                "hostname": "whois.first.org",
                "last_seen": "Recently",
                "notes": "Tutorial server"
            },
            {
                "ip": "10.0.0.1",
                "hostname": "home-pc.local",
                "last_seen": "1 hour ago",
                "notes": "Easy target"
            }
        ],
        "recent_hacks": [
            "10.0.0.1 - Home PC",
            "172.16.0.5 - Small Company"
        ],
        "bounties": [
            {
                "corporation": "MegaCorp",
                "target_ip": "192.168.100.50",
                "reward": 100000,
                "difficulty": "Tier 3"
            }
        ]
    }))
}

// Process endpoints
async fn get_processes(data: web::Data<AppState>) -> HttpResponse {
    let processes = data.processes.lock().unwrap();
    HttpResponse::Ok().json(serde_json::json!({
        "processes": processes.clone()
    }))
}

async fn create_process(data: web::Data<AppState>) -> HttpResponse {
    let mut processes = data.processes.lock().unwrap();
    let new_process = Process {
        pid: processes.len() as i64 + 1,
        user_id: 1,
        process_type: "scan".to_string(),
        state: "running".to_string(),
        started_at: "2025-09-17T10:00:00Z".to_string(),
        completed_at: None,
        source_server: "localhost".to_string(),
        target_server: Some("1.2.3.4".to_string()),
        priority: 2,
        cpu_usage: 150,
        ram_usage: 32,
        completion_percentage: 45.0,
        estimated_time_remaining: 30,
    };

    processes.push(new_process.clone());

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "process": new_process,
        "message": "Process created"
    }))
}

// Software endpoints
async fn get_software(data: web::Data<AppState>) -> HttpResponse {
    let software = data.software.lock().unwrap();
    HttpResponse::Ok().json(serde_json::json!({
        "local_software": software.clone(),
        "external_software": [],
        "local_used": 500,
        "local_total": 100000,
        "external_used": 0,
        "external_total": 200000
    }))
}

async fn software_action(data: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Action completed",
        "process_id": 123
    }))
}

// Mission endpoints
async fn get_missions(data: web::Data<AppState>) -> HttpResponse {
    let missions = data.missions.lock().unwrap();
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "missions": missions.clone()
    }))
}

async fn get_available_missions(data: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!([
        {
            "id": "mission_1",
            "name": "First Score",
            "description": "Your first real hacking job",
            "mission_type": "hack",
            "difficulty": "Easy",
            "reward_money": 5000,
            "reward_xp": 100,
            "reward_items": [],
            "requirements": ["Level 1"],
            "objectives": ["Hack the target server", "Download files", "Delete logs"]
        },
        {
            "id": "mission_2",
            "name": "Corporate Espionage",
            "description": "Steal data from MegaCorp",
            "mission_type": "data_theft",
            "difficulty": "Medium",
            "reward_money": 25000,
            "reward_xp": 500,
            "reward_items": ["Advanced Cracker v2.0"],
            "requirements": ["Level 5", "Cracker v1.5+"],
            "objectives": ["Infiltrate MegaCorp network", "Find classified files", "Transfer to client"]
        }
    ]))
}

async fn accept_mission(data: web::Data<AppState>) -> HttpResponse {
    let mut missions = data.missions.lock().unwrap();
    missions.push(Mission {
        id: missions.len() as i64 + 1,
        mission_type: "hack".to_string(),
        status: "active".to_string(),
        reward_money: 5000,
        reward_xp: 100,
        progress: 0,
        total_steps: 3,
    });

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Mission accepted"
    }))
}

// Auth endpoints (mock)
async fn login() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "token": "mock_jwt_token_12345",
        "user": {
            "id": 1,
            "username": "hacker"
        }
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Starting HackerExperience Test Server");
    println!("📡 API Server: http://localhost:3000");
    println!("🌐 Frontend: http://localhost:8080");
    println!("");
    println!("Available endpoints:");
    println!("  POST /api/login");
    println!("  POST /api/hacking/scan");
    println!("  POST /api/hacking/hack");
    println!("  POST /api/hacking/action");
    println!("  GET  /api/hacking/internet");
    println!("  GET  /api/process/list");
    println!("  POST /api/process/create");
    println!("  GET  /api/software/list");
    println!("  POST /api/software/action");
    println!("  GET  /api/missions");
    println!("  GET  /api/missions/available");
    println!("  POST /api/missions/{id}/accept");
    println!("");

    let app_state = web::Data::new(AppState {
        processes: Arc::new(Mutex::new(vec![
            Process {
                pid: 1,
                user_id: 1,
                process_type: "crack".to_string(),
                state: "running".to_string(),
                started_at: "2025-09-17T09:00:00Z".to_string(),
                completed_at: None,
                source_server: "localhost".to_string(),
                target_server: Some("192.168.1.100".to_string()),
                priority: 2,
                cpu_usage: 350,
                ram_usage: 128,
                completion_percentage: 75.0,
                estimated_time_remaining: 15,
            }
        ])),
        missions: Arc::new(Mutex::new(vec![
            Mission {
                id: 1,
                mission_type: "tutorial".to_string(),
                status: "active".to_string(),
                reward_money: 1000,
                reward_xp: 50,
                progress: 2,
                total_steps: 3,
            }
        ])),
        software: Arc::new(Mutex::new(vec![
            Software {
                id: 1,
                name: "Basic Cracker".to_string(),
                version: "1.0".to_string(),
                software_type: "cracker".to_string(),
                size: 100,
                cpu_usage: 200,
                ram_usage: 64,
                effectiveness: 50,
                is_hidden: false,
                is_installed: true,
                location: "local".to_string(),
            },
            Software {
                id: 2,
                name: "Port Scanner".to_string(),
                version: "2.0".to_string(),
                software_type: "exploit".to_string(),
                size: 50,
                cpu_usage: 100,
                ram_usage: 32,
                effectiveness: 75,
                is_hidden: false,
                is_installed: true,
                location: "local".to_string(),
            }
        ])),
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(app_state.clone())
            .wrap(cors)
            .wrap(middleware::Logger::default())

            // Auth
            .route("/api/login", web::post().to(login))

            // Hacking endpoints
            .route("/api/hacking/scan", web::post().to(scan_server))
            .route("/api/hacking/hack", web::post().to(hack_server))
            .route("/api/hacking/action", web::post().to(server_action))
            .route("/api/hacking/internet", web::get().to(internet_view))

            // Process endpoints
            .route("/api/process/list", web::get().to(get_processes))
            .route("/api/process/create", web::post().to(create_process))
            .route("/api/process/{id}/cancel", web::post().to(|_: web::Path<i64>| async {
                HttpResponse::Ok().json(serde_json::json!({"success": true}))
            }))
            .route("/api/process/{id}/toggle", web::post().to(|_: web::Path<i64>| async {
                HttpResponse::Ok().json(serde_json::json!({"success": true}))
            }))

            // Software endpoints
            .route("/api/software/list", web::get().to(get_software))
            .route("/api/software/action", web::post().to(software_action))

            // Mission endpoints
            .route("/api/missions", web::get().to(get_missions))
            .route("/api/missions/available", web::get().to(get_available_missions))
            .route("/api/missions/{id}/accept", web::post().to(accept_mission))

            // Health check
            .route("/health", web::get().to(|| async {
                HttpResponse::Ok().json(serde_json::json!({"status": "healthy"}))
            }))
    })
    .bind("0.0.0.0:3000")?
    .run()
    .await
}