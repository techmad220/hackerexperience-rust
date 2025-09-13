//! Client rendering system for various data types

pub mod network;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{ClientError, ClientResult};

pub use network::NetworkRenderer;

/// Main renderer struct that delegates to specific renderers
#[derive(Debug, Default)]
pub struct Renderer;

impl Renderer {
    /// Create a new renderer instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Render a bounce for client consumption
    pub fn render_bounce(&self, bounce: &Bounce) -> ClientResult<RenderedBounce> {
        NetworkRenderer::render_bounce(bounce)
    }
}

/// Bounce data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bounce {
    pub bounce_id: Uuid,
    pub name: String,
    pub links: Vec<(String, Uuid, String)>, // (server_id, network_id, ip)
}

/// Rendered bounce for client consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedBounce {
    pub bounce_id: String,
    pub name: String,
    pub links: Vec<ClientNip>,
}

/// Client Network IP representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientNip {
    pub network_id: String,
    pub ip: String,
}

impl ClientNip {
    /// Create a new ClientNip from network_id and IP
    pub fn new(network_id: Uuid, ip: impl Into<String>) -> Self {
        Self {
            network_id: network_id.to_string(),
            ip: ip.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_nip_creation() {
        let network_id = Uuid::new_v4();
        let ip = "192.168.1.1";
        
        let nip = ClientNip::new(network_id, ip);
        
        assert_eq!(nip.network_id, network_id.to_string());
        assert_eq!(nip.ip, ip);
    }

    #[test]
    fn test_renderer_creation() {
        let renderer = Renderer::new();
        // Renderer should be successfully created
        let _ = renderer;
    }
}