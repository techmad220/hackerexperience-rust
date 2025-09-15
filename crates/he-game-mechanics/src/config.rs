//! Game configuration module
//! 
//! Contains all configuration parameters for game mechanics calculations.
//! These values can be adjusted to balance gameplay without changing core logic.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Main game configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub hacking: HackingConfig,
    pub defense: DefenseConfig,
    pub experience: ExperienceConfig,
    pub financial: FinancialConfig,
    pub process: ProcessConfig,
    pub hardware: HardwareConfig,
    pub software: SoftwareConfig,
    pub network: NetworkConfig,
    pub missions: MissionConfig,
    pub clans: ClanConfig,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            hacking: HackingConfig::default(),
            defense: DefenseConfig::default(),
            experience: ExperienceConfig::default(),
            financial: FinancialConfig::default(),
            process: ProcessConfig::default(),
            hardware: HardwareConfig::default(),
            software: SoftwareConfig::default(),
            network: NetworkConfig::default(),
            missions: MissionConfig::default(),
            clans: ClanConfig::default(),
        }
    }
}

/// Hacking system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HackingConfig {
    // Success rate parameters
    pub base_success_rate: Decimal,
    pub min_success_rate: Decimal,
    pub max_success_rate: Decimal,
    pub skill_bonus_per_level: Decimal,
    pub skill_penalty_per_level: Decimal,
    pub security_penalty_per_point: Decimal,
    pub reputation_bonus_per_point: Decimal,
    
    // Equipment modifiers
    pub software_effectiveness_multiplier: Decimal,
    pub hardware_bonus_multiplier: Decimal,
    pub advanced_tool_speed_bonus: Decimal,
    
    // Time calculations
    pub base_hack_time: i32,
    pub min_hack_time: i32,
    pub time_per_difficulty_point: i32,
    pub skill_speed_bonus: Decimal,
    pub cpu_speed_multiplier: Decimal,
    
    // Detection system
    pub base_detection_rate: Decimal,
    pub min_detection_rate: Decimal,
    pub max_detection_rate: Decimal,
    pub stealth_effectiveness_multiplier: Decimal,
    pub stealth_skill_bonus: Decimal,
    pub security_detection_multiplier: Decimal,
    
    // Rewards
    pub level_reward_multiplier: Decimal,
    pub base_experience_per_difficulty: i64,
    
    // Specialized attacks
    pub brute_force_base_time: i32,
    pub min_brute_force_time: i32,
    pub min_dictionary_time: i32,
    
    // Scanning
    pub port_scan_base_time: i32,
    pub vulnerability_scan_base_time: i32,
    pub deep_scan_base_time: i32,
    pub min_scan_time: i32,
}

impl Default for HackingConfig {
    fn default() -> Self {
        Self {
            // Success rate parameters (matching original HE formulas)
            base_success_rate: dec!(0.50),        // 50% base success
            min_success_rate: dec!(0.05),         // 5% minimum
            max_success_rate: dec!(0.95),         // 95% maximum
            skill_bonus_per_level: dec!(0.02),    // +2% per level advantage
            skill_penalty_per_level: dec!(0.05),  // -5% per level disadvantage
            security_penalty_per_point: dec!(0.01), // -1% per security point
            reputation_bonus_per_point: dec!(0.001), // +0.1% per reputation point
            
            // Equipment modifiers
            software_effectiveness_multiplier: dec!(0.01), // 1% bonus per effectiveness point
            hardware_bonus_multiplier: dec!(0.001),        // Hardware performance bonus
            advanced_tool_speed_bonus: dec!(0.15),         // 15% speed bonus for advanced tools
            
            // Time calculations (in seconds)
            base_hack_time: 300,                  // 5 minutes base
            min_hack_time: 30,                    // 30 seconds minimum
            time_per_difficulty_point: 30,        // +30 seconds per difficulty
            skill_speed_bonus: dec!(0.02),        // -2% time per skill level
            cpu_speed_multiplier: dec!(0.1),      // CPU speed affects time
            
            // Detection system
            base_detection_rate: dec!(0.20),      // 20% base detection
            min_detection_rate: dec!(0.05),       // 5% minimum detection
            max_detection_rate: dec!(0.80),       // 80% maximum detection
            stealth_effectiveness_multiplier: dec!(0.01), // Stealth software effectiveness
            stealth_skill_bonus: dec!(0.001),     // Stealth skill bonus
            security_detection_multiplier: dec!(0.01),    // Security increases detection
            
            // Rewards
            level_reward_multiplier: dec!(0.05),  // +5% reward per level
            base_experience_per_difficulty: 100,  // 100 XP per difficulty point
            
            // Specialized attacks
            brute_force_base_time: 60,            // 1 minute base per complexity level
            min_brute_force_time: 10,             // 10 seconds minimum
            min_dictionary_time: 5,               // 5 seconds minimum
            
            // Scanning times
            port_scan_base_time: 60,              // 1 minute
            vulnerability_scan_base_time: 300,    // 5 minutes
            deep_scan_base_time: 900,             // 15 minutes
            min_scan_time: 10,                    // 10 seconds minimum
        }
    }
}

