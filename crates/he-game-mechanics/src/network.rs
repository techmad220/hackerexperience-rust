//! Network system mechanics - Connection protocols, routing, bandwidth calculations

use crate::{PlayerState, HardwareSpecs};
use crate::config::NetworkConfig;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::net::{IpAddr, Ipv4Addr};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Network node types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    HomeComputer,
    PersonalServer,
    CorporateServer,
    BankServer,
    GovernmentServer,
    UniversityServer,
    ISPNode,
    DNSServer,
    WebServer,
    DatabaseServer,
    MailServer,
    GameServer,
    BitcoinNode,
    TorRelay,
    VPNServer,
    ProxyServer,
    Honeypot,
    Firewall,
    Router,
    Switch,
}

impl NodeType {
    pub fn base_security_level(&self) -> i32 {
        match self {
            NodeType::HomeComputer => 10,
            NodeType::PersonalServer => 25,
            NodeType::CorporateServer => 60,
            NodeType::BankServer => 90,
            NodeType::GovernmentServer => 95,
            NodeType::UniversityServer => 40,
            NodeType::ISPNode => 70,
            NodeType::DNSServer => 50,
            NodeType::WebServer => 45,
            NodeType::DatabaseServer => 65,
            NodeType::MailServer => 35,
            NodeType::GameServer => 30,
            NodeType::BitcoinNode => 55,
            NodeType::TorRelay => 40,
            NodeType::VPNServer => 50,
            NodeType::ProxyServer => 35,
            NodeType::Honeypot => 20, // Low security to attract attackers
            NodeType::Firewall => 80,
            NodeType::Router => 25,
            NodeType::Switch => 15,
        }
    }
    
    pub fn bandwidth_capacity_mbps(&self) -> i32 {
        match self {
            NodeType::HomeComputer => 100,
            NodeType::PersonalServer => 250,
            NodeType::CorporateServer => 1000,
            NodeType::BankServer => 10000,
            NodeType::GovernmentServer => 10000,
            NodeType::UniversityServer => 5000,
            NodeType::ISPNode => 100000,
            NodeType::DNSServer => 10000,
            NodeType::WebServer => 5000,
            NodeType::DatabaseServer => 2000,
            NodeType::MailServer => 1000,
            NodeType::GameServer => 10000,
            NodeType::BitcoinNode => 500,
            NodeType::TorRelay => 100,
            NodeType::VPNServer => 1000,
            NodeType::ProxyServer => 500,
            NodeType::Honeypot => 100,
            NodeType::Firewall => 10000,
            NodeType::Router => 10000,
            NodeType::Switch => 1000,
        }
    }
}

/// Network node representing a computer or server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkNode {
    pub id: Uuid,
    pub node_type: NodeType,
    pub ip_address: IpAddr,
    pub hostname: String,
    pub owner_id: Option<i32>,
    pub is_online: bool,
    pub is_visible: bool,
    pub security_level: i32,
    pub firewall_strength: i32,
    pub bandwidth_total: i32,
    pub bandwidth_used: i32,
    pub open_ports: HashSet<u16>,
    pub services: HashMap<u16, String>,
    pub connected_nodes: HashSet<Uuid>,
    pub routing_table: HashMap<IpAddr, Uuid>,
    pub access_logs: VecDeque<AccessLog>,
    pub intrusion_attempts: i32,
    pub last_maintenance: Option<SystemTime>,
    pub uptime: Duration,
    pub hardware_specs: Option<HardwareSpecs>,
}

impl NetworkNode {
    pub fn new(node_type: NodeType, ip_address: IpAddr) -> Self {
        let hostname = format!("{:?}-{}", node_type, rand::random::<u16>());
        
        NetworkNode {
            id: Uuid::new_v4(),
            node_type: node_type.clone(),
            ip_address,
            hostname,
            owner_id: None,
            is_online: true,
            is_visible: true,
            security_level: node_type.base_security_level(),
            firewall_strength: 50,
            bandwidth_total: node_type.bandwidth_capacity_mbps(),
            bandwidth_used: 0,
            open_ports: Self::default_open_ports(&node_type),
            services: Self::default_services(&node_type),
            connected_nodes: HashSet::new(),
            routing_table: HashMap::new(),
            access_logs: VecDeque::with_capacity(1000),
            intrusion_attempts: 0,
            last_maintenance: Some(SystemTime::now()),
            uptime: Duration::from_secs(0),
            hardware_specs: None,
        }
    }
    
