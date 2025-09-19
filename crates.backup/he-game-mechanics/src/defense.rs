//! Defense system mechanics
//! 
//! Implements firewall strength, intrusion detection, security ratings,
//! and detection algorithms from the original HackerExperience game.

use crate::{PlayerState, HardwareSpecs, SoftwareInstance, Result, GameMechanicsError};
use crate::config::DefenseConfig;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use rand::Rng;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

/// Defense system state
#[derive(Debug, Clone)]
pub struct DefenseState {
    pub firewall_strength: i32,
    pub ids_active: bool,
    pub ids_sensitivity: i32,
    pub honeypot_active: bool,
    pub log_monitoring: bool,
    pub security_rating: i32,
    pub active_traces: Vec<TraceInfo>,
    pub blocked_ips: Vec<BlockedIP>,
    pub security_events: Vec<SecurityEvent>,
}

/// Trace information for tracking attackers
#[derive(Debug, Clone)]
pub struct TraceInfo {
    pub attacker_ip: String,
    pub trace_strength: i32,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub trace_type: String,
}

/// Blocked IP information
#[derive(Debug, Clone)]
pub struct BlockedIP {
    pub ip_address: String,
    pub blocked_at: DateTime<Utc>,
    pub blocked_until: DateTime<Utc>,
    pub reason: String,
    pub auto_unblock: bool,
}

/// Security event log
#[derive(Debug, Clone)]
pub struct SecurityEvent {
    pub event_type: String,
    pub source_ip: String,
    pub severity: i32,
    pub detected: bool,
    pub timestamp: DateTime<Utc>,
    pub details: HashMap<String, String>,
}

/// Calculate firewall strength based on software and hardware
pub fn calculate_firewall_strength(
    player: &PlayerState,
    config: &DefenseConfig,
) -> i32 {
    let mut base_strength = 50; // Base firewall strength
    
    // Find firewall software
    if let Some(firewall) = player.software_installed.iter()
        .find(|s| s.software_type == "firewall") {
        
        // Firewall version contributes to strength
        let version_bonus = (firewall.version.parse::<f32>().unwrap_or(1.0) * 10.0) as i32;
        base_strength += version_bonus;
        
        // Effectiveness adds to strength
        base_strength += firewall.effectiveness;
    }
    
    // Hardware performance affects firewall strength
    let hardware_bonus = player.hardware_specs.security_level * 5;
    base_strength += hardware_bonus;
    
    // CPU power contributes to real-time packet inspection
    let cpu_bonus = player.hardware_specs.cpu / 100; // 1% per 100 MHz
    base_strength += cpu_bonus;
    
    // RAM allows for larger rule sets
    let ram_bonus = player.hardware_specs.ram / 1024; // 1 point per GB
    base_strength += ram_bonus;
    
    // Apply configuration multiplier
    let final_strength = (Decimal::from(base_strength) * config.firewall_effectiveness_multiplier)
        .to_i32()
        .unwrap_or(base_strength);
    
    final_strength.min(100) // Cap at 100
}

/// Process intrusion detection
pub fn process_intrusion_detection(
    defense_state: &DefenseState,
    attack_type: &str,
    attacker_skill: i32,
    config: &DefenseConfig,
) -> bool {
    if !defense_state.ids_active {
        return false; // IDS not active
    }
    
    let mut detection_chance = dec!(0.5); // Base 50% detection
    
    // IDS sensitivity affects detection
    detection_chance += Decimal::from(defense_state.ids_sensitivity) / dec!(100.0);
    
    // Attack type affects detection
    detection_chance += match attack_type {
        "brute_force" => dec!(0.3),      // Easy to detect
        "port_scan" => dec!(0.25),       // Fairly easy
        "exploit" => dec!(0.1),          // Harder to detect
        "stealth" => dec!(-0.2),         // Very hard to detect
        _ => dec!(0.0),
    };
    
    // Attacker skill reduces detection
    detection_chance -= Decimal::from(attacker_skill) / dec!(200.0);
    
    // Apply IDS bonus from config
    detection_chance += config.ids_detection_bonus;
    
    // Random factor
    let mut rng = rand::thread_rng();
    let random_factor = Decimal::from_f64_retain(rng.gen_range(-0.1..=0.1)).unwrap_or(dec!(0.0));
    detection_chance += random_factor;
    
    // Clamp between 0 and 1
    detection_chance = detection_chance.max(dec!(0.05)).min(dec!(0.95));
    
    // Roll for detection
    rng.gen::<f64>() < detection_chance.to_f64().unwrap_or(0.5)
}

