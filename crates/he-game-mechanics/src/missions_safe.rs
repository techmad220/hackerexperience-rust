use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Safe, original mission system - no HE2 AGPL content
/// All content here is original and safe for closed-source/open-core use

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MissionType {
    Tutorial,
    Practice,
    Challenge,
    Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RewardType {
    Money(u64),
    Software(String),
    Hardware(String),
    Experience(u32),
    Reputation(i32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionObjective {
    pub id: String,
    pub description: String,
    pub completed: bool,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionTemplate {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub mission_type: MissionType,
    pub difficulty: Difficulty,
    pub prerequisites: Vec<String>,
    pub objectives: Vec<MissionObjective>,
    pub rewards: Vec<RewardType>,
    pub time_limit: Option<u64>, // seconds
    pub max_attempts: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeCharacter {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    pub description: String,
    pub dialogue_templates: HashMap<String, Vec<String>>,
}

/// Original mission templates - completely safe for commercial use
pub struct SafeMissionSystem;

impl SafeMissionSystem {
    /// Get comprehensive tutorial mission series
    pub fn get_tutorial_missions() -> Vec<MissionTemplate> {
        vec![
            MissionTemplate {
                id: Uuid::new_v4(),
                title: "System Basics".to_string(),
                description: "Learn fundamental computer systems concepts in a safe practice environment.".to_string(),
                mission_type: MissionType::Tutorial,
                difficulty: Difficulty::Beginner,
                prerequisites: vec![],
                objectives: vec![
                    MissionObjective {
                        id: "login".to_string(),
                        description: "Access the practice terminal".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "explore".to_string(),
                        description: "Navigate the file system".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "complete".to_string(),
                        description: "Run basic commands successfully".to_string(),
                        completed: false,
                        required: true,
                    },
                ],
                rewards: vec![
                    RewardType::Money(500),
                    RewardType::Experience(100),
                    RewardType::Software("Basic Terminal".to_string()),
                ],
                time_limit: None,
                max_attempts: None,
            },
            MissionTemplate {
                id: Uuid::new_v4(),
                title: "Network Fundamentals".to_string(),
                description: "Learn network scanning techniques using designated practice targets.".to_string(),
                mission_type: MissionType::Practice,
                difficulty: Difficulty::Intermediate,
                prerequisites: vec!["System Basics".to_string()],
                objectives: vec![
                    MissionObjective {
                        id: "scan_target".to_string(),
                        description: "Scan the designated practice server".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "identify_services".to_string(),
                        description: "Identify running services".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "report_findings".to_string(),
                        description: "Document your findings".to_string(),
                        completed: false,
                        required: false,
                    },
                ],
                rewards: vec![
                    RewardType::Money(2500),
                    RewardType::Experience(250),
                    RewardType::Software("Network Scanner".to_string()),
                ],
                time_limit: Some(1800), // 30 minutes
                max_attempts: Some(3),
            },
            MissionTemplate {
                id: Uuid::new_v4(),
                title: "Advanced Security Analysis".to_string(),
                description: "Perform comprehensive security assessments on authorized test systems.".to_string(),
                mission_type: MissionType::Challenge,
                difficulty: Difficulty::Advanced,
                prerequisites: vec!["Network Fundamentals".to_string()],
                objectives: vec![
                    MissionObjective {
                        id: "vulnerability_scan".to_string(),
                        description: "Perform automated vulnerability assessment".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "manual_verification".to_string(),
                        description: "Manually verify discovered vulnerabilities".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "exploitation_test".to_string(),
                        description: "Test exploit viability in controlled environment".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "remediation_plan".to_string(),
                        description: "Create detailed remediation recommendations".to_string(),
                        completed: false,
                        required: false,
                    },
                ],
                rewards: vec![
                    RewardType::Money(15000),
                    RewardType::Experience(750),
                    RewardType::Software("Advanced Security Suite".to_string()),
                    RewardType::Reputation(50),
                ],
                time_limit: Some(7200), // 2 hours
                max_attempts: Some(2),
            },
            MissionTemplate {
                id: Uuid::new_v4(),
                title: "Digital Forensics Investigation".to_string(),
                description: "Analyze compromised systems to determine attack vectors and timeline.".to_string(),
                mission_type: MissionType::Challenge,
                difficulty: Difficulty::Expert,
                prerequisites: vec!["Advanced Security Analysis".to_string()],
                objectives: vec![
                    MissionObjective {
                        id: "evidence_collection".to_string(),
                        description: "Collect and preserve digital evidence".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "timeline_reconstruction".to_string(),
                        description: "Reconstruct attack timeline from logs".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "ioc_identification".to_string(),
                        description: "Identify indicators of compromise".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "report_generation".to_string(),
                        description: "Generate comprehensive forensics report".to_string(),
                        completed: false,
                        required: true,
                    },
                ],
                rewards: vec![
                    RewardType::Money(35000),
                    RewardType::Experience(1500),
                    RewardType::Software("Forensics Toolkit Professional".to_string()),
                    RewardType::Reputation(100),
                ],
                time_limit: Some(14400), // 4 hours
                max_attempts: Some(1),
            },
        ]
    }

    /// Get practice missions for skill development
    pub fn get_practice_missions() -> Vec<MissionTemplate> {
        vec![
            MissionTemplate {
                id: Uuid::new_v4(),
                title: "Web Application Security Assessment".to_string(),
                description: "Evaluate web application security using standard testing methodologies.".to_string(),
                mission_type: MissionType::Practice,
                difficulty: Difficulty::Intermediate,
                prerequisites: vec!["Network Fundamentals".to_string()],
                objectives: vec![
                    MissionObjective {
                        id: "recon_phase".to_string(),
                        description: "Perform reconnaissance on target application".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "auth_bypass".to_string(),
                        description: "Test authentication mechanisms for weaknesses".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "injection_testing".to_string(),
                        description: "Test for common injection vulnerabilities".to_string(),
                        completed: false,
                        required: true,
                    },
                ],
                rewards: vec![
                    RewardType::Money(8000),
                    RewardType::Experience(400),
                    RewardType::Software("Web Security Scanner".to_string()),
                ],
                time_limit: Some(3600), // 1 hour
                max_attempts: Some(3),
            },
            MissionTemplate {
                id: Uuid::new_v4(),
                title: "Wireless Network Penetration Testing".to_string(),
                description: "Assess wireless network security using authorized testing procedures.".to_string(),
                mission_type: MissionType::Practice,
                difficulty: Difficulty::Advanced,
                prerequisites: vec!["Web Application Security Assessment".to_string()],
                objectives: vec![
                    MissionObjective {
                        id: "wireless_survey".to_string(),
                        description: "Conduct wireless network survey and mapping".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "encryption_analysis".to_string(),
                        description: "Analyze wireless encryption implementations".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "access_testing".to_string(),
                        description: "Test wireless access controls and authentication".to_string(),
                        completed: false,
                        required: true,
                    },
                ],
                rewards: vec![
                    RewardType::Money(12000),
                    RewardType::Experience(600),
                    RewardType::Software("Wireless Penetration Suite".to_string()),
                    RewardType::Reputation(25),
                ],
                time_limit: Some(5400), // 1.5 hours
                max_attempts: Some(2),
            },
        ]
    }

    /// Get event-based special missions
    pub fn get_event_missions() -> Vec<MissionTemplate> {
        vec![
            MissionTemplate {
                id: Uuid::new_v4(),
                title: "Incident Response Simulation".to_string(),
                description: "Respond to a simulated security incident using industry best practices.".to_string(),
                mission_type: MissionType::Event,
                difficulty: Difficulty::Expert,
                prerequisites: vec!["Digital Forensics Investigation".to_string()],
                objectives: vec![
                    MissionObjective {
                        id: "incident_triage".to_string(),
                        description: "Perform initial incident triage and classification".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "containment".to_string(),
                        description: "Implement containment measures to limit damage".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "threat_hunting".to_string(),
                        description: "Hunt for additional threats in the environment".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "recovery_plan".to_string(),
                        description: "Develop and implement recovery procedures".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "lessons_learned".to_string(),
                        description: "Document lessons learned and improvements".to_string(),
                        completed: false,
                        required: false,
                    },
                ],
                rewards: vec![
                    RewardType::Money(50000),
                    RewardType::Experience(2000),
                    RewardType::Software("Incident Response Platform".to_string()),
                    RewardType::Reputation(200),
                ],
                time_limit: Some(21600), // 6 hours
                max_attempts: Some(1),
            },
        ]
    }

    /// Get safe, original NPCs/characters
    pub fn get_safe_characters() -> Vec<SafeCharacter> {
        vec![
            SafeCharacter {
                id: Uuid::new_v4(),
                name: "Tutorial Bot".to_string(),
                role: "Guide".to_string(),
                description: "An automated guide to help new users learn the system.".to_string(),
                dialogue_templates: {
                    let mut templates = HashMap::new();
                    templates.insert("welcome".to_string(), vec![
                        "Welcome to the practice environment!".to_string(),
                        "Let's start with the basics.".to_string(),
                        "Follow the instructions carefully.".to_string(),
                    ]);
                    templates.insert("success".to_string(), vec![
                        "Great work! You've completed the task.".to_string(),
                        "Excellent! You're making good progress.".to_string(),
                        "Well done! Ready for the next challenge?".to_string(),
                    ]);
                    templates.insert("help".to_string(), vec![
                        "Type 'help' to see available commands.".to_string(),
                        "Need assistance? Check the documentation.".to_string(),
                        "Stuck? Try reviewing the objective description.".to_string(),
                    ]);
                    templates
                },
            },
            SafeCharacter {
                id: Uuid::new_v4(),
                name: "Practice Coordinator".to_string(),
                role: "Supervisor".to_string(),
                description: "Oversees practice sessions and provides challenges.".to_string(),
                dialogue_templates: {
                    let mut templates = HashMap::new();
                    templates.insert("challenge_intro".to_string(), vec![
                        "Ready for a new challenge?".to_string(),
                        "This exercise will test your skills.".to_string(),
                        "Remember, this is a safe learning environment.".to_string(),
                    ]);
                    templates.insert("challenge_complete".to_string(), vec![
                        "Challenge completed successfully!".to_string(),
                        "Your skills are improving.".to_string(),
                        "Ready to try something more advanced?".to_string(),
                    ]);
                    templates
                },
            },
            SafeCharacter {
                id: Uuid::new_v4(),
                name: "Security Analyst Maya".to_string(),
                role: "Mentor".to_string(),
                description: "Senior security analyst who provides advanced training and insights.".to_string(),
                dialogue_templates: {
                    let mut templates = HashMap::new();
                    templates.insert("introduction".to_string(), vec![
                        "I'm Maya, your security analysis mentor.".to_string(),
                        "I'll help you understand the deeper aspects of cybersecurity.".to_string(),
                        "Years of experience have taught me what really matters in this field.".to_string(),
                    ]);
                    templates.insert("advanced_tips".to_string(), vec![
                        "Remember, true security is about understanding your adversaries.".to_string(),
                        "The best defense is knowing how attacks really work.".to_string(),
                        "Every vulnerability tells a story about the system's design.".to_string(),
                    ]);
                    templates.insert("encouragement".to_string(), vec![
                        "You're developing excellent analytical skills.".to_string(),
                        "That's the kind of thinking that leads to breakthroughs.".to_string(),
                        "Keep questioning assumptions - that's how you find the real issues.".to_string(),
                    ]);
                    templates
                },
            },
            SafeCharacter {
                id: Uuid::new_v4(),
                name: "Tech Specialist Alex".to_string(),
                role: "Technical Expert".to_string(),
                description: "Hardware and software specialist who helps with technical challenges.".to_string(),
                dialogue_templates: {
                    let mut templates = HashMap::new();
                    templates.insert("technical_guidance".to_string(), vec![
                        "Let me explain the technical details behind this system.".to_string(),
                        "Understanding the hardware is crucial for effective testing.".to_string(),
                        "Each component has its own security considerations.".to_string(),
                    ]);
                    templates.insert("troubleshooting".to_string(), vec![
                        "When tools don't work as expected, check your configurations first.".to_string(),
                        "Network issues are often caused by simple connectivity problems.".to_string(),
                        "Software compatibility can make or break your testing environment.".to_string(),
                    ]);
                    templates.insert("recommendations".to_string(), vec![
                        "I'd recommend upgrading your scanner for better accuracy.".to_string(),
                        "This hardware configuration should handle more complex tasks.".to_string(),
                        "Consider automating repetitive tasks to improve efficiency.".to_string(),
                    ]);
                    templates
                },
            },
            SafeCharacter {
                id: Uuid::new_v4(),
                name: "Training Administrator Jordan".to_string(),
                role: "Administrator".to_string(),
                description: "Manages training programs and tracks student progress.".to_string(),
                dialogue_templates: {
                    let mut templates = HashMap::new();
                    templates.insert("progress_update".to_string(), vec![
                        "Your training progress has been excellent so far.".to_string(),
                        "I've noted your completion of the latest exercises.".to_string(),
                        "Ready to move on to more advanced coursework?".to_string(),
                    ]);
                    templates.insert("certification".to_string(), vec![
                        "You've met the requirements for this certification level.".to_string(),
                        "Your skills demonstrate real competency in this area.".to_string(),
                        "This credential will serve you well in professional environments.".to_string(),
                    ]);
                    templates.insert("next_steps".to_string(), vec![
                        "Based on your progress, I recommend focusing on these areas next.".to_string(),
                        "These advanced modules will challenge your current skill set.".to_string(),
                        "You're ready for more complex real-world scenarios.".to_string(),
                    ]);
                    templates
                },
            },
            SafeCharacter {
                id: Uuid::new_v4(),
                name: "Forensics Expert Dr. Chen".to_string(),
                role: "Digital Forensics Specialist".to_string(),
                description: "Digital forensics expert who teaches investigation techniques and evidence handling.".to_string(),
                dialogue_templates: {
                    let mut templates = HashMap::new();
                    templates.insert("forensics_intro".to_string(), vec![
                        "Digital forensics requires methodical thinking and attention to detail.".to_string(),
                        "Every piece of evidence must be properly documented and preserved.".to_string(),
                        "The story is always hidden in the data - we just need to find it.".to_string(),
                    ]);
                    templates.insert("investigation_tips".to_string(), vec![
                        "Always maintain chain of custody for digital evidence.".to_string(),
                        "Timeline analysis often reveals the true sequence of events.".to_string(),
                        "Look for artifacts that attackers might not realize they've left behind.".to_string(),
                    ]);
                    templates.insert("case_analysis".to_string(), vec![
                        "This case shows typical patterns we see in real investigations.".to_string(),
                        "Notice how the evidence points to specific attack methodologies.".to_string(),
                        "Your analysis demonstrates understanding of the underlying techniques.".to_string(),
                    ]);
                    templates
                },
            },
            SafeCharacter {
                id: Uuid::new_v4(),
                name: "Incident Response Lead Sam".to_string(),
                role: "Incident Commander".to_string(),
                description: "Experienced incident response leader who coordinates emergency security responses.".to_string(),
                dialogue_templates: {
                    let mut templates = HashMap::new();
                    templates.insert("incident_briefing".to_string(), vec![
                        "We have a developing security incident that requires immediate attention.".to_string(),
                        "Time is critical - every minute counts in incident response.".to_string(),
                        "Follow established procedures but be ready to adapt as the situation evolves.".to_string(),
                    ]);
                    templates.insert("coordination".to_string(), vec![
                        "Effective communication is essential during incident response.".to_string(),
                        "Keep stakeholders informed with regular status updates.".to_string(),
                        "Document everything - we'll need detailed records for the post-incident review.".to_string(),
                    ]);
                    templates.insert("resolution".to_string(), vec![
                        "Excellent work containing the incident and minimizing impact.".to_string(),
                        "Your quick thinking prevented this from becoming a major breach.".to_string(),
                        "Let's schedule a lessons learned session to improve our response procedures.".to_string(),
                    ]);
                    templates
                },
            },
        ]
    }

    /// Get comprehensive training scenarios
    pub fn get_training_scenarios() -> Vec<TrainingScenario> {
        vec![
            TrainingScenario {
                id: Uuid::new_v4(),
                name: "Corporate Network Assessment".to_string(),
                description: "Simulated corporate environment for comprehensive penetration testing practice.".to_string(),
                difficulty: Difficulty::Intermediate,
                environment_type: "Corporate".to_string(),
                target_systems: vec![
                    "Domain Controller (Windows Server 2019)".to_string(),
                    "Web Server (Linux/Apache)".to_string(),
                    "Database Server (MySQL)".to_string(),
                    "Workstations (Windows 10)".to_string(),
                    "Network Infrastructure (Cisco)".to_string(),
                ],
                learning_objectives: vec![
                    "Network reconnaissance and mapping".to_string(),
                    "Vulnerability assessment and prioritization".to_string(),
                    "Exploitation of common vulnerabilities".to_string(),
                    "Lateral movement techniques".to_string(),
                    "Privilege escalation methods".to_string(),
                    "Report generation and recommendations".to_string(),
                ],
                duration_hours: 4,
            },
            TrainingScenario {
                id: Uuid::new_v4(),
                name: "Cloud Security Assessment".to_string(),
                description: "Modern cloud infrastructure security assessment training environment.".to_string(),
                difficulty: Difficulty::Advanced,
                environment_type: "Cloud".to_string(),
                target_systems: vec![
                    "AWS EC2 Instances".to_string(),
                    "S3 Buckets and Storage".to_string(),
                    "Lambda Functions".to_string(),
                    "RDS Database Instances".to_string(),
                    "Load Balancers and Networking".to_string(),
                ],
                learning_objectives: vec![
                    "Cloud service enumeration".to_string(),
                    "IAM policy analysis and exploitation".to_string(),
                    "Storage security assessment".to_string(),
                    "Serverless security evaluation".to_string(),
                    "Cloud-native attack techniques".to_string(),
                    "Cloud security best practices".to_string(),
                ],
                duration_hours: 6,
            },
            TrainingScenario {
                id: Uuid::new_v4(),
                name: "Mobile Application Security".to_string(),
                description: "Comprehensive mobile app security testing environment for iOS and Android applications.".to_string(),
                difficulty: Difficulty::Advanced,
                environment_type: "Mobile".to_string(),
                target_systems: vec![
                    "Android Application (APK)".to_string(),
                    "iOS Application (IPA)".to_string(),
                    "Mobile API Backend".to_string(),
                    "Device Emulators".to_string(),
                    "Mobile Device Management".to_string(),
                ],
                learning_objectives: vec![
                    "Static application analysis".to_string(),
                    "Dynamic testing techniques".to_string(),
                    "API security assessment".to_string(),
                    "Data storage security".to_string(),
                    "Communication channel analysis".to_string(),
                    "Mobile-specific attack vectors".to_string(),
                ],
                duration_hours: 5,
            },
        ]
    }

    /// Generate original practice targets (safe for commercial use)
    pub fn get_practice_targets() -> Vec<PracticeTarget> {
        vec![
            PracticeTarget {
                id: Uuid::new_v4(),
                name: "Training Server Alpha".to_string(),
                ip: "192.168.100.10".to_string(),
                description: "Basic practice server for beginners".to_string(),
                difficulty: Difficulty::Beginner,
                services: vec!["SSH".to_string(), "HTTP".to_string()],
                vulnerabilities: vec!["Weak passwords".to_string()],
                objectives: vec!["Login successfully".to_string(), "Read system info".to_string()],
            },
            PracticeTarget {
                id: Uuid::new_v4(),
                name: "Training Server Beta".to_string(),
                ip: "192.168.100.20".to_string(),
                description: "Intermediate practice server with multiple services".to_string(),
                difficulty: Difficulty::Intermediate,
                services: vec!["SSH".to_string(), "HTTP".to_string(), "FTP".to_string(), "MySQL".to_string()],
                vulnerabilities: vec!["Unpatched software".to_string(), "Default configurations".to_string()],
                objectives: vec!["Enumerate services".to_string(), "Identify vulnerabilities".to_string(), "Access database".to_string()],
            },
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PracticeTarget {
    pub id: Uuid,
    pub name: String,
    pub ip: String,
    pub description: String,
    pub difficulty: Difficulty,
    pub services: Vec<String>,
    pub vulnerabilities: Vec<String>,
    pub objectives: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingScenario {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub difficulty: Difficulty,
    pub environment_type: String,
    pub target_systems: Vec<String>,
    pub learning_objectives: Vec<String>,
    pub duration_hours: u32,
}

/// Mission progress tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionProgress {
    pub mission_id: Uuid,
    pub player_id: Uuid,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub objectives_completed: Vec<String>,
    pub attempts_used: u32,
    pub score: Option<u32>,
}

impl MissionProgress {
    pub fn new(mission_id: Uuid, player_id: Uuid) -> Self {
        Self {
            mission_id,
            player_id,
            started_at: chrono::Utc::now(),
            completed_at: None,
            objectives_completed: vec![],
            attempts_used: 1,
            score: None,
        }
    }

    pub fn complete_objective(&mut self, objective_id: String) {
        if !self.objectives_completed.contains(&objective_id) {
            self.objectives_completed.push(objective_id);
        }
    }

    pub fn is_mission_complete(&self, mission: &MissionTemplate) -> bool {
        let required_objectives: Vec<&String> = mission.objectives
            .iter()
            .filter(|obj| obj.required)
            .map(|obj| &obj.id)
            .collect();

        required_objectives.iter().all(|&obj_id| {
            self.objectives_completed.contains(obj_id)
        })
    }

    pub fn complete_mission(&mut self, score: Option<u32>) {
        self.completed_at = Some(chrono::Utc::now());
        self.score = score;
    }

    pub fn get_completion_percentage(&self, mission: &MissionTemplate) -> f32 {
        if mission.objectives.is_empty() {
            return 100.0;
        }
        
        let completed_count = self.objectives_completed.len() as f32;
        let total_count = mission.objectives.len() as f32;
        (completed_count / total_count) * 100.0
    }
}

/// Advanced mission system with dynamic content generation
pub struct AdvancedMissionSystem;

impl AdvancedMissionSystem {
    /// Generate difficulty-scaled missions based on player skill level
    pub fn generate_adaptive_mission(player_level: u32, skill_areas: &[String]) -> MissionTemplate {
        let difficulty = match player_level {
            1..=10 => Difficulty::Beginner,
            11..=25 => Difficulty::Intermediate,
            26..=50 => Difficulty::Advanced,
            _ => Difficulty::Expert,
        };

        let base_reward = match difficulty {
            Difficulty::Beginner => 1000,
            Difficulty::Intermediate => 5000,
            Difficulty::Advanced => 20000,
            Difficulty::Expert => 50000,
        };

        let skill_focus = skill_areas.first().unwrap_or(&"general".to_string()).clone();
        
        MissionTemplate {
            id: Uuid::new_v4(),
            title: format!("Adaptive {} Challenge", skill_focus),
            description: format!("Personalized {} challenge scaled to your current skill level.", skill_focus),
            mission_type: MissionType::Challenge,
            difficulty,
            prerequisites: if player_level > 10 { 
                vec!["System Basics".to_string()] 
            } else { 
                vec![] 
            },
            objectives: Self::generate_objectives_for_skill(&skill_focus, &difficulty),
            rewards: vec![
                RewardType::Money(base_reward * player_level as u64),
                RewardType::Experience((base_reward / 10) * player_level),
                RewardType::Software(format!("Advanced {} Tools", skill_focus)),
                RewardType::Reputation((player_level * 5) as i32),
            ],
            time_limit: Some(3600 * (player_level / 10 + 1) as u64), // Scales with level
            max_attempts: Some(3),
        }
    }

    fn generate_objectives_for_skill(skill: &str, difficulty: &Difficulty) -> Vec<MissionObjective> {
        match skill.to_lowercase().as_str() {
            "networking" => match difficulty {
                Difficulty::Beginner => vec![
                    MissionObjective {
                        id: "ping_sweep".to_string(),
                        description: "Perform network discovery using ping sweep".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "port_scan".to_string(),
                        description: "Identify open ports on target systems".to_string(),
                        completed: false,
                        required: true,
                    },
                ],
                Difficulty::Intermediate => vec![
                    MissionObjective {
                        id: "service_enumeration".to_string(),
                        description: "Enumerate services and versions".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "vulnerability_scanning".to_string(),
                        description: "Perform automated vulnerability scanning".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "network_mapping".to_string(),
                        description: "Create comprehensive network topology map".to_string(),
                        completed: false,
                        required: false,
                    },
                ],
                Difficulty::Advanced => vec![
                    MissionObjective {
                        id: "advanced_recon".to_string(),
                        description: "Perform advanced reconnaissance using multiple techniques".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "traffic_analysis".to_string(),
                        description: "Analyze network traffic for security insights".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "lateral_movement".to_string(),
                        description: "Demonstrate controlled lateral movement techniques".to_string(),
                        completed: false,
                        required: true,
                    },
                ],
                Difficulty::Expert => vec![
                    MissionObjective {
                        id: "zero_day_research".to_string(),
                        description: "Identify novel attack vectors in network protocols".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "advanced_persistence".to_string(),
                        description: "Establish covert persistence mechanisms".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "counter_forensics".to_string(),
                        description: "Implement anti-forensics techniques".to_string(),
                        completed: false,
                        required: false,
                    },
                ],
            },
            "web_security" => match difficulty {
                Difficulty::Beginner => vec![
                    MissionObjective {
                        id: "directory_traversal".to_string(),
                        description: "Test for directory traversal vulnerabilities".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "sql_injection_basic".to_string(),
                        description: "Identify basic SQL injection points".to_string(),
                        completed: false,
                        required: true,
                    },
                ],
                Difficulty::Intermediate => vec![
                    MissionObjective {
                        id: "xss_testing".to_string(),
                        description: "Test for cross-site scripting vulnerabilities".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "csrf_analysis".to_string(),
                        description: "Analyze CSRF protection mechanisms".to_string(),
                        completed: false,
                        required: true,
                    },
                    MissionObjective {
                        id: "session_management".to_string(),
                        description: "Evaluate session management security".to_string(),
                        completed: false,
                        required: false,
                    },
                ],
                _ => vec![], // Advanced and Expert would have more complex objectives
            },
            _ => vec![
                MissionObjective {
                    id: "general_assessment".to_string(),
                    description: "Perform comprehensive security assessment".to_string(),
                    completed: false,
                    required: true,
                },
            ],
        }
    }

    /// Create mission chains that build upon each other
    pub fn create_mission_chain(theme: &str) -> Vec<MissionTemplate> {
        match theme {
            "penetration_testing" => vec![
                Self::create_recon_mission(),
                Self::create_vulnerability_assessment_mission(),
                Self::create_exploitation_mission(),
                Self::create_post_exploitation_mission(),
                Self::create_reporting_mission(),
            ],
            "incident_response" => vec![
                Self::create_incident_detection_mission(),
                Self::create_containment_mission(),
                Self::create_investigation_mission(),
                Self::create_recovery_mission(),
                Self::create_lessons_learned_mission(),
            ],
            _ => vec![],
        }
    }

    fn create_recon_mission() -> MissionTemplate {
        MissionTemplate {
            id: Uuid::new_v4(),
            title: "Reconnaissance Phase".to_string(),
            description: "Gather intelligence about target systems using passive and active techniques.".to_string(),
            mission_type: MissionType::Practice,
            difficulty: Difficulty::Intermediate,
            prerequisites: vec!["Network Fundamentals".to_string()],
            objectives: vec![
                MissionObjective {
                    id: "passive_recon".to_string(),
                    description: "Conduct passive reconnaissance using OSINT techniques".to_string(),
                    completed: false,
                    required: true,
                },
                MissionObjective {
                    id: "active_scanning".to_string(),
                    description: "Perform active network scanning and enumeration".to_string(),
                    completed: false,
                    required: true,
                },
                MissionObjective {
                    id: "target_profiling".to_string(),
                    description: "Create detailed target profiles and attack surface maps".to_string(),
                    completed: false,
                    required: false,
                },
            ],
            rewards: vec![
                RewardType::Money(7500),
                RewardType::Experience(375),
                RewardType::Software("Advanced Reconnaissance Suite".to_string()),
            ],
            time_limit: Some(7200),
            max_attempts: Some(2),
        }
    }

    fn create_vulnerability_assessment_mission() -> MissionTemplate {
        MissionTemplate {
            id: Uuid::new_v4(),
            title: "Vulnerability Assessment".to_string(),
            description: "Systematically identify and classify security vulnerabilities in target systems.".to_string(),
            mission_type: MissionType::Practice,
            difficulty: Difficulty::Intermediate,
            prerequisites: vec!["Reconnaissance Phase".to_string()],
            objectives: vec![
                MissionObjective {
                    id: "automated_scanning".to_string(),
                    description: "Deploy automated vulnerability scanners".to_string(),
                    completed: false,
                    required: true,
                },
                MissionObjective {
                    id: "manual_testing".to_string(),
                    description: "Perform manual vulnerability verification".to_string(),
                    completed: false,
                    required: true,
                },
                MissionObjective {
                    id: "risk_assessment".to_string(),
                    description: "Classify vulnerabilities by risk level and exploitability".to_string(),
                    completed: false,
                    required: true,
                },
            ],
            rewards: vec![
                RewardType::Money(10000),
                RewardType::Experience(500),
                RewardType::Software("Professional Vulnerability Scanner".to_string()),
                RewardType::Reputation(30),
            ],
            time_limit: Some(10800),
            max_attempts: Some(2),
        }
    }

    fn create_exploitation_mission() -> MissionTemplate {
        MissionTemplate {
            id: Uuid::new_v4(),
            title: "Controlled Exploitation".to_string(),
            description: "Safely demonstrate exploitation of identified vulnerabilities in controlled environment.".to_string(),
            mission_type: MissionType::Challenge,
            difficulty: Difficulty::Advanced,
            prerequisites: vec!["Vulnerability Assessment".to_string()],
            objectives: vec![
                MissionObjective {
                    id: "exploit_development".to_string(),
                    description: "Develop or adapt exploits for identified vulnerabilities".to_string(),
                    completed: false,
                    required: true,
                },
                MissionObjective {
                    id: "safe_exploitation".to_string(),
                    description: "Execute exploits in controlled manner to minimize impact".to_string(),
                    completed: false,
                    required: true,
                },
                MissionObjective {
                    id: "access_verification".to_string(),
                    description: "Verify achieved access level and document findings".to_string(),
                    completed: false,
                    required: true,
                },
            ],
            rewards: vec![
                RewardType::Money(25000),
                RewardType::Experience(1000),
                RewardType::Software("Exploitation Framework Professional".to_string()),
                RewardType::Reputation(75),
            ],
            time_limit: Some(14400),
            max_attempts: Some(1),
        }
    }

    fn create_post_exploitation_mission() -> MissionTemplate {
        MissionTemplate {
            id: Uuid::new_v4(),
            title: "Post-Exploitation Analysis".to_string(),
            description: "Demonstrate controlled post-exploitation techniques while maintaining system integrity.".to_string(),
            mission_type: MissionType::Challenge,
            difficulty: Difficulty::Advanced,
            prerequisites: vec!["Controlled Exploitation".to_string()],
            objectives: vec![
                MissionObjective {
                    id: "privilege_escalation".to_string(),
                    description: "Safely escalate privileges using discovered methods".to_string(),
                    completed: false,
                    required: true,
                },
                MissionObjective {
                    id: "data_exfiltration_demo".to_string(),
                    description: "Demonstrate data access capabilities (test data only)".to_string(),
                    completed: false,
                    required: true,
                },
                MissionObjective {
                    id: "persistence_testing".to_string(),
                    description: "Test persistence mechanisms in controlled environment".to_string(),
                    completed: false,
                    required: false,
                },
            ],
            rewards: vec![
                RewardType::Money(35000),
                RewardType::Experience(1250),
                RewardType::Software("Advanced Post-Exploitation Toolkit".to_string()),
                RewardType::Reputation(100),
            ],
            time_limit: Some(18000),
            max_attempts: Some(1),
        }
    }

    fn create_reporting_mission() -> MissionTemplate {
        MissionTemplate {
            id: Uuid::new_v4(),
            title: "Professional Reporting".to_string(),
            description: "Create comprehensive penetration testing report with findings, risk analysis, and recommendations.".to_string(),
            mission_type: MissionType::Practice,
            difficulty: Difficulty::Intermediate,
            prerequisites: vec!["Post-Exploitation Analysis".to_string()],
            objectives: vec![
                MissionObjective {
                    id: "executive_summary".to_string(),
                    description: "Write clear executive summary for business stakeholders".to_string(),
                    completed: false,
                    required: true,
                },
                MissionObjective {
                    id: "technical_details".to_string(),
                    description: "Document technical findings with proof-of-concept details".to_string(),
                    completed: false,
                    required: true,
                },
                MissionObjective {
                    id: "remediation_plan".to_string(),
                    description: "Provide prioritized remediation recommendations".to_string(),
                    completed: false,
                    required: true,
                },
            ],
            rewards: vec![
                RewardType::Money(15000),
                RewardType::Experience(750),
                RewardType::Software("Professional Reporting Suite".to_string()),
                RewardType::Reputation(50),
            ],
            time_limit: Some(14400),
            max_attempts: Some(3),
        }
    }

    // Incident Response Mission Chain
    fn create_incident_detection_mission() -> MissionTemplate {
        MissionTemplate {
            id: Uuid::new_v4(),
            title: "Incident Detection & Triage".to_string(),
            description: "Rapidly detect, classify, and triage security incidents using monitoring tools and analysis techniques.".to_string(),
            mission_type: MissionType::Event,
            difficulty: Difficulty::Advanced,
            prerequisites: vec!["Digital Forensics Investigation".to_string()],
            objectives: vec![
                MissionObjective {
                    id: "alert_analysis".to_string(),
                    description: "Analyze security alerts and determine true positives".to_string(),
                    completed: false,
                    required: true,
                },
                MissionObjective {
                    id: "incident_classification".to_string(),
                    description: "Classify incident severity and impact level".to_string(),
                    completed: false,
                    required: true,
                },
                MissionObjective {
                    id: "initial_response".to_string(),
                    description: "Execute initial response procedures and notifications".to_string(),
                    completed: false,
                    required: true,
                },
            ],
            rewards: vec![
                RewardType::Money(20000),
                RewardType::Experience(800),
                RewardType::Software("Incident Detection Platform".to_string()),
                RewardType::Reputation(60),
            ],
            time_limit: Some(1800), // 30 minutes - time critical
            max_attempts: Some(2),
        }
    }

    fn create_containment_mission() -> MissionTemplate {
        MissionTemplate {
            id: Uuid::new_v4(),
            title: "Incident Containment".to_string(),
            description: "Implement containment measures to prevent incident spread while preserving evidence.".to_string(),
            mission_type: MissionType::Event,
            difficulty: Difficulty::Advanced,
            prerequisites: vec!["Incident Detection & Triage".to_string()],
            objectives: vec![
                MissionObjective {
                    id: "isolation_procedures".to_string(),
                    description: "Safely isolate affected systems without data loss".to_string(),
                    completed: false,
                    required: true,
                },
                MissionObjective {
                    id: "evidence_preservation".to_string(),
                    description: "Preserve digital evidence for investigation".to_string(),
                    completed: false,
                    required: true,
                },
                MissionObjective {
                    id: "business_continuity".to_string(),
                    description: "Maintain business operations where possible".to_string(),
                    completed: false,
                    required: false,
                },
            ],
            rewards: vec![
                RewardType::Money(30000),
                RewardType::Experience(1000),
                RewardType::Software("Containment Automation Tools".to_string()),
                RewardType::Reputation(80),
            ],
            time_limit: Some(3600), // 1 hour
            max_attempts: Some(1),
        }
    }

    fn create_investigation_mission() -> MissionTemplate {
        MissionTemplate {
            id: Uuid::new_v4(),
            title: "Deep Investigation".to_string(),
            description: "Conduct thorough investigation to determine attack vectors, scope, and attribution.".to_string(),
            mission_type: MissionType::Event,
            difficulty: Difficulty::Expert,
            prerequisites: vec!["Incident Containment".to_string()],
            objectives: vec![
                MissionObjective {
                    id: "root_cause_analysis".to_string(),
                    description: "Identify initial attack vector and root cause".to_string(),
                    completed: false,
                    required: true,
                },
                MissionObjective {
                    id: "scope_determination".to_string(),
                    description: "Determine full scope of compromise".to_string(),
                    completed: false,
                    required: true,
                },
                MissionObjective {
                    id: "threat_attribution".to_string(),
                    description: "Analyze attack patterns for threat attribution".to_string(),
                    completed: false,
                    required: false,
                },
            ],
            rewards: vec![
                RewardType::Money(45000),
                RewardType::Experience(1500),
                RewardType::Software("Advanced Investigation Suite".to_string()),
                RewardType::Reputation(120),
            ],
            time_limit: Some(21600), // 6 hours
            max_attempts: Some(1),
        }
    }

    fn create_recovery_mission() -> MissionTemplate {
        MissionTemplate {
            id: Uuid::new_v4(),
            title: "System Recovery".to_string(),
            description: "Safely restore systems to normal operations with improved security posture.".to_string(),
            mission_type: MissionType::Practice,
            difficulty: Difficulty::Advanced,
            prerequisites: vec!["Deep Investigation".to_string()],
            objectives: vec![
                MissionObjective {
                    id: "system_restoration".to_string(),
                    description: "Restore systems from clean backups or rebuilds".to_string(),
                    completed: false,
                    required: true,
                },
                MissionObjective {
                    id: "security_hardening".to_string(),
                    description: "Implement additional security measures".to_string(),
                    completed: false,
                    required: true,
                },
                MissionObjective {
                    id: "monitoring_enhancement".to_string(),
                    description: "Enhance monitoring to detect similar attacks".to_string(),
                    completed: false,
                    required: true,
                },
            ],
            rewards: vec![
                RewardType::Money(25000),
                RewardType::Experience(1000),
                RewardType::Software("Recovery Automation Platform".to_string()),
                RewardType::Reputation(75),
            ],
            time_limit: Some(18000), // 5 hours
            max_attempts: Some(2),
        }
    }

    fn create_lessons_learned_mission() -> MissionTemplate {
        MissionTemplate {
            id: Uuid::new_v4(),
            title: "Lessons Learned Analysis".to_string(),
            description: "Document lessons learned and implement process improvements for future incidents.".to_string(),
            mission_type: MissionType::Practice,
            difficulty: Difficulty::Intermediate,
            prerequisites: vec!["System Recovery".to_string()],
            objectives: vec![
                MissionObjective {
                    id: "incident_documentation".to_string(),
                    description: "Create comprehensive incident documentation".to_string(),
                    completed: false,
                    required: true,
                },
                MissionObjective {
                    id: "process_improvements".to_string(),
                    description: "Identify and document process improvements".to_string(),
                    completed: false,
                    required: true,
                },
                MissionObjective {
                    id: "training_updates".to_string(),
                    description: "Update training materials based on lessons learned".to_string(),
                    completed: false,
                    required: false,
                },
            ],
            rewards: vec![
                RewardType::Money(15000),
                RewardType::Experience(750),
                RewardType::Software("Documentation and Analysis Tools".to_string()),
                RewardType::Reputation(50),
            ],
            time_limit: Some(14400), // 4 hours
            max_attempts: Some(3),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_missions_creation() {
        let missions = SafeMissionSystem::get_tutorial_missions();
        assert!(!missions.is_empty());
        
        for mission in missions {
            // Ensure no AGPL-contaminated content
            assert!(!mission.title.to_lowercase().contains("megabank"));
            assert!(!mission.description.to_lowercase().contains("infiltrate"));
            assert!(!mission.title.to_lowercase().contains("federal"));
        }
    }

    #[test]
    fn test_safe_characters_creation() {
        let characters = SafeMissionSystem::get_safe_characters();
        assert!(!characters.is_empty());
        
        for character in characters {
            // Ensure original, non-AGPL content
            assert!(character.name.contains("Bot") || character.name.contains("Coordinator"));
            assert!(!character.name.to_lowercase().contains("npc names from he2"));
        }
    }

    #[test]
    fn test_mission_progress() {
        let mission_id = Uuid::new_v4();
        let player_id = Uuid::new_v4();
        let mut progress = MissionProgress::new(mission_id, player_id);
        
        progress.complete_objective("test_objective".to_string());
        assert!(progress.objectives_completed.contains(&"test_objective".to_string()));
    }
}