//! Client-related events

use he_events::{Event, EventResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::{Action, Client};

/// Client action performed event
/// 
/// Emitted when a client performs a custom action that should be tracked
/// by the backend for specific behavior (e.g., tutorial progression)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientActionPerformedEvent {
    pub client: Client,
    pub entity_id: Uuid,
    pub action: Action,
}

impl ClientActionPerformedEvent {
    /// Create a new client action performed event
    pub fn new(client: Client, entity_id: Uuid, action: Action) -> Self {
        Self {
            client,
            entity_id,
            action,
        }
    }
}

impl Event for ClientActionPerformedEvent {
    fn name(&self) -> &'static str {
        "client_action_performed"
    }

    fn version(&self) -> u32 {
        1
    }
}

/// Event handler for client actions
#[async_trait::async_trait]
pub trait ClientActionHandler {
    /// Handle a client action performed event
    async fn handle_client_action(&self, event: &ClientActionPerformedEvent) -> EventResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_action_event_creation() {
        let entity_id = Uuid::new_v4();
        let action = Action::OpenApp("terminal".to_string());
        
        let event = ClientActionPerformedEvent::new(Client::Web1, entity_id, action.clone());
        
        assert_eq!(event.client, Client::Web1);
        assert_eq!(event.entity_id, entity_id);
        assert_eq!(event.action, action);
        assert_eq!(event.name(), "client_action_performed");
        assert_eq!(event.version(), 1);
    }
}