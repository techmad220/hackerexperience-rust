//! Software catalog - all available software in the game

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Complete software catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareCatalog {
    pub crackers: Vec<Software>,
    pub exploits: Vec<Software>,
    pub viruses: Vec<Software>,
    pub firewalls: Vec<Software>,
    pub antivirus: Vec<Software>,
    pub encryptors: Vec<Software>,
    pub decryptors: Vec<Software>,
    pub log_deleters: Vec<Software>,
    pub ddos_tools: Vec<Software>,
    pub analyzers: Vec<Software>,
    pub collectors: Vec<Software>,
    pub spam_tools: Vec<Software>,
}

/// Individual software item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Software {
    pub id: Uuid,
    pub name: String,
    pub version: f32,
    pub software_type: SoftwareType,
    pub description: String,
    pub size: i64, // MB
    pub ram_usage: i32, // MB
    pub cpu_usage: i32, // %
    pub effectiveness: i32, // 0-100
    pub price: i64,
    pub level_required: i32,
    pub research_time: i32, // seconds
    pub install_time: i32, // seconds
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoftwareType {
    Cracker,
    Exploit,
    Virus,
    Firewall,
    AntiVirus,
    Encryptor,
    Decryptor,
    LogDeleter,
    DDoS,
    Analyzer,
    Collector,
    SpamTool,
    Miner,
    Spyware,
}

impl Default for SoftwareCatalog {
    fn default() -> Self {
        let mut catalog = Self {
            crackers: Vec::new(),
            exploits: Vec::new(),
            viruses: Vec::new(),
            firewalls: Vec::new(),
            antivirus: Vec::new(),
            encryptors: Vec::new(),
            decryptors: Vec::new(),
            log_deleters: Vec::new(),
            ddos_tools: Vec::new(),
            analyzers: Vec::new(),
            collectors: Vec::new(),
            spam_tools: Vec::new(),
        };

        catalog.generate_crackers();
        catalog.generate_exploits();
        catalog.generate_viruses();
        catalog.generate_firewalls();
        catalog.generate_antivirus();
        catalog.generate_utilities();
        catalog.generate_attack_tools();

        catalog
    }
}

impl SoftwareCatalog {
    fn generate_crackers(&mut self) {
        let crackers = vec![
            ("Basic Cracker", 1.0, 10, 100, 1),
            ("Password Cracker", 2.0, 20, 500, 3),
            ("Advanced Cracker", 3.0, 35, 2000, 5),
            ("Elite Cracker", 4.0, 50, 10000, 10),
            ("Quantum Cracker", 5.0, 70, 50000, 15),
            ("Neural Cracker", 6.0, 85, 100000, 20),
            ("Ultimate Cracker", 10.0, 95, 1000000, 30),
        ];

        for (name, version, effectiveness, price, level) in crackers {
            self.crackers.push(Software {
                id: Uuid::new_v4(),
                name: name.to_string(),
                version,
                software_type: SoftwareType::Cracker,
                description: format!("Cracks passwords with {}% success rate", effectiveness),
                size: (version * 10.0) as i64,
                ram_usage: (version * 128.0) as i32,
                cpu_usage: effectiveness / 2,
                effectiveness,
                price,
                level_required: level,
                research_time: level * 3600, // hours to seconds
                install_time: 60 * level,
            });
        }
    }

    fn generate_exploits(&mut self) {
        let exploits = vec![
            ("Port Scanner", 1.0, 15, 200, 1),
            ("SQL Injection", 2.0, 30, 1000, 4),
            ("Buffer Overflow", 3.0, 45, 5000, 7),
            ("Zero-Day Exploit", 4.0, 60, 20000, 12),
            ("Kernel Exploit", 5.0, 75, 100000, 18),
            ("Hypervisor Escape", 10.0, 90, 500000, 25),
        ];

        for (name, version, effectiveness, price, level) in exploits {
            self.exploits.push(Software {
                id: Uuid::new_v4(),
                name: name.to_string(),
                version,
                software_type: SoftwareType::Exploit,
                description: format!("Exploits system vulnerabilities ({}% success)", effectiveness),
                size: (version * 15.0) as i64,
                ram_usage: (version * 256.0) as i32,
                cpu_usage: effectiveness / 3,
                effectiveness,
                price,
                level_required: level,
                research_time: level * 7200,
                install_time: 120 * level,
            });
        }
    }

