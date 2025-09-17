//! Hardware management handlers

use actix_web::{web, HttpResponse, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use crate::handlers::game::extract_user_id;

#[derive(Serialize)]
pub struct HardwareInfo {
    pub cpu_speed: f64,
    pub ram_size: i64,
    pub hdd_size: i64,
    pub hdd_used: i64,
    pub net_speed: f64,
}

#[derive(Deserialize)]
pub struct UpgradeRequest {
    pub component: String, // cpu, ram, hdd, net
    pub level: u32,
}

pub async fn get_hardware(
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

    let pc_id = format!("pc_{}", user_id);

    match he_database::queries::HardwareQueries::get_hardware(&state.db.pool, &pc_id).await {
        Ok(Some(hw)) => {
            HttpResponse::Ok().json(HardwareInfo {
                cpu_speed: hw.cpu_speed,
                ram_size: hw.ram_size,
                hdd_size: hw.hdd_size,
                hdd_used: hw.hdd_used,
                net_speed: hw.net_speed,
            })
        }
        Ok(None) => {
            // Create default hardware for new user
            HttpResponse::Ok().json(HardwareInfo {
                cpu_speed: 1.0,
                ram_size: 256,
                hdd_size: 10000,
                hdd_used: 0,
                net_speed: 1.0,
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("Failed to get hardware: {}", e)
            }))
        }
    }
}

pub async fn upgrade_hardware(
    state: web::Data<AppState>,
    req: HttpRequest,
    data: web::Json<UpgradeRequest>,
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

    let pc_id = format!("pc_{}", user_id);

    // Calculate new values based on component and level
    let (cpu, ram, hdd, net) = match data.component.as_str() {
        "cpu" => (Some(data.level as f64), None, None, None),
        "ram" => (None, Some((256 * data.level) as i64), None, None),
        "hdd" => (None, None, Some((10000 * data.level) as i64), None),
        "net" => (None, None, None, Some(data.level as f64)),
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "Invalid component"
            }));
        }
    };

    match he_database::queries::HardwareQueries::update_hardware(
        &state.db.pool,
        &pc_id,
        cpu,
        ram,
        hdd,
        net,
    ).await {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": format!("{} upgraded to level {}", data.component, data.level)
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("Upgrade failed: {}", e)
            }))
        }
    }
}