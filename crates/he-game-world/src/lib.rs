//! Game World - NPC servers, corporations, and hackable targets

use anyhow::Result;
use chrono::{DateTime, Utc};
use rand::{seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub mod npc_servers;
pub mod software_catalog;
pub mod missions;
pub mod world_generator;

pub use npc_servers::*;
pub use software_catalog::*;
pub use missions::*;
pub use world_generator::*;

/// Represents the entire game world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameWorld {
    pub servers: HashMap<String, NPCServer>,
    pub corporations: Vec<Corporation>,
    pub software_catalog: SoftwareCatalog,
    pub mission_templates: Vec<MissionTemplate>,
    pub network_topology: NetworkTopology,
    pub created_at: DateTime<Utc>,
}

impl GameWorld {
    /// Create a new game world with default content
    pub fn new() -> Self {
        let mut world = Self {
            servers: HashMap::new(),
            corporations: Vec::new(),
            software_catalog: SoftwareCatalog::default(),
            mission_templates: Vec::new(),
            network_topology: NetworkTopology::new(),
            created_at: Utc::now(),
        };

        // Generate initial world content
        world.generate_npc_servers();
        world.generate_corporations();
        world.generate_software();
        world.generate_missions();
        world.generate_network();

        world
    }

    /// Generate NPC servers for players to hack
    fn generate_npc_servers(&mut self) {
        // Generate different tiers of servers

        // Tier 1: Easy targets (home computers, small businesses)
        for i in 0..50 {
            let server = NPCServer::generate_tier1(i);
            self.servers.insert(server.ip_address.clone(), server);
        }

        // Tier 2: Medium targets (companies, schools)
        for i in 0..30 {
            let server = NPCServer::generate_tier2(i);
            self.servers.insert(server.ip_address.clone(), server);
        }

        // Tier 3: Hard targets (banks, corporations)
        for i in 0..15 {
            let server = NPCServer::generate_tier3(i);
            self.servers.insert(server.ip_address.clone(), server);
        }

        // Tier 4: Elite targets (government, military)
        for i in 0..5 {
            let server = NPCServer::generate_tier4(i);
            self.servers.insert(server.ip_address.clone(), server);
        }

        // Special servers
        self.add_special_servers();
    }

    /// Add special storyline servers
    fn add_special_servers(&mut self) {
        // First Whois server (tutorial)
        let whois = NPCServer {
            id: Uuid::new_v4(),
            ip_address: "1.2.3.4".to_string(),
            hostname: "whois.first.org".to_string(),
            owner_name: "First Whois Database".to_string(),
            server_type: ServerType::Whois,
            tier: 1,
            security_level: 10,
            firewall_level: 1,
            has_encryption: false,
            money_available: 0,
            files: vec![
                ServerFile {
                    id: Uuid::new_v4(),
                    name: "database.txt".to_string(),
                    file_type: FileType::Text,
                    size: 1024,
                    content: Some("Welcome to HackerExperience!\nThis is the First Whois server.\nTry: scan 1.2.3.4".to_string()),
                    is_encrypted: false,
                    is_hidden: false,
                },
            ],
            logs: vec![],
            running_software: vec![],
            hardware: ServerHardware {
                cpu: 500,
                ram: 512,
                hdd: 10000,
                network: 10,
            },
            is_online: true,
            last_reset: Utc::now(),
        };
        self.servers.insert(whois.ip_address.clone(), whois);

        // Mystery server (storyline)
        let mystery = NPCServer {
            id: Uuid::new_v4(),
            ip_address: "13.37.13.37".to_string(),
            hostname: "unknown.mystery".to_string(),
            owner_name: "???".to_string(),
            server_type: ServerType::Mystery,
            tier: 5,
            security_level: 100,
            firewall_level: 10,
            has_encryption: true,
            money_available: 1000000,
            files: vec![
                ServerFile {
                    id: Uuid::new_v4(),
                    name: "README.txt".to_string(),
                    file_type: FileType::Text,
                    size: 666,
                    content: Some("You shouldn't be here...".to_string()),
                    is_encrypted: true,
                    is_hidden: true,
                },
            ],
            logs: vec![],
            running_software: vec![],
            hardware: ServerHardware {
                cpu: 50000,
                ram: 65536,
                hdd: 10000000,
                network: 10000,
            },
            is_online: true,
            last_reset: Utc::now(),
        };
        self.servers.insert(mystery.ip_address.clone(), mystery);
    }

    /// Generate corporations
    fn generate_corporations(&mut self) {
        let corp_names = vec![
            ("MegaCorp", "Technology giant with deep pockets"),
            ("CyberBank", "International banking consortium"),
            ("DataMine Inc", "Big data analytics company"),
            ("SecureNet", "Cybersecurity firm"),
            ("CloudNine", "Cloud storage provider"),
            ("BitStream", "Cryptocurrency exchange"),
            ("NeuralLink", "AI research company"),
            ("QuantumCore", "Quantum computing startup"),
        ];

        for (name, description) in corp_names {
            let corp = Corporation {
                id: Uuid::new_v4(),
                name: name.to_string(),
                description: description.to_string(),
                reputation_required: rand::thread_rng().gen_range(100..5000),
                servers: Vec::new(), // Will be linked to actual servers
                bounties: Vec::new(),
            };
            self.corporations.push(corp);
        }
    }

