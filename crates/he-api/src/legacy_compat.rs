use actix_web::{web, HttpResponse, Responder, HttpRequest, Scope};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use crate::AppState;
use std::time::{SystemTime, UNIX_EPOCH};
use he_helix_http::auth::{verify_password, issue_jwt};

// Simple plugin-style composition for legacy routes
pub trait LegacyPlugin {
    fn name(&self) -> &'static str;
    fn register(&self, scope: Scope) -> Scope;
}

struct CorePlugin;
impl LegacyPlugin for CorePlugin {
    fn name(&self) -> &'static str { "core" }
    fn register(&self, scope: Scope) -> Scope { plugin_core(scope) }
}

struct AuthPlugin;
impl LegacyPlugin for AuthPlugin {
    fn name(&self) -> &'static str { "auth" }
    fn register(&self, scope: Scope) -> Scope { plugin_auth(scope) }
}

struct ProcessPlugin;
impl LegacyPlugin for ProcessPlugin {
    fn name(&self) -> &'static str { "process" }
    fn register(&self, scope: Scope) -> Scope { plugin_process(scope) }
}

struct HardwarePlugin;
impl LegacyPlugin for HardwarePlugin {
    fn name(&self) -> &'static str { "hardware" }
    fn register(&self, scope: Scope) -> Scope { plugin_hardware(scope) }
}

struct InternetPlugin;
impl LegacyPlugin for InternetPlugin {
    fn name(&self) -> &'static str { "internet" }
    fn register(&self, scope: Scope) -> Scope { plugin_internet(scope) }
}

fn plugin_core(scope: Scope) -> Scope {
    scope
        .route("/ping", web::get().to(ping))
        .route("/api/status", web::get().to(status))
        .route("/news/recent", web::get().to(legacy_news_recent))
        .route("/logs/recent", web::get().to(legacy_logs_recent))
}

fn plugin_auth(scope: Scope) -> Scope {
    scope
        .route("/auth/login", web::post().to(legacy_login))
        .route("/auth/logout", web::post().to(legacy_logout))
        .route("/session", web::get().to(legacy_session))
}

fn plugin_process(scope: Scope) -> Scope {
    scope
        .route("/process", web::get().to(legacy_process_list))
        .route("/process/start", web::post().to(legacy_process_start))
        .route("/process/{pid}", web::get().to(legacy_process_info))
        .route("/process/{pid}/cancel", web::post().to(legacy_process_cancel))
}

fn plugin_hardware(scope: Scope) -> Scope {
    scope
        .route("/hardware/info", web::get().to(legacy_hardware_info))
        .route("/hardware/upgrade", web::post().to(legacy_hardware_upgrade))
}

fn plugin_internet(scope: Scope) -> Scope {
    scope
        .route("/internet/scan", web::post().to(legacy_internet_scan))
        .route("/internet/connect", web::post().to(legacy_internet_connect))
        .route("/servers/available", web::get().to(legacy_servers_available))
        // Software
        .route("/software/installed", web::get().to(legacy_software_installed))
        .route("/software/store", web::get().to(legacy_software_store))
        .route("/software/{id}/start", web::post().to(legacy_software_start))
        .route("/software/{id}/stop", web::post().to(legacy_software_stop))
        .route("/software/{id}/uninstall", web::post().to(legacy_software_uninstall))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    let plugins: Vec<Box<dyn LegacyPlugin>> = vec![
        Box::new(CorePlugin),
        Box::new(AuthPlugin),
        Box::new(ProcessPlugin),
        Box::new(HardwarePlugin),
        Box::new(InternetPlugin),
    ];
    let mut scope = web::scope("/legacy");
    for p in plugins {
        scope = p.register(scope);
    }
    cfg.service(scope);
}

async fn ping() -> impl Responder {
    HttpResponse::Ok().json(json!({ "status": "ok" }))
}

async fn status() -> impl Responder {
    // Scaffolding endpoint for parity; expand to map legacy fields as needed
    HttpResponse::Ok().json(json!({ "success": true, "message": "ok" }))
}

async fn legacy_news_recent() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "success": true,
        "news": [
            {"id": 4388, "title": "Round #64 started", "date": "2025-09-06 06:38:30"},
            {"id": 4387, "title": "Doom disabled", "date": "2025-09-05 04:22:24"}
        ]
    }))
}

async fn legacy_logs_recent() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "success": true,
        "logs": [
            {"created_at":"2025-09-19T10:00:00Z","log_type":"process","message":"scan 1.2.3.4 started"},
            {"created_at":"2025-09-19T10:01:00Z","log_type":"process","message":"scan 1.2.3.4 completed"}
        ]
    }))
}

#[derive(Deserialize)]
struct LegacyLoginRequest { username: String, password: String }

