//! Clan system mechanics - Warfare mechanics, reputation formulas, contribution tracking

use crate::{PlayerState};
use crate::config::ClanConfig;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Clan ranks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ClanRank {
    Recruit = 0,
    Member = 1,
    Elite = 2,
    Officer = 3,
    Commander = 4,
    ViceLeader = 5,
    Leader = 6,
}

impl ClanRank {
    pub fn from_level(level: i32) -> Self {
        match level {
            0 => ClanRank::Recruit,
            1 => ClanRank::Member,
            2 => ClanRank::Elite,
            3 => ClanRank::Officer,
            4 => ClanRank::Commander,
            5 => ClanRank::ViceLeader,
            6 => ClanRank::Leader,
            _ => ClanRank::Member,
        }
    }
    
    pub fn permissions(&self) -> ClanPermissions {
        match self {
            ClanRank::Recruit => ClanPermissions {
                can_invite: false,
                can_kick: false,
                can_promote: false,
                can_start_war: false,
                can_manage_resources: false,
                can_edit_description: false,
                can_manage_alliances: false,
            },
            ClanRank::Member => ClanPermissions {
                can_invite: false,
                can_kick: false,
                can_promote: false,
                can_start_war: false,
                can_manage_resources: false,
                can_edit_description: false,
                can_manage_alliances: false,
            },
            ClanRank::Elite => ClanPermissions {
                can_invite: true,
                can_kick: false,
                can_promote: false,
                can_start_war: false,
                can_manage_resources: false,
                can_edit_description: false,
                can_manage_alliances: false,
            },
            ClanRank::Officer => ClanPermissions {
                can_invite: true,
                can_kick: true,
                can_promote: false,
                can_start_war: false,
                can_manage_resources: true,
                can_edit_description: true,
                can_manage_alliances: false,
            },
            ClanRank::Commander => ClanPermissions {
                can_invite: true,
                can_kick: true,
                can_promote: true,
                can_start_war: true,
                can_manage_resources: true,
                can_edit_description: true,
                can_manage_alliances: false,
            },
            ClanRank::ViceLeader => ClanPermissions {
                can_invite: true,
                can_kick: true,
                can_promote: true,
                can_start_war: true,
                can_manage_resources: true,
                can_edit_description: true,
                can_manage_alliances: true,
            },
            ClanRank::Leader => ClanPermissions {
                can_invite: true,
                can_kick: true,
                can_promote: true,
                can_start_war: true,
                can_manage_resources: true,
                can_edit_description: true,
                can_manage_alliances: true,
            },
        }
    }
    
    pub fn contribution_multiplier(&self) -> f32 {
        match self {
            ClanRank::Recruit => 0.8,
            ClanRank::Member => 1.0,
            ClanRank::Elite => 1.2,
            ClanRank::Officer => 1.5,
            ClanRank::Commander => 1.8,
            ClanRank::ViceLeader => 2.0,
            ClanRank::Leader => 2.5,
        }
    }
}

/// Clan permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanPermissions {
    pub can_invite: bool,
    pub can_kick: bool,
    pub can_promote: bool,
    pub can_start_war: bool,
    pub can_manage_resources: bool,
    pub can_edit_description: bool,
    pub can_manage_alliances: bool,
}

/// Clan member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanMember {
    pub player_id: i32,
    pub player_name: String,
    pub rank: ClanRank,
    pub joined_at: SystemTime,
    pub last_active: SystemTime,
    pub contribution_points: i64,
    pub wars_participated: i32,
    pub resources_donated: i64,
    pub enemies_defeated: i32,
    pub daily_activity: f32,
    pub warnings: i32,
    pub notes: String,
}

impl ClanMember {
    pub fn new(player_id: i32, player_name: String) -> Self {
        ClanMember {
            player_id,
            player_name,
            rank: ClanRank::Recruit,
            joined_at: SystemTime::now(),
            last_active: SystemTime::now(),
            contribution_points: 0,
            wars_participated: 0,
            resources_donated: 0,
            enemies_defeated: 0,
            daily_activity: 0.0,
            warnings: 0,
            notes: String::new(),
        }
    }
    
