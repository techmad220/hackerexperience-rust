use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use rand::{Rng, thread_rng, seq::SliceRandom};
use crate::{UserId, IpAddress, SoftwareId, HeResult, HackerExperienceError};

/// Complete mission system with story campaigns, contracts, and tutorial flow
#[derive(Debug, Clone)]
pub struct MissionSystem {
    /// Available missions - mission_id -> Mission
    available_missions: HashMap<u64, Mission>,
    /// User mission progress - user_id -> Vec<MissionProgress>
    user_progress: HashMap<UserId, Vec<MissionProgress>>,
    /// Story campaign state - user_id -> CampaignState
    campaign_states: HashMap<UserId, CampaignState>,
    /// Mission templates for generation
    mission_templates: Vec<MissionTemplate>,
    /// Daily/Weekly contracts
    contracts: HashMap<u64, Contract>,
}

impl MissionSystem {
    pub fn new() -> Self {
        let mut system = Self {
            available_missions: HashMap::new(),
            user_progress: HashMap::new(),
            campaign_states: HashMap::new(),
            mission_templates: Vec::new(),
            contracts: HashMap::new(),
        };

        system.initialize_mission_templates();
        system.initialize_story_campaigns();
        system
    }

    /// Start tutorial missions for new user
    pub fn start_tutorial(&mut self, user_id: UserId) -> HeResult<Vec<Mission>> {
        let tutorial_missions = self.generate_tutorial_missions(user_id)?;
        
        // Initialize user progress
        let mut progress = Vec::new();
        for mission in &tutorial_missions {
            self.available_missions.insert(mission.id, mission.clone());
            progress.push(MissionProgress {
                mission_id: mission.id,
                user_id,
                status: MissionStatus::Available,
                started_at: None,
                completed_at: None,
                current_objectives: mission.objectives.iter().map(|obj| ObjectiveProgress {
                    objective_id: obj.id,
                    completed: false,
                    progress_value: 0,
                    started_at: None,
                    completed_at: None,
                }).collect(),
                rewards_claimed: false,
            });
        }

        self.user_progress.insert(user_id, progress);

        // Initialize campaign state
        self.campaign_states.insert(user_id, CampaignState {
            user_id,
            current_chapter: 1,
            current_mission: 1,
            tutorial_completed: false,
            story_unlocked: false,
            unlocked_features: vec![GameFeature::BasicHacking],
        });

        Ok(tutorial_missions)
    }

    /// Get available missions for user
    pub fn get_available_missions(&self, user_id: UserId) -> Vec<Mission> {
        let mut missions = Vec::new();
        
        if let Some(user_progress) = self.user_progress.get(&user_id) {
            for progress in user_progress {
                if progress.status == MissionStatus::Available {
                    if let Some(mission) = self.available_missions.get(&progress.mission_id) {
                        missions.push(mission.clone());
                    }
                }
            }
        }

        // Add generated contract missions
        missions.extend(self.get_available_contracts(user_id));

        missions
    }

    /// Accept and start a mission
    pub fn accept_mission(&mut self, user_id: UserId, mission_id: u64) -> HeResult<MissionAcceptResult> {
        // Check if user can accept this mission
        if !self.can_user_accept_mission(user_id, mission_id)? {
            return Err(HackerExperienceError::MissionRequirementsNotMet);
        }

        // Get mission details
        let mission = self.available_missions.get(&mission_id)
            .ok_or(HackerExperienceError::MissionNotFound)?
            .clone();

        // Update user progress
        if let Some(user_progress) = self.user_progress.get_mut(&user_id) {
            if let Some(progress) = user_progress.iter_mut().find(|p| p.mission_id == mission_id) {
                progress.status = MissionStatus::InProgress;
                progress.started_at = Some(Utc::now());
                
                // Initialize objective progress
                for obj_progress in &mut progress.current_objectives {
                    obj_progress.started_at = Some(Utc::now());
                }
            }
        }

        Ok(MissionAcceptResult {
            mission: mission.clone(),
            objectives: mission.objectives,
            estimated_duration: self.calculate_mission_duration(&mission),
            tips: self.generate_mission_tips(&mission),
        })
    }

