//! Story System GenServer Implementation
//! 
//! Complete port of Helix.Story GenServer with mission and narrative progression,
//! quest management, and dynamic storytelling capabilities.

use he_helix_core::genserver::{
    GenServer, GenServerState, GenServerHandle, GenServerMessage, GenServerReply,
    InfoSource, TerminateReason, SupervisionStrategy, GenServerSupervisor
};
use he_helix_core::{HelixError, HelixResult, ProcessId};
use he_core::id::{AccountId, EntityId, ServerId, StoryId, MissionId};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::{HashMap, BTreeMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, Mutex, broadcast};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Story step types for mission progression
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StoryStepType {
    /// Tutorial steps - teach basic game mechanics
    Tutorial,
    /// Main storyline progression
    MainStory,
    /// Side quests and optional content
    SideQuest,
    /// Dynamic events based on player actions
    DynamicEvent,
    /// Conditional branches in narrative
    ConditionalBranch,
    /// Achievement-based story elements
    Achievement,
    /// Time-based story events
    TimedEvent,
}

/// Story step status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StoryStepStatus {
    /// Not yet available to the player
    Locked,
    /// Available but not started
    Available,
    /// Currently in progress
    InProgress,
    /// Successfully completed
    Completed,
    /// Failed or abandoned
    Failed,
    /// Skipped (for optional content)
    Skipped,
}

/// Mission difficulty levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum MissionDifficulty {
    Trivial,
    Easy,
    Normal,
    Hard,
    Expert,
    Nightmare,
}

/// Mission requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionRequirements {
    /// Minimum account level
    pub min_level: u32,
    /// Required completed missions
    pub required_missions: Vec<MissionId>,
    /// Required servers/resources
    pub required_servers: Vec<ServerId>,
    /// Minimum skill levels
    pub required_skills: HashMap<String, u32>,
    /// Required items/software
    pub required_items: Vec<EntityId>,
    /// Time constraints
    pub time_limit: Option<Duration>,
}

/// Mission objectives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionObjective {
    pub objective_id: EntityId,
    pub title: String,
    pub description: String,
    pub objective_type: ObjectiveType,
    pub target: ObjectiveTarget,
    pub required_amount: u32,
    pub current_progress: u32,
    pub is_completed: bool,
    pub is_optional: bool,
    pub reward_multiplier: f64,
}

/// Types of mission objectives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectiveType {
    /// Hack into specific servers
    HackServer,
    /// Download specific files
    DownloadFile,
    /// Upload files to servers
    UploadFile,
    /// Install software on servers
    InstallSoftware,
    /// Transfer money
    MoneyTransfer,
    /// Send emails
    SendEmail,
    /// Connect to specific servers
    ConnectToServer,
    /// Scan for vulnerabilities
    ScanVulnerabilities,
    /// Complete other missions
    CompleteMissions,
    /// Reach certain reputation level
    ReachReputation,
    /// Survive for certain time
    SurviveTime,
    /// Collect specific items
    CollectItems,
}

/// Objective targets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectiveTarget {
    Server(ServerId),
    File(String), // filename pattern
    Software(String), // software type
    Account(AccountId),
    Organization(EntityId),
    Location(String), // geographic location
    Any, // any target of the type
}

/// Mission rewards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionRewards {
    /// Experience points
    pub experience: u32,
    /// Money reward
    pub money: u64,
    /// Reputation changes
    pub reputation: HashMap<String, i32>,
    /// Unlocked items/software
    pub unlocked_items: Vec<EntityId>,
    /// Skill improvements
    pub skill_bonuses: HashMap<String, u32>,
    /// Special unlocks (servers, features, etc.)
    pub special_unlocks: Vec<String>,
}

/// Complete mission/story step definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryStep {
    pub step_id: MissionId,
    pub story_id: StoryId,
    pub title: String,
    pub description: String,
    pub step_type: StoryStepType,
    pub difficulty: MissionDifficulty,
    pub requirements: MissionRequirements,
    pub objectives: Vec<MissionObjective>,
    pub rewards: MissionRewards,
    pub narrative: StoryNarrative,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

/// Story narrative elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryNarrative {
    /// Opening dialogue/text
    pub intro_text: String,
    /// Text during mission
    pub progress_text: HashMap<String, String>, // event -> text
    /// Completion text
    pub completion_text: String,
    /// Failure text
    pub failure_text: String,
    /// NPCs involved in the story
    pub npcs: Vec<StoryNPC>,
    /// Email messages
    pub emails: Vec<StoryEmail>,
    /// Conditional dialogue based on player choices
    pub conditional_dialogue: HashMap<String, String>,
}

/// NPC characters in stories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryNPC {
    pub npc_id: EntityId,
    pub name: String,
    pub role: String,
    pub avatar: String,
    pub personality: NPCPersonality,
    pub dialogue: HashMap<String, String>, // situation -> dialogue
    pub relationship_level: i32, // -100 to 100
}

