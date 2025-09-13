//! Integration tests for game mechanics

use he_game_mechanics::*;
use std::sync::Arc;

#[tokio::test]
async fn test_complete_game_mechanics_engine() {
    let engine = GameMechanicsEngine::new().expect("Failed to create engine");
    
    // Validate all systems
    engine.validate().expect("Engine validation failed");
    
    // Test hacking system
    let skill = types::SkillLevel::new(50).unwrap();
    let difficulty = engine.hacking().calculate_difficulty(30, skill, 50, false).unwrap();
    assert!(difficulty.numeric_value > 0);
    
    // Test experience system  
    let player_level = engine.experience().calculate_level_from_experience(5000).unwrap();
    assert!(player_level.current_level > 0);
    
    // Test financial system
    let interest = engine.financial().calculate_interest(types::Money::new(1000), 12).unwrap();
    assert!(interest.amount() > 1000);
    
    // Test defense system
    let hardware = types::HardwareSpec { cpu: 100, ram: 2000, hdd: 1000, net: 100 };
    let firewall_software = defense::FirewallSoftware {
        name: "TestFirewall".to_string(),
        version: 5,
        effectiveness: 0.8,
        rule_processing_speed: 1000,
    };
    let firewall_strength = engine.defense().calculate_firewall_strength(
        &hardware, &firewall_software, 0.9, 0.95
    ).unwrap();
    assert!(firewall_strength.numeric_value > 0);
}

#[tokio::test]
async fn test_hacking_to_defense_interaction() {
    let engine = GameMechanicsEngine::new().unwrap();
    
    let attacker_skill = types::SkillLevel::new(70).unwrap();
    let hardware = types::HardwareSpec { cpu: 80, ram: 1500, hdd: 500, net: 100 };
    
    // Calculate hacking attempt
    let difficulty = engine.hacking().calculate_difficulty(40, attacker_skill, 60, false).unwrap();
    let tools = vec![hacking::HackingTool::AdvancedCracker];
    let success_rate = engine.hacking().calculate_success_rate(
        &difficulty,
        attacker_skill,
        &tools,
        chrono::Duration::seconds(120),
    ).unwrap();
    
    // Calculate defense response
    let firewall_software = defense::FirewallSoftware {
        name: "DefenseWall".to_string(),
        version: 3,
        effectiveness: 0.7,
        rule_processing_speed: 800,
    };
    let firewall_strength = engine.defense().calculate_firewall_strength(
        &hardware, &firewall_software, 0.8, 0.9
    ).unwrap();
    
    let ids_software = defense::IDSSoftware {
        name: "SecurityMonitor".to_string(),
        detection_accuracy: 0.75,
        rule_count: 3000,
        advanced_detection: true,
    };
    let topology = defense::NetworkTopology {
        complexity: defense::NetworkComplexity::Standard,
        node_count: 15,
        monitoring_points: 3,
    };
    let ids_effectiveness = engine.defense().calculate_ids_effectiveness(
        &hardware, &ids_software, &topology, chrono::Duration::days(2)
    ).unwrap();
    
    let defense_response = engine.defense().analyze_attack_attempt(
        &hacking::HackingMethod::BruteForce,
        &firewall_strength,
        &ids_effectiveness,
        attacker_skill,
        chrono::Duration::minutes(5),
    ).unwrap();
    
    // The interaction should produce realistic results
    assert!(success_rate.value() > 0.0 && success_rate.value() < 1.0);
    assert!(defense_response.threat_level != defense::ThreatLevel::Low || attacker_skill.value() < 30);
}

#[tokio::test]
async fn test_experience_and_skill_progression() {
    let engine = GameMechanicsEngine::new().unwrap();
    
    let current_skill = types::SkillLevel::new(25).unwrap();
    
    // Test skill progression
    let progression = engine.experience().calculate_skill_progression(
        current_skill, 150, 0.3, 1.2
    ).unwrap();
    
    assert!(progression.experience_gained > 0);
    assert!(progression.new_skill_level.value() >= current_skill.value());
    
    // Test learning efficiency
    let practice_sessions = vec![
        experience::PracticeSession {
            timestamp: chrono::Utc::now() - chrono::Duration::days(2),
            skill_type: experience::SkillType::Hacking,
            duration: chrono::Duration::hours(3),
            performance_score: 0.75,
            difficulty_level: 4,
        },
        experience::PracticeSession {
            timestamp: chrono::Utc::now() - chrono::Duration::days(1),
            skill_type: experience::SkillType::Hacking,
            duration: chrono::Duration::hours(2),
            performance_score: 0.85,
            difficulty_level: 5,
        },
    ];
    
    let efficiency = engine.experience().calculate_learning_efficiency(
        current_skill, &practice_sessions, None
    ).unwrap();
    
    assert!(efficiency.total_efficiency > efficiency.base_rate);
    assert!(efficiency.practice_bonus > 0.0);
}

