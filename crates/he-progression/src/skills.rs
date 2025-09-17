//! Skill Tree System - Player abilities and specializations

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Main skill tree containing all skill branches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTree {
    pub skill_points_available: u32,
    pub skill_points_spent: u32,
    pub unlocked_skills: HashSet<String>,
    pub skill_levels: HashMap<String, u32>,
    pub branches: SkillBranches,
}

/// Different skill branches for specialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillBranches {
    pub hacking: HackingSkills,
    pub defense: DefenseSkills,
    pub stealth: StealthSkills,
    pub hardware: HardwareSkills,
    pub software: SoftwareSkills,
    pub networking: NetworkingSkills,
}

/// Hacking skill branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HackingSkills {
    pub password_cracking: SkillNode,
    pub exploit_development: SkillNode,
    pub sql_injection: SkillNode,
    pub buffer_overflow: SkillNode,
    pub social_engineering: SkillNode,
    pub cryptanalysis: SkillNode,
}

/// Defense skill branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefenseSkills {
    pub firewall_mastery: SkillNode,
    pub intrusion_detection: SkillNode,
    pub log_analysis: SkillNode,
    pub honeypot_deployment: SkillNode,
    pub encryption_protocols: SkillNode,
    pub backup_systems: SkillNode,
}

/// Stealth skill branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StealthSkills {
    pub log_deletion: SkillNode,
    pub proxy_chains: SkillNode,
    pub vpn_mastery: SkillNode,
    pub trace_evasion: SkillNode,
    pub identity_spoofing: SkillNode,
    pub ghost_mode: SkillNode,
}

/// Hardware skill branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSkills {
    pub cpu_overclocking: SkillNode,
    pub ram_optimization: SkillNode,
    pub storage_compression: SkillNode,
    pub network_bandwidth: SkillNode,
    pub cooling_systems: SkillNode,
    pub quantum_processing: SkillNode,
}

/// Software skill branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareSkills {
    pub virus_development: SkillNode,
    pub worm_creation: SkillNode,
    pub trojan_engineering: SkillNode,
    pub rootkit_mastery: SkillNode,
    pub ai_assistants: SkillNode,
    pub automated_scripts: SkillNode,
}

/// Networking skill branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkingSkills {
    pub packet_sniffing: SkillNode,
    pub port_scanning: SkillNode,
    pub ddos_amplification: SkillNode,
    pub mesh_networking: SkillNode,
    pub satellite_hacking: SkillNode,
    pub quantum_tunneling: SkillNode,
}

/// Individual skill node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillNode {
    pub id: String,
    pub name: String,
    pub description: String,
    pub max_level: u32,
    pub current_level: u32,
    pub cost_per_level: Vec<u32>,
    pub requirements: SkillRequirements,
    pub effects: Vec<SkillEffect>,
}

/// Requirements to unlock/upgrade a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRequirements {
    pub player_level: u32,
    pub prerequisite_skills: Vec<(String, u32)>, // (skill_id, required_level)
    pub money_cost: Option<i64>,
    pub item_requirements: Vec<String>,
}

/// Effects of having a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillEffect {
    PercentageBoost {
        stat: String,
        value: f32,
    },
    FlatBoost {
        stat: String,
        value: i32,
    },
    UnlockFeature {
        feature: String,
    },
    ReduceCost {
        resource: String,
        percentage: f32,
    },
    IncreaseSpeed {
        process_type: String,
        percentage: f32,
    },
    AddAbility {
        ability: String,
        description: String,
    },
}

impl SkillTree {
    /// Create a new skill tree
    pub fn new() -> Self {
        Self {
            skill_points_available: 0,
            skill_points_spent: 0,
            unlocked_skills: HashSet::new(),
            skill_levels: HashMap::new(),
            branches: SkillBranches::new(),
        }
    }

    /// Add skill points
    pub fn add_skill_points(&mut self, points: u32) {
        self.skill_points_available += points;
    }