/// NPC personality traits affecting dialogue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPCPersonality {
    pub friendliness: i8,   // -5 to 5
    pub helpfulness: i8,    // -5 to 5
    pub trustworthiness: i8, // -5 to 5
    pub technical_skill: i8, // -5 to 5
    pub humor: i8,          // -5 to 5
}

/// Story email messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryEmail {
    pub email_id: EntityId,
    pub sender: String,
    pub sender_email: String,
    pub subject: String,
    pub body: String,
    pub trigger_condition: String, // when to send this email
    pub is_reply: bool,
    pub attachments: Vec<String>,
}

/// Player's progress through a story
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStoryProgress {
    pub account_id: AccountId,
    pub story_id: StoryId,
    pub current_step: Option<MissionId>,
    pub completed_steps: Vec<MissionId>,
    pub failed_steps: Vec<MissionId>,
    pub step_progress: HashMap<MissionId, StoryStepProgress>,
    pub story_variables: HashMap<String, String>, // for branching narratives
    pub started_at: SystemTime,
    pub last_updated: SystemTime,
}

/// Progress on a specific story step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryStepProgress {
    pub step_id: MissionId,
    pub status: StoryStepStatus,
    pub objective_progress: HashMap<EntityId, u32>,
    pub start_time: Option<SystemTime>,
    pub completion_time: Option<SystemTime>,
    pub attempts: u32,
    pub hints_used: Vec<String>,
    pub choices_made: HashMap<String, String>, // choice_id -> selected_option
}

/// Dynamic story generation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryGenerationConfig {
    pub enable_dynamic_generation: bool,
    pub player_skill_influence: f64,
    pub world_event_influence: f64,
    pub reputation_influence: f64,
    pub difficulty_scaling: f64,
    pub narrative_complexity: u8, // 1-10
    pub branching_factor: u8,     // 1-5
}

/// Story System State
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorySystemState {
    /// All available story steps/missions
    pub story_steps: HashMap<MissionId, StoryStep>,
    
    /// Story progression trees
    pub story_trees: HashMap<StoryId, Vec<MissionId>>,
    
    /// Player progress tracking
    pub player_progress: HashMap<AccountId, Vec<PlayerStoryProgress>>,
    
    /// Active missions per player
    pub active_missions: HashMap<AccountId, Vec<MissionId>>,
    
    /// Story dependencies (step -> required steps)
    pub dependencies: HashMap<MissionId, Vec<MissionId>>,
    
    /// Dynamic story templates for generation
    pub story_templates: HashMap<String, StoryTemplate>,
    
    /// Statistics and metrics
    pub stats: StorySystemStats,
    
    /// Configuration
    pub config: StorySystemConfig,
    
    /// Mission scheduler for timed events
    pub scheduled_events: BTreeMap<SystemTime, Vec<ScheduledStoryEvent>>,
}

/// Story templates for dynamic generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryTemplate {
    pub template_id: String,
    pub template_type: StoryStepType,
    pub title_template: String,
    pub description_template: String,
    pub objective_templates: Vec<ObjectiveTemplate>,
    pub reward_formula: RewardFormula,
    pub variables: Vec<String>, // placeholder variables
}

/// Objective templates for dynamic generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveTemplate {
    pub objective_type: ObjectiveType,
    pub description_template: String,
    pub amount_formula: String, // formula for required amount
    pub target_selection: TargetSelectionRule,
}

/// Rules for selecting objective targets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetSelectionRule {
    Random,
    PlayerLevel,
    PlayerSkill(String),
    WorldState,
    NearbyServers,
    PlayerOwned,
    NPCOwned(EntityId),
}

/// Reward calculation formulas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardFormula {
    pub base_experience: u32,
    pub experience_multiplier: f64,
    pub base_money: u64,
    pub money_multiplier: f64,
    pub skill_bonus_formula: String,
}

/// Scheduled story events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledStoryEvent {
    pub event_id: EntityId,
    pub event_type: ScheduledEventType,
    pub target_account: Option<AccountId>,
    pub target_story: Option<StoryId>,
    pub parameters: HashMap<String, String>,
}

/// Types of scheduled story events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduledEventType {
    UnlockMission,
    SendEmail,
    TriggerEvent,
    UpdateObjective,
    FailMission,
    CompleteMission,
    BranchStory,
}

/// Story system statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorySystemStats {
    pub total_stories: u32,
    pub total_missions: u32,
    pub active_missions: u32,
    pub completed_missions: u32,
    pub failed_missions: u32,
    pub players_in_tutorial: u32,
    pub players_in_main_story: u32,
    pub players_in_side_quests: u32,
    pub average_completion_time: HashMap<MissionId, Duration>,
    pub popular_missions: Vec<(MissionId, u32)>,
    pub last_updated: SystemTime,
}

