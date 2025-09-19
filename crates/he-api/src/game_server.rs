use anyhow::{anyhow, Result};
//! Game Server - REST API that connects the game engine to the frontend

use actix_web::{web, App, HttpResponse, HttpServer, Result, middleware, HttpRequest};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, Instant};
use uuid::Uuid;
use std::collections::HashMap;

// Simple inline game engine (avoiding the broken dependencies)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub id: Uuid,
    pub process_type: String,
    pub state: String,
    pub priority: String,
    pub progress: f32,
    pub time_remaining: String,
    pub cpu_usage: f32,
    pub ram_usage: f32,
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub cpu_mhz: f32,
    pub ram_mb: f32,
    pub disk_gb: f32,
    pub network_mbps: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub processes: Vec<ProcessInfo>,
    pub hardware: HardwareInfo,
    pub cpu_available: f32,
    pub ram_available: f32,
    pub last_update: SystemTime,
}

// Simple process implementation
#[derive(Debug, Clone)]
struct Process {
    id: Uuid,
    process_type: String,
    priority: String,
    progress: f32,
    cpu_usage: f32,
    ram_usage: f32,
    time_total: Duration,
    time_elapsed: Duration,
    target: Option<String>,
}

impl Process {
    fn new(process_type: String, priority: String, target: Option<String>) -> Self {
        let (cpu, ram, duration) = match process_type.as_str() {
            "Crack" => (350.0, 128.0, Duration::from_secs(10)),
            "Download" => (100.0, 64.0, Duration::from_secs(5)),
            "Scan" => (150.0, 32.0, Duration::from_secs(3)),
            "Install" => (200.0, 256.0, Duration::from_secs(7)),
            "DDoS" => (400.0, 512.0, Duration::from_secs(15)),
            "Mine" => (800.0, 1024.0, Duration::from_secs(30)),
            _ => (100.0, 64.0, Duration::from_secs(5)),
        };

        Self {
            id: Uuid::new_v4(),
            process_type,
            priority,
            progress: 0.0,
            cpu_usage: cpu,
            ram_usage: ram,
            time_total: duration,
            time_elapsed: Duration::ZERO,
            target,
        }
    }

    fn update(&mut self, delta: Duration) {
        self.time_elapsed = self.time_elapsed.saturating_add(delta);
        self.progress = (self.time_elapsed.as_secs_f32() / self.time_total.as_secs_f32() * 100.0).min(100.0);
    }

    fn is_complete(&self) -> bool {
        self.progress >= 100.0
    }

    fn time_remaining(&self) -> Duration {
        self.time_total.saturating_sub(self.time_elapsed)
    }

    fn to_info(&self) -> ProcessInfo {
        let state = if self.is_complete() {
            "Completed".to_string()
        } else {
            "Running".to_string()
        };

        let remaining = self.time_remaining();
        let time_str = if remaining.as_secs() > 60 {
            format!("{}m {}s", remaining.as_secs() / 60, remaining.as_secs() % 60)
        } else {
            format!("{}s", remaining.as_secs())
        };

        ProcessInfo {
            id: self.id,
            process_type: self.process_type.clone(),
            state,
            priority: self.priority.clone(),
            progress: self.progress,
            time_remaining: time_str,
            cpu_usage: self.cpu_usage,
            ram_usage: self.ram_usage,
            target: self.target.clone(),
        }
    }
}

// Game Engine
pub struct GameEngine {
    processes: Vec<Process>,
    hardware: HardwareInfo,
    cpu_available: f32,
    ram_available: f32,
    last_update: Instant,
}

impl GameEngine {
    pub fn new() -> Self {
        let hardware = HardwareInfo {
            cpu_mhz: 1000.0,
            ram_mb: 1024.0,
            disk_gb: 100.0,
            network_mbps: 100.0,
        };

        Self {
            processes: Vec::new(),
            hardware: hardware.clone(),
            cpu_available: hardware.cpu_mhz,
            ram_available: hardware.ram_mb,
            last_update: Instant::now(),
        }
    }