    /// Invest points in a skill
    pub fn invest_skill(&mut self, skill_id: &str, points: u32) -> Result<(), SkillError> {
        if points > self.skill_points_available {
            return Err(SkillError::InsufficientPoints);
        }

        let skill = self.get_skill_mut(skill_id)?;

        if skill.current_level + points > skill.max_level {
            return Err(SkillError::MaxLevelReached);
        }

        // Check requirements
        if !self.check_requirements(&skill.requirements) {
            return Err(SkillError::RequirementsNotMet);
        }

        // Calculate cost
        let mut total_cost = 0;
        for i in 0..points {
            let level_idx = (skill.current_level + i) as usize;
            if level_idx >= skill.cost_per_level.len() {
                total_cost += skill.cost_per_level.last().unwrap_or(&1);
            } else {
                total_cost += skill.cost_per_level[level_idx];
            }
        }

        if total_cost > self.skill_points_available {
            return Err(SkillError::InsufficientPoints);
        }

        // Apply the investment
        skill.current_level += points;
        self.skill_points_available -= total_cost;
        self.skill_points_spent += total_cost;
        self.unlocked_skills.insert(skill_id.to_string());
        self.skill_levels.insert(skill_id.to_string(), skill.current_level);

        Ok(())
    }

    /// Get a skill by ID
    fn get_skill_mut(&mut self, skill_id: &str) -> Result<&mut SkillNode, SkillError> {
        // This is simplified - in real implementation, search all branches
        match skill_id {
            "password_cracking" => Ok(&mut self.branches.hacking.password_cracking),
            "firewall_mastery" => Ok(&mut self.branches.defense.firewall_mastery),
            "log_deletion" => Ok(&mut self.branches.stealth.log_deletion),
            "cpu_overclocking" => Ok(&mut self.branches.hardware.cpu_overclocking),
            "virus_development" => Ok(&mut self.branches.software.virus_development),
            "packet_sniffing" => Ok(&mut self.branches.networking.packet_sniffing),
            _ => Err(SkillError::SkillNotFound),
        }
    }