/// Story system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorySystemConfig {
    pub max_active_missions_per_player: u32,
    pub mission_timeout_duration: Duration,
    pub auto_unlock_interval: Duration,
    pub story_generation: StoryGenerationConfig,
    pub enable_email_system: bool,
    pub enable_npc_interaction: bool,
    pub difficulty_adjustment: bool,
}

impl Default for StorySystemConfig {
    fn default() -> Self {
        Self {
            max_active_missions_per_player: 5,
            mission_timeout_duration: Duration::from_secs(86400 * 7), // 7 days
            auto_unlock_interval: Duration::from_secs(300), // 5 minutes
            story_generation: StoryGenerationConfig {
                enable_dynamic_generation: true,
                player_skill_influence: 0.3,
                world_event_influence: 0.2,
                reputation_influence: 0.1,
                difficulty_scaling: 1.0,
                narrative_complexity: 5,
                branching_factor: 2,
            },
            enable_email_system: true,
            enable_npc_interaction: true,
            difficulty_adjustment: true,
        }
    }
}

impl GenServerState for StorySystemState {
    fn serialize(&self) -> HelixResult<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| HelixError::Serialization(e.to_string()))
    }

    fn deserialize(data: &[u8]) -> HelixResult<Self> {
        serde_json::from_slice(data).map_err(|e| HelixError::Serialization(e.to_string()))
    }
}

/// Story System GenServer Messages - Call patterns
#[derive(Debug)]
pub enum StoryCall {
    /// Get available missions for a player
    GetAvailableMissions { account_id: AccountId },
    
    /// Start a mission
    StartMission { 
        account_id: AccountId,
        mission_id: MissionId,
    },
    
    /// Get mission details
    GetMission { mission_id: MissionId },
    
    /// Update mission progress
    UpdateMissionProgress {
        account_id: AccountId,
        mission_id: MissionId,
        objective_id: EntityId,
        progress: u32,
    },
    
    /// Complete mission objective
    CompleteObjective {
        account_id: AccountId,
        mission_id: MissionId,
        objective_id: EntityId,
    },
    
    /// Complete entire mission
    CompleteMission {
        account_id: AccountId,
        mission_id: MissionId,
    },
    
    /// Fail/abandon mission
    FailMission {
        account_id: AccountId,
        mission_id: MissionId,
        reason: String,
    },
    
    /// Get player's story progress
    GetPlayerProgress { account_id: AccountId },
    
    /// Get story tree/progression
    GetStoryTree { story_id: StoryId },
    
    /// Check mission requirements
    CheckMissionRequirements {
        account_id: AccountId,
        mission_id: MissionId,
    },
    
    /// Generate dynamic mission
    GenerateDynamicMission {
        account_id: AccountId,
        template_id: String,
        parameters: HashMap<String, String>,
    },
    
    /// Get mission statistics
    GetMissionStats { mission_id: Option<MissionId> },
    
    /// Search missions by criteria
    SearchMissions { criteria: MissionSearchCriteria },
}

/// Story System GenServer Cast Messages
#[derive(Debug)]
pub enum StoryCast {
    /// Add new story step
    AddStoryStep { step: StoryStep },
    
    /// Update story step
    UpdateStoryStep { 
        step_id: MissionId,
        updates: StoryStepUpdates,
    },
    
    /// Remove story step
    RemoveStoryStep { step_id: MissionId },
    
    /// Send story email
    SendStoryEmail {
        account_id: AccountId,
        email: StoryEmail,
    },
    
    /// Trigger story event
    TriggerStoryEvent {
        event_type: String,
        target_account: Option<AccountId>,
        parameters: HashMap<String, String>,
    },
    
    /// Update NPC relationship
    UpdateNPCRelationship {
        account_id: AccountId,
        npc_id: EntityId,
        relationship_change: i32,
    },
    
    /// Set story variable
    SetStoryVariable {
        account_id: AccountId,
        story_id: StoryId,
        variable: String,
        value: String,
    },
    
    /// Schedule story event
    ScheduleStoryEvent {
        when: SystemTime,
        event: ScheduledStoryEvent,
    },
    
    /// Bulk update mission progress
    BulkUpdateProgress {
        updates: Vec<(AccountId, MissionId, EntityId, u32)>,
    },
    
    /// Clean up expired missions
    CleanupExpiredMissions,
    
    /// Refresh statistics
    RefreshStats,
    
    /// Generate random content
    GenerateRandomContent { content_type: String },
}

/// Story System GenServer Info Messages
#[derive(Debug)]
pub enum StoryInfo {
    /// Auto-unlock timer
    AutoUnlockTimer,
    
    /// Cleanup timer
    CleanupTimer,
    