    pub fn promote(&mut self) -> Result<(), String> {
        self.rank = match self.rank {
            ClanRank::Recruit => ClanRank::Member,
            ClanRank::Member => ClanRank::Elite,
            ClanRank::Elite => ClanRank::Officer,
            ClanRank::Officer => ClanRank::Commander,
            ClanRank::Commander => ClanRank::ViceLeader,
            ClanRank::ViceLeader => return Err("Cannot promote vice leader".to_string()),
            ClanRank::Leader => return Err("Cannot promote leader".to_string()),
        };
        Ok(())
    }
    
    pub fn demote(&mut self) -> Result<(), String> {
        self.rank = match self.rank {
            ClanRank::Recruit => return Err("Cannot demote recruit".to_string()),
            ClanRank::Member => ClanRank::Recruit,
            ClanRank::Elite => ClanRank::Member,
            ClanRank::Officer => ClanRank::Elite,
            ClanRank::Commander => ClanRank::Officer,
            ClanRank::ViceLeader => ClanRank::Commander,
            ClanRank::Leader => return Err("Cannot demote leader".to_string()),
        };
        Ok(())
    }
    
    pub fn add_contribution(&mut self, points: i64) {
        self.contribution_points += points;
        self.last_active = SystemTime::now();
    }
    
    pub fn calculate_activity_score(&self) -> f32 {
        let days_since_join = SystemTime::now()
            .duration_since(self.joined_at)
            .unwrap_or_default()
            .as_secs() / 86400;
        
        if days_since_join == 0 {
            return 100.0;
        }
        
        let avg_contribution = self.contribution_points as f32 / days_since_join.max(1) as f32;
        let activity_factor = self.daily_activity;
        let war_factor = self.wars_participated as f32 * 10.0;
        
        (avg_contribution + activity_factor + war_factor).min(100.0)
    }
}

/// Clan resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanResources {
    pub money: i64,
    pub bitcoin: f64,
    pub cpu_power: i64,
    pub bandwidth: i64,
    pub storage: i64,
    pub research_points: i64,
    pub war_funds: i64,
}

impl ClanResources {
    pub fn new() -> Self {
        ClanResources {
            money: 0,
            bitcoin: 0.0,
            cpu_power: 0,
            bandwidth: 0,
            storage: 0,
            research_points: 0,
            war_funds: 0,
        }
    }
    
    pub fn donate_money(&mut self, amount: i64) {
        self.money += amount;
        self.war_funds += amount / 10; // 10% goes to war funds
    }
    
    pub fn donate_resources(&mut self, cpu: i64, bandwidth: i64, storage: i64) {
        self.cpu_power += cpu;
        self.bandwidth += bandwidth;
        self.storage += storage;
    }
    
    pub fn spend_war_funds(&mut self, amount: i64) -> Result<(), String> {
        if self.war_funds >= amount {
            self.war_funds -= amount;
            Ok(())
        } else {
            Err("Insufficient war funds".to_string())
        }
    }
    
    pub fn calculate_power_level(&self) -> i64 {
        self.money / 1000 +
        (self.bitcoin * 10000.0) as i64 +
        self.cpu_power / 100 +
        self.bandwidth / 50 +
        self.storage / 200 +
        self.research_points * 10
    }
}

/// Clan war status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarStatus {
    Pending,
    Active,
    CeaseFire,
    Victory,
    Defeat,
    Draw,
}

/// Clan war
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanWar {
    pub id: Uuid,
    pub attacker_clan_id: Uuid,
    pub defender_clan_id: Uuid,
    pub status: WarStatus,
    pub started_at: SystemTime,
    pub ends_at: SystemTime,
    pub attacker_score: i64,
    pub defender_score: i64,
    pub battles: Vec<WarBattle>,
    pub war_goals: Vec<WarGoal>,
    pub stakes: WarStakes,
    pub ceasefire_offered_by: Option<Uuid>,
    pub surrender_offered_by: Option<Uuid>,
}

