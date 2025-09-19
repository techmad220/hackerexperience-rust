//! Reputation System - Faction standings and relationships

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reputation system managing faction relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationSystem {
    pub faction_standings: HashMap<String, FactionStanding>,
    pub total_reputation: i32,
    pub highest_standing: Option<String>,
    pub faction_bonuses: HashMap<String, Vec<ReputationBonus>>,
}

/// Standing with a specific faction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionStanding {
    pub faction_id: String,
    pub faction_name: String,
    pub current_reputation: i32,
    pub reputation_level: ReputationLevel,
    pub rank: String,
    pub benefits: Vec<String>,
    pub is_hostile: bool,
}

/// Reputation levels with factions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReputationLevel {
    Hostile,      // -1000 or less
    Unfriendly,   // -999 to -500
    Neutral,      // -499 to 499
    Friendly,     // 500 to 1499
    Honored,      // 1500 to 2999
    Revered,      // 3000 to 4999
    Exalted,      // 5000+
}

/// Bonuses from reputation levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReputationBonus {
    PriceDiscount { percentage: f32 },
    ExperienceBonus { percentage: f32 },
    AccessToContent { content_id: String },
    SpecialMissions { mission_ids: Vec<String> },
    UniqueItems { item_ids: Vec<String> },
    SafeHaven { location: String },
    FactionProtection,
    BlackMarketAccess,
}

/// Factions in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faction {
    pub id: String,
    pub name: String,
    pub description: String,
    pub faction_type: FactionType,
    pub opposed_factions: Vec<String>,
    pub allied_factions: Vec<String>,
}

/// Types of factions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FactionType {
    Corporation,
    Underground,
    Government,
    Criminal,
    Neutral,
    Special,
}

impl ReputationSystem {
    /// Create new reputation system
    pub fn new() -> Self {
        let mut system = Self {
            faction_standings: HashMap::new(),
            total_reputation: 0,
            highest_standing: None,
            faction_bonuses: HashMap::new(),
        };

        // Initialize default factions
        system.initialize_factions();
        system
    }

    /// Initialize all factions with neutral standing
    fn initialize_factions(&mut self) {
        let factions = vec![
            ("megacorp", "MegaCorp Industries", FactionType::Corporation),
            ("underground", "The Underground", FactionType::Underground),
            ("government", "World Government", FactionType::Government),
            ("black_market", "Black Market Dealers", FactionType::Criminal),
            ("tech_guild", "Tech Guild", FactionType::Neutral),
            ("shadow_collective", "Shadow Collective", FactionType::Special),
            ("cyber_police", "Cyber Police", FactionType::Government),
            ("anarchists", "Digital Anarchists", FactionType::Underground),
        ];

        for (id, name, faction_type) in factions {
            self.faction_standings.insert(
                id.to_string(),
                FactionStanding {
                    faction_id: id.to_string(),
                    faction_name: name.to_string(),
                    current_reputation: 0,
                    reputation_level: ReputationLevel::Neutral,
                    rank: "Unknown".to_string(),
                    benefits: vec![],
                    is_hostile: false,
                },
            );
        }
    }

    /// Modify reputation with a faction
    pub fn modify_reputation(&mut self, faction_id: &str, amount: i32) -> ReputationChange {
        let old_level = self.faction_standings
            .get(faction_id)
            .map(|f| f.reputation_level.clone())
            .unwrap_or(ReputationLevel::Neutral);

        if let Some(standing) = self.faction_standings.get_mut(faction_id) {
            standing.current_reputation += amount;

            // Update reputation level
            standing.reputation_level = Self::calculate_level(standing.current_reputation);

            // Update rank
            standing.rank = Self::get_rank(&standing.reputation_level, &standing.faction_id);

            // Update benefits
            standing.benefits = Self::get_benefits(&standing.reputation_level);

            // Check if hostile
            standing.is_hostile = standing.reputation_level == ReputationLevel::Hostile;

            // Apply faction relationships
            let related_changes = self.apply_faction_relationships(faction_id, amount);

            // Update total reputation
            self.calculate_total_reputation();
            self.update_highest_standing();

            ReputationChange {
                faction: faction_id.to_string(),
                change: amount,
                new_value: standing.current_reputation,
                new_level: standing.reputation_level.clone(),
                old_level,
                new_rank: standing.rank.clone(),
                related_changes,
            }
        } else {
            ReputationChange {
                faction: faction_id.to_string(),
                change: 0,
                new_value: 0,
                new_level: ReputationLevel::Neutral,
                old_level: ReputationLevel::Neutral,
                new_rank: "Unknown".to_string(),
                related_changes: vec![],
            }
        }
    }

