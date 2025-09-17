//! Unlockable Content System - Progressive content unlocking

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Manages all unlockable content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockableContent {
    pub unlocked_items: HashSet<String>,
    pub locked_items: HashMap<String, UnlockRequirement>,
    pub categories: ContentCategories,
}

/// Categories of unlockable content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentCategories {
    pub software: Vec<UnlockedContent>,
    pub hardware: Vec<UnlockedContent>,
    pub servers: Vec<UnlockedContent>,
    pub missions: Vec<UnlockedContent>,
    pub areas: Vec<UnlockedContent>,
    pub features: Vec<UnlockedContent>,
    pub cosmetics: Vec<UnlockedContent>,
}

/// Individual unlockable content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockedContent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: ContentCategory,
    pub requirement: UnlockRequirement,
    pub preview_available: bool,
}

/// Content categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentCategory {
    Software,
    Hardware,
    Server,
    Mission,
    Area,
    Feature,
    Cosmetic,
}

/// Requirements to unlock content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnlockRequirement {
    Level(u32),
    Achievement(String),
    Mission(String),
    Reputation { faction: String, level: i32 },
    Item(String),
    Purchase { cost: i64 },
    MultiRequirement(Vec<UnlockRequirement>),
    Event(String),
    Premium,
}

impl UnlockableContent {
    /// Create new unlockable content system
    pub fn new() -> Self {
        let mut content = Self {
            unlocked_items: HashSet::new(),
            locked_items: HashMap::new(),
            categories: ContentCategories::default(),
        };

        // Initialize locked items
        content.initialize_locked_content();
        content
    }

    /// Initialize all locked content
    fn initialize_locked_content(&mut self) {
        // Add all content to locked items
        for item in &self.categories.software {
            self.locked_items.insert(item.id.clone(), item.requirement.clone());
        }
        for item in &self.categories.hardware {
            self.locked_items.insert(item.id.clone(), item.requirement.clone());
        }
        for item in &self.categories.servers {
            self.locked_items.insert(item.id.clone(), item.requirement.clone());
        }
        for item in &self.categories.missions {
            self.locked_items.insert(item.id.clone(), item.requirement.clone());
        }
        for item in &self.categories.areas {
            self.locked_items.insert(item.id.clone(), item.requirement.clone());
        }
        for item in &self.categories.features {
            self.locked_items.insert(item.id.clone(), item.requirement.clone());
        }
        for item in &self.categories.cosmetics {
            self.locked_items.insert(item.id.clone(), item.requirement.clone());
        }
    }

    /// Check for level-based unlocks
    pub fn check_level_unlocks(&mut self, level: u32) -> Vec<UnlockedContent> {
        let mut newly_unlocked = Vec::new();

        for (id, requirement) in self.locked_items.clone().iter() {
            if let UnlockRequirement::Level(required_level) = requirement {
                if level >= *required_level && !self.unlocked_items.contains(id) {
                    self.unlock_item(id);
                    if let Some(content) = self.find_content(id) {
                        newly_unlocked.push(content);
                    }
                }
            }
        }

        newly_unlocked
    }

    /// Unlock an item
    pub fn unlock_item(&mut self, item_id: &str) {
        self.unlocked_items.insert(item_id.to_string());
        self.locked_items.remove(item_id);
    }

    /// Check if an item is unlocked
    pub fn is_unlocked(&self, item_id: &str) -> bool {
        self.unlocked_items.contains(item_id)
    }

    /// Find content by ID
    fn find_content(&self, id: &str) -> Option<UnlockedContent> {
        // Search all categories
        self.categories.software.iter()
            .chain(self.categories.hardware.iter())
            .chain(self.categories.servers.iter())
            .chain(self.categories.missions.iter())
            .chain(self.categories.areas.iter())
            .chain(self.categories.features.iter())
            .chain(self.categories.cosmetics.iter())
            .find(|c| c.id == id)
            .cloned()
    }

    /// Get progress statistics
    pub fn get_unlock_progress(&self) -> UnlockProgress {
        let total_items = self.unlocked_items.len() + self.locked_items.len();
        let unlocked = self.unlocked_items.len();

        UnlockProgress {
            total_items,
            unlocked_items: unlocked,
            locked_items: self.locked_items.len(),
            percentage: if total_items > 0 {
                (unlocked as f32 / total_items as f32) * 100.0
            } else {
                0.0
            },
        }
    }
}

/// Progress statistics for unlockables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockProgress {
    pub total_items: usize,
    pub unlocked_items: usize,
    pub locked_items: usize,
    pub percentage: f32,
}

