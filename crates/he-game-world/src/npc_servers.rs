//! NPC Server definitions and generation

use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Types of NPC servers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerType {
    HomePC,
    SmallBusiness,
    School,
    Company,
    Bank,
    Government,
    Military,
    Datacenter,
    CryptoExchange,
    Whois,
    DNS,
    ISP,
    Mystery,
}

/// NPC Server that can be hacked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPCServer {
    pub id: Uuid,
    pub ip_address: String,
    pub hostname: String,
    pub owner_name: String,
    pub server_type: ServerType,
    pub tier: i32, // 1-5 difficulty
    pub security_level: i32, // 0-100
    pub firewall_level: i32, // 0-10
    pub has_encryption: bool,
    pub money_available: i64,
    pub files: Vec<ServerFile>,
    pub logs: Vec<LogEntry>,
    pub running_software: Vec<RunningSoftware>,
    pub hardware: ServerHardware,
    pub is_online: bool,
    pub last_reset: DateTime<Utc>,
}

/// File on server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerFile {
    pub id: Uuid,
    pub name: String,
    pub file_type: FileType,
    pub size: i64, // bytes
    pub content: Option<String>,
    pub is_encrypted: bool,
    pub is_hidden: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    Text,
    Software,
    Virus,
    Data,
    Password,
    Log,
    Config,
    Database,
}

/// Log entry on server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub ip_address: String,
    pub is_hidden: bool,
}

/// Software running on server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningSoftware {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub software_type: String,
    pub ram_usage: i32,
    pub effectiveness: i32,
}

/// Server hardware specs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHardware {
    pub cpu: i32,  // MHz
    pub ram: i32,  // MB
    pub hdd: i32,  // MB
    pub network: i32, // Mbps
}

impl NPCServer {
    /// Generate a tier 1 server (easy)
    pub fn generate_tier1(index: i32) -> Self {
        let mut rng = rand::thread_rng();

        let server_types = [ServerType::HomePC, ServerType::SmallBusiness, ServerType::School];
        let server_type = server_types[rng.gen_range(0..server_types.len())];

        let (hostname, owner) = match server_type {
            ServerType::HomePC => {
                let names = ["john", "mary", "bob", "alice", "dave", "susan"];
                let name = names[rng.gen_range(0..names.len())];
                (format!("{}-pc.home", name), format!("{}'s Computer", name.to_uppercase()))
            },
            ServerType::SmallBusiness => {
                let businesses = ["cafe", "shop", "store", "restaurant", "salon"];
                let biz = businesses[rng.gen_range(0..businesses.len())];
                (format!("{}.local", biz), format!("Local {}", biz.to_uppercase()))
            },
            ServerType::School => {
                (format!("school{}.edu", index), format!("School #{}", index))
            },
            _ => unreachable!(),
        };

        let mut files = vec![
            ServerFile {
                id: Uuid::new_v4(),
                name: "passwords.txt".to_string(),
                file_type: FileType::Password,
                size: rng.gen_range(100..1000),
                content: Some(format!("admin:{}\nuser:{}",
                    generate_weak_password(),
                    generate_weak_password())),
                is_encrypted: false,
                is_hidden: rng.gen_bool(0.3),
            }
        ];

        // Add some random files
        for i in 0..rng.gen_range(2..5) {
            files.push(generate_random_file(i));
        }

        Self {
            id: Uuid::new_v4(),
            ip_address: generate_ip_address(10, index), // 10.x.x.x range
            hostname,
            owner_name: owner,
            server_type,
            tier: 1,
            security_level: rng.gen_range(5..20),
            firewall_level: rng.gen_range(0..2),
            has_encryption: false,
            money_available: rng.gen_range(50..500),
            files,
            logs: generate_fake_logs(rng.gen_range(3..10)),
            running_software: vec![],
            hardware: ServerHardware {
                cpu: rng.gen_range(500..1500),
                ram: rng.gen_range(512..2048),
                hdd: rng.gen_range(10000..50000),
                network: rng.gen_range(10..50),
            },
            is_online: true,
            last_reset: Utc::now(),
        }
    }

