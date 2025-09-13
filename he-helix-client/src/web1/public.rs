//! Web1 public API and bootstrap functionality

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Bootstrap configuration for Web1 client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapConfig {
    pub client_id: Uuid,
    pub entity_id: Uuid,
    pub server_config: ServerConfig,
}

/// Server configuration sent to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub version: String,
    pub features: Vec<String>,
    pub endpoints: ApiEndpoints,
}

/// API endpoints configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoints {
    pub websocket: String,
    pub rest: String,
    pub cdn: String,
}

/// Web1 bootstrap handler
pub struct BootstrapHandler;

impl BootstrapHandler {
    /// Generate bootstrap configuration for Web1 client
    pub async fn generate_bootstrap(
        client_id: Uuid,
        entity_id: Uuid,
    ) -> crate::ClientResult<BootstrapConfig> {
        let config = BootstrapConfig {
            client_id,
            entity_id,
            server_config: ServerConfig {
                version: "2.0.0".to_string(),
                features: vec![
                    "websockets".to_string(),
                    "real_time_updates".to_string(),
                    "tutorial".to_string(),
                ],
                endpoints: ApiEndpoints {
                    websocket: "wss://api.hackerexperience.com/ws".to_string(),
                    rest: "https://api.hackerexperience.com".to_string(),
                    cdn: "https://cdn.hackerexperience.com".to_string(),
                },
            },
        };

        tracing::info!("Generated bootstrap config for client: {}", client_id);
        Ok(config)
    }
}