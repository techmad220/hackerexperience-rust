use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

/// Test game mechanics and calculations
#[test]
fn test_process_completion_time() {
    fn calculate_completion_time(
        process_type: &str,
        software_version: i32,
        target_difficulty: i32,
        server_performance: i32,
    ) -> i64 {
        let base_time = match process_type {
            "cracker" => 300, // 5 minutes base
            "hasher" => 180,  // 3 minutes base
            "encryptor" => 240, // 4 minutes base
            "decryptor" => 360, // 6 minutes base
            _ => 600, // 10 minutes default
        };
        
        let difficulty_modifier = (target_difficulty as f64 / 100.0).max(0.1);
        let software_modifier = 1.0 + (software_version as f64 * 0.1);
        let performance_modifier = 1.0 + (server_performance as f64 / 1000.0);
        
        let final_time = (base_time as f64 * difficulty_modifier / software_modifier / performance_modifier) as i64;
        final_time.max(10) // Minimum 10 seconds
    }
    
    // Test basic cracker process
    let time = calculate_completion_time("cracker", 1, 100, 1000);
    assert_eq!(time, 150); // 300 * 1.0 / 1.1 / 2.0 = 136.36 -> 136
    
    // Test with higher software version
    let time_improved = calculate_completion_time("cracker", 5, 100, 1000);
    assert!(time_improved < time);
    
    // Test with higher difficulty
    let time_harder = calculate_completion_time("cracker", 1, 200, 1000);
    assert!(time_harder > time);
    
    // Test minimum time constraint
    let time_minimum = calculate_completion_time("cracker", 100, 1, 10000);
    assert_eq!(time_minimum, 10);
}

#[test]
fn test_experience_calculation() {
    fn calculate_experience(process_type: &str, success: bool, target_level: i32) -> i32 {
        let base_exp = match process_type {
            "cracker" => 50,
            "hasher" => 30,
            "virus" => 100,
            "research" => 75,
            _ => 25,
        };
        
        let level_multiplier = (target_level as f64 / 10.0).max(1.0);
        let success_multiplier = if success { 1.0 } else { 0.3 };
        
        (base_exp as f64 * level_multiplier * success_multiplier) as i32
    }
    
    // Test successful cracker process
    let exp = calculate_experience("cracker", true, 50);
    assert_eq!(exp, 250); // 50 * 5.0 * 1.0
    
    // Test failed process
    let exp_failed = calculate_experience("cracker", false, 50);
    assert_eq!(exp_failed, 75); // 50 * 5.0 * 0.3
    
    // Test low level target
    let exp_low = calculate_experience("virus", true, 5);
    assert_eq!(exp_low, 100); // 100 * 1.0 * 1.0
}

#[test]
fn test_money_calculation() {
    fn calculate_mission_reward(
        mission_type: &str,
        difficulty: i32,
        player_reputation: i32,
    ) -> i32 {
        let base_reward = match mission_type {
            "hack" => 1000,
            "ddos" => 500,
            "virus" => 2000,
            "data_theft" => 1500,
            _ => 750,
        };
        
        let difficulty_multiplier = (difficulty as f64 / 100.0).max(0.5);
        let reputation_bonus = (player_reputation as f64 / 1000.0).min(2.0);
        
        (base_reward as f64 * difficulty_multiplier * (1.0 + reputation_bonus)) as i32
    }
    
    // Test basic hack mission
    let reward = calculate_mission_reward("hack", 100, 500);
    assert_eq!(reward, 1500); // 1000 * 1.0 * 1.5
    
    // Test high difficulty
    let reward_hard = calculate_mission_reward("hack", 200, 500);
    assert!(reward_hard > reward);
    
    // Test high reputation bonus
    let reward_rep = calculate_mission_reward("hack", 100, 2000);
    assert_eq!(reward_rep, 3000); // 1000 * 1.0 * 3.0 (capped at 2.0 bonus)
}

