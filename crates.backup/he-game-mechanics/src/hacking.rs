//! Hacking system mechanics
//! 
//! Implements all hacking algorithms, difficulty calculations, success rates,
//! and intrusion mechanics from the original HackerExperience game.

use crate::{PlayerState, TargetInfo, DefenseSystem, Result, GameMechanicsError};
use crate::config::HackingConfig;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use rand::Rng;
use std::collections::HashMap;

/// Hacking attempt result
#[derive(Debug, Clone)]
pub struct HackingResult {
    pub success: bool,
    pub detection_level: Decimal, // 0.0 to 1.0
    pub time_taken: i32,         // seconds
    pub experience_gained: i64,
    pub money_gained: i64,
    pub traces_left: i32,
    pub damage_dealt: i32,
}

/// Calculate hacking success rate based on player skills, target difficulty, and equipment
/// 
/// Formula: Base rate × Skill modifier × Equipment modifier × Target difficulty modifier × Luck factor
/// 
/// Original formula from HE Legacy:
/// - Base success rate: 50%
/// - Skill bonus: +2% per level above minimum requirement
/// - Equipment bonus: Software effectiveness / 10
/// - Target penalty: -5% per difficulty point above player level
/// - Luck factor: ±10% random variation
pub fn calculate_success_rate(
    player: &PlayerState,
    target: &TargetInfo,
    config: &HackingConfig,
) -> Decimal {
    // Base success rate (typically 50%)
    let mut success_rate = config.base_success_rate;
    
    // Skill modifier based on player level vs target difficulty
    let skill_diff = player.level - target.difficulty_level;
    let skill_modifier = if skill_diff > 0 {
        dec!(1.0) + (Decimal::from(skill_diff) * config.skill_bonus_per_level)
    } else {
        dec!(1.0) + (Decimal::from(skill_diff) * config.skill_penalty_per_level)
    };
    
    // Equipment modifier from installed hacking software
    let equipment_modifier = calculate_equipment_modifier(player, config);
    
    // Target security modifier
    let security_modifier = dec!(1.0) - (Decimal::from(target.security_rating) * config.security_penalty_per_point / dec!(100.0));
    
    // Reputation bonus for hacking reputation
    let reputation_bonus = player.reputation.get("hacking")
        .map(|rep| dec!(1.0) + (Decimal::from(*rep) * config.reputation_bonus_per_point / dec!(1000.0)))
        .unwrap_or(dec!(1.0));
    
    // Calculate final success rate
    success_rate = success_rate * skill_modifier * equipment_modifier * security_modifier * reputation_bonus;
    
    // Clamp between minimum and maximum success rates
    success_rate.max(config.min_success_rate).min(config.max_success_rate)
}

/// Calculate equipment modifier based on installed hacking software
fn calculate_equipment_modifier(player: &PlayerState, config: &HackingConfig) -> Decimal {
    let mut modifier = dec!(1.0);
    
    // Check for password cracker
    if let Some(cracker) = player.software_installed.iter().find(|s| s.software_type == "password_cracker") {
        modifier += Decimal::from(cracker.effectiveness) * config.software_effectiveness_multiplier / dec!(100.0);
    }
    
    // Check for vulnerability scanner
    if let Some(scanner) = player.software_installed.iter().find(|s| s.software_type == "vulnerability_scanner") {
        modifier += Decimal::from(scanner.effectiveness) * config.software_effectiveness_multiplier / dec!(100.0);
    }
    
    // Check for exploit toolkit
    if let Some(exploit) = player.software_installed.iter().find(|s| s.software_type == "exploit_toolkit") {
        modifier += Decimal::from(exploit.effectiveness) * config.software_effectiveness_multiplier / dec!(100.0);
    }
    
    // Hardware performance bonus
    let hardware_bonus = Decimal::from(player.hardware_specs.performance_rating) * config.hardware_bonus_multiplier / dec!(1000.0);
    modifier += hardware_bonus;
    
    modifier
}