    fn default_open_ports(node_type: &NodeType) -> HashSet<u16> {
        let mut ports = HashSet::new();
        
        match node_type {
            NodeType::WebServer => {
                ports.insert(80);   // HTTP
                ports.insert(443);  // HTTPS
                ports.insert(8080); // Alt HTTP
            },
            NodeType::MailServer => {
                ports.insert(25);   // SMTP
                ports.insert(110);  // POP3
                ports.insert(143);  // IMAP
                ports.insert(587);  // SMTP Submission
            },
            NodeType::DatabaseServer => {
                ports.insert(3306); // MySQL
                ports.insert(5432); // PostgreSQL
                ports.insert(27017); // MongoDB
            },
            NodeType::GameServer => {
                ports.insert(25565); // Minecraft
                ports.insert(27015); // Source Engine
                ports.insert(3724);  // WoW
            },
            NodeType::DNSServer => {
                ports.insert(53);    // DNS
            },
            NodeType::BitcoinNode => {
                ports.insert(8333);  // Bitcoin
            },
            _ => {
                ports.insert(22);    // SSH
                ports.insert(80);    // HTTP
            }
        }
        
        ports
    }
    
    fn default_services(node_type: &NodeType) -> HashMap<u16, String> {
        let mut services = HashMap::new();
        
        for port in Self::default_open_ports(node_type) {
            let service = match port {
                22 => "SSH",
                25 => "SMTP",
                53 => "DNS",
                80 => "HTTP",
                110 => "POP3",
                143 => "IMAP",
                443 => "HTTPS",
                587 => "SMTP-Submission",
                3306 => "MySQL",
                5432 => "PostgreSQL",
                8080 => "HTTP-Alt",
                8333 => "Bitcoin",
                25565 => "Minecraft",
                27015 => "Source",
                27017 => "MongoDB",
                _ => "Unknown",
            };
            services.insert(port, service.to_string());
        }
        
        services
    }
    
    pub fn add_connection(&mut self, node_id: Uuid) {
        self.connected_nodes.insert(node_id);
    }
    
    pub fn remove_connection(&mut self, node_id: Uuid) {
        self.connected_nodes.remove(&node_id);
    }
    
    pub fn open_port(&mut self, port: u16, service: String) {
        self.open_ports.insert(port);
        self.services.insert(port, service);
    }
    
    pub fn close_port(&mut self, port: u16) {
        self.open_ports.remove(&port);
        self.services.remove(&port);
    }
    
    pub fn is_port_open(&self, port: u16) -> bool {
        self.open_ports.contains(&port)
    }
    
    pub fn log_access(&mut self, access: AccessLog) {
        if self.access_logs.len() >= 1000 {
            self.access_logs.pop_front();
        }
        self.access_logs.push_back(access);
    }
    
    pub fn calculate_effective_bandwidth(&self) -> i32 {
        let available = self.bandwidth_total - self.bandwidth_used;
        
        // Apply node type efficiency
        let efficiency = match self.node_type {
            NodeType::ISPNode => 0.95,
            NodeType::CorporateServer | NodeType::BankServer => 0.90,
            NodeType::GovernmentServer => 0.85,
            NodeType::WebServer | NodeType::DatabaseServer => 0.80,
            NodeType::HomeComputer => 0.70,
            NodeType::TorRelay => 0.50, // Tor is slower
            _ => 0.75,
        };
        
        (available as f32 * efficiency) as i32
    }
    
    pub fn calculate_security_score(&self) -> i32 {
        let base = self.security_level;
        let firewall_bonus = self.firewall_strength / 2;
        let port_penalty = (self.open_ports.len() as i32 - 2) * 5; // Penalty for too many open ports
        let intrusion_penalty = self.intrusion_attempts.min(50);
        
        (base + firewall_bonus - port_penalty - intrusion_penalty).max(0)
    }
}

