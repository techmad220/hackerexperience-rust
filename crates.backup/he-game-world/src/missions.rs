//! Mission content and templates

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Mission template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub mission_type: MissionType,
    pub difficulty: i32, // 1-5
    pub objectives: Vec<MissionObjective>,
    pub rewards: MissionRewards,
    pub requirements: MissionRequirements,
    pub story_text: String,
    pub completion_text: String,
    pub is_tutorial: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionType {
    Tutorial,
    Story,
    Daily,
    Hack,
    Steal,
    Delete,
    Install,
    Transfer,
    Defend,
    Research,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionObjective {
    pub id: Uuid,
    pub description: String,
    pub objective_type: ObjectiveType,
    pub target: Option<String>, // IP or item name
    pub amount: Option<i32>,
    pub is_completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectiveType {
    HackServer,
    DownloadFile,
    DeleteFile,
    InstallSoftware,
    TransferMoney,
    DeleteLogs,
    StayUndetected,
    CollectData,
    ResearchSoftware,
    UpgradeHardware,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionRewards {
    pub money: i64,
    pub experience: i32,
    pub reputation: i32,
    pub software: Option<String>,
    pub unlock_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionRequirements {
    pub min_level: i32,
    pub prerequisite_missions: Vec<Uuid>,
    pub required_software: Vec<String>,
    pub required_hardware_cpu: Option<i32>,
}

/// Generate default missions for the game
pub fn generate_default_missions() -> Vec<MissionTemplate> {
    let mut missions = Vec::new();

    // Tutorial missions
    missions.push(create_tutorial_mission_1());
    missions.push(create_tutorial_mission_2());
    missions.push(create_tutorial_mission_3());

    // Early game missions
    missions.push(create_first_hack_mission());
    missions.push(create_bank_heist_mission());
    missions.push(create_virus_mission());

    // Mid game missions
    missions.push(create_corporate_espionage_mission());
    missions.push(create_government_hack_mission());

    // Late game missions
    missions.push(create_elite_server_mission());

    // Daily missions
    missions.extend(create_daily_missions());

    missions
}

fn create_tutorial_mission_1() -> MissionTemplate {
    MissionTemplate {
        id: Uuid::new_v4(),
        name: "Welcome to HackerExperience".to_string(),
        description: "Learn the basics of hacking".to_string(),
        mission_type: MissionType::Tutorial,
        difficulty: 1,
        objectives: vec![
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Open the Internet page".to_string(),
                objective_type: ObjectiveType::HackServer,
                target: Some("1.2.3.4".to_string()), // First Whois
                amount: None,
                is_completed: false,
            },
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Scan the First Whois server (1.2.3.4)".to_string(),
                objective_type: ObjectiveType::CollectData,
                target: Some("1.2.3.4".to_string()),
                amount: None,
                is_completed: false,
            },
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Login to the First Whois server".to_string(),
                objective_type: ObjectiveType::HackServer,
                target: Some("1.2.3.4".to_string()),
                amount: None,
                is_completed: false,
            },
        ],
        rewards: MissionRewards {
            money: 100,
            experience: 50,
            reputation: 10,
            software: Some("Basic Cracker 1.0".to_string()),
            unlock_content: None,
        },
        requirements: MissionRequirements {
            min_level: 1,
            prerequisite_missions: vec![],
            required_software: vec![],
            required_hardware_cpu: None,
        },
        story_text: "Welcome, rookie hacker! The underground network awaits. Your first task is to access the First Whois server - a public database that will teach you the basics of our world.".to_string(),
        completion_text: "Excellent! You've taken your first steps into the hacking world. The First Whois server is just the beginning. Greater challenges await!".to_string(),
        is_tutorial: true,
    }
}

fn create_tutorial_mission_2() -> MissionTemplate {
    MissionTemplate {
        id: Uuid::new_v4(),
        name: "Data Acquisition".to_string(),
        description: "Learn how to download and manage files".to_string(),
        mission_type: MissionType::Tutorial,
        difficulty: 1,
        objectives: vec![
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Download a file from any server".to_string(),
                objective_type: ObjectiveType::DownloadFile,
                target: None,
                amount: Some(1),
                is_completed: false,
            },
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Check your Log File".to_string(),
                objective_type: ObjectiveType::CollectData,
                target: None,
                amount: None,
                is_completed: false,
            },
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Delete your logs to hide your tracks".to_string(),
                objective_type: ObjectiveType::DeleteLogs,
                target: None,
                amount: None,
                is_completed: false,
            },
        ],
        rewards: MissionRewards {
            money: 250,
            experience: 100,
            reputation: 15,
            software: Some("Log Cleaner 1.0".to_string()),
            unlock_content: None,
        },
        requirements: MissionRequirements {
            min_level: 1,
            prerequisite_missions: vec![], // Should require tutorial 1
            required_software: vec!["Basic Cracker 1.0".to_string()],
            required_hardware_cpu: None,
        },
        story_text: "Every action leaves a trace. Smart hackers know how to cover their tracks. Let me teach you about logs and why they matter.".to_string(),
        completion_text: "Good work! Remember: always clean your logs. The authorities are always watching, and a careless hacker is a caught hacker.".to_string(),
        is_tutorial: true,
    }
}