/// Calculate hacking time based on target difficulty and player capabilities
/// 
/// Formula: Base time × Difficulty multiplier × Equipment speed modifier × Skill speed modifier
/// 
/// Original HE formula:
/// - Base time: 300 seconds (5 minutes)
/// - Difficulty multiplier: +30 seconds per difficulty point
/// - Equipment speed: -10% per software effectiveness point
/// - Skill speed: -2% per level above minimum
pub fn calculate_hacking_time(
    player: &PlayerState,
    target: &TargetInfo,
    config: &HackingConfig,
) -> i32 {
    // Base hacking time
    let mut base_time = config.base_hack_time;
    
    // Difficulty modifier
    let difficulty_modifier = target.difficulty_level * config.time_per_difficulty_point;
    base_time += difficulty_modifier;
    
    // Equipment speed modifier
    let equipment_speed = calculate_equipment_speed_modifier(player, config);
    let equipment_time_reduction = (Decimal::from(base_time) * equipment_speed).to_i32().unwrap_or(0);
    base_time -= equipment_time_reduction;
    
    // Skill speed modifier
    let skill_diff = player.level - target.difficulty_level;
    let skill_speed_reduction = if skill_diff > 0 {
        (Decimal::from(base_time) * Decimal::from(skill_diff) * config.skill_speed_bonus).to_i32().unwrap_or(0)
    } else {
        0
    };
    base_time -= skill_speed_reduction;
    
    // Hardware performance bonus
    let cpu_bonus = (Decimal::from(player.hardware_specs.cpu) * config.cpu_speed_multiplier / dec!(1000.0)).to_i32().unwrap_or(0);
    base_time -= cpu_bonus;
    
    // Minimum time constraint
    base_time.max(config.min_hack_time)
}

/// Calculate equipment speed modifier
fn calculate_equipment_speed_modifier(player: &PlayerState, config: &HackingConfig) -> Decimal {
    let mut speed_modifier = dec!(0.0);
    
    // Fast tools reduce time
    if player.software_installed.iter().any(|s| s.software_type == "advanced_scanner") {
        speed_modifier += config.advanced_tool_speed_bonus;
    }
    
    if player.software_installed.iter().any(|s| s.software_type == "exploit_framework") {
        speed_modifier += config.advanced_tool_speed_bonus;
    }
    
    speed_modifier
}

/// Calculate detection probability during hacking attempt
/// 
/// Based on target's intrusion detection systems and player's stealth capabilities
pub fn calculate_detection_probability(
    player: &PlayerState,
    target: &TargetInfo,
    config: &HackingConfig,
) -> Decimal {
    let mut detection_rate = config.base_detection_rate;
    
    // Target's defense systems increase detection
    for defense in &target.defense_systems {
        if defense.system_type == "intrusion_detection" {
            detection_rate += defense.detection_rate;
        }
    }
    
    // Player's stealth software reduces detection
    if let Some(log_cleaner) = player.software_installed.iter().find(|s| s.software_type == "log_cleaner") {
        detection_rate -= Decimal::from(log_cleaner.effectiveness) * config.stealth_effectiveness_multiplier / dec!(100.0);
    }
    
    if let Some(proxy) = player.software_installed.iter().find(|s| s.software_type == "proxy_chain") {
        detection_rate -= Decimal::from(proxy.effectiveness) * config.stealth_effectiveness_multiplier / dec!(100.0);
    }
    
    // Skill-based stealth improvement
    let stealth_skill = player.reputation.get("stealth").unwrap_or(&0);
    detection_rate -= Decimal::from(*stealth_skill) * config.stealth_skill_bonus / dec!(1000.0);
    
    // Network security level affects detection
    detection_rate += Decimal::from(target.security_rating) * config.security_detection_multiplier / dec!(100.0);
    
    detection_rate.max(config.min_detection_rate).min(config.max_detection_rate)
}