/// Defense system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefenseConfig {
    pub firewall_effectiveness_multiplier: Decimal,
    pub ids_detection_bonus: Decimal,
    pub honeypot_trap_rate: Decimal,
    pub log_retention_multiplier: Decimal,
    pub security_update_bonus: Decimal,
    pub intrusion_response_time: i32,
    pub trace_decay_rate: Decimal,
    pub defense_skill_multiplier: Decimal,
}

impl Default for DefenseConfig {
    fn default() -> Self {
        Self {
            firewall_effectiveness_multiplier: dec!(0.015), // 1.5% per effectiveness point
            ids_detection_bonus: dec!(0.25),               // 25% detection bonus
            honeypot_trap_rate: dec!(0.30),                // 30% chance to trap attackers
            log_retention_multiplier: dec!(1.5),           // Logs retained 1.5x longer
            security_update_bonus: dec!(0.10),             // 10% defense bonus when updated
            intrusion_response_time: 60,                   // 60 seconds response time
            trace_decay_rate: dec!(0.10),                  // 10% trace decay per day
            defense_skill_multiplier: dec!(0.02),          // 2% defense per skill level
        }
    }
}

/// Experience and leveling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceConfig {
    pub base_experience_per_level: i64,
    pub level_multiplier: Decimal,
    pub max_level: i32,
    pub experience_curve_factor: Decimal,
    pub skill_specialization_bonus: Decimal,
    pub learning_efficiency_decay: Decimal,
}

impl Default for ExperienceConfig {
    fn default() -> Self {
        Self {
            base_experience_per_level: 1000,      // 1000 XP for level 1
            level_multiplier: dec!(1.15),         // 15% more XP per level
            max_level: 100,                       // Maximum level 100
            experience_curve_factor: dec!(1.2),   // Curve steepness
            skill_specialization_bonus: dec!(0.20), // 20% bonus for specialization
            learning_efficiency_decay: dec!(0.95), // Learning efficiency decreases
        }
    }
}

/// Financial system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialConfig {
    pub base_bank_interest_rate: Decimal,
    pub transaction_fee_rate: Decimal,
    pub money_transfer_delay: i32,
    pub cryptocurrency_volatility: Decimal,
    pub market_maker_fee: Decimal,
    pub inflation_rate: Decimal,
    pub reward_multiplier_cap: Decimal,
    pub bankruptcy_threshold: i64,
}

impl Default for FinancialConfig {
    fn default() -> Self {
        Self {
            base_bank_interest_rate: dec!(0.001),  // 0.1% daily interest
            transaction_fee_rate: dec!(0.02),      // 2% transaction fee
            money_transfer_delay: 300,             // 5 minutes delay
            cryptocurrency_volatility: dec!(0.10), // 10% daily volatility
            market_maker_fee: dec!(0.005),         // 0.5% market making fee
            inflation_rate: dec!(0.0001),          // 0.01% daily inflation
            reward_multiplier_cap: dec!(5.0),      // 5x maximum reward multiplier
            bankruptcy_threshold: 0,               // $0 bankruptcy threshold
        }
    }
}