fn create_tutorial_mission_3() -> MissionTemplate {
    MissionTemplate {
        id: Uuid::new_v4(),
        name: "First Score".to_string(),
        description: "Steal money from a vulnerable target".to_string(),
        mission_type: MissionType::Tutorial,
        difficulty: 1,
        objectives: vec![
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Hack into a home computer".to_string(),
                objective_type: ObjectiveType::HackServer,
                target: None, // Any tier 1 server
                amount: None,
                is_completed: false,
            },
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Transfer at least $100 to your account".to_string(),
                objective_type: ObjectiveType::TransferMoney,
                target: None,
                amount: Some(100),
                is_completed: false,
            },
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Delete all logs on the target server".to_string(),
                objective_type: ObjectiveType::DeleteLogs,
                target: None,
                amount: None,
                is_completed: false,
            },
        ],
        rewards: MissionRewards {
            money: 500,
            experience: 200,
            reputation: 25,
            software: Some("Password Cracker 2.0".to_string()),
            unlock_content: Some("Bank Accounts".to_string()),
        },
        requirements: MissionRequirements {
            min_level: 2,
            prerequisite_missions: vec![],
            required_software: vec!["Basic Cracker 1.0".to_string()],
            required_hardware_cpu: None,
        },
        story_text: "Time for your first real hack. Find a vulnerable home computer and transfer their money. Don't forget to cover your tracks!".to_string(),
        completion_text: "Excellent! You're learning fast. But home computers are small-time. Soon, we'll go after bigger targets.".to_string(),
        is_tutorial: true,
    }
}

fn create_first_hack_mission() -> MissionTemplate {
    MissionTemplate {
        id: Uuid::new_v4(),
        name: "Corporate Infiltration".to_string(),
        description: "Hack into your first corporate server".to_string(),
        mission_type: MissionType::Story,
        difficulty: 2,
        objectives: vec![
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Hack a company server".to_string(),
                objective_type: ObjectiveType::HackServer,
                target: None, // Any tier 2 company
                amount: None,
                is_completed: false,
            },
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Download the customer database".to_string(),
                objective_type: ObjectiveType::DownloadFile,
                target: Some("database.db".to_string()),
                amount: None,
                is_completed: false,
            },
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Stay undetected for 5 minutes".to_string(),
                objective_type: ObjectiveType::StayUndetected,
                target: None,
                amount: Some(300), // seconds
                is_completed: false,
            },
        ],
        rewards: MissionRewards {
            money: 2000,
            experience: 500,
            reputation: 50,
            software: Some("Advanced Cracker 3.0".to_string()),
            unlock_content: None,
        },
        requirements: MissionRequirements {
            min_level: 5,
            prerequisite_missions: vec![],
            required_software: vec!["Password Cracker 2.0".to_string()],
            required_hardware_cpu: Some(1500),
        },
        story_text: "A client needs corporate data. They're paying well, but corporate servers have better security. Are you ready?".to_string(),
        completion_text: "Impressive work! The client is pleased. Your reputation in the underground is growing.".to_string(),
        is_tutorial: false,
    }
}