    pub fn start_process(&mut self, process_type: String, priority: String, target: Option<String>) -> Result<Uuid, String> {
        let process = Process::new(process_type.clone(), priority, target);

        // Check resources
        if self.cpu_available < process.cpu_usage {
            return Err(format!("Insufficient CPU: need {} MHz, have {} MHz",
                              process.cpu_usage, self.cpu_available));
        }
        if self.ram_available < process.ram_usage {
            return Err(format!("Insufficient RAM: need {} MB, have {} MB",
                              process.ram_usage, self.ram_available));
        }

        // Allocate resources
        self.cpu_available -= process.cpu_usage;
        self.ram_available -= process.ram_usage;

        let id = process.id;
        self.processes.push(process);
        Ok(id)
    }

    pub fn cancel_process(&mut self, id: Uuid) -> Result<(), String> {
        if let Some(index) = self.processes.iter().position(|p| p.id == id) {
            let process = self.processes.remove(index);
            // Return resources, ensuring we don't exceed hardware limits
            self.cpu_available = (self.cpu_available + process.cpu_usage).min(self.hardware.cpu_mhz);
            self.ram_available = (self.ram_available + process.ram_usage).min(self.hardware.ram_mb);
            Ok(())
        } else {
            Err("Process not found".to_string())
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let delta = now - self.last_update;
        self.last_update = now;

        // Update all processes
        for process in &mut self.processes {
            process.update(delta);
        }

        // Remove completed processes and return resources
        let completed: Vec<usize> = self.processes
            .iter()
            .enumerate()
            .filter(|(_, p)| p.is_complete())
            .map(|(i, _)| i)
            .collect();

        for i in completed.iter().rev() {
            let process = self.processes.remove(*i);
            // Return resources, ensuring we don't exceed hardware limits
            self.cpu_available = (self.cpu_available + process.cpu_usage).min(self.hardware.cpu_mhz);
            self.ram_available = (self.ram_available + process.ram_usage).min(self.hardware.ram_mb);
        }
    }

    pub fn get_state(&self) -> GameState {
        GameState {
            processes: self.processes.iter().map(|p| p.to_info()).collect(),
            hardware: self.hardware.clone(),
            cpu_available: self.cpu_available,
            ram_available: self.ram_available,
            last_update: SystemTime::now(),
        }
    }
}

// User profile structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: u32,
    pub username: String,
    pub level: u32,
    pub reputation: i32,
    pub clan: Option<String>,
    pub joined: String,
    pub last_seen: String,
}

// News article structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticle {
    pub id: u32,
    pub title: String,
    pub content: String,
    pub author: String,
    pub date: String,
}

// Blog post structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogPost {
    pub id: u32,
    pub title: String,
    pub content: String,
    pub author: String,
    pub date: String,
    pub tags: Vec<String>,
}

// Session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub session_id: String,
    pub user_id: u32,
    pub username: String,
    pub created_at: SystemTime,
}

// Shared application state
pub struct AppState {
    pub engine: Arc<Mutex<GameEngine>>,
    pub users: Arc<Mutex<HashMap<u32, UserProfile>>>,
    pub news: Arc<Mutex<Vec<NewsArticle>>>,
    pub blogs: Arc<Mutex<Vec<BlogPost>>>,
    pub sessions: Arc<Mutex<HashMap<String, SessionData>>>,
}

// API Request/Response types
#[derive(Deserialize)]
pub struct StartProcessRequest {
    pub process_type: String,
    pub priority: String,
    pub target: Option<String>,
}

#[derive(Serialize)]
pub struct StartProcessResponse {
    pub success: bool,
    pub process_id: Option<Uuid>,
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct CancelProcessRequest {
    pub process_id: Uuid,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

// API Endpoints

async fn get_game_state(data: web::Data<AppState>) -> Result<HttpResponse> {
    let mut engine = data.engine.lock().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
    engine.update();
    let state = engine.get_state();

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(state),
        error: None,
    }))
}

async fn start_process(
    data: web::Data<AppState>,
    req: web::Json<StartProcessRequest>,
) -> Result<HttpResponse> {
    let mut engine = data.engine.lock().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
    engine.update();

    match engine.start_process(req.process_type.clone(), req.priority.clone(), req.target.clone()) {
        Ok(id) => Ok(HttpResponse::Ok().json(StartProcessResponse {
            success: true,
            process_id: Some(id),
            error: None,
        })),
        Err(e) => Ok(HttpResponse::Ok().json(StartProcessResponse {
            success: false,
            process_id: None,
            error: Some(e),
        })),
    }
}

