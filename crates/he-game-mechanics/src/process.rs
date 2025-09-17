//! Process system mechanics - Time calculations, resource management, scheduling

use crate::config::ProcessConfig;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Process types in the game
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProcessType {
    Download,
    Upload,
    Delete,
    Install,
    Uninstall,
    Crack,
    Decrypt,
    Encrypt,
    HideLog,
    DeleteLog,
    BruteForce,
    PortScan,
    SystemScan,
    VirusScan,
    AntiVirusRun,
    FirewallAnalysis,
    DDoSAttack,
    Hijack,
    Research,
    Upgrade,
    BankTransfer,
    BitcoinMine,
    BitcoinTransfer,
    MissionTask,
    Custom(String),
}

impl ProcessType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "download" => ProcessType::Download,
            "upload" => ProcessType::Upload,
            "delete" => ProcessType::Delete,
            "install" => ProcessType::Install,
            "uninstall" => ProcessType::Uninstall,
            "crack" => ProcessType::Crack,
            "decrypt" => ProcessType::Decrypt,
            "encrypt" => ProcessType::Encrypt,
            "hidelog" | "hide_log" => ProcessType::HideLog,
            "deletelog" | "delete_log" => ProcessType::DeleteLog,
            "bruteforce" | "brute_force" => ProcessType::BruteForce,
            "portscan" | "port_scan" => ProcessType::PortScan,
            "systemscan" | "system_scan" => ProcessType::SystemScan,
            "virusscan" | "virus_scan" => ProcessType::VirusScan,
            "antivirus" | "anti_virus_run" => ProcessType::AntiVirusRun,
            "firewall_analysis" => ProcessType::FirewallAnalysis,
            "ddos" | "ddos_attack" => ProcessType::DDoSAttack,
            "hijack" => ProcessType::Hijack,
            "research" => ProcessType::Research,
            "upgrade" => ProcessType::Upgrade,
            "bank_transfer" => ProcessType::BankTransfer,
            "bitcoin_mine" => ProcessType::BitcoinMine,
            "bitcoin_transfer" => ProcessType::BitcoinTransfer,
            "mission" | "mission_task" => ProcessType::MissionTask,
            other => ProcessType::Custom(other.to_string()),
        }
    }
    
    pub fn base_complexity(&self) -> f32 {
        match self {
            ProcessType::Download => 1.0,
            ProcessType::Upload => 1.0,
            ProcessType::Delete => 0.5,
            ProcessType::Install => 1.5,
            ProcessType::Uninstall => 0.8,
            ProcessType::Crack => 3.0,
            ProcessType::Decrypt => 2.5,
            ProcessType::Encrypt => 2.0,
            ProcessType::HideLog => 1.2,
            ProcessType::DeleteLog => 0.3,
            ProcessType::BruteForce => 4.0,
            ProcessType::PortScan => 1.5,
            ProcessType::SystemScan => 2.0,
            ProcessType::VirusScan => 1.8,
            ProcessType::AntiVirusRun => 2.2,
            ProcessType::FirewallAnalysis => 2.5,
            ProcessType::DDoSAttack => 3.5,
            ProcessType::Hijack => 5.0,
            ProcessType::Research => 3.0,
            ProcessType::Upgrade => 2.0,
            ProcessType::BankTransfer => 1.5,
            ProcessType::BitcoinMine => 4.0,
            ProcessType::BitcoinTransfer => 1.0,
            ProcessType::MissionTask => 2.0,
            ProcessType::Custom(_) => 1.0,
        }
    }
}

/// Process priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl ProcessPriority {
    pub fn speed_multiplier(&self) -> f32 {
        match self {
            ProcessPriority::Low => 0.5,
            ProcessPriority::Normal => 1.0,
            ProcessPriority::High => 1.5,
            ProcessPriority::Critical => 2.0,
        }
    }
    
    pub fn resource_multiplier(&self) -> f32 {
        match self {
            ProcessPriority::Low => 0.7,
            ProcessPriority::Normal => 1.0,
            ProcessPriority::High => 1.3,
            ProcessPriority::Critical => 1.6,
        }
    }
}