impl ClanWar {
    pub fn new(
        attacker_clan_id: Uuid,
        defender_clan_id: Uuid,
        duration: Duration,
        stakes: WarStakes,
    ) -> Self {
        ClanWar {
            id: Uuid::new_v4(),
            attacker_clan_id,
            defender_clan_id,
            status: WarStatus::Pending,
            started_at: SystemTime::now(),
            ends_at: SystemTime::now() + duration,
            attacker_score: 0,
            defender_score: 0,
            battles: Vec::new(),
            war_goals: Vec::new(),
            stakes,
            ceasefire_offered_by: None,
            surrender_offered_by: None,
        }
    }
    
    pub fn start(&mut self) {
        self.status = WarStatus::Active;
        self.started_at = SystemTime::now();
    }
    
    pub fn add_battle(&mut self, battle: WarBattle) {
        if battle.winner_clan_id == self.attacker_clan_id {
            self.attacker_score += battle.points_earned;
        } else if battle.winner_clan_id == self.defender_clan_id {
            self.defender_score += battle.points_earned;
        }
        self.battles.push(battle);
    }
    
    pub fn check_war_end(&mut self) -> bool {
        if SystemTime::now() > self.ends_at {
            if self.attacker_score > self.defender_score {
                self.status = WarStatus::Victory;
            } else if self.defender_score > self.attacker_score {
                self.status = WarStatus::Defeat;
            } else {
                self.status = WarStatus::Draw;
            }
            return true;
        }
        
        // Check if any war goals are met
        for goal in &self.war_goals {
            if goal.is_completed {
                self.status = if goal.owner_clan_id == self.attacker_clan_id {
                    WarStatus::Victory
                } else {
                    WarStatus::Defeat
                };
                return true;
            }
        }
        
        false
    }
    
    pub fn offer_ceasefire(&mut self, clan_id: Uuid) {
        self.ceasefire_offered_by = Some(clan_id);
    }
    
    pub fn accept_ceasefire(&mut self) -> Result<(), String> {
        if self.ceasefire_offered_by.is_some() {
            self.status = WarStatus::CeaseFire;
            Ok(())
        } else {
            Err("No ceasefire offer pending".to_string())
        }
    }
    
    pub fn calculate_winner(&self) -> Option<Uuid> {
        match self.status {
            WarStatus::Victory => Some(self.attacker_clan_id),
            WarStatus::Defeat => Some(self.defender_clan_id),
            _ => None,
        }
    }
}

/// War battle record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarBattle {
    pub id: Uuid,
    pub attacker_id: i32,
    pub defender_id: i32,
    pub winner_clan_id: Uuid,
    pub battle_type: BattleType,
    pub points_earned: i64,
    pub timestamp: SystemTime,
    pub details: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleType {
    ServerHack,
    DataTheft,
    DDoSAttack,
    Sabotage,
    Defense,
    CounterAttack,
    Espionage,
}

/// War goals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarGoal {
    pub id: Uuid,
    pub owner_clan_id: Uuid,
    pub goal_type: WarGoalType,
    pub description: String,
    pub target_value: i64,
    pub current_value: i64,
    pub is_completed: bool,
    pub reward_multiplier: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarGoalType {
    DestroyServers(i32),
    StealData(i64),
    CaptureMembers(i32),
    ReachScore(i64),
    DefendFor(Duration),
    Custom(String),
}

/// War stakes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarStakes {
    pub money_stake: i64,
    pub resource_percentage: f32,
    pub member_poaching: bool,
    pub territory_control: Vec<String>,
    pub reputation_points: i32,
}

/// Clan alliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanAlliance {
    pub id: Uuid,
    pub name: String,
    pub member_clans: HashSet<Uuid>,
    pub leader_clan: Uuid,
    pub formed_at: SystemTime,
    pub alliance_type: AllianceType,
    pub shared_resources: bool,
    pub mutual_defense: bool,
    pub trade_agreements: Vec<TradeAgreement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllianceType {
    Defensive,
    Offensive,
    Economic,
    Research,
    Full,
}