/// Process system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfig {
    pub max_concurrent_processes: i32,
    pub cpu_usage_per_process: i32,
    pub ram_usage_per_process: i32,
    pub network_usage_per_process: i32,
    pub process_priority_levels: i32,
    pub resource_efficiency_bonus: Decimal,
    pub multitasking_penalty: Decimal,
    pub process_failure_rate: Decimal,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            max_concurrent_processes: 5,          // 5 max concurrent processes
            cpu_usage_per_process: 20,            // 20% CPU per process
            ram_usage_per_process: 512,           // 512 MB RAM per process
            network_usage_per_process: 10,        // 10 Kbps per process
            process_priority_levels: 3,           // 3 priority levels
            resource_efficiency_bonus: dec!(0.10), // 10% efficiency bonus
            multitasking_penalty: dec!(0.15),     // 15% penalty for multitasking
            process_failure_rate: dec!(0.02),     // 2% random failure rate
        }
    }
}

/// Hardware system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareConfig {
    pub cpu_performance_weight: Decimal,
    pub ram_performance_weight: Decimal,
    pub hdd_performance_weight: Decimal,
    pub net_performance_weight: Decimal,
    pub upgrade_cost_multiplier: Decimal,
    pub hardware_degradation_rate: Decimal,
    pub overclocking_risk_factor: Decimal,
    pub compatibility_bonus: Decimal,
}

impl Default for HardwareConfig {
    fn default() -> Self {
        Self {
            cpu_performance_weight: dec!(0.40),   // CPU is 40% of performance
            ram_performance_weight: dec!(0.25),   // RAM is 25% of performance
            hdd_performance_weight: dec!(0.20),   // HDD is 20% of performance
            net_performance_weight: dec!(0.15),   // Network is 15% of performance
            upgrade_cost_multiplier: dec!(1.5),   // Upgrades cost 1.5x more each time
            hardware_degradation_rate: dec!(0.001), // 0.1% degradation per day
            overclocking_risk_factor: dec!(0.05), // 5% failure risk when overclocking
            compatibility_bonus: dec!(0.10),      // 10% bonus for compatible hardware
        }
    }
}

/// Software system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareConfig {
    pub installation_time_base: i32,
    pub compilation_time_multiplier: Decimal,
    pub dependency_resolution_time: i32,
    pub software_update_interval: i32,
    pub version_compatibility_range: i32,
    pub effectiveness_decay_rate: Decimal,
    pub bug_fix_effectiveness_bonus: Decimal,
    pub open_source_cost_reduction: Decimal,
}

impl Default for SoftwareConfig {
    fn default() -> Self {
        Self {
            installation_time_base: 60,           // 1 minute base installation
            compilation_time_multiplier: dec!(2.0), // Compilation takes 2x longer
            dependency_resolution_time: 30,       // 30 seconds per dependency
            software_update_interval: 86400,      // 24 hours between updates
            version_compatibility_range: 3,       // 3 version compatibility
            effectiveness_decay_rate: dec!(0.01), // 1% effectiveness decay per month
            bug_fix_effectiveness_bonus: dec!(0.05), // 5% bonus per bug fix
            open_source_cost_reduction: dec!(0.30), // 30% cost reduction for open source
        }
    }
}

/// Network system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub connection_timeout: i32,
    pub bandwidth_efficiency: Decimal,
    pub routing_optimization_bonus: Decimal,
    pub network_congestion_penalty: Decimal,
    pub packet_loss_threshold: Decimal,
    pub latency_calculation_base: i32,
    pub proxy_chain_effectiveness: Decimal,
    pub vpn_security_bonus: Decimal,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            connection_timeout: 30,               // 30 second timeout
            bandwidth_efficiency: dec!(0.85),     // 85% bandwidth efficiency
            routing_optimization_bonus: dec!(0.15), // 15% speed bonus for optimal routing
            network_congestion_penalty: dec!(0.25), // 25% penalty during congestion
            packet_loss_threshold: dec!(0.05),    // 5% packet loss threshold
            latency_calculation_base: 50,         // 50ms base latency
            proxy_chain_effectiveness: dec!(0.80), // 80% effectiveness through proxies
            vpn_security_bonus: dec!(0.30),       // 30% security bonus with VPN
        }
    }
}

