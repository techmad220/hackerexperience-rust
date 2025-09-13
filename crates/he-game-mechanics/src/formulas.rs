//! Core mathematical formulas and algorithms used throughout the game
//!
//! This module contains all the fundamental mathematical calculations that
//! power the game mechanics, ensuring 1:1 parity with the original game.

use crate::types::*;
use crate::{GameMechanicsError, Result};
use std::f64::consts::{E, PI};

/// Mathematical constants used in game calculations
pub mod constants {
    /// Golden ratio for natural progression curves
    pub const GOLDEN_RATIO: f64 = 1.618033988749895;
    
    /// Natural logarithm base for exponential calculations
    pub const E: f64 = std::f64::consts::E;
    
    /// Pi for trigonometric calculations
    pub const PI: f64 = std::f64::consts::PI;
    
    /// Base 2 logarithm for binary-like calculations
    pub const LOG2: f64 = 0.6931471805599453;
}

/// Core formula implementations
pub struct Formulas;

impl Formulas {
    /// Calculate success probability based on multiple factors
    /// Formula: base_rate * (1 + skill_bonus) * difficulty_modifier * equipment_bonus
    pub fn success_probability(
        base_rate: f64,
        skill_level: u8,
        difficulty: u32,
        equipment_bonus: f64,
        luck_factor: f64,
    ) -> Result<Probability> {
        if base_rate < 0.0 || base_rate > 1.0 {
            return Err(GameMechanicsError::InvalidParameter {
                param: "base_rate".to_string(),
                value: base_rate.to_string(),
            });
        }

        // Skill bonus: logarithmic scaling to prevent overpowering
        let skill_bonus = (skill_level as f64 / 100.0).ln() / constants::LOG2;
        
        // Difficulty modifier: exponential decay
        let difficulty_modifier = (-0.1 * difficulty as f64).exp();
        
        // Equipment bonus should be reasonable
        let equipment_bonus = equipment_bonus.max(0.0).min(2.0);
        
        // Combine all factors
        let raw_probability = base_rate * (1.0 + skill_bonus) * difficulty_modifier * equipment_bonus * luck_factor;
        
        // Clamp to valid probability range
        let clamped = raw_probability.max(0.0).min(1.0);
        
        Probability::new(clamped)
    }

    /// Calculate experience required for a given level
    /// Formula: base_exp * (level^scaling_factor) * difficulty_multiplier
    pub fn experience_required(level: u32, base_exp: u64, scaling_factor: f64) -> Result<u64> {
        if level == 0 {
            return Ok(0);
        }
        
        if scaling_factor <= 1.0 {
            return Err(GameMechanicsError::InvalidParameter {
                param: "scaling_factor".to_string(),
                value: scaling_factor.to_string(),
            });
        }
        
        let raw_exp = base_exp as f64 * (level as f64).powf(scaling_factor);
        
        // Check for overflow
        if raw_exp > u64::MAX as f64 {
            return Err(GameMechanicsError::CalculationOverflow {
                operation: "experience_required".to_string(),
            });
        }
        
        Ok(raw_exp as u64)
    }

    /// Calculate process time based on complexity and resources
    /// Formula: base_time * complexity_factor / (resource_efficiency + 1)
    pub fn process_time(
        base_time: u64,
        complexity: f64,
        cpu_power: u32,
        ram_available: u32,
        optimization_level: f64,
    ) -> Result<u64> {
        if complexity <= 0.0 {
            return Err(GameMechanicsError::InvalidParameter {
                param: "complexity".to_string(),
                value: complexity.to_string(),
            });
        }
        
        // Resource efficiency calculation
        let cpu_efficiency = (cpu_power as f64 / 100.0).ln() + 1.0;
        let ram_efficiency = (ram_available as f64 / 1000.0).sqrt();
        let total_efficiency = cpu_efficiency * ram_efficiency * optimization_level;
        
        let raw_time = base_time as f64 * complexity / total_efficiency.max(0.1);
        
        // Minimum time is 1 second
        Ok((raw_time as u64).max(1))
    }

