//! # HackerExperience Game Mechanics Engine
//! 
//! This crate implements all game mechanics, algorithms, formulas, and business rules
//! from the original HackerExperience game with complete 1:1 parity.
//! 
//! ## Core Systems
//! 
//! - **Hacking System**: Difficulty calculations, success rates, intrusion mechanics
//! - **Defense System**: Firewall strength, security ratings, detection algorithms  
//! - **Experience System**: Level progression, skill development, learning curves
//! - **Financial System**: Economy balance, market dynamics, transaction processing
//! - **Process System**: Time calculations, resource management, scheduling
//! - **Hardware System**: Performance ratings, compatibility, upgrade mechanics
//! - **Software System**: Dependencies, effectiveness, installation mechanics
//! - **Network System**: Connection protocols, routing, bandwidth calculations
//! - **Mission System**: Difficulty scaling, reward calculations, prerequisites
//! - **Clan System**: Warfare mechanics, reputation formulas, contribution tracking

pub mod hacking;
pub mod defense;
pub mod experience;
pub mod financial;
pub mod process;
pub mod hardware;
pub mod software;
pub mod network;
pub mod missions;
pub mod missions_safe;  // Safe, original mission system - no AGPL content
pub mod clans;
pub mod config;

// The complete game engine!
pub mod engine;

// Re-export the main engine components
pub use engine::{
    GameEngine, GameState, GameAction, GameEvent,
    ProcessEngine, HardwareEngine, SoftwareEngine, NetworkEngine,
    EngineComponent, EngineError, EngineResult,
};
pub mod utils;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Core error types for game mechanics
#[derive(Error, Debug)]
pub enum GameMechanicsError {
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("Insufficient resources: {0}")]
    InsufficientResources(String),
    #[error("Precondition failed: {0}")]
    PreconditionFailed(String),
    #[error("Calculation error: {0}")]
    CalculationError(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

pub type Result<T> = std::result::Result<T, GameMechanicsError>;

/// Player statistics and current state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub user_id: i32,
    pub level: i32,
    pub experience: i64,
    pub money: i64,
    pub reputation: HashMap<String, i32>,
    pub hardware_specs: HardwareSpecs,
    pub software_installed: Vec<SoftwareInstance>,
    pub active_processes: Vec<ProcessInstance>,
    pub clan_membership: Option<ClanMembership>,
    pub last_updated: DateTime<Utc>,
}

/// Hardware specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSpecs {
    pub cpu: i32,      // MHz
    pub ram: i32,      // MB
    pub hdd: i32,      // MB
    pub net: i32,      // Mbps
    pub security_level: i32,
    pub performance_rating: i32,
}

/// Software instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareInstance {
    pub software_type: String,
    pub version: String,
    pub effectiveness: i32,
    pub dependencies: Vec<String>,
    pub installation_date: DateTime<Utc>,
}

/// Process instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInstance {
    pub process_id: i32,
    pub process_type: String,
    pub target_ip: String,
    pub progress: Decimal,
    pub time_started: DateTime<Utc>,
    pub time_end: DateTime<Utc>,
    pub resource_usage: ResourceUsage,
}

/// Resource usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_usage: i32,     // Percentage
    pub ram_usage: i32,     // MB
    pub net_usage: i32,     // Kbps
    pub hdd_usage: i32,     // MB
}

/// Clan membership information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanMembership {
    pub clan_id: i32,
    pub role: String,
    pub contribution_points: i32,
    pub joined_date: DateTime<Utc>,
}

/// Target information for hacking attempts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetInfo {
    pub ip_address: String,
    pub target_type: String,
    pub difficulty_level: i32,
    pub security_rating: i32,
    pub reward_money: i64,
    pub defense_systems: Vec<DefenseSystem>,
}

/// Defense system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefenseSystem {
    pub system_type: String,
    pub strength: i32,
    pub detection_rate: Decimal,
    pub response_time: i32, // seconds
}

/// Game mechanics engine
pub struct GameEngine {
    config: config::GameConfig,
}

impl GameEngine {
    /// Create new game engine with default configuration
    pub fn new() -> Self {
        Self {
            config: config::GameConfig::default(),
        }
    }
    
    /// Create new game engine with custom configuration
    pub fn with_config(config: config::GameConfig) -> Self {
        Self { config }
    }
    
    /// Get current game configuration
    pub fn config(&self) -> &config::GameConfig {
        &self.config
    }
    
    /// Update game configuration
    pub fn update_config(&mut self, config: config::GameConfig) {
        self.config = config;
    }
}

impl Default for GameEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Core game mechanics trait
pub trait GameMechanics {
    /// Calculate experience required for a specific level
    fn experience_for_level(&self, level: i32) -> i64;
    
    /// Calculate level from experience points
    fn level_from_experience(&self, experience: i64) -> i32;
    
    /// Calculate process duration based on parameters
    fn calculate_process_duration(&self, process_type: &str, player: &PlayerState, target: &TargetInfo) -> i32;
    
    /// Calculate hack success probability
    fn calculate_success_rate(&self, player: &PlayerState, target: &TargetInfo) -> Decimal;
    
    /// Calculate resource requirements for a process
    fn calculate_resource_usage(&self, process_type: &str, target: &TargetInfo) -> ResourceUsage;
    
    /// Calculate financial rewards
    fn calculate_rewards(&self, process_type: &str, success: bool, target: &TargetInfo, player: &PlayerState) -> (i64, i64); // (money, experience)
    
    /// Calculate hardware performance rating
    fn calculate_hardware_performance(&self, specs: &HardwareSpecs) -> i32;
    
    /// Calculate software effectiveness
    fn calculate_software_effectiveness(&self, software: &SoftwareInstance, hardware: &HardwareSpecs) -> i32;
}

impl GameMechanics for GameEngine {
    fn experience_for_level(&self, level: i32) -> i64 {
        experience::calculate_experience_for_level(level, &self.config.experience)
    }
    
    fn level_from_experience(&self, experience: i64) -> i32 {
        experience::calculate_level_from_experience(experience, &self.config.experience)
    }
    
    fn calculate_process_duration(&self, process_type: &str, player: &PlayerState, target: &TargetInfo) -> i32 {
        process::calculate_duration(process_type, player, target, &self.config.process)
    }
    
    fn calculate_success_rate(&self, player: &PlayerState, target: &TargetInfo) -> Decimal {
        hacking::calculate_success_rate(player, target, &self.config.hacking)
    }
    
    fn calculate_resource_usage(&self, process_type: &str, target: &TargetInfo) -> ResourceUsage {
        process::calculate_resource_usage(process_type, target, &self.config.process)
    }
    
    fn calculate_rewards(&self, process_type: &str, success: bool, target: &TargetInfo, player: &PlayerState) -> (i64, i64) {
        financial::calculate_rewards(process_type, success, target, player, &self.config.financial)
    }
    
    fn calculate_hardware_performance(&self, specs: &HardwareSpecs) -> i32 {
        hardware::calculate_performance_rating(specs, &self.config.hardware)
    }
    
    fn calculate_software_effectiveness(&self, software: &SoftwareInstance, hardware: &HardwareSpecs) -> i32 {
        software::calculate_effectiveness(software, hardware, &self.config.software)
    }
}