fn create_bank_heist_mission() -> MissionTemplate {
    MissionTemplate {
        id: Uuid::new_v4(),
        name: "The First Bank Job".to_string(),
        description: "Rob your first bank - a rite of passage for any hacker".to_string(),
        mission_type: MissionType::Story,
        difficulty: 3,
        objectives: vec![
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Hack into a bank server".to_string(),
                objective_type: ObjectiveType::HackServer,
                target: None, // Any tier 3 bank
                amount: None,
                is_completed: false,
            },
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Transfer at least $10,000".to_string(),
                objective_type: ObjectiveType::TransferMoney,
                target: None,
                amount: Some(10000),
                is_completed: false,
            },
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Install a virus for future access".to_string(),
                objective_type: ObjectiveType::InstallSoftware,
                target: Some("virus".to_string()),
                amount: None,
                is_completed: false,
            },
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Delete all traces of your presence".to_string(),
                objective_type: ObjectiveType::DeleteLogs,
                target: None,
                amount: None,
                is_completed: false,
            },
        ],
        rewards: MissionRewards {
            money: 25000,
            experience: 1500,
            reputation: 200,
            software: Some("Bank Collector 3.0".to_string()),
            unlock_content: Some("Cryptocurrency".to_string()),
        },
        requirements: MissionRequirements {
            min_level: 10,
            prerequisite_missions: vec![],
            required_software: vec!["Advanced Cracker 3.0".to_string()],
            required_hardware_cpu: Some(3000),
        },
        story_text: "Every hacker remembers their first bank job. Banks have serious security, but the payoff is worth it. Just don't get caught - bank fraud is serious business.".to_string(),
        completion_text: "You did it! Your first successful bank heist. The authorities will be looking for you now. Stay sharp and keep moving.".to_string(),
        is_tutorial: false,
    }
}

fn create_virus_mission() -> MissionTemplate {
    MissionTemplate {
        id: Uuid::new_v4(),
        name: "Viral Outbreak".to_string(),
        description: "Spread a virus across multiple servers".to_string(),
        mission_type: MissionType::Story,
        difficulty: 3,
        objectives: vec![
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Install viruses on 5 different servers".to_string(),
                objective_type: ObjectiveType::InstallSoftware,
                target: Some("virus".to_string()),
                amount: Some(5),
                is_completed: false,
            },
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Ensure viruses remain undetected for 10 minutes".to_string(),
                objective_type: ObjectiveType::StayUndetected,
                target: None,
                amount: Some(600),
                is_completed: false,
            },
        ],
        rewards: MissionRewards {
            money: 15000,
            experience: 1000,
            reputation: 150,
            software: Some("Worm 2.0".to_string()),
            unlock_content: None,
        },
        requirements: MissionRequirements {
            min_level: 8,
            prerequisite_missions: vec![],
            required_software: vec!["Basic Virus 1.0".to_string()],
            required_hardware_cpu: Some(2500),
        },
        story_text: "A client wants to create chaos. Spread this virus to as many servers as possible. The more infections, the bigger your bonus.".to_string(),
        completion_text: "The virus is spreading rapidly! Your client is thrilled with the chaos. Your reputation as a virus specialist is growing.".to_string(),
        is_tutorial: false,
    }
}