/// Perform a complete hacking attempt with all calculations
pub fn perform_hack(
    player: &PlayerState,
    target: &TargetInfo,
    config: &HackingConfig,
) -> Result<HackingResult> {
    // Validate inputs
    if player.level < 1 {
        return Err(GameMechanicsError::InvalidParameter("Player level must be at least 1".to_string()));
    }
    
    if target.difficulty_level < 1 || target.difficulty_level > 10 {
        return Err(GameMechanicsError::InvalidParameter("Target difficulty must be between 1 and 10".to_string()));
    }
    
    let mut rng = rand::thread_rng();
    
    // Calculate success probability
    let success_rate = calculate_success_rate(player, target, config);
    let luck_factor: f64 = rng.gen_range(-0.1..=0.1);
    let final_success_rate = success_rate + Decimal::from_f64_retain(luck_factor).unwrap_or(dec!(0.0));
    
    // Determine success
    let success = rng.gen::<f64>() < final_success_rate.to_f64().unwrap_or(0.0);
    
    // Calculate time taken
    let time_taken = calculate_hacking_time(player, target, config);
    
    // Calculate detection level
    let detection_probability = calculate_detection_probability(player, target, config);
    let detection_level = if rng.gen::<f64>() < detection_probability.to_f64().unwrap_or(0.0) {
        Decimal::from_f64_retain(rng.gen_range(0.3..=1.0)).unwrap_or(dec!(0.5))
    } else {
        Decimal::from_f64_retain(rng.gen_range(0.0..=0.2)).unwrap_or(dec!(0.1))
    };
    
    // Calculate rewards based on success and target value
    let (money_gained, experience_gained) = if success {
        let base_money = target.reward_money;
        let skill_multiplier = dec!(1.0) + (Decimal::from(player.level) * config.level_reward_multiplier);
        let money = (Decimal::from(base_money) * skill_multiplier).to_i64().unwrap_or(0);
        
        let base_exp = target.difficulty_level as i64 * config.base_experience_per_difficulty;
        let experience = (Decimal::from(base_exp) * skill_multiplier).to_i64().unwrap_or(0);
        
        (money, experience)
    } else {
        // Failed attempts still give some experience
        let failed_exp = (target.difficulty_level as i64 * config.base_experience_per_difficulty) / 4;
        (0, failed_exp)
    };
    
    // Calculate traces left (affects future hacking attempts)
    let traces_left = if detection_level > dec!(0.7) {
        rng.gen_range(3..=7)
    } else if detection_level > dec!(0.4) {
        rng.gen_range(1..=3)
    } else {
        0
    };
    
    // Calculate damage dealt to target (for clan wars)
    let damage_dealt = if success {
        let base_damage = target.difficulty_level * 10;
        let damage_modifier = player.reputation.get("warfare").unwrap_or(&0) / 100;
        base_damage + damage_modifier
    } else {
        0
    };
    
    Ok(HackingResult {
        success,
        detection_level,
        time_taken,
        experience_gained,
        money_gained,
        traces_left,
        damage_dealt,
    })
}

/// Calculate brute force attack parameters for password cracking
pub fn calculate_brute_force_time(
    password_complexity: i32,
    cpu_power: i32,
    software_effectiveness: i32,
    config: &HackingConfig,
) -> i32 {
    // Base time exponentially increases with password complexity
    let complexity_factor = 2_i32.pow(password_complexity as u32);
    let base_time = complexity_factor * config.brute_force_base_time;
    
    // CPU power reduces time
    let cpu_multiplier = dec!(1000.0) / Decimal::from(cpu_power.max(1));
    
    // Software effectiveness reduces time
    let software_multiplier = dec!(100.0) / Decimal::from(software_effectiveness.max(1));
    
    let final_time = Decimal::from(base_time) * cpu_multiplier * software_multiplier;
    final_time.to_i32().unwrap_or(base_time).max(config.min_brute_force_time)
}

/// Calculate dictionary attack parameters
pub fn calculate_dictionary_attack_time(
    dictionary_size: i32,
    cpu_power: i32,
    password_in_dictionary: bool,
    config: &HackingConfig,
) -> i32 {
    if password_in_dictionary {
        // Password found in dictionary - much faster
        let base_time = dictionary_size / 100; // Check 100 passwords per second baseline
        let cpu_bonus = cpu_power / 100;
        (base_time - cpu_bonus).max(config.min_dictionary_time)
    } else {
        // Password not in dictionary - need to try entire dictionary
        let base_time = dictionary_size / 10; // Slower when password not found
        let cpu_bonus = cpu_power / 100;
        (base_time - cpu_bonus).max(config.min_dictionary_time * 5)
    }
}

/// Calculate firewall bypass difficulty
pub fn calculate_firewall_bypass_difficulty(
    firewall_strength: i32,
    player_exploit_skill: i32,
    exploit_effectiveness: i32,
) -> Decimal {
    let base_difficulty = Decimal::from(firewall_strength) / dec!(100.0);
    let skill_reduction = Decimal::from(player_exploit_skill) / dec!(500.0);
    let tool_reduction = Decimal::from(exploit_effectiveness) / dec!(200.0);
    
    (base_difficulty - skill_reduction - tool_reduction).max(dec!(0.1)).min(dec!(0.9))
}

