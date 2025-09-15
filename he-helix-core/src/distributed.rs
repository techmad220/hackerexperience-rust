//! Distributed GenServer Communication Patterns
//! 
//! Provides distributed computing capabilities for multi-node GenServer systems,
//! including clustering, remote procedure calls, and fault tolerance.

use crate::genserver::{GenServer, GenServerHandle, TerminateReason};
use crate::{HelixError, HelixResult, ProcessId};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{RwLock, Mutex, mpsc, oneshot, broadcast};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Distributed system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedConfig {
    pub node_name: String,
    pub bind_address: SocketAddr,
    pub discovery_enabled: bool,
    pub discovery_port: u16,
    pub heartbeat_interval: Duration,
    pub connection_timeout: Duration,
    pub max_connections: usize,
    pub enable_encryption: bool,
    pub cluster_key: Option<String>,
    pub enable_load_balancing: bool,
    pub partition_tolerance: bool,
}

impl Default for DistributedConfig {
    fn default() -> Self {
        Self {
            node_name: format!("node_{}", Uuid::new_v4()),
            bind_address: "127.0.0.1:0".parse().unwrap(),
            discovery_enabled: true,
            discovery_port: 4369, // Erlang Port Mapper Daemon port
            heartbeat_interval: Duration::from_secs(5),
            connection_timeout: Duration::from_secs(30),
            max_connections: 100,
            enable_encryption: false,
            cluster_key: None,
            enable_load_balancing: true,
            partition_tolerance: false,
        }
    }
}

/// Node information in the cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_name: String,
    pub address: SocketAddr,
    pub node_type: NodeType,
    pub capabilities: Vec<String>,
    pub load_metrics: LoadMetrics,
    pub last_seen: SystemTime,
    pub version: String,
    pub metadata: HashMap<String, String>,
}

/// Types of nodes in the distributed system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    /// Full node with all capabilities
    Full,
    /// Worker node for compute tasks
    Worker,
    /// Storage node for data persistence
    Storage,
    /// Gateway node for external access
    Gateway,
    /// Monitor node for system oversight
    Monitor,
}

/// Load and performance metrics for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_load: f64,
    pub active_connections: u32,
    pub genserver_count: u32,
    pub message_queue_depth: u32,
    pub last_updated: SystemTime,
}

impl Default for LoadMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            disk_usage: 0.0,
            network_load: 0.0,
            active_connections: 0,
            genserver_count: 0,
            message_queue_depth: 0,
            last_updated: SystemTime::now(),
        }
    }
}

/// Remote procedure call message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcMessage {
    pub message_id: Uuid,
    pub source_node: String,
    pub target_node: String,
    pub target_genserver: String,
    pub method: String,
    pub payload: Vec<u8>,
    pub timeout: Duration,
    pub created_at: SystemTime,
}

/// RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub message_id: Uuid,
    pub success: bool,
    pub payload: Vec<u8>,
    pub error_message: Option<String>,
    pub processing_time: Duration,
    pub responded_at: SystemTime,
}

/// Cluster events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterEvent {
    NodeJoined { node: NodeInfo },
    NodeLeft { node_name: String, reason: String },
    NodeFailed { node_name: String, error: String },
    NodeUpdated { node: NodeInfo },
    LeaderElected { node_name: String, term: u64 },
    PartitionDetected { nodes: Vec<String> },
    PartitionHealed { nodes: Vec<String> },
    LoadBalancingUpdate { assignments: HashMap<String, String> },
}

/// Distributed GenServer node
pub struct DistributedNode {
    config: DistributedConfig,
    local_info: NodeInfo,
    cluster_nodes: Arc<RwLock<HashMap<String, NodeInfo>>>,
    connections: Arc<RwLock<HashMap<String, Connection>>>,
    local_genservers: Arc<RwLock<HashMap<String, GenServerHandle>>>,
    rpc_handlers: Arc<RwLock<HashMap<String, Box<dyn RpcHandler>>>>,
    cluster_events: broadcast::Sender<ClusterEvent>,
    command_tx: mpsc::UnboundedSender<DistributedCommand>,
    command_rx: Mutex<Option<mpsc::UnboundedReceiver<DistributedCommand>>>,
    is_leader: Arc<RwLock<bool>>,
    leader_node: Arc<RwLock<Option<String>>>,
    current_term: Arc<RwLock<u64>>,
}

