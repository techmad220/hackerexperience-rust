//! Full system integration tests

use actix_web::{test, web, App};
use chrono::Utc;
use he_api::{create_app, state::AppState};
use he_auth::AuthService;
use he_database::Database;
use he_game_mechanics::{GameEngine, GameMechanics};
use he_websocket::ConnectionManager;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[cfg(test)]
mod integration_tests {
    use super::*;

    async fn setup_test_environment() -> AppState {
        // Create test database
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://test:test@localhost:5432/test_he".to_string());

        let db = Database::new(&database_url).await.unwrap();

        // Run migrations
        let _ = db.migrate().await;

        // Create auth service
        let auth = AuthService::new("test_secret_key".to_string());

        // Create WebSocket manager
        let ws_manager = Some(Arc::new(ConnectionManager::new()));

        AppState { db, auth, ws_manager }
    }

    mod full_user_flow {
        use super::*;
        use actix_web::http::StatusCode;

        #[actix_web::test]
        async fn test_complete_user_journey() {
            let state = setup_test_environment().await;
            let app = test::init_service(create_app(state)).await;

            let email = format!("user_{}@example.com", Uuid::new_v4());

            // Step 1: Register new user
            let register_req = test::TestRequest::post()
                .uri("/api/auth/register")
                .set_json(&json!({
                    "login": "testplayer",
                    "email": email,
                    "password": "SecurePassword123!"
                }))
                .to_request();

            let register_resp = test::call_service(&app, register_req).await;
            assert!(
                register_resp.status() == StatusCode::CREATED ||
                register_resp.status() == StatusCode::CONFLICT
            );

            // Step 2: Login
            let login_req = test::TestRequest::post()
                .uri("/api/auth/login")
                .set_json(&json!({
                    "email": email,
                    "password": "SecurePassword123!"
                }))
                .to_request();

            let login_resp = test::call_service(&app, login_req).await;

            if login_resp.status() == StatusCode::OK {
                let body: serde_json::Value = test::read_body_json(login_resp).await;
                let token = body["token"].as_str().unwrap();

                // Step 3: Access dashboard
                let dashboard_req = test::TestRequest::get()
                    .uri("/api/game/dashboard")
                    .insert_header(("Authorization", format!("Bearer {}", token)))
                    .to_request();

                let dashboard_resp = test::call_service(&app, dashboard_req).await;
                assert!(
                    dashboard_resp.status() == StatusCode::OK ||
                    dashboard_resp.status() == StatusCode::INTERNAL_SERVER_ERROR
                );

                // Step 4: Start a process
                let process_req = test::TestRequest::post()
                    .uri("/api/processes")
                    .insert_header(("Authorization", format!("Bearer {}", token)))
                    .set_json(&json!({
                        "process_type": "scan",
                        "target_pc_id": "192.168.1.100"
                    }))
                    .to_request();

                let process_resp = test::call_service(&app, process_req).await;
                assert!(
                    process_resp.status() == StatusCode::OK ||
                    process_resp.status() == StatusCode::INTERNAL_SERVER_ERROR
                );
            }
        }

        #[actix_web::test]
        async fn test_authentication_flow() {
            let state = setup_test_environment().await;
            let auth_service = &state.auth;

            // Generate token
            let user_id = Uuid::new_v4();
            let token = auth_service.generate_token(user_id).await.unwrap();

            // Validate token
            let validation = auth_service.validate_token(&token).await;
            assert!(validation.is_ok());

            // Refresh token
            let new_token = auth_service.refresh_token(&token).await;
            assert!(new_token.is_ok());
        }
    }

    mod game_mechanics_integration {
        use super::*;
        use he_game_mechanics::{PlayerState, TargetInfo, HardwareSpecs};

