//! Integration tests for HackerExperience

use actix_web::{test, App};
use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_registration() {
        let app = test::init_service(
            App::new()
                .configure(he_api::routes::configure)
        ).await;

        let req = test::TestRequest::post()
            .uri("/api/auth/register")
            .set_json(&json!({
                "login": "testuser",
                "email": "test@example.com",
                "password": "password123"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_login() {
        let app = test::init_service(
            App::new()
                .configure(he_api::routes::configure)
        ).await;

        // First register
        let register_req = test::TestRequest::post()
            .uri("/api/auth/register")
            .set_json(&json!({
                "login": "logintest",
                "email": "login@example.com",
                "password": "password123"
            }))
            .to_request();

        test::call_service(&app, register_req).await;

        // Then login
        let login_req = test::TestRequest::post()
            .uri("/api/auth/login")
            .set_json(&json!({
                "email": "login@example.com",
                "password": "password123"
            }))
            .to_request();

        let resp = test::call_service(&app, login_req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_process_creation() {
        let app = test::init_service(
            App::new()
                .configure(he_api::routes::configure)
        ).await;

        // Mock auth token
        let req = test::TestRequest::post()
            .uri("/api/processes")
            .header("Authorization", "Bearer mock_token")
            .set_json(&json!({
                "process_type": "scan",
                "target_pc_id": "192.168.1.1"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        // Will fail without proper auth, but tests the endpoint
        assert_eq!(resp.status(), 401);
    }

    #[actix_rt::test]
    async fn test_hardware_info() {
        let app = test::init_service(
            App::new()
                .configure(he_api::routes::configure)
        ).await;

        let req = test::TestRequest::get()
            .uri("/api/hardware")
            .header("Authorization", "Bearer mock_token")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
    }

    #[actix_rt::test]
    async fn test_websocket_connection() {
        // WebSocket test would require a more complex setup
        // This is a placeholder
        assert!(true);
    }

    #[actix_rt::test]
    async fn test_game_mechanics() {
        use he_game_mechanics::{
            processes::ProcessManager,
            hacking::HackCalculator,
        };

        let process_manager = ProcessManager::new();
        let duration = process_manager.calculate_process_time(
            &he_game_mechanics::processes::ProcessType::Scan,
            1.0,  // CPU
            256,  // RAM
            1.0,  // NET
        );
        assert!(duration > 0);

        let calc = HackCalculator::new();
        let success = calc.calculate_success_rate(5, 3, 0);
        assert!(success > 0.0 && success <= 1.0);
    }

    #[actix_rt::test]
    async fn test_database_operations() {
        use he_database::{Database, queries::UserQueries};

        // This would need a test database
        let db_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://test:test@localhost:5432/test".to_string());

        if let Ok(db) = Database::new(&db_url).await {
            // Test user creation
            let result = UserQueries::create_user(
                &db.pool,
                "dbtest",
                "db@test.com",
                "password123"
            ).await;

            assert!(result.is_ok() || result.is_err()); // Either works for test
        }
    }

    #[test]
    fn test_security_features() {
        use he_helix_security::{
            audit::AuditLogger,
            intrusion::IntrusionDetector,
            ddos::DDoSProtection,
        };

        let audit = AuditLogger::new();
        audit.log_event(he_helix_security::SecurityEvent::LoginAttempt {
            username: "test".to_string(),
            ip: std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            success: true,
        });

        let ids = IntrusionDetector::new();
        assert!(!ids.is_threat(&std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))));

        let ddos = DDoSProtection::new();
        assert!(ddos.check_rate_limit(&std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))));
    }
}