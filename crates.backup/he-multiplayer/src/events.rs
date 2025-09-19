//! Global Events System - Server-wide events and competitions

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use std::collections::{HashMap, HashSet};

/// Global event affecting all players
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalEvent {
    pub id: Uuid,
    pub event_type: EventType,
    pub name: String,
    pub description: String,
    pub started_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub status: EventStatus,
    pub participation: EventParticipation,
    pub rewards: EventRewards,
    pub leaderboard: Vec<EventScore>,
    pub special_rules: EventRules,
}

/// Types of global events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    // Competitive Events
    HackingTournament,
    PvPChampionship,
    SpeedHacking,
    ClanWars,

    // Cooperative Events
    WorldBoss {
        boss_health: u64,
        current_health: u64,
    },
    CommunityGoal {
        goal: u64,
        progress: u64,
    },

    // Economic Events
    MarketCrash,
    GoldRush,
    BlackMarketSale,

    // Seasonal Events
    Holiday {
        holiday_name: String,
    },
    Anniversary,

    // Story Events
    InvasionEvent {
        invader: String,
        threat_level: u32,
    },
    MysteryEvent {
        clues_found: u32,
        total_clues: u32,
    },

    // Special Events
    DoubleXP,
    TripleLoot,
    UnlimitedEnergy,
}

/// Event status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventStatus {
    Announced,   // Event announced but not started
    Active,      // Event is running
    FinalHour,   // Last hour of event
    Completed,   // Event finished
    Cancelled,   // Event cancelled
}

/// Event participation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventParticipation {
    pub total_participants: u32,
    pub active_participants: u32,
    pub participant_list: HashSet<Uuid>,
    pub clan_participation: HashMap<Uuid, u32>,
}

/// Event rewards structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRewards {
    pub participation_rewards: RewardTier,
    pub ranking_rewards: Vec<RewardTier>,
    pub milestone_rewards: Vec<MilestoneReward>,
    pub random_drops: Vec<RandomDrop>,
}

/// Reward tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardTier {
    pub rank_range: (u32, u32),
    pub money: i64,
    pub experience: u32,
    pub items: Vec<String>,
    pub titles: Vec<String>,
    pub special_currency: u32,
}

/// Milestone reward
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneReward {
    pub milestone: u64,
    pub description: String,
    pub reward: RewardTier,
    pub claimed_by: HashSet<Uuid>,
}

/// Random drop during event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomDrop {
    pub item_id: String,
    pub drop_chance: f32,
    pub max_drops: u32,
    pub current_drops: u32,
}

/// Event leaderboard entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventScore {
    pub player_id: Uuid,
    pub player_name: String,
    pub clan_id: Option<Uuid>,
    pub score: u64,
    pub achievements: Vec<String>,
    pub last_updated: DateTime<Utc>,
}

/// Special rules for event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRules {
    pub pvp_enabled: bool,
    pub clan_competition: bool,
    pub level_brackets: bool,
    pub special_modifiers: Vec<EventModifier>,
    pub restricted_items: Vec<String>,
    pub bonus_objectives: Vec<BonusObjective>,
}

/// Event modifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventModifier {
    SpeedBoost { multiplier: f32 },
    DamageBoost { multiplier: f32 },
    CostReduction { percentage: f32 },
    CooldownReduction { percentage: f32 },
    SpecialAbility { ability: String },
}

/// Bonus objective during event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonusObjective {
    pub id: String,
    pub description: String,
    pub points: u32,
    pub repeatable: bool,
    pub max_completions: u32,
    pub completed_by: HashMap<Uuid, u32>,
}

/// Tournament structure for competitive events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tournament {
    pub id: Uuid,
    pub name: String,
    pub format: TournamentFormat,
    pub brackets: Vec<TournamentBracket>,
    pub current_round: u32,
    pub total_rounds: u32,
    pub participants: Vec<TournamentParticipant>,
    pub matches: Vec<TournamentMatch>,
    pub prizes: TournamentPrizes,
}

/// Tournament format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TournamentFormat {
    SingleElimination,
    DoubleElimination,
    RoundRobin,
    Swiss,
    KingOfTheHill,
}

/// Tournament bracket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentBracket {
    pub bracket_id: String,
    pub level_range: (u32, u32),
    pub participants: Vec<Uuid>,
    pub matches: Vec<Uuid>,
}