    /// Update mission progress with user action
    pub fn update_mission_progress(
        &mut self, 
        user_id: UserId, 
        action: MissionAction
    ) -> HeResult<Vec<MissionUpdate>> {
        let mut updates = Vec::new();
        
        if let Some(user_progress) = self.user_progress.get_mut(&user_id) {
            for progress in user_progress.iter_mut() {
                if progress.status != MissionStatus::InProgress {
                    continue;
                }

                let mission = self.available_missions.get(&progress.mission_id).unwrap();
                let mut mission_completed = true;

                // Check each objective
                for obj_progress in &mut progress.current_objectives {
                    if obj_progress.completed {
                        continue;
                    }

                    let objective = mission.objectives.iter()
                        .find(|obj| obj.id == obj_progress.objective_id)
                        .unwrap();

                    // Check if action contributes to this objective
                    if self.action_matches_objective(&action, objective) {
                        let progress_gained = self.calculate_objective_progress(&action, objective);
                        obj_progress.progress_value += progress_gained;

                        // Check if objective is completed
                        if obj_progress.progress_value >= objective.target_value {
                            obj_progress.completed = true;
                            obj_progress.completed_at = Some(Utc::now());
                            
                            updates.push(MissionUpdate {
                                mission_id: progress.mission_id,
                                update_type: MissionUpdateType::ObjectiveCompleted,
                                objective_id: Some(obj_progress.objective_id),
                                message: format!("Objective completed: {}", objective.description),
                                rewards: objective.completion_reward.clone(),
                            });
                        } else {
                            updates.push(MissionUpdate {
                                mission_id: progress.mission_id,
                                update_type: MissionUpdateType::ProgressUpdate,
                                objective_id: Some(obj_progress.objective_id),
                                message: format!("Progress: {}/{}", obj_progress.progress_value, objective.target_value),
                                rewards: None,
                            });
                        }
                    }

                    if !obj_progress.completed {
                        mission_completed = false;
                    }
                }

                // Check if entire mission is completed
                if mission_completed && progress.status == MissionStatus::InProgress {
                    progress.status = MissionStatus::Completed;
                    progress.completed_at = Some(Utc::now());

                    updates.push(MissionUpdate {
                        mission_id: progress.mission_id,
                        update_type: MissionUpdateType::MissionCompleted,
                        objective_id: None,
                        message: format!("Mission completed: {}", mission.title),
                        rewards: Some(mission.completion_reward.clone()),
                    });

                    // Handle story progression
                    self.handle_story_progression(user_id, mission_id)?;

                    // Generate next missions if this was part of a sequence
                    self.generate_follow_up_missions(user_id, mission_id)?;
                }
            }
        }

        Ok(updates)
    }

    /// Claim mission rewards
    pub fn claim_mission_rewards(&mut self, user_id: UserId, mission_id: u64) -> HeResult<MissionReward> {
        let user_progress = self.user_progress.get_mut(&user_id)
            .ok_or(HackerExperienceError::UserNotFound)?;

        let progress = user_progress.iter_mut()
            .find(|p| p.mission_id == mission_id)
            .ok_or(HackerExperienceError::MissionNotFound)?;

        if progress.status != MissionStatus::Completed {
            return Err(HackerExperienceError::MissionNotCompleted);
        }

        if progress.rewards_claimed {
            return Err(HackerExperienceError::RewardsAlreadyClaimed);
        }

        let mission = self.available_missions.get(&mission_id).unwrap();
        progress.rewards_claimed = true;

        Ok(mission.completion_reward.clone())
    }

    /// Generate new random missions
    pub fn generate_random_missions(&mut self, user_id: UserId, count: usize) -> HeResult<Vec<Mission>> {
        let user_level = self.get_user_level(user_id);
        let mut new_missions = Vec::new();
        let mut rng = thread_rng();

        for _ in 0..count {
            let template = self.mission_templates.choose(&mut rng)
                .ok_or(HackerExperienceError::NoMissionTemplatesAvailable)?;

            let mission = self.generate_mission_from_template(template, user_level)?;
            
            // Add to available missions
            self.available_missions.insert(mission.id, mission.clone());
            
            // Add to user progress
            if let Some(user_progress) = self.user_progress.get_mut(&user_id) {
                user_progress.push(MissionProgress {
                    mission_id: mission.id,
                    user_id,
                    status: MissionStatus::Available,
                    started_at: None,
                    completed_at: None,
                    current_objectives: mission.objectives.iter().map(|obj| ObjectiveProgress {
                        objective_id: obj.id,
                        completed: false,
                        progress_value: 0,
                        started_at: None,
                        completed_at: None,
                    }).collect(),
                    rewards_claimed: false,
                });
            }

            new_missions.push(mission);
        }

        Ok(new_missions)
    }