/// Process state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessState {
    Queued,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Individual process instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    pub id: Uuid,
    pub process_type: ProcessType,
    pub priority: ProcessPriority,
    pub state: ProcessState,
    pub player_id: i32,
    pub target_id: Option<i32>,
    pub target_ip: Option<String>,
    pub file_id: Option<i32>,
    pub started_at: Option<SystemTime>,
    pub paused_at: Option<SystemTime>,
    pub completed_at: Option<SystemTime>,
    pub total_duration: Duration,
    pub elapsed_duration: Duration,
    pub resource_usage: ResourceUsage,
    pub metadata: HashMap<String, String>,
    pub error_message: Option<String>,
    pub parent_process_id: Option<Uuid>,
    pub child_process_ids: Vec<Uuid>,
}

impl Process {
    pub fn new(
        process_type: ProcessType,
        player_id: i32,
        total_duration: Duration,
        resource_usage: ResourceUsage,
    ) -> Self {
        Process {
            id: Uuid::new_v4(),
            process_type,
            priority: ProcessPriority::Normal,
            state: ProcessState::Queued,
            player_id,
            target_id: None,
            target_ip: None,
            file_id: None,
            started_at: None,
            paused_at: None,
            completed_at: None,
            total_duration,
            elapsed_duration: Duration::from_secs(0),
            resource_usage,
            metadata: HashMap::new(),
            error_message: None,
            parent_process_id: None,
            child_process_ids: Vec::new(),
        }
    }
    
    pub fn with_target(mut self, target_id: i32, target_ip: String) -> Self {
        self.target_id = Some(target_id);
        self.target_ip = Some(target_ip);
        self
    }
    
    pub fn with_file(mut self, file_id: i32) -> Self {
        self.file_id = Some(file_id);
        self
    }
    
    pub fn with_priority(mut self, priority: ProcessPriority) -> Self {
        self.priority = priority;
        self
    }
    
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    pub fn start(&mut self) {
        self.state = ProcessState::Running;
        self.started_at = Some(SystemTime::now());
    }
    
    pub fn pause(&mut self) {
        if self.state == ProcessState::Running {
            self.state = ProcessState::Paused;
            self.paused_at = Some(SystemTime::now());
            if let Some(started) = self.started_at {
                if let Ok(elapsed) = SystemTime::now().duration_since(started) {
                    self.elapsed_duration += elapsed;
                }
            }
        }
    }
    
    pub fn resume(&mut self) {
        if self.state == ProcessState::Paused {
            self.state = ProcessState::Running;
            self.started_at = Some(SystemTime::now());
            self.paused_at = None;
        }
    }
    
    pub fn complete(&mut self) {
        self.state = ProcessState::Completed;
        self.completed_at = Some(SystemTime::now());
        if let Some(started) = self.started_at {
            if let Ok(elapsed) = SystemTime::now().duration_since(started) {
                self.elapsed_duration += elapsed;
            }
        }
    }
    
    pub fn fail(&mut self, error: String) {
        self.state = ProcessState::Failed;
        self.error_message = Some(error);
        self.completed_at = Some(SystemTime::now());
    }
    
    pub fn cancel(&mut self) {
        self.state = ProcessState::Cancelled;
        self.completed_at = Some(SystemTime::now());
    }
    
    pub fn get_progress(&self) -> f32 {
        if self.total_duration.as_secs() == 0 {
            return 100.0;
        }
        
        let current_elapsed = if self.state == ProcessState::Running {
            if let Some(started) = self.started_at {
                if let Ok(elapsed) = SystemTime::now().duration_since(started) {
                    self.elapsed_duration + elapsed
                } else {
                    self.elapsed_duration
                }
            } else {
                self.elapsed_duration
            }
        } else {
            self.elapsed_duration
        };
        
        let progress = (current_elapsed.as_secs() as f32 / self.total_duration.as_secs() as f32) * 100.0;
        progress.min(100.0)
    }
    
    pub fn get_remaining_time(&self) -> Duration {
        if self.state != ProcessState::Running {
            return self.total_duration.saturating_sub(self.elapsed_duration);
        }
        
        let current_elapsed = if let Some(started) = self.started_at {
            if let Ok(elapsed) = SystemTime::now().duration_since(started) {
                self.elapsed_duration + elapsed
            } else {
                self.elapsed_duration
            }
        } else {
            self.elapsed_duration
        };
        
        self.total_duration.saturating_sub(current_elapsed)
    }
    
    pub fn is_complete(&self) -> bool {
        self.state == ProcessState::Completed
    }
    
