//! PvP System - Player versus Player hacking battles

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

/// PvP match between two players
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PvPMatch {
    pub id: Uuid,
    pub attacker: PlayerCombatant,
    pub defender: PlayerCombatant,
    pub status: MatchStatus,
    pub started_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub rounds: Vec<PvPRound>,
    pub stakes: MatchStakes,
    pub result: Option<MatchResult>,
    pub spectators: Vec<Uuid>,
}

/// Player in PvP combat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerCombatant {
    pub player_id: Uuid,
    pub username: String,
    pub level: u32,
    pub rating: i32,
    pub hardware_power: u32,
    pub software_arsenal: Vec<CombatSoftware>,
    pub defenses: PlayerDefenses,
    pub health: i32,
    pub max_health: i32,
}

/// Combat software for PvP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatSoftware {
    pub id: String,
    pub name: String,
    pub software_type: CombatSoftwareType,
    pub power: u32,
    pub cooldown: u32,
    pub current_cooldown: u32,
}

/// Types of combat software
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CombatSoftwareType {
    Virus { damage: u32, duration: u32 },
    Exploit { penetration: u32 },
    Scanner { reveal_chance: f32 },
    Firewall { block_chance: f32 },
    Antivirus { heal_amount: u32 },
    DDoS { stun_duration: u32 },
    Backdoor { bypass_defense: bool },
    Trojan { steal_resources: u32 },
}

/// Player defenses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDefenses {
    pub firewall_level: u32,
    pub antivirus_level: u32,
    pub encryption_level: u32,
    pub honeypot_active: bool,
    pub shields_remaining: u32,
}

/// Match status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MatchStatus {
    Challenging,    // Waiting for defender to accept
    Preparing,      // Both players preparing loadout
    InProgress,     // Active combat
    Completed,      // Match finished
    Cancelled,      // Match cancelled
}

/// What's at stake in the match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchStakes {
    pub money_bet: i64,
    pub rating_change: i32,
    pub experience_reward: u32,
    pub items_wagered: Vec<String>,
    pub ranked: bool,
}

/// Round of combat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PvPRound {
    pub round_number: u32,
    pub attacker_action: CombatAction,
    pub defender_action: CombatAction,
    pub outcome: RoundOutcome,
    pub timestamp: DateTime<Utc>,
}

/// Combat action taken by player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CombatAction {
    UseSoftware { software_id: String },
    Defend,
    Scan,
    DirectAttack,
    UseItem { item_id: String },
    Forfeit,
}

/// Outcome of a round
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundOutcome {
    pub attacker_damage: i32,
    pub defender_damage: i32,
    pub status_effects: Vec<StatusEffect>,
    pub resources_stolen: i64,
    pub defenses_breached: bool,
}

/// Status effects applied during combat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatusEffect {
    Infected { duration: u32, damage_per_turn: i32 },
    Stunned { duration: u32 },
    Shielded { duration: u32, absorption: i32 },
    Revealed { duration: u32 },
    Slowed { duration: u32, speed_reduction: f32 },
    Burning { duration: u32, damage: i32 },
}

/// Match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    pub winner_id: Uuid,
    pub loser_id: Uuid,
    pub duration: i64, // seconds
    pub total_rounds: u32,
    pub winner_rewards: MatchRewards,
    pub loser_penalties: MatchPenalties,
    pub mvp_action: Option<CombatAction>,
}

/// Rewards for winning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRewards {
    pub money: i64,
    pub experience: u32,
    pub rating_gain: i32,
    pub items: Vec<String>,
    pub achievements: Vec<String>,
}

/// Penalties for losing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchPenalties {
    pub money_lost: i64,
    pub rating_loss: i32,
    pub reputation_loss: i32,
}

/// PvP matchmaking system
#[derive(Debug, Clone)]
pub struct Matchmaking {
    pub queue: Vec<MatchmakingEntry>,
    pub active_matches: HashMap<Uuid, PvPMatch>,
    pub player_stats: HashMap<Uuid, PlayerPvPStats>,
}

