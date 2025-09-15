//! Mission system mechanics - Difficulty scaling, reward calculations, prerequisites

use crate::{PlayerState, TargetInfo};
use crate::config::MissionConfig;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Mission types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MissionType {
    // Tutorial Missions
    TutorialBasicHack,
    TutorialInstallSoftware,
    TutorialUpgradeHardware,
    TutorialJoinClan,
    
    // Hacking Missions
    HackServer,
    StealData,
    PlantVirus,
    DeleteLogs,
    CrackPassword,
    BypassFirewall,
    
    // Infiltration Missions
    InfiltrateCompany,
    StealCorporateSecrets,
    SabotageOperations,
    IndustrialEspionage,
    
    // Financial Missions
    BankHeist,
    CryptoTheft,
    MoneyLaundering,
    MarketManipulation,
    
    // Defensive Missions
    DefendServer,
    RemoveVirus,
    TraceAttacker,
    SecureNetwork,
    
    // Investigation Missions
    TrackHacker,
    GatherIntelligence,
    UnmaskIdentity,
    RecoverStolenData,
    
    // Destruction Missions
    DDoSAttack,
    WipeServer,
    DestroyBackups,
    SystemSabotage,
    
    // Special Missions
    GovernmentContract,
    BlackOps,
    CorporateWarfare,
    EliteChallenge,
    
    // Story Missions
    MainStory(i32),
    SideStory(String),
    
    Custom(String),
}

impl MissionType {
    pub fn difficulty_base(&self) -> i32 {
        match self {
            // Tutorial missions are easy
            MissionType::TutorialBasicHack => 1,
            MissionType::TutorialInstallSoftware => 1,
            MissionType::TutorialUpgradeHardware => 2,
            MissionType::TutorialJoinClan => 2,
            
            // Basic missions
            MissionType::HackServer => 3,
            MissionType::StealData => 4,
            MissionType::PlantVirus => 4,
            MissionType::DeleteLogs => 3,
            MissionType::CrackPassword => 5,
            MissionType::BypassFirewall => 5,
            
            // Intermediate missions
            MissionType::InfiltrateCompany => 6,
            MissionType::StealCorporateSecrets => 7,
            MissionType::SabotageOperations => 7,
            MissionType::IndustrialEspionage => 8,
            
            // Advanced missions
            MissionType::BankHeist => 9,
            MissionType::CryptoTheft => 8,
            MissionType::MoneyLaundering => 7,
            MissionType::MarketManipulation => 9,
            
            // Defensive missions (varied difficulty)
            MissionType::DefendServer => 5,
            MissionType::RemoveVirus => 4,
            MissionType::TraceAttacker => 6,
            MissionType::SecureNetwork => 7,
            
            // Investigation missions
            MissionType::TrackHacker => 6,
            MissionType::GatherIntelligence => 5,
            MissionType::UnmaskIdentity => 7,
            MissionType::RecoverStolenData => 8,
            
            // Destruction missions
            MissionType::DDoSAttack => 6,
            MissionType::WipeServer => 8,
            MissionType::DestroyBackups => 7,
            MissionType::SystemSabotage => 9,
            
            // Special missions are hardest
            MissionType::GovernmentContract => 10,
            MissionType::BlackOps => 10,
            MissionType::CorporateWarfare => 9,
            MissionType::EliteChallenge => 10,
            
            // Story missions scale
            MissionType::MainStory(chapter) => 3 + (chapter / 2).min(7),
            MissionType::SideStory(_) => 5,
            
            MissionType::Custom(_) => 5,
        }
    }
    
