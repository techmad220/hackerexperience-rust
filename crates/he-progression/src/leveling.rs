//! Leveling System - Experience points and level progression

use serde::{Deserialize, Serialize};

/// Player level and experience information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelInfo {
    pub level: u32,
    pub current_experience: u64,
    pub total_experience: u64,
    pub experience_to_next: u64,
}

impl LevelInfo {
    /// Create new level info starting at level 1
    pub fn new() -> Self {
        Self {
            level: 1,
            current_experience: 0,
            total_experience: 0,
            experience_to_next: Self::calculate_experience_required(2),
        }
    }

    /// Add experience and return number of level ups
    pub fn add_experience(&mut self, amount: u32) -> u32 {
        let mut level_ups = 0;
        self.current_experience += amount as u64;
        self.total_experience += amount as u64;

        // Check for level ups
        while self.current_experience >= self.experience_to_next {
            self.current_experience -= self.experience_to_next;
            self.level += 1;
            level_ups += 1;
            self.experience_to_next = Self::calculate_experience_required(self.level + 1);
        }

        level_ups
    }

    /// Calculate experience required for a specific level
    /// Uses a quadratic formula for smooth progression
    fn calculate_experience_required(level: u32) -> u64 {
        // Base: 100 XP for level 2
        // Formula: 100 * level^1.5
        let base = 100.0;
        let exponent = 1.5;
        (base * (level as f64).powf(exponent)) as u64
    }

    /// Get experience required for current level
    pub fn get_current_level_requirement(&self) -> u64 {
        if self.level == 1 {
            0
        } else {
            Self::calculate_experience_required(self.level)
        }
    }

    /// Get total experience required for a level from level 1
    pub fn get_total_experience_for_level(level: u32) -> u64 {
        let mut total = 0u64;
        for l in 2..=level {
            total += Self::calculate_experience_required(l);
        }
        total
    }

    /// Get progress percentage to next level
    pub fn get_progress_percentage(&self) -> f32 {
        if self.experience_to_next == 0 {
            100.0
        } else {
            (self.current_experience as f32 / self.experience_to_next as f32) * 100.0
        }
    }

    /// Calculate level from total experience
    pub fn level_from_experience(total_exp: u64) -> u32 {
        let mut level = 1u32;
        let mut required = 0u64;

        loop {
            let next_required = Self::calculate_experience_required(level + 1);
            if required + next_required > total_exp {
                break;
            }
            required += next_required;
            level += 1;
        }

        level
    }
}

/// Experience multipliers for different activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceMultipliers {
    pub server_hack_base: f32,
    pub mission_complete: f32,
    pub pvp_victory: f32,
    pub first_time_bonus: f32,
    pub clan_bonus: f32,
    pub premium_bonus: f32,
    pub event_bonus: f32,
}

impl Default for ExperienceMultipliers {
    fn default() -> Self {
        Self {
            server_hack_base: 1.0,
            mission_complete: 1.5,
            pvp_victory: 2.0,
            first_time_bonus: 3.0,
            clan_bonus: 1.1,
            premium_bonus: 1.5,
            event_bonus: 2.0,
        }
    }
}

/// Calculate experience for different activities
pub struct ExperienceCalculator {
    multipliers: ExperienceMultipliers,
}

impl ExperienceCalculator {
    pub fn new() -> Self {
        Self {
            multipliers: ExperienceMultipliers::default(),
        }
    }

    /// Calculate experience for hacking a server
    pub fn calculate_hack_experience(&self, server_tier: u32, player_level: u32, first_time: bool) -> u32 {
        let base = match server_tier {
            1 => 50,
            2 => 150,
            3 => 400,
            4 => 1000,
            _ => 2000,
        };

        // Level difference modifier
        let level_diff = server_tier as i32 - player_level as i32;
        let level_modifier = match level_diff {
            d if d >= 10 => 2.0,  // Much harder enemy
            d if d >= 5 => 1.5,   // Harder enemy
            d if d >= 0 => 1.0,   // Same level or harder
            d if d >= -5 => 0.7,  // Slightly easier
            _ => 0.5,              // Much easier
        };

        let mut exp = (base as f32 * level_modifier * self.multipliers.server_hack_base) as u32;

        if first_time {
            exp = (exp as f32 * self.multipliers.first_time_bonus) as u32;
        }

        exp
    }

    /// Calculate experience for completing a mission
    pub fn calculate_mission_experience(&self, mission_difficulty: &str, objectives_completed: u32) -> u32 {
        let base = match mission_difficulty {
            "Easy" => 200,
            "Medium" => 500,
            "Hard" => 1200,
            "Expert" => 2500,
            _ => 100,
        };

        let completion_bonus = objectives_completed * 50;
        ((base + completion_bonus) as f32 * self.multipliers.mission_complete) as u32
    }

    /// Calculate experience for PvP victory
    pub fn calculate_pvp_experience(&self, winner_level: u32, loser_level: u32) -> u32 {
        let base = 300;
        let level_difference = (loser_level as i32 - winner_level as i32).max(-10).min(10);
        let level_modifier = 1.0 + (level_difference as f32 * 0.1);

        (base as f32 * level_modifier * self.multipliers.pvp_victory) as u32
    }

    /// Apply clan and premium bonuses
    pub fn apply_bonuses(&self, base_exp: u32, has_clan: bool, has_premium: bool) -> u32 {
        let mut exp = base_exp as f32;

        if has_clan {
            exp *= self.multipliers.clan_bonus;
        }

        if has_premium {
            exp *= self.multipliers.premium_bonus;
        }

        exp as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leveling_progression() {
        let mut level_info = LevelInfo::new();

        assert_eq!(level_info.level, 1);

        // Add enough experience for level 2
        let level_ups = level_info.add_experience(150);
        assert_eq!(level_ups, 1);
        assert_eq!(level_info.level, 2);

        // Add more experience
        let level_ups = level_info.add_experience(500);
        assert!(level_info.level > 2);
    }

    #[test]
    fn test_experience_calculator() {
        let calc = ExperienceCalculator::new();

        // Test server hack experience
        let exp = calc.calculate_hack_experience(2, 5, false);
        assert!(exp > 0);

        // Test with first time bonus
        let first_time_exp = calc.calculate_hack_experience(2, 5, true);
        assert!(first_time_exp > exp);

        // Test mission experience
        let mission_exp = calc.calculate_mission_experience("Medium", 3);
        assert!(mission_exp > 0);

        // Test PvP experience
        let pvp_exp = calc.calculate_pvp_experience(10, 12);
        assert!(pvp_exp > 0);
    }
}