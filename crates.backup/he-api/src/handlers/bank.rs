//! Banking handlers

use actix_web::{web, HttpResponse, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use crate::handlers::game::extract_user_id;

#[derive(Serialize)]
pub struct AccountInfo {
    pub account_number: String,
    pub balance: i64,
}

#[derive(Deserialize)]
pub struct TransferRequest {
    pub from_account: String,
    pub to_account: String,
    pub amount: i64,
}

pub async fn get_accounts(
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

    match he_database::queries::BankQueries::get_user_accounts(&state.db.pool, user_id).await {
        Ok(accounts) => {
            let account_list: Vec<AccountInfo> = accounts.into_iter().map(|a| AccountInfo {
                account_number: a.account_number,
                balance: a.balance,
            }).collect();

            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "accounts": account_list
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("Failed to get accounts: {}", e)
            }))
        }
    }
}

pub async fn transfer_money(
    state: web::Data<AppState>,
    _req: HttpRequest,
    data: web::Json<TransferRequest>,
) -> HttpResponse {
    // Validate amount
    if data.amount <= 0 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "Invalid amount"
        }));
    }

    match he_database::queries::BankQueries::transfer_money(
        &state.db.pool,
        &data.from_account,
        &data.to_account,
        data.amount,
    ).await {
        Ok(true) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": format!("Transferred ${} successfully", data.amount)
            }))
        }
        Ok(false) => {
            HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "message": "Insufficient funds"
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": format!("Transfer failed: {}", e)
            }))
        }
    }
}