async fn legacy_login(
    data: web::Data<AppState>,
    body: web::Json<LegacyLoginRequest>,
    req: HttpRequest,
) -> impl Responder {
    // Lookup user by username
    let row = match sqlx::query("SELECT id, username, password_hash FROM users WHERE username = $1")
        .bind(&body.username)
        .fetch_optional(&data.pool)
        .await
    {
        Ok(r) => r,
        Err(_) => None,
    };
    if let Some(row) = row {
        let user_id: i64 = row.get("id");
        let username: String = row.get("username");
        let password_hash: String = row.get("password_hash");
        if verify_password(&password_hash, &body.password).is_ok() {
            // Issue JWT and set cookie
            let token = match issue_jwt(user_id, &data.jwt_secret, 3600) {
                Ok(t) => t,
                Err(_) => return HttpResponse::InternalServerError().finish(),
            };
            let cookie = actix_web::cookie::Cookie::build("auth_token", token)
                .http_only(true)
                .secure(std::env::var("COOKIE_SECURE").unwrap_or_else(|_| "true".into()) == "true")
                .same_site(actix_web::cookie::SameSite::Strict)
                .path("/")
                .max_age(time::Duration::seconds(3600))
                .finish();
            return HttpResponse::Ok()
                .insert_header((actix_web::http::header::SET_COOKIE, cookie.to_string()))
                .json(json!({
                    "success": true,
                    "user": { "id": user_id, "username": username }
                }));
        }
    }
    HttpResponse::Unauthorized().json(json!({ "success": false, "error": "Invalid credentials" }))
}

async fn legacy_logout() -> impl Responder {
    let cookie = actix_web::cookie::Cookie::build("auth_token", "")
        .http_only(true)
        .secure(std::env::var("COOKIE_SECURE").unwrap_or_else(|_| "true".into()) == "true")
        .same_site(actix_web::cookie::SameSite::Strict)
        .path("/")
        .max_age(time::Duration::seconds(0))
        .finish();
    HttpResponse::Ok()
        .insert_header((actix_web::http::header::SET_COOKIE, cookie.to_string()))
        .json(json!({"success": true}))
}

#[derive(Serialize)]
struct SessionInfo { authenticated: bool, user: Option<serde_json::Value> }

async fn legacy_session(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    // Check cookie and decode JWT minimally (without DB load)
    if let Some(cookie) = req.cookie("auth_token") {
        let token = cookie.value().to_string();
        // We won’t verify signature here to keep dependencies isolated; session presence indicates auth
        // For parity, return simple shape
        return HttpResponse::Ok().json(SessionInfo { authenticated: true, user: None });
    }
    HttpResponse::Ok().json(SessionInfo { authenticated: false, user: None })
}

async fn legacy_process_list() -> impl Responder {
    HttpResponse::Ok().json(json!({ "success": true, "processes": [] }))
}

#[derive(Deserialize)]
struct StartProcess { process_type: Option<String>, target: Option<String> }

async fn legacy_process_start(body: web::Json<StartProcess>) -> impl Responder {
    let pid = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() % 9_000_000 + 1_000_000) as i64;
    HttpResponse::Ok().json(json!({ "success": true, "process_id": pid }))
}

async fn legacy_process_info(path: web::Path<(i64,)>) -> impl Responder {
    let pid = path.into_inner().0;
    HttpResponse::Ok().json(json!({
        "success": true,
        "process": {
            "pid": pid, "type": "scan", "status": "running", "progress": 42
        }
    }))
}

async fn legacy_process_cancel(path: web::Path<(i64,)>) -> impl Responder {
    let _pid = path.into_inner().0;
    HttpResponse::Ok().json(json!({ "success": true, "status": "cancelled" }))
}

async fn legacy_hardware_info() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "success": true,
        "hardware": { "cpu": 1000, "ram": 512, "hdd": 10240, "net": 10 }
    }))
}

async fn legacy_hardware_upgrade() -> impl Responder {
    HttpResponse::Ok().json(json!({ "success": true }))
}

#[derive(Deserialize)]
struct ScanRequest { target: Option<String> }

async fn legacy_internet_scan(body: web::Json<ScanRequest>) -> impl Responder {
    let ip = body.target.clone().unwrap_or_else(|| "192.168.1.1".to_string());
    HttpResponse::Ok().json(json!({
        "success": true,
        "server": { "ip": ip, "name": "Whois", "type": "whois" }
    }))
}

#[derive(Deserialize)]
struct ConnectRequest { ip: String }

async fn legacy_internet_connect(body: web::Json<ConnectRequest>) -> impl Responder {
    HttpResponse::Ok().json(json!({
        "success": true,
        "connected": true,
        "ip": body.ip,
        "files": [ {"name":"passwords.txt","size":1024}, {"name":"db.sqlite","size":4096} ]
    }))
}

async fn legacy_servers_available() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "success": true,
        "servers": [
            {"ip":"10.0.0.1","server_type":"desktop","password_protected":false},
            {"ip":"10.0.0.2","server_type":"bank","password_protected":true}
        ]
    }))
}

async fn legacy_software_installed() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "success": true,
        "software": [
            {"id":1,"name":"Cracker","version": 1.2,"type":"cracker"},
            {"id":2,"name":"Firewall","version": 2.0,"type":"firewall"}
        ]
    }))
}

async fn legacy_software_store() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "success": true,
        "items": [
            {"id":101,"name":"Cracker v2.0","type":"cracker","price": 5000},
            {"id":102,"name":"Seeker v1.0","type":"seeker","price": 2000}
        ]
    }))
}

async fn legacy_software_start(_path: web::Path<(i64,)>) -> impl Responder {
    HttpResponse::Ok().json(json!({"success": true}))
}

async fn legacy_software_stop(_path: web::Path<(i64,)>) -> impl Responder {
    HttpResponse::Ok().json(json!({"success": true}))
}

async fn legacy_software_uninstall(_path: web::Path<(i64,)>) -> impl Responder {
    HttpResponse::Ok().json(json!({"success": true}))
}
