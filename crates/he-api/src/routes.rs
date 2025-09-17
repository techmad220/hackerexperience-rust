//! API Routes

use actix_web::web;
use crate::handlers::{auth, game, process, hardware, bank, missions};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        // Authentication routes
        .route("/api/auth/register", web::post().to(auth::register))
        .route("/api/auth/login", web::post().to(auth::login))
        .route("/api/auth/logout", web::post().to(auth::logout))
        .route("/api/auth/refresh", web::post().to(auth::refresh_token))

        // Game routes
        .route("/api/game/status", web::get().to(game::get_status))
        .route("/api/game/dashboard", web::get().to(game::get_dashboard))

        // Process management
        .route("/api/processes", web::get().to(process::list_processes))
        .route("/api/processes", web::post().to(process::create_process))
        .route("/api/processes/{pid}/cancel", web::delete().to(process::cancel_process))

        // Hardware management
        .route("/api/hardware", web::get().to(hardware::get_hardware))
        .route("/api/hardware/upgrade", web::post().to(hardware::upgrade_hardware))

        // Banking
        .route("/api/bank/accounts", web::get().to(bank::get_accounts))
        .route("/api/bank/transfer", web::post().to(bank::transfer_money))

        // Missions
        .route("/api/missions", web::get().to(missions::get_missions))
        .route("/api/missions/{id}/accept", web::post().to(missions::accept_mission))
        .route("/api/missions/{id}/progress", web::post().to(missions::update_progress))

        // WebSocket endpoint
        .route("/ws", web::get().to(crate::websocket::websocket_handler));
}