fn create_corporate_espionage_mission() -> MissionTemplate {
    MissionTemplate {
        id: Uuid::new_v4(),
        name: "Industrial Espionage".to_string(),
        description: "Steal trade secrets from a rival corporation".to_string(),
        mission_type: MissionType::Story,
        difficulty: 4,
        objectives: vec![
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Infiltrate MegaCorp's research server".to_string(),
                objective_type: ObjectiveType::HackServer,
                target: Some("research.megacorp.com".to_string()),
                amount: None,
                is_completed: false,
            },
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Download the prototype blueprints".to_string(),
                objective_type: ObjectiveType::DownloadFile,
                target: Some("prototype_v3.blueprint".to_string()),
                amount: None,
                is_completed: false,
            },
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Plant false data to mislead them".to_string(),
                objective_type: ObjectiveType::InstallSoftware,
                target: Some("fake_data.txt".to_string()),
                amount: None,
                is_completed: false,
            },
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Frame another hacker for the breach".to_string(),
                objective_type: ObjectiveType::DeleteLogs,
                target: None,
                amount: None,
                is_completed: false,
            },
        ],
        rewards: MissionRewards {
            money: 100000,
            experience: 5000,
            reputation: 500,
            software: Some("Elite Cracker 4.0".to_string()),
            unlock_content: None,
        },
        requirements: MissionRequirements {
            min_level: 15,
            prerequisite_missions: vec![],
            required_software: vec!["Advanced Cracker 3.0".to_string(), "Trojan Horse 3.0".to_string()],
            required_hardware_cpu: Some(5000),
        },
        story_text: "Corporate warfare at its finest. Our client wants their competitor's latest research. This is high-stakes hacking - one mistake and you'll have corporate lawyers AND law enforcement after you.".to_string(),
        completion_text: "Perfect execution! The client got their data, and MegaCorp is chasing ghosts. You're becoming a legend in corporate espionage circles.".to_string(),
        is_tutorial: false,
    }
}

fn create_government_hack_mission() -> MissionTemplate {
    MissionTemplate {
        id: Uuid::new_v4(),
        name: "State Secrets".to_string(),
        description: "Breach government security - extremely dangerous".to_string(),
        mission_type: MissionType::Story,
        difficulty: 5,
        objectives: vec![
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Hack into a government server".to_string(),
                objective_type: ObjectiveType::HackServer,
                target: None, // Any tier 4 government
                amount: None,
                is_completed: false,
            },
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Download classified documents".to_string(),
                objective_type: ObjectiveType::DownloadFile,
                target: Some("classified.txt".to_string()),
                amount: None,
                is_completed: false,
            },
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Remain completely undetected".to_string(),
                objective_type: ObjectiveType::StayUndetected,
                target: None,
                amount: Some(900), // 15 minutes
                is_completed: false,
            },
        ],
        rewards: MissionRewards {
            money: 500000,
            experience: 10000,
            reputation: 1000,
            software: Some("Quantum Cracker 5.0".to_string()),
            unlock_content: Some("Elite Hackers Club".to_string()),
        },
        requirements: MissionRequirements {
            min_level: 20,
            prerequisite_missions: vec![],
            required_software: vec!["Elite Cracker 4.0".to_string(), "Ghost Mode 3.0".to_string()],
            required_hardware_cpu: Some(10000),
        },
        story_text: "This is it - the big leagues. Government servers are protected by the best security in the world. If you pull this off, you'll be a legend. If you fail... well, let's not think about that.".to_string(),
        completion_text: "Unbelievable! You actually did it! You're now on every government watchlist, but you're also a legend. Welcome to the elite.".to_string(),
        is_tutorial: false,
    }
}