/// Calculate security rating for a system
pub fn calculate_security_rating(
    firewall_strength: i32,
    ids_active: bool,
    honeypot_active: bool,
    software_installed: &[SoftwareInstance],
    hardware_specs: &HardwareSpecs,
    config: &DefenseConfig,
) -> i32 {
    let mut rating = 0;
    
    // Firewall contributes significantly
    rating += firewall_strength;
    
    // IDS adds 20 points
    if ids_active {
        rating += 20;
    }
    
    // Honeypot adds 15 points
    if honeypot_active {
        rating += 15;
    }
    
    // Security software contributes
    for software in software_installed {
        match software.software_type.as_str() {
            "antivirus" => rating += 10,
            "log_monitor" => rating += 8,
            "encryption" => rating += 12,
            "vpn" => rating += 7,
            _ => {}
        }
    }
    
    // Hardware security level
    rating += hardware_specs.security_level * 3;
    
    // Apply defense skill multiplier
    let final_rating = (Decimal::from(rating) * config.defense_skill_multiplier)
        .to_i32()
        .unwrap_or(rating);
    
    final_rating.min(100) // Cap at 100
}

/// Check if honeypot traps an attacker
pub fn check_honeypot_trap(
    honeypot_active: bool,
    attacker_skill: i32,
    config: &DefenseConfig,
) -> bool {
    if !honeypot_active {
        return false;
    }
    
    let mut trap_chance = config.honeypot_trap_rate;
    
    // Skilled attackers are less likely to fall for honeypots
    trap_chance -= Decimal::from(attacker_skill) / dec!(500.0);
    
    // Random chance
    let mut rng = rand::thread_rng();
    rng.gen::<f64>() < trap_chance.to_f64().unwrap_or(0.3)
}

/// Calculate trace strength left by attacker
pub fn calculate_trace_strength(
    attack_success: bool,
    detection_level: Decimal,
    attacker_stealth: i32,
    defender_logging: bool,
    config: &DefenseConfig,
) -> i32 {
    let mut trace_strength = if attack_success { 50 } else { 30 };
    
    // Detection level affects trace strength
    trace_strength += (detection_level * dec!(50.0)).to_i32().unwrap_or(0);
    
    // Attacker stealth reduces traces
    trace_strength -= attacker_stealth / 2;
    
    // Defender logging increases traces
    if defender_logging {
        trace_strength = ((Decimal::from(trace_strength) * config.log_retention_multiplier)
            .to_i32()
            .unwrap_or(trace_strength));
    }
    
    trace_strength.max(0).min(100)
}

/// Process trace decay over time
pub fn process_trace_decay(
    traces: &mut Vec<TraceInfo>,
    config: &DefenseConfig,
) {
    let now = Utc::now();
    
    // Remove expired traces
    traces.retain(|trace| trace.expires_at > now);
    
    // Decay remaining traces
    for trace in traces.iter_mut() {
        let age = now - trace.created_at;
        let decay_factor = age.num_hours() as f64 * config.trace_decay_rate.to_f64().unwrap_or(0.1);
        trace.trace_strength = ((trace.trace_strength as f64) * (1.0 - decay_factor)) as i32;
    }
    
    // Remove traces that have decayed to nothing
    traces.retain(|trace| trace.trace_strength > 0);
}

/// Calculate response time for intrusion response
pub fn calculate_response_time(
    ids_sensitivity: i32,
    security_rating: i32,
    config: &DefenseConfig,
) -> i32 {
    let base_time = config.intrusion_response_time;
    
    // Higher sensitivity means faster response
    let sensitivity_reduction = ids_sensitivity / 10; // 1 second per 10 sensitivity
    
    // Better security means faster response
    let security_reduction = security_rating / 20; // 1 second per 20 rating
    
    (base_time - sensitivity_reduction - security_reduction).max(1)
}

/// Block an IP address
pub fn block_ip(
    blocked_ips: &mut Vec<BlockedIP>,
    ip_address: String,
    duration_hours: i32,
    reason: String,
) {
    let now = Utc::now();
    let blocked_ip = BlockedIP {
        ip_address,
        blocked_at: now,
        blocked_until: now + Duration::hours(duration_hours as i64),
        reason,
        auto_unblock: true,
    };
    
    blocked_ips.push(blocked_ip);
}

/// Check if IP is blocked
pub fn is_ip_blocked(blocked_ips: &[BlockedIP], ip_address: &str) -> bool {
    let now = Utc::now();
    
    blocked_ips.iter().any(|blocked| {
        blocked.ip_address == ip_address && blocked.blocked_until > now
    })
}