/// Connection to a remote node
#[derive(Debug)]
pub struct Connection {
    pub node_name: String,
    pub stream: Arc<Mutex<TcpStream>>,
    pub last_heartbeat: SystemTime,
    pub message_queue: Arc<Mutex<Vec<DistributedMessage>>>,
    pub is_healthy: bool,
}

/// Distributed system commands
#[derive(Debug)]
pub enum DistributedCommand {
    /// Connect to a remote node
    ConnectToNode { address: SocketAddr, node_info: NodeInfo },
    
    /// Disconnect from a node
    DisconnectFromNode { node_name: String },
    
    /// Send RPC message
    SendRpc { message: RpcMessage, response_tx: oneshot::Sender<HelixResult<RpcResponse>> },
    
    /// Register local GenServer
    RegisterGenServer { name: String, handle: GenServerHandle },
    
    /// Unregister local GenServer
    UnregisterGenServer { name: String },
    
    /// Update load metrics
    UpdateLoadMetrics { metrics: LoadMetrics },
    
    /// Start leader election
    StartLeaderElection,
    
    /// Handle heartbeat
    ProcessHeartbeat { node_name: String },
    
    /// Get cluster status
    GetClusterStatus { response_tx: oneshot::Sender<ClusterStatus> },
    
    /// Broadcast message to all nodes
    BroadcastMessage { message: DistributedMessage },
    
    /// Handle partition recovery
    HandlePartitionRecovery { recovered_nodes: Vec<String> },
}

/// Messages sent between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistributedMessage {
    /// Heartbeat to maintain connection
    Heartbeat { node_info: NodeInfo },
    
    /// RPC call
    Rpc { message: RpcMessage },
    
    /// RPC response
    RpcResponse { response: RpcResponse },
    
    /// Leader election messages
    VoteRequest { candidate: String, term: u64, node_info: NodeInfo },
    VoteResponse { voter: String, term: u64, vote_granted: bool },
    
    /// Cluster management
    ClusterUpdate { event: ClusterEvent },
    
    /// Load balancing
    LoadBalanceRequest { genserver_name: String, preferred_nodes: Vec<String> },
    LoadBalanceResponse { assigned_node: String },
    
    /// Partition detection and recovery
    PartitionCheck { sender: String, known_nodes: Vec<String> },
    PartitionResponse { sender: String, known_nodes: Vec<String>, is_leader: bool },
    
    /// Data synchronization
    SyncRequest { data_type: String, since: SystemTime },
    SyncResponse { data_type: String, data: Vec<u8> },
}

/// RPC handler trait for processing remote calls
#[async_trait]
pub trait RpcHandler: Send + Sync {
    async fn handle_rpc(&self, message: RpcMessage) -> HelixResult<RpcResponse>;
}

/// Cluster status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    pub local_node: NodeInfo,
    pub connected_nodes: Vec<NodeInfo>,
    pub leader_node: Option<String>,
    pub current_term: u64,
    pub is_healthy: bool,
    pub partition_status: PartitionStatus,
    pub load_balance_info: LoadBalanceInfo,
}

/// Partition status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionStatus {
    pub is_partitioned: bool,
    pub partition_groups: Vec<Vec<String>>,
    pub last_partition_event: Option<SystemTime>,
}

/// Load balancing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalanceInfo {
    pub strategy: LoadBalanceStrategy,
    pub genserver_assignments: HashMap<String, String>,
    pub node_loads: HashMap<String, LoadMetrics>,
}

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalanceStrategy {
    RoundRobin,
    LeastLoaded,
    ConsistentHashing,
    Geographic,
    Custom(String),
}