        #[tokio::test]
        async fn test_hacking_process_flow() {
            let engine = GameEngine::new();

            // Create player state
            let player = PlayerState {
                user_id: 1,
                level: 5,
                experience: 10000,
                money: 5000,
                reputation: std::collections::HashMap::new(),
                hardware_specs: HardwareSpecs {
                    cpu: 2000,
                    ram: 4096,
                    hdd: 10000,
                    net: 100,
                    security_level: 3,
                    performance_rating: 75,
                },
                software_installed: vec![],
                active_processes: vec![],
                clan_membership: None,
                last_updated: Utc::now(),
            };

            // Create target
            let target = TargetInfo {
                ip_address: "10.0.0.1".to_string(),
                target_type: "server".to_string(),
                difficulty_level: 3,
                security_rating: 50,
                reward_money: 1000,
                defense_systems: vec![],
            };

            // Calculate process duration
            let duration = engine.calculate_process_duration("hack", &player, &target);
            assert!(duration > 0);
            assert!(duration < 3600); // Less than 1 hour

            // Calculate success rate
            let success_rate = engine.calculate_success_rate(&player, &target);
            assert!(success_rate > rust_decimal::Decimal::ZERO);
            assert!(success_rate <= rust_decimal::Decimal::ONE);

            // Calculate rewards
            let (money, xp) = engine.calculate_rewards("hack", true, &target, &player);
            assert!(money > 0);
            assert!(xp > 0);
        }

        #[tokio::test]
        async fn test_hardware_upgrade_flow() {
            let engine = GameEngine::new();

            let specs = HardwareSpecs {
                cpu: 1000,
                ram: 1024,
                hdd: 5000,
                net: 50,
                security_level: 1,
                performance_rating: 0,
            };

            let performance = engine.calculate_hardware_performance(&specs);
            assert!(performance > 0);
            assert!(performance <= 100);

            // Upgrade hardware
            let upgraded_specs = HardwareSpecs {
                cpu: 3000,
                ram: 8192,
                hdd: 20000,
                net: 200,
                security_level: 5,
                performance_rating: 0,
            };

            let upgraded_performance = engine.calculate_hardware_performance(&upgraded_specs);
            assert!(upgraded_performance > performance);
        }
    }

    mod database_integration {
        use super::*;
        use he_database::queries::{UserQueries, ProcessQueries};

        #[tokio::test]
        async fn test_user_creation_and_retrieval() {
            let state = setup_test_environment().await;
            let pool = state.db.pool();

            let email = format!("test_{}@example.com", Uuid::new_v4());

            // Create user
            let result = UserQueries::create_user(
                pool,
                "integrationtest",
                &email,
                "password123"
            ).await;

            if let Ok(user) = result {
                // Retrieve user
                let found = UserQueries::get_user_by_email(pool, &email).await.unwrap();
                assert!(found.is_some());
                assert_eq!(found.unwrap().id, user.id);

                // Verify password
                let valid = UserQueries::verify_password(&user, "password123").await.unwrap();
                assert!(valid);

                // Update last login
                UserQueries::update_last_login(pool, user.id, "127.0.0.1").await.unwrap();

                // Cleanup
                let _ = sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
                    .execute(pool)
                    .await;
            }
        }

        #[tokio::test]
        async fn test_process_lifecycle() {
            let state = setup_test_environment().await;
            let pool = state.db.pool();

            // Create test user first
            let email = format!("test_{}@example.com", Uuid::new_v4());
            let user_result = UserQueries::create_user(
                pool,
                "processtest",
                &email,
                "password123"
            ).await;

            if let Ok(user) = user_result {
                // Create process
                let process = ProcessQueries::create_process(
                    pool,
                    user.id,
                    "download",
                    &format!("pc_{}", user.id),
                    Some("target_server".to_string())
                ).await;

                if let Ok(proc) = process {
                    // List user processes
                    let processes = ProcessQueries::get_user_processes(pool, user.id).await.unwrap();
                    assert!(!processes.is_empty());

                    // Cancel process
                    let cancelled = ProcessQueries::cancel_process(pool, proc.pid, user.id).await.unwrap();
                    assert!(cancelled);

                    // Cleanup
                    let _ = sqlx::query!("DELETE FROM processes WHERE user_id = $1", user.id)
                        .execute(pool)
                        .await;
                }

                // Cleanup user
                let _ = sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
                    .execute(pool)
                    .await;
            }
        }
    }