async fn cancel_process(
    data: web::Data<AppState>,
    req: web::Json<CancelProcessRequest>,
) -> Result<HttpResponse> {
    let mut engine = data.engine.lock().map_err(|e| anyhow::anyhow!("Error: {}", e))?;

    match engine.cancel_process(req.process_id) {
        Ok(()) => Ok(HttpResponse::Ok().json(ApiResponse::<()> {
            success: true,
            data: None,
            error: None,
        })),
        Err(e) => Ok(HttpResponse::Ok().json(ApiResponse::<()> {
            success: false,
            data: None,
            error: Some(e),
        })),
    }
}

async fn get_processes(data: web::Data<AppState>) -> Result<HttpResponse> {
    let mut engine = data.engine.lock().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
    engine.update();
    let state = engine.get_state();

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(state.processes),
        error: None,
    }))
}

async fn get_hardware(data: web::Data<AppState>) -> Result<HttpResponse> {
    let engine = data.engine.lock().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
    let state = engine.get_state();

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(state.hardware),
        error: None,
    }))
}

// Dynamic content handlers

async fn get_user_profile(req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse> {
    let query = web::Query::<HashMap<String, String>>::from_query(req.query_string())?;

    if let Some(id_str) = query.get("id") {
        if let Ok(id) = id_str.parse::<u32>() {
            let users = data.users.lock().map_err(|e| anyhow::anyhow!("Error: {}", e))?;

            if let Some(user) = users.get(&id) {
                return Ok(HttpResponse::Ok().json(ApiResponse {
                    success: true,
                    data: Some(user.clone()),
                    error: None,
                }));
            }
        }
    }

    // Return mock data if user not found
    let mock_user = UserProfile {
        id: query.get("id").and_then(|s| s.parse().ok()).unwrap_or(1),
        username: format!("Player_{}", query.get("id").unwrap_or(&"1".to_string())),
        level: 15,
        reputation: 1250,
        clan: Some("Elite Hackers".to_string()),
        joined: "2025-01-01".to_string(),
        last_seen: "2025-09-16".to_string(),
    };

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(mock_user),
        error: None,
    }))
}

async fn get_news(req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse> {
    let query = web::Query::<HashMap<String, String>>::from_query(req.query_string())?;

    if let Some(id_str) = query.get("id") {
        if let Ok(id) = id_str.parse::<u32>() {
            let news = data.news.lock().map_err(|e| anyhow::anyhow!("Error: {}", e))?;

            if let Some(article) = news.iter().find(|n| n.id == id) {
                return Ok(HttpResponse::Ok().json(ApiResponse {
                    success: true,
                    data: Some(article.clone()),
                    error: None,
                }));
            }
        }

        // Return mock article
        let mock_article = NewsArticle {
            id: query.get("id").and_then(|s| s.parse().ok()).unwrap_or(1),
            title: "New Security Update Released".to_string(),
            content: "A critical security update has been released for all game servers. Players are advised to update their systems immediately.".to_string(),
            author: "Admin".to_string(),
            date: "2025-09-16".to_string(),
        };

        return Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(mock_article),
            error: None,
        }));
    }

    // Return all news
    let news = data.news.lock().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
    let all_news = if news.is_empty() {
        vec![
            NewsArticle {
                id: 4388,
                title: "Game Server Upgrade Complete".to_string(),
                content: "The game servers have been upgraded for better performance.".to_string(),
                author: "System".to_string(),
                date: "2025-09-16".to_string(),
            },
            NewsArticle {
                id: 4387,
                title: "New Hacking Tools Available".to_string(),
                content: "Check out the latest hacking tools in the software center.".to_string(),
                author: "DevTeam".to_string(),
                date: "2025-09-15".to_string(),
            },
        ]
    } else {
        news.clone()
    };

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(all_news),
        error: None,
    }))
}

async fn get_blog(req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse> {
    let query = web::Query::<HashMap<String, String>>::from_query(req.query_string())?;

    if let Some(id_str) = query.get("id") {
        if let Ok(id) = id_str.parse::<u32>() {
            // Return mock blog post
            let mock_post = BlogPost {
                id,
                title: format!("Hacking Tutorial #{}", id),
                content: "Learn advanced hacking techniques and improve your skills.".to_string(),
                author: "HackerPro".to_string(),
                date: "2025-09-16".to_string(),
                tags: vec!["tutorial".to_string(), "hacking".to_string()],
            };

            return Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some(mock_post),
                error: None,
            }));
        }
    }

    // Return all blog posts
    let blogs = vec![
        BlogPost {
            id: 1,
            title: "Getting Started with HackerExperience".to_string(),
            content: "Welcome to the world of hacking!".to_string(),
            author: "Admin".to_string(),
            date: "2025-09-01".to_string(),
            tags: vec!["beginner".to_string(), "guide".to_string()],
        },
    ];

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(blogs),
        error: None,
    }))
}