/// Trade agreement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeAgreement {
    pub id: Uuid,
    pub clan1_id: Uuid,
    pub clan2_id: Uuid,
    pub resource_type: String,
    pub exchange_rate: f32,
    pub duration: Duration,
    pub auto_renew: bool,
}

/// Main clan structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clan {
    pub id: Uuid,
    pub name: String,
    pub tag: String,
    pub description: String,
    pub motto: String,
    pub founded_at: SystemTime,
    pub founder_id: i32,
    pub leader_id: i32,
    pub members: HashMap<i32, ClanMember>,
    pub max_members: usize,
    pub resources: ClanResources,
    pub reputation: i64,
    pub level: i32,
    pub experience: i64,
    pub wars: Vec<Uuid>,
    pub alliances: HashSet<Uuid>,
    pub enemies: HashSet<Uuid>,
    pub pending_applications: HashMap<i32, ClanApplication>,
    pub achievements: Vec<ClanAchievement>,
    pub headquarters_ip: Option<String>,
    pub is_recruiting: bool,
    pub minimum_level: i32,
    pub tax_rate: f32,
    pub activity_log: VecDeque<ClanActivityLog>,
}

impl Clan {
    pub fn new(name: String, tag: String, founder_id: i32, founder_name: String) -> Self {
        let mut members = HashMap::new();
        let mut founder = ClanMember::new(founder_id, founder_name);
        founder.rank = ClanRank::Leader;
        members.insert(founder_id, founder);
        
        Clan {
            id: Uuid::new_v4(),
            name,
            tag,
            description: String::new(),
            motto: String::new(),
            founded_at: SystemTime::now(),
            founder_id,
            leader_id: founder_id,
            members,
            max_members: 20,
            resources: ClanResources::new(),
            reputation: 1000,
            level: 1,
            experience: 0,
            wars: Vec::new(),
            alliances: HashSet::new(),
            enemies: HashSet::new(),
            pending_applications: HashMap::new(),
            achievements: Vec::new(),
            headquarters_ip: None,
            is_recruiting: true,
            minimum_level: 1,
            tax_rate: 0.1,
            activity_log: VecDeque::with_capacity(100),
        }
    }
    
    pub fn add_member(&mut self, player_id: i32, player_name: String) -> Result<(), String> {
        if self.members.len() >= self.max_members {
            return Err("Clan is full".to_string());
        }
        
        if self.members.contains_key(&player_id) {
            return Err("Player already in clan".to_string());
        }
        
        let member = ClanMember::new(player_id, player_name.clone());
        self.members.insert(player_id, member);
        
        self.log_activity(ClanActivityLog {
            timestamp: SystemTime::now(),
            activity_type: ActivityType::MemberJoined,
            actor_id: player_id,
            details: format!("{} joined the clan", player_name),
        });
        
        Ok(())
    }
    
    pub fn remove_member(&mut self, player_id: i32) -> Result<(), String> {
        let member = self.members.remove(&player_id)
            .ok_or("Member not found")?;
        
        if player_id == self.leader_id {
            // Need to assign new leader
            if let Some(new_leader) = self.find_next_leader() {
                self.leader_id = new_leader;
                self.members.get_mut(&new_leader).unwrap().rank = ClanRank::Leader;
            } else {
                return Err("Cannot remove last member".to_string());
            }
        }
        
        self.log_activity(ClanActivityLog {
            timestamp: SystemTime::now(),
            activity_type: ActivityType::MemberLeft,
            actor_id: player_id,
            details: format!("{} left the clan", member.player_name),
        });
        
        Ok(())
    }
    
    pub fn promote_member(&mut self, player_id: i32, promoter_id: i32) -> Result<(), String> {
        // Check permissions
        let promoter = self.members.get(&promoter_id)
            .ok_or("Promoter not found")?;
        
        if !promoter.rank.permissions().can_promote {
            return Err("No permission to promote".to_string());
        }
        
        let member = self.members.get_mut(&player_id)
            .ok_or("Member not found")?;
        
        // Can't promote above your own rank
        if member.rank >= promoter.rank {
            return Err("Cannot promote to or above your rank".to_string());
        }
        
        let old_rank = member.rank;
        member.promote()?;
        
        self.log_activity(ClanActivityLog {
            timestamp: SystemTime::now(),
            activity_type: ActivityType::Promotion,
            actor_id: promoter_id,
            details: format!("{} promoted from {:?} to {:?}", 
                member.player_name, old_rank, member.rank),
        });
        
        Ok(())
    }
    