    pub fn category(&self) -> MissionCategory {
        match self {
            MissionType::TutorialBasicHack | MissionType::TutorialInstallSoftware |
            MissionType::TutorialUpgradeHardware | MissionType::TutorialJoinClan => MissionCategory::Tutorial,
            
            MissionType::HackServer | MissionType::StealData | MissionType::PlantVirus |
            MissionType::DeleteLogs | MissionType::CrackPassword | MissionType::BypassFirewall => MissionCategory::Hacking,
            
            MissionType::InfiltrateCompany | MissionType::StealCorporateSecrets |
            MissionType::SabotageOperations | MissionType::IndustrialEspionage => MissionCategory::Infiltration,
            
            MissionType::BankHeist | MissionType::CryptoTheft |
            MissionType::MoneyLaundering | MissionType::MarketManipulation => MissionCategory::Financial,
            
            MissionType::DefendServer | MissionType::RemoveVirus |
            MissionType::TraceAttacker | MissionType::SecureNetwork => MissionCategory::Defensive,
            
            MissionType::TrackHacker | MissionType::GatherIntelligence |
            MissionType::UnmaskIdentity | MissionType::RecoverStolenData => MissionCategory::Investigation,
            
            MissionType::DDoSAttack | MissionType::WipeServer |
            MissionType::DestroyBackups | MissionType::SystemSabotage => MissionCategory::Destruction,
            
            MissionType::GovernmentContract | MissionType::BlackOps |
            MissionType::CorporateWarfare | MissionType::EliteChallenge => MissionCategory::Special,
            
            MissionType::MainStory(_) | MissionType::SideStory(_) => MissionCategory::Story,
            
            MissionType::Custom(_) => MissionCategory::Custom,
        }
    }
}

/// Mission categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionCategory {
    Tutorial,
    Hacking,
    Infiltration,
    Financial,
    Defensive,
    Investigation,
    Destruction,
    Special,
    Story,
    Custom,
}

/// Mission status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionStatus {
    Locked,
    Available,
    Active,
    Completed,
    Failed,
    Expired,
    Abandoned,
}

/// Mission objective types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectiveType {
    HackTarget(String),
    StealFile(String),
    DeleteFile(String),
    InstallSoftware(String),
    UploadFile(String),
    TransferMoney(i32),
    MaintainAccess(Duration),
    DefendFor(Duration),
    TraceIP(String),
    CollectData(String, i32),
    DestroyTarget(String),
    ProtectTarget(String),
    Custom(String),
}

/// Individual mission objective
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionObjective {
    pub id: Uuid,
    pub objective_type: ObjectiveType,
    pub description: String,
    pub is_completed: bool,
    pub is_optional: bool,
    pub progress: i32,
    pub target_progress: i32,
    pub bonus_reward: Option<MissionReward>,
}

impl MissionObjective {
    pub fn new(objective_type: ObjectiveType, description: String, target_progress: i32) -> Self {
        MissionObjective {
            id: Uuid::new_v4(),
            objective_type,
            description,
            is_completed: false,
            is_optional: false,
            progress: 0,
            target_progress,
            bonus_reward: None,
        }
    }
    
    pub fn with_bonus(mut self, reward: MissionReward) -> Self {
        self.bonus_reward = Some(reward);
        self
    }
    
    pub fn set_optional(mut self, optional: bool) -> Self {
        self.is_optional = optional;
        self
    }
    
    pub fn update_progress(&mut self, amount: i32) {
        self.progress = (self.progress + amount).min(self.target_progress);
        if self.progress >= self.target_progress {
            self.is_completed = true;
        }
    }
    
    pub fn get_completion_percentage(&self) -> f32 {
        if self.target_progress == 0 {
            return 100.0;
        }
        (self.progress as f32 / self.target_progress as f32 * 100.0).min(100.0)
    }
}

/// Mission rewards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionReward {
    pub money: i32,
    pub experience: i32,
    pub reputation: i32,
    pub bitcoin: f32,
    pub items: Vec<String>,
    pub software: Vec<String>,
    pub unlock_missions: Vec<MissionType>,
    pub special_rewards: HashMap<String, String>,
}

impl MissionReward {
    pub fn new() -> Self {
        MissionReward {
            money: 0,
            experience: 0,
            reputation: 0,
            bitcoin: 0.0,
            items: Vec::new(),
            software: Vec::new(),
            unlock_missions: Vec::new(),
            special_rewards: HashMap::new(),
        }
    }
    
    pub fn with_money(mut self, amount: i32) -> Self {
        self.money = amount;
        self
    }
    
    pub fn with_experience(mut self, amount: i32) -> Self {
        self.experience = amount;
        self
    }
    
    pub fn with_reputation(mut self, amount: i32) -> Self {
        self.reputation = amount;
        self
    }
    
    pub fn with_bitcoin(mut self, amount: f32) -> Self {
        self.bitcoin = amount;
        self
    }
    