    /// Generate a tier 2 server (medium)
    pub fn generate_tier2(index: i32) -> Self {
        let mut rng = rand::thread_rng();

        let server_types = [ServerType::Company, ServerType::School];
        let server_type = server_types[rng.gen_range(0..server_types.len())];

        let (hostname, owner) = match server_type {
            ServerType::Company => {
                let companies = ["techcorp", "datasoft", "webco", "netinc", "compuware"];
                let company = companies[rng.gen_range(0..companies.len())];
                (format!("{}.com", company), format!("{} Inc.", company.to_uppercase()))
            },
            ServerType::School => {
                (format!("university{}.edu", index), format!("University #{}", index))
            },
            _ => unreachable!(),
        };

        let mut files = vec![
            ServerFile {
                id: Uuid::new_v4(),
                name: "database.db".to_string(),
                file_type: FileType::Database,
                size: rng.gen_range(10000..100000),
                content: Some("Customer database with valuable information".to_string()),
                is_encrypted: rng.gen_bool(0.5),
                is_hidden: false,
            },
            ServerFile {
                id: Uuid::new_v4(),
                name: "firewall.cfg".to_string(),
                file_type: FileType::Config,
                size: rng.gen_range(1000..5000),
                content: Some("Firewall configuration v2.0".to_string()),
                is_encrypted: false,
                is_hidden: true,
            },
        ];

        // Add more files
        for i in 0..rng.gen_range(5..10) {
            files.push(generate_random_file(i));
        }

        Self {
            id: Uuid::new_v4(),
            ip_address: generate_ip_address(172, index), // 172.x.x.x range
            hostname,
            owner_name: owner,
            server_type,
            tier: 2,
            security_level: rng.gen_range(20..40),
            firewall_level: rng.gen_range(2..5),
            has_encryption: rng.gen_bool(0.5),
            money_available: rng.gen_range(500..5000),
            files,
            logs: generate_fake_logs(rng.gen_range(10..20)),
            running_software: vec![
                RunningSoftware {
                    id: Uuid::new_v4(),
                    name: "Firewall".to_string(),
                    version: "2.0".to_string(),
                    software_type: "Defense".to_string(),
                    ram_usage: 256,
                    effectiveness: 50,
                },
            ],
            hardware: ServerHardware {
                cpu: rng.gen_range(2000..4000),
                ram: rng.gen_range(4096..8192),
                hdd: rng.gen_range(100000..500000),
                network: rng.gen_range(100..500),
            },
            is_online: true,
            last_reset: Utc::now(),
        }
    }

