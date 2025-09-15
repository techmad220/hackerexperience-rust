//! Complete Network Engine - IP management, routing, and topology

use super::{EngineComponent, EngineError, EngineResult, ComponentStatus};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::net::Ipv4Addr;
use std::time::{Duration, SystemTime};
use uuid::Uuid;
use rand::Rng;

/// Server types in the network
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServerType {
    Personal,
    NPC,
    Bank,
    Corporation,
    ISP,
    FBI,
    NewsAgency,
    University,
    Whois,
    DownloadCenter,
}

/// Network node representing a server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkNode {
    pub id: Uuid,
    pub ip_address: Ipv4Addr,
    pub server_type: ServerType,
    pub owner_id: Uuid,
    pub hostname: String,
    pub is_online: bool,
    pub firewall_level: u32,
    pub trace_time: Duration,
    pub connections: HashSet<Ipv4Addr>,
    pub logs: VecDeque<NetworkLog>,
    pub max_logs: usize,
}

impl NetworkNode {
    pub fn new(
        ip_address: Ipv4Addr,
        server_type: ServerType,
        owner_id: Uuid,
        hostname: String,
    ) -> Self {
        let trace_time = match server_type {
            ServerType::Personal => Duration::from_secs(30),
            ServerType::NPC => Duration::from_secs(60),
            ServerType::Bank => Duration::from_secs(180),
            ServerType::Corporation => Duration::from_secs(120),
            ServerType::FBI => Duration::from_secs(300),
            _ => Duration::from_secs(90),
        };

        Self {
            id: Uuid::new_v4(),
            ip_address,
            server_type,
            owner_id,
            hostname,
            is_online: true,
            firewall_level: 0,
            trace_time,
            connections: HashSet::new(),
            logs: VecDeque::with_capacity(100),
            max_logs: 100,
        }
    }

    pub fn add_log(&mut self, log: NetworkLog) {
        if self.logs.len() >= self.max_logs {
            self.logs.pop_front();
        }
        self.logs.push_back(log);
    }

    pub fn clear_logs(&mut self) {
        self.logs.clear();
    }

    pub fn hide_log(&mut self, log_id: Uuid) -> EngineResult<()> {
        match self.logs.iter_mut().find(|l| l.id == log_id) {
            Some(log) => {
                log.hidden = true;
                Ok(())
            }
            None => Err(EngineError::NotFound("Log not found".into())),
        }
    }

    pub fn edit_log(&mut self, log_id: Uuid, new_content: String) -> EngineResult<()> {
        match self.logs.iter_mut().find(|l| l.id == log_id) {
            Some(log) => {
                log.message = new_content;
                log.edited = true;
                Ok(())
            }
            None => Err(EngineError::NotFound("Log not found".into())),
        }
    }
}

/// Network log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkLog {
    pub id: Uuid,
    pub timestamp: SystemTime,
    pub source_ip: Ipv4Addr,
    pub action: LogAction,
    pub message: String,
    pub hidden: bool,
    pub edited: bool,
}

/// Log action types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogAction {
    Login,
    Logout,
    Download,
    Upload,
    Delete,
    Install,
    Crack,
    Scan,
    DDoS,
    Transfer,
    Custom(String),
}

impl NetworkLog {
    pub fn new(source_ip: Ipv4Addr, action: LogAction, message: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            source_ip,
            action,
            message,
            hidden: false,
            edited: false,
        }
    }
}

/// Connection route through the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRoute {
    pub hops: Vec<Ipv4Addr>,
    pub total_latency: Duration,
    pub is_bounced: bool,
}

impl NetworkRoute {
    pub fn direct(from: Ipv4Addr, to: Ipv4Addr) -> Self {
        Self {
            hops: vec![from, to],
            total_latency: Duration::from_millis(50),
            is_bounced: false,
        }
    }

    pub fn bounced(from: Ipv4Addr, through: Vec<Ipv4Addr>, to: Ipv4Addr) -> Self {
        let mut hops = vec![from];
        hops.extend(through);
        hops.push(to);

        let latency = Duration::from_millis(50 * hops.len() as u64);

        Self {
            hops,
            total_latency: latency,
            is_bounced: true,
        }
    }

    pub fn trace_difficulty(&self) -> u32 {
        if self.is_bounced {
            (self.hops.len() as u32 - 2) * 2
        } else {
            1
        }
    }
}

/// Network topology manager
pub struct NetworkTopology {
    nodes: HashMap<Ipv4Addr, NetworkNode>,
    routes: HashMap<(Ipv4Addr, Ipv4Addr), NetworkRoute>,
    dns: HashMap<String, Ipv4Addr>,
    ip_pool: Vec<Ipv4Addr>,
    next_ip_index: usize,
}

impl NetworkTopology {
    pub fn new() -> Self {
        let mut topology = Self {
            nodes: HashMap::new(),
            routes: HashMap::new(),
            dns: HashMap::new(),
            ip_pool: Vec::new(),
            next_ip_index: 0,
        };
        topology.initialize_ip_pool();
        topology.create_initial_nodes();
        topology
    }

