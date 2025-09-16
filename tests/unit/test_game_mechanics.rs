use he_game_mechanics::*;
use he_core::*;
use he_db::*;
use chrono::Utc;

#[cfg(test)]
mod process_engine_tests {
    use super::*;
    use he_game_mechanics::engine::process_engine::*;

    #[tokio::test]
    async fn test_process_creation() {
        let mut engine = ProcessEngine::new();

        let process = GameProcess {
            id: 1,
            process_type: ProcessType::Crack,
            priority: ProcessPriority::High,
            cpu_usage: 50.0,
            ram_usage: 256.0,
            bandwidth_usage: 10.0,
            progress: 0.0,
            time_remaining: 120,
            owner_id: 1,
            target_id: Some(2),
            software_id: Some(100),
            status: ProcessStatus::Running,
            created_at: Utc::now(),
        };

        engine.add_process(process.clone());
        assert_eq!(engine.get_process(1), Some(&process));
    }

    #[tokio::test]
    async fn test_process_scheduling() {
        let mut engine = ProcessEngine::new();

        // Add multiple processes with different priorities
        let high_priority = GameProcess {
            id: 1,
            process_type: ProcessType::Hack,
            priority: ProcessPriority::High,
            cpu_usage: 30.0,
            ram_usage: 128.0,
            bandwidth_usage: 5.0,
            progress: 0.0,
            time_remaining: 60,
            owner_id: 1,
            target_id: None,
            software_id: None,
            status: ProcessStatus::Running,
            created_at: Utc::now(),
        };

        let low_priority = GameProcess {
            id: 2,
            process_type: ProcessType::Download,
            priority: ProcessPriority::Low,
            cpu_usage: 20.0,
            ram_usage: 64.0,
            bandwidth_usage: 50.0,
            progress: 0.0,
            time_remaining: 30,
            owner_id: 1,
            target_id: None,
            software_id: None,
            status: ProcessStatus::Running,
            created_at: Utc::now(),
        };

        engine.add_process(high_priority);
        engine.add_process(low_priority);

        // High priority should be scheduled first
        let scheduled = engine.get_scheduled_processes();
        assert_eq!(scheduled[0].id, 1);
        assert_eq!(scheduled[1].id, 2);
    }

    #[tokio::test]
    async fn test_resource_allocation() {
        let engine = ProcessEngine::new();

        let hardware = Hardware {
            cpu_cores: 4,
            cpu_speed: 3000,
            ram_size: 8192,
            bandwidth: 100,
        };

        let process = GameProcess {
            id: 1,
            process_type: ProcessType::Research,
            priority: ProcessPriority::Normal,
            cpu_usage: 75.0,
            ram_usage: 4096.0,
            bandwidth_usage: 10.0,
            progress: 0.0,
            time_remaining: 300,
            owner_id: 1,
            target_id: None,
            software_id: None,
            status: ProcessStatus::Running,
            created_at: Utc::now(),
        };

        assert!(engine.can_allocate_resources(&hardware, &process));

        let heavy_process = GameProcess {
            id: 2,
            process_type: ProcessType::Research,
            priority: ProcessPriority::Normal,
            cpu_usage: 95.0,
            ram_usage: 9000.0, // More than available RAM
            bandwidth_usage: 10.0,
            progress: 0.0,
            time_remaining: 300,
            owner_id: 1,
            target_id: None,
            software_id: None,
            status: ProcessStatus::Running,
            created_at: Utc::now(),
        };

        assert!(!engine.can_allocate_resources(&hardware, &heavy_process));
    }

    #[tokio::test]
    async fn test_process_completion() {
        let mut engine = ProcessEngine::new();

        let mut process = GameProcess {
            id: 1,
            process_type: ProcessType::Install,
            priority: ProcessPriority::Normal,
            cpu_usage: 25.0,
            ram_usage: 512.0,
            bandwidth_usage: 0.0,
            progress: 95.0,
            time_remaining: 5,
            owner_id: 1,
            target_id: None,
            software_id: Some(200),
            status: ProcessStatus::Running,
            created_at: Utc::now(),
        };

        engine.add_process(process.clone());

        // Simulate time passing
        engine.update_process_progress(1, 5.0);

        let updated = engine.get_process(1).unwrap();
        assert_eq!(updated.progress, 100.0);
        assert_eq!(updated.status, ProcessStatus::Completed);
    }
}