    pub fn is_active(&self) -> bool {
        matches!(self.state, ProcessState::Running | ProcessState::Paused)
    }
}

/// Process scheduler for managing multiple processes
#[derive(Debug, Clone)]
pub struct ProcessScheduler {
    pub processes: HashMap<Uuid, Process>,
    pub queue: VecDeque<Uuid>,
    pub running: Vec<Uuid>,
    pub max_concurrent: usize,
    pub total_cpu: i32,
    pub total_ram: i32,
    pub total_net: i32,
    pub used_cpu: i32,
    pub used_ram: i32,
    pub used_net: i32,
}

impl ProcessScheduler {
    pub fn new(max_concurrent: usize, total_cpu: i32, total_ram: i32, total_net: i32) -> Self {
        ProcessScheduler {
            processes: HashMap::new(),
            queue: VecDeque::new(),
            running: Vec::new(),
            max_concurrent,
            total_cpu,
            total_ram,
            total_net,
            used_cpu: 0,
            used_ram: 0,
            used_net: 0,
        }
    }
    
    pub fn add_process(&mut self, process: Process) -> Result<Uuid, String> {
        // Check if resources are available
        if !self.can_allocate_resources(&process.resource_usage) {
            return Err("Insufficient resources to queue process".to_string());
        }
        
        let process_id = process.id;
        self.processes.insert(process_id, process);
        self.queue.push_back(process_id);
        
        // Try to start processes from queue
        self.schedule_next();
        
        Ok(process_id)
    }
    
    pub fn schedule_next(&mut self) {
        while self.running.len() < self.max_concurrent && !self.queue.is_empty() {
            if let Some(process_id) = self.queue.pop_front() {
                if let Some(process) = self.processes.get_mut(&process_id) {
                    if self.allocate_resources(&process.resource_usage) {
                        process.start();
                        self.running.push(process_id);
                    } else {
                        // Put back in queue if can't allocate resources
                        self.queue.push_front(process_id);
                        break;
                    }
                }
            }
        }
    }
    
    pub fn can_allocate_resources(&self, usage: &ResourceUsage) -> bool {
        self.used_cpu + usage.cpu_usage <= self.total_cpu &&
        self.used_ram + usage.ram_usage <= self.total_ram &&
        self.used_net + usage.net_usage <= self.total_net
    }
    
    pub fn allocate_resources(&mut self, usage: &ResourceUsage) -> bool {
        if self.can_allocate_resources(usage) {
            self.used_cpu += usage.cpu_usage;
            self.used_ram += usage.ram_usage;
            self.used_net += usage.net_usage;
            true
        } else {
            false
        }
    }
    
    pub fn free_resources(&mut self, usage: &ResourceUsage) {
        self.used_cpu = (self.used_cpu - usage.cpu_usage).max(0);
        self.used_ram = (self.used_ram - usage.ram_usage).max(0);
        self.used_net = (self.used_net - usage.net_usage).max(0);
    }
    
    pub fn pause_process(&mut self, process_id: Uuid) -> Result<(), String> {
        if let Some(process) = self.processes.get_mut(&process_id) {
            process.pause();
            self.free_resources(&process.resource_usage);
            self.running.retain(|&id| id != process_id);
            self.schedule_next();
            Ok(())
        } else {
            Err("Process not found".to_string())
        }
    }
    
    pub fn resume_process(&mut self, process_id: Uuid) -> Result<(), String> {
        if let Some(process) = self.processes.get_mut(&process_id) {
            if self.running.len() < self.max_concurrent && self.allocate_resources(&process.resource_usage) {
                process.resume();
                self.running.push(process_id);
                Ok(())
            } else {
                // Add to queue if can't resume immediately
                self.queue.push_back(process_id);
                Ok(())
            }
        } else {
            Err("Process not found".to_string())
        }
    }
    
    pub fn cancel_process(&mut self, process_id: Uuid) -> Result<(), String> {
        if let Some(process) = self.processes.get_mut(&process_id) {
            if process.is_active() {
                self.free_resources(&process.resource_usage);
                self.running.retain(|&id| id != process_id);
            }
            process.cancel();
            self.queue.retain(|&id| id != process_id);
            self.schedule_next();
            Ok(())
        } else {
            Err("Process not found".to_string())
        }
    }
    