    /// Generate a tier 3 server (hard)
    pub fn generate_tier3(index: i32) -> Self {
        let mut rng = rand::thread_rng();

        let server_types = [ServerType::Bank, ServerType::Datacenter, ServerType::CryptoExchange];
        let server_type = server_types[rng.gen_range(0..server_types.len())];

        let (hostname, owner) = match server_type {
            ServerType::Bank => {
                let banks = ["firstbank", "megabank", "cryptobank", "globalbank"];
                let bank = banks[rng.gen_range(0..banks.len())];
                (format!("{}.bank", bank), format!("{} International", bank.to_uppercase()))
            },
            ServerType::Datacenter => {
                (format!("dc{}.cloud", index), format!("Datacenter #{}", index))
            },
            ServerType::CryptoExchange => {
                let exchanges = ["bitex", "coinbase", "cryptex", "blockex"];
                let exchange = exchanges[rng.gen_range(0..exchanges.len())];
                (format!("{}.exchange", exchange), format!("{} Exchange", exchange.to_uppercase()))
            },
            _ => unreachable!(),
        };

        let mut files = vec![
            ServerFile {
                id: Uuid::new_v4(),
                name: "accounts.db".to_string(),
                file_type: FileType::Database,
                size: rng.gen_range(100000..1000000),
                content: Some("Bank account database - CONFIDENTIAL".to_string()),
                is_encrypted: true,
                is_hidden: true,
            },
            ServerFile {
                id: Uuid::new_v4(),
                name: "transactions.log".to_string(),
                file_type: FileType::Log,
                size: rng.gen_range(50000..200000),
                content: Some("Transaction history".to_string()),
                is_encrypted: true,
                is_hidden: false,
            },
        ];

        // Add valuable software
        files.push(ServerFile {
            id: Uuid::new_v4(),
            name: "cracker_v5.exe".to_string(),
            file_type: FileType::Software,
            size: 50000,
            content: None,
            is_encrypted: false,
            is_hidden: true,
        });

        for i in 0..rng.gen_range(10..15) {
            files.push(generate_random_file(i));
        }

        Self {
            id: Uuid::new_v4(),
            ip_address: generate_ip_address(192, index),
            hostname,
            owner_name: owner,
            server_type,
            tier: 3,
            security_level: rng.gen_range(40..70),
            firewall_level: rng.gen_range(5..8),
            has_encryption: true,
            money_available: rng.gen_range(5000..50000),
            files,
            logs: generate_fake_logs(rng.gen_range(20..50)),
            running_software: vec![
                RunningSoftware {
                    id: Uuid::new_v4(),
                    name: "Firewall".to_string(),
                    version: "5.0".to_string(),
                    software_type: "Defense".to_string(),
                    ram_usage: 512,
                    effectiveness: 75,
                },
                RunningSoftware {
                    id: Uuid::new_v4(),
                    name: "AntiVirus".to_string(),
                    version: "3.0".to_string(),
                    software_type: "Defense".to_string(),
                    ram_usage: 256,
                    effectiveness: 60,
                },
                RunningSoftware {
                    id: Uuid::new_v4(),
                    name: "IDS".to_string(),
                    version: "2.0".to_string(),
                    software_type: "Detection".to_string(),
                    ram_usage: 384,
                    effectiveness: 70,
                },
            ],
            hardware: ServerHardware {
                cpu: rng.gen_range(5000..10000),
                ram: rng.gen_range(16384..32768),
                hdd: rng.gen_range(1000000..5000000),
                network: rng.gen_range(1000..5000),
            },
            is_online: true,
            last_reset: Utc::now(),
        }
    }

    /// Generate a tier 4 server (elite)
    pub fn generate_tier4(index: i32) -> Self {
        let mut rng = rand::thread_rng();

        let server_types = [ServerType::Government, ServerType::Military];
        let server_type = server_types[rng.gen_range(0..server_types.len())];

        let (hostname, owner) = match server_type {
            ServerType::Government => {
                let agencies = ["nsa", "cia", "fbi", "dhs"];
                let agency = agencies[rng.gen_range(0..agencies.len())];
                (format!("{}.gov", agency), format!("{}", agency.to_uppercase()))
            },
            ServerType::Military => {
                (format!("military{}.mil", index), format!("Military Base #{}", index))
            },
            _ => unreachable!(),
        };

        let mut files = vec![
            ServerFile {
                id: Uuid::new_v4(),
                name: "classified.txt".to_string(),
                file_type: FileType::Data,
                size: rng.gen_range(1000000..10000000),
                content: Some("CLASSIFIED - TOP SECRET".to_string()),
                is_encrypted: true,
                is_hidden: true,
            },
        ];

        // Add elite software
        files.push(ServerFile {
            id: Uuid::new_v4(),
            name: "quantum_crack.exe".to_string(),
            file_type: FileType::Software,
            size: 100000,
            content: None,
            is_encrypted: true,
            is_hidden: true,
        });

        Self {
            id: Uuid::new_v4(),
            ip_address: generate_ip_address(8, index), // 8.x.x.x (like real government IPs)
            hostname,
            owner_name: owner,
            server_type,
            tier: 4,
            security_level: rng.gen_range(70..95),
            firewall_level: rng.gen_range(8..10),
            has_encryption: true,
            money_available: rng.gen_range(50000..1000000),
            files,
            logs: generate_fake_logs(rng.gen_range(50..100)),
            running_software: vec![
                RunningSoftware {
                    id: Uuid::new_v4(),
                    name: "Military Firewall".to_string(),
                    version: "10.0".to_string(),
                    software_type: "Defense".to_string(),
                    ram_usage: 2048,
                    effectiveness: 95,
                },
                RunningSoftware {
                    id: Uuid::new_v4(),
                    name: "Quantum Shield".to_string(),
                    version: "1.0".to_string(),
                    software_type: "Defense".to_string(),
                    ram_usage: 4096,
                    effectiveness: 99,
                },
            ],
            hardware: ServerHardware {
                cpu: rng.gen_range(20000..50000),
                ram: rng.gen_range(65536..131072),
                hdd: rng.gen_range(10000000..100000000),
                network: rng.gen_range(10000..100000),
            },
            is_online: true,
            last_reset: Utc::now(),
        }
    }