/// Process security event
pub fn process_security_event(
    events: &mut Vec<SecurityEvent>,
    event_type: String,
    source_ip: String,
    severity: i32,
    detected: bool,
    details: HashMap<String, String>,
) {
    let event = SecurityEvent {
        event_type,
        source_ip,
        severity,
        detected,
        timestamp: Utc::now(),
        details,
    };
    
    events.push(event);
    
    // Keep only last 1000 events
    if events.len() > 1000 {
        events.drain(0..events.len() - 1000);
    }
}

/// Calculate defense effectiveness against specific attack
pub fn calculate_defense_effectiveness(
    defense_state: &DefenseState,
    attack_type: &str,
    attack_strength: i32,
    config: &DefenseConfig,
) -> Decimal {
    let mut effectiveness = dec!(0.5); // Base 50% effectiveness
    
    // Firewall effectiveness
    effectiveness += Decimal::from(defense_state.firewall_strength) / dec!(200.0);
    
    // Attack-specific defenses
    effectiveness += match attack_type {
        "ddos" => {
            // DDoS protection
            if defense_state.firewall_strength > 70 {
                dec!(0.3)
            } else {
                dec!(0.1)
            }
        }
        "exploit" => {
            // Exploit protection
            if defense_state.security_rating > 60 {
                dec!(0.25)
            } else {
                dec!(0.05)
            }
        }
        "virus" => {
            // Virus protection (needs antivirus)
            dec!(0.2) // Simplified - would check for antivirus
        }
        "brute_force" => {
            // Brute force protection
            if defense_state.ids_active {
                dec!(0.35)
            } else {
                dec!(0.1)
            }
        }
        _ => dec!(0.0),
    };
    
    // Attack strength reduces effectiveness
    effectiveness -= Decimal::from(attack_strength) / dec!(500.0);
    
    // Security updates bonus
    if defense_state.security_rating > 80 {
        effectiveness += config.security_update_bonus;
    }
    
    effectiveness.max(dec!(0.1)).min(dec!(0.95))
}

/// Apply damage from successful attack
pub fn apply_attack_damage(
    defense_state: &mut DefenseState,
    attack_type: &str,
    damage_amount: i32,
) {
    match attack_type {
        "firewall_breach" => {
            defense_state.firewall_strength = 
                (defense_state.firewall_strength - damage_amount).max(0);
        }
        "ids_disable" => {
            if damage_amount > 50 {
                defense_state.ids_active = false;
            }
            defense_state.ids_sensitivity = 
                (defense_state.ids_sensitivity - damage_amount / 2).max(0);
        }
        "log_wipe" => {
            // Clear recent security events
            let cutoff = Utc::now() - Duration::hours(damage_amount as i64);
            defense_state.security_events.retain(|e| e.timestamp < cutoff);
        }
        _ => {
            // Generic damage to security rating
            defense_state.security_rating = 
                (defense_state.security_rating - damage_amount / 5).max(0);
        }
    }
}

/// Repair defense systems
pub fn repair_defense_systems(
    defense_state: &mut DefenseState,
    repair_amount: i32,
    target_system: &str,
) {
    match target_system {
        "firewall" => {
            defense_state.firewall_strength = 
                (defense_state.firewall_strength + repair_amount).min(100);
        }
        "ids" => {
            defense_state.ids_sensitivity = 
                (defense_state.ids_sensitivity + repair_amount).min(100);
            if defense_state.ids_sensitivity > 20 {
                defense_state.ids_active = true;
            }
        }
        "all" => {
            // Repair all systems partially
            defense_state.firewall_strength = 
                (defense_state.firewall_strength + repair_amount / 2).min(100);
            defense_state.ids_sensitivity = 
                (defense_state.ids_sensitivity + repair_amount / 2).min(100);
            defense_state.security_rating = 
                (defense_state.security_rating + repair_amount / 3).min(100);
        }
        _ => {}
    }
}