    pub fn complete_process(&mut self, process_id: Uuid) -> Result<(), String> {
        if let Some(process) = self.processes.get_mut(&process_id) {
            if process.is_active() {
                self.free_resources(&process.resource_usage);
                self.running.retain(|&id| id != process_id);
            }
            process.complete();
            self.schedule_next();
            Ok(())
        } else {
            Err("Process not found".to_string())
        }
    }
    
    pub fn update_processes(&mut self) {
        let mut completed = Vec::new();
        
        for &process_id in &self.running {
            if let Some(process) = self.processes.get(&process_id) {
                if process.get_remaining_time().as_secs() == 0 {
                    completed.push(process_id);
                }
            }
        }
        
        for process_id in completed {
            let _ = self.complete_process(process_id);
        }
    }
    
    pub fn get_process(&self, process_id: &Uuid) -> Option<&Process> {
        self.processes.get(process_id)
    }
    
    pub fn get_running_processes(&self) -> Vec<&Process> {
        self.running.iter()
            .filter_map(|id| self.processes.get(id))
            .collect()
    }
    
    pub fn get_queued_processes(&self) -> Vec<&Process> {
        self.queue.iter()
            .filter_map(|id| self.processes.get(id))
            .collect()
    }
    
    pub fn get_resource_usage(&self) -> (i32, i32, i32) {
        (self.used_cpu, self.used_ram, self.used_net)
    }
    
    pub fn get_resource_percentage(&self) -> (f32, f32, f32) {
        let cpu_pct = if self.total_cpu > 0 {
            (self.used_cpu as f32 / self.total_cpu as f32) * 100.0
        } else {
            0.0
        };
        
        let ram_pct = if self.total_ram > 0 {
            (self.used_ram as f32 / self.total_ram as f32) * 100.0
        } else {
            0.0
        };
        
        let net_pct = if self.total_net > 0 {
            (self.used_net as f32 / self.total_net as f32) * 100.0
        } else {
            0.0
        };
        
        (cpu_pct, ram_pct, net_pct)
    }
}

/// Calculate process duration based on type, player stats, and target
pub fn calculate_duration(process_type: &str, player: &crate::PlayerState, target: &crate::TargetInfo, config: &ProcessConfig) -> i32 {
    // Convert to extended versions for full calculation support
    let extended_player = crate::extended::extend_player_state(player);
    let extended_target = crate::extended::extend_target_info(target);
    calculate_duration_extended(process_type, &extended_player, &extended_target, config)
}

/// Calculate process duration with extended data structures
pub fn calculate_duration_extended(process_type: &str, player: &crate::extended::ExtendedPlayerState, target: &crate::extended::ExtendedTargetInfo, config: &ProcessConfig) -> i32 {
    let p_type = ProcessType::from_str(process_type);
    let base_complexity = p_type.base_complexity();
    
    // Base duration calculation
    let base_duration = match &p_type {
        ProcessType::Download | ProcessType::Upload => {
            // Network speed dependent
            let file_size = target.file_size.unwrap_or(1024 * 1024); // Default 1MB
            let net_speed = player.internet_speed as f32;
            let target_speed = target.internet_speed.unwrap_or(100) as f32;
            let effective_speed = net_speed.min(target_speed);
            
            ((file_size as f32 / 1024.0) / effective_speed * config.network_time_factor).ceil() as i32
        },
        ProcessType::Crack | ProcessType::BruteForce => {
            // CPU intensive
            let cpu_power = player.cpu_mhz as f32;
            let target_security = target.security_level.unwrap_or(50) as f32;
            
            (base_complexity * target_security * config.cpu_time_factor / (cpu_power / 1000.0)).ceil() as i32
        },
        ProcessType::Decrypt | ProcessType::Encrypt => {
            // CPU and RAM intensive
            let cpu_power = player.cpu_mhz as f32;
            let ram_size = player.ram_mb as f32;
            let encryption_strength = target.encryption_level.unwrap_or(128) as f32;
            
            (base_complexity * encryption_strength * config.crypto_time_factor / 
             ((cpu_power / 1000.0) * (ram_size / 1024.0).sqrt())).ceil() as i32
        },
        ProcessType::VirusScan | ProcessType::SystemScan => {
            // HDD intensive scan
            let hdd_size = target.hdd_size.unwrap_or(10 * 1024) as f32; // Default 10GB
            let scan_speed = player.cpu_mhz as f32 / 100.0;
            
            (hdd_size / scan_speed * config.scan_time_factor).ceil() as i32
        },
        ProcessType::DDoSAttack => {
            // Network intensive
            let attack_power = player.internet_speed as f32 * (player.cpu_mhz as f32 / 1000.0);
            let target_defense = target.ddos_protection.unwrap_or(100) as f32;
            
            (base_complexity * target_defense * config.ddos_time_factor / attack_power).ceil() as i32
        },
        ProcessType::Research => {
            // Research time based on software version
            let current_version = player.software_levels.get("research").unwrap_or(&1.0);
            let target_version = target.research_target.unwrap_or(current_version + 1.0);
            let version_diff = target_version - current_version;
            
            (base_complexity * version_diff * version_diff * config.research_time_factor * 100.0).ceil() as i32
        },
        ProcessType::BankTransfer => {
            // Financial transaction
            let amount = target.transfer_amount.unwrap_or(1000) as f32;
            let security_checks = (amount / 10000.0).ln().max(1.0);
            
            (base_complexity * security_checks * config.finance_time_factor * 10.0).ceil() as i32
        },
        ProcessType::BitcoinMine => {
            // Mining difficulty
            let hash_power = player.cpu_mhz as f32 * player.gpu_cores.unwrap_or(1) as f32;
            let difficulty = target.mining_difficulty.unwrap_or(1000000) as f32;
            
            (difficulty / hash_power * config.mining_time_factor).ceil() as i32
        },
        _ => {
            // Default calculation for other types
            (base_complexity * config.default_time_factor * 60.0).ceil() as i32
        }
    };
    
    // Apply player skill modifiers
    let skill_modifier = calculate_skill_modifier(player, &p_type);
    let final_duration = (base_duration as f32 / skill_modifier).max(config.min_process_time as f32) as i32;
    
    final_duration.min(config.max_process_time)
}

