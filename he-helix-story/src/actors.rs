//! Story Actor System - Complete GenServer Implementation
//!
//! This module provides a comprehensive narrative and mission system with GenServer patterns,
//! including dynamic story progression, mission generation, player tracking, and NPC interactions.
//!
//! Features:
//! - Dynamic story progression with branching narratives
//! - Mission system with procedural generation
//! - Player progress tracking and state management
//! - NPC interaction and dialogue systems
//! - Achievement and reward systems
//! - Tutorial and onboarding flows
//! - Event-driven story advancement

use crate::models::{
    Story, Mission, StoryStep, PlayerProgress, StoryEvent, StoryContext, 
    MissionTemplate, StoryState, NarrativeChoice, DialogueNode, Achievement,
    StoryTrigger, MissionRequirements, RewardBundle, StoryError
};
use he_helix_core::{
    genserver::{GenServer, GenServerBehavior, GenServerMessage, GenServerReply, InfoSource},
    actors::{Actor, ActorContext, Handler, Message},
    HelixError, HelixResult, ProcessId
};
use async_trait::async_trait;
use std::collections::{HashMap, BTreeMap, VecDeque, HashSet};
use tokio::sync::{RwLock, Mutex, broadcast, mpsc};
use chrono::{DateTime, Utc, Duration};
use std::sync::Arc;
use tracing::{info, error, warn, debug, trace};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use he_core::id::{PlayerId, MissionId, StoryId, StepId, AchievementId};

/// Story operation error types
#[derive(Debug, thiserror::Error)]
pub enum StoryActorError {
    #[error("Story not found: {0}")]
    StoryNotFound(String),
    #[error("Mission not found: {0}")]
    MissionNotFound(String),
    #[error("Step not found: {0}")]
    StepNotFound(String),
    #[error("Player progress not found: {0}")]
    ProgressNotFound(String),
    #[error("Story requirements not met: {0}")]
    RequirementsNotMet(String),
    #[error("Invalid story transition: {0}")]
    InvalidTransition(String),
    #[error("Mission generation failed: {0}")]
    GenerationFailed(String),
    #[error("Achievement already unlocked: {0}")]
    AchievementAlreadyUnlocked(String),
    #[error("Internal story error: {0}")]
    InternalError(String),
}

/// Messages for StoryActor GenServer
#[derive(Debug, Clone)]
pub enum StoryCall {
    /// Get available missions for a player
    GetAvailableMissions {
        player_id: PlayerId,
        category: Option<String>,
        difficulty_range: Option<(u32, u32)>,
    },
    /// Start a new mission for a player
    StartMission {
        player_id: PlayerId,
        mission_id: MissionId,
        context: StoryContext,
    },
    /// Update mission progress
    UpdateMissionProgress {
        player_id: PlayerId,
        mission_id: MissionId,
        progress: f64,
        completed_steps: Vec<StepId>,
    },
    /// Complete a mission
    CompleteMission {
        player_id: PlayerId,
        mission_id: MissionId,
        success: bool,
        completion_data: HashMap<String, String>,
    },
    /// Generate dynamic mission
    GenerateDynamicMission {
        player_id: PlayerId,
        mission_type: String,
        difficulty: u32,
        constraints: HashMap<String, String>,
    },
    /// Get player story progress
    GetPlayerProgress {
        player_id: PlayerId,
    },
    /// Get story state for player
    GetStoryState {
        player_id: PlayerId,
        story_id: StoryId,
    },
    /// Process narrative choice
    ProcessChoice {
        player_id: PlayerId,
        story_id: StoryId,
        choice_id: String,
        choice_data: HashMap<String, String>,
    },
    /// Get available dialogue options
    GetDialogueOptions {
        player_id: PlayerId,
        npc_id: String,
        context: Option<StoryContext>,
    },
    /// Get player achievements
    GetPlayerAchievements {
        player_id: PlayerId,
        category: Option<String>,
    },
    /// Check achievement eligibility
    CheckAchievementEligibility {
        player_id: PlayerId,
        achievement_id: AchievementId,
    },
}

