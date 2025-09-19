//! Complete game simulation example demonstrating all mechanics
//!
//! This example shows how all the game mechanics work together to create
//! a complete gaming experience with accurate calculations and balance.

use he_game_mechanics::*;
use std::collections::HashMap;
use chrono::{Duration, Utc};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🎮 HackerExperience Game Mechanics Simulation");
    println!("============================================\n");

    // Initialize the game mechanics engine
    let engine = GameMechanicsEngine::new()?;
    println!("✅ Game mechanics engine initialized");

    // Validate all systems
    engine.validate()?;
    println!("✅ All systems validated");

    // Simulate a complete player session
    simulate_player_session(&engine).await?;

    Ok(())
}

async fn simulate_player_session(engine: &GameMechanicsEngine) -> Result<()> {
    println!("\n🎯 Starting Player Session Simulation");
    println!("=====================================");

    // 1. Player Initialization
    let player_id = 12345;
    let mut player_level = 1u32;
    let mut total_experience = 0u64;
    let mut hacking_skill = types::SkillLevel::new(25)?;
    let mut stealth_skill = types::SkillLevel::new(20)?;
    let mut money = types::Money::new(5000);

    println!("\n🚀 Player Profile:");
    println!("   Player ID: {}", player_id);
    println!("   Level: {}", player_level);
    println!("   Hacking Skill: {}", hacking_skill.value());
    println!("   Stealth Skill: {}", stealth_skill.value());
    println!("   Money: ${}", money.amount());

    // 2. Hardware Setup
    let player_hardware = types::HardwareSpec {
        cpu: 80,
        ram: 1600,
        hdd: 1000,
        net: 100,
    };

    let performance = engine.hardware().calculate_performance_rating(&player_hardware)?;
    println!("\n💻 Hardware Performance:");
    println!("   Overall Score: {}", performance.overall_score);
    println!("   CPU: {} | RAM: {} | HDD: {} | NET: {}", 
        performance.cpu_performance,
        performance.memory_performance, 
        performance.storage_performance,
        performance.network_performance
    );

    // 3. Software Setup
    let hacking_software = types::SoftwareSpec {
        name: "Advanced Cracker Pro".to_string(),
        version: "3.1".to_string(),
        size: 500,
        requirements: types::HardwareSpec { cpu: 60, ram: 1000, hdd: 400, net: 50 },
    };

    let software_effectiveness = engine.software().calculate_effectiveness(
        &hacking_software, &player_hardware, hacking_skill
    )?;
    println!("\n💿 Software Effectiveness:");
    println!("   Rating: {:.2}", software_effectiveness.effectiveness_rating);
    println!("   Hardware Compatibility: {:.2}", software_effectiveness.hardware_compatibility);
    println!("   User Skill Factor: {:.2}", software_effectiveness.user_skill_factor);

    // 4. Network Analysis
    let target_distance = 1200.0; // km
    let infrastructure_quality = 0.75;
    let network_congestion = 0.3;

    let latency = engine.network().calculate_latency(
        target_distance, infrastructure_quality, network_congestion
    )?;
    println!("\n🌐 Network Analysis:");
    println!("   Total Latency: {}ms", latency.total_latency_ms);
    println!("   Propagation: {}ms | Processing: {}ms | Congestion: {}ms",
        latency.propagation_delay, latency.processing_delay, latency.congestion_delay);

    // 5. Target System Defense Analysis
    let target_hardware = types::HardwareSpec {
        cpu: 120,
        ram: 2400,
        hdd: 1500,
        net: 150,
    };

    let firewall_software = defense::FirewallSoftware {
        name: "CorporateShield".to_string(),
        version: 4,
        effectiveness: 0.85,
        rule_processing_speed: 1200,
    };

    let firewall_strength = engine.defense().calculate_firewall_strength(
        &target_hardware, &firewall_software, 0.9, 0.95
    )?;
    println!("\n🛡️ Target Defense System:");
    println!("   Firewall Level: {:?}", firewall_strength.level);
    println!("   Numeric Strength: {}", firewall_strength.numeric_value);
    println!("   Blocking Rules: {}", firewall_strength.blocking_rules);

    let ids_software = defense::IDSSoftware {
        name: "SecureWatch Enterprise".to_string(),
        detection_accuracy: 0.82,
        rule_count: 7500,
        advanced_detection: true,
    };

    let topology = defense::NetworkTopology {
        complexity: defense::NetworkComplexity::Enterprise,
        node_count: 50,
        monitoring_points: 12,
    };

    let ids_effectiveness = engine.defense().calculate_ids_effectiveness(
        &target_hardware, &ids_software, &topology, Duration::hours(6)
    )?;
    println!("   IDS Detection Rate: {:.1}%", ids_effectiveness.detection_rate.percentage());
    println!("   IDS Response Time: {}s", ids_effectiveness.response_time.num_seconds());

    // 6. Hacking Attempt Simulation
    let attack_method = hacking::HackingMethod::Exploit;
    let tools = vec![hacking::HackingTool::ExpertCracker, hacking::HackingTool::CustomExploit];

    let difficulty = engine.hacking().calculate_difficulty(
        45, hacking_skill, firewall_strength.numeric_value, false
    )?;
    println!("\n⚔️ Hacking Attempt:");
    println!("   Target Difficulty: {:?} ({})", difficulty.level, difficulty.numeric_value);
    println!("   Attack Method: {:?}", attack_method);
    println!("   Tools: {:?}", tools);

    let success_rate = engine.hacking().calculate_success_rate(
        &difficulty, hacking_skill, &tools, Duration::minutes(8)
    )?;
    println!("   Success Probability: {:.1}%", success_rate.percentage());

    let hacking_time = engine.hacking().calculate_hacking_time(
        &difficulty, hacking_skill, player_hardware.cpu, &attack_method
    )?;
    println!("   Estimated Time: {}s", hacking_time.num_seconds());

    let detection_probability = engine.hacking().calculate_detection_probability(
        &attack_method, stealth_skill, ids_effectiveness.detection_rate.value() as u32 * 100,
        Duration::minutes(5)
    )?;
    println!("   Detection Risk: {:.1}%", detection_probability.percentage());

    // 7. Defense Response Simulation
    let defense_response = engine.defense().analyze_attack_attempt(
        &attack_method, &firewall_strength, &ids_effectiveness, 
        hacking_skill, Duration::minutes(5)
    )?;
    println!("\n🛡️ Defense Response:");
    println!("   Attack Blocked: {}", defense_response.blocked);
    println!("   Attack Detected: {}", defense_response.detected);
    println!("   Threat Level: {:?}", defense_response.threat_level);
    println!("   Actions: {:?}", defense_response.actions_taken);

    // 8. Experience and Skill Progression
    if utils::RandomUtils::roll_probability(success_rate) && !defense_response.blocked {
        println!("\n🎉 HACKING ATTEMPT SUCCESSFUL!");
        
        let experience_gain = engine.hacking().calculate_experience_gain(
            &difficulty, true, &attack_method, 1.0
        )?;
        total_experience += experience_gain as u64;
        println!("   Experience Gained: {}", experience_gain);

        let target_money = types::Money::new(25000);
        let money_stolen = engine.hacking().calculate_money_stolen(
            target_money, &difficulty, hacking_skill, 1.0
        )?;
        money = types::Money::new(money.amount() + money_stolen.amount());
        println!("   Money Stolen: ${}", money_stolen.amount());

        // Skill progression
        let skill_progression = engine.experience().calculate_skill_progression(
            hacking_skill, experience_gain, 0.2, 1.3
        )?;
        hacking_skill = skill_progression.new_skill_level;
        println!("   New Hacking Skill: {}", hacking_skill.value());

    } else {
        println!("\n💥 HACKING ATTEMPT FAILED!");
        if defense_response.blocked {
            println!("   Reason: Blocked by firewall");
        }
        if defense_response.detected {
            println!("   Reason: Detected by IDS");
        }
    }

    // 9. Financial System Simulation
    println!("\n💰 Financial Operations:");
    let transaction_fee = engine.financial().calculate_transaction_fee(money)?;
    println!("   Transaction Fee: ${}", transaction_fee.amount());

    let interest_earned = engine.financial().calculate_interest(money, 30)?;
    println!("   30-Day Interest: ${}", interest_earned.amount() - money.amount());

    // 10. Mission System
    let mission = mission::Mission {
        id: 1001,
        name: "Corporate Infiltration".to_string(),
        description: "Infiltrate corporate network and extract sensitive data".to_string(),
        mission_type: mission::MissionType::Complex,
        min_level: 1,
        required_skills: {
            let mut skills = HashMap::new();
            skills.insert(experience::SkillType::Hacking, types::SkillLevel::new(20)?);
            skills.insert(experience::SkillType::Networking, types::SkillLevel::new(15)?);
            skills
        },
        prerequisite_missions: vec![],
        base_difficulty: 35,
    };

    let mission_difficulty = engine.mission().calculate_mission_difficulty(
        mission.base_difficulty, player_level, mission.mission_type
    )?;
    println!("\n🎯 Mission: {}", mission.name);
    println!("   Difficulty: {}", mission_difficulty.numeric_value);

    let mission_rewards = engine.mission().calculate_mission_rewards(
        &mission_difficulty, player_level, 1.0
    )?;
    println!("   Potential Rewards:");
    println!("     Experience: {}", mission_rewards.experience);
    println!("     Money: ${}", mission_rewards.money.amount());
    println!("     Reputation: {}", mission_rewards.reputation);

    // 11. Clan System Simulation
    let member_contributions = vec![
        (200, 1.2), // Player contribution
        (150, 1.0), // Other members
        (300, 2.0),
        (175, 1.5),
        (125, 0.8),
    ];

    let clan_reputation = engine.clan().calculate_clan_reputation(
        800, &member_contributions, 6, 2, 25
    )?;
    println!("\n👥 Clan Status:");
    println!("   Reputation Level: {:?}", clan_reputation.level);
    println!("   Reputation Score: {}", clan_reputation.numeric_value);
    println!("   War Performance: {}", clan_reputation.war_performance_score);

    // 12. Performance Metrics
    let process_time = engine.process().calculate_execution_time(
        2.5, &player_hardware, 1.2
    )?;
    println!("\n⚡ Performance Metrics:");
    println!("   Process Execution Time: {}s", process_time.num_seconds());

    let resource_usage = engine.process().calculate_resource_usage(
        100, 0.85, 60.0, 100.0
    )?;
    println!("   Resource Usage: {}%", resource_usage);

    // 13. Final Player Status
    let final_player_level = engine.experience().calculate_level_from_experience(total_experience)?;
    println!("\n📊 Final Player Status:");
    println!("   Level: {}", final_player_level.current_level);
    println!("   Total Experience: {}", total_experience);
    println!("   Experience Progress: {:.1}%", final_player_level.progress_percentage);
    println!("   Current Money: ${}", money.amount());
    println!("   Hacking Skill: {}", hacking_skill.value());

    println!("\n🎮 Simulation Complete!");
    println!("=====================================");

    Ok(())
}