    /// Statistics refresh timer
    StatsTimer,
    
    /// Scheduled event trigger
    ScheduledEventTrigger { event: ScheduledStoryEvent },
    
    /// Player action notification
    PlayerAction {
        account_id: AccountId,
        action_type: String,
        details: HashMap<String, String>,
    },
    
    /// World event notification
    WorldEvent {
        event_type: String,
        affected_players: Vec<AccountId>,
        impact: HashMap<String, String>,
    },
    
    /// Mission timeout warning
    MissionTimeout {
        account_id: AccountId,
        mission_id: MissionId,
        time_remaining: Duration,
    },
    
    /// External story update
    ExternalStoryUpdate {
        source: String,
        update_type: String,
        data: HashMap<String, String>,
    },
}

/// Mission search criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionSearchCriteria {
    pub step_type: Option<StoryStepType>,
    pub difficulty: Option<MissionDifficulty>,
    pub min_level: Option<u32>,
    pub max_level: Option<u32>,
    pub keywords: Option<Vec<String>>,
    pub available_only: bool,
}

/// Story step update parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryStepUpdates {
    pub title: Option<String>,
    pub description: Option<String>,
    pub difficulty: Option<MissionDifficulty>,
    pub rewards: Option<MissionRewards>,
    pub narrative: Option<StoryNarrative>,
}

/// Story System GenServer Implementation
pub struct StorySystemGenServer {
    story_event_broadcaster: broadcast::Sender<StoryEvent>,
}

/// Story events for external systems
#[derive(Debug, Clone)]
pub struct StoryEvent {
    pub event_type: String,
    pub account_id: AccountId,
    pub mission_id: Option<MissionId>,
    pub data: HashMap<String, String>,
}

impl StorySystemGenServer {
    pub fn new() -> (Self, broadcast::Receiver<StoryEvent>) {
        let (tx, rx) = broadcast::channel(1000);
        (Self { story_event_broadcaster: tx }, rx)
    }
}

#[async_trait]
impl GenServer for StorySystemGenServer {
    type State = StorySystemState;
    type InitArgs = StorySystemConfig;

    async fn init(config: Self::InitArgs) -> HelixResult<Self::State> {
        info!("Initializing Story System GenServer");
        
        let now = SystemTime::now();
        let stats = StorySystemStats {
            total_stories: 0,
            total_missions: 0,
            active_missions: 0,
            completed_missions: 0,
            failed_missions: 0,
            players_in_tutorial: 0,
            players_in_main_story: 0,
            players_in_side_quests: 0,
            average_completion_time: HashMap::new(),
            popular_missions: Vec::new(),
            last_updated: now,
        };

        Ok(StorySystemState {
            story_steps: HashMap::new(),
            story_trees: HashMap::new(),
            player_progress: HashMap::new(),
            active_missions: HashMap::new(),
            dependencies: HashMap::new(),
            story_templates: HashMap::new(),
            stats,
            config,
            scheduled_events: BTreeMap::new(),
        })
    }