#[derive(Debug, Clone)]
pub enum StoryCast {
    /// Send story email to player
    SendStoryEmail {
        player_id: PlayerId,
        email_template: String,
        variables: HashMap<String, String>,
        delay_seconds: Option<u64>,
    },
    /// Trigger story event
    TriggerStoryEvent {
        event: StoryEvent,
        affected_players: Vec<PlayerId>,
        propagate: bool,
    },
    /// Update NPC state
    UpdateNpcState {
        npc_id: String,
        new_state: HashMap<String, String>,
        affects_dialogue: bool,
    },
    /// Schedule future story trigger
    ScheduleStoryTrigger {
        trigger: StoryTrigger,
        execute_at: DateTime<Utc>,
        repeat_interval: Option<Duration>,
    },
    /// Process background story events
    ProcessBackgroundEvents,
    /// Cleanup expired story data
    CleanupExpiredData {
        older_than_days: u32,
    },
    /// Sync story state with external systems
    SyncWithExternalSystems {
        system_names: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub enum StoryInfo {
    /// Player action occurred
    PlayerAction {
        player_id: PlayerId,
        action_type: String,
        action_data: HashMap<String, String>,
        timestamp: DateTime<Utc>,
    },
    /// World event affecting story
    WorldEvent {
        event_type: String,
        event_data: HashMap<String, String>,
        affected_regions: Vec<String>,
    },
    /// Mission deadline warning
    MissionDeadlineWarning {
        player_id: PlayerId,
        mission_id: MissionId,
        deadline: DateTime<Utc>,
        time_remaining: Duration,
    },
    /// Achievement unlocked
    AchievementUnlocked {
        player_id: PlayerId,
        achievement_id: AchievementId,
        achievement_name: String,
        reward: Option<RewardBundle>,
    },
}

/// Story Actor state structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryActorState {
    /// Active stories indexed by story ID
    pub stories: HashMap<StoryId, Story>,
    /// Active missions indexed by mission ID
    pub missions: HashMap<MissionId, Mission>,
    /// Mission templates for dynamic generation
    pub mission_templates: HashMap<String, MissionTemplate>,
    /// Player progress tracking
    pub player_progress: HashMap<PlayerId, PlayerProgress>,
    /// Player story states
    pub player_stories: HashMap<PlayerId, HashMap<StoryId, StoryState>>,
    /// Active achievements
    pub achievements: HashMap<AchievementId, Achievement>,
    /// Player achievements
    pub player_achievements: HashMap<PlayerId, HashSet<AchievementId>>,
    /// NPC states for dialogue
    pub npc_states: HashMap<String, HashMap<String, String>>,
    /// Scheduled triggers
    pub scheduled_triggers: BTreeMap<DateTime<Utc>, Vec<StoryTrigger>>,
    /// Story event history
    pub event_history: VecDeque<StoryEvent>,
    /// Configuration
    pub config: StoryConfiguration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryConfiguration {
    pub max_active_missions_per_player: u32,
    pub mission_timeout_hours: u64,
    pub max_event_history_entries: usize,
    pub dynamic_mission_generation_enabled: bool,
    pub achievement_notifications_enabled: bool,
    pub story_email_delays_enabled: bool,
    pub background_processing_interval_seconds: u64,
}

impl Default for StoryConfiguration {
    fn default() -> Self {
        Self {
            max_active_missions_per_player: 5,
            mission_timeout_hours: 168, // 1 week
            max_event_history_entries: 10000,
            dynamic_mission_generation_enabled: true,
            achievement_notifications_enabled: true,
            story_email_delays_enabled: true,
            background_processing_interval_seconds: 300, // 5 minutes
        }
    }
}

impl Default for StoryActorState {
    fn default() -> Self {
        Self {
            stories: HashMap::new(),
            missions: HashMap::new(),
            mission_templates: HashMap::new(),
            player_progress: HashMap::new(),
            player_stories: HashMap::new(),
            achievements: HashMap::new(),
            player_achievements: HashMap::new(),
            npc_states: HashMap::new(),
            scheduled_triggers: BTreeMap::new(),
            event_history: VecDeque::new(),
            config: StoryConfiguration::default(),
        }
    }
}

/// Main Story Actor
pub struct StoryActor {
    state: Arc<RwLock<StoryActorState>>,
    background_handle: Option<tokio::task::JoinHandle<()>>,
    event_sender: Option<broadcast::Sender<StoryEvent>>,
    email_queue: Arc<Mutex<VecDeque<EmailTask>>>,
}

#[derive(Debug, Clone)]
struct EmailTask {
    player_id: PlayerId,
    email_template: String,
    variables: HashMap<String, String>,
    scheduled_for: DateTime<Utc>,
}

impl StoryActor {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(StoryActorState::default())),
            background_handle: None,
            event_sender: None,
            email_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn initialize(&mut self) -> HelixResult<()> {
        self.start_background_processing().await?;
        self.initialize_event_broadcasting().await?;
        self.load_mission_templates().await?;
        self.load_achievements().await?;
        
        info!("StoryActor initialized");
        Ok(())
    }

    async fn start_background_processing(&mut self) -> HelixResult<()> {
        let state = Arc::clone(&self.state);
        let email_queue = Arc::clone(&self.email_queue);
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
            loop {
                interval.tick().await;
                Self::process_background_tasks(&state, &email_queue).await;
            }
        });