#[tokio::test]
async fn test_financial_system_integration() {
    let engine = GameMechanicsEngine::new().unwrap();
    
    let principal = types::Money::new(10000);
    
    // Test compound interest over time
    let interest_1_month = engine.financial().calculate_interest(principal, 30).unwrap();
    let interest_1_year = engine.financial().calculate_interest(principal, 365).unwrap();
    
    assert!(interest_1_month.amount() > principal.amount());
    assert!(interest_1_year.amount() > interest_1_month.amount());
    
    // Test transaction fees
    let small_transaction = types::Money::new(100);
    let large_transaction = types::Money::new(50000);
    
    let small_fee = engine.financial().calculate_transaction_fee(small_transaction).unwrap();
    let large_fee = engine.financial().calculate_transaction_fee(large_transaction).unwrap();
    
    assert!(small_fee.amount() > 0);
    assert!(large_fee.amount() > small_fee.amount());
    
    // Test market dynamics
    let base_price = 100.0;
    let low_supply_high_demand = engine.financial().calculate_market_price(base_price, 10.0, 100.0).unwrap();
    let high_supply_low_demand = engine.financial().calculate_market_price(base_price, 100.0, 10.0).unwrap();
    
    assert!(low_supply_high_demand > base_price);
    assert!(high_supply_low_demand < base_price);
}

#[tokio::test]
async fn test_mission_system() {
    let engine = GameMechanicsEngine::new().unwrap();
    
    // Test difficulty scaling
    let base_difficulty = 20;
    let player_level = 15;
    
    let tutorial_difficulty = engine.mission().calculate_mission_difficulty(
        base_difficulty, player_level, mission::MissionType::Tutorial
    ).unwrap();
    
    let expert_difficulty = engine.mission().calculate_mission_difficulty(
        base_difficulty, player_level, mission::MissionType::Expert
    ).unwrap();
    
    assert!(tutorial_difficulty.numeric_value < expert_difficulty.numeric_value);
    
    // Test reward calculation
    let tutorial_rewards = engine.mission().calculate_mission_rewards(
        &tutorial_difficulty, player_level, 1.0
    ).unwrap();
    
    let expert_rewards = engine.mission().calculate_mission_rewards(
        &expert_difficulty, player_level, 1.0
    ).unwrap();
    
    assert!(expert_rewards.experience > tutorial_rewards.experience);
    assert!(expert_rewards.money.amount() > tutorial_rewards.money.amount());
}

#[tokio::test]
async fn test_clan_warfare() {
    let engine = GameMechanicsEngine::new().unwrap();
    
    // Test clan reputation calculation
    let member_contributions = vec![
        (150, 1.0), // Normal member
        (200, 1.5), // Officer
        (100, 0.8), // Recruit
        (300, 2.0), // Leader
    ];
    
    let reputation = engine.clan().calculate_clan_reputation(
        500,
        &member_contributions,
        8,  // victories
        3,  // defeats
        50, // penalties
    ).unwrap();
    
    assert!(reputation.numeric_value > 500);
    assert!(reputation.war_performance_score > 0);
    
    // Test warfare effectiveness
    let warfare_result = engine.clan().calculate_warfare_effectiveness(
        1200, // attacking power
        1000, // defending power
        1.2,  // strategy bonus
        0.9,  // coordination
    ).unwrap();
    
    assert!(warfare_result.power_ratio > 1.0);
    assert!(warfare_result.success_probability.value() > 0.5);
}

