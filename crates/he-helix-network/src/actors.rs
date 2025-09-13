//! Network Actor System
//!
//! This module provides actor implementations for network management,
//! including connections, tunnels, and network topology.

use crate::{NetworkId, ConnectionId, TunnelId, Connection, Tunnel, Network, NetworkType, ConnectionType, ConnectionState};
use he_helix_core::actors::{Actor, ActorContext, Handler, Message};
use he_helix_core::HelixError;
use he_helix_server::ServerId;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::Utc;
use std::sync::Arc;
use tracing::{info, error, warn, debug};
use serde::{Serialize, Deserialize};

/// Messages for Network Actor
#[derive(Debug)]
pub struct CreateConnection {
    pub source_server: ServerId,
    pub target_server: ServerId,
    pub connection_type: ConnectionType,
}

impl Message for CreateConnection {
    type Result = Result<Connection, HelixError>;
}

#[derive(Debug)]
pub struct GetConnection {
    pub connection_id: ConnectionId,
}

impl Message for GetConnection {
    type Result = Result<Option<Connection>, HelixError>;
}

#[derive(Debug)]
pub struct CloseConnection {
    pub connection_id: ConnectionId,
}

impl Message for CloseConnection {
    type Result = Result<(), HelixError>;
}

#[derive(Debug)]
pub struct GetServerConnections {
    pub server_id: ServerId,
}

impl Message for GetServerConnections {
    type Result = Result<Vec<Connection>, HelixError>;
}

#[derive(Debug)]
pub struct CreateTunnel {
    pub source_server: ServerId,
    pub target_server: ServerId,
    pub tunnel_type: String,
    pub bounce_servers: Vec<ServerId>,
}

impl Message for CreateTunnel {
    type Result = Result<Tunnel, HelixError>;
}

#[derive(Debug)]
pub struct GetTunnel {
    pub tunnel_id: TunnelId,
}

impl Message for GetTunnel {
    type Result = Result<Option<Tunnel>, HelixError>;
}

#[derive(Debug)]
pub struct CloseTunnel {
    pub tunnel_id: TunnelId,
}

impl Message for CloseTunnel {
    type Result = Result<(), HelixError>;
}

#[derive(Debug)]
pub struct CreateNetwork {
    pub network_type: NetworkType,
    pub name: String,
    pub owner_server: ServerId,
}

impl Message for CreateNetwork {
    type Result = Result<Network, HelixError>;
}

#[derive(Debug)]
pub struct JoinNetwork {
    pub network_id: NetworkId,
    pub server_id: ServerId,
}

impl Message for JoinNetwork {
    type Result = Result<(), HelixError>;
}

#[derive(Debug)]
pub struct LeaveNetwork {
    pub network_id: NetworkId,
    pub server_id: ServerId,
}

impl Message for LeaveNetwork {
    type Result = Result<(), HelixError>;
}

#[derive(Debug)]
pub struct GetNetworkMembers {
    pub network_id: NetworkId,
}

impl Message for GetNetworkMembers {
    type Result = Result<Vec<ServerId>, HelixError>;
}

#[derive(Debug)]
pub struct UpdateConnectionState {
    pub connection_id: ConnectionId,
    pub new_state: ConnectionState,
}

impl Message for UpdateConnectionState {
    type Result = Result<(), HelixError>;
}

/// Network Actor - manages network topology and connections
#[derive(Debug)]
pub struct NetworkActor {
    /// Active connections
    connections: Arc<RwLock<HashMap<ConnectionId, Connection>>>,
    /// Active tunnels
    tunnels: Arc<RwLock<HashMap<TunnelId, Tunnel>>>,
    /// Networks
    networks: Arc<RwLock<HashMap<NetworkId, Network>>>,
    /// Server to connections mapping for efficient lookup
    server_connections: Arc<RwLock<HashMap<ServerId, Vec<ConnectionId>>>>,
    /// Network memberships
    network_memberships: Arc<RwLock<HashMap<NetworkId, Vec<ServerId>>>>,
}

impl NetworkActor {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            tunnels: Arc::new(RwLock::new(HashMap::new())),
            networks: Arc::new(RwLock::new(HashMap::new())),
            server_connections: Arc::new(RwLock::new(HashMap::new())),
            network_memberships: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a new unique connection ID
    fn generate_connection_id(&self) -> ConnectionId {
        ConnectionId::new()
    }

