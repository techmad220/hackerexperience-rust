//! Game status handlers

use actix_web::{web, HttpResponse, HttpRequest};
use serde::Serialize;
use crate::state::AppState;
use crate::handlers::process::extract_user_id;

#[derive(Serialize)]
pub struct GameStatus {
    pub online: bool,
    pub level: u32,
    pub experience: u64,
    pub reputation: i32,
}

#[derive(Serialize)]
pub struct DashboardData {
    pub status: GameStatus,
    pub active_processes: usize,
    pub hardware_load: f32,
    pub bank_balance: i64,
    pub unread_messages: u32,
}

pub async fn get_status(
    state: web::Data<AppState>,
    req: HttpRequest,
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

    // Get user data from database
    let user = match he_database::queries::UserQueries::get_user_by_id(&state.db.pool, user_id).await {
        Ok(Some(u)) => u,
        _ => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "User not found"
            }));
        }
    };

    HttpResponse::Ok().json(GameStatus {
        online: user.online,
        level: 1, // TODO: Calculate from experience
        experience: 0, // TODO: Get from user stats
        reputation: 0, // TODO: Get from user stats
    })
}

pub async fn get_dashboard(
    state: web::Data<AppState>,
    req: HttpRequest,
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

    // Get various data for dashboard
    let processes = he_database::queries::ProcessQueries::get_user_processes(&state.db.pool, user_id)
        .await
        .unwrap_or_default();

    let accounts = he_database::queries::BankQueries::get_user_accounts(&state.db.pool, user_id)
        .await
        .unwrap_or_default();

    let total_balance: i64 = accounts.iter().map(|a| a.balance).sum();

    HttpResponse::Ok().json(DashboardData {
        status: GameStatus {
            online: true,
            level: 1,
            experience: 0,
            reputation: 0,
        },
        active_processes: processes.len(),
        hardware_load: (processes.len() as f32 * 10.0).min(100.0),
        bank_balance: total_balance,
        unread_messages: 0,
    })
}

// Helper made public for other modules
pub async fn extract_user_id(state: &web::Data<AppState>, req: &HttpRequest) -> Option<i64> {
    let token = req.headers()
        .get("Authorization")?
        .to_str().ok()?
        .strip_prefix("Bearer ")?;

    let validated = state.auth.validate_token(token).await.ok()??;
    Some(validated.user_id.as_u128() as i64)
}