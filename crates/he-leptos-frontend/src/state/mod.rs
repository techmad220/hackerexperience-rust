use leptos::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

// Temporary type definitions until he-game-mechanics is integrated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSpecs {
    pub cpu: u32,
    pub ram: u32,
    pub hdd: u32,
    pub net: u32,
    pub security_level: u32,
    pub performance_rating: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareInstance {
    pub software_type: String,
    pub version: String,
    pub effectiveness: u32,
    pub dependencies: Vec<String>,
    pub installation_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetInfo {
    pub ip_address: String,
    pub target_type: String,
    pub difficulty_level: u32,
    pub security_rating: u32,
    pub reward_money: u64,
    pub defense_systems: Vec<DefenseSystem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefenseSystem {
    pub system_type: String,
    pub strength: u32,
    pub detection_rate: Decimal,
    pub response_time: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub level: i32,
    pub experience: i64,
    pub money: i64,
    pub hardware: HardwareSpecs,
    pub software: Vec<SoftwareInstance>,
    pub connections: HashMap<i32, ConnectionInstance>,
    pub servers: HashMap<String, ServerInstance>,
    pub target_pcs: HashMap<String, TargetInfo>,
    pub bounce_route: Vec<String>,
    pub logs: Vec<LogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInstance {
    pub target_ip: String,
    pub connection_type: String,
    pub status: String,
    pub established: Option<DateTime<Utc>>,
    pub duration: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInstance {
    pub name: String,
    pub location: String,
    pub status: String,
    pub uptime: String,
    pub cost: i64,
    pub connection_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub log_type: String,
}

impl Default for GameState {
    fn default() -> Self {
        let mut target_pcs = HashMap::new();
        
        // Add some default target PCs (same as JavaScript version)
        target_pcs.insert("192.168.100.10".to_string(), TargetInfo {
            ip_address: "192.168.100.10".to_string(),
            target_type: "Personal PC".to_string(),
            difficulty_level: 2,
            security_rating: 2,
            reward_money: 5000,
            defense_systems: vec![
                DefenseSystem {
                    system_type: "Basic Firewall".to_string(),
                    strength: 30,
                    detection_rate: Decimal::from(25),
                    response_time: 30,
                }
            ],
        });

        target_pcs.insert("10.0.0.50".to_string(), TargetInfo {
            ip_address: "10.0.0.50".to_string(),
            target_type: "Corporate Server".to_string(),
            difficulty_level: 6,
            security_rating: 6,
            reward_money: 25000,
            defense_systems: vec![
                DefenseSystem {
                    system_type: "Advanced Firewall".to_string(),
                    strength: 75,
                    detection_rate: Decimal::from(60),
                    response_time: 15,
                },
                DefenseSystem {
                    system_type: "Intrusion Detection".to_string(),
                    strength: 60,
                    detection_rate: Decimal::from(80),
                    response_time: 10,
                }
            ],
        });

        Self {
            level: 1,
            experience: 0,
            money: 10000,
            hardware: HardwareSpecs {
                cpu: 1000,  // MHz
                ram: 512,   // MB  
                hdd: 10000, // MB
                net: 1,     // Mbps
                security_level: 1,
                performance_rating: 50,
            },
            software: vec![
                SoftwareInstance {
                    software_type: "Basic Terminal".to_string(),
                    version: "1.0".to_string(),
                    effectiveness: 50,
                    dependencies: vec![],
                    installation_date: Utc::now(),
                },
                SoftwareInstance {
                    software_type: "Text Editor".to_string(),
                    version: "1.0".to_string(),
                    effectiveness: 50,
                    dependencies: vec![],
                    installation_date: Utc::now(),
                },
            ],
            connections: HashMap::new(),
            servers: HashMap::new(),
            target_pcs,
            bounce_route: vec![],
            logs: vec![],
        }
    }
}

pub fn create_game_state() -> RwSignal<GameState> {
    create_rw_signal(GameState::default())
}