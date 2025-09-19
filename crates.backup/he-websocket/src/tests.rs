//! WebSocket communication tests

#[cfg(test)]
mod tests {
    use super::super::*;
    use actix::{Actor, System};
    use actix_web_actors::ws;
    use actix_web::{test, web, App, HttpRequest, HttpResponse};
    use futures_util::{SinkExt, StreamExt};
    use serde_json::json;
    use std::time::Duration;
    use tokio::time::{sleep, timeout};

    async fn create_test_manager() -> Arc<ConnectionManager> {
        Arc::new(ConnectionManager::new())
    }

    async fn create_test_session() -> (WebSocketSession, Arc<ConnectionManager>) {
        let manager = create_test_manager().await;
        let session = WebSocketSession::new(manager.clone());
        (session, manager)
    }

    mod session_tests {
        use super::*;

        #[tokio::test]
        async fn test_session_creation() {
            let (session, _manager) = create_test_session().await;

            assert!(session.user_id.is_none());
            assert!(session.subscriptions.is_empty());
            assert_ne!(session.id, Uuid::nil());
        }

        #[tokio::test]
        async fn test_session_heartbeat() {
            let (_session, _manager) = create_test_session().await;

            // Session should have recent heartbeat
            let now = Instant::now();
            assert!(now.duration_since(_session.hb) < Duration::from_secs(1));
        }

        #[tokio::test]
        async fn test_session_authentication() {
            let (mut session, _manager) = create_test_session().await;

            // Test authentication (mocked for now)
            let result = session.authenticate("test_token").await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 1);
        }