impl Default for ContentCategories {
    fn default() -> Self {
        Self {
            software: vec![
                UnlockedContent {
                    id: "advanced_cracker_v2".to_string(),
                    name: "Advanced Cracker v2.0".to_string(),
                    description: "50% faster password cracking".to_string(),
                    category: ContentCategory::Software,
                    requirement: UnlockRequirement::Level(10),
                    preview_available: true,
                },
                UnlockedContent {
                    id: "quantum_exploit".to_string(),
                    name: "Quantum Exploit Kit".to_string(),
                    description: "Break quantum encryption".to_string(),
                    category: ContentCategory::Software,
                    requirement: UnlockRequirement::Level(50),
                    preview_available: false,
                },
                UnlockedContent {
                    id: "ai_virus".to_string(),
                    name: "AI Virus Generator".to_string(),
                    description: "Self-evolving virus creation".to_string(),
                    category: ContentCategory::Software,
                    requirement: UnlockRequirement::Achievement("virus_master".to_string()),
                    preview_available: false,
                },
            ],
            hardware: vec![
                UnlockedContent {
                    id: "quantum_cpu".to_string(),
                    name: "Quantum CPU".to_string(),
                    description: "10x processing power".to_string(),
                    category: ContentCategory::Hardware,
                    requirement: UnlockRequirement::Level(75),
                    preview_available: true,
                },
                UnlockedContent {
                    id: "neural_interface".to_string(),
                    name: "Neural Interface".to_string(),
                    description: "Direct brain-computer connection".to_string(),
                    category: ContentCategory::Hardware,
                    requirement: UnlockRequirement::MultiRequirement(vec![
                        UnlockRequirement::Level(100),
                        UnlockRequirement::Achievement("cyborg".to_string()),
                    ]),
                    preview_available: false,
                },
            ],
            servers: vec![
                UnlockedContent {
                    id: "bank_mainframe".to_string(),
                    name: "Bank Mainframe Access".to_string(),
                    description: "Access to high-value banking servers".to_string(),
                    category: ContentCategory::Server,
                    requirement: UnlockRequirement::Level(25),
                    preview_available: true,
                },
                UnlockedContent {
                    id: "government_network".to_string(),
                    name: "Government Network".to_string(),
                    description: "Access classified government servers".to_string(),
                    category: ContentCategory::Server,
                    requirement: UnlockRequirement::Reputation {
                        faction: "Underground".to_string(),
                        level: 50,
                    },
                    preview_available: false,
                },
            ],
            missions: vec![
                UnlockedContent {
                    id: "corporate_espionage".to_string(),
                    name: "Corporate Espionage Chain".to_string(),
                    description: "High-stakes corporate infiltration missions".to_string(),
                    category: ContentCategory::Mission,
                    requirement: UnlockRequirement::Level(20),
                    preview_available: true,
                },
                UnlockedContent {
                    id: "zero_day_crisis".to_string(),
                    name: "Zero Day Crisis".to_string(),
                    description: "Stop a global cyber attack".to_string(),
                    category: ContentCategory::Mission,
                    requirement: UnlockRequirement::Mission("corporate_espionage_complete".to_string()),
                    preview_available: false,
                },
            ],
            areas: vec![
                UnlockedContent {
                    id: "dark_web".to_string(),
                    name: "Dark Web Access".to_string(),
                    description: "Enter the hidden internet".to_string(),
                    category: ContentCategory::Area,
                    requirement: UnlockRequirement::Level(15),
                    preview_available: true,
                },
                UnlockedContent {
                    id: "quantum_realm".to_string(),
                    name: "Quantum Network".to_string(),
                    description: "Access quantum computing network".to_string(),
                    category: ContentCategory::Area,
                    requirement: UnlockRequirement::Item("quantum_key".to_string()),
                    preview_available: false,
                },
            ],
            features: vec![
                UnlockedContent {
                    id: "automation".to_string(),
                    name: "Process Automation".to_string(),
                    description: "Automate repetitive tasks".to_string(),
                    category: ContentCategory::Feature,
                    requirement: UnlockRequirement::Level(30),
                    preview_available: true,
                },
                UnlockedContent {
                    id: "clan_creation".to_string(),
                    name: "Clan Creation".to_string(),
                    description: "Create and lead your own clan".to_string(),
                    category: ContentCategory::Feature,
                    requirement: UnlockRequirement::Level(40),
                    preview_available: true,
                },
                UnlockedContent {
                    id: "market_trading".to_string(),
                    name: "Black Market Trading".to_string(),
                    description: "Buy and sell on the black market".to_string(),
                    category: ContentCategory::Feature,
                    requirement: UnlockRequirement::Reputation {
                        faction: "Black Market".to_string(),
                        level: 25,
                    },
                    preview_available: true,
                },
            ],
            cosmetics: vec![
                UnlockedContent {
                    id: "neon_terminal".to_string(),
                    name: "Neon Terminal Theme".to_string(),
                    description: "Cyberpunk terminal aesthetics".to_string(),
                    category: ContentCategory::Cosmetic,
                    requirement: UnlockRequirement::Purchase { cost: 10000 },
                    preview_available: true,
                },
                UnlockedContent {
                    id: "elite_badge".to_string(),
                    name: "Elite Hacker Badge".to_string(),
                    description: "Show your elite status".to_string(),
                    category: ContentCategory::Cosmetic,
                    requirement: UnlockRequirement::Achievement("elite_hacker".to_string()),
                    preview_available: true,
                },
                UnlockedContent {
                    id: "matrix_effect".to_string(),
                    name: "Matrix Rain Effect".to_string(),
                    description: "Digital rain background effect".to_string(),
                    category: ContentCategory::Cosmetic,
                    requirement: UnlockRequirement::Event("matrix_event".to_string()),
                    preview_available: true,
                },
            ],
        }
    }
}