    /// Calculate reputation level from points
    fn calculate_level(reputation: i32) -> ReputationLevel {
        match reputation {
            r if r <= -1000 => ReputationLevel::Hostile,
            r if r <= -500 => ReputationLevel::Unfriendly,
            r if r < 500 => ReputationLevel::Neutral,
            r if r < 1500 => ReputationLevel::Friendly,
            r if r < 3000 => ReputationLevel::Honored,
            r if r < 5000 => ReputationLevel::Revered,
            _ => ReputationLevel::Exalted,
        }
    }

    /// Get rank name for reputation level
    fn get_rank(level: &ReputationLevel, faction_id: &str) -> String {
        match (faction_id, level) {
            ("megacorp", ReputationLevel::Exalted) => "Board Member".to_string(),
            ("megacorp", ReputationLevel::Revered) => "Executive".to_string(),
            ("megacorp", ReputationLevel::Honored) => "Manager".to_string(),
            ("megacorp", ReputationLevel::Friendly) => "Employee".to_string(),

            ("underground", ReputationLevel::Exalted) => "Shadow Master".to_string(),
            ("underground", ReputationLevel::Revered) => "Elite Hacker".to_string(),
            ("underground", ReputationLevel::Honored) => "Trusted Member".to_string(),
            ("underground", ReputationLevel::Friendly) => "Associate".to_string(),

            ("government", ReputationLevel::Exalted) => "Special Agent".to_string(),
            ("government", ReputationLevel::Revered) => "Senior Analyst".to_string(),
            ("government", ReputationLevel::Honored) => "Analyst".to_string(),
            ("government", ReputationLevel::Friendly) => "Informant".to_string(),

            (_, ReputationLevel::Hostile) => "Enemy".to_string(),
            (_, ReputationLevel::Unfriendly) => "Distrusted".to_string(),
            (_, ReputationLevel::Neutral) => "Unknown".to_string(),
            (_, _) => "Member".to_string(),
        }
    }

    /// Get benefits for reputation level
    fn get_benefits(level: &ReputationLevel) -> Vec<String> {
        match level {
            ReputationLevel::Friendly => vec![
                "5% price discount".to_string(),
                "Access to faction store".to_string(),
            ],
            ReputationLevel::Honored => vec![
                "10% price discount".to_string(),
                "Access to faction missions".to_string(),
                "Faction safe house access".to_string(),
            ],
            ReputationLevel::Revered => vec![
                "15% price discount".to_string(),
                "Access to elite missions".to_string(),
                "Special faction items".to_string(),
                "Faction protection".to_string(),
            ],
            ReputationLevel::Exalted => vec![
                "20% price discount".to_string(),
                "Access to legendary missions".to_string(),
                "Unique faction rewards".to_string(),
                "Full faction support".to_string(),
                "Special faction title".to_string(),
            ],
            _ => vec![],
        }
    }

    /// Apply faction relationships (allies/enemies)
    fn apply_faction_relationships(&mut self, faction_id: &str, amount: i32) -> Vec<RelatedChange> {
        let mut related = Vec::new();

        // Define faction relationships
        let relationships = self.get_faction_relationships(faction_id);

        for (related_faction, modifier) in relationships {
            let modified_amount = (amount as f32 * modifier) as i32;
            if modified_amount != 0 {
                if let Some(standing) = self.faction_standings.get_mut(&related_faction) {
                    standing.current_reputation += modified_amount;
                    standing.reputation_level = Self::calculate_level(standing.current_reputation);

                    related.push(RelatedChange {
                        faction: related_faction,
                        change: modified_amount,
                    });
                }
            }
        }

        related
    }