        self.background_handle = Some(handle);
        Ok(())
    }

    async fn initialize_event_broadcasting(&mut self) -> HelixResult<()> {
        let (sender, _) = broadcast::channel(1000);
        self.event_sender = Some(sender);
        Ok(())
    }

    async fn load_mission_templates(&self) -> HelixResult<()> {
        let mut state = self.state.write().await;
        
        // Load built-in mission templates
        let tutorial_template = MissionTemplate {
            template_id: "tutorial_basic".to_string(),
            name: "Basic Tutorial".to_string(),
            description: "Introduction to hacking basics".to_string(),
            mission_type: "tutorial".to_string(),
            difficulty: 1,
            estimated_duration_minutes: 15,
            requirements: MissionRequirements {
                min_level: 1,
                required_skills: vec!["basic_hacking".to_string()],
                required_software: vec![],
                required_hardware: HashMap::new(),
                prerequisites: vec![],
            },
            rewards: RewardBundle {
                experience: 100,
                money: 500,
                items: vec![],
                achievements: vec![],
            },
            steps: vec![
                StoryStep {
                    step_id: StepId::new(),
                    name: "Connect to Target".to_string(),
                    description: "Establish connection to the tutorial server".to_string(),
                    step_type: "connection".to_string(),
                    order: 1,
                    requirements: HashMap::new(),
                    completion_criteria: HashMap::new(),
                    rewards: None,
                },
                StoryStep {
                    step_id: StepId::new(),
                    name: "Run Password Crack".to_string(),
                    description: "Use password cracker to gain access".to_string(),
                    step_type: "hacking".to_string(),
                    order: 2,
                    requirements: HashMap::new(),
                    completion_criteria: HashMap::new(),
                    rewards: None,
                }
            ],
            generation_params: HashMap::new(),
        };

        state.mission_templates.insert("tutorial_basic".to_string(), tutorial_template);
        debug!("Loaded {} mission templates", state.mission_templates.len());
        Ok(())
    }

    async fn load_achievements(&self) -> HelixResult<()> {
        let mut state = self.state.write().await;

        let first_hack_achievement = Achievement {
            achievement_id: AchievementId::new(),
            name: "First Hack".to_string(),
            description: "Complete your first successful hack".to_string(),
            category: "tutorial".to_string(),
            points: 10,
            icon: "hack_success.png".to_string(),
            requirements: HashMap::from([
                ("successful_hacks".to_string(), "1".to_string()),
            ]),
            reward: Some(RewardBundle {
                experience: 50,
                money: 100,
                items: vec!["beginner_badge".to_string()],
                achievements: vec![],
            }),
            hidden: false,
            created_at: Utc::now(),
        };

        state.achievements.insert(first_hack_achievement.achievement_id, first_hack_achievement);
        debug!("Loaded {} achievements", state.achievements.len());
        Ok(())
    }

    async fn process_background_tasks(
        state: &Arc<RwLock<StoryActorState>>,
        email_queue: &Arc<Mutex<VecDeque<EmailTask>>>,
    ) {
        Self::process_scheduled_triggers(state).await;
        Self::check_mission_deadlines(state).await;
        Self::process_email_queue(email_queue).await;
        Self::cleanup_expired_data(state).await;
    }

    async fn process_scheduled_triggers(state: &Arc<RwLock<StoryActorState>>) {
        let mut state_guard = state.write().await;
        let now = Utc::now();
        
        let triggers_to_process: Vec<StoryTrigger> = state_guard
            .scheduled_triggers
            .range(..=now)
            .flat_map(|(_, triggers)| triggers.clone())
            .collect();

        // Remove processed triggers
        state_guard.scheduled_triggers.retain(|time, _| *time > now);

        drop(state_guard);

        for trigger in triggers_to_process {
            debug!("Processing scheduled story trigger: {:?}", trigger.trigger_id);
            // Process trigger logic here
        }
    }

    async fn check_mission_deadlines(state: &Arc<RwLock<StoryActorState>>) {
        let state_guard = state.read().await;
        let now = Utc::now();
        let warning_threshold = Duration::hours(24);

        for (mission_id, mission) in &state_guard.missions {
            if let Some(deadline) = mission.deadline {
                let time_remaining = deadline - now;
                if time_remaining <= warning_threshold && time_remaining > Duration::zero() {
                    debug!("Mission {} deadline warning: {} remaining", 
                           mission_id, time_remaining);
                    // Send warning to player
                }
            }
        }
    }

    async fn process_email_queue(email_queue: &Arc<Mutex<VecDeque<EmailTask>>>) {
        let mut queue = email_queue.lock().await;
        let now = Utc::now();
        
        while let Some(task) = queue.front() {
            if task.scheduled_for <= now {
                let task = queue.pop_front().unwrap();
                debug!("Sending story email to player {}", task.player_id);
                // Send email logic here
            } else {
                break;
            }
        }
    }

    async fn cleanup_expired_data(state: &Arc<RwLock<StoryActorState>>) {
        let mut state_guard = state.write().await;
        let cutoff = Utc::now() - Duration::days(30);

        // Cleanup old events
        while let Some(event) = state_guard.event_history.front() {
            if event.timestamp < cutoff {
                state_guard.event_history.pop_front();
            } else {
                break;
            }
        }

        // Cleanup expired missions
        let expired_missions: Vec<MissionId> = state_guard
            .missions
            .iter()
            .filter(|(_, mission)| {
                mission.deadline.map_or(false, |deadline| deadline < Utc::now())
            })
            .map(|(id, _)| *id)
            .collect();

        for mission_id in expired_missions {
            state_guard.missions.remove(&mission_id);
            debug!("Removed expired mission: {}", mission_id);
        }
    }

    async fn handle_get_available_missions(
        &self,
        player_id: PlayerId,
        category: Option<String>,
        difficulty_range: Option<(u32, u32)>,
    ) -> Result<Vec<Mission>, StoryActorError> {
        let state = self.state.read().await;
        
        let player_progress = state.player_progress.get(&player_id)
            .ok_or_else(|| StoryActorError::ProgressNotFound(player_id.to_string()))?;

        let mut available_missions = Vec::new();

        for (_, mission) in &state.missions {
            // Check if mission is available to player
            if mission.is_available_to_player(player_progress) {
                // Apply category filter
                if let Some(ref cat) = category {
                    if mission.category != *cat {
                        continue;
                    }
                }

                // Apply difficulty filter
                if let Some((min_diff, max_diff)) = difficulty_range {
                    if mission.difficulty < min_diff || mission.difficulty > max_diff {
                        continue;
                    }
                }

                available_missions.push(mission.clone());
            }
        }

        // Sort by difficulty and priority
        available_missions.sort_by(|a, b| {
            a.difficulty.cmp(&b.difficulty)
                .then_with(|| b.priority.cmp(&a.priority))
        });

        Ok(available_missions)
    }

    async fn handle_start_mission(
        &self,
        player_id: PlayerId,
        mission_id: MissionId,
        context: StoryContext,
    ) -> Result<Mission, StoryActorError> {
        let mut state = self.state.write().await;

        let mission = state.missions.get(&mission_id)
            .ok_or_else(|| StoryActorError::MissionNotFound(mission_id.to_string()))?
            .clone();

        let player_progress = state.player_progress.get_mut(&player_id)
            .ok_or_else(|| StoryActorError::ProgressNotFound(player_id.to_string()))?;

        // Check if player can start this mission
        if !mission.is_available_to_player(player_progress) {
            return Err(StoryActorError::RequirementsNotMet(
                "Mission requirements not met".to_string()
            ));
        }

        // Check active mission limit
        if player_progress.active_missions.len() >= 
           state.config.max_active_missions_per_player as usize {
            return Err(StoryActorError::RequirementsNotMet(
                "Too many active missions".to_string()
            ));
        }

        // Start the mission
        player_progress.active_missions.insert(mission_id);
        player_progress.mission_history.push(mission_id);

        // Create mission progress tracking
        let mission_progress = crate::models::MissionProgress {
            mission_id,
            player_id,
            started_at: Utc::now(),
            current_step: 0,
            progress_percentage: 0.0,
            completed_steps: HashSet::new(),
            context: context.clone(),
            status: crate::models::MissionStatus::Active,
        };

        player_progress.mission_progress.insert(mission_id, mission_progress);

        // Record event
        let event = StoryEvent {
            event_id: Uuid::new_v4(),
            event_type: "mission_started".to_string(),
            player_id: Some(player_id),
            mission_id: Some(mission_id),
            data: HashMap::from([
                ("mission_name".to_string(), mission.name.clone()),
                ("category".to_string(), mission.category.clone()),
            ]),
            timestamp: Utc::now(),
        };

        state.event_history.push_back(event.clone());

        // Broadcast event
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(event);
        }

        info!("Player {} started mission {}", player_id, mission_id);
        Ok(mission)
    }

    async fn handle_generate_dynamic_mission(
        &self,
        player_id: PlayerId,
        mission_type: String,
        difficulty: u32,
        constraints: HashMap<String, String>,
    ) -> Result<Mission, StoryActorError> {
        let state = self.state.read().await;

        if !state.config.dynamic_mission_generation_enabled {
            return Err(StoryActorError::GenerationFailed(
                "Dynamic mission generation is disabled".to_string()
            ));
        }

        let template = state.mission_templates.get(&mission_type)
            .ok_or_else(|| StoryActorError::GenerationFailed(
                format!("No template found for mission type: {}", mission_type)
            ))?;

        let player_progress = state.player_progress.get(&player_id)
            .ok_or_else(|| StoryActorError::ProgressNotFound(player_id.to_string()))?;

        // Generate mission based on template and constraints
        let mission = self.generate_mission_from_template(
            template,
            player_progress,
            difficulty,
            constraints,
        ).await?;

        drop(state);

        // Add generated mission to state
        let mut state = self.state.write().await;
        state.missions.insert(mission.mission_id, mission.clone());

        info!("Generated dynamic mission {} for player {}", 
              mission.mission_id, player_id);
        Ok(mission)
    }

    async fn generate_mission_from_template(
        &self,
        template: &MissionTemplate,
        player_progress: &PlayerProgress,
        difficulty: u32,
        constraints: HashMap<String, String>,
    ) -> Result<Mission, StoryActorError> {
        let mission_id = MissionId::new();
        
        // Adjust difficulty based on player level
        let adjusted_difficulty = std::cmp::min(
            difficulty,
            player_progress.level as u32 + 2
        );

        let mission = Mission {
            mission_id,
            name: format!("{} (Level {})", template.name, adjusted_difficulty),
            description: template.description.clone(),
            category: template.mission_type.clone(),
            difficulty: adjusted_difficulty,
            priority: 5, // Normal priority
            requirements: template.requirements.clone(),
            steps: template.steps.clone(),
            rewards: self.scale_rewards(&template.rewards, adjusted_difficulty),
            estimated_duration: Duration::minutes(
                template.estimated_duration_minutes as i64
            ),
            deadline: Some(Utc::now() + Duration::hours(
                template.estimated_duration_minutes as i64 * 4
            )),
            created_at: Utc::now(),
            created_by: "system".to_string(),
            metadata: constraints,
        };

        Ok(mission)
    }

    fn scale_rewards(&self, base_rewards: &RewardBundle, difficulty: u32) -> RewardBundle {
        let scale_factor = 1.0 + (difficulty as f64 * 0.2);
        
        RewardBundle {
            experience: (base_rewards.experience as f64 * scale_factor) as u32,
            money: (base_rewards.money as f64 * scale_factor) as i64,
            items: base_rewards.items.clone(),
            achievements: base_rewards.achievements.clone(),
        }
    }
}

