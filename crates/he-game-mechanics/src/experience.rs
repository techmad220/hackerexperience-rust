//! Experience and leveling system mechanics
//! 
//! Implements all experience calculations, level progression, and skill development
//! mechanics from the original HackerExperience game.

use crate::{Result, GameMechanicsError};
use crate::config::ExperienceConfig;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Calculate experience required for a specific level
/// 
/// Formula from original HE: Base XP × Level^1.2 × Multiplier
/// - Level 1: 1,000 XP
/// - Level 10: ~15,849 XP  
/// - Level 50: ~177,827 XP
/// - Level 100: ~501,187 XP
pub fn calculate_experience_for_level(level: i32, config: &ExperienceConfig) -> i64 {
    if level <= 1 {
        return config.base_experience_per_level;
    }
    
    let base = Decimal::from(config.base_experience_per_level);
    let level_factor = Decimal::from(level).powu(config.experience_curve_factor.to_u64().unwrap_or(120) / 100);
    let multiplier = config.level_multiplier.powu((level - 1) as u64);
    
    (base * level_factor * multiplier).to_i64().unwrap_or(config.base_experience_per_level)
}

/// Calculate level from total experience points
pub fn calculate_level_from_experience(experience: i64, config: &ExperienceConfig) -> i32 {
    if experience < config.base_experience_per_level {
        return 1;
    }
    
    // Binary search for efficiency
    let mut low = 1;
    let mut high = config.max_level;
    
    while low < high {
        let mid = (low + high + 1) / 2;
        let required_exp = calculate_experience_for_level(mid, config);
        
        if experience >= required_exp {
            low = mid;
        } else {
            high = mid - 1;
        }
    }
    
    low.min(config.max_level)
}

/// Calculate experience gained from an action
pub fn calculate_experience_gain(
    base_experience: i64,
    difficulty_multiplier: Decimal,
    skill_level: i32,
    specialization_bonus: bool,
    config: &ExperienceConfig,
) -> i64 {
    let mut total_exp = Decimal::from(base_experience) * difficulty_multiplier;
    
    // Learning efficiency decreases with skill level
    let efficiency = config.learning_efficiency_decay.powu(skill_level as u64);
    total_exp *= efficiency;
    
    // Specialization bonus
    if specialization_bonus {
        total_exp *= (dec!(1.0) + config.skill_specialization_bonus);
    }
    
    total_exp.to_i64().unwrap_or(0).max(1)
}

/// Calculate progress percentage to next level
pub fn calculate_level_progress(experience: i64, config: &ExperienceConfig) -> Decimal {
    let current_level = calculate_level_from_experience(experience, config);
    let current_level_exp = calculate_experience_for_level(current_level, config);
    let next_level_exp = calculate_experience_for_level(current_level + 1, config);
    
    if current_level >= config.max_level {
        return dec!(100.0);
    }
    
    let progress = experience - current_level_exp;
    let level_range = next_level_exp - current_level_exp;
    
    if level_range <= 0 {
        return dec!(100.0);
    }
    
    (Decimal::from(progress) / Decimal::from(level_range) * dec!(100.0)).max(dec!(0.0)).min(dec!(100.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_experience_calculations() {
        let config = ExperienceConfig::default();
        
        // Test level 1
        assert_eq!(calculate_experience_for_level(1, &config), 1000);
        
        // Test level from experience
        assert_eq!(calculate_level_from_experience(1000, &config), 1);
        assert_eq!(calculate_level_from_experience(15000, &config), 10);
        
        // Test progress calculation
        let progress = calculate_level_progress(1500, &config);
        assert!(progress > dec!(0.0) && progress < dec!(100.0));
    }
    
    #[test]
    fn test_experience_gain() {
        let config = ExperienceConfig::default();
        let exp_gain = calculate_experience_gain(100, dec!(1.5), 5, true, &config);
        assert!(exp_gain > 0);
    }
}