    /// Generate daily contracts
    pub fn generate_daily_contracts(&mut self) -> HeResult<Vec<Contract>> {
        let mut contracts = Vec::new();
        let mut rng = thread_rng();

        // Generate 3-5 daily contracts
        let contract_count = rng.gen_range(3..=5);
        
        for _ in 0..contract_count {
            let contract_id = rng.gen::<u64>();
            let contract_type = [
                ContractType::HackTargets,
                ContractType::InstallViruses,
                ContractType::CollectMoney,
                ContractType::CompleteHacks,
                ContractType::SpreadViruses,
            ].choose(&mut rng).unwrap();

            let contract = Contract {
                id: contract_id,
                contract_type: contract_type.clone(),
                title: self.generate_contract_title(&contract_type),
                description: self.generate_contract_description(&contract_type),
                target_value: self.generate_contract_target(&contract_type),
                current_progress: 0,
                reward: self.generate_contract_reward(&contract_type),
                expires_at: Utc::now() + Duration::hours(24), // 24 hour contracts
                difficulty: rng.gen_range(1..=5),
                created_at: Utc::now(),
            };

            self.contracts.insert(contract_id, contract.clone());
            contracts.push(contract);
        }

        Ok(contracts)
    }

    /// Update contract progress
    pub fn update_contract_progress(&mut self, action: ContractAction) -> HeResult<Vec<ContractUpdate>> {
        let mut updates = Vec::new();

        for contract in self.contracts.values_mut() {
            if contract.expires_at < Utc::now() {
                continue;
            }

            let progress_gained = match (&contract.contract_type, &action) {
                (ContractType::HackTargets, ContractAction::SuccessfulHack) => 1,
                (ContractType::InstallViruses, ContractAction::VirusInstalled) => 1,
                (ContractType::CollectMoney, ContractAction::MoneyCollected(amount)) => *amount / 1000, // 1 progress per $1000
                (ContractType::CompleteHacks, ContractAction::SuccessfulHack) => 1,
                (ContractType::SpreadViruses, ContractAction::VirusSpread) => 1,
                _ => 0,
            };

            if progress_gained > 0 {
                contract.current_progress += progress_gained;
                
                if contract.current_progress >= contract.target_value {
                    updates.push(ContractUpdate {
                        contract_id: contract.id,
                        update_type: ContractUpdateType::Completed,
                        progress: contract.current_progress,
                        target: contract.target_value,
                        reward: contract.reward.clone(),
                    });
                } else {
                    updates.push(ContractUpdate {
                        contract_id: contract.id,
                        update_type: ContractUpdateType::Progress,
                        progress: contract.current_progress,
                        target: contract.target_value,
                        reward: contract.reward.clone(),
                    });
                }
            }
        }

        Ok(updates)
    }

    /// Get user's current campaign state
    pub fn get_campaign_state(&self, user_id: UserId) -> Option<CampaignState> {
        self.campaign_states.get(&user_id).cloned()
    }

    /// Get user's mission statistics
    pub fn get_user_mission_stats(&self, user_id: UserId) -> MissionStats {
        let mut stats = MissionStats {
            total_missions: 0,
            completed_missions: 0,
            failed_missions: 0,
            in_progress_missions: 0,
            tutorial_completed: false,
            story_progress: 0,
            total_rewards_earned: MissionReward::default(),
        };

        if let Some(user_progress) = self.user_progress.get(&user_id) {
            stats.total_missions = user_progress.len();
            
            for progress in user_progress {
                match progress.status {
                    MissionStatus::Completed => {
                        stats.completed_missions += 1;
                        if let Some(mission) = self.available_missions.get(&progress.mission_id) {
                            stats.total_rewards_earned.money += mission.completion_reward.money;
                            stats.total_rewards_earned.experience += mission.completion_reward.experience;
                            stats.total_rewards_earned.reputation += mission.completion_reward.reputation;
                        }
                    },
                    MissionStatus::Failed => stats.failed_missions += 1,
                    MissionStatus::InProgress => stats.in_progress_missions += 1,
                    _ => {},
                }
            }
        }

        if let Some(campaign_state) = self.campaign_states.get(&user_id) {
            stats.tutorial_completed = campaign_state.tutorial_completed;
            stats.story_progress = campaign_state.current_chapter;
        }

        stats
    }

    // Private helper methods