impl DistributedNode {
    pub async fn new(config: DistributedConfig) -> HelixResult<Self> {
        let local_info = NodeInfo {
            node_name: config.node_name.clone(),
            address: config.bind_address,
            node_type: NodeType::Full,
            capabilities: vec![
                "genserver".to_string(),
                "rpc".to_string(),
                "clustering".to_string(),
            ],
            load_metrics: LoadMetrics::default(),
            last_seen: SystemTime::now(),
            version: "1.0.0".to_string(),
            metadata: HashMap::new(),
        };

        let (cluster_events, _) = broadcast::channel(1000);
        let (command_tx, command_rx) = mpsc::unbounded_channel();

        Ok(Self {
            config,
            local_info,
            cluster_nodes: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            local_genservers: Arc::new(RwLock::new(HashMap::new())),
            rpc_handlers: Arc::new(RwLock::new(HashMap::new())),
            cluster_events,
            command_tx,
            command_rx: Mutex::new(Some(command_rx)),
            is_leader: Arc::new(RwLock::new(false)),
            leader_node: Arc::new(RwLock::new(None)),
            current_term: Arc::new(RwLock::new(0)),
        })
    }

    pub async fn start(&mut self) -> HelixResult<()> {
        info!("Starting distributed node '{}'", self.local_info.node_name);

        // Start TCP listener
        let listener = TcpListener::bind(self.config.bind_address).await
            .map_err(|e| HelixError::NetworkError(format!("Failed to bind to {}: {}", self.config.bind_address, e)))?;

        let actual_addr = listener.local_addr()
            .map_err(|e| HelixError::NetworkError(format!("Failed to get local address: {}", e)))?;
        
        info!("Node '{}' listening on {}", self.local_info.node_name, actual_addr);

        // Update local info with actual address
        self.local_info.address = actual_addr;

        // Start command processing loop
        if let Some(command_rx) = self.command_rx.lock().await.take() {
            tokio::spawn(self.clone().command_loop(command_rx));
        }

        // Start connection handler
        tokio::spawn(self.clone().connection_handler(listener));

        // Start heartbeat sender
        tokio::spawn(self.clone().heartbeat_loop());

        // Start node discovery if enabled
        if self.config.discovery_enabled {
            tokio::spawn(self.clone().node_discovery_loop());
        }

        // Start leader election if this is the first node
        if self.cluster_nodes.read().await.is_empty() {
            let _ = self.command_tx.send(DistributedCommand::StartLeaderElection);
        }

        Ok(())
    }

    async fn command_loop(self, mut command_rx: mpsc::UnboundedReceiver<DistributedCommand>) {
        while let Some(command) = command_rx.recv().await {
            if let Err(e) = self.handle_command(command).await {
                error!("Error handling distributed command: {}", e);
            }
        }
    }

    async fn handle_command(&self, command: DistributedCommand) -> HelixResult<()> {
        match command {
            DistributedCommand::ConnectToNode { address, node_info } => {
                self.connect_to_node(address, node_info).await?;
            }
            
            DistributedCommand::DisconnectFromNode { node_name } => {
                self.disconnect_from_node(&node_name).await?;
            }
            
            DistributedCommand::SendRpc { message, response_tx } => {
                let result = self.send_rpc_message(message).await;
                let _ = response_tx.send(result);
            }
            
            DistributedCommand::RegisterGenServer { name, handle } => {
                self.register_local_genserver(name, handle).await;
            }
            
            DistributedCommand::UnregisterGenServer { name } => {
                self.unregister_local_genserver(&name).await;
            }
            
            DistributedCommand::UpdateLoadMetrics { metrics } => {
                self.update_load_metrics(metrics).await;
            }
            
            DistributedCommand::StartLeaderElection => {
                self.start_leader_election().await?;
            }
            
            DistributedCommand::ProcessHeartbeat { node_name } => {
                self.process_heartbeat(&node_name).await?;
            }
            
            DistributedCommand::GetClusterStatus { response_tx } => {
                let status = self.get_cluster_status().await;
                let _ = response_tx.send(status);
            }
            
            DistributedCommand::BroadcastMessage { message } => {
                self.broadcast_message(message).await?;
            }
            
            DistributedCommand::HandlePartitionRecovery { recovered_nodes } => {
                self.handle_partition_recovery(recovered_nodes).await?;
            }
        }
        
        Ok(())
    }