    pub fn start_war(&mut self, target_clan_id: Uuid, stakes: WarStakes) -> ClanWar {
        let war = ClanWar::new(
            self.id,
            target_clan_id,
            Duration::from_secs(3 * 24 * 3600), // 3 days default
            stakes,
        );
        
        self.wars.push(war.id);
        self.enemies.insert(target_clan_id);
        
        self.log_activity(ClanActivityLog {
            timestamp: SystemTime::now(),
            activity_type: ActivityType::WarDeclared,
            actor_id: self.leader_id,
            details: format!("War declared against clan {}", target_clan_id),
        });
        
        war
    }
    
    pub fn donate_resources(&mut self, player_id: i32, money: i64) -> Result<i64, String> {
        let member = self.members.get_mut(&player_id)
            .ok_or("Member not found")?;
        
        let contribution_points = (money as f32 * member.rank.contribution_multiplier()) as i64;
        member.add_contribution(contribution_points);
        member.resources_donated += money;
        
        self.resources.donate_money(money);
        self.add_experience(contribution_points / 10);
        
        self.log_activity(ClanActivityLog {
            timestamp: SystemTime::now(),
            activity_type: ActivityType::Donation,
            actor_id: player_id,
            details: format!("{} donated ${}", member.player_name, money),
        });
        
        Ok(contribution_points)
    }
    
    pub fn calculate_power(&self) -> i64 {
        let member_power: i64 = self.members.values()
            .map(|m| m.contribution_points / 100)
            .sum();
        
        let resource_power = self.resources.calculate_power_level();
        let reputation_power = self.reputation;
        let level_power = self.level as i64 * 1000;
        
        member_power + resource_power + reputation_power + level_power
    }
    
    pub fn add_experience(&mut self, exp: i64) {
        self.experience += exp;
        
        // Level up check
        let required_exp = self.calculate_level_requirement();
        if self.experience >= required_exp {
            self.level += 1;
            self.max_members += 5;
            self.experience -= required_exp;
            
            self.log_activity(ClanActivityLog {
                timestamp: SystemTime::now(),
                activity_type: ActivityType::LevelUp,
                actor_id: 0,
                details: format!("Clan reached level {}", self.level),
            });
        }
    }
    
    fn calculate_level_requirement(&self) -> i64 {
        (1000 * self.level * self.level) as i64
    }
    
    fn find_next_leader(&self) -> Option<i32> {
        // Find highest ranking member after leader
        self.members.iter()
            .filter(|(id, _)| **id != self.leader_id)
            .max_by_key(|(_, member)| member.rank)
            .map(|(id, _)| *id)
    }
    
    fn log_activity(&mut self, log: ClanActivityLog) {
        if self.activity_log.len() >= 100 {
            self.activity_log.pop_front();
        }
        self.activity_log.push_back(log);
    }
}

/// Clan application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanApplication {
    pub player_id: i32,
    pub player_name: String,
    pub player_level: i32,
    pub message: String,
    pub applied_at: SystemTime,
}

/// Clan achievement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanAchievement {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub earned_at: SystemTime,
    pub achievement_type: AchievementType,
    pub reward: ClanReward,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AchievementType {
    MemberMilestone(i32),
    WarVictories(i32),
    ResourceGoal(i64),
    ReputationLevel(i64),
    AllianceFormed,
    Custom(String),
}

/// Clan rewards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanReward {
    pub reputation: i64,
    pub resources: i64,
    pub member_slots: i32,
    pub special_perks: Vec<String>,
}