    /// Calculate damage/effectiveness in combat scenarios
    /// Formula: base_damage * attacker_skill / defender_skill * equipment_modifier * random_variance
    pub fn combat_damage(
        base_damage: u32,
        attacker_skill: u8,
        defender_skill: u8,
        attacker_equipment: f64,
        defender_equipment: f64,
        random_factor: f64,
    ) -> Result<u32> {
        let skill_ratio = (attacker_skill as f64 + 1.0) / (defender_skill as f64 + 1.0);
        let equipment_ratio = attacker_equipment / defender_equipment.max(0.1);
        
        let raw_damage = base_damage as f64 * skill_ratio * equipment_ratio * random_factor;
        
        // Damage cannot be negative or exceed reasonable bounds
        Ok((raw_damage as u32).min(u32::MAX / 2))
    }

    /// Calculate financial interest with compound interest
    /// Formula: principal * (1 + rate)^time_periods
    pub fn compound_interest(
        principal: i64,
        interest_rate: f64,
        time_periods: u32,
    ) -> Result<i64> {
        if interest_rate < -1.0 {
            return Err(GameMechanicsError::InvalidParameter {
                param: "interest_rate".to_string(),
                value: interest_rate.to_string(),
            });
        }
        
        let compound_factor = (1.0 + interest_rate).powi(time_periods as i32);
        let final_amount = principal as f64 * compound_factor;
        
        // Check for overflow
        if final_amount > i64::MAX as f64 || final_amount < i64::MIN as f64 {
            return Err(GameMechanicsError::CalculationOverflow {
                operation: "compound_interest".to_string(),
            });
        }
        
        Ok(final_amount as i64)
    }

    /// Calculate market price based on supply and demand
    /// Formula: base_price * (demand / supply)^elasticity * volatility_factor
    pub fn market_price(
        base_price: f64,
        supply: f64,
        demand: f64,
        elasticity: f64,
        volatility: f64,
    ) -> Result<f64> {
        if supply <= 0.0 || demand <= 0.0 {
            return Err(GameMechanicsError::InvalidParameter {
                param: "supply_or_demand".to_string(),
                value: format!("supply: {}, demand: {}", supply, demand),
            });
        }
        
        let supply_demand_ratio = demand / supply;
        let price_modifier = supply_demand_ratio.powf(elasticity);
        let volatile_price = base_price * price_modifier * volatility;
        
        // Price cannot be negative
        Ok(volatile_price.max(0.01))
    }

    /// Calculate network latency based on distance and infrastructure
    /// Formula: base_latency + (distance * propagation_delay) + processing_delay
    pub fn network_latency(
        base_latency: u32,
        distance_km: f64,
        infrastructure_quality: f64,
        congestion_factor: f64,
    ) -> Result<u32> {
        // Speed of light in fiber optic cable (approximately 200,000 km/s)
        const FIBER_SPEED: f64 = 200_000.0;
        
        let propagation_delay = (distance_km / FIBER_SPEED) * 1000.0; // Convert to milliseconds
        let infrastructure_delay = 1.0 / infrastructure_quality.max(0.1);
        let congestion_delay = congestion_factor;
        
        let total_latency = base_latency as f64 + propagation_delay + infrastructure_delay + congestion_delay;
        
        Ok(total_latency as u32)
    }

    /// Calculate skill progression with diminishing returns
    /// Formula: base_gain * (1 - current_skill/max_skill)^diminishing_factor
    pub fn skill_progression(
        base_gain: u32,
        current_skill: u8,
        max_skill: u8,
        diminishing_factor: f64,
    ) -> Result<u32> {
        if current_skill > max_skill {
            return Err(GameMechanicsError::InvalidParameter {
                param: "current_skill".to_string(),
                value: format!("{} > {}", current_skill, max_skill),
            });
        }
        
        let skill_ratio = current_skill as f64 / max_skill as f64;
        let diminishing_multiplier = (1.0 - skill_ratio).powf(diminishing_factor);
        let actual_gain = base_gain as f64 * diminishing_multiplier;
        
        Ok(actual_gain as u32)
    }

    /// Calculate clan reputation based on member actions
    /// Formula: base_rep + sum(member_contributions * weight) + war_bonus - penalties
    pub fn clan_reputation(
        base_reputation: i32,
        member_contributions: &[(u32, f64)], // (contribution, weight)
        war_victories: u32,
        war_defeats: u32,
        penalties: i32,
    ) -> Result<i32> {
        let contribution_sum: f64 = member_contributions
            .iter()
            .map(|(contrib, weight)| *contrib as f64 * weight)
            .sum();
        
        let war_bonus = (war_victories as i32 * 10) - (war_defeats as i32 * 5);
        
        let total_reputation = base_reputation + contribution_sum as i32 + war_bonus - penalties;
        
        // Reputation cannot go below 0
        Ok(total_reputation.max(0))
    }