/// Entry in matchmaking queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchmakingEntry {
    pub player_id: Uuid,
    pub rating: i32,
    pub preferred_stakes: MatchStakes,
    pub queued_at: DateTime<Utc>,
    pub acceptable_rating_range: (i32, i32),
}

/// Player PvP statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPvPStats {
    pub total_matches: u32,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
    pub current_streak: i32,
    pub best_streak: u32,
    pub rating: i32,
    pub peak_rating: i32,
    pub rank: PvPRank,
    pub favorite_software: Vec<String>,
    pub nemesis: Option<Uuid>, // Player they lose to most
    pub rival: Option<Uuid>,   // Player they fight most
}

/// PvP ranking tiers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum PvPRank {
    Unranked,           // < 1000
    Bronze,             // 1000-1199
    Silver,             // 1200-1399
    Gold,               // 1400-1599
    Platinum,           // 1600-1799
    Diamond,            // 1800-1999
    Master,             // 2000-2199
    Grandmaster,        // 2200-2399
    Legend,             // 2400+
}

impl PvPMatch {
    /// Create a new PvP match
    pub fn new(attacker: PlayerCombatant, defender: PlayerCombatant, stakes: MatchStakes) -> Self {
        Self {
            id: Uuid::new_v4(),
            attacker,
            defender,
            status: MatchStatus::Challenging,
            started_at: Utc::now(),
            ends_at: Utc::now() + Duration::minutes(10),
            rounds: Vec::new(),
            stakes,
            result: None,
            spectators: Vec::new(),
        }
    }

    /// Start the match
    pub fn start(&mut self) {
        self.status = MatchStatus::InProgress;
        self.started_at = Utc::now();
        self.ends_at = Utc::now() + Duration::minutes(15);
    }

    /// Process a round of combat
    pub fn process_round(&mut self, attacker_action: CombatAction, defender_action: CombatAction) -> RoundOutcome {
        let round_number = self.rounds.len() as u32 + 1;

        // Calculate outcome based on actions
        let outcome = self.calculate_round_outcome(&attacker_action, &defender_action);

        // Apply damage
        self.attacker.health -= outcome.defender_damage;
        self.defender.health -= outcome.attacker_damage;

        // Record the round
        self.rounds.push(PvPRound {
            round_number,
            attacker_action,
            defender_action,
            outcome: outcome.clone(),
            timestamp: Utc::now(),
        });

        // Check for match end
        if self.attacker.health <= 0 || self.defender.health <= 0 {
            self.end_match();
        }

        outcome
    }

    /// Calculate outcome of actions
    fn calculate_round_outcome(&self, attacker_action: &CombatAction, defender_action: &CombatAction) -> RoundOutcome {
        let mut outcome = RoundOutcome {
            attacker_damage: 0,
            defender_damage: 0,
            status_effects: Vec::new(),
            resources_stolen: 0,
            defenses_breached: false,
        };

        match (attacker_action, defender_action) {
            (CombatAction::DirectAttack, CombatAction::Defend) => {
                // Defender blocks most damage
                outcome.attacker_damage = 10;
                outcome.defender_damage = 5;
            }
            (CombatAction::DirectAttack, _) => {
                // Direct hit
                outcome.attacker_damage = 0;
                outcome.defender_damage = 30;
            }
            (CombatAction::UseSoftware { software_id }, _) => {
                // Apply software effects
                outcome.defender_damage = 25;
                outcome.status_effects.push(StatusEffect::Infected {
                    duration: 3,
                    damage_per_turn: 10,
                });
            }
            _ => {}
        }

        outcome
    }