    fn initialize_ip_pool(&mut self) {
        // Generate IP addresses for the game world
        for a in 1..=255 {
            for b in 0..=255 {
                // Skip private and reserved ranges
                if a == 10 || a == 127 || (a == 172 && b >= 16 && b <= 31) || (a == 192 && b == 168) {
                    continue;
                }

                // Generate some IPs (not all 16M)
                if a % 10 == 0 && b % 10 == 0 {
                    let ip = Ipv4Addr::new(a, b, rand::thread_rng().gen_range(1..255), rand::thread_rng().gen_range(1..255));
                    self.ip_pool.push(ip);
                }
            }
        }
    }

    fn create_initial_nodes(&mut self) {
        // Create important servers
        self.create_node(
            Ipv4Addr::new(1, 2, 3, 4),
            ServerType::FBI,
            Uuid::nil(),
            "fbi.gov".to_string(),
        );

        self.create_node(
            Ipv4Addr::new(8, 8, 8, 8),
            ServerType::ISP,
            Uuid::nil(),
            "isp.net".to_string(),
        );

        self.create_node(
            Ipv4Addr::new(10, 10, 10, 10),
            ServerType::Bank,
            Uuid::nil(),
            "firstbank.com".to_string(),
        );

        self.create_node(
            Ipv4Addr::new(20, 20, 20, 20),
            ServerType::University,
            Uuid::nil(),
            "university.edu".to_string(),
        );

        self.create_node(
            Ipv4Addr::new(30, 30, 30, 30),
            ServerType::Whois,
            Uuid::nil(),
            "whois.net".to_string(),
        );

        self.create_node(
            Ipv4Addr::new(40, 40, 40, 40),
            ServerType::DownloadCenter,
            Uuid::nil(),
            "downloads.com".to_string(),
        );

        // Create NPC servers
        for i in 1..=50 {
            let ip = self.get_random_ip();
            self.create_node(
                ip,
                ServerType::NPC,
                Uuid::nil(),
                format!("npc{}.local", i),
            );
        }
    }

    pub fn create_node(&mut self, ip: Ipv4Addr, server_type: ServerType, owner_id: Uuid, hostname: String) {
        let node = NetworkNode::new(ip, server_type, owner_id, hostname.clone());
        self.dns.insert(hostname, ip);
        self.nodes.insert(ip, node);
    }

    pub fn get_random_ip(&mut self) -> Ipv4Addr {
        if self.next_ip_index >= self.ip_pool.len() {
            self.next_ip_index = 0;
        }
        let ip = self.ip_pool[self.next_ip_index];
        self.next_ip_index += 1;
        ip
    }

    pub fn resolve_hostname(&self, hostname: &str) -> Option<Ipv4Addr> {
        self.dns.get(hostname).copied()
    }

    pub fn get_node(&self, ip: Ipv4Addr) -> Option<&NetworkNode> {
        self.nodes.get(&ip)
    }

    pub fn get_node_mut(&mut self, ip: Ipv4Addr) -> Option<&mut NetworkNode> {
        self.nodes.get_mut(&ip)
    }

    pub fn create_route(&mut self, from: Ipv4Addr, to: Ipv4Addr, bounce_nodes: Vec<Ipv4Addr>) {
        let route = if bounce_nodes.is_empty() {
            NetworkRoute::direct(from, to)
        } else {
            NetworkRoute::bounced(from, bounce_nodes, to)
        };
        self.routes.insert((from, to), route);
    }

    pub fn get_route(&self, from: Ipv4Addr, to: Ipv4Addr) -> NetworkRoute {
        self.routes.get(&(from, to))
            .cloned()
            .unwrap_or_else(|| NetworkRoute::direct(from, to))
    }

    pub fn trace_route(&self, from: Ipv4Addr, to: Ipv4Addr) -> Vec<Ipv4Addr> {
        let route = self.get_route(from, to);
        route.hops
    }

    pub fn scan_network(&self, from: Ipv4Addr, radius: u32) -> Vec<Ipv4Addr> {
        let mut found = Vec::new();
        let from_octets = from.octets();

        for node in self.nodes.keys() {
            let octets = node.octets();
            let distance = ((octets[0] as i32 - from_octets[0] as i32).abs() +
                           (octets[1] as i32 - from_octets[1] as i32).abs()) as u32;

            if distance <= radius {
                found.push(*node);
            }
        }

        found
    }
}

/// Main Network Engine
pub struct NetworkEngine {
    topology: NetworkTopology,
    active_connections: HashMap<Uuid, (Ipv4Addr, Ipv4Addr)>,
    ddos_attacks: HashMap<Ipv4Addr, Vec<Ipv4Addr>>,
    last_update: SystemTime,
}

impl NetworkEngine {
    pub fn new() -> Self {
        Self {
            topology: NetworkTopology::new(),
            active_connections: HashMap::new(),
            ddos_attacks: HashMap::new(),
            last_update: SystemTime::now(),
        }
    }