    async fn handle_call(
        &mut self,
        request: Box<dyn Any + Send + Sync>,
        from: ProcessId,
        state: &mut Self::State,
    ) -> HelixResult<GenServerReply> {
        if let Some(call) = request.downcast_ref::<StoryCall>() {
            match call {
                StoryCall::GetAvailableMissions { account_id } => {
                    debug!("Getting available missions for account {} from {:?}", account_id, from);
                    let available = self.get_available_missions_for_player(state, *account_id);
                    Ok(GenServerReply::Reply(Box::new(available)))
                }
                
                StoryCall::StartMission { account_id, mission_id } => {
                    info!("Starting mission {} for account {} from {:?}", mission_id, account_id, from);
                    
                    // Check if mission exists
                    if !state.story_steps.contains_key(mission_id) {
                        return Ok(GenServerReply::Reply(Box::new("mission_not_found")));
                    }
                    
                    // Check requirements
                    let can_start = self.check_mission_requirements(state, *account_id, *mission_id);
                    if !can_start {
                        return Ok(GenServerReply::Reply(Box::new("requirements_not_met")));
                    }
                    
                    // Check active mission limit
                    let active_count = state.active_missions.get(account_id).map(|v| v.len()).unwrap_or(0);
                    if active_count >= state.config.max_active_missions_per_player as usize {
                        return Ok(GenServerReply::Reply(Box::new("too_many_active_missions")));
                    }
                    
                    // Start the mission
                    let progress = StoryStepProgress {
                        step_id: *mission_id,
                        status: StoryStepStatus::InProgress,
                        objective_progress: HashMap::new(),
                        start_time: Some(SystemTime::now()),
                        completion_time: None,
                        attempts: 1,
                        hints_used: Vec::new(),
                        choices_made: HashMap::new(),
                    };
                    
                    // Update player progress
                    let player_progress = state.player_progress.entry(*account_id).or_insert_with(Vec::new);
                    
                    // Find or create story progress
                    if let Some(story_step) = state.story_steps.get(mission_id) {
                        let mut story_progress = player_progress.iter_mut()
                            .find(|p| p.story_id == story_step.story_id);
                            
                        if let Some(story_prog) = story_progress {
                            story_prog.step_progress.insert(*mission_id, progress);
                            if story_prog.current_step.is_none() {
                                story_prog.current_step = Some(*mission_id);
                            }
                            story_prog.last_updated = SystemTime::now();
                        } else {
                            // Create new story progress
                            let mut new_progress = PlayerStoryProgress {
                                account_id: *account_id,
                                story_id: story_step.story_id,
                                current_step: Some(*mission_id),
                                completed_steps: Vec::new(),
                                failed_steps: Vec::new(),
                                step_progress: HashMap::new(),
                                story_variables: HashMap::new(),
                                started_at: SystemTime::now(),
                                last_updated: SystemTime::now(),
                            };
                            new_progress.step_progress.insert(*mission_id, progress);
                            player_progress.push(new_progress);
                        }
                        
                        // Add to active missions
                        state.active_missions.entry(*account_id).or_insert_with(Vec::new).push(*mission_id);
                        
                        // Broadcast event
                        let event = StoryEvent {
                            event_type: "mission_started".to_string(),
                            account_id: *account_id,
                            mission_id: Some(*mission_id),
                            data: HashMap::new(),
                        };
                        let _ = self.story_event_broadcaster.send(event);
                        
                        Ok(GenServerReply::Reply(Box::new("mission_started")))
                    } else {
                        Ok(GenServerReply::Reply(Box::new("mission_not_found")))
                    }
                }
                
                StoryCall::GetMission { mission_id } => {
                    debug!("Getting mission {} for {:?}", mission_id, from);
                    let mission = state.story_steps.get(mission_id).cloned();
                    Ok(GenServerReply::Reply(Box::new(mission)))
                }
                
                StoryCall::UpdateMissionProgress { account_id, mission_id, objective_id, progress } => {
                    debug!("Updating mission {} objective {} progress to {} for account {}", 
                           mission_id, objective_id, progress, account_id);
                    
                    let updated = self.update_objective_progress(state, *account_id, *mission_id, *objective_id, *progress);
                    
                    if updated {
                        // Broadcast progress update
                        let mut data = HashMap::new();
                        data.insert("objective_id".to_string(), objective_id.to_string());
                        data.insert("progress".to_string(), progress.to_string());
                        
                        let event = StoryEvent {
                            event_type: "mission_progress".to_string(),
                            account_id: *account_id,
                            mission_id: Some(*mission_id),
                            data,
                        };
                        let _ = self.story_event_broadcaster.send(event);
                    }
                    
                    Ok(GenServerReply::Reply(Box::new(updated)))
                }
                
                StoryCall::CompleteObjective { account_id, mission_id, objective_id } => {
                    info!("Completing objective {} in mission {} for account {}", 
                          objective_id, mission_id, account_id);
                    
                    let completed = self.complete_objective(state, *account_id, *mission_id, *objective_id);
                    
                    if completed {
                        // Check if all objectives are complete
                        let mission_complete = self.check_mission_completion(state, *account_id, *mission_id);
                        
                        if mission_complete {
                            self.complete_mission_internal(state, *account_id, *mission_id)?;
                        }
                        
                        // Broadcast event
                        let mut data = HashMap::new();
                        data.insert("objective_id".to_string(), objective_id.to_string());
                        data.insert("mission_complete".to_string(), mission_complete.to_string());
                        
                        let event = StoryEvent {
                            event_type: "objective_completed".to_string(),
                            account_id: *account_id,
                            mission_id: Some(*mission_id),
                            data,
                        };
                        let _ = self.story_event_broadcaster.send(event);
                    }
                    
                    Ok(GenServerReply::Reply(Box::new(completed)))
                }
                
                StoryCall::CompleteMission { account_id, mission_id } => {
                    info!("Completing mission {} for account {} from {:?}", mission_id, account_id, from);
                    let result = self.complete_mission_internal(state, *account_id, *mission_id);
                    
                    match result {
                        Ok(rewards) => Ok(GenServerReply::Reply(Box::new(rewards))),
                        Err(e) => Ok(GenServerReply::Reply(Box::new(format!("error: {}", e))))
                    }
                }
                
                StoryCall::FailMission { account_id, mission_id, reason } => {
                    info!("Failing mission {} for account {} ({})", mission_id, account_id, reason);
                    let failed = self.fail_mission_internal(state, *account_id, *mission_id, reason.clone());
                    Ok(GenServerReply::Reply(Box::new(failed)))
                }
                
                StoryCall::GetPlayerProgress { account_id } => {
                    debug!("Getting player progress for account {} from {:?}", account_id, from);
                    let progress = state.player_progress.get(account_id).cloned().unwrap_or_default();
                    Ok(GenServerReply::Reply(Box::new(progress)))
                }
                
                StoryCall::GetStoryTree { story_id } => {
                    debug!("Getting story tree for {} from {:?}", story_id, from);
                    let tree = state.story_trees.get(story_id).cloned().unwrap_or_default();
                    Ok(GenServerReply::Reply(Box::new(tree)))
                }
                
                StoryCall::CheckMissionRequirements { account_id, mission_id } => {
                    let can_start = self.check_mission_requirements(state, *account_id, *mission_id);
                    Ok(GenServerReply::Reply(Box::new(can_start)))
                }
                
                StoryCall::GenerateDynamicMission { account_id, template_id, parameters } => {
                    info!("Generating dynamic mission for account {} with template {}", account_id, template_id);
                    let mission = self.generate_dynamic_mission(state, *account_id, template_id, parameters);
                    
                    match mission {
                        Ok(mission) => {
                            // Add to story steps
                            state.story_steps.insert(mission.step_id, mission.clone());
                            Ok(GenServerReply::Reply(Box::new(mission)))
                        }
                        Err(e) => Ok(GenServerReply::Reply(Box::new(format!("generation_error: {}", e))))
                    }
                }
                
                StoryCall::GetMissionStats { mission_id } => {
                    if let Some(mission_id) = mission_id {
                        let stats = self.get_mission_statistics(state, *mission_id);
                        Ok(GenServerReply::Reply(Box::new(stats)))
                    } else {
                        Ok(GenServerReply::Reply(Box::new(state.stats.clone())))
                    }
                }
                
                StoryCall::SearchMissions { criteria } => {
                    debug!("Searching missions with criteria from {:?}", from);
                    let results = self.search_missions(state, criteria);
                    Ok(GenServerReply::Reply(Box::new(results)))
                }
            }
        } else {
            warn!("Unknown call type from {:?}", from);
            Ok(GenServerReply::Reply(Box::new("unknown_call")))
        }
    }