    /// End the match
    fn end_match(&mut self) {
        self.status = MatchStatus::Completed;

        let winner_id;
        let loser_id;

        if self.attacker.health > self.defender.health {
            winner_id = self.attacker.player_id;
            loser_id = self.defender.player_id;
        } else {
            winner_id = self.defender.player_id;
            loser_id = self.attacker.player_id;
        }

        self.result = Some(MatchResult {
            winner_id,
            loser_id,
            duration: (Utc::now() - self.started_at).num_seconds(),
            total_rounds: self.rounds.len() as u32,
            winner_rewards: MatchRewards {
                money: self.stakes.money_bet * 2,
                experience: self.stakes.experience_reward,
                rating_gain: self.stakes.rating_change,
                items: Vec::new(),
                achievements: Vec::new(),
            },
            loser_penalties: MatchPenalties {
                money_lost: self.stakes.money_bet,
                rating_loss: self.stakes.rating_change,
                reputation_loss: 10,
            },
            mvp_action: None,
        });
    }

    /// Add a spectator
    pub fn add_spectator(&mut self, player_id: Uuid) {
        if !self.spectators.contains(&player_id) {
            self.spectators.push(player_id);
        }
    }
}

impl Matchmaking {
    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
            active_matches: HashMap::new(),
            player_stats: HashMap::new(),
        }
    }

    /// Add player to matchmaking queue
    pub fn join_queue(&mut self, entry: MatchmakingEntry) {
        self.queue.push(entry);
        self.try_match_players();
    }

    /// Try to match players in queue
    fn try_match_players(&mut self) {
        if self.queue.len() < 2 {
            return;
        }

        // Sort by rating for better matching
        self.queue.sort_by_key(|e| e.rating);

        let mut matched_indices = Vec::new();

        for i in 0..self.queue.len() {
            if matched_indices.contains(&i) {
                continue;
            }

            for j in (i + 1)..self.queue.len() {
                if matched_indices.contains(&j) {
                    continue;
                }

                let p1 = &self.queue[i];
                let p2 = &self.queue[j];

                // Check if ratings are compatible
                if p1.rating >= p2.acceptable_rating_range.0
                    && p1.rating <= p2.acceptable_rating_range.1
                    && p2.rating >= p1.acceptable_rating_range.0
                    && p2.rating <= p1.acceptable_rating_range.1
                {
                    // Create match
                    matched_indices.push(i);
                    matched_indices.push(j);
                    break;
                }
            }
        }

        // Remove matched players from queue
        matched_indices.sort_by(|a, b| b.cmp(a));
        for idx in matched_indices {
            self.queue.remove(idx);
        }
    }

    /// Get player's PvP rank based on rating
    pub fn get_rank(rating: i32) -> PvPRank {
        match rating {
            r if r < 1000 => PvPRank::Unranked,
            r if r < 1200 => PvPRank::Bronze,
            r if r < 1400 => PvPRank::Silver,
            r if r < 1600 => PvPRank::Gold,
            r if r < 1800 => PvPRank::Platinum,
            r if r < 2000 => PvPRank::Diamond,
            r if r < 2200 => PvPRank::Master,
            r if r < 2400 => PvPRank::Grandmaster,
            _ => PvPRank::Legend,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pvp_match() {
        let attacker = PlayerCombatant {
            player_id: Uuid::new_v4(),
            username: "Attacker".to_string(),
            level: 10,
            rating: 1500,
            hardware_power: 100,
            software_arsenal: vec![],
            defenses: PlayerDefenses {
                firewall_level: 5,
                antivirus_level: 5,
                encryption_level: 5,
                honeypot_active: false,
                shields_remaining: 3,
            },
            health: 100,
            max_health: 100,
        };

        let defender = attacker.clone();
        let stakes = MatchStakes {
            money_bet: 1000,
            rating_change: 25,
            experience_reward: 500,
            items_wagered: vec![],
            ranked: true,
        };

        let mut match_obj = PvPMatch::new(attacker, defender, stakes);
        match_obj.start();

        assert_eq!(match_obj.status, MatchStatus::InProgress);

        // Simulate round
        let outcome = match_obj.process_round(
            CombatAction::DirectAttack,
            CombatAction::Defend,
        );

        assert!(outcome.defender_damage > 0 || outcome.attacker_damage > 0);
    }
}