//! Idempotent process cancellation handler

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth::AuthedUser;

#[derive(Deserialize)]
pub struct CancelReq {
    pub process_id: i64,
}

#[derive(Serialize)]
pub struct CancelResp {
    pub status: String,
    pub process_id: i64,
}

/// Cancel a process idempotently
///
/// This endpoint:
/// - Always returns 200 OK for idempotency
/// - Validates user owns the process
/// - Uses database row locking to prevent races
/// - Silently succeeds if process is already cancelled
pub async fn cancel(
    pool: web::Data<PgPool>,
    user: AuthedUser,  // Extracted from JWT by middleware
    payload: web::Json<CancelReq>,
) -> actix_web::Result<impl Responder> {
    // Call the idempotent cancel function from he-helix-core
    match he_helix_core::process_cancel::cancel_process(&pool, payload.process_id, user.id).await {
        Ok(()) => {
            tracing::info!("Process {} cancelled by user {}", payload.process_id, user.id);
        }
        Err(e) => {
            // Log the error but still return success for idempotency
            tracing::warn!(
                "cancel_process failed for pid={} uid={}: {:?} (treating as success)",
                payload.process_id, user.id, e
            );
        }
    }

    // Always return OK for idempotent semantics
    Ok(HttpResponse::Ok().json(CancelResp {
        status: "ok".to_string(),
        process_id: payload.process_id,
    }))
}

/// Rate-limited cancel endpoint configuration
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/api/processes/cancel")
            .route(web::post().to(cancel))
            // Rate limit: 10 cancellations per minute per user
            .wrap(crate::middleware::RateLimit::new(10, 60))
    );
}