#[tokio::test]
async fn test_network_and_process_systems() {
    let engine = GameMechanicsEngine::new().unwrap();
    
    // Test network latency calculation
    let latency = engine.network().calculate_latency(
        1500.0, // 1500km distance
        0.85,   // good infrastructure
        0.3,    // moderate congestion
    ).unwrap();
    
    assert!(latency.total_latency_ms > latency.propagation_delay);
    assert!(latency.congestion_delay > 0);
    
    // Test connection speed calculation
    let connection = engine.network().calculate_connection_speed(
        1000, // 1000 Mbps base
        0.4,  // 40% network load
        800,  // 800 Mbps hardware limit
    ).unwrap();
    
    assert!(connection.effective_speed <= connection.nominal_speed);
    assert!(connection.effective_speed <= 800); // Hardware limited
    
    // Test process execution time
    let hardware = types::HardwareSpec { cpu: 120, ram: 3000, hdd: 2000, net: 150 };
    let execution_time = engine.process().calculate_execution_time(
        2.5, // complexity
        &hardware,
        1.3, // optimization
    ).unwrap();
    
    assert!(execution_time.num_seconds() > 0);
    
    // Test resource usage
    let usage = engine.process().calculate_resource_usage(
        100,  // base consumption
        0.85, // efficiency
        60.0, // current load
        100.0, // max load
    ).unwrap();
    
    assert!(usage > 100); // Should be higher due to load
}

#[tokio::test]
async fn test_hardware_software_compatibility() {
    let engine = GameMechanicsEngine::new().unwrap();
    
    let hardware = types::HardwareSpec { cpu: 80, ram: 1600, hdd: 1000, net: 100 };
    let software = types::SoftwareSpec {
        name: "TestSoftware".to_string(),
        version: "2.1".to_string(),
        size: 500,
        requirements: types::HardwareSpec { cpu: 60, ram: 1000, hdd: 400, net: 50 },
    };
    
    // Test hardware performance rating
    let performance = engine.hardware().calculate_performance_rating(&hardware).unwrap();
    assert!(performance.overall_score > 0);
    assert_eq!(performance.cpu_performance, hardware.cpu);
    
    // Test compatibility check
    let compatibility = engine.hardware().check_compatibility(&hardware, &software.requirements).unwrap();
    assert!(compatibility.compatible);
    assert!(compatibility.missing_requirements.is_empty());
    assert!(compatibility.compatibility_score >= 1.0);
    
    // Test software effectiveness
    let skill = types::SkillLevel::new(65).unwrap();
    let effectiveness = engine.software().calculate_effectiveness(&software, &hardware, skill).unwrap();
    assert!(effectiveness.effectiveness_rating > 0.0);
    assert!(effectiveness.hardware_compatibility > 0.0);
    assert!(effectiveness.user_skill_factor > 0.5);
    
    // Test upgrade cost calculation
    let target_hardware = types::HardwareSpec { cpu: 120, ram: 2400, hdd: 1500, net: 150 };
    let upgrade_cost = engine.hardware().calculate_upgrade_cost(&hardware, &target_hardware).unwrap();
    assert!(upgrade_cost.amount() > 0);
}

#[tokio::test]
async fn test_formula_accuracy() {
    // Test that our formulas produce consistent and reasonable results
    let config = config::GameConfig::default();
    
    // Test success probability formula
    let prob1 = formulas::Formulas::success_probability(0.3, 50, 25, 1.2, 1.0).unwrap();
    let prob2 = formulas::Formulas::success_probability(0.3, 75, 25, 1.2, 1.0).unwrap();
    
    // Higher skill should give better success rate
    assert!(prob2.value() > prob1.value());
    
    // Test experience requirements
    let exp_level_5 = formulas::Formulas::experience_required(5, 1000, 1.1).unwrap();
    let exp_level_10 = formulas::Formulas::experience_required(10, 1000, 1.1).unwrap();
    
    // Higher levels should require more experience
    assert!(exp_level_10 > exp_level_5);
    
    // Test compound interest
    let interest_1 = formulas::Formulas::compound_interest(1000, 0.05, 12).unwrap();
    let interest_2 = formulas::Formulas::compound_interest(1000, 0.10, 12).unwrap();
    
    // Higher interest rate should yield more money
    assert!(interest_2 > interest_1);
}

#[tokio::test]
async fn test_engine_configuration() {
    // Test custom configuration
    let mut custom_config = config::GameConfig::default();
    custom_config.hacking.base_success_rate = 0.4;
    custom_config.experience.base_exp_required = 2000;
    
    let engine = GameMechanicsEngine::with_config(Arc::new(custom_config)).unwrap();
    
    // Verify configuration is applied
    assert_eq!(engine.config().hacking.base_success_rate, 0.4);
    assert_eq!(engine.config().experience.base_exp_required, 2000);
    
    // Test validation
    engine.validate().expect("Custom configuration should be valid");
}