/// Demonstrate configuration customization
async fn demonstrate_configuration_system() -> Result<()> {
    println!("\n⚙️ Configuration System Demo");
    println!("============================");

    // Create custom configuration
    let mut custom_config = config::GameConfig::default();
    
    // Adjust game balance
    custom_config.hacking.base_success_rate = 0.35; // Slightly easier hacking
    custom_config.experience.exp_scaling_factor = 1.15; // Slower progression
    custom_config.financial.bank_interest_rate = 0.002; // Higher interest
    
    println!("Custom configuration created:");
    println!("   Base hacking success: {:.1}%", custom_config.hacking.base_success_rate * 100.0);
    println!("   Experience scaling: {:.2}", custom_config.experience.exp_scaling_factor);
    println!("   Bank interest rate: {:.1}%", custom_config.financial.bank_interest_rate * 100.0);

    let engine = GameMechanicsEngine::with_config(std::sync::Arc::new(custom_config))?;
    engine.validate()?;
    
    println!("✅ Custom configuration validated");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_simulation() {
        let engine = GameMechanicsEngine::new().unwrap();
        simulate_player_session(&engine).await.unwrap();
    }

    #[tokio::test]
    async fn test_configuration_demo() {
        demonstrate_configuration_system().await.unwrap();
    }
}