    /// Generate a new unique tunnel ID
    fn generate_tunnel_id(&self) -> TunnelId {
        TunnelId::new()
    }

    /// Generate a new unique network ID
    fn generate_network_id(&self) -> NetworkId {
        NetworkId::new()
    }

    /// Add connection to server mapping
    async fn add_connection_to_server(&self, server_id: ServerId, connection_id: ConnectionId) {
        let mut server_connections = self.server_connections.write().await;
        server_connections.entry(server_id).or_insert_with(Vec::new).push(connection_id);
    }

    /// Remove connection from server mapping
    async fn remove_connection_from_server(&self, server_id: &ServerId, connection_id: &ConnectionId) {
        let mut server_connections = self.server_connections.write().await;
        if let Some(connections) = server_connections.get_mut(server_id) {
            connections.retain(|id| id != connection_id);
        }
    }

    /// Validate connection parameters
    fn validate_connection(&self, source: &ServerId, target: &ServerId) -> Result<(), HelixError> {
        if source == target {
            return Err(HelixError::validation("Cannot connect server to itself"));
        }
        Ok(())
    }

    /// Calculate connection latency based on network topology
    fn calculate_latency(&self, _source: &ServerId, _target: &ServerId) -> u32 {
        // Simplified latency calculation - in reality would consider network topology
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen_range(10..=200) // 10-200ms latency
    }

    /// Start connection monitoring
    async fn start_connection_monitoring(&self) {
        let connections = self.connections.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // Monitor connection health
                let mut connections_to_check = Vec::new();
                {
                    let connections_guard = connections.read().await;
                    for connection in connections_guard.values() {
                        if matches!(connection.state, ConnectionState::Connected) {
                            connections_to_check.push(connection.connection_id);
                        }
                    }
                }

                // In a real implementation, this would ping connections and update their state
                for connection_id in connections_to_check {
                    debug!("Monitoring connection health: {}", connection_id);
                }
            }
        });
    }
}

impl Actor for NetworkActor {
    fn started(&mut self, ctx: &mut ActorContext) {
        info!("NetworkActor started with process_id: {}", ctx.process_id);
        
        // Start connection monitoring
        let actor = self.clone();
        tokio::spawn(async move {
            actor.start_connection_monitoring().await;
        });
    }

    fn stopping(&mut self, ctx: &mut ActorContext) {
        info!("NetworkActor stopping with process_id: {}", ctx.process_id);
    }

    fn error(&mut self, err: HelixError, ctx: &mut ActorContext) {
        error!("NetworkActor error on process_id {}: {}", ctx.process_id, err);
    }
}

impl Clone for NetworkActor {
    fn clone(&self) -> Self {
        Self {
            connections: self.connections.clone(),
            tunnels: self.tunnels.clone(),
            networks: self.networks.clone(),
            server_connections: self.server_connections.clone(),
            network_memberships: self.network_memberships.clone(),
        }
    }
}

#[async_trait]
impl Handler<CreateConnection> for NetworkActor {
    async fn handle(&mut self, msg: CreateConnection, _ctx: &mut ActorContext) -> Result<Connection, HelixError> {
        info!("Creating connection from {} to {}", msg.source_server, msg.target_server);
        
        self.validate_connection(&msg.source_server, &msg.target_server)?;

        let mut connections = self.connections.write().await;
        
        let connection_id = self.generate_connection_id();
        let now = Utc::now();
        let latency = self.calculate_latency(&msg.source_server, &msg.target_server);

        let connection = Connection {
            connection_id,
            source_server: msg.source_server,
            target_server: msg.target_server,
            connection_type: msg.connection_type,
            state: ConnectionState::Connecting,
            established_at: now,
            last_activity: now,
            latency,
            bandwidth_used: 0,
            total_bytes_sent: 0,
            total_bytes_received: 0,
        };

        // Store connection
        connections.insert(connection_id, connection.clone());

        // Update server mappings
        self.add_connection_to_server(msg.source_server, connection_id).await;
        self.add_connection_to_server(msg.target_server, connection_id).await;

        // Simulate connection establishment
        tokio::spawn({
            let connections = self.connections.clone();
            async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(latency as u64)).await;
                
                let mut connections_guard = connections.write().await;
                if let Some(conn) = connections_guard.get_mut(&connection_id) {
                    conn.state = ConnectionState::Connected;
                }
                
                info!("Connection established: {}", connection_id);
            }
        });

        info!("Connection created: {}", connection_id);
        Ok(connection)
    }
}

