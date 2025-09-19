//! Extended data structures for comprehensive game state

use crate::{PlayerState, TargetInfo, HardwareSpecs};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Extended player state with all fields needed by game mechanics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedPlayerState {
    pub player_id: i32,
    pub user_id: i32,
    pub level: i32,
    pub experience: i64,
    pub total_exp: i64,
    pub money: i32,
    pub bitcoin_amount: f32,
    pub reputation: HashMap<String, i32>,

    // Hardware specs
    pub cpu_mhz: i32,
    pub ram_mb: i32,
    pub hdd_mb: i32,
    pub internet_speed: i32,  // Mbps
    pub gpu_cores: Option<i32>,

    // Skills
    pub hacking_skill: Option<i32>,
    pub crypto_skill: Option<i32>,
    pub antivirus_skill: Option<i32>,
    pub stealth_skill: Option<i32>,
    pub research_skill: Option<i32>,
    pub network_skill: Option<i32>,

    // Software versions
    pub software_levels: HashMap<String, f32>,

    // Stats
    pub downloaded_files: Vec<i32>,
    pub cracked_systems: std::collections::HashSet<i32>,
    pub ddos_attacks_performed: i32,

    pub last_updated: DateTime<Utc>,
}

impl ExtendedPlayerState {
    /// Create from basic PlayerState
    pub fn from_player_state(player: &PlayerState) -> Self {
        Self {
            player_id: player.user_id,
            user_id: player.user_id,
            level: player.level,
            experience: player.experience,
            total_exp: player.experience,
            money: player.money as i32,
            bitcoin_amount: 0.0,
            reputation: player.reputation.clone(),

            cpu_mhz: player.hardware_specs.cpu,
            ram_mb: player.hardware_specs.ram,
            hdd_mb: player.hardware_specs.hdd,
            internet_speed: player.hardware_specs.net,
            gpu_cores: Some(1),

            hacking_skill: Some(50),
            crypto_skill: Some(30),
            antivirus_skill: Some(20),
            stealth_skill: Some(25),
            research_skill: Some(35),
            network_skill: Some(40),

            software_levels: HashMap::new(),
            downloaded_files: Vec::new(),
            cracked_systems: std::collections::HashSet::new(),
            ddos_attacks_performed: 0,

            last_updated: player.last_updated,
        }
    }
}

/// Extended target info with all fields needed for calculations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtendedTargetInfo {
    pub target_id: Option<i32>,
    pub target_ip: Option<String>,
    pub ip_address: String,
    pub target_type: String,
    pub difficulty_level: i32,
    pub security_rating: i32,
    pub security_level: Option<i32>,
    pub reward_money: i64,

    // File operations
    pub file_size: Option<i32>,
    pub software_size: Option<i32>,

    // Network
    pub internet_speed: Option<i32>,

    // Security
    pub encryption_level: Option<i32>,
    pub password_strength: Option<i32>,
    pub ddos_protection: Option<i32>,
    pub hijack_protection: Option<i32>,

    // Storage
    pub hdd_size: Option<i32>,

    // Research
    pub research_target: Option<f32>,

    // Financial
    pub transfer_amount: Option<i32>,

    // Mining
    pub mining_difficulty: Option<i32>,
}

impl ExtendedTargetInfo {
    /// Create from basic TargetInfo with defaults
    pub fn from_target_info(target: &TargetInfo) -> Self {
        Self {
            target_id: Some(1),
            target_ip: Some(target.ip_address.clone()),
            ip_address: target.ip_address.clone(),
            target_type: target.target_type.clone(),
            difficulty_level: target.difficulty_level,
            security_rating: target.security_rating,
            security_level: Some(target.security_rating),
            reward_money: target.reward_money,

            file_size: Some(1024 * 1024), // 1MB default
            software_size: Some(100),
            internet_speed: Some(100),
            encryption_level: Some(128),
            password_strength: Some(100),
            ddos_protection: Some(100),
            hijack_protection: Some(80),
            hdd_size: Some(10 * 1024), // 10GB default
            research_target: Some(2.0),
            transfer_amount: Some(1000),
            mining_difficulty: Some(1000000),
        }
    }
}

/// Convert basic PlayerState to extended for process calculations
pub fn extend_player_state(player: &PlayerState) -> ExtendedPlayerState {
    ExtendedPlayerState::from_player_state(player)
}

/// Convert basic TargetInfo to extended for process calculations
pub fn extend_target_info(target: &TargetInfo) -> ExtendedTargetInfo {
    ExtendedTargetInfo::from_target_info(target)
}