    pub fn with_item(mut self, item: String) -> Self {
        self.items.push(item);
        self
    }
    
    pub fn with_software(mut self, software: String) -> Self {
        self.software.push(software);
        self
    }
    
    pub fn unlocks_mission(mut self, mission: MissionType) -> Self {
        self.unlock_missions.push(mission);
        self
    }
    
    pub fn calculate_total_value(&self) -> i32 {
        self.money + 
        self.experience * 10 + 
        self.reputation * 50 + 
        (self.bitcoin * 10000.0) as i32 +
        self.items.len() as i32 * 100 +
        self.software.len() as i32 * 500
    }
}

/// Mission instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub id: Uuid,
    pub mission_type: MissionType,
    pub name: String,
    pub description: String,
    pub briefing: String,
    pub status: MissionStatus,
    pub difficulty: i32,
    pub level_requirement: i32,
    pub time_limit: Option<Duration>,
    pub expires_at: Option<SystemTime>,
    pub started_at: Option<SystemTime>,
    pub completed_at: Option<SystemTime>,
    pub objectives: Vec<MissionObjective>,
    pub prerequisites: Vec<MissionPrerequisite>,
    pub rewards: MissionReward,
    pub failure_penalty: Option<MissionPenalty>,
    pub client_name: String,
    pub target_info: Option<TargetInfo>,
    pub attempts: i32,
    pub max_attempts: i32,
    pub is_repeatable: bool,
    pub cooldown: Option<Duration>,
    pub next_available: Option<SystemTime>,
}

impl Mission {
    pub fn new(mission_type: MissionType, name: String, description: String) -> Self {
        let difficulty = mission_type.difficulty_base();
        
        Mission {
            id: Uuid::new_v4(),
            mission_type,
            name,
            description: description.clone(),
            briefing: description,
            status: MissionStatus::Locked,
            difficulty,
            level_requirement: difficulty * 5,
            time_limit: None,
            expires_at: None,
            started_at: None,
            completed_at: None,
            objectives: Vec::new(),
            prerequisites: Vec::new(),
            rewards: MissionReward::new(),
            failure_penalty: None,
            client_name: "Anonymous".to_string(),
            target_info: None,
            attempts: 0,
            max_attempts: 3,
            is_repeatable: false,
            cooldown: None,
            next_available: None,
        }
    }
    
    pub fn with_time_limit(mut self, duration: Duration) -> Self {
        self.time_limit = Some(duration);
        self
    }
    
    pub fn with_level_requirement(mut self, level: i32) -> Self {
        self.level_requirement = level;
        self
    }
    
    pub fn with_objective(mut self, objective: MissionObjective) -> Self {
        self.objectives.push(objective);
        self
    }
    
    pub fn with_prerequisite(mut self, prereq: MissionPrerequisite) -> Self {
        self.prerequisites.push(prereq);
        self
    }
    
    pub fn with_reward(mut self, reward: MissionReward) -> Self {
        self.rewards = reward;
        self
    }
    
    pub fn with_penalty(mut self, penalty: MissionPenalty) -> Self {
        self.failure_penalty = Some(penalty);
        self
    }
    
    pub fn with_target(mut self, target: TargetInfo) -> Self {
        self.target_info = Some(target);
        self
    }
    
    pub fn set_repeatable(mut self, repeatable: bool, cooldown: Option<Duration>) -> Self {
        self.is_repeatable = repeatable;
        self.cooldown = cooldown;
        self
    }
    
    pub fn can_start(&self, player: &PlayerState) -> Result<(), String> {
        // Check status
        if self.status != MissionStatus::Available {
            return Err("Mission is not available".to_string());
        }
        
        // Check level requirement
        if player.level < self.level_requirement {
            return Err(format!("Requires level {}", self.level_requirement));
        }
        
        // Check prerequisites
        for prereq in &self.prerequisites {
            if !prereq.is_met(player) {
                return Err(format!("Prerequisite not met: {}", prereq.description()));
            }
        }
        
        // Check attempts
        if self.attempts >= self.max_attempts {
            return Err("Maximum attempts reached".to_string());
        }
        
        // Check cooldown
        if let Some(next) = self.next_available {
            if SystemTime::now() < next {
                return Err("Mission is on cooldown".to_string());
            }
        }
        
        Ok(())
    }
    