/// Calculate resource usage for a process
pub fn calculate_resource_usage(process_type: &str, target: &crate::TargetInfo, config: &ProcessConfig) -> crate::ResourceUsage {
    let extended_target = crate::extended::extend_target_info(target);
    calculate_resource_usage_extended(process_type, &extended_target, config)
}

/// Calculate resource usage with extended target info
pub fn calculate_resource_usage_extended(process_type: &str, target: &crate::extended::ExtendedTargetInfo, config: &ProcessConfig) -> crate::ResourceUsage {
    let p_type = ProcessType::from_str(process_type);
    let base_complexity = p_type.base_complexity();
    
    let (cpu, ram, net, hdd) = match &p_type {
        ProcessType::Download => {
            let file_size = target.file_size.unwrap_or(1024 * 1024);
            (
                10,
                256,
                (file_size / 1024 / 100).max(10) as i32,
                (file_size / 1024 / 1024) as i32
            )
        },
        ProcessType::Upload => {
            let file_size = target.file_size.unwrap_or(1024 * 1024);
            (
                10,
                256,
                (file_size / 1024 / 100).max(10) as i32,
                0
            )
        },
        ProcessType::Crack | ProcessType::BruteForce => {
            let security = target.security_level.unwrap_or(50);
            (
                (base_complexity * security as f32 / 2.0) as i32,
                512 * (security / 25).max(1),
                5,
                0
            )
        },
        ProcessType::Decrypt | ProcessType::Encrypt => {
            let encryption = target.encryption_level.unwrap_or(128);
            (
                (base_complexity * 30.0) as i32,
                encryption * 8,
                0,
                0
            )
        },
        ProcessType::VirusScan | ProcessType::SystemScan => {
            let scan_size = target.hdd_size.unwrap_or(10 * 1024);
            (
                20,
                512,
                0,
                (scan_size / 100).min(100) as i32
            )
        },
        ProcessType::DDoSAttack => {
            (
                40,
                1024,
                100,
                0
            )
        },
        ProcessType::Research => {
            (
                50,
                2048,
                10,
                100
            )
        },
        ProcessType::BitcoinMine => {
            (
                90,
                4096,
                20,
                50
            )
        },
        ProcessType::Install | ProcessType::Uninstall => {
            let software_size = target.software_size.unwrap_or(100);
            (
                15,
                software_size * 2,
                0,
                software_size
            )
        },
        _ => {
            // Default resource usage
            (
                (base_complexity * 10.0) as i32,
                (base_complexity * 256.0) as i32,
                (base_complexity * 5.0) as i32,
                0
            )
        }
    };
    
    crate::ResourceUsage {
        cpu_usage: cpu.max(1).min(100),
        ram_usage: ram.max(64),
        net_usage: net.max(0).min(1000),
        hdd_usage: hdd.max(0),
    }
}