#[async_trait]
impl Handler<GetConnection> for NetworkActor {
    async fn handle(&mut self, msg: GetConnection, _ctx: &mut ActorContext) -> Result<Option<Connection>, HelixError> {
        let connections = self.connections.read().await;
        Ok(connections.get(&msg.connection_id).cloned())
    }
}

#[async_trait]
impl Handler<CloseConnection> for NetworkActor {
    async fn handle(&mut self, msg: CloseConnection, _ctx: &mut ActorContext) -> Result<(), HelixError> {
        let mut connections = self.connections.write().await;
        
        if let Some(connection) = connections.get_mut(&msg.connection_id) {
            connection.state = ConnectionState::Disconnected;
            
            // Remove from server mappings
            self.remove_connection_from_server(&connection.source_server, &msg.connection_id).await;
            self.remove_connection_from_server(&connection.target_server, &msg.connection_id).await;
            
            info!("Connection closed: {}", msg.connection_id);
            Ok(())
        } else {
            Err(HelixError::not_found("Connection not found"))
        }
    }
}

#[async_trait]
impl Handler<GetServerConnections> for NetworkActor {
    async fn handle(&mut self, msg: GetServerConnections, _ctx: &mut ActorContext) -> Result<Vec<Connection>, HelixError> {
        let connections = self.connections.read().await;
        let server_connections = self.server_connections.read().await;
        
        if let Some(connection_ids) = server_connections.get(&msg.server_id) {
            Ok(connection_ids.iter()
                .filter_map(|id| connections.get(id).cloned())
                .collect())
        } else {
            Ok(Vec::new())
        }
    }
}

#[async_trait]
impl Handler<CreateTunnel> for NetworkActor {
    async fn handle(&mut self, msg: CreateTunnel, _ctx: &mut ActorContext) -> Result<Tunnel, HelixError> {
        info!("Creating tunnel from {} to {} via {} bounces", 
            msg.source_server, msg.target_server, msg.bounce_servers.len());

        let mut tunnels = self.tunnels.write().await;
        
        let tunnel_id = self.generate_tunnel_id();
        let now = Utc::now();

        let tunnel = Tunnel {
            tunnel_id,
            source_server: msg.source_server,
            target_server: msg.target_server,
            bounce_servers: msg.bounce_servers.clone(),
            tunnel_type: msg.tunnel_type,
            established_at: now,
            last_activity: now,
            is_active: true,
            bandwidth_used: 0,
            encryption_level: if msg.bounce_servers.len() > 2 { 3 } else { 1 },
        };

        tunnels.insert(tunnel_id, tunnel.clone());

        info!("Tunnel created: {}", tunnel_id);
        Ok(tunnel)
    }
}

#[async_trait]
impl Handler<GetTunnel> for NetworkActor {
    async fn handle(&mut self, msg: GetTunnel, _ctx: &mut ActorContext) -> Result<Option<Tunnel>, HelixError> {
        let tunnels = self.tunnels.read().await;
        Ok(tunnels.get(&msg.tunnel_id).cloned())
    }
}

#[async_trait]
impl Handler<CloseTunnel> for NetworkActor {
    async fn handle(&mut self, msg: CloseTunnel, _ctx: &mut ActorContext) -> Result<(), HelixError> {
        let mut tunnels = self.tunnels.write().await;
        
        if let Some(tunnel) = tunnels.get_mut(&msg.tunnel_id) {
            tunnel.is_active = false;
            tunnel.last_activity = Utc::now();
            
            info!("Tunnel closed: {}", msg.tunnel_id);
            Ok(())
        } else {
            Err(HelixError::not_found("Tunnel not found"))
        }
    }
}