#[test]
fn test_server_scanning() {
    fn generate_network_range(base_ip: &str) -> Vec<String> {
        let parts: Vec<&str> = base_ip.split('.').collect();
        if parts.len() != 4 {
            return vec![];
        }
        
        let base = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
        let mut ips = Vec::new();
        
        for i in 1..255 {
            ips.push(format!("{}.{}", base, i));
        }
        
        ips
    }
    
    fn scan_for_servers(network: &[String], active_servers: &[String]) -> Vec<String> {
        network.iter()
            .filter(|ip| active_servers.contains(ip))
            .cloned()
            .collect()
    }
    
    let network = generate_network_range("192.168.1.1");
    assert_eq!(network.len(), 254);
    assert_eq!(network[0], "192.168.1.1");
    assert_eq!(network[253], "192.168.1.254");
    
    let active_servers = vec![
        "192.168.1.100".to_string(),
        "192.168.1.101".to_string(),
        "192.168.1.150".to_string(),
    ];
    
    let discovered = scan_for_servers(&network, &active_servers);
    assert_eq!(discovered.len(), 3);
    assert!(discovered.contains(&"192.168.1.100".to_string()));
}

#[test]
fn test_software_upgrade_cost() {
    fn calculate_upgrade_cost(current_version: i32, software_type: &str) -> i32 {
        let base_cost = match software_type {
            "cracker" => 1000,
            "hasher" => 800,
            "encryptor" => 1200,
            "firewall" => 1500,
            "antivirus" => 1800,
            _ => 1000,
        };
        
        let version_multiplier = (current_version as f64).powf(1.5);
        (base_cost as f64 * version_multiplier) as i32
    }
    
    // Test version 1 to 2
    let cost_v2 = calculate_upgrade_cost(1, "cracker");
    assert_eq!(cost_v2, 1000); // 1000 * 1^1.5
    
    // Test version 2 to 3
    let cost_v3 = calculate_upgrade_cost(2, "cracker");
    assert_eq!(cost_v3, 2828); // 1000 * 2^1.5 ≈ 2828
    
    // Test expensive software
    let cost_antivirus = calculate_upgrade_cost(1, "antivirus");
    assert_eq!(cost_antivirus, 1800);
}

#[test]
fn test_clan_reputation_system() {
    fn calculate_clan_reputation_change(
        action: &str,
        success: bool,
        target_clan_reputation: i32,
    ) -> i32 {
        let base_change = match action {
            "war_victory" => 100,
            "war_defeat" => -50,
            "member_achievement" => 25,
            "member_scandal" => -30,
            _ => 0,
        };
        
        let success_multiplier = if success { 1.0 } else { 0.0 };
        let target_multiplier = (target_clan_reputation as f64 / 1000.0).max(0.5).min(2.0);
        
        (base_change as f64 * success_multiplier * target_multiplier) as i32
    }
    
    // Test war victory against strong clan
    let rep_change = calculate_clan_reputation_change("war_victory", true, 2000);
    assert_eq!(rep_change, 200); // 100 * 1.0 * 2.0
    
    // Test failed war
    let rep_loss = calculate_clan_reputation_change("war_victory", false, 2000);
    assert_eq!(rep_loss, 0); // 100 * 0.0 * 2.0
    
    // Test scandal
    let scandal_loss = calculate_clan_reputation_change("member_scandal", true, 1000);
    assert_eq!(scandal_loss, -30); // -30 * 1.0 * 1.0
}

#[test]
fn test_cooldown_system() {
    use std::time::{Duration, Instant};
    
    struct CooldownManager {
        cooldowns: HashMap<String, Instant>,
    }
    
    impl CooldownManager {
        fn new() -> Self {
            Self {
                cooldowns: HashMap::new(),
            }
        }
        
        fn set_cooldown(&mut self, key: &str, duration: Duration) {
            self.cooldowns.insert(key.to_string(), Instant::now() + duration);
        }
        
        fn is_on_cooldown(&self, key: &str) -> bool {
            if let Some(cooldown_end) = self.cooldowns.get(key) {
                Instant::now() < *cooldown_end
            } else {
                false
            }
        }
        
        fn remaining_cooldown(&self, key: &str) -> Option<Duration> {
            if let Some(cooldown_end) = self.cooldowns.get(key) {
                let now = Instant::now();
                if now < *cooldown_end {
                    Some(cooldown_end.duration_since(now))
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
    
    let mut cooldowns = CooldownManager::new();
    
    // Test setting cooldown
    cooldowns.set_cooldown("attack", Duration::from_secs(300));
    assert!(cooldowns.is_on_cooldown("attack"));
    
    // Test no cooldown for different action
    assert!(!cooldowns.is_on_cooldown("defend"));
    
    // Test remaining time
    let remaining = cooldowns.remaining_cooldown("attack");
    assert!(remaining.is_some());
    assert!(remaining.unwrap() <= Duration::from_secs(300));
}