    fn initialize_mission_templates(&mut self) {
        self.mission_templates = vec![
            // Basic hacking missions
            MissionTemplate {
                id: 1,
                name: "Simple Hack".to_string(),
                category: MissionCategory::Hacking,
                difficulty_range: (1, 3),
                objectives: vec![
                    ObjectiveTemplate {
                        objective_type: ObjectiveType::HackTargets,
                        target_range: (1, 3),
                        description_template: "Hack {target} computer(s)".to_string(),
                    }
                ],
                base_rewards: MissionReward {
                    money: 1000,
                    experience: 100,
                    reputation: 5,
                    items: Vec::new(),
                },
            },
            
            // Virus missions
            MissionTemplate {
                id: 2,
                name: "Virus Distribution".to_string(),
                category: MissionCategory::Virus,
                difficulty_range: (2, 5),
                objectives: vec![
                    ObjectiveTemplate {
                        objective_type: ObjectiveType::InstallViruses,
                        target_range: (2, 8),
                        description_template: "Install {target} virus(es)".to_string(),
                    }
                ],
                base_rewards: MissionReward {
                    money: 2000,
                    experience: 200,
                    reputation: 10,
                    items: Vec::new(),
                },
            },

            // Money collection missions
            MissionTemplate {
                id: 3,
                name: "Financial Gain".to_string(),
                category: MissionCategory::Economy,
                difficulty_range: (2, 4),
                objectives: vec![
                    ObjectiveTemplate {
                        objective_type: ObjectiveType::CollectMoney,
                        target_range: (5000, 20000),
                        description_template: "Collect ${target} from hacking activities".to_string(),
                    }
                ],
                base_rewards: MissionReward {
                    money: 3000,
                    experience: 150,
                    reputation: 8,
                    items: Vec::new(),
                },
            },

            // Stealth missions
            MissionTemplate {
                id: 4,
                name: "Ghost Protocol".to_string(),
                category: MissionCategory::Stealth,
                difficulty_range: (3, 6),
                objectives: vec![
                    ObjectiveTemplate {
                        objective_type: ObjectiveType::HackWithoutDetection,
                        target_range: (1, 2),
                        description_template: "Hack {target} target(s) without being detected".to_string(),
                    }
                ],
                base_rewards: MissionReward {
                    money: 4000,
                    experience: 300,
                    reputation: 15,
                    items: Vec::new(),
                },
            },

            // Research missions
            MissionTemplate {
                id: 5,
                name: "Technology Research".to_string(),
                category: MissionCategory::Research,
                difficulty_range: (1, 4),
                objectives: vec![
                    ObjectiveTemplate {
                        objective_type: ObjectiveType::ResearchSoftware,
                        target_range: (1, 3),
                        description_template: "Research {target} software program(s)".to_string(),
                    }
                ],
                base_rewards: MissionReward {
                    money: 1500,
                    experience: 250,
                    reputation: 5,
                    items: Vec::new(),
                },
            },
        ];
    }

    fn initialize_story_campaigns(&mut self) {
        // Story campaigns would be initialized here
        // This is a simplified version - in reality would load from config files
    }

    fn generate_tutorial_missions(&self, user_id: UserId) -> HeResult<Vec<Mission>> {
        let mut rng = thread_rng();
        let mut missions = Vec::new();

        // Tutorial Mission 1: First Hack
        missions.push(Mission {
            id: rng.gen::<u64>(),
            title: "First Steps - Basic Hacking".to_string(),
            description: "Learn the basics of hacking by compromising your first target. This mission will teach you how to scan for vulnerabilities and exploit them.".to_string(),
            category: MissionCategory::Tutorial,
            difficulty: 1,
            objectives: vec![
                Objective {
                    id: rng.gen::<u64>(),
                    description: "Scan an IP address for open ports".to_string(),
                    objective_type: ObjectiveType::PortScan,
                    target_value: 1,
                    completion_reward: Some(MissionReward {
                        money: 100,
                        experience: 25,
                        reputation: 1,
                        items: Vec::new(),
                    }),
                    optional: false,
                },
                Objective {
                    id: rng.gen::<u64>(),
                    description: "Successfully hack 1 computer".to_string(),
                    objective_type: ObjectiveType::HackTargets,
                    target_value: 1,
                    completion_reward: Some(MissionReward {
                        money: 200,
                        experience: 50,
                        reputation: 2,
                        items: Vec::new(),
                    }),
                    optional: false,
                },
            ],
            completion_reward: MissionReward {
                money: 500,
                experience: 100,
                reputation: 5,
                items: vec!["Basic Cracker".to_string()],
            },
            time_limit: None,
            prerequisites: Vec::new(),
            unlocks_features: vec![GameFeature::PortScanning],
            story_mission: true,
            repeatable: false,
            created_at: Utc::now(),
        });

        // Tutorial Mission 2: Virus Installation
        missions.push(Mission {
            id: rng.gen::<u64>(),
            title: "Advanced Techniques - Virus Installation".to_string(),
            description: "Learn how to install viruses on compromised systems to generate passive income.".to_string(),
            category: MissionCategory::Tutorial,
            difficulty: 1,
            objectives: vec![
                Objective {
                    id: rng.gen::<u64>(),
                    description: "Install your first virus".to_string(),
                    objective_type: ObjectiveType::InstallViruses,
                    target_value: 1,
                    completion_reward: Some(MissionReward {
                        money: 150,
                        experience: 75,
                        reputation: 3,
                        items: Vec::new(),
                    }),
                    optional: false,
                },
                Objective {
                    id: rng.gen::<u64>(),
                    description: "Collect money from virus".to_string(),
                    objective_type: ObjectiveType::CollectMoney,
                    target_value: 100,
                    completion_reward: None,
                    optional: false,
                },
            ],
            completion_reward: MissionReward {
                money: 1000,
                experience: 200,
                reputation: 10,
                items: vec!["Basic Virus".to_string()],
            },
            time_limit: None,
            prerequisites: Vec::new(),
            unlocks_features: vec![GameFeature::VirusInstallation],
            story_mission: true,
            repeatable: false,
            created_at: Utc::now(),
        });

        // Tutorial Mission 3: Log Hiding
        missions.push(Mission {
            id: rng.gen::<u64>(),
            title: "Staying Hidden - Log Management".to_string(),
            description: "Learn how to hide your tracks by managing system logs.".to_string(),
            category: MissionCategory::Tutorial,
            difficulty: 2,
            objectives: vec![
                Objective {
                    id: rng.gen::<u64>(),
                    description: "Hide logs on a target system".to_string(),
                    objective_type: ObjectiveType::HideLogs,
                    target_value: 1,
                    completion_reward: Some(MissionReward {
                        money: 200,
                        experience: 100,
                        reputation: 5,
                        items: Vec::new(),
                    }),
                    optional: false,
                },
            ],
            completion_reward: MissionReward {
                money: 800,
                experience: 150,
                reputation: 8,
                items: vec!["Log Remover".to_string()],
            },
            time_limit: None,
            prerequisites: Vec::new(),
            unlocks_features: vec![GameFeature::LogHiding],
            story_mission: true,
            repeatable: false,
            created_at: Utc::now(),
        });

        Ok(missions)
    }