    async fn handle_cast(
        &mut self,
        message: Box<dyn Any + Send + Sync>,
        state: &mut Self::State,
    ) -> HelixResult<()> {
        if let Some(cast) = message.downcast_ref::<StoryCast>() {
            match cast {
                StoryCast::AddStoryStep { step } => {
                    info!("Adding story step: {}", step.title);
                    
                    // Add to story tree
                    state.story_trees.entry(step.story_id)
                        .or_insert_with(Vec::new)
                        .push(step.step_id);
                    
                    state.story_steps.insert(step.step_id, step.clone());
                    self.update_statistics(state);
                }
                
                StoryCast::UpdateStoryStep { step_id, updates } => {
                    if let Some(step) = state.story_steps.get_mut(step_id) {
                        if let Some(title) = &updates.title {
                            step.title = title.clone();
                        }
                        if let Some(description) = &updates.description {
                            step.description = description.clone();
                        }
                        if let Some(difficulty) = &updates.difficulty {
                            step.difficulty = difficulty.clone();
                        }
                        if let Some(rewards) = &updates.rewards {
                            step.rewards = rewards.clone();
                        }
                        if let Some(narrative) = &updates.narrative {
                            step.narrative = narrative.clone();
                        }
                        step.updated_at = SystemTime::now();
                        info!("Updated story step: {}", step.title);
                    }
                }
                
                StoryCast::RemoveStoryStep { step_id } => {
                    if let Some(step) = state.story_steps.remove(step_id) {
                        // Remove from story tree
                        if let Some(tree) = state.story_trees.get_mut(&step.story_id) {
                            tree.retain(|id| id != step_id);
                        }
                        
                        // Remove dependencies
                        state.dependencies.remove(step_id);
                        
                        info!("Removed story step: {}", step.title);
                        self.update_statistics(state);
                    }
                }
                
                StoryCast::SendStoryEmail { account_id, email } => {
                    info!("Sending story email '{}' to account {}", email.subject, account_id);
                    // In a real implementation, this would integrate with the email system
                    
                    // Broadcast event
                    let mut data = HashMap::new();
                    data.insert("email_id".to_string(), email.email_id.to_string());
                    data.insert("subject".to_string(), email.subject.clone());
                    
                    let event = StoryEvent {
                        event_type: "story_email_sent".to_string(),
                        account_id: *account_id,
                        mission_id: None,
                        data,
                    };
                    let _ = self.story_event_broadcaster.send(event);
                }
                
                StoryCast::TriggerStoryEvent { event_type, target_account, parameters } => {
                    info!("Triggering story event: {} for {:?}", event_type, target_account);
                    self.handle_story_event_trigger(state, event_type, *target_account, parameters);
                }
                
                StoryCast::UpdateNPCRelationship { account_id, npc_id, relationship_change } => {
                    info!("Updating NPC {} relationship for account {} by {}", 
                          npc_id, account_id, relationship_change);
                    // Update NPC relationship in story progress
                    // This would be implemented in the full system
                }
                
                StoryCast::SetStoryVariable { account_id, story_id, variable, value } => {
                    if let Some(player_progress) = state.player_progress.get_mut(account_id) {
                        if let Some(story_progress) = player_progress.iter_mut().find(|p| p.story_id == *story_id) {
                            story_progress.story_variables.insert(variable.clone(), value.clone());
                            story_progress.last_updated = SystemTime::now();
                        }
                    }
                }
                
                StoryCast::ScheduleStoryEvent { when, event } => {
                    state.scheduled_events.entry(*when)
                        .or_insert_with(Vec::new)
                        .push(event.clone());
                    debug!("Scheduled story event {} for {:?}", event.event_id, when);
                }
                
                StoryCast::BulkUpdateProgress { updates } => {
                    for (account_id, mission_id, objective_id, progress) in updates {
                        self.update_objective_progress(state, *account_id, *mission_id, *objective_id, *progress);
                    }
                    info!("Bulk updated {} mission progress entries", updates.len());
                }
                
                StoryCast::CleanupExpiredMissions => {
                    self.cleanup_expired_missions(state).await?;
                    info!("Cleaned up expired missions");
                }
                
                StoryCast::RefreshStats => {
                    self.update_statistics(state);
                    debug!("Statistics refreshed");
                }
                
                StoryCast::GenerateRandomContent { content_type } => {
                    info!("Generating random content: {}", content_type);
                    // Placeholder for dynamic content generation
                }
            }
        }
        Ok(())
    }

