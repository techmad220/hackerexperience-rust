//! Web1 WebSocket specific handlers

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Web1 WebSocket request types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Web1WebSocketRequest {
    Setup(Web1SetupRequest),
    TutorialAction(TutorialActionRequest),
    AppLaunch(AppLaunchRequest),
}

/// Web1 setup request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web1SetupRequest {
    pub entity_id: Uuid,
    pub session_token: String,
}

/// Tutorial action request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialActionRequest {
    pub action: String, // "next", "prev", "skip"
    pub current_step: u32,
}

/// App launch request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppLaunchRequest {
    pub app_name: String,
    pub parameters: Option<serde_json::Value>,
}

/// Web1 WebSocket handler
pub struct Web1WebSocketHandler;

impl Web1WebSocketHandler {
    /// Handle Web1 WebSocket request
    pub async fn handle_request(
        request: Web1WebSocketRequest,
    ) -> crate::ClientResult<serde_json::Value> {
        match request {
            Web1WebSocketRequest::Setup(setup) => {
                Self::handle_setup(setup).await
            }
            Web1WebSocketRequest::TutorialAction(tutorial) => {
                Self::handle_tutorial_action(tutorial).await
            }
            Web1WebSocketRequest::AppLaunch(app_launch) => {
                Self::handle_app_launch(app_launch).await
            }
        }
    }

    async fn handle_setup(request: Web1SetupRequest) -> crate::ClientResult<serde_json::Value> {
        tracing::info!("Web1 setup for entity: {}", request.entity_id);
        
        Ok(serde_json::json!({
            "type": "setup_complete",
            "entity_id": request.entity_id,
            "client_type": "web1",
            "setup_data": {
                "tutorial_enabled": true,
                "first_time_user": false
            }
        }))
    }

    async fn handle_tutorial_action(
        request: TutorialActionRequest,
    ) -> crate::ClientResult<serde_json::Value> {
        tracing::info!("Tutorial action: {} at step {}", request.action, request.current_step);

        let next_step = match request.action.as_str() {
            "next" => request.current_step + 1,
            "prev" => request.current_step.saturating_sub(1),
            "skip" => 999, // Skip to end
            _ => request.current_step,
        };

        Ok(serde_json::json!({
            "type": "tutorial_update",
            "action": request.action,
            "current_step": next_step,
            "completed": next_step >= 10
        }))
    }

    async fn handle_app_launch(
        request: AppLaunchRequest,
    ) -> crate::ClientResult<serde_json::Value> {
        tracing::info!("Launching app: {}", request.app_name);

        Ok(serde_json::json!({
            "type": "app_launched",
            "app_name": request.app_name,
            "success": true,
            "window_id": Uuid::new_v4()
        }))
    }
}