/// Calculate skill modifier based on player stats and process type
pub fn calculate_skill_modifier(player: &crate::extended::ExtendedPlayerState, process_type: &ProcessType) -> f32 {
    let base_skill = match process_type {
        ProcessType::Crack | ProcessType::BruteForce => {
            player.hacking_skill.unwrap_or(1) as f32 / 100.0
        },
        ProcessType::Decrypt | ProcessType::Encrypt => {
            player.crypto_skill.unwrap_or(1) as f32 / 100.0
        },
        ProcessType::VirusScan | ProcessType::AntiVirusRun => {
            player.antivirus_skill.unwrap_or(1) as f32 / 100.0
        },
        ProcessType::HideLog | ProcessType::DeleteLog => {
            player.stealth_skill.unwrap_or(1) as f32 / 100.0
        },
        ProcessType::Research => {
            player.research_skill.unwrap_or(1) as f32 / 100.0
        },
        ProcessType::DDoSAttack => {
            player.network_skill.unwrap_or(1) as f32 / 100.0
        },
        _ => 1.0
    };
    
    // Apply experience bonus
    let exp_bonus = (player.total_exp as f32 / 1000000.0).sqrt().min(2.0);
    
    (base_skill + 1.0) * (1.0 + exp_bonus * 0.1)
}

/// Process completion handler
pub fn handle_process_completion(process: &Process, player: &mut crate::extended::ExtendedPlayerState, config: &ProcessConfig) -> Result<String, String> {
    match &process.process_type {
        ProcessType::Download => {
            if let Some(file_id) = process.file_id {
                player.downloaded_files.push(file_id);
                Ok(format!("File {} downloaded successfully", file_id))
            } else {
                Err("No file specified for download".to_string())
            }
        },
        ProcessType::Crack => {
            if let Some(target_id) = process.target_id {
                player.cracked_systems.insert(target_id);
                player.total_exp += config.crack_exp_reward;
                Ok(format!("System {} cracked successfully. +{} EXP", target_id, config.crack_exp_reward))
            } else {
                Err("No target specified for crack".to_string())
            }
        },
        ProcessType::Research => {
            if let Some(software) = process.metadata.get("software") {
                let current = player.software_levels.get(software).unwrap_or(&1.0);
                player.software_levels.insert(software.clone(), current + 0.1);
                player.total_exp += config.research_exp_reward;
                Ok(format!("{} upgraded to version {:.1}", software, current + 0.1))
            } else {
                Err("No software specified for research".to_string())
            }
        },
        ProcessType::BitcoinMine => {
            let mined = process.metadata.get("amount")
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(0.001);
            player.bitcoin_amount += mined;
            Ok(format!("Mined {} BTC", mined))
        },
        ProcessType::BankTransfer => {
            if let Some(amount_str) = process.metadata.get("amount") {
                if let Ok(amount) = amount_str.parse::<i32>() {
                    player.money += amount;
                    Ok(format!("Transferred ${}", amount))
                } else {
                    Err("Invalid transfer amount".to_string())
                }
            } else {
                Err("No amount specified for transfer".to_string())
            }
        },
        ProcessType::DDoSAttack => {
            if let Some(target_ip) = &process.target_ip {
                player.ddos_attacks_performed += 1;
                Ok(format!("DDoS attack on {} completed", target_ip))
            } else {
                Err("No target specified for DDoS".to_string())
            }
        },
        ProcessType::VirusScan => {
            let viruses_found = process.metadata.get("viruses_found")
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            if viruses_found > 0 {
                Ok(format!("Scan complete: {} viruses found", viruses_found))
            } else {
                Ok("Scan complete: System clean".to_string())
            }
        },
        _ => Ok(format!("{:?} process completed", process.process_type))
    }
}