async fn get_stats() -> Result<HttpResponse> {
    let stats = HashMap::from([
        ("total_users", 15234),
        ("online_users", 342),
        ("total_hacks", 98765),
        ("servers_online", 42),
    ]);

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(stats),
        error: None,
    }))
}

async fn logout(data: web::Data<AppState>) -> Result<HttpResponse> {
    // In a real app, we'd clear the session
    Ok(HttpResponse::Ok().json(ApiResponse::<()> {
        success: true,
        data: None,
        error: None,
    }))
}

async fn get_clan(req: HttpRequest) -> Result<HttpResponse> {
    let query = web::Query::<HashMap<String, String>>::from_query(req.query_string())?;

    let clan_data = HashMap::from([
        ("id", query.get("id").unwrap_or(&"1".to_string()).clone()),
        ("name", "Elite Hackers".to_string()),
        ("members", "42".to_string()),
        ("rank", "5".to_string()),
    ]);

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(clan_data),
        error: None,
    }))
}

async fn get_internet_ip(req: HttpRequest) -> Result<HttpResponse> {
    let query = web::Query::<HashMap<String, String>>::from_query(req.query_string())?;

    let server_data = HashMap::from([
        ("ip", query.get("ip").unwrap_or(&"127.0.0.1".to_string()).clone()),
        ("hostname", "target.server.com".to_string()),
        ("status", "online".to_string()),
        ("firewall", "active".to_string()),
    ]);

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(server_data),
        error: None,
    }))
}

pub async fn run_server() -> std::io::Result<()> {
    println!("🚀 Starting HackerExperience Game Server on http://localhost:3005");

    // Initialize with sample data
    let mut initial_users = HashMap::new();
    initial_users.insert(1672, UserProfile {
        id: 1672,
        username: "Neo".to_string(),
        level: 42,
        reputation: 9999,
        clan: Some("The Matrix".to_string()),
        joined: "2024-01-01".to_string(),
        last_seen: "2025-09-16".to_string(),
    });

    let app_state = web::Data::new(AppState {
        engine: Arc::new(Mutex::new(GameEngine::new())),
        users: Arc::new(Mutex::new(initial_users)),
        news: Arc::new(Mutex::new(Vec::new())),
        blogs: Arc::new(Mutex::new(Vec::new())),
        sessions: Arc::new(Mutex::new(HashMap::new())),
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .app_data(app_state.clone())
            .wrap(cors)
            .wrap(middleware::Logger::default())
            // Core game APIs
            .route("/api/state", web::get().to(get_game_state))
            .route("/api/processes", web::get().to(get_processes))
            .route("/api/processes/start", web::post().to(start_process))
            .route("/api/processes/cancel", web::post().to(cancel_process))
            .route("/api/hardware", web::get().to(get_hardware))
            // Dynamic content APIs
            .route("/api/profile", web::get().to(get_user_profile))
            .route("/api/news", web::get().to(get_news))
            .route("/api/blog", web::get().to(get_blog))
            .route("/api/stats", web::get().to(get_stats))
            .route("/api/logout", web::post().to(logout))
            .route("/api/clan", web::get().to(get_clan))
            .route("/api/internet", web::get().to(get_internet_ip))
            // WebSocket
            .route("/ws", web::get().to(crate::websocket::websocket_handler))
            // Health check
            .route("/health", web::get().to(|| async { HttpResponse::Ok().body("OK") }))
    })
    .bind("127.0.0.1:3005")?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_creation() {
        let process = Process::new("Crack".to_string(), "High".to_string(), None);
        assert_eq!(process.cpu_usage, 350.0);
        assert_eq!(process.ram_usage, 128.0);
        assert_eq!(process.progress, 0.0);
    }

    #[test]
    fn test_game_engine() {
        let mut engine = GameEngine::new();
        let result = engine.start_process("Scan".to_string(), "Normal".to_string(), Some("192.168.1.1".to_string()));
        assert!(result.is_ok());
        assert_eq!(engine.processes.len(), 1);
    }
}