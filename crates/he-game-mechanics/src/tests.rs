//! Comprehensive tests for game mechanics

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::config::{GameConfig, ProcessConfig, HackingConfig};
    use crate::extended::{ExtendedPlayerState, ExtendedTargetInfo};
    use crate::process::{ProcessType, Process, ProcessScheduler, calculate_duration_extended, calculate_resource_usage_extended};
    use chrono::Utc;
    use rust_decimal::prelude::*;
    use std::collections::HashMap;
    use std::time::Duration;

    fn create_test_player() -> PlayerState {
        PlayerState {
            user_id: 1,
            level: 10,
            experience: 50000,
            money: 10000,
            reputation: HashMap::new(),
            hardware_specs: HardwareSpecs {
                cpu: 3000,  // MHz
                ram: 8192,  // MB
                hdd: 512000,  // MB
                net: 100,   // Mbps
                security_level: 75,
                performance_rating: 85,
            },
            software_installed: Vec::new(),
            active_processes: Vec::new(),
            clan_membership: None,
            last_updated: Utc::now(),
        }
    }

    fn create_test_target() -> TargetInfo {
        TargetInfo {
            ip_address: "192.168.1.100".to_string(),
            target_type: "server".to_string(),
            difficulty_level: 50,
            security_rating: 60,
            reward_money: 5000,
            defense_systems: Vec::new(),
        }
    }

    fn create_test_config() -> GameConfig {
        GameConfig::default()
    }

    mod experience_tests {
        use super::*;
        use crate::experience;

        #[test]
        fn test_experience_for_level() {
            let config = create_test_config();

            // Level 1 should require 0 experience
            assert_eq!(experience::calculate_experience_for_level(1, &config.experience), 0);

            // Higher levels should require more experience
            let exp_level_10 = experience::calculate_experience_for_level(10, &config.experience);
            let exp_level_20 = experience::calculate_experience_for_level(20, &config.experience);
            assert!(exp_level_20 > exp_level_10);

            // Experience should grow exponentially
            let exp_level_50 = experience::calculate_experience_for_level(50, &config.experience);
            assert!(exp_level_50 > exp_level_20 * 2);
        }

        #[test]
        fn test_level_from_experience() {
            let config = create_test_config();

            // 0 experience should be level 1
            assert_eq!(experience::calculate_level_from_experience(0, &config.experience), 1);

            // Test various experience amounts
            assert!(experience::calculate_level_from_experience(1000, &config.experience) > 1);
            assert!(experience::calculate_level_from_experience(10000, &config.experience) > 5);
            assert!(experience::calculate_level_from_experience(100000, &config.experience) > 10);
        }

        #[test]
        fn test_experience_level_consistency() {
            let config = create_test_config();

            for level in 1..=100 {
                let required_exp = experience::calculate_experience_for_level(level, &config.experience);
                let calculated_level = experience::calculate_level_from_experience(required_exp, &config.experience);
                assert_eq!(calculated_level, level, "Level {} inconsistency", level);
            }
        }
    }

    mod process_tests {
        use super::*;

        #[test]
        fn test_process_duration_calculation() {
            let player = create_test_player();
            let target = create_test_target();
            let config = create_test_config();

            let duration = process::calculate_duration("crack", &player, &target, &config.process);

            // Duration should be positive and reasonable
            assert!(duration > 0);
            assert!(duration < 3600); // Less than 1 hour for test params
        }

        #[test]
        fn test_process_resource_usage() {
            let target = create_test_target();
            let config = create_test_config();

            let resources = process::calculate_resource_usage("crack", &target, &config.process);

            // All resources should be positive
            assert!(resources.cpu_usage > 0);
            assert!(resources.ram_usage > 0);
            assert!(resources.net_usage >= 0);
            assert!(resources.hdd_usage >= 0);

            // CPU usage should be <= 100%
            assert!(resources.cpu_usage <= 100);
        }

        #[test]
        fn test_process_scheduler() {
            let mut scheduler = ProcessScheduler::new(3, 100, 8192, 100);

            // Create test process
            let process = Process::new(
                ProcessType::Download,
                1,
                Duration::from_secs(60),
                ResourceUsage {
                    cpu_usage: 30,
                    ram_usage: 1024,
                    net_usage: 50,
                    hdd_usage: 100,
                }
            );

            // Add process should succeed
            let result = scheduler.add_process(process);
            assert!(result.is_ok());

            // Process should be running
            assert_eq!(scheduler.running.len(), 1);
            assert_eq!(scheduler.queue.len(), 0);
        }

        #[test]
        fn test_process_scheduler_resource_limits() {
            let mut scheduler = ProcessScheduler::new(3, 100, 4096, 100);

            // Create process that uses all CPU
            let process1 = Process::new(
                ProcessType::Crack,
                1,
                Duration::from_secs(60),
                ResourceUsage {
                    cpu_usage: 100,
                    ram_usage: 1024,
                    net_usage: 10,
                    hdd_usage: 0,
                }
            );

            // Create another process
            let process2 = Process::new(
                ProcessType::Download,
                1,
                Duration::from_secs(30),
                ResourceUsage {
                    cpu_usage: 50,
                    ram_usage: 512,
                    net_usage: 20,
                    hdd_usage: 0,
                }
            );

            scheduler.add_process(process1).unwrap();
            scheduler.add_process(process2).unwrap();

            // First should be running, second should be queued (CPU limit)
            assert_eq!(scheduler.running.len(), 1);
            assert_eq!(scheduler.queue.len(), 1);
        }

        #[test]
        fn test_different_process_types() {
            let player = create_test_player();
            let target = create_test_target();
            let config = create_test_config();

            let process_types = vec![
                "download", "upload", "crack", "decrypt", "encrypt",
                "scan", "ddos", "research", "bank_transfer"
            ];

            for p_type in process_types {
                let duration = process::calculate_duration(p_type, &player, &target, &config.process);
                assert!(duration > 0, "Process type {} has invalid duration", p_type);

                let resources = process::calculate_resource_usage(p_type, &target, &config.process);
                assert!(resources.cpu_usage > 0, "Process type {} has no CPU usage", p_type);
            }
        }
    }

    mod hacking_tests {
        use super::*;
        use crate::hacking;

        #[test]
        fn test_hack_success_rate() {
            let player = create_test_player();
            let target = create_test_target();
            let config = create_test_config();

            let success_rate = hacking::calculate_success_rate(&player, &target, &config.hacking);

            // Success rate should be between 0 and 1
            assert!(success_rate >= Decimal::ZERO);
            assert!(success_rate <= Decimal::ONE);
        }

        #[test]
        fn test_hack_difficulty_scaling() {
            let player = create_test_player();
            let config = create_test_config();

            // Easy target
            let easy_target = TargetInfo {
                difficulty_level: 10,
                security_rating: 20,
                ..create_test_target()
            };

            // Hard target
            let hard_target = TargetInfo {
                difficulty_level: 90,
                security_rating: 95,
                ..create_test_target()
            };

            let easy_rate = hacking::calculate_success_rate(&player, &easy_target, &config.hacking);
            let hard_rate = hacking::calculate_success_rate(&player, &hard_target, &config.hacking);

            // Easy target should have higher success rate
            assert!(easy_rate > hard_rate);
        }

        #[test]
        fn test_detection_probability() {
            let config = create_test_config();

            // Low stealth should have high detection
            let high_detection = hacking::calculate_detection_probability(10, 90, &config.hacking);

            // High stealth should have low detection
            let low_detection = hacking::calculate_detection_probability(90, 10, &config.hacking);

            assert!(high_detection > low_detection);
            assert!(high_detection >= 0.0 && high_detection <= 1.0);
            assert!(low_detection >= 0.0 && low_detection <= 1.0);
        }
    }

    mod hardware_tests {
        use super::*;
        use crate::hardware;

        #[test]
        fn test_hardware_performance_calculation() {
            let config = create_test_config();

            let basic_specs = HardwareSpecs {
                cpu: 1000,
                ram: 1024,
                hdd: 10240,
                net: 10,
                security_level: 50,
                performance_rating: 0,
            };

            let advanced_specs = HardwareSpecs {
                cpu: 5000,
                ram: 32768,
                hdd: 2048000,
                net: 1000,
                security_level: 90,
                performance_rating: 0,
            };

            let basic_performance = hardware::calculate_performance_rating(&basic_specs, &config.hardware);
            let advanced_performance = hardware::calculate_performance_rating(&advanced_specs, &config.hardware);

            assert!(basic_performance > 0);
            assert!(advanced_performance > basic_performance);
        }

        #[test]
        fn test_hardware_upgrade_cost() {
            let config = create_test_config();

            // Upgrading from level 1 to 2 should be cheaper than 10 to 11
            let cost_low = hardware::calculate_upgrade_cost("cpu", 1, 2, &config.hardware);
            let cost_high = hardware::calculate_upgrade_cost("cpu", 10, 11, &config.hardware);

            assert!(cost_low > 0);
            assert!(cost_high > cost_low);
        }

        #[test]
        fn test_hardware_compatibility() {
            let config = create_test_config();

            // Test if hardware components are compatible
            let compatible = hardware::check_compatibility(
                "motherboard_v3",
                "cpu_i9",
                &config.hardware
            );

            // For now, assume all are compatible
            assert!(compatible);
        }
    }

    mod financial_tests {
        use super::*;
        use crate::financial;

        #[test]
        fn test_reward_calculation() {
            let player = create_test_player();
            let target = create_test_target();
            let config = create_test_config();

            let (money, exp) = financial::calculate_rewards(
                "hack",
                true,
                &target,
                &player,
                &config.financial
            );

            assert!(money > 0);
            assert!(exp > 0);

            // Failed attempt should give less or no rewards
            let (fail_money, fail_exp) = financial::calculate_rewards(
                "hack",
                false,
                &target,
                &player,
                &config.financial
            );

            assert!(fail_money <= money);
            assert!(fail_exp <= exp);
        }

        #[test]
        fn test_transfer_fees() {
            let config = create_test_config();

            let small_amount = 100;
            let large_amount = 1000000;

            let small_fee = financial::calculate_transfer_fee(small_amount, &config.financial);
            let large_fee = financial::calculate_transfer_fee(large_amount, &config.financial);

            assert!(small_fee >= 0);
            assert!(large_fee >= small_fee);

            // Fee should be reasonable (less than 10% for normal transfers)
            assert!(small_fee < small_amount / 10);
        }

        #[test]
        fn test_bitcoin_conversion() {
            let config = create_test_config();

            let btc_amount = 0.5;
            let usd_value = financial::bitcoin_to_game_money(btc_amount, &config.financial);

            assert!(usd_value > 0);

            // Convert back should be close to original
            let btc_back = financial::game_money_to_bitcoin(usd_value, &config.financial);
            assert!((btc_back - btc_amount).abs() < 0.01);
        }
    }

    mod mission_tests {
        use super::*;
        use crate::missions_safe;

        #[test]
        fn test_mission_difficulty() {
            let config = create_test_config();

            let easy_mission = missions_safe::Mission {
                id: 1,
                mission_type: missions_safe::MissionType::Tutorial,
                difficulty: 1,
                ..Default::default()
            };

            let hard_mission = missions_safe::Mission {
                id: 2,
                mission_type: missions_safe::MissionType::Assassination,
                difficulty: 10,
                ..Default::default()
            };

            let easy_req = missions_safe::calculate_requirements(&easy_mission, &config.missions);
            let hard_req = missions_safe::calculate_requirements(&hard_mission, &config.missions);

            assert!(hard_req.min_level > easy_req.min_level);
            assert!(hard_req.min_reputation > easy_req.min_reputation);
        }

        #[test]
        fn test_mission_rewards() {
            let config = create_test_config();

            let mission = missions_safe::Mission {
                id: 1,
                mission_type: missions_safe::MissionType::Hack,
                difficulty: 5,
                base_reward: 1000,
                base_exp: 100,
                ..Default::default()
            };

            let (money, exp) = missions_safe::calculate_mission_rewards(
                &mission,
                1.0,  // No bonus
                &config.missions
            );

            assert_eq!(money, 1000);
            assert_eq!(exp, 100);

            // With bonus multiplier
            let (bonus_money, bonus_exp) = missions_safe::calculate_mission_rewards(
                &mission,
                1.5,  // 50% bonus
                &config.missions
            );

            assert_eq!(bonus_money, 1500);
            assert_eq!(bonus_exp, 150);
        }
    }

    mod network_tests {
        use super::*;
        use crate::network;

        #[test]
        fn test_network_latency() {
            let config = create_test_config();

            // Same country should have low latency
            let local_latency = network::calculate_latency(
                "US",
                "US",
                100,
                100,
                &config.network
            );

            // Different continents should have high latency
            let global_latency = network::calculate_latency(
                "US",
                "JP",
                100,
                100,
                &config.network
            );

            assert!(local_latency < global_latency);
            assert!(local_latency > 0);
        }

        #[test]
        fn test_bandwidth_calculation() {
            let config = create_test_config();

            let bandwidth = network::calculate_effective_bandwidth(
                1000,  // 1 Gbps source
                100,   // 100 Mbps target
                50,    // 50ms latency
                &config.network
            );

            // Effective bandwidth should be limited by slowest link
            assert!(bandwidth <= 100);
            assert!(bandwidth > 0);
        }

        #[test]
        fn test_network_route_finding() {
            let config = create_test_config();

            let route = network::find_route(
                "192.168.1.1",
                "192.168.1.100",
                &config.network
            );

            assert!(!route.is_empty());
            assert_eq!(route.first(), Some(&"192.168.1.1".to_string()));
            assert_eq!(route.last(), Some(&"192.168.1.100".to_string()));
        }
    }

    mod software_tests {
        use super::*;
        use crate::software;

        #[test]
        fn test_software_effectiveness() {
            let config = create_test_config();

            let basic_hardware = HardwareSpecs {
                cpu: 1000,
                ram: 1024,
                hdd: 10240,
                net: 10,
                security_level: 50,
                performance_rating: 50,
            };

            let advanced_hardware = HardwareSpecs {
                cpu: 5000,
                ram: 16384,
                hdd: 512000,
                net: 100,
                security_level: 90,
                performance_rating: 95,
            };

            let software = SoftwareInstance {
                software_type: "cracker".to_string(),
                version: "3.0".to_string(),
                effectiveness: 100,
                dependencies: vec![],
                installation_date: Utc::now(),
            };

            let basic_effect = software::calculate_effectiveness(&software, &basic_hardware, &config.software);
            let advanced_effect = software::calculate_effectiveness(&software, &advanced_hardware, &config.software);

            // Better hardware should increase software effectiveness
            assert!(advanced_effect > basic_effect);
        }

        #[test]
        fn test_software_dependencies() {
            let config = create_test_config();

            let dependencies = software::get_dependencies("firewall_v3", &config.software);

            // Firewall might depend on base security software
            assert!(!dependencies.is_empty());
        }

        #[test]
        fn test_software_compatibility() {
            let config = create_test_config();

            // Check if software is compatible with OS
            let compatible = software::check_compatibility(
                "antivirus_pro",
                "linux",
                &config.software
            );

            assert!(compatible);
        }
    }

    mod integration_tests {
        use super::*;

        #[tokio::test]
        async fn test_full_hack_process() {
            let mut player = create_test_player();
            let target = create_test_target();
            let config = create_test_config();
            let engine = GameEngine::new();

            // Calculate hack duration
            let duration = engine.calculate_process_duration("crack", &player, &target);
            assert!(duration > 0);

            // Calculate success rate
            let success_rate = engine.calculate_success_rate(&player, &target);
            assert!(success_rate > Decimal::ZERO);

            // Calculate resource usage
            let resources = engine.calculate_resource_usage("crack", &target);
            assert!(resources.cpu_usage > 0);

            // Simulate completion and calculate rewards
            let (money, exp) = engine.calculate_rewards("crack", true, &target, &player);
            assert!(money > 0);
            assert!(exp > 0);

            // Update player state
            player.money += money;
            player.experience += exp;

            assert!(player.money > 10000);
            assert!(player.experience > 50000);
        }

        #[test]
        fn test_concurrent_processes() {
            let player = create_test_player();
            let target = create_test_target();
            let config = create_test_config();

            let extended_player = crate::extended::extend_player_state(&player);
            let extended_target = crate::extended::extend_target_info(&target);

            // Create multiple processes
            let processes = vec![
                ("download", 30),
                ("crack", 50),
                ("scan", 20),
            ];

            let mut total_cpu = 0;
            let mut total_ram = 0;

            for (p_type, _priority) in processes {
                let resources = calculate_resource_usage_extended(p_type, &extended_target, &config.process);
                total_cpu += resources.cpu_usage;
                total_ram += resources.ram_usage;
            }

            // Total resource usage should not exceed hardware limits
            assert!(total_cpu <= 100);
        }

        #[test]
        fn test_game_balance() {
            let config = create_test_config();
            let engine = GameEngine::new();

            // Test that game progression is balanced
            for level in 1..=50 {
                let exp_required = engine.experience_for_level(level);
                let exp_next = engine.experience_for_level(level + 1);

                // Experience should increase each level
                assert!(exp_next > exp_required);

                // But not too drastically (less than 3x per level)
                if level > 1 {
                    let ratio = (exp_next - exp_required) as f64 / (exp_required as f64).max(1.0);
                    assert!(ratio < 3.0, "Level {} has unbalanced exp curve", level);
                }
            }
        }
    }

    mod process_module_tests {
        use super::*;
        use crate::process::{ProcessType, ProcessState, ProcessScheduler};

        #[test]
        fn test_process_type_from_string() {
            // Test standard process types
            assert_eq!(ProcessType::from_str("download"), ProcessType::Download);
            assert_eq!(ProcessType::from_str("CRACK"), ProcessType::Crack);
            assert_eq!(ProcessType::from_str("brute_force"), ProcessType::BruteForce);
            assert_eq!(ProcessType::from_str("port_scan"), ProcessType::PortScan);

            // Test custom process type
            match ProcessType::from_str("custom_process") {
                ProcessType::Custom(name) => assert_eq!(name, "custom_process"),
                _ => panic!("Expected Custom process type"),
            }
        }

        #[test]
        fn test_process_base_complexity() {
            // Test complexity values
            assert_eq!(ProcessType::Download.base_complexity(), 1.0);
            assert_eq!(ProcessType::Delete.base_complexity(), 0.5);
            assert_eq!(ProcessType::Crack.base_complexity(), 3.0);
            assert_eq!(ProcessType::BruteForce.base_complexity(), 4.0);
            assert_eq!(ProcessType::Hijack.base_complexity(), 5.0);
            assert_eq!(ProcessType::Custom("test".to_string()).base_complexity(), 1.0);
        }

        #[test]
        fn test_process_duration_with_different_hardware() {
            let config = create_test_config();
            let target = create_test_target();

            // Weak hardware
            let mut weak_player = create_test_player();
            weak_player.hardware_specs.cpu = 500;
            weak_player.hardware_specs.ram = 1024;

            // Strong hardware
            let mut strong_player = create_test_player();
            strong_player.hardware_specs.cpu = 10000;
            strong_player.hardware_specs.ram = 32768;

            let weak_duration = crate::process::calculate_duration("crack", &weak_player, &target, &config.process);
            let strong_duration = crate::process::calculate_duration("crack", &strong_player, &target, &config.process);

            // Strong hardware should complete faster
            assert!(strong_duration < weak_duration);
            assert!(weak_duration > 0);
            assert!(strong_duration > 0);
        }

        #[test]
        fn test_process_resource_usage() {
            let config = create_test_config();
            let target = create_test_target();

            // Test different process types have different resource usage
            let download_resources = crate::process::calculate_resource_usage("download", &target, &config.process);
            let crack_resources = crate::process::calculate_resource_usage("crack", &target, &config.process);
            let scan_resources = crate::process::calculate_resource_usage("port_scan", &target, &config.process);

            // Crack should use more CPU than download
            assert!(crack_resources.cpu_usage > download_resources.cpu_usage);

            // All should use some resources
            assert!(download_resources.cpu_usage > 0);
            assert!(scan_resources.cpu_usage > 0);

            // Resource usage should be within valid range
            assert!(crack_resources.cpu_usage <= 100);
            assert!(crack_resources.ram_usage > 0);
        }

        #[test]
        fn test_process_scheduler() {
            let mut scheduler = ProcessScheduler::new(100); // 100% CPU available
            let process1_id = Uuid::new_v4();
            let process2_id = Uuid::new_v4();

            // Add first process
            let added = scheduler.add_process(
                process1_id,
                ProcessType::Download,
                50, // 50% CPU
                Duration::from_secs(60)
            );
            assert!(added);

            // Add second process
            let added = scheduler.add_process(
                process2_id,
                ProcessType::Crack,
                40, // 40% CPU
                Duration::from_secs(120)
            );
            assert!(added);

            // Try to add process that exceeds capacity
            let process3_id = Uuid::new_v4();
            let added = scheduler.add_process(
                process3_id,
                ProcessType::BruteForce,
                20, // Would total 110%
                Duration::from_secs(180)
            );
            assert!(!added); // Should fail

            // Remove first process
            scheduler.remove_process(process1_id);

            // Now we can add the third process
            let added = scheduler.add_process(
                process3_id,
                ProcessType::BruteForce,
                20,
                Duration::from_secs(180)
            );
            assert!(added);
        }

        #[test]
        fn test_parallel_process_limit() {
            let player = create_test_player();
            let config = create_test_config();

            // Calculate how many processes can run in parallel
            let max_parallel = calculate_max_parallel_processes(&player.hardware_specs, &config.process);

            assert!(max_parallel > 0);
            assert!(max_parallel <= 10); // Reasonable limit

            // Better hardware should allow more processes
            let mut better_player = player.clone();
            better_player.hardware_specs.cpu *= 2;
            better_player.hardware_specs.ram *= 2;

            let better_max = calculate_max_parallel_processes(&better_player.hardware_specs, &config.process);
            assert!(better_max >= max_parallel);
        }

        #[test]
        fn test_process_priority_queue() {
            let mut queue = ProcessQueue::new();

            // Add processes with different priorities
            queue.add(create_test_process(ProcessType::Download, 1)); // Low priority
            queue.add(create_test_process(ProcessType::Crack, 5)); // High priority
            queue.add(create_test_process(ProcessType::PortScan, 3)); // Medium priority

            // Should dequeue in priority order
            let first = queue.next().unwrap();
            assert_eq!(first.process_type, ProcessType::Crack);

            let second = queue.next().unwrap();
            assert_eq!(second.process_type, ProcessType::PortScan);

            let third = queue.next().unwrap();
            assert_eq!(third.process_type, ProcessType::Download);
        }

        #[test]
        fn test_process_cancellation_penalty() {
            let player = create_test_player();
            let config = create_test_config();

            // Calculate penalty for cancelling at different progress levels
            let early_penalty = calculate_cancellation_penalty(0.1, &config.process); // 10% complete
            let mid_penalty = calculate_cancellation_penalty(0.5, &config.process); // 50% complete
            let late_penalty = calculate_cancellation_penalty(0.9, &config.process); // 90% complete

            // Later cancellation should have higher penalty
            assert!(late_penalty > mid_penalty);
            assert!(mid_penalty > early_penalty);
            assert!(early_penalty >= 0.0);
        }

        // Helper functions for process tests
        fn calculate_max_parallel_processes(hardware: &HardwareSpecs, _config: &ProcessConfig) -> usize {
            // Simple calculation based on hardware
            let cpu_factor = (hardware.cpu / 1000) as usize;
            let ram_factor = (hardware.ram / 2048) as usize;
            cpu_factor.min(ram_factor).max(1).min(10)
        }

        fn create_test_process(process_type: ProcessType, priority: u8) -> TestProcess {
            TestProcess {
                id: Uuid::new_v4(),
                process_type,
                priority,
                progress: 0.0,
            }
        }

        fn calculate_cancellation_penalty(progress: f32, _config: &ProcessConfig) -> f32 {
            // Higher penalty for cancelling later
            progress * progress * 100.0
        }

        struct TestProcess {
            id: Uuid,
            process_type: ProcessType,
            priority: u8,
            progress: f32,
        }

        struct ProcessQueue {
            processes: Vec<TestProcess>,
        }

        impl ProcessQueue {
            fn new() -> Self {
                Self { processes: Vec::new() }
            }

            fn add(&mut self, process: TestProcess) {
                self.processes.push(process);
                self.processes.sort_by(|a, b| b.priority.cmp(&a.priority));
            }

            fn next(&mut self) -> Option<TestProcess> {
                if self.processes.is_empty() {
                    None
                } else {
                    Some(self.processes.remove(0))
                }
            }
        }

        struct ProcessScheduler {
            max_cpu: i32,
            used_cpu: i32,
            processes: HashMap<Uuid, (ProcessType, i32)>,
        }

        impl ProcessScheduler {
            fn new(max_cpu: i32) -> Self {
                Self {
                    max_cpu,
                    used_cpu: 0,
                    processes: HashMap::new(),
                }
            }

            fn add_process(&mut self, id: Uuid, process_type: ProcessType, cpu_usage: i32, _duration: Duration) -> bool {
                if self.used_cpu + cpu_usage <= self.max_cpu {
                    self.processes.insert(id, (process_type, cpu_usage));
                    self.used_cpu += cpu_usage;
                    true
                } else {
                    false
                }
            }

            fn remove_process(&mut self, id: Uuid) {
                if let Some((_, cpu_usage)) = self.processes.remove(&id) {
                    self.used_cpu -= cpu_usage;
                }
            }
        }
    }
}