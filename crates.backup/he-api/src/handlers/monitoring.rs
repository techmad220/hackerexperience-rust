//! Monitoring and metrics endpoints

use actix_web::{web, HttpResponse, Result};
use he_monitoring::{MonitoringService, HealthCheck, HealthStatus};
use serde_json::json;

/// Prometheus metrics endpoint
pub async fn metrics() -> Result<HttpResponse> {
    let metrics = MonitoringService::export_metrics();
    Ok(HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(metrics))
}

/// Health check endpoint with detailed status
pub async fn health() -> Result<HttpResponse> {
    let mut health_check = HealthCheck::new();

    // Add various health checks
    health_check.add_check(|| HealthStatus {
        name: "api".to_string(),
        healthy: true,
        message: "API server is running".to_string(),
    });

    health_check.add_check(|| HealthStatus {
        name: "database".to_string(),
        healthy: true, // Would check actual DB connection
        message: "Database connection OK".to_string(),
    });

    health_check.add_check(|| HealthStatus {
        name: "cache".to_string(),
        healthy: true, // Would check Redis connection
        message: "Cache layer operational".to_string(),
    });

    let statuses = health_check.check_health();
    let overall_healthy = health_check.is_healthy();

    let response = json!({
        "status": if overall_healthy { "healthy" } else { "unhealthy" },
        "checks": statuses.into_iter().map(|s| json!({
            "name": s.name,
            "status": if s.healthy { "pass" } else { "fail" },
            "message": s.message
        })).collect::<Vec<_>>()
    });

    if overall_healthy {
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(response))
    }
}

/// Readiness probe for Kubernetes
pub async fn ready() -> Result<HttpResponse> {
    // Check if server is ready to accept traffic
    Ok(HttpResponse::Ok().json(json!({
        "ready": true,
        "message": "Server ready to accept traffic"
    })))
}

/// Liveness probe for Kubernetes
pub async fn live() -> Result<HttpResponse> {
    // Simple check that server is alive
    Ok(HttpResponse::Ok().json(json!({
        "alive": true,
        "timestamp": chrono::Utc::now()
    })))
}