/// Clan activity log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanActivityLog {
    pub timestamp: SystemTime,
    pub activity_type: ActivityType,
    pub actor_id: i32,
    pub details: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityType {
    MemberJoined,
    MemberLeft,
    MemberKicked,
    Promotion,
    Demotion,
    Donation,
    WarDeclared,
    WarWon,
    WarLost,
    AllianceFormed,
    AllianceBroken,
    LevelUp,
    AchievementEarned,
    Custom(String),
}

/// Calculate clan reputation
pub fn calculate_clan_reputation(clan: &Clan, config: &ClanConfig) -> i64 {
    let base_reputation = clan.reputation;
    
    // Member contribution
    let member_contribution: i64 = clan.members.values()
        .map(|m| m.contribution_points / 1000)
        .sum();
    
    // War score
    let war_bonus = clan.wars.len() as i64 * config.war_reputation_bonus;
    
    // Level bonus
    let level_bonus = clan.level as i64 * config.level_reputation_multiplier;
    
    // Activity bonus
    let active_members = clan.members.values()
        .filter(|m| {
            SystemTime::now().duration_since(m.last_active)
                .unwrap_or_default().as_secs() < 86400
        })
        .count() as i64;
    
    let activity_bonus = active_members * config.activity_reputation_bonus;
    
    base_reputation + member_contribution + war_bonus + level_bonus + activity_bonus
}

/// Process clan warfare
pub fn process_clan_warfare(
    war: &mut ClanWar,
    attacker_clan: &mut Clan,
    defender_clan: &mut Clan,
    config: &ClanConfig
) -> i64 {
    if war.status != WarStatus::Active {
        return 0;
    }
    
    // Check if war should end
    if war.check_war_end() {
        let winner_id = war.calculate_winner();
        
        if let Some(winner) = winner_id {
            if winner == attacker_clan.id {
                attacker_clan.reputation += config.war_victory_reputation;
                defender_clan.reputation -= config.war_defeat_reputation;
                
                // Transfer stakes
                if war.stakes.money_stake > 0 {
                    attacker_clan.resources.money += war.stakes.money_stake;
                    defender_clan.resources.money -= war.stakes.money_stake;
                }
            } else {
                defender_clan.reputation += config.war_victory_reputation;
                attacker_clan.reputation -= config.war_defeat_reputation;
                
                // Transfer stakes
                if war.stakes.money_stake > 0 {
                    defender_clan.resources.money += war.stakes.money_stake;
                    attacker_clan.resources.money -= war.stakes.money_stake;
                }
            }
        }
        
        return war.attacker_score + war.defender_score;
    }
    
    // War is still active, return current score
    war.attacker_score + war.defender_score
}

/// Calculate member contribution
pub fn calculate_member_contribution(
    member: &ClanMember,
    action: ContributionAction,
    config: &ClanConfig
) -> i64 {
    let base_points = match action {
        ContributionAction::Donation(amount) => amount / 100,
        ContributionAction::WarParticipation => config.war_participation_points,
        ContributionAction::WarVictory => config.war_victory_points,
        ContributionAction::MemberRecruit => config.recruit_points,
        ContributionAction::ResourceGather(amount) => amount / 50,
        ContributionAction::ResearchContribution(points) => points * 2,
        ContributionAction::DefenseSuccess => config.defense_points,
        ContributionAction::Custom(points) => points,
    };
    
    (base_points as f32 * member.rank.contribution_multiplier()) as i64
}

#[derive(Debug, Clone)]
pub enum ContributionAction {
    Donation(i64),
    WarParticipation,
    WarVictory,
    MemberRecruit,
    ResourceGather(i64),
    ResearchContribution(i64),
    DefenseSuccess,
    Custom(i64),
}

/// Generate clan rankings
pub fn generate_clan_rankings(clans: &[Clan]) -> Vec<(Uuid, i64)> {
    let mut rankings: Vec<(Uuid, i64)> = clans.iter()
        .map(|clan| (clan.id, clan.calculate_power()))
        .collect();
    
    rankings.sort_by_key(|&(_, power)| -power);
    rankings
}