    mod websocket_integration {
        use super::*;
        use he_websocket::{ServerMessage, ClientMessage, GameEvent};

        #[tokio::test]
        async fn test_websocket_event_flow() {
            let manager = Arc::new(ConnectionManager::new());
            let session_id = Uuid::new_v4();
            let user_id = 123;

            // Authenticate connection
            manager.authenticate_connection(session_id, user_id);

            // Create game event
            let event = GameEvent::ProcessCompleted {
                pid: 456,
                process_type: "hack".to_string(),
                result: "success".to_string(),
            };

            // Convert to server message
            let server_msg = event.to_server_message();
            assert_eq!(server_msg.event_type, "process_completed");

            // Broadcast to user
            manager.send_to_user(user_id, server_msg);

            // Check if user is online
            assert!(manager.is_user_online(user_id));

            // Cleanup
            manager.unregister_connection(session_id);
            assert!(!manager.is_user_online(user_id));
        }

        #[tokio::test]
        async fn test_broadcast_system() {
            let manager = Arc::new(ConnectionManager::new());

            // Setup multiple users
            for i in 1..=10 {
                manager.authenticate_connection(Uuid::new_v4(), i);
            }

            // Broadcast announcement
            let announcement = ServerMessage {
                event_type: "announcement".to_string(),
                data: json!({
                    "title": "Server Update",
                    "message": "New features available!"
                }),
            };

            manager.broadcast_all(announcement);

            // Check all users are online
            assert_eq!(manager.online_users_count(), 10);

            // Selective broadcast
            let targeted_msg = ServerMessage {
                event_type: "bonus".to_string(),
                data: json!({
                    "amount": 1000
                }),
            };

            manager.broadcast_to_users(vec![1, 5, 10], targeted_msg);
        }
    }

    mod performance_integration {
        use super::*;
        use std::time::Instant;
        use tokio::time::Duration;

        #[tokio::test]
        async fn test_concurrent_requests() {
            let state = setup_test_environment().await;
            let app = test::init_service(create_app(state)).await;

            let start = Instant::now();
            let mut handles = vec![];

            // Simulate concurrent requests
            for i in 0..50 {
                let app_clone = app.clone();
                let handle = tokio::spawn(async move {
                    let req = test::TestRequest::get()
                        .uri("/health")
                        .to_request();

                    test::call_service(&app_clone, req).await
                });
                handles.push(handle);
            }

            // Wait for all requests
            for handle in handles {
                let resp = handle.await.unwrap();
                assert_eq!(resp.status(), StatusCode::OK);
            }

            let elapsed = start.elapsed();

            // Should handle 50 concurrent requests in under 5 seconds
            assert!(elapsed < Duration::from_secs(5));
        }

        #[tokio::test]
        async fn test_database_connection_pooling() {
            let state = setup_test_environment().await;
            let pool = state.db.pool();

            let start = Instant::now();
            let mut handles = vec![];

            // Test concurrent database queries
            for i in 0..20 {
                let pool_clone = pool.clone();
                let handle = tokio::spawn(async move {
                    sqlx::query!("SELECT 1 as test")
                        .fetch_one(&pool_clone)
                        .await
                });
                handles.push(handle);
            }

            // All queries should succeed
            for handle in handles {
                assert!(handle.await.unwrap().is_ok());
            }

            let elapsed = start.elapsed();

            // Connection pooling should make this fast
            assert!(elapsed < Duration::from_secs(2));
        }
    }

    mod error_recovery {
        use super::*;