    /// Generate software catalog
    fn generate_software(&mut self) {
        self.software_catalog = SoftwareCatalog::default();
    }

    /// Generate missions
    fn generate_missions(&mut self) {
        let missions = missions::generate_default_missions();
        self.mission_templates = missions;
    }

    /// Generate network topology
    fn generate_network(&mut self) {
        self.network_topology.generate_connections(&self.servers);
    }

    /// Get a random hackable server for player level
    pub fn get_random_target(&self, player_level: i32) -> Option<&NPCServer> {
        let appropriate_tier = match player_level {
            1..=5 => 1,
            6..=10 => 2,
            11..=20 => 3,
            21..=30 => 4,
            _ => 5,
        };

        let suitable_servers: Vec<&NPCServer> = self.servers
            .values()
            .filter(|s| s.tier <= appropriate_tier && s.is_online)
            .collect();

        suitable_servers.choose(&mut rand::thread_rng()).copied()
    }

    /// Get servers by type
    pub fn get_servers_by_type(&self, server_type: ServerType) -> Vec<&NPCServer> {
        self.servers
            .values()
            .filter(|s| s.server_type == server_type)
            .collect()
    }

    /// Check if IP exists
    pub fn server_exists(&self, ip: &str) -> bool {
        self.servers.contains_key(ip)
    }

    /// Get server by IP
    pub fn get_server(&self, ip: &str) -> Option<&NPCServer> {
        self.servers.get(ip)
    }

    /// Get mutable server reference
    pub fn get_server_mut(&mut self, ip: &str) -> Option<&mut NPCServer> {
        self.servers.get_mut(ip)
    }
}

/// Corporation entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Corporation {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub reputation_required: i32,
    pub servers: Vec<String>, // IP addresses
    pub bounties: Vec<Bounty>,
}

/// Bounty/contract from corporation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bounty {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub target_ip: String,
    pub reward_money: i64,
    pub reward_reputation: i32,
    pub time_limit: Option<i64>, // seconds
    pub requirements: BountyRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BountyRequirements {
    pub hack_server: bool,
    pub steal_file: Option<String>,
    pub install_virus: bool,
    pub delete_logs: bool,
    pub remain_undetected: bool,
}

/// Network topology for server connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopology {
    pub connections: HashMap<String, Vec<String>>, // IP -> connected IPs
    pub backbone_nodes: Vec<String>, // Major routing nodes
}

impl NetworkTopology {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            backbone_nodes: Vec::new(),
        }
    }

    pub fn generate_connections(&mut self, servers: &HashMap<String, NPCServer>) {
        // Create realistic network topology
        let mut rng = rand::thread_rng();

        // Designate some servers as backbone nodes
        let server_ips: Vec<String> = servers.keys().cloned().collect();
        let backbone_count = (server_ips.len() / 10).max(5);

        for _ in 0..backbone_count {
            if let Some(ip) = server_ips.choose(&mut rng) {
                self.backbone_nodes.push(ip.clone());
            }
        }

        // Connect servers based on tier and type
        for (ip, server) in servers {
            let mut connections = Vec::new();

            // Connect to 1-3 backbone nodes
            for backbone in self.backbone_nodes.iter().take(rng.gen_range(1..=3)) {
                if backbone != ip {
                    connections.push(backbone.clone());
                }
            }

            // Connect to 2-5 servers of similar tier
            let similar_servers: Vec<String> = servers
                .iter()
                .filter(|(other_ip, other_server)| {
                    *other_ip != ip &&
                    (other_server.tier - server.tier).abs() <= 1
                })
                .map(|(ip, _)| ip.clone())
                .collect();

            for _ in 0..rng.gen_range(2..=5) {
                if let Some(connected_ip) = similar_servers.choose(&mut rng) {
                    if !connections.contains(connected_ip) {
                        connections.push(connected_ip.clone());
                    }
                }
            }

            self.connections.insert(ip.clone(), connections);
        }
    }

    pub fn get_route(&self, from: &str, to: &str) -> Option<Vec<String>> {
        // Simple pathfinding between servers
        use std::collections::{VecDeque, HashSet};

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parents = HashMap::new();

        queue.push_back(from.to_string());
        visited.insert(from.to_string());

        while let Some(current) = queue.pop_front() {
            if current == to {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = to.to_string();

                while node != from {
                    path.push(node.clone());
                    node = parents.get(&node)?.clone();
                }
                path.push(from.to_string());
                path.reverse();

                return Some(path);
            }

            if let Some(neighbors) = self.connections.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        parents.insert(neighbor.clone(), current.clone());
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        None
    }
}

impl Default for GameWorld {
    fn default() -> Self {
        Self::new()
    }
}