#[async_trait]
impl Handler<CreateNetwork> for NetworkActor {
    async fn handle(&mut self, msg: CreateNetwork, _ctx: &mut ActorContext) -> Result<Network, HelixError> {
        info!("Creating network '{}' owned by {}", msg.name, msg.owner_server);

        let mut networks = self.networks.write().await;
        let mut memberships = self.network_memberships.write().await;
        
        let network_id = self.generate_network_id();
        let now = Utc::now();

        let network = Network {
            network_id,
            network_type: msg.network_type,
            name: msg.name,
            owner_server: msg.owner_server,
            created_at: now,
            is_active: true,
            member_count: 1,
        };

        networks.insert(network_id, network.clone());
        memberships.insert(network_id, vec![msg.owner_server]);

        info!("Network created: {} ({})", network.name, network_id);
        Ok(network)
    }
}

#[async_trait]
impl Handler<JoinNetwork> for NetworkActor {
    async fn handle(&mut self, msg: JoinNetwork, _ctx: &mut ActorContext) -> Result<(), HelixError> {
        let mut networks = self.networks.write().await;
        let mut memberships = self.network_memberships.write().await;

        let network = networks.get_mut(&msg.network_id)
            .ok_or_else(|| HelixError::not_found("Network not found"))?;

        let members = memberships.entry(msg.network_id).or_insert_with(Vec::new);
        
        if !members.contains(&msg.server_id) {
            members.push(msg.server_id);
            network.member_count = members.len();
            
            info!("Server {} joined network {}", msg.server_id, msg.network_id);
        }

        Ok(())
    }
}

#[async_trait]
impl Handler<LeaveNetwork> for NetworkActor {
    async fn handle(&mut self, msg: LeaveNetwork, _ctx: &mut ActorContext) -> Result<(), HelixError> {
        let mut networks = self.networks.write().await;
        let mut memberships = self.network_memberships.write().await;

        let network = networks.get_mut(&msg.network_id)
            .ok_or_else(|| HelixError::not_found("Network not found"))?;

        if let Some(members) = memberships.get_mut(&msg.network_id) {
            members.retain(|&server| server != msg.server_id);
            network.member_count = members.len();
            
            info!("Server {} left network {}", msg.server_id, msg.network_id);
        }

        Ok(())
    }
}

#[async_trait]
impl Handler<GetNetworkMembers> for NetworkActor {
    async fn handle(&mut self, msg: GetNetworkMembers, _ctx: &mut ActorContext) -> Result<Vec<ServerId>, HelixError> {
        let memberships = self.network_memberships.read().await;
        Ok(memberships.get(&msg.network_id).cloned().unwrap_or_default())
    }
}

#[async_trait]
impl Handler<UpdateConnectionState> for NetworkActor {
    async fn handle(&mut self, msg: UpdateConnectionState, _ctx: &mut ActorContext) -> Result<(), HelixError> {
        let mut connections = self.connections.write().await;
        
        if let Some(connection) = connections.get_mut(&msg.connection_id) {
            connection.state = msg.new_state;
            connection.last_activity = Utc::now();
            
            debug!("Connection state updated: {} -> {:?}", msg.connection_id, msg.new_state);
            Ok(())
        } else {
            Err(HelixError::not_found("Connection not found"))
        }
    }
}

/// Network Supervisor - manages network actor and provides supervision
#[derive(Debug)]
pub struct NetworkSupervisor {
    network_actor: Option<he_helix_core::actors::ActorAddress>,
}

impl NetworkSupervisor {
    pub fn new() -> Self {
        Self {
            network_actor: None,
        }
    }
    
    pub async fn start(&mut self) -> Result<he_helix_core::actors::ActorAddress, HelixError> {
        let mut supervisor = he_helix_core::actors::ActorSupervisor::new();
        let network_actor = NetworkActor::new();
        let address = supervisor.spawn(network_actor);
        
        self.network_actor = Some(address.clone());
        info!("NetworkSupervisor started successfully");
        
        Ok(address)
    }
    
    pub fn get_network_actor(&self) -> Option<&he_helix_core::actors::ActorAddress> {
        self.network_actor.as_ref()
    }
}

impl Default for NetworkSupervisor {
    fn default() -> Self {
        Self::new()
    }
}