    /// Check if requirements are met
    fn check_requirements(&self, reqs: &SkillRequirements) -> bool {
        // Check prerequisite skills
        for (skill_id, required_level) in &reqs.prerequisite_skills {
            if let Some(&current_level) = self.skill_levels.get(skill_id) {
                if current_level < *required_level {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    /// Get total invested points
    pub fn get_total_invested_points(&self) -> u32 {
        self.skill_points_spent
    }

    /// Calculate total skill bonuses
    pub fn calculate_bonuses(&self) -> HashMap<String, f32> {
        let mut bonuses = HashMap::new();

        // Iterate through all skills and accumulate bonuses
        // This is simplified - real implementation would check all branches
        for skill_id in &self.unlocked_skills {
            if let Some(level) = self.skill_levels.get(skill_id) {
                // Apply skill effects based on level
                // Simplified example
                match skill_id.as_str() {
                    "password_cracking" => {
                        *bonuses.entry("crack_speed".to_string()).or_insert(0.0) += 5.0 * *level as f32;
                    }
                    "firewall_mastery" => {
                        *bonuses.entry("defense_rating".to_string()).or_insert(0.0) += 10.0 * *level as f32;
                    }
                    _ => {}
                }
            }
        }

        bonuses
    }

    /// Reset all skills (requires special item or payment)
    pub fn reset_skills(&mut self) {
        let total_points = self.skill_points_spent + self.skill_points_available;
        self.skill_points_available = total_points;
        self.skill_points_spent = 0;
        self.unlocked_skills.clear();
        self.skill_levels.clear();
        self.branches = SkillBranches::new();
    }
}

impl SkillBranches {
    fn new() -> Self {
        Self {
            hacking: HackingSkills::new(),
            defense: DefenseSkills::new(),
            stealth: StealthSkills::new(),
            hardware: HardwareSkills::new(),
            software: SoftwareSkills::new(),
            networking: NetworkingSkills::new(),
        }
    }
}

impl HackingSkills {
    fn new() -> Self {
        Self {
            password_cracking: SkillNode {
                id: "password_cracking".to_string(),
                name: "Password Cracking".to_string(),
                description: "Increases password cracking speed".to_string(),
                max_level: 10,
                current_level: 0,
                cost_per_level: vec![1, 1, 2, 2, 3, 3, 4, 4, 5, 5],
                requirements: SkillRequirements {
                    player_level: 1,
                    prerequisite_skills: vec![],
                    money_cost: None,
                    item_requirements: vec![],
                },
                effects: vec![
                    SkillEffect::IncreaseSpeed {
                        process_type: "crack".to_string(),
                        percentage: 5.0,
                    }
                ],
            },
            exploit_development: SkillNode {
                id: "exploit_development".to_string(),
                name: "Exploit Development".to_string(),
                description: "Create custom exploits for specific targets".to_string(),
                max_level: 5,
                current_level: 0,
                cost_per_level: vec![2, 3, 4, 5, 6],
                requirements: SkillRequirements {
                    player_level: 10,
                    prerequisite_skills: vec![("password_cracking".to_string(), 5)],
                    money_cost: Some(10000),
                    item_requirements: vec![],
                },
                effects: vec![
                    SkillEffect::UnlockFeature {
                        feature: "custom_exploits".to_string(),
                    }
                ],
            },
            sql_injection: SkillNode::default(),
            buffer_overflow: SkillNode::default(),
            social_engineering: SkillNode::default(),
            cryptanalysis: SkillNode::default(),
        }
    }
}

// Implement similar new() methods for other skill branches
impl DefenseSkills {
    fn new() -> Self {
        Self {
            firewall_mastery: SkillNode {
                id: "firewall_mastery".to_string(),
                name: "Firewall Mastery".to_string(),
                description: "Improves firewall effectiveness".to_string(),
                max_level: 10,
                current_level: 0,
                cost_per_level: vec![1; 10],
                requirements: SkillRequirements::default(),
                effects: vec![
                    SkillEffect::PercentageBoost {
                        stat: "firewall_strength".to_string(),
                        value: 10.0,
                    }
                ],
            },
            intrusion_detection: SkillNode::default(),
            log_analysis: SkillNode::default(),
            honeypot_deployment: SkillNode::default(),
            encryption_protocols: SkillNode::default(),
            backup_systems: SkillNode::default(),
        }
    }
}

// Default implementations for other branches
impl StealthSkills {
    fn new() -> Self {
        Self {
            log_deletion: SkillNode::default(),
            proxy_chains: SkillNode::default(),
            vpn_mastery: SkillNode::default(),
            trace_evasion: SkillNode::default(),
            identity_spoofing: SkillNode::default(),
            ghost_mode: SkillNode::default(),
        }
    }
}

impl HardwareSkills {
    fn new() -> Self {
        Self {
            cpu_overclocking: SkillNode::default(),
            ram_optimization: SkillNode::default(),
            storage_compression: SkillNode::default(),
            network_bandwidth: SkillNode::default(),
            cooling_systems: SkillNode::default(),
            quantum_processing: SkillNode::default(),
        }
    }
}

impl SoftwareSkills {
    fn new() -> Self {
        Self {
            virus_development: SkillNode::default(),
            worm_creation: SkillNode::default(),
            trojan_engineering: SkillNode::default(),
            rootkit_mastery: SkillNode::default(),
            ai_assistants: SkillNode::default(),
            automated_scripts: SkillNode::default(),
        }
    }
}

impl NetworkingSkills {
    fn new() -> Self {
        Self {
            packet_sniffing: SkillNode::default(),
            port_scanning: SkillNode::default(),
            ddos_amplification: SkillNode::default(),
            mesh_networking: SkillNode::default(),
            satellite_hacking: SkillNode::default(),
            quantum_tunneling: SkillNode::default(),
        }
    }
}

impl Default for SkillNode {
    fn default() -> Self {
        Self {
            id: "".to_string(),
            name: "Unnamed Skill".to_string(),
            description: "No description".to_string(),
            max_level: 1,
            current_level: 0,
            cost_per_level: vec![1],
            requirements: SkillRequirements::default(),
            effects: vec![],
        }
    }
}

impl Default for SkillRequirements {
    fn default() -> Self {
        Self {
            player_level: 1,
            prerequisite_skills: vec![],
            money_cost: None,
            item_requirements: vec![],
        }
    }
}

/// Errors that can occur in skill operations
#[derive(Debug, thiserror::Error)]
pub enum SkillError {
    #[error("Insufficient skill points")]
    InsufficientPoints,
    #[error("Skill not found")]
    SkillNotFound,
    #[error("Max level reached")]
    MaxLevelReached,
    #[error("Requirements not met")]
    RequirementsNotMet,
}