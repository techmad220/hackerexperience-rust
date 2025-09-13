//! Network model definitions

use crate::types::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use he_helix_server::ServerId;

/// Network entity model
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Network {
    pub network_id: NetworkId,
    pub name: String,
    pub network_type: NetworkType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Network {
    /// Create a new network instance
    pub fn new(
        network_id: NetworkId,
        name: String,
        network_type: NetworkType,
    ) -> Self {
        let now = Utc::now();
        Self {
            network_id,
            name,
            network_type,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the network name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    /// Check if this network is public (internet)
    pub fn is_public(&self) -> bool {
        matches!(self.network_type, NetworkType::Internet)
    }

    /// Check if this network is private (LAN, story, mission)
    pub fn is_private(&self) -> bool {
        !self.is_public()
    }
}

/// Tunnel model representing a connection path between servers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tunnel {
    pub tunnel_id: TunnelId,
    pub network_id: NetworkId,
    pub gateway_id: ServerId,
    pub target_id: ServerId,
    pub bounce_id: Option<BounceId>,
    pub hops: Vec<BounceLink>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Tunnel {
    /// Create a new tunnel instance
    pub fn new(
        tunnel_id: TunnelId,
        network_id: NetworkId,
        gateway_id: ServerId,
        target_id: ServerId,
    ) -> Self {
        let now = Utc::now();
        Self {
            tunnel_id,
            network_id,
            gateway_id,
            target_id,
            bounce_id: None,
            hops: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a tunnel with bounce routing
    pub fn new_with_bounce(
        tunnel_id: TunnelId,
        network_id: NetworkId,
        gateway_id: ServerId,
        target_id: ServerId,
        bounce_id: BounceId,
        hops: Vec<BounceLink>,
    ) -> Self {
        let now = Utc::now();
        Self {
            tunnel_id,
            network_id,
            gateway_id,
            target_id,
            bounce_id: Some(bounce_id),
            hops,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if this tunnel uses bounce routing
    pub fn has_bounce(&self) -> bool {
        self.bounce_id.is_some()
    }

    /// Get the number of hops in the bounce chain
    pub fn hop_count(&self) -> usize {
        self.hops.len()
    }

    /// Check if this tunnel creates a cycle (gateway == target)
    pub fn is_cyclic(&self) -> bool {
        self.gateway_id == self.target_id
    }

    /// Get the final target after all bounces
    pub fn final_target(&self) -> ServerId {
        self.hops
            .last()
            .map(|hop| hop.server_id)
            .unwrap_or(self.target_id)
    }

    /// Add a hop to the bounce chain
    pub fn add_hop(&mut self, hop: BounceLink) {
        self.hops.push(hop);
        self.updated_at = Utc::now();
    }

    /// Remove all hops and clear bounce
    pub fn clear_bounce(&mut self) {
        self.bounce_id = None;
        self.hops.clear();
        self.updated_at = Utc::now();
    }
}

/// Connection model representing a specific connection type within a tunnel
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connection {
    pub connection_id: ConnectionId,
    pub tunnel_id: TunnelId,
    pub connection_type: ConnectionType,
    pub meta: Option<ConnectionMeta>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

impl Connection {
    /// Create a new connection instance
    pub fn new(
        connection_id: ConnectionId,
        tunnel_id: TunnelId,
        connection_type: ConnectionType,
    ) -> Self {
        let now = Utc::now();
        Self {
            connection_id,
            tunnel_id,
            connection_type,
            meta: None,
            created_at: now,
            updated_at: now,
            closed_at: None,
        }
    }

    /// Create a connection with metadata
    pub fn new_with_meta(
        connection_id: ConnectionId,
        tunnel_id: TunnelId,
        connection_type: ConnectionType,
        meta: ConnectionMeta,
    ) -> Self {
        let now = Utc::now();
        Self {
            connection_id,
            tunnel_id,
            connection_type,
            meta: Some(meta),
            created_at: now,
            updated_at: now,
            closed_at: None,
        }
    }

    /// Check if the connection is active (not closed)
    pub fn is_active(&self) -> bool {
        self.closed_at.is_none()
    }

    /// Close the connection
    pub fn close(&mut self, reason: CloseReason) {
        self.closed_at = Some(Utc::now());
        self.updated_at = Utc::now();
        
        // Store close reason in metadata
        if let Some(ref mut meta) = self.meta {
            let _ = meta.insert("close_reason".to_string(), reason.as_str());
        } else {
            let mut meta = ConnectionMeta::new();
            let _ = meta.insert("close_reason".to_string(), reason.as_str());
            self.meta = Some(meta);
        }
    }

    /// Update connection metadata
    pub fn update_meta(&mut self, meta: ConnectionMeta) {
        self.meta = Some(meta);
        self.updated_at = Utc::now();
    }

    /// Get a value from the connection metadata
    pub fn get_meta_value<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>, serde_json::Error> {
        match &self.meta {
            Some(meta) => meta.get(key),
            None => Ok(None),
        }
    }

    /// Set a value in the connection metadata
    pub fn set_meta_value<V: serde::Serialize>(&mut self, key: String, value: V) -> Result<(), serde_json::Error> {
        match &mut self.meta {
            Some(meta) => {
                meta.insert(key, value)?;
            }
            None => {
                let mut meta = ConnectionMeta::new();
                meta.insert(key, value)?;
                self.meta = Some(meta);
            }
        }
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Get the connection type as a string
    pub fn type_name(&self) -> &'static str {
        self.connection_type.as_str()
    }
}

/// Bounce model for routing connections through proxy servers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bounce {
    pub bounce_id: BounceId,
    pub name: String,
    pub links: Vec<BounceLink>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Bounce {
    /// Create a new bounce instance
    pub fn new(bounce_id: BounceId, name: String) -> Self {
        let now = Utc::now();
        Self {
            bounce_id,
            name,
            links: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a link to the bounce chain
    pub fn add_link(&mut self, link: BounceLink) {
        self.links.push(link);
        self.links.sort_by_key(|l| l.sequence);
        self.updated_at = Utc::now();
    }

    /// Remove a link from the bounce chain
    pub fn remove_link(&mut self, link_id: LinkId) -> bool {
        let initial_len = self.links.len();
        self.links.retain(|l| l.link_id != link_id);
        
        if self.links.len() < initial_len {
            // Re-sequence the remaining links
            for (index, link) in self.links.iter_mut().enumerate() {
                link.sequence = index as u32;
            }
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Get the total number of links in this bounce
    pub fn link_count(&self) -> usize {
        self.links.len()
    }

    /// Check if this bounce is valid (has at least one link)
    pub fn is_valid(&self) -> bool {
        !self.links.is_empty()
    }

    /// Get all the server IDs in the bounce chain
    pub fn server_ids(&self) -> Vec<ServerId> {
        self.links.iter().map(|l| l.server_id).collect()
    }

    /// Clear all links
    pub fn clear_links(&mut self) {
        self.links.clear();
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use std::net::Ipv4Addr;

    #[test]
    fn test_network_creation() {
        let network_id = Uuid::new_v4();
        let network = Network::new(
            network_id,
            "Test Network".to_string(),
            NetworkType::Internet,
        );

        assert_eq!(network.network_id, network_id);
        assert_eq!(network.name, "Test Network");
        assert_eq!(network.network_type, NetworkType::Internet);
        assert!(network.is_public());
        assert!(!network.is_private());
    }

    #[test]
    fn test_tunnel_creation() {
        let tunnel_id = Uuid::new_v4();
        let network_id = Uuid::new_v4();
        let gateway_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();

        let tunnel = Tunnel::new(tunnel_id, network_id, gateway_id, target_id);

        assert_eq!(tunnel.tunnel_id, tunnel_id);
        assert_eq!(tunnel.network_id, network_id);
        assert_eq!(tunnel.gateway_id, gateway_id);
        assert_eq!(tunnel.target_id, target_id);
        assert!(!tunnel.has_bounce());
        assert_eq!(tunnel.hop_count(), 0);
        assert!(!tunnel.is_cyclic());
    }

    #[test]
    fn test_connection_management() {
        let connection_id = Uuid::new_v4();
        let tunnel_id = Uuid::new_v4();
        let mut connection = Connection::new(
            connection_id,
            tunnel_id,
            ConnectionType::Ssh,
        );

        assert!(connection.is_active());
        assert_eq!(connection.type_name(), "ssh");

        connection.close(CloseReason::Normal);
        assert!(!connection.is_active());

        let close_reason: Option<String> = connection.get_meta_value("close_reason").unwrap();
        assert_eq!(close_reason, Some("normal".to_string()));
    }

    #[test]
    fn test_bounce_management() {
        let bounce_id = Uuid::new_v4();
        let mut bounce = Bounce::new(bounce_id, "Test Bounce".to_string());

        assert!(!bounce.is_valid());
        assert_eq!(bounce.link_count(), 0);

        let link = BounceLink {
            link_id: Uuid::new_v4(),
            server_id: Uuid::new_v4(),
            network_id: Uuid::new_v4(),
            ip: Ipv4Addr::new(192, 168, 1, 1),
            sequence: 0,
        };

        bounce.add_link(link);
        assert!(bounce.is_valid());
        assert_eq!(bounce.link_count(), 1);
    }

    #[test]
    fn test_connection_metadata() {
        let connection_id = Uuid::new_v4();
        let tunnel_id = Uuid::new_v4();
        let mut connection = Connection::new(
            connection_id,
            tunnel_id,
            ConnectionType::Ftp,
        );

        let result = connection.set_meta_value("test_key".to_string(), "test_value");
        assert!(result.is_ok());

        let value: Option<String> = connection.get_meta_value("test_key").unwrap();
        assert_eq!(value, Some("test_value".to_string()));
    }
}