fn create_elite_server_mission() -> MissionTemplate {
    MissionTemplate {
        id: Uuid::new_v4(),
        name: "The Mystery Server".to_string(),
        description: "Investigate the legendary 13.37.13.37 server".to_string(),
        mission_type: MissionType::Story,
        difficulty: 5,
        objectives: vec![
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Find and hack the mystery server (13.37.13.37)".to_string(),
                objective_type: ObjectiveType::HackServer,
                target: Some("13.37.13.37".to_string()),
                amount: None,
                is_completed: false,
            },
            MissionObjective {
                id: Uuid::new_v4(),
                description: "Decrypt the hidden message".to_string(),
                objective_type: ObjectiveType::DownloadFile,
                target: Some("README.txt".to_string()),
                amount: None,
                is_completed: false,
            },
        ],
        rewards: MissionRewards {
            money: 1000000,
            experience: 50000,
            reputation: 5000,
            software: Some("Ultimate Cracker 10.0".to_string()),
            unlock_content: Some("End Game Content".to_string()),
        },
        requirements: MissionRequirements {
            min_level: 30,
            prerequisite_missions: vec![],
            required_software: vec!["Quantum Cracker 5.0".to_string(), "Quantum Decryption 10.0".to_string()],
            required_hardware_cpu: Some(50000),
        },
        story_text: "Legends speak of a server that no one has ever successfully hacked. They say it contains the secrets of the original hackers. Many have tried. All have failed. Will you be the first?".to_string(),
        completion_text: "IMPOSSIBLE! You've done what no one thought could be done! The secrets of 13.37.13.37 are yours. You are now a legend among legends.".to_string(),
        is_tutorial: false,
    }
}

fn create_daily_missions() -> Vec<MissionTemplate> {
    vec![
        MissionTemplate {
            id: Uuid::new_v4(),
            name: "Daily Hack".to_string(),
            description: "Hack 3 servers of any type".to_string(),
            mission_type: MissionType::Daily,
            difficulty: 2,
            objectives: vec![
                MissionObjective {
                    id: Uuid::new_v4(),
                    description: "Hack 3 different servers".to_string(),
                    objective_type: ObjectiveType::HackServer,
                    target: None,
                    amount: Some(3),
                    is_completed: false,
                },
            ],
            rewards: MissionRewards {
                money: 1000,
                experience: 250,
                reputation: 25,
                software: None,
                unlock_content: None,
            },
            requirements: MissionRequirements {
                min_level: 1,
                prerequisite_missions: vec![],
                required_software: vec![],
                required_hardware_cpu: None,
            },
            story_text: "Daily practice keeps skills sharp.".to_string(),
            completion_text: "Daily mission complete!".to_string(),
            is_tutorial: false,
        },
        MissionTemplate {
            id: Uuid::new_v4(),
            name: "Data Collector".to_string(),
            description: "Download 10 files from various servers".to_string(),
            mission_type: MissionType::Daily,
            difficulty: 2,
            objectives: vec![
                MissionObjective {
                    id: Uuid::new_v4(),
                    description: "Download 10 files".to_string(),
                    objective_type: ObjectiveType::DownloadFile,
                    target: None,
                    amount: Some(10),
                    is_completed: false,
                },
            ],
            rewards: MissionRewards {
                money: 1500,
                experience: 300,
                reputation: 30,
                software: None,
                unlock_content: None,
            },
            requirements: MissionRequirements {
                min_level: 3,
                prerequisite_missions: vec![],
                required_software: vec![],
                required_hardware_cpu: None,
            },
            story_text: "Collect data for analysis.".to_string(),
            completion_text: "Data collected successfully!".to_string(),
            is_tutorial: false,
        },
        MissionTemplate {
            id: Uuid::new_v4(),
            name: "Clean Sweep".to_string(),
            description: "Delete logs from 5 servers".to_string(),
            mission_type: MissionType::Daily,
            difficulty: 2,
            objectives: vec![
                MissionObjective {
                    id: Uuid::new_v4(),
                    description: "Clean logs on 5 servers".to_string(),
                    objective_type: ObjectiveType::DeleteLogs,
                    target: None,
                    amount: Some(5),
                    is_completed: false,
                },
            ],
            rewards: MissionRewards {
                money: 2000,
                experience: 400,
                reputation: 40,
                software: None,
                unlock_content: None,
            },
            requirements: MissionRequirements {
                min_level: 5,
                prerequisite_missions: vec![],
                required_software: vec!["Log Cleaner 1.0".to_string()],
                required_hardware_cpu: None,
            },
            story_text: "Help other hackers stay hidden.".to_string(),
            completion_text: "Tracks covered!".to_string(),
            is_tutorial: false,
        },
    ]
}