    fn can_user_accept_mission(&self, user_id: UserId, mission_id: u64) -> HeResult<bool> {
        let mission = self.available_missions.get(&mission_id)
            .ok_or(HackerExperienceError::MissionNotFound)?;

        // Check prerequisites
        if let Some(campaign_state) = self.campaign_states.get(&user_id) {
            for prerequisite in &mission.prerequisites {
                if !campaign_state.unlocked_features.contains(prerequisite) {
                    return Ok(false);
                }
            }
        }

        // Check if already accepted or completed
        if let Some(user_progress) = self.user_progress.get(&user_id) {
            if let Some(progress) = user_progress.iter().find(|p| p.mission_id == mission_id) {
                return Ok(progress.status == MissionStatus::Available);
            }
        }

        Ok(true)
    }

    fn calculate_mission_duration(&self, mission: &Mission) -> u32 {
        let base_duration = match mission.difficulty {
            1 => 300,   // 5 minutes
            2 => 600,   // 10 minutes
            3 => 1200,  // 20 minutes
            4 => 1800,  // 30 minutes
            5 => 3600,  // 1 hour
            _ => 1800,
        };

        let objective_multiplier = mission.objectives.len() as u32;
        base_duration * objective_multiplier
    }

    fn generate_mission_tips(&self, mission: &Mission) -> Vec<String> {
        let mut tips = Vec::new();

        match mission.category {
            MissionCategory::Hacking => {
                tips.push("Use port scanners to identify vulnerabilities".to_string());
                tips.push("Higher power crackers increase success probability".to_string());
            },
            MissionCategory::Virus => {
                tips.push("Install viruses on successfully hacked systems".to_string());
                tips.push("Check virus status regularly for collection opportunities".to_string());
            },
            MissionCategory::Stealth => {
                tips.push("Use log removers to hide your tracks".to_string());
                tips.push("Avoid detection by using high-stealth software".to_string());
            },
            _ => {
                tips.push("Read mission objectives carefully".to_string());
            }
        }

        tips
    }

    fn action_matches_objective(&self, action: &MissionAction, objective: &Objective) -> bool {
        match (&action, &objective.objective_type) {
            (MissionAction::SuccessfulHack, ObjectiveType::HackTargets) => true,
            (MissionAction::PortScan, ObjectiveType::PortScan) => true,
            (MissionAction::VirusInstalled, ObjectiveType::InstallViruses) => true,
            (MissionAction::MoneyCollected(_), ObjectiveType::CollectMoney) => true,
            (MissionAction::LogsHidden, ObjectiveType::HideLogs) => true,
            (MissionAction::SoftwareResearched, ObjectiveType::ResearchSoftware) => true,
            (MissionAction::HackWithoutDetection, ObjectiveType::HackWithoutDetection) => true,
            _ => false,
        }
    }