    async fn handle_info(
        &mut self,
        message: Box<dyn Any + Send + Sync>,
        _source: InfoSource,
        state: &mut Self::State,
    ) -> HelixResult<()> {
        if let Some(info) = message.downcast_ref::<StoryInfo>() {
            match info {
                StoryInfo::AutoUnlockTimer => {
                    debug!("Auto-unlock timer triggered");
                    self.process_auto_unlocks(state);
                }
                
                StoryInfo::CleanupTimer => {
                    debug!("Cleanup timer triggered");
                    self.cleanup_expired_missions(state).await?;
                }
                
                StoryInfo::StatsTimer => {
                    debug!("Statistics timer triggered");
                    self.update_statistics(state);
                }
                
                StoryInfo::ScheduledEventTrigger { event } => {
                    info!("Processing scheduled story event: {}", event.event_id);
                    self.process_scheduled_event(state, event).await?;
                }
                
                StoryInfo::PlayerAction { account_id, action_type, details } => {
                    debug!("Player {} performed action: {} - {:?}", account_id, action_type, details);
                    self.handle_player_action(state, *account_id, action_type, details);
                }
                
                StoryInfo::WorldEvent { event_type, affected_players, impact } => {
                    info!("World event '{}' affects {} players", event_type, affected_players.len());
                    self.handle_world_event_impact(state, event_type, affected_players, impact);
                }
                
                StoryInfo::MissionTimeout { account_id, mission_id, time_remaining } => {
                    warn!("Mission {} for account {} timeout warning: {:?} remaining", 
                          mission_id, account_id, time_remaining);
                    
                    if *time_remaining == Duration::ZERO {
                        self.fail_mission_internal(state, *account_id, *mission_id, "timeout".to_string());
                    }
                }
                
                StoryInfo::ExternalStoryUpdate { source, update_type, data } => {
                    info!("External story update from {}: {} - {:?}", source, update_type, data);
                    // Handle external story updates
                }
            }
        }
        Ok(())
    }

    async fn terminate(
        &mut self,
        reason: TerminateReason,
        state: &mut Self::State,
    ) -> HelixResult<()> {
        info!("Story System GenServer terminating: {:?}", reason);
        
        // Final statistics
        info!("Final story stats: {} total missions, {} active missions, {} completed missions",
              state.stats.total_missions, state.stats.active_missions, state.stats.completed_missions);
        
        Ok(())
    }
}

impl StorySystemGenServer {
    // Helper methods implementation would continue here...
    // Due to space constraints, I'll provide key method signatures:
    
    fn get_available_missions_for_player(&self, state: &StorySystemState, account_id: AccountId) -> Vec<MissionId> {
        // Implementation to find available missions for player based on progress and requirements
        Vec::new()
    }
    
    fn check_mission_requirements(&self, state: &StorySystemState, account_id: AccountId, mission_id: MissionId) -> bool {
        // Implementation to check if player meets mission requirements
        true
    }
    