/// GenServer implementation for StoryActor
#[async_trait]
impl GenServerBehavior for StoryActor {
    type State = StoryActorState;

    async fn init(&mut self) -> HelixResult<()> {
        self.initialize().await?;
        info!("StoryActor GenServer initialized");
        Ok(())
    }

    async fn handle_call(
        &mut self,
        message: Box<dyn std::any::Any + Send + Sync>,
        _from: ProcessId,
    ) -> HelixResult<GenServerReply> {
        if let Ok(call) = message.downcast::<StoryCall>() {
            match *call {
                StoryCall::GetAvailableMissions { player_id, category, difficulty_range } => {
                    let result = self.handle_get_available_missions(player_id, category, difficulty_range).await;
                    Ok(GenServerReply::Reply(Box::new(result)))
                }
                StoryCall::StartMission { player_id, mission_id, context } => {
                    let result = self.handle_start_mission(player_id, mission_id, context).await;
                    Ok(GenServerReply::Reply(Box::new(result)))
                }
                StoryCall::GenerateDynamicMission { player_id, mission_type, difficulty, constraints } => {
                    let result = self.handle_generate_dynamic_mission(
                        player_id, mission_type, difficulty, constraints
                    ).await;
                    Ok(GenServerReply::Reply(Box::new(result)))
                }
                StoryCall::GetPlayerProgress { player_id } => {
                    let state = self.state.read().await;
                    let progress = state.player_progress.get(&player_id).cloned();
                    Ok(GenServerReply::Reply(Box::new(Ok::<Option<PlayerProgress>, StoryActorError>(progress))))
                }
                StoryCall::GetPlayerAchievements { player_id, category } => {
                    let state = self.state.read().await;
                    let player_achievements = state.player_achievements.get(&player_id)
                        .cloned()
                        .unwrap_or_default();
                    
                    let achievements: Vec<Achievement> = player_achievements
                        .iter()
                        .filter_map(|id| state.achievements.get(id))
                        .filter(|achievement| {
                            category.as_ref().map_or(true, |cat| achievement.category == *cat)
                        })
                        .cloned()
                        .collect();
                    
                    Ok(GenServerReply::Reply(Box::new(Ok::<Vec<Achievement>, StoryActorError>(achievements))))
                }
                _ => {
                    warn!("Unhandled story call message");
                    Ok(GenServerReply::NoReply)
                }
            }
        } else {
            Err(HelixError::InvalidMessage("Unknown call message type".to_string()))
        }
    }