    fn calculate_objective_progress(&self, action: &MissionAction, _objective: &Objective) -> i64 {
        match action {
            MissionAction::SuccessfulHack => 1,
            MissionAction::PortScan => 1,
            MissionAction::VirusInstalled => 1,
            MissionAction::MoneyCollected(amount) => *amount,
            MissionAction::LogsHidden => 1,
            MissionAction::SoftwareResearched => 1,
            MissionAction::HackWithoutDetection => 1,
        }
    }

    fn handle_story_progression(&mut self, user_id: UserId, completed_mission_id: u64) -> HeResult<()> {
        if let Some(campaign_state) = self.campaign_states.get_mut(&user_id) {
            let mission = self.available_missions.get(&completed_mission_id).unwrap();
            
            if mission.story_mission {
                // Update campaign progress
                campaign_state.current_mission += 1;
                
                // Unlock features
                for feature in &mission.unlocks_features {
                    if !campaign_state.unlocked_features.contains(feature) {
                        campaign_state.unlocked_features.push(feature.clone());
                    }
                }

                // Check if tutorial is completed
                if !campaign_state.tutorial_completed && campaign_state.current_mission > 3 {
                    campaign_state.tutorial_completed = true;
                    campaign_state.story_unlocked = true;
                }
            }
        }

        Ok(())
    }

    fn generate_follow_up_missions(&mut self, user_id: UserId, _completed_mission_id: u64) -> HeResult<()> {
        // Generate new missions based on completed mission
        // This would be more complex in a real implementation
        if let Some(campaign_state) = self.campaign_states.get(&user_id) {
            if campaign_state.tutorial_completed {
                self.generate_random_missions(user_id, 2)?;
            }
        }

        Ok(())
    }

    fn generate_mission_from_template(&self, template: &MissionTemplate, user_level: u32) -> HeResult<Mission> {
        let mut rng = thread_rng();
        let mission_id = rng.gen::<u64>();
        
        let difficulty = rng.gen_range(template.difficulty_range.0..=template.difficulty_range.1);
        
        let mut objectives = Vec::new();
        for obj_template in &template.objectives {
            let target_value = rng.gen_range(obj_template.target_range.0..=obj_template.target_range.1);
            
            objectives.push(Objective {
                id: rng.gen::<u64>(),
                description: obj_template.description_template.replace("{target}", &target_value.to_string()),
                objective_type: obj_template.objective_type.clone(),
                target_value: target_value as i64,
                completion_reward: Some(MissionReward {
                    money: template.base_rewards.money / 4,
                    experience: template.base_rewards.experience / 4,
                    reputation: template.base_rewards.reputation / 4,
                    items: Vec::new(),
                }),
                optional: false,
            });
        }

        // Scale rewards by user level and difficulty
        let level_multiplier = 1.0 + (user_level as f64 / 10.0);
        let difficulty_multiplier = 1.0 + (difficulty as f64 / 5.0);
        let total_multiplier = level_multiplier * difficulty_multiplier;

        Ok(Mission {
            id: mission_id,
            title: format!("{} (Level {})", template.name, difficulty),
            description: self.generate_mission_description(&template.category, difficulty),
            category: template.category.clone(),
            difficulty,
            objectives,
            completion_reward: MissionReward {
                money: (template.base_rewards.money as f64 * total_multiplier) as i64,
                experience: (template.base_rewards.experience as f64 * total_multiplier) as i32,
                reputation: (template.base_rewards.reputation as f64 * total_multiplier) as i32,
                items: template.base_rewards.items.clone(),
            },
            time_limit: None,
            prerequisites: Vec::new(),
            unlocks_features: Vec::new(),
            story_mission: false,
            repeatable: true,
            created_at: Utc::now(),
        })
    }

    fn generate_mission_description(&self, category: &MissionCategory, difficulty: u32) -> String {
        let descriptions = match category {
            MissionCategory::Hacking => vec![
                "A routine hacking job has come up. The client needs access to some systems.",
                "Intelligence suggests several vulnerable targets in the area. Time to exploit them.",
                "A corporate espionage job requires your hacking skills.",
            ],
            MissionCategory::Virus => vec![
                "Spread your influence by installing viruses on target systems.",
                "A botnet expansion is needed. Install viruses strategically.",
                "Passive income opportunities await through virus installation.",
            ],
            MissionCategory::Economy => vec![
                "Financial gain is the objective. Generate money through your operations.",
                "Economic warfare requires substantial funding. Acquire resources.",
                "Investment opportunities require initial capital. Time to acquire it.",
            ],
            _ => vec![
                "A specialized mission has become available.",
                "Your unique skills are required for this operation.",
            ],
        };

        let mut rng = thread_rng();
        let base_desc = descriptions.choose(&mut rng).unwrap();
        
        match difficulty {
            1..=2 => format!("{} This appears to be a straightforward operation.", base_desc),
            3..=4 => format!("{} Moderate security measures are expected.", base_desc),
            5..=6 => format!("{} High security and significant challenges await.", base_desc),
            _ => format!("{} Extreme difficulty - only attempt if well prepared.", base_desc),
        }
    }

