//! Dashboard router that connects to legacy-compat endpoints
//!
//! This module wires all the dashboard sections to their corresponding
//! legacy-compat PHP compatibility layer endpoints

use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::AppState;
use tera::Context;

/// Dashboard API routes structure
pub struct DashboardRouter;

impl DashboardRouter {
    /// Configure all dashboard routes
    pub fn configure(cfg: &mut web::ServiceConfig) {
        cfg
            // Main dashboard
            .service(
                web::resource("/game")
                    .route(web::get().to(render_dashboard))
            )
            // API endpoints for dashboard data
            .service(
                web::scope("/api")
                    .route("/processes", web::get().to(get_processes))
                    .route("/hardware", web::get().to(get_hardware))
                    .route("/news", web::get().to(get_news))
                    .route("/ranking/top", web::get().to(get_top_users))
                    .route("/logs/recent", web::get().to(get_recent_logs))
                    .route("/missions/active", web::get().to(get_active_missions))
                    .route("/mail/unread", web::get().to(get_unread_mail))
            )
            // Page routes (legacy PHP compatibility)
            .service(
                web::scope("")
                    .route("/processes.php", web::get().to(render_processes_page))
                    .route("/software.php", web::get().to(render_software_page))
                    .route("/internet.php", web::get().to(render_internet_page))
                    .route("/log.php", web::get().to(render_log_page))
                    .route("/hardware.php", web::get().to(render_hardware_page))
                    .route("/university.php", web::get().to(render_university_page))
                    .route("/finances.php", web::get().to(render_finances_page))
                    .route("/list.php", web::get().to(render_hacked_db_page))
                    .route("/missions.php", web::get().to(render_missions_page))
                    .route("/clan.php", web::get().to(render_clan_page))
                    .route("/ranking.php", web::get().to(render_ranking_page))
                    .route("/mail.php", web::get().to(render_mail_page))
            );
    }
}

/// Render main dashboard
async fn render_dashboard(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // Get user session
    let user_id = get_user_id_from_session(&req).unwrap_or(0);

    // Prepare template context
    let mut context = Context::new();
    context.insert("username", &get_username(user_id, &data).await);
    context.insert("money", &get_user_money(user_id, &data).await);
    context.insert("ip", &get_user_ip(user_id, &data).await);
    context.insert("nonce", &generate_nonce());

    // Render template
    let html = data.templates
        .render("game_classic.html", &context)
        .unwrap_or_else(|_| "Error rendering template".to_string());

    HttpResponse::Ok()
        .content_type("text/html")
        .body(html)
}

/// Get active processes for current user
async fn get_processes(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    let user_id = get_user_id_from_session(&req).unwrap_or(0);

    let processes = sqlx::query!(
        r#"
        SELECT id, type as name,
               COALESCE(progress, 0) as progress,
               COALESCE(eta_seconds, 0) as eta
        FROM processes
        WHERE user_id = $1 AND state = 'running'
        ORDER BY created_at DESC
        LIMIT 10
        "#,
        user_id
    )
    .fetch_all(&data.pool)
    .await
    .unwrap_or_default();

    let result: Vec<_> = processes.iter().map(|p| {
        json!({
            "id": p.id,
            "name": p.name,
            "progress": p.progress,
            "eta": p.eta
        })
    }).collect();

    HttpResponse::Ok().json(result)
}

/// Get hardware information
async fn get_hardware(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    let user_id = get_user_id_from_session(&req).unwrap_or(0);

    let hw = sqlx::query!(
        r#"
        SELECT
            COALESCE(cpu_mhz, 1000) as cpu,
            COALESCE(ram_mb, 512) as ram,
            COALESCE(hdd_gb, 10) as hdd_total,
            COALESCE(hdd_used_gb, 0) as hdd_used,
            COALESCE(net_mbps, 10) as net
        FROM hardware
        WHERE user_id = $1
        LIMIT 1
        "#,
        user_id
    )
    .fetch_optional(&data.pool)
    .await
    .unwrap_or(None);

    let result = if let Some(h) = hw {
        json!({
            "cpu": h.cpu,
            "ram": h.ram,
            "hdd_total": h.hdd_total,
            "hdd_used": h.hdd_used,
            "net": h.net
        })
    } else {
        // Default hardware
        json!({
            "cpu": 1000,
            "ram": 512,
            "hdd_total": 10,
            "hdd_used": 0,
            "net": 10
        })
    };

    HttpResponse::Ok().json(result)
}

/// Get latest news
async fn get_news(
    data: web::Data<AppState>,
) -> impl Responder {
    let news = sqlx::query!(
        r#"
        SELECT title, content, created_at as date
        FROM news
        ORDER BY created_at DESC
        LIMIT 5
        "#
    )
    .fetch_all(&data.pool)
    .await
    .unwrap_or_default();

    let result: Vec<_> = news.iter().map(|n| {
        json!({
            "title": n.title,
            "content": n.content,
            "date": n.date.format("%Y-%m-%d %H:%M").to_string()
        })
    }).collect();

    HttpResponse::Ok().json(result)
}