    /// Get faction relationships
    fn get_faction_relationships(&self, faction_id: &str) -> Vec<(String, f32)> {
        match faction_id {
            "megacorp" => vec![
                ("government".to_string(), 0.25),      // Slight positive
                ("underground".to_string(), -0.5),     // Negative
                ("cyber_police".to_string(), 0.1),     // Slight positive
            ],
            "underground" => vec![
                ("megacorp".to_string(), -0.5),        // Negative
                ("government".to_string(), -0.75),     // Very negative
                ("anarchists".to_string(), 0.5),       // Positive
                ("black_market".to_string(), 0.25),    // Slight positive
            ],
            "government" => vec![
                ("megacorp".to_string(), 0.25),        // Slight positive
                ("underground".to_string(), -0.75),    // Very negative
                ("cyber_police".to_string(), 0.5),     // Positive
                ("anarchists".to_string(), -1.0),      // Very negative
            ],
            _ => vec![],
        }
    }

    /// Calculate total reputation across all factions
    fn calculate_total_reputation(&mut self) {
        self.total_reputation = self.faction_standings
            .values()
            .map(|s| s.current_reputation.max(0)) // Only count positive reputation
            .sum();
    }

    /// Update highest standing faction
    fn update_highest_standing(&mut self) {
        self.highest_standing = self.faction_standings
            .iter()
            .max_by_key(|(_, s)| s.current_reputation)
            .map(|(id, _)| id.clone());
    }

    /// Get total reputation
    pub fn get_total_reputation(&self) -> i32 {
        self.total_reputation
    }

    /// Check if player has required reputation
    pub fn has_reputation(&self, faction_id: &str, required: i32) -> bool {
        self.faction_standings
            .get(faction_id)
            .map(|s| s.current_reputation >= required)
            .unwrap_or(false)
    }

    /// Get faction bonuses
    pub fn get_faction_bonuses(&self, faction_id: &str) -> Vec<ReputationBonus> {
        if let Some(standing) = self.faction_standings.get(faction_id) {
            match standing.reputation_level {
                ReputationLevel::Friendly => vec![
                    ReputationBonus::PriceDiscount { percentage: 5.0 },
                ],
                ReputationLevel::Honored => vec![
                    ReputationBonus::PriceDiscount { percentage: 10.0 },
                    ReputationBonus::ExperienceBonus { percentage: 5.0 },
                ],
                ReputationLevel::Revered => vec![
                    ReputationBonus::PriceDiscount { percentage: 15.0 },
                    ReputationBonus::ExperienceBonus { percentage: 10.0 },
                    ReputationBonus::FactionProtection,
                ],
                ReputationLevel::Exalted => vec![
                    ReputationBonus::PriceDiscount { percentage: 20.0 },
                    ReputationBonus::ExperienceBonus { percentage: 15.0 },
                    ReputationBonus::FactionProtection,
                    ReputationBonus::BlackMarketAccess,
                ],
                _ => vec![],
            }
        } else {
            vec![]
        }
    }
}

/// Result of a reputation change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationChange {
    pub faction: String,
    pub change: i32,
    pub new_value: i32,
    pub new_level: ReputationLevel,
    pub old_level: ReputationLevel,
    pub new_rank: String,
    pub related_changes: Vec<RelatedChange>,
}

/// Related faction changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedChange {
    pub faction: String,
    pub change: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reputation_system() {
        let mut rep_system = ReputationSystem::new();

        // Test reputation modification
        let change = rep_system.modify_reputation("underground", 1000);
        assert_eq!(change.new_level, ReputationLevel::Friendly);

        // Test faction relationships
        let change = rep_system.modify_reputation("megacorp", 1000);
        assert!(!change.related_changes.is_empty());

        // Test total reputation
        assert!(rep_system.get_total_reputation() > 0);
    }

    #[test]
    fn test_reputation_levels() {
        assert_eq!(ReputationSystem::calculate_level(-1500), ReputationLevel::Hostile);
        assert_eq!(ReputationSystem::calculate_level(0), ReputationLevel::Neutral);
        assert_eq!(ReputationSystem::calculate_level(1000), ReputationLevel::Friendly);
        assert_eq!(ReputationSystem::calculate_level(5500), ReputationLevel::Exalted);
    }
}