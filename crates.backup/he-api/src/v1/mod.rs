//! API v1 module - Current stable API version
//!
//! This module contains all v1 API endpoints with backwards compatibility guarantees.

use actix_web::web;
use crate::handlers;

/// Configure all v1 API routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            // Authentication endpoints
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(handlers::auth::register))
                    .route("/login", web::post().to(handlers::auth::login))
                    .route("/logout", web::post().to(handlers::auth::logout))
                    .route("/refresh", web::post().to(handlers::auth::refresh_token))
                    .route("/verify", web::get().to(handlers::auth::verify_token))
            )
            // User endpoints
            .service(
                web::scope("/users")
                    .route("", web::get().to(handlers::user::list_users))
                    .route("/{id}", web::get().to(handlers::user::get_user))
                    .route("/{id}", web::patch().to(handlers::user::update_user))
                    .route("/{id}/stats", web::get().to(handlers::user::get_user_stats))
                    .route("/{id}/experience", web::patch().to(handlers::progression::update_experience))
            )
            // Process endpoints
            .service(
                web::scope("/processes")
                    .route("", web::get().to(handlers::process::list_processes))
                    .route("", web::post().to(handlers::process::create_process))
                    .route("/{id}", web::get().to(handlers::process::get_process))
                    .route("/{id}", web::delete().to(handlers::process::cancel_process))
                    .route("/{id}/status", web::patch().to(handlers::process::update_process_status))
            )
            // Hardware endpoints
            .service(
                web::scope("/hardware")
                    .route("", web::get().to(handlers::hardware::list_hardware))
                    .route("/{id}", web::get().to(handlers::hardware::get_hardware))
                    .route("/{id}/upgrade", web::post().to(handlers::hardware::upgrade_hardware))
            )
            // Bank endpoints
            .service(
                web::scope("/bank")
                    .route("/accounts", web::get().to(handlers::bank::list_accounts))
                    .route("/accounts/{id}", web::get().to(handlers::bank::get_account))
                    .route("/transfer", web::post().to(handlers::bank::transfer_money))
                    .route("/transactions", web::get().to(handlers::bank::list_transactions))
            )
            // Mission endpoints
            .service(
                web::scope("/missions")
                    .route("", web::get().to(handlers::mission::list_missions))
                    .route("/{id}", web::get().to(handlers::mission::get_mission))
                    .route("/{id}/accept", web::post().to(handlers::mission::accept_mission))
                    .route("/{id}/complete", web::post().to(handlers::mission::complete_mission))
                    .route("/{id}/progress", web::patch().to(handlers::mission::update_progress))
            )
            // Clan endpoints
            .service(
                web::scope("/clans")
                    .route("", web::get().to(handlers::clan::list_clans))
                    .route("", web::post().to(handlers::clan::create_clan))
                    .route("/{id}", web::get().to(handlers::clan::get_clan))
                    .route("/{id}/join", web::post().to(handlers::clan::join_clan))
                    .route("/{id}/leave", web::post().to(handlers::clan::leave_clan))
            )
            // Game mechanics endpoints
            .service(
                web::scope("/game")
                    .route("/hack", web::post().to(handlers::game::initiate_hack))
                    .route("/defend", web::post().to(handlers::game::defend_against_hack))
                    .route("/research", web::post().to(handlers::game::research_software))
                    .route("/market", web::get().to(handlers::game::get_market_prices))
            )
            // Admin endpoints (protected)
            .service(
                web::scope("/admin")
                    .route("/users", web::get().to(handlers::admin::list_all_users))
                    .route("/stats", web::get().to(handlers::admin::get_system_stats))
                    .route("/logs", web::get().to(handlers::admin::get_security_logs))
            )
            // Health and status endpoints
            .route("/health", web::get().to(handlers::health::health_check))
            .route("/status", web::get().to(handlers::health::status))
    );
}

/// API v1 response wrapper for consistent formatting
#[derive(serde::Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub meta: ResponseMeta,
}

#[derive(serde::Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(serde::Serialize)]
pub struct ResponseMeta {
    pub version: &'static str,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_id: String,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, request_id: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: ResponseMeta {
                version: "v1",
                timestamp: chrono::Utc::now(),
                request_id,
            },
        }
    }

    pub fn error(code: String, message: String, request_id: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(ApiError {
                code,
                message,
                details: None,
            }),
            meta: ResponseMeta {
                version: "v1",
                timestamp: chrono::Utc::now(),
                request_id,
            },
        }
    }
}