#[cfg(test)]
mod hardware_system_tests {
    use super::*;
    use he_game_mechanics::engine::hardware_engine::*;

    #[tokio::test]
    async fn test_hardware_performance_calculation() {
        let engine = HardwareEngine::new();

        let components = vec![
            Component {
                id: 1,
                component_type: ComponentType::CPU,
                name: "Intel Core i9".to_string(),
                level: 10,
                performance: 95.0,
                power_consumption: 150,
                price: 50000,
            },
            Component {
                id: 2,
                component_type: ComponentType::RAM,
                name: "32GB DDR5".to_string(),
                level: 8,
                performance: 85.0,
                power_consumption: 20,
                price: 15000,
            },
            Component {
                id: 3,
                component_type: ComponentType::HDD,
                name: "2TB NVMe SSD".to_string(),
                level: 7,
                performance: 90.0,
                power_consumption: 10,
                price: 10000,
            },
        ];

        let performance = engine.calculate_performance(&components);
        assert!(performance > 85.0);
        assert!(performance <= 100.0);
    }

    #[tokio::test]
    async fn test_hardware_bottleneck_detection() {
        let engine = HardwareEngine::new();

        let components = vec![
            Component {
                id: 1,
                component_type: ComponentType::CPU,
                name: "Budget CPU".to_string(),
                level: 2,
                performance: 25.0,
                power_consumption: 50,
                price: 1000,
            },
            Component {
                id: 2,
                component_type: ComponentType::RAM,
                name: "64GB DDR5".to_string(),
                level: 10,
                performance: 95.0,
                power_consumption: 30,
                price: 30000,
            },
        ];

        let bottleneck = engine.detect_bottleneck(&components);
        assert_eq!(bottleneck, Some(ComponentType::CPU));
    }

    #[tokio::test]
    async fn test_power_consumption() {
        let engine = HardwareEngine::new();

        let components = vec![
            Component {
                id: 1,
                component_type: ComponentType::CPU,
                name: "High-end CPU".to_string(),
                level: 9,
                performance: 90.0,
                power_consumption: 200,
                price: 40000,
            },
            Component {
                id: 2,
                component_type: ComponentType::GPU,
                name: "RTX 4090".to_string(),
                level: 10,
                performance: 98.0,
                power_consumption: 450,
                price: 150000,
            },
            Component {
                id: 3,
                component_type: ComponentType::PSU,
                name: "1000W PSU".to_string(),
                level: 8,
                performance: 0.0,
                power_consumption: -1000, // Negative means it provides power
                price: 10000,
            },
        ];

        let total_consumption = engine.calculate_power_consumption(&components);
        assert_eq!(total_consumption, 650);
        assert!(engine.has_sufficient_power(&components));
    }
}

#[cfg(test)]
mod software_system_tests {
    use super::*;
    use he_game_mechanics::engine::software_engine::*;