    pub fn start(&mut self) -> Result<(), String> {
        if self.status != MissionStatus::Available {
            return Err("Mission cannot be started".to_string());
        }
        
        self.status = MissionStatus::Active;
        self.started_at = Some(SystemTime::now());
        self.attempts += 1;
        
        // Set expiration if time limit exists
        if let Some(limit) = self.time_limit {
            self.expires_at = Some(SystemTime::now() + limit);
        }
        
        Ok(())
    }
    
    pub fn check_completion(&self) -> bool {
        // All required objectives must be completed
        self.objectives.iter()
            .filter(|obj| !obj.is_optional)
            .all(|obj| obj.is_completed)
    }
    
    pub fn complete(&mut self) -> Result<MissionReward, String> {
        if !self.check_completion() {
            return Err("Not all objectives completed".to_string());
        }
        
        self.status = MissionStatus::Completed;
        self.completed_at = Some(SystemTime::now());
        
        // Calculate bonus rewards for optional objectives
        let mut total_reward = self.rewards.clone();
        for obj in &self.objectives {
            if obj.is_optional && obj.is_completed {
                if let Some(bonus) = &obj.bonus_reward {
                    total_reward.money += bonus.money;
                    total_reward.experience += bonus.experience;
                    total_reward.reputation += bonus.reputation;
                    total_reward.bitcoin += bonus.bitcoin;
                }
            }
        }
        
        // Set cooldown for repeatable missions
        if self.is_repeatable {
            if let Some(cooldown) = self.cooldown {
                self.next_available = Some(SystemTime::now() + cooldown);
            }
        }
        
        Ok(total_reward)
    }
    
    pub fn fail(&mut self) -> Option<MissionPenalty> {
        self.status = MissionStatus::Failed;
        self.completed_at = Some(SystemTime::now());
        
        // Set cooldown even on failure
        if let Some(cooldown) = self.cooldown {
            self.next_available = Some(SystemTime::now() + cooldown);
        }
        
        self.failure_penalty.clone()
    }
    
    pub fn abandon(&mut self) {
        self.status = MissionStatus::Abandoned;
        self.completed_at = Some(SystemTime::now());
    }
    
    pub fn get_time_remaining(&self) -> Option<Duration> {
        if let Some(expires) = self.expires_at {
            SystemTime::now().duration_since(expires).ok()
        } else {
            None
        }
    }
    
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            SystemTime::now() > expires
        } else {
            false
        }
    }
    
    pub fn get_progress(&self) -> f32 {
        if self.objectives.is_empty() {
            return 0.0;
        }
        
        let required_objectives: Vec<_> = self.objectives.iter()
            .filter(|obj| !obj.is_optional)
            .collect();
        
        if required_objectives.is_empty() {
            return 100.0;
        }
        
        let completed = required_objectives.iter()
            .filter(|obj| obj.is_completed)
            .count() as f32;
        
        (completed / required_objectives.len() as f32 * 100.0).min(100.0)
    }
}

/// Mission prerequisites
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MissionPrerequisite {
    Level(i32),
    Money(i32),
    Reputation(i32),
    CompletedMission(MissionType),
    CompletedMissionCount(MissionCategory, i32),
    OwnsSoftware(String, f32), // Software name and minimum version
    OwnsHardware(String, i32), // Hardware type and minimum value
    ClanMembership,
    ClanRank(i32),
    Skill(String, i32), // Skill name and minimum level
    Custom(String),
}

