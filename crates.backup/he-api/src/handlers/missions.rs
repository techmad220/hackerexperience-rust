//! Mission handlers

use actix_web::{web, HttpResponse, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use crate::handlers::game::extract_user_id;

#[derive(Serialize)]
pub struct MissionInfo {
    pub id: i64,
    pub mission_type: String,
    pub status: String,
    pub reward_money: i64,
    pub reward_xp: i32,
    pub progress: i32,
    pub total_steps: i32,
}

#[derive(Deserialize)]
pub struct ProgressUpdate {
    pub progress: i32,
}

pub async fn get_missions(
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

    match he_database::queries::MissionQueries::get_user_missions(&state.db.pool, user_id).await {
        Ok(missions) => {
            let mission_list: Vec<MissionInfo> = missions.into_iter().map(|m| MissionInfo {
                id: m.id,
                mission_type: m.mission_type,
                status: m.status,
                reward_money: m.reward_money,
                reward_xp: m.reward_xp,
                progress: m.progress,
                total_steps: m.total_steps,
            }).collect();

            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "missions": mission_list
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("Failed to get missions: {}", e)
            }))
        }
    }
}

pub async fn accept_mission(
    _state: web::Data<AppState>,
    _req: HttpRequest,
    path: web::Path<i64>,
) -> HttpResponse {
    let mission_id = path.into_inner();

    // TODO: Implement mission acceptance logic
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": format!("Mission {} accepted", mission_id)
    }))
}

pub async fn update_progress(
    state: web::Data<AppState>,
    _req: HttpRequest,
    path: web::Path<i64>,
    data: web::Json<ProgressUpdate>,
) -> HttpResponse {
    let mission_id = path.into_inner();

    match he_database::queries::MissionQueries::update_mission_progress(
        &state.db.pool,
        mission_id,
        data.progress,
    ).await {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "Progress updated"
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("Failed to update progress: {}", e)
            }))
        }
    }
}