    fn generate_viruses(&mut self) {
        let viruses = vec![
            ("Basic Virus", 1.0, 10, 500, 2),
            ("Worm", 2.0, 25, 2500, 5),
            ("Trojan Horse", 3.0, 40, 10000, 8),
            ("Ransomware", 4.0, 55, 50000, 12),
            ("Rootkit", 5.0, 70, 200000, 16),
            ("Botnet Client", 6.0, 80, 500000, 20),
            ("Polymorphic Virus", 10.0, 95, 2000000, 30),
        ];

        for (name, version, effectiveness, price, level) in viruses {
            self.viruses.push(Software {
                id: Uuid::new_v4(),
                name: name.to_string(),
                version,
                software_type: SoftwareType::Virus,
                description: format!("Malicious software ({}% stealth)", effectiveness),
                size: (version * 5.0) as i64,
                ram_usage: (version * 64.0) as i32,
                cpu_usage: 10,
                effectiveness,
                price,
                level_required: level,
                research_time: level * 5400,
                install_time: 30,
            });
        }
    }

    fn generate_firewalls(&mut self) {
        let firewalls = vec![
            ("Basic Firewall", 1.0, 20, 300, 1),
            ("Advanced Firewall", 2.0, 35, 1500, 3),
            ("Corporate Firewall", 3.0, 50, 7500, 6),
            ("Military Firewall", 4.0, 65, 30000, 10),
            ("Quantum Firewall", 5.0, 80, 150000, 15),
            ("AI Firewall", 10.0, 95, 1000000, 25),
        ];

        for (name, version, effectiveness, price, level) in firewalls {
            self.firewalls.push(Software {
                id: Uuid::new_v4(),
                name: name.to_string(),
                version,
                software_type: SoftwareType::Firewall,
                description: format!("Blocks {}% of attacks", effectiveness),
                size: (version * 20.0) as i64,
                ram_usage: (version * 512.0) as i32,
                cpu_usage: effectiveness / 4,
                effectiveness,
                price,
                level_required: level,
                research_time: level * 3600,
                install_time: 180 * level,
            });
        }
    }

    fn generate_antivirus(&mut self) {
        let antiviruses = vec![
            ("Basic AntiVirus", 1.0, 25, 400, 2),
            ("Advanced AntiVirus", 2.0, 40, 2000, 4),
            ("Premium AntiVirus", 3.0, 55, 10000, 7),
            ("Enterprise AntiVirus", 4.0, 70, 50000, 11),
            ("Military AntiVirus", 5.0, 85, 250000, 16),
            ("Quantum Shield", 10.0, 98, 2000000, 28),
        ];

        for (name, version, effectiveness, price, level) in antiviruses {
            self.antivirus.push(Software {
                id: Uuid::new_v4(),
                name: name.to_string(),
                version,
                software_type: SoftwareType::AntiVirus,
                description: format!("Detects {}% of viruses", effectiveness),
                size: (version * 25.0) as i64,
                ram_usage: (version * 384.0) as i32,
                cpu_usage: 15,
                effectiveness,
                price,
                level_required: level,
                research_time: level * 4500,
                install_time: 240 * level,
            });
        }
    }