impl MissionPrerequisite {
    pub fn is_met(&self, player: &PlayerState) -> bool {
        match self {
            MissionPrerequisite::Level(required) => player.level >= *required,
            MissionPrerequisite::Money(required) => player.money >= *required,
            MissionPrerequisite::Reputation(required) => player.reputation >= *required,
            MissionPrerequisite::CompletedMission(mission_type) => {
                player.completed_missions.contains(&format!("{:?}", mission_type))
            },
            MissionPrerequisite::CompletedMissionCount(category, count) => {
                // This would need mission tracking by category in PlayerState
                true // Placeholder
            },
            MissionPrerequisite::OwnsSoftware(name, version) => {
                player.software_levels.get(name)
                    .map(|v| *v >= *version)
                    .unwrap_or(false)
            },
            MissionPrerequisite::OwnsHardware(hw_type, min_value) => {
                // This would need hardware tracking in PlayerState
                true // Placeholder
            },
            MissionPrerequisite::ClanMembership => player.clan_id.is_some(),
            MissionPrerequisite::ClanRank(required) => {
                player.clan_rank.unwrap_or(0) >= *required
            },
            MissionPrerequisite::Skill(skill, level) => {
                match skill.as_str() {
                    "hacking" => player.hacking_skill.unwrap_or(0) >= *level,
                    "crypto" => player.crypto_skill.unwrap_or(0) >= *level,
                    "stealth" => player.stealth_skill.unwrap_or(0) >= *level,
                    _ => false,
                }
            },
            MissionPrerequisite::Custom(_) => true,
        }
    }
    
    pub fn description(&self) -> String {
        match self {
            MissionPrerequisite::Level(lvl) => format!("Reach level {}", lvl),
            MissionPrerequisite::Money(amt) => format!("Have ${}", amt),
            MissionPrerequisite::Reputation(rep) => format!("Have {} reputation", rep),
            MissionPrerequisite::CompletedMission(mission) => format!("Complete {:?}", mission),
            MissionPrerequisite::CompletedMissionCount(cat, count) => {
                format!("Complete {} {:?} missions", count, cat)
            },
            MissionPrerequisite::OwnsSoftware(name, ver) => format!("Own {} v{}", name, ver),
            MissionPrerequisite::OwnsHardware(hw, val) => format!("Have {} with value {}", hw, val),
            MissionPrerequisite::ClanMembership => "Join a clan".to_string(),
            MissionPrerequisite::ClanRank(rank) => format!("Reach clan rank {}", rank),
            MissionPrerequisite::Skill(skill, lvl) => format!("Have {} skill level {}", skill, lvl),
            MissionPrerequisite::Custom(desc) => desc.clone(),
        }
    }
}

/// Mission failure penalties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionPenalty {
    pub money_loss: i32,
    pub reputation_loss: i32,
    pub experience_loss: i32,
    pub wanted_level_increase: i32,
    pub lock_missions: Vec<MissionType>,
    pub custom_penalties: HashMap<String, String>,
}

impl MissionPenalty {
    pub fn new() -> Self {
        MissionPenalty {
            money_loss: 0,
            reputation_loss: 0,
            experience_loss: 0,
            wanted_level_increase: 0,
            lock_missions: Vec::new(),
            custom_penalties: HashMap::new(),
        }
    }
    
    pub fn with_money_loss(mut self, amount: i32) -> Self {
        self.money_loss = amount;
        self
    }
    
    pub fn with_reputation_loss(mut self, amount: i32) -> Self {
        self.reputation_loss = amount;
        self
    }
    
    pub fn with_wanted_level(mut self, increase: i32) -> Self {
        self.wanted_level_increase = increase;
        self
    }
}

/// Mission manager
#[derive(Debug, Clone)]
pub struct MissionManager {
    pub missions: HashMap<Uuid, Mission>,
    pub active_missions: HashSet<Uuid>,
    pub completed_missions: HashSet<Uuid>,
    pub failed_missions: HashSet<Uuid>,
    pub mission_chains: HashMap<String, Vec<MissionType>>,
    pub daily_missions: Vec<Mission>,
    pub special_events: Vec<Mission>,
}

impl MissionManager {
    pub fn new() -> Self {
        MissionManager {
            missions: HashMap::new(),
            active_missions: HashSet::new(),
            completed_missions: HashSet::new(),
            failed_missions: HashSet::new(),
            mission_chains: HashMap::new(),
            daily_missions: Vec::new(),
            special_events: Vec::new(),
        }
    }
    
    pub fn add_mission(&mut self, mission: Mission) -> Uuid {
        let mission_id = mission.id;
        self.missions.insert(mission_id, mission);
        mission_id
    }
    