/// Mission system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionConfig {
    pub difficulty_scaling_factor: Decimal,
    pub reward_scaling_factor: Decimal,
    pub prerequisite_check_strictness: Decimal,
    pub mission_timeout_multiplier: Decimal,
    pub completion_bonus_percentage: Decimal,
    pub failure_penalty_percentage: Decimal,
    pub daily_mission_limit: i32,
    pub reputation_requirement_scaling: Decimal,
}

impl Default for MissionConfig {
    fn default() -> Self {
        Self {
            difficulty_scaling_factor: dec!(1.2), // 20% difficulty increase per level
            reward_scaling_factor: dec!(1.15),    // 15% reward increase per level
            prerequisite_check_strictness: dec!(0.90), // 90% strictness for prerequisites
            mission_timeout_multiplier: dec!(2.0), // Missions timeout in 2x estimated time
            completion_bonus_percentage: dec!(0.20), // 20% bonus for fast completion
            failure_penalty_percentage: dec!(0.10), // 10% penalty for failure
            daily_mission_limit: 10,              // 10 missions per day
            reputation_requirement_scaling: dec!(1.1), // Reputation requirements scale 10%
        }
    }
}

/// Clan system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanConfig {
    pub max_clan_members: i32,
    pub contribution_decay_rate: Decimal,
    pub warfare_damage_multiplier: Decimal,
    pub reputation_sharing_rate: Decimal,
    pub clan_bonus_effectiveness: Decimal,
    pub war_declaration_cooldown: i32,
    pub member_kick_cooldown: i32,
    pub leadership_transfer_requirement: Decimal,
}

impl Default for ClanConfig {
    fn default() -> Self {
        Self {
            max_clan_members: 50,                 // Maximum 50 members per clan
            contribution_decay_rate: dec!(0.02), // 2% contribution decay per week
            warfare_damage_multiplier: dec!(1.5), // 50% damage bonus in wars
            reputation_sharing_rate: dec!(0.10), // 10% reputation shared with clan
            clan_bonus_effectiveness: dec!(0.25), // 25% effectiveness bonus from clan
            war_declaration_cooldown: 86400,     // 24 hour cooldown between wars
            member_kick_cooldown: 3600,          // 1 hour cooldown after kicking member
            leadership_transfer_requirement: dec!(0.75), // Need 75% member approval
        }
    }
}

/// Load configuration from file or environment
pub fn load_config() -> Result<GameConfig, Box<dyn std::error::Error>> {
    // Try to load from file first
    if let Ok(contents) = std::fs::read_to_string("game_config.toml") {
        match toml::from_str(&contents) {
            Ok(config) => return Ok(config),
            Err(e) => eprintln!("Failed to parse config file: {}", e),
        }
    }
    
    // Try to load from JSON file
    if let Ok(contents) = std::fs::read_to_string("game_config.json") {
        match serde_json::from_str(&contents) {
            Ok(config) => return Ok(config),
            Err(e) => eprintln!("Failed to parse JSON config: {}", e),
        }
    }
    
    // Fall back to environment variables or defaults
    Ok(GameConfig::default())
}

/// Save configuration to file
pub fn save_config(config: &GameConfig) -> Result<(), Box<dyn std::error::Error>> {
    let toml_content = toml::to_string_pretty(config)?;
    std::fs::write("game_config.toml", toml_content)?;
    
    let json_content = serde_json::to_string_pretty(config)?;
    std::fs::write("game_config.json", json_content)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config_creation() {
        let config = GameConfig::default();
        assert_eq!(config.hacking.base_success_rate, dec!(0.50));
        assert_eq!(config.experience.max_level, 100);
        assert_eq!(config.process.max_concurrent_processes, 5);
    }
    
    #[test]
    fn test_config_serialization() {
        let config = GameConfig::default();
        
        // Test JSON serialization
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: GameConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.hacking.base_success_rate, deserialized.hacking.base_success_rate);
    }
}