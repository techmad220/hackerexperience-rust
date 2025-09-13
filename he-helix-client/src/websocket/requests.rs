//! WebSocket request handlers for clients

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request to set up a client connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupRequestPayload {
    pub client_type: String,
    pub session_token: Option<String>,
    pub user_agent: Option<String>,
}

/// Response to setup request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupResponsePayload {
    pub client_id: String,
    pub session_id: String,
    pub server_time: chrono::DateTime<chrono::Utc>,
}

/// Generic action request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRequestPayload {
    pub action_type: String,
    pub target: Option<String>,
    pub parameters: Option<serde_json::Value>,
}

/// Action response payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResponsePayload {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Request handlers for various client operations
pub struct ClientRequestHandler;

impl ClientRequestHandler {
    /// Handle client setup request
    pub async fn handle_setup(
        payload: SetupRequestPayload,
    ) -> crate::ClientResult<SetupResponsePayload> {
        let client_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();

        tracing::info!(
            "Setting up client: type={}, client_id={}, session_id={}",
            payload.client_type,
            client_id,
            session_id
        );

        Ok(SetupResponsePayload {
            client_id: client_id.to_string(),
            session_id: session_id.to_string(),
            server_time: chrono::Utc::now(),
        })
    }

    /// Handle client action request
    pub async fn handle_action(
        payload: ActionRequestPayload,
    ) -> crate::ClientResult<ActionResponsePayload> {
        tracing::info!(
            "Processing client action: type={}, target={:?}",
            payload.action_type,
            payload.target
        );

        // Process different action types
        match payload.action_type.as_str() {
            "bootstrap" => Ok(ActionResponsePayload {
                success: true,
                message: "Client bootstrap completed".to_string(),
                data: Some(serde_json::json!({"status": "ready"})),
            }),
            "heartbeat" => Ok(ActionResponsePayload {
                success: true,
                message: "Heartbeat received".to_string(),
                data: Some(serde_json::json!({"timestamp": chrono::Utc::now()})),
            }),
            _ => Ok(ActionResponsePayload {
                success: true,
                message: format!("Action '{}' processed", payload.action_type),
                data: None,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_setup_handler() {
        let payload = SetupRequestPayload {
            client_type: "web1".to_string(),
            session_token: Some("test_token".to_string()),
            user_agent: Some("Test Agent".to_string()),
        };

        let response = ClientRequestHandler::handle_setup(payload).await.unwrap();

        assert!(!response.client_id.is_empty());
        assert!(!response.session_id.is_empty());
        assert!(response.server_time <= chrono::Utc::now());
    }

    #[tokio::test]
    async fn test_bootstrap_action() {
        let payload = ActionRequestPayload {
            action_type: "bootstrap".to_string(),
            target: None,
            parameters: None,
        };

        let response = ClientRequestHandler::handle_action(payload).await.unwrap();

        assert!(response.success);
        assert_eq!(response.message, "Client bootstrap completed");
        assert!(response.data.is_some());
    }

    #[tokio::test]
    async fn test_heartbeat_action() {
        let payload = ActionRequestPayload {
            action_type: "heartbeat".to_string(),
            target: None,
            parameters: None,
        };

        let response = ClientRequestHandler::handle_action(payload).await.unwrap();

        assert!(response.success);
        assert_eq!(response.message, "Heartbeat received");
        assert!(response.data.is_some());
    }
}