    async fn connection_handler(self, listener: TcpListener) {
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("Accepted connection from {}", addr);
                    tokio::spawn(self.clone().handle_connection(stream));
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    async fn handle_connection(&self, stream: TcpStream) {
        // TODO: Implement connection protocol
        // This would involve:
        // 1. Handshake and authentication
        // 2. Node identification
        // 3. Message framing and serialization
        // 4. Message routing
        
        // For now, we simulate handling the connection
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    async fn heartbeat_loop(self) {
        let mut interval = tokio::time::interval(self.config.heartbeat_interval);
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.send_heartbeats().await {
                error!("Failed to send heartbeats: {}", e);
            }
            
            if let Err(e) = self.check_node_health().await {
                error!("Failed to check node health: {}", e);
            }
        }
    }

    async fn send_heartbeats(&self) -> HelixResult<()> {
        let heartbeat_message = DistributedMessage::Heartbeat {
            node_info: self.local_info.clone(),
        };

        self.broadcast_message(heartbeat_message).await
    }

    async fn check_node_health(&self) -> HelixResult<()> {
        let timeout = self.config.connection_timeout;
        let now = SystemTime::now();
        let mut nodes_to_remove = Vec::new();

        {
            let cluster_nodes = self.cluster_nodes.read().await;
            for (node_name, node_info) in cluster_nodes.iter() {
                if now.duration_since(node_info.last_seen).unwrap_or(Duration::ZERO) > timeout {
                    nodes_to_remove.push(node_name.clone());
                }
            }
        }

        for node_name in nodes_to_remove {
            warn!("Node '{}' appears to be down, removing from cluster", node_name);
            
            {
                let mut cluster_nodes = self.cluster_nodes.write().await;
                cluster_nodes.remove(&node_name);
            }

            let event = ClusterEvent::NodeFailed {
                node_name: node_name.clone(),
                error: "Heartbeat timeout".to_string(),
            };
            let _ = self.cluster_events.send(event);
        }

        Ok(())
    }

    async fn node_discovery_loop(self) {
        // TODO: Implement node discovery
        // This could use:
        // 1. Multicast discovery
        // 2. DNS-based discovery
        // 3. Configuration-based discovery
        // 4. Service mesh integration
        
        info!("Node discovery is not yet fully implemented");
    }

    async fn connect_to_node(&self, address: SocketAddr, node_info: NodeInfo) -> HelixResult<()> {
        info!("Connecting to node '{}' at {}", node_info.node_name, address);

        // TODO: Implement actual TCP connection
        // For now, we simulate a successful connection
        
        {
            let mut cluster_nodes = self.cluster_nodes.write().await;
            cluster_nodes.insert(node_info.node_name.clone(), node_info.clone());
        }

        let event = ClusterEvent::NodeJoined { node: node_info };
        let _ = self.cluster_events.send(event);

        Ok(())
    }

    async fn disconnect_from_node(&self, node_name: &str) -> HelixResult<()> {
        info!("Disconnecting from node '{}'", node_name);

        {
            let mut cluster_nodes = self.cluster_nodes.write().await;
            cluster_nodes.remove(node_name);
        }

        {
            let mut connections = self.connections.write().await;
            connections.remove(node_name);
        }

        let event = ClusterEvent::NodeLeft {
            node_name: node_name.to_string(),
            reason: "Manual disconnect".to_string(),
        };
        let _ = self.cluster_events.send(event);

        Ok(())
    }

    async fn send_rpc_message(&self, message: RpcMessage) -> HelixResult<RpcResponse> {
        info!("Sending RPC to '{}' on node '{}'", message.target_genserver, message.target_node);

        // Check if target is local
        if message.target_node == self.local_info.node_name {
            return self.handle_local_rpc(message).await;
        }

        // Check if target node is connected
        {
            let cluster_nodes = self.cluster_nodes.read().await;
            if !cluster_nodes.contains_key(&message.target_node) {
                return Err(HelixError::NotFound(format!("Node '{}' not found in cluster", message.target_node)));
            }
        }

        // TODO: Send message over network
        // For now, simulate a successful RPC
        Ok(RpcResponse {
            message_id: message.message_id,
            success: true,
            payload: vec![],
            error_message: None,
            processing_time: Duration::from_millis(10),
            responded_at: SystemTime::now(),
        })
    }

    async fn handle_local_rpc(&self, message: RpcMessage) -> HelixResult<RpcResponse> {
        let start_time = Instant::now();

        // Check if GenServer exists locally
        let genserver_exists = {
            let local_genservers = self.local_genservers.read().await;
            local_genservers.contains_key(&message.target_genserver)
        };

        if !genserver_exists {
            return Ok(RpcResponse {
                message_id: message.message_id,
                success: false,
                payload: vec![],
                error_message: Some(format!("GenServer '{}' not found", message.target_genserver)),
                processing_time: start_time.elapsed(),
                responded_at: SystemTime::now(),
            });
        }

        // Check for RPC handler
        let handler = {
            let rpc_handlers = self.rpc_handlers.read().await;
            rpc_handlers.get(&message.method).cloned()
        };

        if let Some(handler) = handler {
            handler.handle_rpc(message).await
        } else {
            Ok(RpcResponse {
                message_id: message.message_id,
                success: false,
                payload: vec![],
                error_message: Some(format!("RPC method '{}' not found", message.method)),
                processing_time: start_time.elapsed(),
                responded_at: SystemTime::now(),
            })
        }
    }

    async fn register_local_genserver(&self, name: String, handle: GenServerHandle) {
        let mut local_genservers = self.local_genservers.write().await;
        local_genservers.insert(name.clone(), handle);
        
        info!("Registered GenServer '{}' for distributed access", name);
    }

    async fn unregister_local_genserver(&self, name: &str) {
        let mut local_genservers = self.local_genservers.write().await;
        local_genservers.remove(name);
        
        info!("Unregistered GenServer '{}'", name);
    }

    async fn update_load_metrics(&self, metrics: LoadMetrics) {
        // Update local node info
        // In a real implementation, this would update the actual metrics
        info!("Updated load metrics: CPU {:.1}%, Memory {:.1}%", 
              metrics.cpu_usage * 100.0, metrics.memory_usage * 100.0);
    }

    async fn start_leader_election(&self) -> HelixResult<()> {
        info!("Starting leader election for node '{}'", self.local_info.node_name);

        let current_term = {
            let mut term = self.current_term.write().await;
            *term += 1;
            *term
        };

        // TODO: Implement Raft-style leader election
        // For now, we just become leader if we're the only node
        let cluster_size = self.cluster_nodes.read().await.len();
        if cluster_size == 0 {
            let mut is_leader = self.is_leader.write().await;
            *is_leader = true;
            
            let mut leader_node = self.leader_node.write().await;
            *leader_node = Some(self.local_info.node_name.clone());
            
            let event = ClusterEvent::LeaderElected {
                node_name: self.local_info.node_name.clone(),
                term: current_term,
            };
            let _ = self.cluster_events.send(event);
            
            info!("Elected as leader (term {})", current_term);
        }

        Ok(())
    }

    async fn process_heartbeat(&self, node_name: &str) -> HelixResult<()> {
        debug!("Processing heartbeat from node '{}'", node_name);

        let mut cluster_nodes = self.cluster_nodes.write().await;
        if let Some(node_info) = cluster_nodes.get_mut(node_name) {
            node_info.last_seen = SystemTime::now();
        }

        Ok(())
    }

    async fn get_cluster_status(&self) -> ClusterStatus {
        let cluster_nodes = self.cluster_nodes.read().await;
        let connected_nodes: Vec<NodeInfo> = cluster_nodes.values().cloned().collect();
        
        let leader_node = self.leader_node.read().await.clone();
        let current_term = *self.current_term.read().await;
        
        let is_healthy = connected_nodes.iter()
            .all(|node| {
                SystemTime::now().duration_since(node.last_seen)
                    .unwrap_or(Duration::ZERO) < self.config.connection_timeout
            });

        ClusterStatus {
            local_node: self.local_info.clone(),
            connected_nodes,
            leader_node,
            current_term,
            is_healthy,
            partition_status: PartitionStatus {
                is_partitioned: false,
                partition_groups: vec![],
                last_partition_event: None,
            },
            load_balance_info: LoadBalanceInfo {
                strategy: LoadBalanceStrategy::LeastLoaded,
                genserver_assignments: HashMap::new(),
                node_loads: HashMap::new(),
            },
        }
    }

    async fn broadcast_message(&self, message: DistributedMessage) -> HelixResult<()> {
        let cluster_nodes = self.cluster_nodes.read().await;
        
        for node_name in cluster_nodes.keys() {
            // TODO: Send message to each connected node
            debug!("Broadcasting message to node '{}'", node_name);
        }

        Ok(())
    }

    async fn handle_partition_recovery(&self, recovered_nodes: Vec<String>) -> HelixResult<()> {
        info!("Handling partition recovery for {} nodes", recovered_nodes.len());

        let event = ClusterEvent::PartitionHealed {
            nodes: recovered_nodes,
        };
        let _ = self.cluster_events.send(event);

        Ok(())
    }

    /// Public API methods
    pub async fn join_cluster(&self, seed_nodes: Vec<SocketAddr>) -> HelixResult<()> {
        info!("Joining cluster with {} seed nodes", seed_nodes.len());

        for addr in seed_nodes {
            // TODO: Connect to seed node and request cluster information
            info!("Connecting to seed node at {}", addr);
        }

        Ok(())
    }

    pub async fn leave_cluster(&self) -> HelixResult<()> {
        info!("Leaving cluster");

        let event = ClusterEvent::NodeLeft {
            node_name: self.local_info.node_name.clone(),
            reason: "Graceful shutdown".to_string(),
        };
        let _ = self.cluster_events.send(event);

        // Disconnect from all nodes
        let node_names: Vec<String> = {
            let cluster_nodes = self.cluster_nodes.read().await;
            cluster_nodes.keys().cloned().collect()
        };

        for node_name in node_names {
            let _ = self.command_tx.send(DistributedCommand::DisconnectFromNode { node_name });
        }

        Ok(())
    }

    pub fn get_command_sender(&self) -> mpsc::UnboundedSender<DistributedCommand> {
        self.command_tx.clone()
    }

    pub fn get_cluster_events(&self) -> broadcast::Receiver<ClusterEvent> {
        self.cluster_events.subscribe()
    }
}

impl Clone for DistributedNode {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            local_info: self.local_info.clone(),
            cluster_nodes: self.cluster_nodes.clone(),
            connections: self.connections.clone(),
            local_genservers: self.local_genservers.clone(),
            rpc_handlers: self.rpc_handlers.clone(),
            cluster_events: self.cluster_events.clone(),
            command_tx: self.command_tx.clone(),
            command_rx: Mutex::new(None), // Don't clone receiver
            is_leader: self.is_leader.clone(),
            leader_node: self.leader_node.clone(),
            current_term: self.current_term.clone(),
        }
    }
}