    pub fn unlock_mission(&mut self, mission_id: Uuid, player: &PlayerState) -> Result<(), String> {
        let mission = self.missions.get_mut(&mission_id)
            .ok_or("Mission not found")?;
        
        // Check prerequisites
        for prereq in &mission.prerequisites {
            if !prereq.is_met(player) {
                return Err(format!("Prerequisite not met: {}", prereq.description()));
            }
        }
        
        if mission.status == MissionStatus::Locked {
            mission.status = MissionStatus::Available;
        }
        
        Ok(())
    }
    
    pub fn start_mission(&mut self, mission_id: Uuid, player: &PlayerState) -> Result<(), String> {
        let mission = self.missions.get_mut(&mission_id)
            .ok_or("Mission not found")?;
        
        mission.can_start(player)?;
        mission.start()?;
        
        self.active_missions.insert(mission_id);
        
        Ok(())
    }
    
    pub fn update_mission_progress(
        &mut self,
        mission_id: Uuid,
        objective_id: Uuid,
        progress: i32
    ) -> Result<(), String> {
        let mission = self.missions.get_mut(&mission_id)
            .ok_or("Mission not found")?;
        
        if mission.status != MissionStatus::Active {
            return Err("Mission is not active".to_string());
        }
        
        // Check if mission expired
        if mission.is_expired() {
            mission.fail();
            self.active_missions.remove(&mission_id);
            self.failed_missions.insert(mission_id);
            return Err("Mission expired".to_string());
        }
        
        // Update objective progress
        if let Some(objective) = mission.objectives.iter_mut().find(|obj| obj.id == objective_id) {
            objective.update_progress(progress);
        } else {
            return Err("Objective not found".to_string());
        }
        
        // Check if mission is complete
        if mission.check_completion() {
            let _ = self.complete_mission(mission_id);
        }
        
        Ok(())
    }
    
    pub fn complete_mission(&mut self, mission_id: Uuid) -> Result<MissionReward, String> {
        let mission = self.missions.get_mut(&mission_id)
            .ok_or("Mission not found")?;
        
        let reward = mission.complete()?;
        
        self.active_missions.remove(&mission_id);
        self.completed_missions.insert(mission_id);
        
        // Unlock follow-up missions
        for unlock_type in &reward.unlock_missions {
            // Find and unlock missions of this type
            for (id, m) in &mut self.missions {
                if m.mission_type == *unlock_type && m.status == MissionStatus::Locked {
                    m.status = MissionStatus::Available;
                }
            }
        }
        
        Ok(reward)
    }
    
    pub fn fail_mission(&mut self, mission_id: Uuid) -> Option<MissionPenalty> {
        let mission = self.missions.get_mut(&mission_id)?;
        
        let penalty = mission.fail();
        
        self.active_missions.remove(&mission_id);
        self.failed_missions.insert(mission_id);
        
        penalty
    }
    
    pub fn abandon_mission(&mut self, mission_id: Uuid) -> Result<(), String> {
        let mission = self.missions.get_mut(&mission_id)
            .ok_or("Mission not found")?;
        
        if mission.status != MissionStatus::Active {
            return Err("Can only abandon active missions".to_string());
        }
        
        mission.abandon();
        self.active_missions.remove(&mission_id);
        
        Ok(())
    }
    
    pub fn get_available_missions(&self, player: &PlayerState) -> Vec<&Mission> {
        self.missions.values()
            .filter(|m| m.status == MissionStatus::Available && m.can_start(player).is_ok())
            .collect()
    }
    
    pub fn get_active_missions(&self) -> Vec<&Mission> {
        self.active_missions.iter()
            .filter_map(|id| self.missions.get(id))
            .collect()
    }
    
    pub fn update_expired_missions(&mut self) {
        let mut to_fail = Vec::new();
        
        for &mission_id in &self.active_missions {
            if let Some(mission) = self.missions.get(&mission_id) {
                if mission.is_expired() {
                    to_fail.push(mission_id);
                }
            }
        }
        
        for mission_id in to_fail {
            self.fail_mission(mission_id);
        }
    }
    
