//! WebSocket handling for clients

pub mod requests;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{ClientError, ClientResult};

/// WebSocket request types for clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientWebSocketRequest {
    Setup(SetupRequest),
    Action(ActionRequest),
}

/// Setup request for client initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupRequest {
    pub client_type: String,
    pub entity_id: Option<Uuid>,
}

/// Action request for client actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRequest {
    pub action: String,
    pub payload: Option<serde_json::Value>,
}

/// WebSocket response types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientWebSocketResponse {
    Success { message: String },
    Error { error: String },
    Setup { client_id: String },
}

/// WebSocket handler for client requests
pub struct ClientWebSocketHandler;

impl ClientWebSocketHandler {
    /// Create a new WebSocket handler
    pub fn new() -> Self {
        Self
    }

    /// Handle incoming WebSocket request
    pub async fn handle_request(
        &self,
        request: ClientWebSocketRequest,
    ) -> ClientResult<ClientWebSocketResponse> {
        match request {
            ClientWebSocketRequest::Setup(setup_req) => {
                self.handle_setup(setup_req).await
            }
            ClientWebSocketRequest::Action(action_req) => {
                self.handle_action(action_req).await
            }
        }
    }

    /// Handle setup request
    async fn handle_setup(&self, request: SetupRequest) -> ClientResult<ClientWebSocketResponse> {
        // Validate client type
        if !crate::model::Client::is_valid_client(&request.client_type) {
            return Err(ClientError::InvalidClientType {
                client_type: request.client_type,
            });
        }

        // Generate client ID
        let client_id = Uuid::new_v4();

        tracing::info!(
            "Client setup: {} (type: {}, entity: {:?})",
            client_id,
            request.client_type,
            request.entity_id
        );

        Ok(ClientWebSocketResponse::Setup {
            client_id: client_id.to_string(),
        })
    }

    /// Handle action request
    async fn handle_action(&self, request: ActionRequest) -> ClientResult<ClientWebSocketResponse> {
        tracing::info!("Client action: {}", request.action);

        // Process action (placeholder implementation)
        match request.action.as_str() {
            "ping" => Ok(ClientWebSocketResponse::Success {
                message: "pong".to_string(),
            }),
            _ => Ok(ClientWebSocketResponse::Success {
                message: format!("Action '{}' processed", request.action),
            }),
        }
    }
}

impl Default for ClientWebSocketHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_setup_request() {
        let handler = ClientWebSocketHandler::new();
        let request = ClientWebSocketRequest::Setup(SetupRequest {
            client_type: "web1".to_string(),
            entity_id: Some(Uuid::new_v4()),
        });

        let response = handler.handle_request(request).await.unwrap();
        
        match response {
            ClientWebSocketResponse::Setup { client_id } => {
                assert!(!client_id.is_empty());
            }
            _ => panic!("Expected Setup response"),
        }
    }

    #[tokio::test]
    async fn test_invalid_client_type() {
        let handler = ClientWebSocketHandler::new();
        let request = ClientWebSocketRequest::Setup(SetupRequest {
            client_type: "invalid".to_string(),
            entity_id: None,
        });

        let result = handler.handle_request(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_action_request() {
        let handler = ClientWebSocketHandler::new();
        let request = ClientWebSocketRequest::Action(ActionRequest {
            action: "ping".to_string(),
            payload: None,
        });

        let response = handler.handle_request(request).await.unwrap();
        
        match response {
            ClientWebSocketResponse::Success { message } => {
                assert_eq!(message, "pong");
            }
            _ => panic!("Expected Success response"),
        }
    }
}