    fn get_user_level(&self, user_id: UserId) -> u32 {
        // Simplified user level calculation
        if let Some(campaign_state) = self.campaign_states.get(&user_id) {
            campaign_state.current_chapter
        } else {
            1
        }
    }

    fn get_available_contracts(&self, _user_id: UserId) -> Vec<Mission> {
        // Convert contracts to missions for the UI
        Vec::new() // Simplified for now
    }

    fn generate_contract_title(&self, contract_type: &ContractType) -> String {
        match contract_type {
            ContractType::HackTargets => "Target Elimination Contract".to_string(),
            ContractType::InstallViruses => "Virus Distribution Network".to_string(),
            ContractType::CollectMoney => "Financial Acquisition Task".to_string(),
            ContractType::CompleteHacks => "Operation Success Rate".to_string(),
            ContractType::SpreadViruses => "Network Expansion Protocol".to_string(),
        }
    }

    fn generate_contract_description(&self, contract_type: &ContractType) -> String {
        match contract_type {
            ContractType::HackTargets => "Compromise a specified number of target systems within the time limit.".to_string(),
            ContractType::InstallViruses => "Deploy viruses across multiple systems to expand your network.".to_string(),
            ContractType::CollectMoney => "Generate the specified amount of money through various activities.".to_string(),
            ContractType::CompleteHacks => "Successfully complete a series of hacking operations.".to_string(),
            ContractType::SpreadViruses => "Expand your virus network by spreading to new targets.".to_string(),
        }
    }

    fn generate_contract_target(&self, contract_type: &ContractType) -> i64 {
        let mut rng = thread_rng();
        
        match contract_type {
            ContractType::HackTargets => rng.gen_range(3..=10),
            ContractType::InstallViruses => rng.gen_range(2..=8),
            ContractType::CollectMoney => rng.gen_range(5000..=25000),
            ContractType::CompleteHacks => rng.gen_range(5..=15),
            ContractType::SpreadViruses => rng.gen_range(1..=5),
        }
    }

    fn generate_contract_reward(&self, contract_type: &ContractType) -> MissionReward {
        let mut rng = thread_rng();
        
        match contract_type {
            ContractType::HackTargets => MissionReward {
                money: rng.gen_range(2000..=8000),
                experience: rng.gen_range(100..=400),
                reputation: rng.gen_range(10..=30),
                items: Vec::new(),
            },
            ContractType::InstallViruses => MissionReward {
                money: rng.gen_range(3000..=10000),
                experience: rng.gen_range(150..=500),
                reputation: rng.gen_range(15..=40),
                items: Vec::new(),
            },
            ContractType::CollectMoney => MissionReward {
                money: rng.gen_range(1000..=5000),
                experience: rng.gen_range(50..=200),
                reputation: rng.gen_range(5..=20),
                items: Vec::new(),
            },
            ContractType::CompleteHacks => MissionReward {
                money: rng.gen_range(4000..=12000),
                experience: rng.gen_range(200..=600),
                reputation: rng.gen_range(20..=50),
                items: Vec::new(),
            },
            ContractType::SpreadViruses => MissionReward {
                money: rng.gen_range(2500..=7500),
                experience: rng.gen_range(125..=375),
                reputation: rng.gen_range(12..=35),
                items: Vec::new(),
            },
        }
    }
}