/// Calculate network scanning time and results
pub fn calculate_network_scan(
    target_ip: &str,
    scan_type: &str,
    scanner_effectiveness: i32,
    network_speed: i32,
    config: &HackingConfig,
) -> Result<(i32, Vec<String>)> {
    let scan_time = match scan_type {
        "port_scan" => {
            let base_time = config.port_scan_base_time;
            let speed_bonus = network_speed / 10;
            let scanner_bonus = scanner_effectiveness / 5;
            (base_time - speed_bonus - scanner_bonus).max(config.min_scan_time)
        }
        "vulnerability_scan" => {
            let base_time = config.vulnerability_scan_base_time;
            let scanner_bonus = scanner_effectiveness / 3;
            (base_time - scanner_bonus).max(config.min_scan_time * 3)
        }
        "deep_scan" => {
            let base_time = config.deep_scan_base_time;
            let scanner_bonus = scanner_effectiveness / 2;
            (base_time - scanner_bonus).max(config.min_scan_time * 10)
        }
        _ => return Err(GameMechanicsError::InvalidParameter(format!("Unknown scan type: {}", scan_type))),
    };
    
    // Generate scan results based on scanner effectiveness
    let mut results = Vec::new();
    
    if scanner_effectiveness > 20 {
        results.push("SSH service detected on port 22".to_string());
    }
    if scanner_effectiveness > 40 {
        results.push("HTTP service detected on port 80".to_string());
        results.push("HTTPS service detected on port 443".to_string());
    }
    if scanner_effectiveness > 60 {
        results.push("FTP service detected on port 21".to_string());
        results.push("Database service detected on port 3306".to_string());
    }
    if scanner_effectiveness > 80 && scan_type != "port_scan" {
        results.push("Vulnerable SSH configuration detected".to_string());
        results.push("Outdated web server version detected".to_string());
    }
    
    Ok((scan_time, results))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PlayerState, TargetInfo, HardwareSpecs, SoftwareInstance, DefenseSystem};
    use chrono::Utc;
    use std::collections::HashMap;
    
    fn create_test_player() -> PlayerState {
        PlayerState {
            user_id: 1,
            level: 10,
            experience: 20000,
            money: 50000,
            reputation: {
                let mut rep = HashMap::new();
                rep.insert("hacking".to_string(), 500);
                rep
            },
            hardware_specs: HardwareSpecs {
                cpu: 2400,
                ram: 8192,
                hdd: 512000,
                net: 100,
                security_level: 5,
                performance_rating: 75,
            },
            software_installed: vec![
                SoftwareInstance {
                    software_type: "password_cracker".to_string(),
                    version: "v2.1".to_string(),
                    effectiveness: 70,
                    dependencies: vec![],
                    installation_date: Utc::now(),
                }
            ],
            active_processes: vec![],
            clan_membership: None,
            last_updated: Utc::now(),
        }
    }
    
    fn create_test_target() -> TargetInfo {
        TargetInfo {
            ip_address: "192.168.1.100".to_string(),
            target_type: "server".to_string(),
            difficulty_level: 8,
            security_rating: 65,
            reward_money: 25000,
            defense_systems: vec![
                DefenseSystem {
                    system_type: "firewall".to_string(),
                    strength: 70,
                    detection_rate: dec!(0.3),
                    response_time: 60,
                }
            ],
        }
    }
    
    #[test]
    fn test_success_rate_calculation() {
        let player = create_test_player();
        let target = create_test_target();
        let config = HackingConfig::default();
        
        let success_rate = calculate_success_rate(&player, &target, &config);
        assert!(success_rate >= dec!(0.0) && success_rate <= dec!(1.0));
    }
    
    #[test]
    fn test_hacking_time_calculation() {
        let player = create_test_player();
        let target = create_test_target();
        let config = HackingConfig::default();
        
        let time = calculate_hacking_time(&player, &target, &config);
        assert!(time > 0);
        assert!(time >= config.min_hack_time);
    }
    
    #[test]
    fn test_detection_probability() {
        let player = create_test_player();
        let target = create_test_target();
        let config = HackingConfig::default();
        
        let detection = calculate_detection_probability(&player, &target, &config);
        assert!(detection >= dec!(0.0) && detection <= dec!(1.0));
    }
    
    #[test]
    fn test_brute_force_time() {
        let config = HackingConfig::default();
        let time = calculate_brute_force_time(6, 2400, 70, &config);
        assert!(time > 0);
        assert!(time >= config.min_brute_force_time);
    }
}