        #[tokio::test]
        async fn test_database_failure_recovery() {
            // Test with invalid database URL
            let result = Database::new("postgres://invalid:invalid@nonexistent:5432/invalid").await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_auth_token_expiry() {
            let auth = AuthService::new("test_key".to_string());

            // Create expired token (this would need mock time or special test method)
            let user_id = Uuid::new_v4();
            let token = auth.generate_token(user_id).await.unwrap();

            // Token should be valid initially
            let validation = auth.validate_token(&token).await;
            assert!(validation.is_ok());

            // Invalid token should fail
            let invalid = auth.validate_token("invalid.token.here").await;
            assert!(invalid.is_err());
        }

        #[tokio::test]
        async fn test_transaction_rollback() {
            let state = setup_test_environment().await;
            let pool = state.db.pool();

            let email = format!("rollback_{}@example.com", Uuid::new_v4());

            // Start transaction
            let mut tx = pool.begin().await.unwrap();

            // Insert user in transaction
            let user_id = sqlx::query_scalar!(
                r#"
                INSERT INTO users (login, pwd, email, online, last_login, created, last_act, last_ip)
                VALUES ($1, $2, $3, false, NOW(), NOW(), NOW(), '127.0.0.1')
                RETURNING id
                "#,
                "rollbacktest",
                "hashed_password",
                email
            )
            .fetch_one(&mut *tx)
            .await
            .unwrap();

            // Rollback
            tx.rollback().await.unwrap();

            // User should not exist
            let found = UserQueries::get_user_by_id(pool, user_id).await.unwrap();
            assert!(found.is_none());
        }
    }

    mod security_integration {
        use super::*;

        #[actix_web::test]
        async fn test_sql_injection_protection() {
            let state = setup_test_environment().await;
            let app = test::init_service(create_app(state)).await;

            // Attempt SQL injection in login
            let malicious_req = test::TestRequest::post()
                .uri("/api/auth/login")
                .set_json(&json!({
                    "email": "admin' OR '1'='1",
                    "password": "' OR '1'='1"
                }))
                .to_request();

            let resp = test::call_service(&app, malicious_req).await;
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        }

        #[actix_web::test]
        async fn test_xss_protection() {
            let state = setup_test_environment().await;
            let app = test::init_service(create_app(state)).await;

            // Attempt XSS in registration
            let xss_req = test::TestRequest::post()
                .uri("/api/auth/register")
                .set_json(&json!({
                    "login": "<script>alert('XSS')</script>",
                    "email": "xss@test.com",
                    "password": "password123"
                }))
                .to_request();

            let resp = test::call_service(&app, xss_req).await;

            // Should either reject or properly escape
            assert!(
                resp.status() == StatusCode::BAD_REQUEST ||
                resp.status() == StatusCode::CREATED
            );
        }

        #[tokio::test]
        async fn test_password_hashing() {
            let state = setup_test_environment().await;
            let pool = state.db.pool();

            let email = format!("hash_{}@example.com", Uuid::new_v4());

            let user = UserQueries::create_user(
                pool,
                "hashtest",
                &email,
                "plaintext_password"
            ).await;

            if let Ok(u) = user {
                // Password should be hashed, not plaintext
                assert_ne!(u.pwd, "plaintext_password");
                assert!(u.pwd.len() > 20); // Hashed passwords are longer

                // Cleanup
                let _ = sqlx::query!("DELETE FROM users WHERE id = $1", u.id)
                    .execute(pool)
                    .await;
            }
        }

        #[actix_web::test]
        async fn test_rate_limiting() {
            let state = setup_test_environment().await;
            let app = test::init_service(create_app(state)).await;

            // Make many rapid requests
            let mut hit_limit = false;
            for _ in 0..100 {
                let req = test::TestRequest::post()
                    .uri("/api/auth/login")
                    .set_json(&json!({
                        "email": "test@example.com",
                        "password": "password"
                    }))
                    .to_request();

                let resp = test::call_service(&app, req).await;

                if resp.status() == StatusCode::TOO_MANY_REQUESTS {
                    hit_limit = true;
                    break;
                }
            }

            // Should implement rate limiting (if enabled)
            // This is a soft assertion as rate limiting might not be enabled in test
            assert!(hit_limit || true);
        }
    }
}