    #[tokio::test]
    async fn test_software_dependency_resolution() {
        let mut engine = SoftwareEngine::new();

        let os = Software {
            id: 1,
            name: "HackOS".to_string(),
            version: "10.0".to_string(),
            software_type: SoftwareType::OS,
            size: 2048,
            requirements: vec![],
            provides: vec!["os:linux".to_string()],
        };

        let firewall = Software {
            id: 2,
            name: "UltraFirewall".to_string(),
            version: "3.5".to_string(),
            software_type: SoftwareType::Firewall,
            size: 512,
            requirements: vec!["os:linux".to_string()],
            provides: vec!["firewall:advanced".to_string()],
        };

        let cracker = Software {
            id: 3,
            name: "EliteCracker".to_string(),
            version: "2.1".to_string(),
            software_type: SoftwareType::Cracker,
            size: 256,
            requirements: vec!["os:linux".to_string(), "firewall:advanced".to_string()],
            provides: vec![],
        };

        engine.add_software(os);
        engine.add_software(firewall);
        engine.add_software(cracker);

        let dependencies = engine.resolve_dependencies(3);
        assert_eq!(dependencies, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_software_conflict_detection() {
        let mut engine = SoftwareEngine::new();

        let antivirus1 = Software {
            id: 1,
            name: "VirusShield".to_string(),
            version: "5.0".to_string(),
            software_type: SoftwareType::Antivirus,
            size: 1024,
            requirements: vec![],
            provides: vec!["antivirus:protection".to_string()],
        };

        let antivirus2 = Software {
            id: 2,
            name: "MegaProtect".to_string(),
            version: "3.0".to_string(),
            software_type: SoftwareType::Antivirus,
            size: 768,
            requirements: vec![],
            provides: vec!["antivirus:protection".to_string()],
        };

        engine.add_software(antivirus1);

        assert!(engine.has_conflict(&antivirus2));
    }

    #[tokio::test]
    async fn test_software_version_compatibility() {
        let engine = SoftwareEngine::new();

        let software_v1 = Software {
            id: 1,
            name: "Hasher".to_string(),
            version: "1.0.0".to_string(),
            software_type: SoftwareType::Hasher,
            size: 128,
            requirements: vec![],
            provides: vec![],
        };

        let software_v2 = Software {
            id: 2,
            name: "Hasher".to_string(),
            version: "2.0.0".to_string(),
            software_type: SoftwareType::Hasher,
            size: 256,
            requirements: vec![],
            provides: vec![],
        };

        assert!(engine.is_upgrade(&software_v1, &software_v2));
        assert!(!engine.is_upgrade(&software_v2, &software_v1));
    }
}

#[cfg(test)]
mod network_system_tests {
    use super::*;
    use he_game_mechanics::engine::network_engine::*;
    use std::net::IpAddr;

    #[tokio::test]
    async fn test_network_topology() {
        let mut engine = NetworkEngine::new();

        engine.generate_topology(100);

        let nodes = engine.get_all_nodes();
        assert_eq!(nodes.len(), 100);

        // Test that nodes have valid IPs
        for node in nodes {
            assert!(node.ip_address.parse::<IpAddr>().is_ok());
        }
    }

    #[tokio::test]
    async fn test_connection_establishment() {
        let mut engine = NetworkEngine::new();

        let source = NetworkNode {
            id: 1,
            ip_address: "192.168.1.100".to_string(),
            node_type: NodeType::Player,
            security_level: 50,
            firewall_strength: 75,
            connected_nodes: vec![],
        };

        let target = NetworkNode {
            id: 2,
            ip_address: "10.0.0.50".to_string(),
            node_type: NodeType::NPC,
            security_level: 30,
            firewall_strength: 40,
            connected_nodes: vec![],
        };

        engine.add_node(source);
        engine.add_node(target);

        let connection = engine.establish_connection(1, 2);
        assert!(connection.is_ok());

        let route = engine.find_route(1, 2);
        assert!(!route.is_empty());
    }

    #[tokio::test]
    async fn test_ddos_simulation() {
        let mut engine = NetworkEngine::new();

        let target = NetworkNode {
            id: 1,
            ip_address: "172.16.0.100".to_string(),
            node_type: NodeType::Server,
            security_level: 60,
            firewall_strength: 80,
            connected_nodes: vec![],
        };

        engine.add_node(target);

        let attackers = vec![2, 3, 4, 5, 6]; // 5 attackers
        let result = engine.simulate_ddos(&attackers, 1);

        assert!(result.packets_sent > 0);
        assert!(result.packets_blocked > 0);
        assert!(result.success_rate < 1.0);
    }

    #[tokio::test]
    async fn test_log_generation() {
        let engine = NetworkEngine::new();

        let log = engine.generate_log(
            LogType::Connection,
            "192.168.1.1",
            Some("10.0.0.1"),
            "SSH connection established",
        );

        assert!(log.timestamp > 0);
        assert_eq!(log.log_type, LogType::Connection);
        assert!(log.message.contains("SSH"));
    }
}

#[cfg(test)]
mod mission_system_tests {
    use super::*;
    use he_game_mechanics::missions::*;

    #[tokio::test]
    async fn test_mission_creation() {
        let mission = Mission {
            id: 1,
            name: "First Hack".to_string(),
            description: "Hack into the tutorial server".to_string(),
            mission_type: MissionType::Tutorial,
            objectives: vec![
                Objective {
                    id: 1,
                    description: "Scan the target IP".to_string(),
                    objective_type: ObjectiveType::Scan,
                    target: Some("10.0.0.1".to_string()),
                    completed: false,
                },
                Objective {
                    id: 2,
                    description: "Hack into the server".to_string(),
                    objective_type: ObjectiveType::Hack,
                    target: Some("10.0.0.1".to_string()),
                    completed: false,
                },
            ],
            rewards: Rewards {
                money: 1000,
                experience: 100,
                items: vec![],
            },
            prerequisites: vec![],
            time_limit: None,
            status: MissionStatus::Available,
        };

        assert_eq!(mission.objectives.len(), 2);
        assert_eq!(mission.rewards.money, 1000);
    }

    #[tokio::test]
    async fn test_mission_progression() {
        let mut mission = Mission {
            id: 1,
            name: "Data Theft".to_string(),
            description: "Steal sensitive data".to_string(),
            mission_type: MissionType::Main,
            objectives: vec![
                Objective {
                    id: 1,
                    description: "Download file.txt".to_string(),
                    objective_type: ObjectiveType::Download,
                    target: Some("file.txt".to_string()),
                    completed: false,
                },
            ],
            rewards: Rewards {
                money: 5000,
                experience: 500,
                items: vec!["elite_cracker_v2".to_string()],
            },
            prerequisites: vec![],
            time_limit: Some(3600), // 1 hour
            status: MissionStatus::InProgress,
        };

        // Complete objective
        mission.objectives[0].completed = true;

        // Check if mission is complete
        let all_complete = mission.objectives.iter().all(|o| o.completed);
        if all_complete {
            mission.status = MissionStatus::Completed;
        }

        assert_eq!(mission.status, MissionStatus::Completed);
    }

    #[tokio::test]
    async fn test_mission_prerequisites() {
        let engine = MissionEngine::new();

        let player_missions = vec![1, 2, 3]; // Completed missions

        let new_mission = Mission {
            id: 4,
            name: "Advanced Hacking".to_string(),
            description: "Advanced techniques".to_string(),
            mission_type: MissionType::Side,
            objectives: vec![],
            rewards: Rewards {
                money: 10000,
                experience: 1000,
                items: vec![],
            },
            prerequisites: vec![1, 2], // Requires missions 1 and 2
            time_limit: None,
            status: MissionStatus::Locked,
        };

        let can_start = engine.check_prerequisites(&new_mission, &player_missions);
        assert!(can_start);

        let locked_mission = Mission {
            id: 5,
            name: "Elite Mission".to_string(),
            description: "Only for the best".to_string(),
            mission_type: MissionType::Special,
            objectives: vec![],
            rewards: Rewards {
                money: 50000,
                experience: 5000,
                items: vec![],
            },
            prerequisites: vec![10, 11], // Requires missions not completed
            time_limit: None,
            status: MissionStatus::Locked,
        };

        let cannot_start = engine.check_prerequisites(&locked_mission, &player_missions);
        assert!(!cannot_start);
    }
}

#[cfg(test)]
mod clan_system_tests {
    use super::*;
    use he_game_mechanics::clan::*;

    #[tokio::test]
    async fn test_clan_creation() {
        let clan = Clan {
            id: 1,
            name: "Elite Hackers".to_string(),
            tag: "ELITE".to_string(),
            description: "The best hackers".to_string(),
            leader_id: 100,
            members: vec![100, 101, 102],
            level: 5,
            reputation: 1500,
            treasury: 1000000,
            created_at: Utc::now(),
        };

        assert_eq!(clan.members.len(), 3);
        assert_eq!(clan.leader_id, 100);
        assert!(clan.members.contains(&clan.leader_id));
    }

    #[tokio::test]
    async fn test_clan_war() {
        let mut engine = ClanWarEngine::new();

        let clan1 = Clan {
            id: 1,
            name: "Alpha".to_string(),
            tag: "ALPHA".to_string(),
            description: "First clan".to_string(),
            leader_id: 1,
            members: vec![1, 2, 3, 4, 5],
            level: 10,
            reputation: 5000,
            treasury: 5000000,
            created_at: Utc::now(),
        };

        let clan2 = Clan {
            id: 2,
            name: "Beta".to_string(),
            tag: "BETA".to_string(),
            description: "Second clan".to_string(),
            leader_id: 10,
            members: vec![10, 11, 12],
            level: 7,
            reputation: 3000,
            treasury: 2000000,
            created_at: Utc::now(),
        };

        let war_result = engine.simulate_war(&clan1, &clan2);

        // Clan1 should have advantage due to more members and higher level
        assert!(war_result.winner_id == 1 || war_result.winner_id == 2);
        assert!(war_result.attacker_score > 0);
        assert!(war_result.defender_score > 0);
    }

    #[tokio::test]
    async fn test_clan_contribution() {
        let mut tracker = ContributionTracker::new();

        tracker.add_contribution(1, 100, ContributionType::Money, 50000);
        tracker.add_contribution(1, 101, ContributionType::Mission, 5);
        tracker.add_contribution(1, 102, ContributionType::War, 10);
        tracker.add_contribution(1, 100, ContributionType::Money, 25000);

        let top_contributor = tracker.get_top_contributor(1);
        assert_eq!(top_contributor, Some(100)); // Player 100 contributed most

        let total = tracker.get_total_contribution(1, 100);
        assert_eq!(total, 75000); // 50000 + 25000
    }
}

#[cfg(test)]
mod defense_system_tests {
    use super::*;
    use he_game_mechanics::defense::*;

    #[tokio::test]
    async fn test_firewall_effectiveness() {
        let firewall = Firewall {
            level: 8,
            strength: 85.0,
            rules: vec![
                FirewallRule {
                    rule_type: RuleType::BlockIP,
                    target: "192.168.1.100".to_string(),
                    active: true,
                },
                FirewallRule {
                    rule_type: RuleType::BlockPort,
                    target: "22".to_string(),
                    active: true,
                },
            ],
        };

        let attack = Attack {
            attack_type: AttackType::PortScan,
            source_ip: "192.168.1.100".to_string(),
            target_port: Some(22),
            strength: 60.0,
        };

        let blocked = firewall.can_block(&attack);
        assert!(blocked); // Should block due to IP rule

        let weak_attack = Attack {
            attack_type: AttackType::BruteForce,
            source_ip: "10.0.0.1".to_string(),
            target_port: Some(80),
            strength: 30.0,
        };

        let blocked_weak = firewall.can_block(&weak_attack);
        assert!(blocked_weak); // Should block due to high firewall strength
    }

    #[tokio::test]
    async fn test_intrusion_detection() {
        let mut ids = IntrusionDetectionSystem::new();

        ids.add_pattern(Pattern {
            name: "SQL Injection".to_string(),
            signature: "'; DROP TABLE".to_string(),
            severity: Severity::Critical,
        });

        ids.add_pattern(Pattern {
            name: "XSS Attack".to_string(),
            signature: "<script>".to_string(),
            severity: Severity::High,
        });

        let malicious_request = "'; DROP TABLE users; --";
        let detection = ids.detect(malicious_request);
        assert!(detection.is_some());
        assert_eq!(detection.unwrap().severity, Severity::Critical);

        let safe_request = "SELECT * FROM users WHERE id = 1";
        let no_detection = ids.detect(safe_request);
        assert!(no_detection.is_none());
    }

    #[tokio::test]
    async fn test_security_rating() {
        let engine = SecurityEngine::new();

        let components = SecurityComponents {
            firewall_level: 10,
            antivirus_level: 8,
            ids_active: true,
            encryption_strength: 256,
            password_strength: 90,
            two_factor_enabled: true,
        };

        let rating = engine.calculate_security_rating(&components);
        assert!(rating >= 85.0); // Should have high security rating
        assert!(rating <= 100.0);
    }
}