    /// Calculate resource consumption rate
    /// Formula: base_consumption * efficiency_factor * load_multiplier
    pub fn resource_consumption(
        base_consumption: u32,
        efficiency_factor: f64,
        current_load: f64,
        max_load: f64,
    ) -> Result<u32> {
        if max_load <= 0.0 {
            return Err(GameMechanicsError::InvalidParameter {
                param: "max_load".to_string(),
                value: max_load.to_string(),
            });
        }
        
        let load_ratio = (current_load / max_load).min(1.0);
        let load_multiplier = 1.0 + load_ratio; // Linear increase with load
        
        let actual_consumption = base_consumption as f64 * efficiency_factor * load_multiplier;
        
        Ok(actual_consumption as u32)
    }

    /// Calculate probability distribution for random events
    /// Uses normal distribution with mean and standard deviation
    pub fn normal_distribution_probability(
        value: f64,
        mean: f64,
        std_dev: f64,
    ) -> Result<f64> {
        if std_dev <= 0.0 {
            return Err(GameMechanicsError::InvalidParameter {
                param: "std_dev".to_string(),
                value: std_dev.to_string(),
            });
        }
        
        let variance = std_dev * std_dev;
        let diff = value - mean;
        let exponent = -(diff * diff) / (2.0 * variance);
        
        let probability = (1.0 / (std_dev * (2.0 * PI).sqrt())) * E.powf(exponent);
        
        Ok(probability)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_success_probability() {
        let result = Formulas::success_probability(0.5, 50, 10, 1.2, 1.0).unwrap();
        assert!(result.value() > 0.0 && result.value() <= 1.0);
    }

    #[test]
    fn test_experience_required() {
        let exp = Formulas::experience_required(10, 1000, 1.1).unwrap();
        assert!(exp > 1000);
        
        // Level 0 should require 0 experience
        let zero_exp = Formulas::experience_required(0, 1000, 1.1).unwrap();
        assert_eq!(zero_exp, 0);
    }

    #[test]
    fn test_process_time() {
        let time = Formulas::process_time(100, 2.0, 80, 1000, 1.0).unwrap();
        assert!(time > 0);
    }

    #[test]
    fn test_combat_damage() {
        let damage = Formulas::combat_damage(100, 80, 60, 1.2, 1.0, 1.0).unwrap();
        assert!(damage > 0);
    }

    #[test]
    fn test_compound_interest() {
        let final_amount = Formulas::compound_interest(1000, 0.05, 12).unwrap();
        assert!(final_amount > 1000);
    }

    #[test]
    fn test_market_price() {
        let price = Formulas::market_price(100.0, 50.0, 75.0, 0.5, 1.0).unwrap();
        assert!(price > 0.0);
    }

    #[test]
    fn test_network_latency() {
        let latency = Formulas::network_latency(10, 1000.0, 0.8, 1.2).unwrap();
        assert!(latency > 10);
    }

    #[test]
    fn test_skill_progression() {
        let gain = Formulas::skill_progression(100, 80, 100, 2.0).unwrap();
        assert!(gain < 100); // Should be less due to diminishing returns
    }

    #[test]
    fn test_clan_reputation() {
        let contributions = vec![(100, 1.0), (200, 0.5), (50, 2.0)];
        let rep = Formulas::clan_reputation(1000, &contributions, 5, 2, 100).unwrap();
        assert!(rep > 1000);
    }

    #[test]
    fn test_resource_consumption() {
        let consumption = Formulas::resource_consumption(100, 0.8, 50.0, 100.0).unwrap();
        assert!(consumption > 0);
    }

    #[test]
    fn test_normal_distribution_probability() {
        let prob = Formulas::normal_distribution_probability(0.0, 0.0, 1.0).unwrap();
        assert!(prob > 0.0);
    }

    #[test]
    fn test_invalid_parameters() {
        // Test invalid base rate
        assert!(Formulas::success_probability(-0.1, 50, 10, 1.2, 1.0).is_err());
        
        // Test invalid scaling factor
        assert!(Formulas::experience_required(10, 1000, 0.5).is_err());
        
        // Test invalid complexity
        assert!(Formulas::process_time(100, -1.0, 80, 1000, 1.0).is_err());
    }
}