/// Tournament participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentParticipant {
    pub player_id: Uuid,
    pub player_name: String,
    pub seed: u32,
    pub wins: u32,
    pub losses: u32,
    pub eliminated: bool,
}

/// Tournament match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentMatch {
    pub match_id: Uuid,
    pub round: u32,
    pub player1: Uuid,
    pub player2: Uuid,
    pub winner: Option<Uuid>,
    pub match_time: DateTime<Utc>,
    pub spectators: u32,
}

/// Tournament prizes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentPrizes {
    pub first_place: RewardTier,
    pub second_place: RewardTier,
    pub third_place: RewardTier,
    pub participation: RewardTier,
}

/// World boss event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldBoss {
    pub boss_id: String,
    pub name: String,
    pub description: String,
    pub total_health: u64,
    pub current_health: u64,
    pub phase: BossPhase,
    pub damage_dealers: HashMap<Uuid, u64>,
    pub special_attacks: Vec<BossAttack>,
    pub loot_table: Vec<BossLoot>,
}

/// Boss phases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BossPhase {
    Phase1,    // 100-75% health
    Phase2,    // 75-50% health
    Phase3,    // 50-25% health
    Enraged,   // <25% health
    Defeated,
}

/// Boss special attack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BossAttack {
    pub name: String,
    pub damage: u32,
    pub targets: u32,
    pub cooldown: u32,
    pub special_effect: Option<String>,
}

/// Boss loot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BossLoot {
    pub item_id: String,
    pub min_damage_required: u64,
    pub drop_chance: f32,
}

/// Seasonal event manager
#[derive(Debug, Clone)]
pub struct EventManager {
    pub active_events: Vec<GlobalEvent>,
    pub scheduled_events: Vec<ScheduledEvent>,
    pub event_history: Vec<GlobalEvent>,
    pub participant_stats: HashMap<Uuid, PlayerEventStats>,
}

/// Scheduled future event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledEvent {
    pub event: GlobalEvent,
    pub announcement_time: DateTime<Utc>,
    pub start_time: DateTime<Utc>,
    pub recurring: bool,
    pub recurrence_pattern: Option<RecurrencePattern>,
}

/// Event recurrence pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecurrencePattern {
    Daily,
    Weekly { day_of_week: u32 },
    Monthly { day_of_month: u32 },
    Yearly { month: u32, day: u32 },
}

/// Player's event statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerEventStats {
    pub events_participated: u32,
    pub events_won: u32,
    pub total_event_points: u64,
    pub best_ranking: u32,
    pub favorite_event_type: EventType,
    pub event_badges: Vec<String>,
}

impl GlobalEvent {
    /// Create a new event
    pub fn new(event_type: EventType, name: String, duration_hours: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            name,
            description: String::new(),
            started_at: Utc::now(),
            ends_at: Utc::now() + Duration::hours(duration_hours),
            status: EventStatus::Announced,
            participation: EventParticipation {
                total_participants: 0,
                active_participants: 0,
                participant_list: HashSet::new(),
                clan_participation: HashMap::new(),
            },
            rewards: EventRewards::default(),
            leaderboard: Vec::new(),
            special_rules: EventRules::default(),
        }
    }

    /// Start the event
    pub fn start(&mut self) {
        self.status = EventStatus::Active;
        self.started_at = Utc::now();
    }

    /// Add participant
    pub fn add_participant(&mut self, player_id: Uuid, player_name: String) {
        if !self.participation.participant_list.contains(&player_id) {
            self.participation.participant_list.insert(player_id);
            self.participation.total_participants += 1;
            self.participation.active_participants += 1;

            // Add to leaderboard
            self.leaderboard.push(EventScore {
                player_id,
                player_name,
                clan_id: None,
                score: 0,
                achievements: Vec::new(),
                last_updated: Utc::now(),
            });
        }
    }

    /// Update player score
    pub fn update_score(&mut self, player_id: Uuid, score_delta: u64) {
        if let Some(entry) = self.leaderboard.iter_mut().find(|e| e.player_id == player_id) {
            entry.score += score_delta;
            entry.last_updated = Utc::now();
        }

        // Re-sort leaderboard
        self.leaderboard.sort_by(|a, b| b.score.cmp(&a.score));
    }

    /// Check if event is active
    pub fn is_active(&self) -> bool {
        self.status == EventStatus::Active && Utc::now() < self.ends_at
    }

    /// Get time remaining
    pub fn time_remaining(&self) -> Duration {
        if self.ends_at > Utc::now() {
            self.ends_at - Utc::now()
        } else {
            Duration::zero()
        }
    }

    /// End the event
    pub fn end(&mut self) {
        self.status = EventStatus::Completed;
        // Calculate and distribute rewards would happen here
    }
}