/// Check alliance compatibility
pub fn check_alliance_compatibility(clan1: &Clan, clan2: &Clan) -> Result<(), String> {
    // Can't ally with enemies
    if clan1.enemies.contains(&clan2.id) || clan2.enemies.contains(&clan1.id) {
        return Err("Cannot form alliance with enemy clan".to_string());
    }
    
    // Check if already allied
    if clan1.alliances.contains(&clan2.id) {
        return Err("Already allied".to_string());
    }
    
    // Check reputation difference
    let rep_diff = (clan1.reputation - clan2.reputation).abs();
    if rep_diff > 10000 {
        return Err("Reputation difference too large".to_string());
    }
    
    Ok(())
}

/// Process clan taxes
pub fn process_clan_taxes(clan: &mut Clan, member_earnings: HashMap<i32, i64>) -> i64 {
    let mut total_tax = 0;
    
    for (member_id, earnings) in member_earnings {
        if clan.members.contains_key(&member_id) {
            let tax = (earnings as f32 * clan.tax_rate) as i64;
            clan.resources.money += tax;
            total_tax += tax;
            
            // Add contribution points for tax paid
            if let Some(member) = clan.members.get_mut(&member_id) {
                member.add_contribution(tax / 10);
            }
        }
    }
    
    total_tax
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_clan_creation() {
        let clan = Clan::new(
            "Test Clan".to_string(),
            "TEST".to_string(),
            1,
            "Founder".to_string()
        );
        
        assert_eq!(clan.name, "Test Clan");
        assert_eq!(clan.tag, "TEST");
        assert_eq!(clan.members.len(), 1);
        assert_eq!(clan.leader_id, 1);
    }
    
    #[test]
    fn test_member_management() {
        let mut clan = Clan::new(
            "Test Clan".to_string(),
            "TEST".to_string(),
            1,
            "Founder".to_string()
        );
        
        // Add member
        assert!(clan.add_member(2, "Member2".to_string()).is_ok());
        assert_eq!(clan.members.len(), 2);
        
        // Remove member
        assert!(clan.remove_member(2).is_ok());
        assert_eq!(clan.members.len(), 1);
    }
    
    #[test]
    fn test_rank_permissions() {
        let recruit_perms = ClanRank::Recruit.permissions();
        assert!(!recruit_perms.can_invite);
        assert!(!recruit_perms.can_kick);
        
        let leader_perms = ClanRank::Leader.permissions();
        assert!(leader_perms.can_invite);
        assert!(leader_perms.can_kick);
        assert!(leader_perms.can_start_war);
    }
    
    #[test]
    fn test_war_creation() {
        let attacker_id = Uuid::new_v4();
        let defender_id = Uuid::new_v4();
        
        let stakes = WarStakes {
            money_stake: 10000,
            resource_percentage: 0.1,
            member_poaching: false,
            territory_control: vec![],
            reputation_points: 100,
        };
        
        let mut war = ClanWar::new(
            attacker_id,
            defender_id,
            Duration::from_secs(86400),
            stakes
        );
        
        assert_eq!(war.status, WarStatus::Pending);
        war.start();
        assert_eq!(war.status, WarStatus::Active);
    }
    
    #[test]
    fn test_contribution_calculation() {
        let mut member = ClanMember::new(1, "Test".to_string());
        member.rank = ClanRank::Elite;
        
        let config = ClanConfig::default();
        let contribution = calculate_member_contribution(
            &member,
            ContributionAction::Donation(10000),
            &config
        );
        
        assert!(contribution > 0);
        // Elite rank has 1.2x multiplier
        assert_eq!(contribution, (10000 / 100) * 1.2 as i64);
    }
    
    #[test]
    fn test_resource_management() {
        let mut resources = ClanResources::new();
        
        resources.donate_money(10000);
        assert_eq!(resources.money, 10000);
        assert_eq!(resources.war_funds, 1000); // 10% to war funds
        
        assert!(resources.spend_war_funds(500).is_ok());
        assert_eq!(resources.war_funds, 500);
        
        assert!(resources.spend_war_funds(1000).is_err());
    }
}