/// Calculate process success chance
pub fn calculate_success_chance(
    process_type: &ProcessType,
    player: &crate::extended::ExtendedPlayerState,
    target: &crate::extended::ExtendedTargetInfo,
    config: &ProcessConfig
) -> f32 {
    let base_chance = match process_type {
        ProcessType::Crack => {
            let player_skill = player.hacking_skill.unwrap_or(1) as f32;
            let target_defense = target.security_level.unwrap_or(50) as f32;
            
            (player_skill / (player_skill + target_defense) * 100.0).min(config.max_success_chance)
        },
        ProcessType::BruteForce => {
            let cpu_power = player.cpu_mhz as f32;
            let password_strength = target.password_strength.unwrap_or(100) as f32;
            
            ((cpu_power / 10000.0) / password_strength * 100.0).min(config.max_success_chance)
        },
        ProcessType::Hijack => {
            let player_skill = player.hacking_skill.unwrap_or(1) as f32;
            let target_protection = target.hijack_protection.unwrap_or(80) as f32;
            
            (player_skill / (player_skill + target_protection * 2.0) * 100.0).min(50.0)
        },
        ProcessType::DDoSAttack => {
            let attack_power = player.internet_speed as f32 * (player.cpu_mhz as f32 / 1000.0);
            let ddos_protection = target.ddos_protection.unwrap_or(100) as f32;
            
            (attack_power / (attack_power + ddos_protection) * 100.0).min(config.max_success_chance)
        },
        _ => 100.0 // Most processes always succeed if they complete
    };
    
    base_chance.max(config.min_success_chance)
}