    pub fn generate_daily_missions(&mut self, player_level: i32) {
        self.daily_missions.clear();
        
        // Generate 3 daily missions based on player level
        let mission_types = vec![
            MissionType::HackServer,
            MissionType::StealData,
            MissionType::DefendServer,
        ];
        
        for (i, mission_type) in mission_types.into_iter().enumerate() {
            let mut mission = Mission::new(
                mission_type,
                format!("Daily Mission #{}", i + 1),
                "Complete this daily mission for bonus rewards".to_string()
            );
            
            // Scale to player level
            mission.difficulty = (player_level / 10 + 1).min(10);
            mission.level_requirement = player_level.saturating_sub(5);
            
            // Set as repeatable with 24 hour cooldown
            mission = mission.set_repeatable(true, Some(Duration::from_secs(86400)));
            
            // Add rewards
            let reward = MissionReward::new()
                .with_money(100 * mission.difficulty)
                .with_experience(50 * mission.difficulty)
                .with_reputation(10 * mission.difficulty);
            mission = mission.with_reward(reward);
            
            mission.status = MissionStatus::Available;
            self.daily_missions.push(mission);
        }
    }
}

/// Calculate mission difficulty with scaling
pub fn calculate_mission_difficulty(
    mission: &Mission,
    player: &PlayerState,
    config: &MissionConfig
) -> i32 {
    let base_difficulty = mission.difficulty;
    
    // Scale based on player level difference
    let level_diff = (mission.level_requirement - player.level).max(0);
    let level_penalty = level_diff * config.level_scaling_factor as i32;
    
    // Scale based on mission type
    let type_multiplier = match mission.mission_type.category() {
        MissionCategory::Tutorial => 0.5,
        MissionCategory::Special => 2.0,
        MissionCategory::Story => 1.5,
        _ => 1.0,
    };
    
    // Apply target difficulty if present
    let target_modifier = if let Some(target) = &mission.target_info {
        (target.security_level.unwrap_or(50) as f32 / 50.0)
    } else {
        1.0
    };
    
    ((base_difficulty + level_penalty) as f32 * type_multiplier * target_modifier) as i32
}

/// Validate mission prerequisites
pub fn validate_prerequisites(mission: &Mission, player: &PlayerState) -> bool {
    mission.prerequisites.iter().all(|prereq| prereq.is_met(player))
}

/// Calculate mission rewards with bonuses
pub fn calculate_mission_rewards(
    mission: &Mission,
    player: &PlayerState,
    completion_time: Duration,
    config: &MissionConfig
) -> MissionReward {
    let mut reward = mission.rewards.clone();
    
    // Time bonus for quick completion
    if let Some(time_limit) = mission.time_limit {
        let time_ratio = completion_time.as_secs() as f32 / time_limit.as_secs() as f32;
        if time_ratio < 0.5 {
            // Completed in less than half the time
            reward.money = (reward.money as f32 * 1.5) as i32;
            reward.experience = (reward.experience as f32 * 1.5) as i32;
        } else if time_ratio < 0.75 {
            // Completed in less than 75% of time
            reward.money = (reward.money as f32 * 1.2) as i32;
            reward.experience = (reward.experience as f32 * 1.2) as i32;
        }
    }
    
    // Difficulty bonus
    let difficulty_multiplier = 1.0 + (mission.difficulty as f32 / 10.0);
    reward.experience = (reward.experience as f32 * difficulty_multiplier) as i32;
    
    // First time completion bonus
    if mission.attempts == 1 {
        reward.reputation += 5;
    }
    
    // Perfect completion bonus (all objectives including optional)
    let all_complete = mission.objectives.iter().all(|obj| obj.is_completed);
    if all_complete && mission.objectives.iter().any(|obj| obj.is_optional) {
        reward.money = (reward.money as f32 * 1.25) as i32;
        reward.reputation += 10;
    }
    
    reward
}

