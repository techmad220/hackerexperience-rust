//! Network-related rendering for client consumption

use crate::{
    renderer::{Bounce, ClientNip, RenderedBounce},
    ClientResult,
};

/// Network renderer for converting network data to client-friendly format
pub struct NetworkRenderer;

impl NetworkRenderer {
    /// Render a bounce structure for client consumption
    pub fn render_bounce(bounce: &Bounce) -> ClientResult<RenderedBounce> {
        // Convert bounce links to client NIPs
        let links: Vec<ClientNip> = bounce
            .links
            .iter()
            .map(|(_, network_id, ip)| ClientNip::new(*network_id, ip.clone()))
            .collect();

        Ok(RenderedBounce {
            bounce_id: bounce.bounce_id.to_string(),
            name: bounce.name.clone(),
            links,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_render_bounce() {
        let bounce_id = Uuid::new_v4();
        let network_id = Uuid::new_v4();
        
        let bounce = Bounce {
            bounce_id,
            name: "TestBounce".to_string(),
            links: vec![
                ("server1".to_string(), network_id, "192.168.1.1".to_string()),
                ("server2".to_string(), network_id, "10.0.0.1".to_string()),
            ],
        };

        let rendered = NetworkRenderer::render_bounce(&bounce).unwrap();

        assert_eq!(rendered.bounce_id, bounce_id.to_string());
        assert_eq!(rendered.name, "TestBounce");
        assert_eq!(rendered.links.len(), 2);
        assert_eq!(rendered.links[0].network_id, network_id.to_string());
        assert_eq!(rendered.links[0].ip, "192.168.1.1");
        assert_eq!(rendered.links[1].ip, "10.0.0.1");
    }
}