    pub fn connect(&mut self, user_id: Uuid, from: Ipv4Addr, to: Ipv4Addr) -> EngineResult<()> {
        // Check if target exists and is online
        let target = self.topology.get_node(to)
            .ok_or_else(|| EngineError::NotFound("Target IP not found".into()))?;

        if !target.is_online {
            return Err(EngineError::InvalidOperation("Target is offline".into()));
        }

        // Log the connection
        if let Some(target) = self.topology.get_node_mut(to) {
            target.add_log(NetworkLog::new(
                from,
                LogAction::Login,
                format!("Connection from {}", from),
            ));
        }

        self.active_connections.insert(user_id, (from, to));
        Ok(())
    }

    pub fn disconnect(&mut self, user_id: Uuid) -> EngineResult<()> {
        match self.active_connections.remove(&user_id) {
            Some((from, to)) => {
                if let Some(target) = self.topology.get_node_mut(to) {
                    target.add_log(NetworkLog::new(
                        from,
                        LogAction::Logout,
                        format!("Disconnection from {}", from),
                    ));
                }
                Ok(())
            }
            None => Err(EngineError::NotFound("No active connection".into())),
        }
    }

    pub fn start_ddos(&mut self, sources: Vec<Ipv4Addr>, target: Ipv4Addr) -> EngineResult<()> {
        let target_node = self.topology.get_node(target)
            .ok_or_else(|| EngineError::NotFound("Target IP not found".into()))?;

        if !target_node.is_online {
            return Err(EngineError::InvalidOperation("Target is already offline".into()));
        }

        self.ddos_attacks.insert(target, sources.clone());

        // Log the attack
        if let Some(target) = self.topology.get_node_mut(target) {
            for source in sources {
                target.add_log(NetworkLog::new(
                    source,
                    LogAction::DDoS,
                    "DDoS attack detected".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub fn stop_ddos(&mut self, target: Ipv4Addr) -> EngineResult<()> {
        self.ddos_attacks.remove(&target)
            .ok_or_else(|| EngineError::NotFound("No active DDoS on target".into()))?;

        if let Some(node) = self.topology.get_node_mut(target) {
            node.is_online = true;
        }

        Ok(())
    }

    pub fn scan(&mut self, from: Ipv4Addr, target: Ipv4Addr) -> EngineResult<Vec<u16>> {
        let target_node = self.topology.get_node(target)
            .ok_or_else(|| EngineError::NotFound("Target IP not found".into()))?;

        if !target_node.is_online {
            return Err(EngineError::InvalidOperation("Target is offline".into()));
        }

        // Log the scan
        if let Some(target) = self.topology.get_node_mut(target) {
            target.add_log(NetworkLog::new(
                from,
                LogAction::Scan,
                "Port scan detected".to_string(),
            ));
        }

        // Return open ports based on server type
        let ports = match target_node.server_type {
            ServerType::Personal => vec![22, 80],
            ServerType::Bank => vec![22, 80, 443, 1521],
            ServerType::Corporation => vec![22, 80, 443, 3306],
            ServerType::FBI => vec![22, 443],
            _ => vec![22, 80, 443],
        };

        Ok(ports)
    }

    pub fn get_topology(&self) -> &NetworkTopology {
        &self.topology
    }

    pub fn get_topology_mut(&mut self) -> &mut NetworkTopology {
        &mut self.topology
    }

    pub fn create_player_server(&mut self, owner_id: Uuid) -> Ipv4Addr {
        let ip = self.topology.get_random_ip();
        self.topology.create_node(
            ip,
            ServerType::Personal,
            owner_id,
            format!("player-{}", owner_id),
        );
        ip
    }
}

impl EngineComponent for NetworkEngine {
    fn initialize(&mut self) -> EngineResult<()> {
        Ok(())
    }

    fn update(&mut self, delta: Duration) -> EngineResult<()> {
        // Update DDoS attacks
        let mut offline_targets = Vec::new();
        for (target, sources) in &self.ddos_attacks {
            if sources.len() >= 5 {
                offline_targets.push(*target);
            }
        }

        for target in offline_targets {
            if let Some(node) = self.topology.get_node_mut(target) {
                node.is_online = false;
            }
        }

        self.last_update = SystemTime::now();
        Ok(())
    }

    fn status(&self) -> ComponentStatus {
        ComponentStatus {
            name: "NetworkEngine".to_string(),
            healthy: true,
            last_update: self.last_update,
            metrics: vec![
                ("nodes".to_string(), self.topology.nodes.len() as f64),
                ("active_connections".to_string(), self.active_connections.len() as f64),
                ("ddos_attacks".to_string(), self.ddos_attacks.len() as f64),
            ],
        }
    }

    fn reset(&mut self) -> EngineResult<()> {
        self.topology = NetworkTopology::new();
        self.active_connections.clear();
        self.ddos_attacks.clear();
        self.last_update = SystemTime::now();
        Ok(())
    }
}