//! # Core Network Topology System
//!
//! This crate provides the network management functionality for the HackerExperience
//! game engine, including network topology, connections, tunnels, and bouncing systems.
//!
//! ## Architecture
//!
//! The network system is built around several key concepts:
//! - **Network**: Network entities (internet, story, mission, LAN)
//! - **Tunnel**: Direct connection paths between servers
//! - **Connection**: Specific connection types (SSH, FTP, etc.) within tunnels
//! - **Bounce**: Proxy routing to hide connection origins
//! - **Links**: Individual hops in a bounce chain
//!
//! ## Key Features
//!
//! - Async/await support using Tokio
//! - Actor-based architecture using Actix
//! - Database persistence with SeaORM
//! - Network topology management
//! - Connection routing and bouncing
//! - IP address management

pub mod actors;
pub mod bounce;
pub mod connection;
pub mod error;
pub mod model;
pub mod network;
pub mod query;
pub mod tunnel;
pub mod types;

pub use model::{Connection, Network, Tunnel};
pub use types::*;

use anyhow::Result;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Global network registry for managing network instances and topology
pub static NETWORK_REGISTRY: Lazy<Arc<RwLock<NetworkRegistry>>> = 
    Lazy::new(|| Arc::new(RwLock::new(NetworkRegistry::new())));

/// Network registry for tracking active networks and connections
#[derive(Debug, Default)]
pub struct NetworkRegistry {
    networks: dashmap::DashMap<NetworkId, Arc<Network>>,
    tunnels: dashmap::DashMap<TunnelId, Arc<Tunnel>>,
    connections: dashmap::DashMap<ConnectionId, Arc<Connection>>,
}

impl NetworkRegistry {
    pub fn new() -> Self {
        Self {
            networks: dashmap::DashMap::new(),
            tunnels: dashmap::DashMap::new(),
            connections: dashmap::DashMap::new(),
        }
    }

    pub async fn register_network(&self, network: Network) -> Arc<Network> {
        let network_arc = Arc::new(network);
        self.networks.insert(network_arc.network_id.clone(), network_arc.clone());
        network_arc
    }

    pub async fn register_tunnel(&self, tunnel: Tunnel) -> Arc<Tunnel> {
        let tunnel_arc = Arc::new(tunnel);
        self.tunnels.insert(tunnel_arc.tunnel_id.clone(), tunnel_arc.clone());
        tunnel_arc
    }

    pub async fn register_connection(&self, connection: Connection) -> Arc<Connection> {
        let connection_arc = Arc::new(connection);
        self.connections.insert(connection_arc.connection_id.clone(), connection_arc.clone());
        connection_arc
    }

    pub async fn get_network(&self, network_id: &NetworkId) -> Option<Arc<Network>> {
        self.networks.get(network_id).map(|entry| entry.clone())
    }

    pub async fn get_tunnel(&self, tunnel_id: &TunnelId) -> Option<Arc<Tunnel>> {
        self.tunnels.get(tunnel_id).map(|entry| entry.clone())
    }

    pub async fn get_connection(&self, connection_id: &ConnectionId) -> Option<Arc<Connection>> {
        self.connections.get(connection_id).map(|entry| entry.clone())
    }

    pub async fn remove_network(&self, network_id: &NetworkId) -> Option<Arc<Network>> {
        self.networks.remove(network_id).map(|(_, network)| network)
    }

    pub async fn remove_tunnel(&self, tunnel_id: &TunnelId) -> Option<Arc<Tunnel>> {
        self.tunnels.remove(tunnel_id).map(|(_, tunnel)| tunnel)
    }

    pub async fn remove_connection(&self, connection_id: &ConnectionId) -> Option<Arc<Connection>> {
        self.connections.remove(connection_id).map(|(_, connection)| connection)
    }

    pub async fn list_networks(&self) -> Vec<Arc<Network>> {
        self.networks.iter().map(|entry| entry.value().clone()).collect()
    }

    pub async fn list_tunnels(&self) -> Vec<Arc<Tunnel>> {
        self.tunnels.iter().map(|entry| entry.value().clone()).collect()
    }

    pub async fn list_connections(&self) -> Vec<Arc<Connection>> {
        self.connections.iter().map(|entry| entry.value().clone()).collect()
    }

    /// Get all tunnels for a specific network
    pub async fn get_network_tunnels(&self, network_id: &NetworkId) -> Vec<Arc<Tunnel>> {
        self.tunnels
            .iter()
            .filter(|entry| &entry.network_id == network_id)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get all connections for a specific tunnel
    pub async fn get_tunnel_connections(&self, tunnel_id: &TunnelId) -> Vec<Arc<Connection>> {
        self.connections
            .iter()
            .filter(|entry| &entry.tunnel_id == tunnel_id)
            .map(|entry| entry.value().clone())
            .collect()
    }
}

/// Initialize the network subsystem
pub async fn init() -> Result<()> {
    tracing::info!("Initializing Core Network subsystem");
    
    // Initialize the network registry
    let _registry = NETWORK_REGISTRY.clone();
    
    tracing::info!("Core Network subsystem initialized successfully");
    Ok(())
}

/// Shutdown the network subsystem gracefully
pub async fn shutdown() -> Result<()> {
    tracing::info!("Shutting down Core Network subsystem");
    
    // Clear all registries
    let registry = NETWORK_REGISTRY.read().await;
    registry.networks.clear();
    registry.tunnels.clear();
    registry.connections.clear();
    
    tracing::info!("Core Network subsystem shutdown complete");
    Ok(())
}