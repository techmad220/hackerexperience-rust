use anyhow::{anyhow, Result};
//! Benchmarks for game mechanics performance

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use he_game_mechanics::*;
use std::sync::Arc;

fn benchmark_hacking_calculations(c: &mut Criterion) {
    let engine = GameMechanicsEngine::new().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
    let skill = types::SkillLevel::new(60).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
    
    c.bench_function("hacking_difficulty_calculation", |b| {
        b.iter(|| {
            engine.hacking().calculate_difficulty(
                black_box(35),
                black_box(skill),
                black_box(55),
                black_box(false),
            ).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
    
    let difficulty = engine.hacking().calculate_difficulty(35, skill, 55, false).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
    let tools = vec![hacking::HackingTool::AdvancedCracker];
    
    c.bench_function("hacking_success_rate_calculation", |b| {
        b.iter(|| {
            engine.hacking().calculate_success_rate(
                black_box(&difficulty),
                black_box(skill),
                black_box(&tools),
                black_box(chrono::Duration::seconds(120)),
            ).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
    
    c.bench_function("hacking_time_calculation", |b| {
        b.iter(|| {
            engine.hacking().calculate_hacking_time(
                black_box(&difficulty),
                black_box(skill),
                black_box(100),
                black_box(&hacking::HackingMethod::BruteForce),
            ).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
}

fn benchmark_defense_calculations(c: &mut Criterion) {
    let engine = GameMechanicsEngine::new().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
    let hardware = types::HardwareSpec { cpu: 100, ram: 2000, hdd: 1000, net: 100 };
    let firewall_software = defense::FirewallSoftware {
        name: "BenchFirewall".to_string(),
        version: 5,
        effectiveness: 0.8,
        rule_processing_speed: 1000,
    };
    
    c.bench_function("firewall_strength_calculation", |b| {
        b.iter(|| {
            engine.defense().calculate_firewall_strength(
                black_box(&hardware),
                black_box(&firewall_software),
                black_box(0.9),
                black_box(0.95),
            ).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
    
    let ids_software = defense::IDSSoftware {
        name: "BenchIDS".to_string(),
        detection_accuracy: 0.85,
        rule_count: 5000,
        advanced_detection: true,
    };
    let topology = defense::NetworkTopology {
        complexity: defense::NetworkComplexity::Standard,
        node_count: 20,
        monitoring_points: 5,
    };
    
    c.bench_function("ids_effectiveness_calculation", |b| {
        b.iter(|| {
            engine.defense().calculate_ids_effectiveness(
                black_box(&hardware),
                black_box(&ids_software),
                black_box(&topology),
                black_box(chrono::Duration::days(1)),
            ).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
}

fn benchmark_experience_calculations(c: &mut Criterion) {
    let engine = GameMechanicsEngine::new().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
    
    c.bench_function("level_from_experience", |b| {
        b.iter(|| {
            engine.experience().calculate_level_from_experience(black_box(25000)).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
    
    c.bench_function("experience_required_for_level", |b| {
        b.iter(|| {
            engine.experience().experience_required_for_level(black_box(25)).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
    
    let current_skill = types::SkillLevel::new(45).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
    c.bench_function("skill_progression", |b| {
        b.iter(|| {
            engine.experience().calculate_skill_progression(
                black_box(current_skill),
                black_box(120),
                black_box(0.25),
                black_box(1.3),
            ).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
    
    let skill_mastery_calc = |b: &mut criterion::Bencher| {
        b.iter(|| {
            engine.experience().calculate_skill_mastery(
                black_box(current_skill),
                black_box(chrono::Duration::hours(750)),
                black_box(85),
            ).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    };
    c.bench_function("skill_mastery_calculation", skill_mastery_calc);
}

fn benchmark_financial_calculations(c: &mut Criterion) {
    let engine = GameMechanicsEngine::new().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
    
    c.bench_function("compound_interest", |b| {
        b.iter(|| {
            engine.financial().calculate_interest(
                black_box(types::Money::new(50000)),
                black_box(365),
            ).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
    
    c.bench_function("transaction_fee", |b| {
        b.iter(|| {
            engine.financial().calculate_transaction_fee(
                black_box(types::Money::new(15000))
            ).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
    
    c.bench_function("market_price", |b| {
        b.iter(|| {
            engine.financial().calculate_market_price(
                black_box(250.0),
                black_box(75.0),
                black_box(125.0),
            ).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
}

fn benchmark_formula_calculations(c: &mut Criterion) {
    c.bench_function("success_probability_formula", |b| {
        b.iter(|| {
            formulas::Formulas::success_probability(
                black_box(0.35),
                black_box(65),
                black_box(30),
                black_box(1.25),
                black_box(1.0),
            ).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
    
    c.bench_function("experience_required_formula", |b| {
        b.iter(|| {
            formulas::Formulas::experience_required(
                black_box(20),
                black_box(1500),
                black_box(1.15),
            ).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
    
    c.bench_function("process_time_formula", |b| {
        b.iter(|| {
            formulas::Formulas::process_time(
                black_box(180),
                black_box(2.3),
                black_box(95),
                black_box(2500),
                black_box(1.4),
            ).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
    
    c.bench_function("combat_damage_formula", |b| {
        b.iter(|| {
            formulas::Formulas::combat_damage(
                black_box(150),
                black_box(78),
                black_box(65),
                black_box(1.35),
                black_box(1.15),
                black_box(0.95),
            ).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
}

fn benchmark_network_calculations(c: &mut Criterion) {
    let engine = GameMechanicsEngine::new().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
    
    c.bench_function("network_latency", |b| {
        b.iter(|| {
            engine.network().calculate_latency(
                black_box(2500.0),
                black_box(0.8),
                black_box(0.4),
            ).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
    
    c.bench_function("connection_speed", |b| {
        b.iter(|| {
            engine.network().calculate_connection_speed(
                black_box(1000),
                black_box(0.6),
                black_box(800),
            ).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
}

fn benchmark_clan_calculations(c: &mut Criterion) {
    let engine = GameMechanicsEngine::new().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
    
    let contributions = vec![
        (150, 1.0), (200, 1.5), (100, 0.8), (300, 2.0), (175, 1.2)
    ];
    
    c.bench_function("clan_reputation", |b| {
        b.iter(|| {
            engine.clan().calculate_clan_reputation(
                black_box(750),
                black_box(&contributions),
                black_box(12),
                black_box(4),
                black_box(75),
            ).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
    
    c.bench_function("warfare_effectiveness", |b| {
        b.iter(|| {
            engine.clan().calculate_warfare_effectiveness(
                black_box(1400),
                black_box(1100),
                black_box(1.3),
                black_box(0.85),
            ).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
}

fn benchmark_hardware_software(c: &mut Criterion) {
    let engine = GameMechanicsEngine::new().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
    let hardware = types::HardwareSpec { cpu: 110, ram: 2800, hdd: 1200, net: 120 };
    let software = types::SoftwareSpec {
        name: "BenchSoftware".to_string(),
        version: "3.2".to_string(),
        size: 750,
        requirements: types::HardwareSpec { cpu: 80, ram: 1500, hdd: 500, net: 80 },
    };
    
    c.bench_function("hardware_performance_rating", |b| {
        b.iter(|| {
            engine.hardware().calculate_performance_rating(black_box(&hardware)).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
    
    c.bench_function("hardware_compatibility_check", |b| {
        b.iter(|| {
            engine.hardware().check_compatibility(
                black_box(&hardware),
                black_box(&software.requirements),
            ).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
    
    let skill = types::SkillLevel::new(70).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
    c.bench_function("software_effectiveness", |b| {
        b.iter(|| {
            engine.software().calculate_effectiveness(
                black_box(&software),
                black_box(&hardware),
                black_box(skill),
            ).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
}

fn benchmark_engine_creation_and_validation(c: &mut Criterion) {
    c.bench_function("engine_creation", |b| {
        b.iter(|| {
            black_box(GameMechanicsEngine::new().map_err(|e| anyhow::anyhow!("Error: {}", e))?)
        })
    });
    
    let engine = GameMechanicsEngine::new().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
    c.bench_function("engine_validation", |b| {
        b.iter(|| {
            black_box(engine.validate()).map_err(|e| anyhow::anyhow!("Error: {}", e))?
        })
    });
    
    c.bench_function("config_creation", |b| {
        b.iter(|| {
            black_box(config::GameConfig::default())
        })
    });
}

fn benchmark_utils_functions(c: &mut Criterion) {
    c.bench_function("random_range", |b| {
        b.iter(|| {
            black_box(utils::RandomUtils::random_range(1.0, 100.0))
        })
    });
    
    c.bench_function("weighted_average", |b| {
        let data = vec![(10.0, 2.0), (20.0, 3.0), (30.0, 1.5), (15.0, 2.5)];
        b.iter(|| {
            black_box(utils::DataUtils::weighted_average(black_box(&data)))
        })
    });
    
    c.bench_function("diminishing_returns", |b| {
        b.iter(|| {
            black_box(utils::BalanceUtils::diminishing_returns(black_box(50.0), black_box(0.1)))
        })
    });
    
    c.bench_function("soft_cap", |b| {
        b.iter(|| {
            black_box(utils::BalanceUtils::soft_cap(black_box(150.0), black_box(100.0), black_box(0.02)))
        })
    });
}

criterion_group!(
    benches,
    benchmark_hacking_calculations,
    benchmark_defense_calculations,
    benchmark_experience_calculations,
    benchmark_financial_calculations,
    benchmark_formula_calculations,
    benchmark_network_calculations,
    benchmark_clan_calculations,
    benchmark_hardware_software,
    benchmark_engine_creation_and_validation,
    benchmark_utils_functions,
);

criterion_main!(benches);