/// Access log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessLog {
    pub timestamp: SystemTime,
    pub source_ip: IpAddr,
    pub destination_port: u16,
    pub action: AccessAction,
    pub success: bool,
    pub data_transferred: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessAction {
    Connect,
    Disconnect,
    Login,
    Logout,
    Download,
    Upload,
    Execute,
    Scan,
    Exploit,
    Blocked,
}

/// Network connection between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub id: Uuid,
    pub source_node: Uuid,
    pub destination_node: Uuid,
    pub connection_type: ConnectionType,
    pub established_at: SystemTime,
    pub latency_ms: i32,
    pub packet_loss: f32,
    pub bandwidth_allocated: i32,
    pub data_transferred: i64,
    pub is_encrypted: bool,
    pub encryption_strength: i32,
    pub route: Vec<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    Direct,
    Proxied,
    VPN,
    Tor,
    Bounced,
    Tunneled,
}

impl NetworkConnection {
    pub fn new(source: Uuid, destination: Uuid, connection_type: ConnectionType) -> Self {
        NetworkConnection {
            id: Uuid::new_v4(),
            source_node: source,
            destination_node: destination,
            connection_type,
            established_at: SystemTime::now(),
            latency_ms: 50,
            packet_loss: 0.0,
            bandwidth_allocated: 100,
            data_transferred: 0,
            is_encrypted: matches!(connection_type, ConnectionType::VPN | ConnectionType::Tor),
            encryption_strength: if matches!(connection_type, ConnectionType::VPN | ConnectionType::Tor) { 256 } else { 0 },
            route: Vec::new(),
        }
    }
    
    pub fn calculate_effective_speed(&self) -> i32 {
        let base_speed = self.bandwidth_allocated;
        let latency_factor = 1.0 - (self.latency_ms as f32 / 1000.0).min(0.5);
        let loss_factor = 1.0 - self.packet_loss;
        
        let type_multiplier = match self.connection_type {
            ConnectionType::Direct => 1.0,
            ConnectionType::Proxied => 0.8,
            ConnectionType::VPN => 0.7,
            ConnectionType::Tor => 0.3,
            ConnectionType::Bounced => 0.5,
            ConnectionType::Tunneled => 0.6,
        };
        
        (base_speed as f32 * latency_factor * loss_factor * type_multiplier) as i32
    }
    
    pub fn add_hop(&mut self, node_id: Uuid) {
        self.route.push(node_id);
        self.latency_ms += 10; // Each hop adds latency
        self.packet_loss += 0.001; // Each hop adds potential packet loss
    }
}

/// Network topology manager
#[derive(Debug, Clone)]
pub struct NetworkTopology {
    pub nodes: HashMap<Uuid, NetworkNode>,
    pub connections: HashMap<Uuid, NetworkConnection>,
    pub ip_to_node: HashMap<IpAddr, Uuid>,
    pub subnets: HashMap<String, Vec<Uuid>>,
}

impl NetworkTopology {
    pub fn new() -> Self {
        NetworkTopology {
            nodes: HashMap::new(),
            connections: HashMap::new(),
            ip_to_node: HashMap::new(),
            subnets: HashMap::new(),
        }
    }
    
    pub fn add_node(&mut self, node: NetworkNode) -> Uuid {
        let node_id = node.id;
        let ip = node.ip_address;
        let subnet = Self::get_subnet(&ip);
        
        self.ip_to_node.insert(ip, node_id);
        self.nodes.insert(node_id, node);
        
        self.subnets
            .entry(subnet)
            .or_insert_with(Vec::new)
            .push(node_id);
        
        node_id
    }
    
    pub fn remove_node(&mut self, node_id: Uuid) {
        if let Some(node) = self.nodes.remove(&node_id) {
            self.ip_to_node.remove(&node.ip_address);
            
            // Remove from subnet
            let subnet = Self::get_subnet(&node.ip_address);
            if let Some(nodes) = self.subnets.get_mut(&subnet) {
                nodes.retain(|&id| id != node_id);
            }
            
            // Remove all connections
            self.connections.retain(|_, conn| {
                conn.source_node != node_id && conn.destination_node != node_id
            });
        }
    }
    