    fn generate_utilities(&mut self) {
        // Log Deleters
        let log_deleters = vec![
            ("Log Cleaner", 1.0, 30, 200, 1),
            ("Trace Remover", 2.0, 50, 1000, 3),
            ("Ghost Mode", 3.0, 70, 5000, 6),
            ("Phantom Delete", 5.0, 90, 25000, 10),
            ("Quantum Eraser", 10.0, 99, 100000, 15),
        ];

        for (name, version, effectiveness, price, level) in log_deleters {
            self.log_deleters.push(Software {
                id: Uuid::new_v4(),
                name: name.to_string(),
                version,
                software_type: SoftwareType::LogDeleter,
                description: format!("Removes logs with {}% efficiency", effectiveness),
                size: (version * 3.0) as i64,
                ram_usage: (version * 32.0) as i32,
                cpu_usage: 5,
                effectiveness,
                price,
                level_required: level,
                research_time: level * 1800,
                install_time: 30,
            });
        }

        // Encryptors
        let encryptors = vec![
            ("Basic Encryptor", 1.0, 40, 500, 2),
            ("AES Encryptor", 3.0, 60, 5000, 5),
            ("Military Cipher", 5.0, 80, 50000, 10),
            ("Quantum Encryption", 10.0, 95, 500000, 20),
        ];

        for (name, version, effectiveness, price, level) in encryptors {
            self.encryptors.push(Software {
                id: Uuid::new_v4(),
                name: name.to_string(),
                version,
                software_type: SoftwareType::Encryptor,
                description: format!("{}% encryption strength", effectiveness),
                size: (version * 8.0) as i64,
                ram_usage: (version * 128.0) as i32,
                cpu_usage: effectiveness / 5,
                effectiveness,
                price,
                level_required: level,
                research_time: level * 2700,
                install_time: 60,
            });
        }

        // Decryptors (matching encryptors)
        for encryptor in &self.encryptors {
            self.decryptors.push(Software {
                id: Uuid::new_v4(),
                name: encryptor.name.replace("Encryptor", "Decryptor").replace("Encryption", "Decryption"),
                version: encryptor.version,
                software_type: SoftwareType::Decryptor,
                description: format!("Breaks {}% of encryptions", encryptor.effectiveness),
                size: encryptor.size * 2,
                ram_usage: encryptor.ram_usage * 2,
                cpu_usage: encryptor.cpu_usage * 3,
                effectiveness: encryptor.effectiveness - 10, // Slightly less effective
                price: encryptor.price * 3, // More expensive
                level_required: encryptor.level_required + 2,
                research_time: encryptor.research_time * 2,
                install_time: encryptor.install_time,
            });
        }
    }