/// Process chaining - create dependent processes
pub fn create_process_chain(
    initial_type: ProcessType,
    player: &crate::extended::ExtendedPlayerState,
    target: &crate::extended::ExtendedTargetInfo,
    config: &ProcessConfig
) -> Vec<Process> {
    let mut chain = Vec::new();
    
    // Create initial process
    let duration = calculate_duration_extended(&format!("{:?}", initial_type), player, target, config);
    let resources = calculate_resource_usage_extended(&format!("{:?}", initial_type), target, config);
    let mut parent = Process::new(
        initial_type.clone(),
        player.player_id,
        Duration::from_secs(duration as u64),
        resources
    );
    
    if let Some(target_id) = target.target_id {
        if let Some(target_ip) = &target.target_ip {
            parent = parent.with_target(target_id, target_ip.clone());
        }
    }
    
    let parent_id = parent.id;
    chain.push(parent);
    
    // Add dependent processes based on type
    match initial_type {
        ProcessType::Crack => {
            // After cracking, scan the system
            let scan_duration = calculate_duration_extended("system_scan", player, target, config);
            let scan_resources = calculate_resource_usage_extended("system_scan", target, config);
            let mut scan = Process::new(
                ProcessType::SystemScan,
                player.player_id,
                Duration::from_secs(scan_duration as u64),
                scan_resources
            );
            scan.parent_process_id = Some(parent_id);
            chain.push(scan);
        },
        ProcessType::Download => {
            // After download, scan for viruses
            let scan_duration = calculate_duration_extended("virus_scan", player, target, config);
            let scan_resources = calculate_resource_usage_extended("virus_scan", target, config);
            let mut scan = Process::new(
                ProcessType::VirusScan,
                player.player_id,
                Duration::from_secs(scan_duration as u64),
                scan_resources
            );
            scan.parent_process_id = Some(parent_id);
            chain.push(scan);
        },
        ProcessType::SystemScan => {
            // After system scan, hide logs
            let hide_duration = calculate_duration_extended("hide_log", player, target, config);
            let hide_resources = calculate_resource_usage_extended("hide_log", target, config);
            let mut hide = Process::new(
                ProcessType::HideLog,
                player.player_id,
                Duration::from_secs(hide_duration as u64),
                hide_resources
            );
            hide.parent_process_id = Some(parent_id);
            chain.push(hide);
        },
        _ => {}
    }
    
    chain
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_process_creation() {
        let process = Process::new(
            ProcessType::Download,
            1,
            Duration::from_secs(300),
            ResourceUsage { cpu_usage: 10, ram_usage: 256, net_usage: 50, hdd_usage: 100 }
        );
        
        assert_eq!(process.state, ProcessState::Queued);
        assert_eq!(process.player_id, 1);
        assert_eq!(process.total_duration, Duration::from_secs(300));
    }
    
    #[test]
    fn test_process_state_transitions() {
        let mut process = Process::new(
            ProcessType::Crack,
            1,
            Duration::from_secs(600),
            ResourceUsage { cpu_usage: 50, ram_usage: 1024, net_usage: 10, hdd_usage: 0 }
        );
        
        // Start process
        process.start();
        assert_eq!(process.state, ProcessState::Running);
        assert!(process.started_at.is_some());
        
        // Pause process
        process.pause();
        assert_eq!(process.state, ProcessState::Paused);
        assert!(process.paused_at.is_some());
        
        // Resume process
        process.resume();
        assert_eq!(process.state, ProcessState::Running);
        assert!(process.paused_at.is_none());
        
        // Complete process
        process.complete();
        assert_eq!(process.state, ProcessState::Completed);
        assert!(process.completed_at.is_some());
    }
    
    #[test]
    fn test_scheduler_resource_management() {
        let mut scheduler = ProcessScheduler::new(3, 100, 4096, 100);
        
        let process1 = Process::new(
            ProcessType::Download,
            1,
            Duration::from_secs(100),
            ResourceUsage { cpu_usage: 30, ram_usage: 1024, net_usage: 50, hdd_usage: 0 }
        );
        
        let process2 = Process::new(
            ProcessType::Crack,
            1,
            Duration::from_secs(200),
            ResourceUsage { cpu_usage: 50, ram_usage: 2048, net_usage: 10, hdd_usage: 0 }
        );
        
        let process3 = Process::new(
            ProcessType::Research,
            1,
            Duration::from_secs(300),
            ResourceUsage { cpu_usage: 40, ram_usage: 2048, net_usage: 20, hdd_usage: 100 }
        );
        
        // Add processes
        let id1 = scheduler.add_process(process1).unwrap();
        let id2 = scheduler.add_process(process2).unwrap();
        let id3 = scheduler.add_process(process3).unwrap();
        
        // Check running processes
        assert_eq!(scheduler.running.len(), 2); // Only 2 should run due to resource limits
        assert_eq!(scheduler.queue.len(), 1); // One should be queued
        
        // Complete first process
        scheduler.complete_process(id1).unwrap();
        assert_eq!(scheduler.running.len(), 2); // Third process should start
        assert_eq!(scheduler.queue.len(), 0);
    }
    
    #[test]
    fn test_duration_calculation() {
        let mut player = crate::extended::ExtendedPlayerState::default();
        player.cpu_mhz = 3000;
        player.ram_mb = 8192;
        player.internet_speed = 100;
        player.hacking_skill = Some(75);

        let mut target = crate::extended::ExtendedTargetInfo::default();
        target.security_level = Some(50);
        target.file_size = Some(10 * 1024 * 1024); // 10MB

        let config = ProcessConfig::default();

        let crack_duration = calculate_duration_extended("crack", &player, &target, &config);
        assert!(crack_duration > 0);
        assert!(crack_duration < config.max_process_time);

        let download_duration = calculate_duration_extended("download", &player, &target, &config);
        assert!(download_duration > 0);
        assert!(download_duration < config.max_process_time);
    }
    
    #[test]
    fn test_success_chance_calculation() {
        let mut player = crate::extended::ExtendedPlayerState::default();
        player.hacking_skill = Some(80);
        player.cpu_mhz = 4000;

        let mut target = crate::extended::ExtendedTargetInfo::default();
        target.security_level = Some(60);
        target.password_strength = Some(100);

        let config = ProcessConfig::default();

        let crack_chance = calculate_success_chance(&ProcessType::Crack, &player, &target, &config);
        assert!(crack_chance > 0.0);
        assert!(crack_chance <= 100.0);

        let brute_chance = calculate_success_chance(&ProcessType::BruteForce, &player, &target, &config);
        assert!(brute_chance > 0.0);
        assert!(brute_chance <= 100.0);
    }
    
    #[test]
    fn test_process_chaining() {
        let player = crate::extended::ExtendedPlayerState::default();
        let target = crate::extended::ExtendedTargetInfo::default();
        let config = ProcessConfig::default();

        let chain = create_process_chain(ProcessType::Crack, &player, &target, &config);
        assert_eq!(chain.len(), 2); // Crack + SystemScan
        assert_eq!(chain[0].process_type, ProcessType::Crack);
        assert_eq!(chain[1].process_type, ProcessType::SystemScan);
        assert_eq!(chain[1].parent_process_id, Some(chain[0].id));
    }
}