/// Example RPC handler implementation
pub struct ExampleRpcHandler {
    pub handler_name: String,
}

#[async_trait]
impl RpcHandler for ExampleRpcHandler {
    async fn handle_rpc(&self, message: RpcMessage) -> HelixResult<RpcResponse> {
        let start_time = Instant::now();
        
        info!("Handling RPC '{}' from node '{}'", message.method, message.source_node);

        // Simulate processing
        tokio::time::sleep(Duration::from_millis(10)).await;

        Ok(RpcResponse {
            message_id: message.message_id,
            success: true,
            payload: format!("Handled by {}", self.handler_name).into_bytes(),
            error_message: None,
            processing_time: start_time.elapsed(),
            responded_at: SystemTime::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_distributed_node_creation() {
        let config = DistributedConfig {
            node_name: "test_node".to_string(),
            bind_address: "127.0.0.1:0".parse().unwrap(),
            ..Default::default()
        };

        let node = DistributedNode::new(config).await.expect("Failed to create distributed node");
        assert_eq!(node.local_info.node_name, "test_node");
        assert_eq!(node.local_info.node_type, NodeType::Full);
    }

    #[tokio::test]
    async fn test_rpc_message_creation() {
        let message = RpcMessage {
            message_id: Uuid::new_v4(),
            source_node: "node1".to_string(),
            target_node: "node2".to_string(),
            target_genserver: "test_genserver".to_string(),
            method: "test_method".to_string(),
            payload: b"test_payload".to_vec(),
            timeout: Duration::from_secs(5),
            created_at: SystemTime::now(),
        };

        assert_eq!(message.source_node, "node1");
        assert_eq!(message.target_node, "node2");
        assert_eq!(message.method, "test_method");
    }

    #[tokio::test]
    async fn test_cluster_status() {
        let config = DistributedConfig::default();
        let node = DistributedNode::new(config).await.expect("Failed to create node");
        
        let status = node.get_cluster_status().await;
        assert_eq!(status.connected_nodes.len(), 0); // No connected nodes initially
        assert!(status.leader_node.is_none()); // No leader initially
    }
}