/// Generate mission chain
pub fn generate_mission_chain(
    chain_type: &str,
    start_difficulty: i32
) -> Vec<Mission> {
    let mut missions = Vec::new();
    
    let chain_missions = match chain_type {
        "infiltration" => vec![
            ("Reconnaissance", MissionType::GatherIntelligence),
            ("Initial Access", MissionType::HackServer),
            ("Deep Infiltration", MissionType::InfiltrateCompany),
            ("Data Extraction", MissionType::StealCorporateSecrets),
            ("Clean Exit", MissionType::DeleteLogs),
        ],
        "heist" => vec![
            ("Case the Joint", MissionType::GatherIntelligence),
            ("Disable Security", MissionType::BypassFirewall),
            ("Access Vault", MissionType::CrackPassword),
            ("Transfer Funds", MissionType::BankHeist),
            ("Cover Tracks", MissionType::DeleteLogs),
        ],
        _ => vec![],
    };
    
    for (i, (name, mission_type)) in chain_missions.into_iter().enumerate() {
        let mut mission = Mission::new(
            mission_type,
            format!("{} - Part {}", name, i + 1),
            format!("Part {} of the {} chain", i + 1, chain_type)
        );
        
        mission.difficulty = start_difficulty + i as i32;
        
        // Chain missions together with prerequisites
        if i > 0 {
            mission.prerequisites.push(
                MissionPrerequisite::Custom(format!("Complete Part {}", i))
            );
        }
        
        // Unlock next mission in chain
        if i < 4 {
            mission.rewards.unlock_missions.push(
                chain_missions.get(i + 1).unwrap().1.clone()
            );
        }
        
        missions.push(mission);
    }
    
    missions
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mission_creation() {
        let mission = Mission::new(
            MissionType::HackServer,
            "Test Hack".to_string(),
            "Hack the test server".to_string()
        );
        
        assert_eq!(mission.mission_type, MissionType::HackServer);
        assert_eq!(mission.status, MissionStatus::Locked);
        assert_eq!(mission.difficulty, 3);
    }
    
    #[test]
    fn test_objective_progress() {
        let mut objective = MissionObjective::new(
            ObjectiveType::HackTarget("192.168.1.1".to_string()),
            "Hack the target server".to_string(),
            100
        );
        
        objective.update_progress(50);
        assert_eq!(objective.progress, 50);
        assert!(!objective.is_completed);
        
        objective.update_progress(50);
        assert_eq!(objective.progress, 100);
        assert!(objective.is_completed);
    }
    
    #[test]
    fn test_mission_prerequisites() {
        let player = PlayerState {
            level: 10,
            money: 5000,
            reputation: 100,
            ..Default::default()
        };
        
        let prereq_level = MissionPrerequisite::Level(5);
        assert!(prereq_level.is_met(&player));
        
        let prereq_money = MissionPrerequisite::Money(10000);
        assert!(!prereq_money.is_met(&player));
    }
    
    #[test]
    fn test_mission_completion() {
        let mut mission = Mission::new(
            MissionType::TutorialBasicHack,
            "Tutorial".to_string(),
            "Complete tutorial".to_string()
        );
        
        let obj = MissionObjective::new(
            ObjectiveType::HackTarget("localhost".to_string()),
            "Hack localhost".to_string(),
            1
        );
        mission = mission.with_objective(obj);
        
        mission.status = MissionStatus::Available;
        assert!(mission.start().is_ok());
        assert_eq!(mission.status, MissionStatus::Active);
        
        // Complete objective
        mission.objectives[0].is_completed = true;
        assert!(mission.check_completion());
        
        let result = mission.complete();
        assert!(result.is_ok());
        assert_eq!(mission.status, MissionStatus::Completed);
    }
    
    #[test]
    fn test_mission_manager() {
        let mut manager = MissionManager::new();
        
        let mission = Mission::new(
            MissionType::HackServer,
            "Test Mission".to_string(),
            "Test description".to_string()
        );
        
        let mission_id = manager.add_mission(mission);
        assert!(manager.missions.contains_key(&mission_id));
        
        let player = PlayerState::default();
        
        // Unlock mission
        manager.missions.get_mut(&mission_id).unwrap().status = MissionStatus::Available;
        
        // Start mission
        let result = manager.start_mission(mission_id, &player);
        assert!(result.is_ok());
        assert!(manager.active_missions.contains(&mission_id));
    }
    
    #[test]
    fn test_mission_rewards() {
        let reward = MissionReward::new()
            .with_money(1000)
            .with_experience(500)
            .with_reputation(10)
            .with_bitcoin(0.001);
        
        assert_eq!(reward.money, 1000);
        assert_eq!(reward.experience, 500);
        assert_eq!(reward.reputation, 10);
        assert_eq!(reward.bitcoin, 0.001);
        
        let total_value = reward.calculate_total_value();
        assert!(total_value > 0);
    }
}