    pub fn establish_connection(
        &mut self,
        source_id: Uuid,
        destination_id: Uuid,
        connection_type: ConnectionType,
    ) -> Result<Uuid, String> {
        // Check if nodes exist
        if !self.nodes.contains_key(&source_id) {
            return Err("Source node not found".to_string());
        }
        if !self.nodes.contains_key(&destination_id) {
            return Err("Destination node not found".to_string());
        }
        
        // Check if connection already exists
        for conn in self.connections.values() {
            if conn.source_node == source_id && conn.destination_node == destination_id {
                return Err("Connection already exists".to_string());
            }
        }
        
        // Calculate route
        let route = self.find_route(source_id, destination_id)?;
        
        // Create connection
        let mut connection = NetworkConnection::new(source_id, destination_id, connection_type);
        
        // Add hops for indirect connections
        for hop in route.iter().skip(1).take(route.len() - 2) {
            connection.add_hop(*hop);
        }
        
        // Calculate latency based on route
        connection.latency_ms = self.calculate_route_latency(&route);
        
        let conn_id = connection.id;
        self.connections.insert(conn_id, connection);
        
        // Update nodes
        if let Some(source) = self.nodes.get_mut(&source_id) {
            source.add_connection(destination_id);
        }
        if let Some(dest) = self.nodes.get_mut(&destination_id) {
            dest.add_connection(source_id);
        }
        
        Ok(conn_id)
    }
    
    pub fn terminate_connection(&mut self, connection_id: Uuid) -> Result<(), String> {
        let connection = self.connections.remove(&connection_id)
            .ok_or("Connection not found")?;
        
        // Update nodes
        if let Some(source) = self.nodes.get_mut(&connection.source_node) {
            source.remove_connection(connection.destination_node);
        }
        if let Some(dest) = self.nodes.get_mut(&connection.destination_node) {
            dest.remove_connection(connection.source_node);
        }
        
        Ok(())
    }
    
    pub fn find_route(&self, source: Uuid, destination: Uuid) -> Result<Vec<Uuid>, String> {
        // Simple BFS pathfinding
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent = HashMap::new();
        
        queue.push_back(source);
        visited.insert(source);
        
        while let Some(current) = queue.pop_front() {
            if current == destination {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = destination;
                
                while node != source {
                    path.push(node);
                    node = *parent.get(&node).unwrap();
                }
                path.push(source);
                path.reverse();
                
                return Ok(path);
            }
            
            if let Some(node) = self.nodes.get(&current) {
                for &neighbor in &node.connected_nodes {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        parent.insert(neighbor, current);
                        queue.push_back(neighbor);
                    }
                }
            }
        }
        