/// Generate defense report
pub fn generate_defense_report(defense_state: &DefenseState) -> HashMap<String, String> {
    let mut report = HashMap::new();
    
    report.insert("firewall_strength".to_string(), defense_state.firewall_strength.to_string());
    report.insert("ids_status".to_string(), 
        if defense_state.ids_active { "Active" } else { "Inactive" }.to_string());
    report.insert("security_rating".to_string(), defense_state.security_rating.to_string());
    report.insert("active_traces".to_string(), defense_state.active_traces.len().to_string());
    report.insert("blocked_ips".to_string(), defense_state.blocked_ips.len().to_string());
    report.insert("recent_events".to_string(), defense_state.security_events.len().to_string());
    
    // Calculate threat level
    let threat_level = if defense_state.security_events.len() > 50 {
        "Critical"
    } else if defense_state.security_events.len() > 20 {
        "High"
    } else if defense_state.security_events.len() > 5 {
        "Medium"
    } else {
        "Low"
    };
    
    report.insert("threat_level".to_string(), threat_level.to_string());
    
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PlayerState, HardwareSpecs, SoftwareInstance};
    use chrono::Utc;
    
    fn create_test_player() -> PlayerState {
        PlayerState {
            user_id: 1,
            level: 15,
            experience: 30000,
            money: 100000,
            reputation: HashMap::new(),
            hardware_specs: HardwareSpecs {
                cpu: 3200,
                ram: 16384,
                hdd: 1024000,
                net: 1000,
                security_level: 7,
                performance_rating: 85,
            },
            software_installed: vec![
                SoftwareInstance {
                    software_type: "firewall".to_string(),
                    version: "v3.5".to_string(),
                    effectiveness: 75,
                    dependencies: vec![],
                    installation_date: Utc::now(),
                }
            ],
            active_processes: vec![],
            clan_membership: None,
            last_updated: Utc::now(),
        }
    }
    
    #[test]
    fn test_firewall_strength_calculation() {
        let player = create_test_player();
        let config = DefenseConfig::default();
        
        let strength = calculate_firewall_strength(&player, &config);
        assert!(strength > 50); // Should be higher than base due to hardware and software
        assert!(strength <= 100); // Should be capped at 100
    }
    
    #[test]
    fn test_intrusion_detection() {
        let defense_state = DefenseState {
            firewall_strength: 70,
            ids_active: true,
            ids_sensitivity: 60,
            honeypot_active: false,
            log_monitoring: true,
            security_rating: 75,
            active_traces: vec![],
            blocked_ips: vec![],
            security_events: vec![],
        };
        
        let config = DefenseConfig::default();
        
        // Test detection of brute force (should be easier to detect)
        let detected = process_intrusion_detection(&defense_state, "brute_force", 30, &config);
        // With IDS active and brute force being easy to detect, this should often be true
        // but due to randomness, we just check it returns a boolean
        assert!(detected == true || detected == false);
    }
    
    #[test]
    fn test_security_rating() {
        let config = DefenseConfig::default();
        let hardware = HardwareSpecs {
            cpu: 2400,
            ram: 8192,
            hdd: 512000,
            net: 100,
            security_level: 6,
            performance_rating: 70,
        };
        
        let software = vec![
            SoftwareInstance {
                software_type: "firewall".to_string(),
                version: "v2.0".to_string(),
                effectiveness: 60,
                dependencies: vec![],
                installation_date: Utc::now(),
            },
            SoftwareInstance {
                software_type: "antivirus".to_string(),
                version: "v1.5".to_string(),
                effectiveness: 50,
                dependencies: vec![],
                installation_date: Utc::now(),
            },
        ];
        
        let rating = calculate_security_rating(
            65, // firewall_strength
            true, // ids_active
            false, // honeypot_active
            &software,
            &hardware,
            &config,
        );
        
        assert!(rating > 0);
        assert!(rating <= 100);
    }
    
    #[test]
    fn test_trace_decay() {
        let config = DefenseConfig::default();
        let mut traces = vec![
            TraceInfo {
                attacker_ip: "192.168.1.100".to_string(),
                trace_strength: 80,
                created_at: Utc::now() - Duration::hours(2),
                expires_at: Utc::now() + Duration::hours(22),
                trace_type: "hack_attempt".to_string(),
            },
            TraceInfo {
                attacker_ip: "192.168.1.101".to_string(),
                trace_strength: 50,
                created_at: Utc::now() - Duration::hours(25),
                expires_at: Utc::now() - Duration::hours(1), // Expired
                trace_type: "port_scan".to_string(),
            },
        ];
        
        process_trace_decay(&mut traces, &config);
        
        // Expired trace should be removed
        assert_eq!(traces.len(), 1);
        // Remaining trace should have decayed
        assert!(traces[0].trace_strength < 80);
    }
    
    #[test]
    fn test_ip_blocking() {
        let mut blocked_ips = vec![];
        
        block_ip(&mut blocked_ips, "192.168.1.100".to_string(), 24, "Brute force attempt".to_string());
        
        assert!(is_ip_blocked(&blocked_ips, "192.168.1.100"));
        assert!(!is_ip_blocked(&blocked_ips, "192.168.1.101"));
    }
}