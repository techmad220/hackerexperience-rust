//! API Routes Configuration

use actix_web::{web, Scope};

/// Configure API routes
pub fn configure_routes() -> Scope {
    web::scope("/api")
        .service(
            web::scope("/v1")
                .route("/health", web::get().to(health_check))
                .route("/version", web::get().to(version_info))
        )
}

async fn health_check() -> actix_web::Result<actix_web::HttpResponse> {
    Ok(actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy"
    })))
}

async fn version_info() -> actix_web::Result<actix_web::HttpResponse> {
    Ok(actix_web::HttpResponse::Ok().json(serde_json::json!({
        "version": "1.0.0"
    })))
}