impl EventRewards {
    fn default() -> Self {
        Self {
            participation_rewards: RewardTier {
                rank_range: (0, 0),
                money: 1000,
                experience: 100,
                items: vec![],
                titles: vec![],
                special_currency: 10,
            },
            ranking_rewards: vec![
                RewardTier {
                    rank_range: (1, 1),
                    money: 100000,
                    experience: 10000,
                    items: vec!["legendary_item".to_string()],
                    titles: vec!["Event Champion".to_string()],
                    special_currency: 1000,
                },
                RewardTier {
                    rank_range: (2, 10),
                    money: 50000,
                    experience: 5000,
                    items: vec!["epic_item".to_string()],
                    titles: vec!["Top 10".to_string()],
                    special_currency: 500,
                },
            ],
            milestone_rewards: vec![],
            random_drops: vec![],
        }
    }
}

impl EventRules {
    fn default() -> Self {
        Self {
            pvp_enabled: false,
            clan_competition: false,
            level_brackets: false,
            special_modifiers: vec![],
            restricted_items: vec![],
            bonus_objectives: vec![],
        }
    }
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            active_events: Vec::new(),
            scheduled_events: Vec::new(),
            event_history: Vec::new(),
            participant_stats: HashMap::new(),
        }
    }

    /// Schedule a new event
    pub fn schedule_event(&mut self, event: GlobalEvent, start_time: DateTime<Utc>) {
        let scheduled = ScheduledEvent {
            event,
            announcement_time: start_time - Duration::hours(24),
            start_time,
            recurring: false,
            recurrence_pattern: None,
        };

        self.scheduled_events.push(scheduled);
    }

    /// Check and start scheduled events
    pub fn check_scheduled_events(&mut self) {
        let now = Utc::now();
        let mut to_start = Vec::new();

        for (idx, scheduled) in self.scheduled_events.iter().enumerate() {
            if scheduled.start_time <= now {
                to_start.push(idx);
            }
        }

        // Start events (in reverse to maintain indices)
        for idx in to_start.iter().rev() {
            let mut scheduled = self.scheduled_events.remove(*idx);
            scheduled.event.start();
            self.active_events.push(scheduled.event.clone());

            // If recurring, schedule next occurrence
            if scheduled.recurring {
                if let Some(pattern) = &scheduled.recurrence_pattern {
                    let next_start = calculate_next_occurrence(scheduled.start_time, pattern);
                    scheduled.start_time = next_start;
                    scheduled.announcement_time = next_start - Duration::hours(24);
                    self.scheduled_events.push(scheduled);
                }
            }
        }
    }

    /// End completed events
    pub fn check_ended_events(&mut self) {
        let now = Utc::now();
        let mut to_end = Vec::new();

        for (idx, event) in self.active_events.iter().enumerate() {
            if event.ends_at <= now {
                to_end.push(idx);
            }
        }

        for idx in to_end.iter().rev() {
            let mut event = self.active_events.remove(*idx);
            event.end();
            self.event_history.push(event);
        }
    }
}

fn calculate_next_occurrence(current: DateTime<Utc>, pattern: &RecurrencePattern) -> DateTime<Utc> {
    match pattern {
        RecurrencePattern::Daily => current + Duration::days(1),
        RecurrencePattern::Weekly { .. } => current + Duration::weeks(1),
        RecurrencePattern::Monthly { .. } => current + Duration::days(30),
        RecurrencePattern::Yearly { .. } => current + Duration::days(365),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_event() {
        let mut event = GlobalEvent::new(
            EventType::HackingTournament,
            "Weekend Tournament".to_string(),
            48,
        );

        event.start();
        assert_eq!(event.status, EventStatus::Active);

        // Add participants
        let player_id = Uuid::new_v4();
        event.add_participant(player_id, "Player1".to_string());
        assert!(event.participation.participant_list.contains(&player_id));

        // Update score
        event.update_score(player_id, 100);
        assert_eq!(event.leaderboard[0].score, 100);
    }
}