    /// Simulate being hacked
    pub fn on_hacked(&mut self, hacker_ip: &str) {
        // Add log entry
        self.logs.push(LogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: "Login successful".to_string(),
            ip_address: hacker_ip.to_string(),
            is_hidden: false,
        });

        // Trigger any alarms if IDS is running
        if self.running_software.iter().any(|s| s.software_type == "Detection") {
            // Would trigger alert to admins
        }
    }

    /// Download file from server
    pub fn download_file(&mut self, file_name: &str, hacker_ip: &str) -> Option<ServerFile> {
        let file = self.files.iter().find(|f| f.name == file_name)?.clone();

        self.logs.push(LogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: format!("Downloaded {}", file_name),
            ip_address: hacker_ip.to_string(),
            is_hidden: false,
        });

        Some(file)
    }

    /// Delete file from server
    pub fn delete_file(&mut self, file_name: &str, hacker_ip: &str) -> bool {
        let initial_count = self.files.len();
        self.files.retain(|f| f.name != file_name);

        if self.files.len() < initial_count {
            self.logs.push(LogEntry {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                action: format!("Deleted {}", file_name),
                ip_address: hacker_ip.to_string(),
                is_hidden: false,
            });
            true
        } else {
            false
        }
    }

    /// Transfer money from server
    pub fn transfer_money(&mut self, amount: i64, hacker_ip: &str) -> bool {
        if self.money_available >= amount {
            self.money_available -= amount;

            self.logs.push(LogEntry {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                action: format!("Bank transfer: ${}", amount),
                ip_address: hacker_ip.to_string(),
                is_hidden: false,
            });

            true
        } else {
            false
        }
    }
}

// Helper functions

fn generate_ip_address(first_octet: u8, index: i32) -> String {
    let mut rng = rand::thread_rng();
    format!("{}.{}.{}.{}",
        first_octet,
        rng.gen_range(0..255),
        rng.gen_range(0..255),
        (index % 255) as u8
    )
}

fn generate_weak_password() -> String {
    let passwords = ["password", "123456", "admin", "letmein", "qwerty", "welcome", "monkey", "dragon"];
    passwords[rand::thread_rng().gen_range(0..passwords.len())].to_string()
}

fn generate_random_file(index: i32) -> ServerFile {
    let mut rng = rand::thread_rng();

    let file_types = [
        ("document.txt", FileType::Text, 1000, 10000),
        ("data.csv", FileType::Data, 5000, 50000),
        ("backup.zip", FileType::Data, 10000, 100000),
        ("config.ini", FileType::Config, 100, 1000),
        ("notes.txt", FileType::Text, 500, 5000),
    ];

    let (name, file_type, min_size, max_size) = file_types[rng.gen_range(0..file_types.len())];

    ServerFile {
        id: Uuid::new_v4(),
        name: format!("{}_{}", name, index),
        file_type,
        size: rng.gen_range(min_size..max_size),
        content: Some(format!("File content for {}", name)),
        is_encrypted: rng.gen_bool(0.2),
        is_hidden: rng.gen_bool(0.3),
    }
}

fn generate_fake_logs(count: usize) -> Vec<LogEntry> {
    let mut logs = Vec::new();
    let mut rng = rand::thread_rng();

    let actions = [
        "Login attempt",
        "File accessed",
        "System scan",
        "Password changed",
        "Backup created",
        "Software updated",
    ];

    for _ in 0..count {
        logs.push(LogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now() - chrono::Duration::hours(rng.gen_range(1..720)),
            action: actions[rng.gen_range(0..actions.len())].to_string(),
            ip_address: generate_ip_address(rng.gen_range(1..255), rng.gen_range(0..1000)),
            is_hidden: false,
        });
    }

    logs
}