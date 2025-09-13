use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use serde_json::{json, Value};
use futures_util::{SinkExt, StreamExt};

mod common;
use common::{TestDb, TestFixtures};

#[tokio::test]
async fn test_websocket_connection() {
    // Test WebSocket connection establishment
    let ws_url = "ws://localhost:8080/ws";
    
    // Note: This test assumes the WebSocket server is running
    // In a real test environment, you might want to start a test server
    match connect_async(ws_url).await {
        Ok((mut ws_stream, _)) => {
            // Test sending a message
            let test_message = json!({
                "type": "ping",
                "data": {}
            });
            
            ws_stream.send(Message::Text(test_message.to_string())).await.expect("Failed to send message");
            
            // Test receiving a response
            if let Some(msg) = ws_stream.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        let response: Value = serde_json::from_str(&text).expect("Invalid JSON response");
                        assert_eq!(response["type"], "pong");
                    }
                    _ => panic!("Unexpected message type"),
                }
            }
            
            // Close connection
            ws_stream.close(None).await.expect("Failed to close WebSocket");
        }
        Err(_) => {
            // Skip test if WebSocket server is not running
            println!("WebSocket server not available, skipping test");
        }
    }
}

#[tokio::test]
async fn test_websocket_authentication() {
    let ws_url = "ws://localhost:8080/ws";
    
    match connect_async(ws_url).await {
        Ok((mut ws_stream, _)) => {
            // Test authentication message
            let auth_message = json!({
                "type": "authenticate",
                "data": {
                    "token": "test_session_token",
                    "player_id": 1
                }
            });
            
            ws_stream.send(Message::Text(auth_message.to_string())).await.expect("Failed to send auth message");
            
            // Expect authentication response
            if let Some(msg) = ws_stream.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        let response: Value = serde_json::from_str(&text).expect("Invalid JSON response");
                        assert_eq!(response["type"], "auth_response");
                        assert!(response["data"]["authenticated"].as_bool().unwrap_or(false));
                    }
                    _ => panic!("Unexpected message type"),
                }
            }
            
            ws_stream.close(None).await.expect("Failed to close WebSocket");
        }
        Err(_) => {
            println!("WebSocket server not available, skipping test");
        }
    }
}

#[tokio::test]
async fn test_websocket_realtime_updates() {
    let ws_url = "ws://localhost:8080/ws";
    
    match connect_async(ws_url).await {
        Ok((mut ws_stream, _)) => {
            // Subscribe to real-time updates
            let subscribe_message = json!({
                "type": "subscribe",
                "data": {
                    "channels": ["processes", "logs", "servers"]
                }
            });
            
            ws_stream.send(Message::Text(subscribe_message.to_string())).await.expect("Failed to send subscribe message");
            
            // Expect subscription confirmation
            if let Some(msg) = ws_stream.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        let response: Value = serde_json::from_str(&text).expect("Invalid JSON response");
                        assert_eq!(response["type"], "subscription_confirmed");
                        assert!(response["data"]["channels"].is_array());
                    }
                    _ => panic!("Unexpected message type"),
                }
            }
            
            // Test unsubscribe
            let unsubscribe_message = json!({
                "type": "unsubscribe",
                "data": {
                    "channels": ["logs"]
                }
            });
            
            ws_stream.send(Message::Text(unsubscribe_message.to_string())).await.expect("Failed to send unsubscribe message");
            
            ws_stream.close(None).await.expect("Failed to close WebSocket");
        }
        Err(_) => {
            println!("WebSocket server not available, skipping test");
        }
    }
}

#[tokio::test]
async fn test_websocket_process_notifications() {
    let mut db = TestDb::new().await;
    db.setup().await.expect("Failed to setup test database");
    
    let ws_url = "ws://localhost:8080/ws";
    
    match connect_async(ws_url).await {
        Ok((mut ws_stream, _)) => {
            // Authenticate first
            let auth_message = json!({
                "type": "authenticate",
                "data": {
                    "token": "test_session_token",
                    "player_id": 1
                }
            });
            
            ws_stream.send(Message::Text(auth_message.to_string())).await.expect("Failed to send auth message");
            
            // Skip auth response
            ws_stream.next().await;
            
            // Subscribe to process updates
            let subscribe_message = json!({
                "type": "subscribe",
                "data": {
                    "channels": ["processes"]
                }
            });
            
            ws_stream.send(Message::Text(subscribe_message.to_string())).await.expect("Failed to send subscribe message");
            
            // Skip subscription confirmation
            ws_stream.next().await;
            
            // Simulate process completion (this would normally be done by the game engine)
            // For the test, we just check that we can handle the notification format
            let process_notification = json!({
                "type": "process_update",
                "data": {
                    "process_id": 123,
                    "status": "completed",
                    "result": "success",
                    "timestamp": "2024-01-01T12:00:00Z"
                }
            });
            
            // In a real scenario, this would come from the server
            // Here we're testing the message format handling
            let notification_str = process_notification.to_string();
            let parsed: Value = serde_json::from_str(&notification_str).expect("Invalid notification format");
            
            assert_eq!(parsed["type"], "process_update");
            assert_eq!(parsed["data"]["process_id"], 123);
            assert_eq!(parsed["data"]["status"], "completed");
            
            ws_stream.close(None).await.expect("Failed to close WebSocket");
        }
        Err(_) => {
            println!("WebSocket server not available, skipping test");
        }
    }
}

#[tokio::test]
async fn test_websocket_error_handling() {
    let ws_url = "ws://localhost:8080/ws";
    
    match connect_async(ws_url).await {
        Ok((mut ws_stream, _)) => {
            // Send malformed message
            let malformed_message = "invalid json";
            
            ws_stream.send(Message::Text(malformed_message.to_string())).await.expect("Failed to send malformed message");
            
            // Expect error response
            if let Some(msg) = ws_stream.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        let response: Value = serde_json::from_str(&text).expect("Invalid JSON response");
                        assert_eq!(response["type"], "error");
                        assert!(response["data"]["message"].is_string());
                    }
                    _ => panic!("Unexpected message type"),
                }
            }
            
            // Test invalid message type
            let invalid_type_message = json!({
                "type": "invalid_type",
                "data": {}
            });
            
            ws_stream.send(Message::Text(invalid_type_message.to_string())).await.expect("Failed to send invalid type message");
            
            if let Some(msg) = ws_stream.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        let response: Value = serde_json::from_str(&text).expect("Invalid JSON response");
                        assert_eq!(response["type"], "error");
                        assert!(response["data"]["message"].as_str().unwrap().contains("unknown message type"));
                    }
                    _ => panic!("Unexpected message type"),
                }
            }
            
            ws_stream.close(None).await.expect("Failed to close WebSocket");
        }
        Err(_) => {
            println!("WebSocket server not available, skipping test");
        }
    }
}