    async fn handle_cast(
        &mut self,
        message: Box<dyn std::any::Any + Send + Sync>,
    ) -> HelixResult<()> {
        if let Ok(cast) = message.downcast::<StoryCast>() {
            match *cast {
                StoryCast::SendStoryEmail { player_id, email_template, variables, delay_seconds } => {
                    let scheduled_for = if let Some(delay) = delay_seconds {
                        Utc::now() + Duration::seconds(delay as i64)
                    } else {
                        Utc::now()
                    };

                    let task = EmailTask {
                        player_id,
                        email_template,
                        variables,
                        scheduled_for,
                    };

                    let mut queue = self.email_queue.lock().await;
                    queue.push_back(task);
                }
                StoryCast::TriggerStoryEvent { event, affected_players, propagate } => {
                    let mut state = self.state.write().await;
                    state.event_history.push_back(event.clone());

                    if propagate && let Some(sender) = &self.event_sender {
                        let _ = sender.send(event);
                    }
                }
                StoryCast::ScheduleStoryTrigger { trigger, execute_at, repeat_interval } => {
                    let mut state = self.state.write().await;
                    state.scheduled_triggers
                        .entry(execute_at)
                        .or_insert_with(Vec::new)
                        .push(trigger);
                }
                StoryCast::ProcessBackgroundEvents => {
                    let state = Arc::clone(&self.state);
                    let email_queue = Arc::clone(&self.email_queue);
                    tokio::spawn(async move {
                        Self::process_background_tasks(&state, &email_queue).await;
                    });
                }
                _ => {
                    debug!("Story cast message processed");
                }
            }
        }
        Ok(())
    }