/// Get top users for ranking
async fn get_top_users(
    data: web::Data<AppState>,
) -> impl Responder {
    let users = sqlx::query!(
        r#"
        SELECT username, reputation
        FROM users
        ORDER BY reputation DESC
        LIMIT 10
        "#
    )
    .fetch_all(&data.pool)
    .await
    .unwrap_or_default();

    let result: Vec<_> = users.iter().map(|u| {
        json!({
            "username": u.username,
            "reputation": u.reputation
        })
    }).collect();

    HttpResponse::Ok().json(result)
}

/// Get recent log entries
async fn get_recent_logs(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    let user_id = get_user_id_from_session(&req).unwrap_or(0);

    let logs = sqlx::query!(
        r#"
        SELECT message, is_localhost, created_at as timestamp
        FROM logs
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 10
        "#,
        user_id
    )
    .fetch_all(&data.pool)
    .await
    .unwrap_or_default();

    let result: Vec<_> = logs.iter().map(|l| {
        json!({
            "message": l.message,
            "localhost": l.is_localhost,
            "timestamp": l.timestamp.format("%H:%M:%S").to_string()
        })
    }).collect();

    HttpResponse::Ok().json(result)
}

/// Get active missions
async fn get_active_missions(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    let user_id = get_user_id_from_session(&req).unwrap_or(0);

    let missions = sqlx::query!(
        r#"
        SELECT title, description, reward, deadline
        FROM missions
        WHERE user_id = $1 AND status = 'active'
        ORDER BY deadline ASC
        LIMIT 5
        "#,
        user_id
    )
    .fetch_all(&data.pool)
    .await
    .unwrap_or_default();

    let result: Vec<_> = missions.iter().map(|m| {
        json!({
            "title": m.title,
            "description": m.description,
            "reward": m.reward,
            "deadline": m.deadline.map(|d| d.format("%Y-%m-%d").to_string())
        })
    }).collect();

    HttpResponse::Ok().json(result)
}

/// Get unread mail count
async fn get_unread_mail(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    let user_id = get_user_id_from_session(&req).unwrap_or(0);

    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as count
        FROM mail
        WHERE recipient_id = $1 AND is_read = false
        "#,
        user_id
    )
    .fetch_one(&data.pool)
    .await
    .unwrap_or(0);

    HttpResponse::Ok().json(json!({"unread": count}))
}

// Page rendering functions (delegate to legacy-compat)

async fn render_processes_page(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    render_legacy_page("processes", req, data).await
}

async fn render_software_page(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    render_legacy_page("software", req, data).await
}

async fn render_internet_page(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    render_legacy_page("internet", req, data).await
}

async fn render_log_page(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    render_legacy_page("log", req, data).await
}

async fn render_hardware_page(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    render_legacy_page("hardware", req, data).await
}

async fn render_university_page(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    render_legacy_page("university", req, data).await
}

async fn render_finances_page(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    render_legacy_page("finances", req, data).await
}

async fn render_hacked_db_page(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    render_legacy_page("list", req, data).await
}

async fn render_missions_page(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    render_legacy_page("missions", req, data).await
}

async fn render_clan_page(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    render_legacy_page("clan", req, data).await
}

async fn render_ranking_page(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    render_legacy_page("ranking", req, data).await
}

async fn render_mail_page(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    render_legacy_page("mail", req, data).await
}

/// Generic legacy page renderer
async fn render_legacy_page(
    page: &str,
    req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // Forward to legacy-compat handler
    let response = reqwest::get(&format!("http://localhost:8080/legacy/{}.php", page))
        .await
        .unwrap_or_else(|_| {
            // Fallback response
            return HttpResponse::ServiceUnavailable()
                .body("Legacy compatibility layer unavailable");
        });

    HttpResponse::Ok()
        .content_type("text/html")
        .body(response.text().await.unwrap_or_else(|_| {
            format!("<h1>{} Page</h1><p>Under construction</p>", page)
        }))
}

// Helper functions

fn get_user_id_from_session(req: &HttpRequest) -> Option<i64> {
    // TODO: Implement proper session extraction
    // For now, return a mock user ID
    Some(1)
}

async fn get_username(user_id: i64, data: &web::Data<AppState>) -> String {
    sqlx::query_scalar!("SELECT username FROM users WHERE id = $1", user_id)
        .fetch_optional(&data.pool)
        .await
        .unwrap_or(None)
        .unwrap_or_else(|| "Guest".to_string())
}

async fn get_user_money(user_id: i64, data: &web::Data<AppState>) -> i64 {
    sqlx::query_scalar!("SELECT COALESCE(money, 0) FROM users WHERE id = $1", user_id)
        .fetch_optional(&data.pool)
        .await
        .unwrap_or(None)
        .unwrap_or(0)
}

async fn get_user_ip(user_id: i64, data: &web::Data<AppState>) -> String {
    sqlx::query_scalar!("SELECT ip_address FROM user_ips WHERE user_id = $1 LIMIT 1", user_id)
        .fetch_optional(&data.pool)
        .await
        .unwrap_or(None)
        .unwrap_or_else(|| "127.0.0.1".to_string())
}

fn generate_nonce() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();

    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}