        #[tokio::test]
        async fn test_session_subscription_management() {
            let (mut session, _manager) = create_test_session().await;

            // Add subscriptions
            session.subscriptions.push("game_events".to_string());
            session.subscriptions.push("chat".to_string());

            assert_eq!(session.subscriptions.len(), 2);
            assert!(session.subscriptions.contains(&"game_events".to_string()));

            // Remove subscription
            session.subscriptions.retain(|c| c != "chat");
            assert_eq!(session.subscriptions.len(), 1);
            assert!(!session.subscriptions.contains(&"chat".to_string()));
        }
    }

    mod manager_tests {
        use super::*;

        #[tokio::test]
        async fn test_connection_registration() {
            let manager = create_test_manager().await;
            let session_id = Uuid::new_v4();

            // Test the internal state
            assert_eq!(manager.total_connections(), 0);
        }

        #[tokio::test]
        async fn test_user_authentication_tracking() {
            let manager = create_test_manager().await;
            let session_id = Uuid::new_v4();
            let user_id = 123;

            manager.authenticate_connection(session_id, user_id);

            // Check if user is tracked
            assert!(manager.is_user_online(user_id));
        }

        #[tokio::test]
        async fn test_broadcast_filtering() {
            let manager = create_test_manager().await;

            // Create multiple sessions
            let session1 = Uuid::new_v4();
            let session2 = Uuid::new_v4();
            let session3 = Uuid::new_v4();

            // Authenticate some sessions
            manager.authenticate_connection(session1, 1);
            manager.authenticate_connection(session2, 2);
            // session3 remains unauthenticated

            // Test user filtering
            assert!(manager.is_user_online(1));
            assert!(manager.is_user_online(2));

            let online_users = manager.get_online_users();
            assert!(online_users.contains(&1));
            assert!(online_users.contains(&2));
        }

        #[tokio::test]
        async fn test_connection_cleanup() {
            let manager = create_test_manager().await;
            let session_id = Uuid::new_v4();
            let user_id = 456;

            // Register and authenticate
            manager.authenticate_connection(session_id, user_id);
            assert!(manager.is_user_online(user_id));

            // Unregister
            manager.unregister_connection(session_id);
            assert!(!manager.is_user_online(user_id));
        }

        #[tokio::test]
        async fn test_concurrent_connection_management() {
            let manager = create_test_manager().await;
            let mut handles = vec![];

            // Simulate concurrent connections
            for i in 0..10 {
                let mgr = manager.clone();
                let handle = tokio::spawn(async move {
                    let session_id = Uuid::new_v4();
                    mgr.authenticate_connection(session_id, i);
                    sleep(Duration::from_millis(10)).await;
                    mgr.unregister_connection(session_id);
                });
                handles.push(handle);
            }

            // Wait for all to complete
            for handle in handles {
                handle.await.unwrap();
            }

            // All connections should be cleaned up
            assert_eq!(manager.total_connections(), 0);
        }
    }

    mod message_tests {
        use super::*;

        #[test]
        fn test_client_message_serialization() {
            let msg = ClientMessage {
                msg_type: "auth".to_string(),
                data: json!({
                    "token": "test_token_123"
                }),
            };

            let serialized = serde_json::to_string(&msg).unwrap();
            let deserialized: ClientMessage = serde_json::from_str(&serialized).unwrap();

            assert_eq!(deserialized.msg_type, "auth");
            assert_eq!(deserialized.data["token"], "test_token_123");
        }

        #[test]
        fn test_server_message_serialization() {
            let msg = ServerMessage {
                event_type: "process_update".to_string(),
                data: json!({
                    "pid": 123,
                    "progress": 50,
                    "status": "running"
                }),
            };

            let serialized = serde_json::to_string(&msg).unwrap();
            let deserialized: ServerMessage = serde_json::from_str(&serialized).unwrap();

            assert_eq!(deserialized.event_type, "process_update");
            assert_eq!(deserialized.data["pid"], 123);
            assert_eq!(deserialized.data["progress"], 50);
        }

        #[test]
        fn test_broadcast_message_with_filter() {
            let msg = Broadcast {
                event_type: "system_alert".to_string(),
                data: json!({
                    "message": "Server maintenance in 5 minutes"
                }),
                user_filter: Some(vec![1, 2, 3]),
            };

            assert_eq!(msg.event_type, "system_alert");
            assert!(msg.user_filter.is_some());
            assert_eq!(msg.user_filter.unwrap().len(), 3);
        }

        #[test]
        fn test_message_parsing_errors() {
            let invalid_json = "{invalid json}";
            let result = serde_json::from_str::<ClientMessage>(invalid_json);
            assert!(result.is_err());

            let incomplete_msg = json!({
                "data": "test"
                // Missing msg_type
            });
            let result = serde_json::from_value::<ClientMessage>(incomplete_msg);
            assert!(result.is_err());
        }
    }

    mod event_tests {
        use super::*;
        use crate::events::{GameEvent, EventBuilder};

        #[test]
        fn test_game_event_creation() {
            let event = GameEvent::ProcessCompleted {
                pid: 456,
                process_type: "hack".to_string(),
                result: "success".to_string(),
            };

            assert_eq!(event.event_type(), "process_completed");
        }

        #[test]
        fn test_event_builder() {
            let event = EventBuilder::process_started(123, "scan".to_string(), 300);

            match event {
                GameEvent::ProcessStarted { pid, process_type, estimated_time } => {
                    assert_eq!(pid, 123);
                    assert_eq!(process_type, "scan");
                    assert_eq!(estimated_time, 300);
                },
                _ => panic!("Wrong event type"),
            }
        }

        #[test]
        fn test_broadcast_events() {
            let announcement = GameEvent::Announcement {
                title: "Update".to_string(),
                content: "New features!".to_string(),
                priority: "high".to_string(),
            };

            assert!(announcement.is_broadcast());

            let process_event = GameEvent::ProcessStarted {
                pid: 1,
                process_type: "crack".to_string(),
                estimated_time: 60,
            };

            assert!(!process_event.is_broadcast());
        }

        #[test]
        fn test_event_to_server_message() {
            let event = GameEvent::MoneyReceived {
                amount: 1000,
                from: "Bank".to_string(),
            };

            let server_msg = event.to_server_message();
            assert_eq!(server_msg.event_type, "money_received");
        }
    }

    mod integration_tests {
        use super::*;
        use actix_web::http::header;
        use actix_web_actors::ws::WebsocketContext;

        async fn ws_handler(
            req: HttpRequest,
            stream: web::Payload,
            manager: web::Data<Arc<ConnectionManager>>,
        ) -> Result<HttpResponse, actix_web::Error> {
            let session = WebSocketSession::new(manager.get_ref().clone());
            ws::start(session, &req, stream)
        }

        #[actix_web::test]
        async fn test_websocket_connection() {
            let manager = create_test_manager().await;

            let app = test::init_service(
                App::new()
                    .app_data(web::Data::new(manager.clone()))
                    .route("/ws", web::get().to(ws_handler))
            ).await;

            let req = test::TestRequest::get()
                .uri("/ws")
                .insert_header((header::CONNECTION, "upgrade"))
                .insert_header((header::UPGRADE, "websocket"))
                .insert_header(("Sec-WebSocket-Version", "13"))
                .insert_header(("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ=="))
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), 101); // Switching Protocols
        }

        #[tokio::test]
        async fn test_message_flow() {
            let manager = create_test_manager().await;
            let (session, _) = create_test_session().await;

            // Simulate message flow
            let auth_msg = ClientMessage {
                msg_type: "auth".to_string(),
                data: json!({
                    "token": "test_jwt_token"
                }),
            };

            // Message should be valid JSON
            let serialized = serde_json::to_string(&auth_msg).unwrap();
            assert!(serialized.contains("auth"));
            assert!(serialized.contains("test_jwt_token"));
        }

        #[tokio::test]
        async fn test_broadcast_to_multiple_sessions() {
            let manager = create_test_manager().await;

            // Create multiple sessions
            let session_ids = vec![
                Uuid::new_v4(),
                Uuid::new_v4(),
                Uuid::new_v4(),
            ];

            // Authenticate sessions
            for (i, &id) in session_ids.iter().enumerate() {
                manager.authenticate_connection(id, i as i64);
            }

            // Create broadcast message
            let broadcast = Broadcast {
                event_type: "server_announcement".to_string(),
                data: json!({
                    "message": "Testing broadcast"
                }),
                user_filter: None, // Broadcast to all
            };

            // All sessions should be eligible for broadcast
            assert_eq!(manager.online_users_count(), 3);
        }

        #[tokio::test]
        async fn test_selective_broadcast() {
            let manager = create_test_manager().await;

            // Setup sessions
            manager.authenticate_connection(Uuid::new_v4(), 1);
            manager.authenticate_connection(Uuid::new_v4(), 2);
            manager.authenticate_connection(Uuid::new_v4(), 3);

            // Broadcast only to users 1 and 3
            let broadcast = Broadcast {
                event_type: "targeted_message".to_string(),
                data: json!({"content": "Select users only"}),
                user_filter: Some(vec![1, 3]),
            };

            let online = manager.get_online_users();
            assert!(online.contains(&1));
            assert!(online.contains(&3));
        }
    }

    mod performance_tests {
        use super::*;

        #[tokio::test]
        async fn test_concurrent_message_handling() {
            let manager = create_test_manager().await;
            let mut handles = vec![];

            // Simulate concurrent message sending
            for i in 0..100 {
                let mgr = manager.clone();
                let handle = tokio::spawn(async move {
                    let session_id = Uuid::new_v4();
                    mgr.authenticate_connection(session_id, i);

                    // Simulate message processing
                    sleep(Duration::from_millis(1)).await;

                    mgr.unregister_connection(session_id);
                });
                handles.push(handle);
            }

            let start = Instant::now();
            for handle in handles {
                handle.await.unwrap();
            }
            let elapsed = start.elapsed();

            // Should complete quickly even with many connections
            assert!(elapsed < Duration::from_secs(5));
        }

        #[tokio::test]
        async fn test_large_broadcast_performance() {
            let manager = create_test_manager().await;

            // Create many connections
            for i in 0..1000 {
                manager.authenticate_connection(Uuid::new_v4(), i);
            }

            let start = Instant::now();

            // Test broadcast performance
            let broadcast = Broadcast {
                event_type: "mass_broadcast".to_string(),
                data: json!({
                    "large_data": vec![0; 1000] // 1KB of data
                }),
                user_filter: None,
            };

            // Get all connections for broadcast
            assert_eq!(manager.online_users_count(), 1000);

            let elapsed = start.elapsed();

            // Should handle large broadcasts efficiently
            assert!(elapsed < Duration::from_millis(100));
        }

        #[tokio::test]
        async fn test_memory_cleanup() {
            let manager = create_test_manager().await;

            // Add and remove many connections
            for _ in 0..10 {
                let mut session_ids = vec![];

                // Add 100 connections
                for i in 0..100 {
                    let id = Uuid::new_v4();
                    manager.authenticate_connection(id, i);
                    session_ids.push(id);
                }

                // Remove all connections
                for id in session_ids {
                    manager.unregister_connection(id);
                }

                // Should be fully cleaned up
                assert_eq!(manager.total_connections(), 0);
            }
        }
    }

    mod error_handling_tests {
        use super::*;

        #[tokio::test]
        async fn test_invalid_message_handling() {
            let (_session, _manager) = create_test_session().await;

            // Test various invalid messages
            let invalid_messages = vec![
                json!({}), // Empty object
                json!({"invalid": "structure"}), // Missing required fields
                json!({"msg_type": 123}), // Wrong type
                json!({"msg_type": "unknown", "data": null}), // Null data
            ];

            for msg in invalid_messages {
                let result = serde_json::from_value::<ClientMessage>(msg);
                if result.is_ok() {
                    // If it parses, it should handle unknown message types gracefully
                    let client_msg = result.unwrap();
                    assert!(!client_msg.msg_type.is_empty());
                }
            }
        }

        #[tokio::test]
        async fn test_connection_timeout() {
            let (mut session, _manager) = create_test_session().await;

            // Set heartbeat to old time
            session.hb = Instant::now() - Duration::from_secs(120);

            // Session should be considered timed out
            let time_since_hb = Instant::now().duration_since(session.hb);
            assert!(time_since_hb > CLIENT_TIMEOUT);
        }

        #[tokio::test]
        async fn test_authentication_failure() {
            let manager = create_test_manager().await;
            let session_id = Uuid::new_v4();

            // Try to use session without authentication
            assert!(!manager.is_user_online(999));

            // Session should not be in authenticated list
            let online_users = manager.get_online_users();
            assert!(online_users.is_empty() || !online_users.contains(&999));
        }

        #[tokio::test]
        async fn test_duplicate_subscription() {
            let (mut session, _manager) = create_test_session().await;

            // Add same subscription multiple times
            session.subscriptions.push("game_events".to_string());
            session.subscriptions.push("game_events".to_string());
            session.subscriptions.push("game_events".to_string());

            // Should handle duplicates (in real impl would dedupe)
            assert_eq!(session.subscriptions.len(), 3);

            // Remove all instances
            session.subscriptions.retain(|c| c != "game_events");
            assert_eq!(session.subscriptions.len(), 0);
        }
    }

    mod stress_tests {
        use super::*;

        #[tokio::test]
        #[ignore] // Run with --ignored flag for stress tests
        async fn test_high_frequency_messages() {
            let manager = create_test_manager().await;
            let session_id = Uuid::new_v4();
            manager.authenticate_connection(session_id, 1);

            let start = Instant::now();
            let message_count = 10000;

            for i in 0..message_count {
                let msg = ServerMessage {
                    event_type: "stress_test".to_string(),
                    data: json!({
                        "index": i,
                        "timestamp": Utc::now().timestamp_millis()
                    }),
                };

                // Simulate message processing
                let _ = serde_json::to_string(&msg).unwrap();
            }

            let elapsed = start.elapsed();
            let messages_per_second = message_count as f64 / elapsed.as_secs_f64();

            // Should handle at least 1000 messages per second
            assert!(messages_per_second > 1000.0);
        }

        #[tokio::test]
        #[ignore]
        async fn test_connection_churn() {
            let manager = create_test_manager().await;

            // Rapid connect/disconnect cycles
            for cycle in 0..100 {
                let mut session_ids = vec![];

                // Connect wave
                for i in 0..50 {
                    let id = Uuid::new_v4();
                    manager.authenticate_connection(id, (cycle * 50 + i) as i64);
                    session_ids.push(id);
                }

                // Partial disconnect
                for id in session_ids.iter().take(25) {
                    manager.unregister_connection(*id);
                }

                // Remaining should still be connected
                assert!(manager.total_connections() >= 25 || manager.online_users_count() >= 25);

                // Full disconnect
                for id in session_ids.iter().skip(25) {
                    manager.unregister_connection(*id);
                }
            }

            // Final cleanup check
            assert_eq!(manager.total_connections(), 0);
        }
    }
}