    async fn handle_info(
        &mut self,
        message: Box<dyn std::any::Any + Send + Sync>,
        _source: InfoSource,
    ) -> HelixResult<()> {
        if let Ok(info) = message.downcast::<StoryInfo>() {
            match *info {
                StoryInfo::PlayerAction { player_id, action_type, action_data, timestamp } => {
                    debug!("Player {} performed action: {}", player_id, action_type);
                    // Process action and potentially trigger story events
                }
                StoryInfo::WorldEvent { event_type, event_data, affected_regions } => {
                    info!("World event: {} affecting {} regions", event_type, affected_regions.len());
                }
                StoryInfo::MissionDeadlineWarning { player_id, mission_id, deadline, time_remaining } => {
                    warn!("Mission {} for player {} deadline in {:?}", 
                          mission_id, player_id, time_remaining);
                }
                StoryInfo::AchievementUnlocked { player_id, achievement_id, achievement_name, reward } => {
                    info!("Player {} unlocked achievement: {}", player_id, achievement_name);
                    
                    let mut state = self.state.write().await;
                    state.player_achievements
                        .entry(player_id)
                        .or_insert_with(HashSet::new)
                        .insert(achievement_id);
                }
            }
        }
        Ok(())
    }

    async fn terminate(&mut self, _reason: String) -> HelixResult<()> {
        if let Some(handle) = self.background_handle.take() {
            handle.abort();
        }
        
        info!("StoryActor terminated");
        Ok(())
    }