    fn generate_attack_tools(&mut self) {
        // DDoS Tools
        let ddos_tools = vec![
            ("Ping Flood", 1.0, 20, 1000, 3),
            ("SYN Flood", 2.0, 40, 5000, 6),
            ("Botnet Controller", 3.0, 60, 25000, 10),
            ("Amplification Attack", 5.0, 80, 100000, 15),
            ("Quantum DDoS", 10.0, 95, 1000000, 25),
        ];

        for (name, version, effectiveness, price, level) in ddos_tools {
            self.ddos_tools.push(Software {
                id: Uuid::new_v4(),
                name: name.to_string(),
                version,
                software_type: SoftwareType::DDoS,
                description: format!("DDoS attack tool ({}% power)", effectiveness),
                size: (version * 12.0) as i64,
                ram_usage: (version * 1024.0) as i32,
                cpu_usage: 80,
                effectiveness,
                price,
                level_required: level,
                research_time: level * 5400,
                install_time: 120,
            });
        }

        // Analyzers
        let analyzers = vec![
            ("Network Scanner", 1.0, 50, 300, 1),
            ("Port Analyzer", 2.0, 65, 1500, 3),
            ("Vulnerability Scanner", 3.0, 80, 10000, 7),
            ("Deep Inspector", 5.0, 90, 50000, 12),
            ("Quantum Analyzer", 10.0, 99, 500000, 20),
        ];

        for (name, version, effectiveness, price, level) in analyzers {
            self.analyzers.push(Software {
                id: Uuid::new_v4(),
                name: name.to_string(),
                version,
                software_type: SoftwareType::Analyzer,
                description: format!("Analyzes targets ({}% accuracy)", effectiveness),
                size: (version * 7.0) as i64,
                ram_usage: (version * 256.0) as i32,
                cpu_usage: 30,
                effectiveness,
                price,
                level_required: level,
                research_time: level * 2400,
                install_time: 90,
            });
        }

        // Collectors (for stealing data/money)
        let collectors = vec![
            ("Data Miner", 1.0, 30, 2000, 4),
            ("Bitcoin Miner", 2.0, 45, 10000, 8),
            ("Bank Collector", 3.0, 60, 50000, 12),
            ("Crypto Harvester", 5.0, 75, 250000, 18),
            ("Quantum Collector", 10.0, 90, 2000000, 30),
        ];

        for (name, version, effectiveness, price, level) in collectors {
            self.collectors.push(Software {
                id: Uuid::new_v4(),
                name: name.to_string(),
                version,
                software_type: SoftwareType::Collector,
                description: format!("Collects resources ({}% efficiency)", effectiveness),
                size: (version * 10.0) as i64,
                ram_usage: (version * 512.0) as i32,
                cpu_usage: 50,
                effectiveness,
                price,
                level_required: level,
                research_time: level * 7200,
                install_time: 300,
            });
        }

        // Spam Tools
        let spam_tools = vec![
            ("Email Spammer", 1.0, 40, 500, 2),
            ("SMS Bomber", 2.0, 55, 2500, 5),
            ("Social Spammer", 3.0, 70, 15000, 9),
            ("Spam Network", 5.0, 85, 100000, 15),
        ];

        for (name, version, effectiveness, price, level) in spam_tools {
            self.spam_tools.push(Software {
                id: Uuid::new_v4(),
                name: name.to_string(),
                version,
                software_type: SoftwareType::SpamTool,
                description: format!("Spam effectiveness: {}%", effectiveness),
                size: (version * 4.0) as i64,
                ram_usage: (version * 64.0) as i32,
                cpu_usage: 20,
                effectiveness,
                price,
                level_required: level,
                research_time: level * 1800,
                install_time: 45,
            });
        }
    }

    /// Get software by type and version
    pub fn get_software(&self, software_type: SoftwareType, min_version: f32) -> Vec<&Software> {
        let collection = match software_type {
            SoftwareType::Cracker => &self.crackers,
            SoftwareType::Exploit => &self.exploits,
            SoftwareType::Virus => &self.viruses,
            SoftwareType::Firewall => &self.firewalls,
            SoftwareType::AntiVirus => &self.antivirus,
            SoftwareType::Encryptor => &self.encryptors,
            SoftwareType::Decryptor => &self.decryptors,
            SoftwareType::LogDeleter => &self.log_deleters,
            SoftwareType::DDoS => &self.ddos_tools,
            SoftwareType::Analyzer => &self.analyzers,
            SoftwareType::Collector => &self.collectors,
            SoftwareType::SpamTool => &self.spam_tools,
            _ => return Vec::new(),
        };

        collection.iter()
            .filter(|s| s.version >= min_version)
            .collect()
    }

    /// Get all software available for a player level
    pub fn get_available_for_level(&self, level: i32) -> Vec<&Software> {
        let mut available = Vec::new();

        for software_list in [
            &self.crackers,
            &self.exploits,
            &self.viruses,
            &self.firewalls,
            &self.antivirus,
            &self.encryptors,
            &self.decryptors,
            &self.log_deleters,
            &self.ddos_tools,
            &self.analyzers,
            &self.collectors,
            &self.spam_tools,
        ] {
            available.extend(
                software_list.iter()
                    .filter(|s| s.level_required <= level)
            );
        }

        available
    }
}