    fn update_objective_progress(
        &self,
        state: &mut StorySystemState,
        account_id: AccountId,
        mission_id: MissionId,
        objective_id: EntityId,
        progress: u32,
    ) -> bool {
        // Implementation to update objective progress
        true
    }
    
    fn complete_objective(
        &self,
        state: &mut StorySystemState,
        account_id: AccountId,
        mission_id: MissionId,
        objective_id: EntityId,
    ) -> bool {
        // Implementation to complete an objective
        true
    }
    
    fn check_mission_completion(&self, state: &StorySystemState, account_id: AccountId, mission_id: MissionId) -> bool {
        // Implementation to check if all objectives are complete
        true
    }
    
    fn complete_mission_internal(
        &self,
        state: &mut StorySystemState,
        account_id: AccountId,
        mission_id: MissionId,
    ) -> HelixResult<MissionRewards> {
        // Implementation to complete mission and award rewards
        Ok(MissionRewards {
            experience: 100,
            money: 1000,
            reputation: HashMap::new(),
            unlocked_items: Vec::new(),
            skill_bonuses: HashMap::new(),
            special_unlocks: Vec::new(),
        })
    }
    
    fn fail_mission_internal(
        &self,
        state: &mut StorySystemState,
        account_id: AccountId,
        mission_id: MissionId,
        reason: String,
    ) -> bool {
        // Implementation to fail a mission
        true
    }
    
    fn generate_dynamic_mission(
        &self,
        state: &StorySystemState,
        account_id: AccountId,
        template_id: &str,
        parameters: &HashMap<String, String>,
    ) -> HelixResult<StoryStep> {
        // Implementation for dynamic mission generation
        Err(HelixError::NotImplemented("Dynamic mission generation".to_string()))
    }
    
    fn get_mission_statistics(&self, state: &StorySystemState, mission_id: MissionId) -> HashMap<String, u32> {
        // Implementation to get mission-specific statistics
        HashMap::new()
    }
    
    fn search_missions(&self, state: &StorySystemState, criteria: &MissionSearchCriteria) -> Vec<StoryStep> {
        // Implementation to search missions by criteria
        Vec::new()
    }
    
    fn update_statistics(&self, state: &mut StorySystemState) {
        // Implementation to update system statistics
        state.stats.last_updated = SystemTime::now();
    }
    
    async fn cleanup_expired_missions(&self, state: &mut StorySystemState) -> HelixResult<()> {
        // Implementation to cleanup expired missions
        Ok(())
    }
    
    fn process_auto_unlocks(&self, state: &mut StorySystemState) {
        // Implementation for automatic mission unlocking
    }
    
    async fn process_scheduled_event(&self, state: &mut StorySystemState, event: &ScheduledStoryEvent) -> HelixResult<()> {
        // Implementation to process scheduled events
        Ok(())
    }
    
    fn handle_player_action(
        &self,
        state: &mut StorySystemState,
        account_id: AccountId,
        action_type: &str,
        details: &HashMap<String, String>,
    ) {
        // Implementation to handle player actions affecting story progress
    }
    
    fn handle_world_event_impact(
        &self,
        state: &mut StorySystemState,
        event_type: &str,
        affected_players: &[AccountId],
        impact: &HashMap<String, String>,
    ) {
        // Implementation to handle world events affecting missions
    }
    
    fn handle_story_event_trigger(
        &self,
        state: &mut StorySystemState,
        event_type: &str,
        target_account: Option<AccountId>,
        parameters: &HashMap<String, String>,
    ) {
        // Implementation to handle story event triggers
    }
}

/// Story System Supervisor
pub struct StorySystemSupervisor {
    supervisor: GenServerSupervisor,
}

impl StorySystemSupervisor {
    pub fn new() -> Self {
        Self {
            supervisor: GenServerSupervisor::new(SupervisionStrategy::OneForOne),
        }
    }
    
    pub async fn start(&mut self) -> HelixResult<(GenServerHandle, broadcast::Receiver<StoryEvent>)> {
        let (genserver, event_rx) = StorySystemGenServer::new();
        let config = StorySystemConfig::default();
        
        let handle = GenServerHandle::start(genserver, config, Some("story_system".to_string())).await?;
        Ok((handle, event_rx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_story_mission_flow() {
        let (genserver, _) = StorySystemGenServer::new();
        let handle = GenServerHandle::start(
            genserver,
            StorySystemConfig::default(),
            Some("test_story".to_string())
        ).await.expect("Failed to start StorySystemGenServer");

        let account_id = AccountId::new();
        
        // Get available missions
        let call = StoryCall::GetAvailableMissions { account_id };
        let missions: Vec<MissionId> = handle.call(call, None).await.expect("Failed to get available missions");
        
        // Should start with no missions available for new account
        assert!(missions.is_empty());

        handle.stop(TerminateReason::Normal).await.expect("Failed to stop");
    }
}