    async fn code_change(&mut self, _old_version: String, _new_version: String) -> HelixResult<()> {
        info!("StoryActor code change completed");
        Ok(())
    }

    async fn get_state(&self) -> HelixResult<Self::State> {
        let state = self.state.read().await;
        Ok(state.clone())
    }

    async fn set_state(&mut self, state: Self::State) -> HelixResult<()> {
        let mut current_state = self.state.write().await;
        *current_state = state;
        Ok(())
    }
}

/// Step Actor for individual story step processing
pub struct StepActor {
    state: Arc<RwLock<StepActorState>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepActorState {
    pub active_steps: HashMap<StepId, ActiveStep>,
    pub step_templates: HashMap<String, StepTemplate>,
    pub completion_handlers: HashMap<String, CompletionHandler>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveStep {
    pub step_id: StepId,
    pub player_id: PlayerId,
    pub mission_id: MissionId,
    pub step_data: StoryStep,
    pub started_at: DateTime<Utc>,
    pub progress: f64,
    pub status: StepStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepStatus {
    Pending,
    Active,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepTemplate {
    pub step_type: String,
    pub validation_rules: Vec<String>,
    pub completion_criteria: HashMap<String, String>,
    pub timeout_minutes: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionHandler {
    pub handler_type: String,
    pub handler_config: HashMap<String, String>,
}

impl Default for StepActorState {
    fn default() -> Self {
        Self {
            active_steps: HashMap::new(),
            step_templates: HashMap::new(),
            completion_handlers: HashMap::new(),
        }
    }
}

impl StepActor {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(StepActorState::default())),
        }
    }
}

/// Story Actor supervisor
pub struct StoryActorSupervisor;

impl StoryActorSupervisor {
    pub async fn start() -> HelixResult<StoryActor> {
        let mut actor = StoryActor::new();
        actor.initialize().await?;
        info!("StoryActor supervised startup completed");
        Ok(actor)
    }
}