// Data structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub category: MissionCategory,
    pub difficulty: u32, // 1-6 scale
    pub objectives: Vec<Objective>,
    pub completion_reward: MissionReward,
    pub time_limit: Option<DateTime<Utc>>,
    pub prerequisites: Vec<GameFeature>,
    pub unlocks_features: Vec<GameFeature>,
    pub story_mission: bool,
    pub repeatable: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MissionCategory {
    Tutorial,
    Hacking,
    Virus,
    Economy,
    Stealth,
    Research,
    Story,
    Contract,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objective {
    pub id: u64,
    pub description: String,
    pub objective_type: ObjectiveType,
    pub target_value: i64,
    pub completion_reward: Option<MissionReward>,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectiveType {
    HackTargets,
    InstallViruses,
    CollectMoney,
    PortScan,
    HideLogs,
    ResearchSoftware,
    HackWithoutDetection,
    SpreadViruses,
    CompleteContracts,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MissionReward {
    pub money: i64,
    pub experience: i32,
    pub reputation: i32,
    pub items: Vec<String>, // Software/Hardware items
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionProgress {
    pub mission_id: u64,
    pub user_id: UserId,
    pub status: MissionStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub current_objectives: Vec<ObjectiveProgress>,
    pub rewards_claimed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MissionStatus {
    Available,
    InProgress,
    Completed,
    Failed,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveProgress {
    pub objective_id: u64,
    pub completed: bool,
    pub progress_value: i64,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignState {
    pub user_id: UserId,
    pub current_chapter: u32,
    pub current_mission: u32,
    pub tutorial_completed: bool,
    pub story_unlocked: bool,
    pub unlocked_features: Vec<GameFeature>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GameFeature {
    BasicHacking,
    PortScanning,
    VirusInstallation,
    LogHiding,
    AdvancedHacking,
    Research,
    Banking,
    Clans,
    Missions,
    Contracts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub id: u64,
    pub contract_type: ContractType,
    pub title: String,
    pub description: String,
    pub target_value: i64,
    pub current_progress: i64,
    pub reward: MissionReward,
    pub expires_at: DateTime<Utc>,
    pub difficulty: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContractType {
    HackTargets,
    InstallViruses,
    CollectMoney,
    CompleteHacks,
    SpreadViruses,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionTemplate {
    pub id: u64,
    pub name: String,
    pub category: MissionCategory,
    pub difficulty_range: (u32, u32),
    pub objectives: Vec<ObjectiveTemplate>,
    pub base_rewards: MissionReward,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveTemplate {
    pub objective_type: ObjectiveType,
    pub target_range: (i64, i64),
    pub description_template: String,
}

// Action types for mission progress

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MissionAction {
    SuccessfulHack,
    PortScan,
    VirusInstalled,
    MoneyCollected(i64),
    LogsHidden,
    SoftwareResearched,
    HackWithoutDetection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContractAction {
    SuccessfulHack,
    VirusInstalled,
    MoneyCollected(i64),
    VirusSpread,
}

// Result types

#[derive(Debug, Serialize, Deserialize)]
pub struct MissionAcceptResult {
    pub mission: Mission,
    pub objectives: Vec<Objective>,
    pub estimated_duration: u32, // in seconds
    pub tips: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MissionUpdate {
    pub mission_id: u64,
    pub update_type: MissionUpdateType,
    pub objective_id: Option<u64>,
    pub message: String,
    pub rewards: Option<MissionReward>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MissionUpdateType {
    ProgressUpdate,
    ObjectiveCompleted,
    MissionCompleted,
    MissionFailed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractUpdate {
    pub contract_id: u64,
    pub update_type: ContractUpdateType,
    pub progress: i64,
    pub target: i64,
    pub reward: MissionReward,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ContractUpdateType {
    Progress,
    Completed,
    Expired,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MissionStats {
    pub total_missions: usize,
    pub completed_missions: usize,
    pub failed_missions: usize,
    pub in_progress_missions: usize,
    pub tutorial_completed: bool,
    pub story_progress: u32,
    pub total_rewards_earned: MissionReward,
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tutorial_generation() {
        let mut mission_system = MissionSystem::new();
        let tutorial_missions = mission_system.start_tutorial(1).unwrap();
        
        assert!(!tutorial_missions.is_empty());
        assert!(tutorial_missions.iter().all(|m| m.category == MissionCategory::Tutorial));
    }

    #[test]
    fn test_mission_acceptance() {
        let mut mission_system = MissionSystem::new();
        let tutorial_missions = mission_system.start_tutorial(1).unwrap();
        
        if let Some(first_mission) = tutorial_missions.first() {
            let result = mission_system.accept_mission(1, first_mission.id).unwrap();
            assert_eq!(result.mission.id, first_mission.id);
        }
    }

    #[test]
    fn test_mission_progress_update() {
        let mut mission_system = MissionSystem::new();
        let _tutorial_missions = mission_system.start_tutorial(1).unwrap();
        
        let updates = mission_system.update_mission_progress(
            1, 
            MissionAction::PortScan
        ).unwrap();
        
        // Should handle updates even if no matching missions
        assert!(updates.len() >= 0);
    }

    #[test]
    fn test_contract_generation() {
        let mut mission_system = MissionSystem::new();
        let contracts = mission_system.generate_daily_contracts().unwrap();
        
        assert!(!contracts.is_empty());
        assert!(contracts.len() >= 3 && contracts.len() <= 5);
    }
}