        Err("No route found".to_string())
    }
    
    fn calculate_route_latency(&self, route: &[Uuid]) -> i32 {
        let base_latency = 10;
        let hop_latency = 5;
        let distance_factor = route.len() as i32 * hop_latency;
        
        base_latency + distance_factor
    }
    
    fn get_subnet(ip: &IpAddr) -> String {
        match ip {
            IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();
                format!("{}.{}.{}.0/24", octets[0], octets[1], octets[2])
            },
            IpAddr::V6(_) => "ipv6".to_string(),
        }
    }
    
    pub fn scan_subnet(&self, subnet: &str) -> Vec<&NetworkNode> {
        self.subnets.get(subnet)
            .map(|node_ids| {
                node_ids.iter()
                    .filter_map(|id| self.nodes.get(id))
                    .filter(|node| node.is_online && node.is_visible)
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Calculate network latency between nodes
pub fn calculate_latency(source: &NetworkNode, destination: &NetworkNode, hops: usize) -> i32 {
    let base_latency = 10; // Base latency in ms
    
    // Distance factor (more hops = more latency)
    let hop_latency = hops as i32 * 5;
    
    // Node type factors
    let source_factor = match source.node_type {
        NodeType::ISPNode => 0.8,
        NodeType::HomeComputer => 1.2,
        NodeType::TorRelay => 2.0,
        _ => 1.0,
    };
    
    let dest_factor = match destination.node_type {
        NodeType::ISPNode => 0.8,
        NodeType::HomeComputer => 1.2,
        NodeType::TorRelay => 2.0,
        _ => 1.0,
    };
    
    let congestion = (source.bandwidth_used as f32 / source.bandwidth_total as f32 * 20.0) as i32;
    
    ((base_latency + hop_latency + congestion) as f32 * source_factor * dest_factor) as i32
}

/// Calculate bandwidth usage for a connection
pub fn calculate_bandwidth_usage(
    connection: &NetworkConnection,
    data_size_mb: i32,
) -> i32 {
    let base_usage = data_size_mb * 8; // Convert to Mbps
    
    // Apply connection type overhead
    let overhead = match connection.connection_type {
        ConnectionType::Direct => 1.0,
        ConnectionType::Proxied => 1.1,
        ConnectionType::VPN => 1.2,
        ConnectionType::Tor => 1.5,
        ConnectionType::Bounced => 1.3,
        ConnectionType::Tunneled => 1.15,
    };
    
    // Apply encryption overhead
    let encryption_overhead = if connection.is_encrypted { 1.1 } else { 1.0 };
    
    (base_usage as f32 * overhead * encryption_overhead) as i32
}

/// Port scanning functionality
pub fn port_scan(target: &NetworkNode, scan_type: ScanType) -> Vec<(u16, bool, Option<String>)> {
    let mut results = Vec::new();
    
    let ports_to_scan = match scan_type {
        ScanType::Quick => vec![21, 22, 23, 25, 80, 443, 3306, 8080],
        ScanType::Common => {
            let mut ports = vec![21, 22, 23, 25, 53, 80, 110, 143, 443, 445, 3306, 3389, 5432, 8080, 8443];
            ports.extend(&[111, 135, 139, 445, 1433, 1521, 2049, 5900, 6379, 8080, 27017]);
            ports
        },
        ScanType::Full => (1..=65535).collect(),
        ScanType::Custom(ref ports) => ports.clone(),
    };
    
    for port in ports_to_scan {
        let is_open = target.is_port_open(port);
        let service = if is_open {
            target.services.get(&port).cloned()
        } else {
            None
        };
        
        results.push((port, is_open, service));
    }
    
    results
}

#[derive(Debug, Clone)]
pub enum ScanType {
    Quick,
    Common,
    Full,
    Custom(Vec<u16>),
}

/// Network intrusion detection
pub fn detect_intrusion(node: &mut NetworkNode, connection: &NetworkConnection) -> Option<IntrusionAlert> {
    let mut threat_level = 0;
    let mut alert_type = IntrusionType::None;
    
    // Check for port scanning
    let recent_connections = node.access_logs.iter()
        .filter(|log| {
            SystemTime::now().duration_since(log.timestamp)
                .unwrap_or_default().as_secs() < 60
        })
        .count();
    
    if recent_connections > 100 {
        threat_level += 50;
        alert_type = IntrusionType::PortScan;
    }
    
    // Check for brute force attempts
    let failed_logins = node.access_logs.iter()
        .filter(|log| log.action == AccessAction::Login && !log.success)
        .count();
    
    if failed_logins > 10 {
        threat_level += 30;
        alert_type = IntrusionType::BruteForce;
    }
    
    // Check for exploit attempts
    if node.access_logs.iter().any(|log| log.action == AccessAction::Exploit) {
        threat_level += 70;
        alert_type = IntrusionType::Exploit;
    }
    
    // Check for DDoS
    if node.bandwidth_used > node.bandwidth_total * 90 / 100 {
        threat_level += 60;
        alert_type = IntrusionType::DDoS;
    }
    
    if threat_level > 30 {
        node.intrusion_attempts += 1;
        
        Some(IntrusionAlert {
            timestamp: SystemTime::now(),
            node_id: node.id,
            source_ip: node.ip_address, // Should be from connection
            intrusion_type: alert_type,
            threat_level,
            blocked: threat_level > 50,
        })
    } else {
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrusionAlert {
    pub timestamp: SystemTime,
    pub node_id: Uuid,
    pub source_ip: IpAddr,
    pub intrusion_type: IntrusionType,
    pub threat_level: i32,
    pub blocked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntrusionType {
    None,
    PortScan,
    BruteForce,
    Exploit,
    DDoS,
    Malware,
    DataExfiltration,
}

/// Calculate network route optimization
pub fn optimize_route(topology: &NetworkTopology, source: Uuid, destination: Uuid) -> Result<Vec<Uuid>, String> {
    // Find all possible routes
    let direct_route = topology.find_route(source, destination)?;
    
    // Calculate metrics for direct route
    let direct_latency = topology.calculate_route_latency(&direct_route);
    
    // Try to find alternative routes through high-bandwidth nodes
    let mut best_route = direct_route.clone();
    let mut best_score = direct_latency;
    
    // Look for ISP nodes or high-bandwidth servers
    for node in topology.nodes.values() {
        if matches!(node.node_type, NodeType::ISPNode | NodeType::CorporateServer) {
            if let Ok(route1) = topology.find_route(source, node.id) {
                if let Ok(route2) = topology.find_route(node.id, destination) {
                    let mut combined_route = route1;
                    combined_route.extend(route2.iter().skip(1));
                    
                    let latency = topology.calculate_route_latency(&combined_route);
                    if latency < best_score {
                        best_score = latency;
                        best_route = combined_route;
                    }
                }
            }
        }
    }
    
    Ok(best_route)
}

/// Bandwidth allocation and QoS
pub fn allocate_bandwidth(
    node: &mut NetworkNode,
    connection: &mut NetworkConnection,
    requested_bandwidth: i32,
) -> Result<i32, String> {
    let available = node.bandwidth_total - node.bandwidth_used;
    
    if available < requested_bandwidth {
        // Apply QoS - allocate what's available
        let allocated = available.max(10); // Minimum 10 Mbps
        connection.bandwidth_allocated = allocated;
        node.bandwidth_used += allocated;
        Ok(allocated)
    } else {
        connection.bandwidth_allocated = requested_bandwidth;
        node.bandwidth_used += requested_bandwidth;
        Ok(requested_bandwidth)
    }
}

/// Network performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub total_nodes: usize,
    pub online_nodes: usize,
    pub total_connections: usize,
    pub average_latency: f32,
    pub total_bandwidth_capacity: i64,
    pub total_bandwidth_used: i64,
    pub packet_loss_rate: f32,
    pub intrusion_attempts: i32,
}

pub fn calculate_network_metrics(topology: &NetworkTopology) -> NetworkMetrics {
    let total_nodes = topology.nodes.len();
    let online_nodes = topology.nodes.values().filter(|n| n.is_online).count();
    let total_connections = topology.connections.len();
    
    let average_latency = if total_connections > 0 {
        topology.connections.values()
            .map(|c| c.latency_ms as f32)
            .sum::<f32>() / total_connections as f32
    } else {
        0.0
    };
    
    let total_bandwidth_capacity: i64 = topology.nodes.values()
        .map(|n| n.bandwidth_total as i64)
        .sum();
    
    let total_bandwidth_used: i64 = topology.nodes.values()
        .map(|n| n.bandwidth_used as i64)
        .sum();
    
    let packet_loss_rate = topology.connections.values()
        .map(|c| c.packet_loss)
        .sum::<f32>() / total_connections.max(1) as f32;
    
    let intrusion_attempts: i32 = topology.nodes.values()
        .map(|n| n.intrusion_attempts)
        .sum();
    
    NetworkMetrics {
        total_nodes,
        online_nodes,
        total_connections,
        average_latency,
        total_bandwidth_capacity,
        total_bandwidth_used,
        packet_loss_rate,
        intrusion_attempts,
    }
}

/// Generate network map
pub fn generate_network_map(topology: &NetworkTopology, center_node: Uuid) -> NetworkMap {
    let mut map = NetworkMap {
        nodes: Vec::new(),
        connections: Vec::new(),
        subnets: HashMap::new(),
    };
    
    // Add center node
    if let Some(node) = topology.nodes.get(&center_node) {
        map.nodes.push(NodeInfo {
            id: node.id,
            ip: node.ip_address,
            hostname: node.hostname.clone(),
            node_type: node.node_type.clone(),
            is_online: node.is_online,
            security_level: node.security_level,
        });
        
        // Add connected nodes (1 hop away)
        for &connected_id in &node.connected_nodes {
            if let Some(connected) = topology.nodes.get(&connected_id) {
                map.nodes.push(NodeInfo {
                    id: connected.id,
                    ip: connected.ip_address,
                    hostname: connected.hostname.clone(),
                    node_type: connected.node_type.clone(),
                    is_online: connected.is_online,
                    security_level: connected.security_level,
                });
            }
        }
    }
    
    // Add connections
    for connection in topology.connections.values() {
        if connection.source_node == center_node || connection.destination_node == center_node ||
           topology.nodes.get(&center_node).map(|n| n.connected_nodes.contains(&connection.source_node)).unwrap_or(false) {
            map.connections.push(ConnectionInfo {
                id: connection.id,
                source: connection.source_node,
                destination: connection.destination_node,
                latency: connection.latency_ms,
                bandwidth: connection.bandwidth_allocated,
            });
        }
    }
    
    map
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMap {
    pub nodes: Vec<NodeInfo>,
    pub connections: Vec<ConnectionInfo>,
    pub subnets: HashMap<String, Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: Uuid,
    pub ip: IpAddr,
    pub hostname: String,
    pub node_type: NodeType,
    pub is_online: bool,
    pub security_level: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub id: Uuid,
    pub source: Uuid,
    pub destination: Uuid,
    pub latency: i32,
    pub bandwidth: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    
    #[test]
    fn test_node_creation() {
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        let node = NetworkNode::new(NodeType::HomeComputer, ip);
        
        assert_eq!(node.node_type, NodeType::HomeComputer);
        assert_eq!(node.ip_address, ip);
        assert!(node.is_online);
        assert_eq!(node.security_level, 10);
    }
    
    #[test]
    fn test_connection_creation() {
        let source = Uuid::new_v4();
        let dest = Uuid::new_v4();
        let conn = NetworkConnection::new(source, dest, ConnectionType::Direct);
        
        assert_eq!(conn.source_node, source);
        assert_eq!(conn.destination_node, dest);
        assert_eq!(conn.connection_type, ConnectionType::Direct);
        assert!(!conn.is_encrypted);
    }
    
    #[test]
    fn test_topology_management() {
        let mut topology = NetworkTopology::new();
        
        let node1 = NetworkNode::new(
            NodeType::HomeComputer,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))
        );
        let node2 = NetworkNode::new(
            NodeType::WebServer,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 200))
        );
        
        let id1 = topology.add_node(node1);
        let id2 = topology.add_node(node2);
        
        assert_eq!(topology.nodes.len(), 2);
        
        let conn_result = topology.establish_connection(id1, id2, ConnectionType::Direct);
        assert!(conn_result.is_ok());
        assert_eq!(topology.connections.len(), 1);
    }
    
    #[test]
    fn test_port_scanning() {
        let mut node = NetworkNode::new(
            NodeType::WebServer,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))
        );
        
        node.open_port(8080, "HTTP-Alt".to_string());
        
        let results = port_scan(&node, ScanType::Quick);
        
        let port_80 = results.iter().find(|(port, _, _)| *port == 80);
        assert!(port_80.is_some());
        assert!(port_80.unwrap().1); // Should be open
        
        let port_8080 = results.iter().find(|(port, _, _)| *port == 8080);
        assert!(port_8080.is_some());
        assert!(port_8080.unwrap().1); // Should be open
    }
    
    #[test]
    fn test_bandwidth_calculation() {
        let conn = NetworkConnection::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            ConnectionType::VPN
        );
        
        let usage = calculate_bandwidth_usage(&conn, 100);
        assert!(usage > 100); // Should have overhead
    }
    
    #[test]
    fn test_latency_calculation() {
        let node1 = NetworkNode::new(
            NodeType::HomeComputer,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))
        );
        let node2 = NetworkNode::new(
            NodeType::ISPNode,
            IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))
        );
        
        let latency = calculate_latency(&node1, &node2, 5);
        assert!(latency > 0);
        assert!(latency < 1000); // Should be reasonable
    }
}