//! Software system mechanics - Dependencies, effectiveness, installation mechanics

use crate::{SoftwareInstance, HardwareSpecs, PlayerState};
use crate::config::SoftwareConfig;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Software types in the game
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SoftwareType {
    // Offensive Software
    Cracker,
    Exploiter,
    Breaker,
    FTPExploit,
    SSHExploit,
    Bruteforcer,
    DictionaryAttacker,
    
    // Defensive Software
    Firewall,
    AntiVirus,
    IntrusionDetection,
    LogCleaner,
    LogForger,
    Hidder,
    
    // Utility Software
    Analyzer,
    Decrypter,
    Encrypter,
    Compressor,
    FileManager,
    ProcessManager,
    
    // Network Software
    Proxy,
    VPN,
    TorClient,
    PortScanner,
    NetworkMonitor,
    PacketSniffer,
    
    // Virus/Malware
    Virus,
    Worm,
    Trojan,
    Spyware,
    Keylogger,
    Ransomware,
    Rootkit,
    Botnet,
    
    // Specialized Software
    BitcoinMiner,
    PasswordManager,
    SystemOptimizer,
    DataRecovery,
    DiskWiper,
    
    // Research Software
    ResearchTool,
    Compiler,
    Debugger,
    ReverseEngineer,
    
    Custom(String),
}

impl SoftwareType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "cracker" => SoftwareType::Cracker,
            "exploiter" => SoftwareType::Exploiter,
            "breaker" => SoftwareType::Breaker,
            "ftp_exploit" | "ftpexploit" => SoftwareType::FTPExploit,
            "ssh_exploit" | "sshexploit" => SoftwareType::SSHExploit,
            "bruteforcer" | "brute" => SoftwareType::Bruteforcer,
            "dictionary" | "dict_attacker" => SoftwareType::DictionaryAttacker,
            "firewall" | "fw" => SoftwareType::Firewall,
            "antivirus" | "av" => SoftwareType::AntiVirus,
            "ids" | "intrusion_detection" => SoftwareType::IntrusionDetection,
            "log_cleaner" | "logcleaner" => SoftwareType::LogCleaner,
            "log_forger" | "logforger" => SoftwareType::LogForger,
            "hidder" | "hider" => SoftwareType::Hidder,
            "analyzer" => SoftwareType::Analyzer,
            "decrypter" | "decrypt" => SoftwareType::Decrypter,
            "encrypter" | "encrypt" => SoftwareType::Encrypter,
            "compressor" | "compress" => SoftwareType::Compressor,
            "file_manager" | "fm" => SoftwareType::FileManager,
            "process_manager" | "pm" => SoftwareType::ProcessManager,
            "proxy" => SoftwareType::Proxy,
            "vpn" => SoftwareType::VPN,
            "tor" | "tor_client" => SoftwareType::TorClient,
            "port_scanner" | "portscan" => SoftwareType::PortScanner,
            "network_monitor" | "netmon" => SoftwareType::NetworkMonitor,
            "packet_sniffer" | "sniffer" => SoftwareType::PacketSniffer,
            "virus" => SoftwareType::Virus,
            "worm" => SoftwareType::Worm,
            "trojan" => SoftwareType::Trojan,
            "spyware" | "spy" => SoftwareType::Spyware,
            "keylogger" | "kl" => SoftwareType::Keylogger,
            "ransomware" | "ransom" => SoftwareType::Ransomware,
            "rootkit" => SoftwareType::Rootkit,
            "botnet" | "bot" => SoftwareType::Botnet,
            "bitcoin_miner" | "miner" => SoftwareType::BitcoinMiner,
            "password_manager" | "pwm" => SoftwareType::PasswordManager,
            "system_optimizer" | "optimizer" => SoftwareType::SystemOptimizer,
            "data_recovery" | "recovery" => SoftwareType::DataRecovery,
            "disk_wiper" | "wiper" => SoftwareType::DiskWiper,
            "research_tool" | "research" => SoftwareType::ResearchTool,
            "compiler" => SoftwareType::Compiler,
            "debugger" | "debug" => SoftwareType::Debugger,
            "reverse_engineer" | "re" => SoftwareType::ReverseEngineer,
            other => SoftwareType::Custom(other.to_string()),
        }
    }
    
    pub fn category(&self) -> SoftwareCategory {
        match self {
            SoftwareType::Cracker | SoftwareType::Exploiter | SoftwareType::Breaker |
            SoftwareType::FTPExploit | SoftwareType::SSHExploit | 
            SoftwareType::Bruteforcer | SoftwareType::DictionaryAttacker => SoftwareCategory::Offensive,
            
            SoftwareType::Firewall | SoftwareType::AntiVirus | SoftwareType::IntrusionDetection |
            SoftwareType::LogCleaner | SoftwareType::LogForger | SoftwareType::Hidder => SoftwareCategory::Defensive,
            
            SoftwareType::Analyzer | SoftwareType::Decrypter | SoftwareType::Encrypter |
            SoftwareType::Compressor | SoftwareType::FileManager | SoftwareType::ProcessManager => SoftwareCategory::Utility,
            
            SoftwareType::Proxy | SoftwareType::VPN | SoftwareType::TorClient |
            SoftwareType::PortScanner | SoftwareType::NetworkMonitor | SoftwareType::PacketSniffer => SoftwareCategory::Network,
            
            SoftwareType::Virus | SoftwareType::Worm | SoftwareType::Trojan | SoftwareType::Spyware |
            SoftwareType::Keylogger | SoftwareType::Ransomware | SoftwareType::Rootkit | 
            SoftwareType::Botnet => SoftwareCategory::Malware,
            
            SoftwareType::BitcoinMiner | SoftwareType::PasswordManager | SoftwareType::SystemOptimizer |
            SoftwareType::DataRecovery | SoftwareType::DiskWiper => SoftwareCategory::Specialized,
            
            SoftwareType::ResearchTool | SoftwareType::Compiler | SoftwareType::Debugger |
            SoftwareType::ReverseEngineer => SoftwareCategory::Research,
            
            SoftwareType::Custom(_) => SoftwareCategory::Custom,
        }
    }
    
    pub fn base_size_mb(&self) -> i32 {
        match self {
            SoftwareType::Cracker => 50,
            SoftwareType::Firewall => 100,
            SoftwareType::AntiVirus => 200,
            SoftwareType::Virus => 5,
            SoftwareType::Worm => 2,
            SoftwareType::BitcoinMiner => 500,
            SoftwareType::VPN => 150,
            SoftwareType::Proxy => 75,
            SoftwareType::Keylogger => 1,
            SoftwareType::Rootkit => 10,
            SoftwareType::SystemOptimizer => 300,
            SoftwareType::ResearchTool => 1000,
            _ => 25,
        }
    }
    
    pub fn base_ram_usage(&self) -> i32 {
        match self {
            SoftwareType::AntiVirus => 512,
            SoftwareType::Firewall => 256,
            SoftwareType::BitcoinMiner => 2048,
            SoftwareType::IntrusionDetection => 384,
            SoftwareType::NetworkMonitor => 128,
            SoftwareType::SystemOptimizer => 768,
            SoftwareType::ResearchTool => 1024,
            SoftwareType::Compiler => 512,
            _ => 64,
        }
    }
}

/// Software categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoftwareCategory {
    Offensive,
    Defensive,
    Utility,
    Network,
    Malware,
    Specialized,
    Research,
    Custom,
}

/// Software license types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseType {
    Cracked,
    Trial,
    Personal,
    Professional,
    Enterprise,
    Lifetime,
}

impl LicenseType {
    pub fn effectiveness_multiplier(&self) -> f32 {
        match self {
            LicenseType::Cracked => 0.7,
            LicenseType::Trial => 0.8,
            LicenseType::Personal => 1.0,
            LicenseType::Professional => 1.2,
            LicenseType::Enterprise => 1.5,
            LicenseType::Lifetime => 1.3,
        }
    }
    
    pub fn duration_days(&self) -> Option<i32> {
        match self {
            LicenseType::Cracked => None,
            LicenseType::Trial => Some(30),
            LicenseType::Personal => Some(365),
            LicenseType::Professional => Some(365),
            LicenseType::Enterprise => Some(365 * 3),
            LicenseType::Lifetime => None,
        }
    }
}

/// Individual software instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Software {
    pub id: Uuid,
    pub software_type: SoftwareType,
    pub name: String,
    pub version: f32,
    pub size_mb: i32,
    pub ram_usage: i32,
    pub cpu_usage: i32,
    pub license: LicenseType,
    pub license_expires: Option<SystemTime>,
    pub installed_at: Option<SystemTime>,
    pub last_updated: Option<SystemTime>,
    pub is_running: bool,
    pub is_hidden: bool,
    pub is_infected: bool,
    pub dependencies: Vec<SoftwareType>,
    pub incompatible_with: Vec<SoftwareType>,
    pub effectiveness: i32,
    pub integrity: f32,
    pub signature: String,
    pub custom_config: HashMap<String, String>,
}

impl Software {
    pub fn new(software_type: SoftwareType, version: f32) -> Self {
        let base_size = software_type.base_size_mb();
        let size_mb = (base_size as f32 * version.sqrt()) as i32;
        
        Software {
            id: Uuid::new_v4(),
            software_type: software_type.clone(),
            name: format!("{:?} v{:.1}", software_type, version),
            version,
            size_mb,
            ram_usage: software_type.base_ram_usage(),
            cpu_usage: 5,
            license: LicenseType::Cracked,
            license_expires: None,
            installed_at: None,
            last_updated: Some(SystemTime::now()),
            is_running: false,
            is_hidden: false,
            is_infected: false,
            dependencies: Vec::new(),
            incompatible_with: Vec::new(),
            effectiveness: 100,
            integrity: 100.0,
            signature: format!("{:x}", rand::random::<u64>()),
            custom_config: HashMap::new(),
        }
    }
    
    pub fn with_license(mut self, license: LicenseType) -> Self {
        self.license = license.clone();
        if let Some(days) = license.duration_days() {
            self.license_expires = Some(SystemTime::now() + Duration::from_secs(days as u64 * 86400));
        }
        self
    }
    
    pub fn with_dependencies(mut self, deps: Vec<SoftwareType>) -> Self {
        self.dependencies = deps;
        self
    }
    
    pub fn install(&mut self) -> Result<(), String> {
        if self.integrity < 50.0 {
            return Err("Software too corrupted to install".to_string());
        }
        
        self.installed_at = Some(SystemTime::now());
        self.is_running = false;
        Ok(())
    }
    
    pub fn start(&mut self) -> Result<(), String> {
        if self.installed_at.is_none() {
            return Err("Software not installed".to_string());
        }
        
        if self.is_infected {
            return Err("Cannot run infected software".to_string());
        }
        
        if let Some(expires) = self.license_expires {
            if SystemTime::now() > expires {
                return Err("License expired".to_string());
            }
        }
        
        self.is_running = true;
        Ok(())
    }
    
    pub fn stop(&mut self) {
        self.is_running = false;
    }
    
    pub fn update(&mut self, new_version: f32) -> Result<(), String> {
        if new_version <= self.version {
            return Err("Cannot downgrade software".to_string());
        }
        
        self.version = new_version;
        self.size_mb = (self.size_mb as f32 * 1.1) as i32;
        self.last_updated = Some(SystemTime::now());
        self.effectiveness = (self.effectiveness as f32 * 1.05).min(150.0) as i32;
        
        Ok(())
    }
    
    pub fn infect(&mut self, virus_strength: i32) {
        self.is_infected = true;
        self.integrity -= virus_strength as f32;
        self.effectiveness = (self.effectiveness as f32 * 0.7) as i32;
    }
    
    pub fn disinfect(&mut self) {
        self.is_infected = false;
        self.integrity = (self.integrity + 20.0).min(100.0);
    }
    
    pub fn hide(&mut self) {
        self.is_hidden = true;
    }
    
    pub fn unhide(&mut self) {
        self.is_hidden = false;
    }
    
    pub fn verify_signature(&self, expected_signature: &str) -> bool {
        self.signature == expected_signature && !self.is_infected
    }
    
    pub fn calculate_effectiveness(&self, hardware: &HardwareSpecs) -> i32 {
        let base = self.effectiveness as f32;
        let version_bonus = self.version * 10.0;
        let license_mult = self.license.effectiveness_multiplier();
        let integrity_factor = self.integrity / 100.0;
        
        let hardware_factor = match self.software_type {
            SoftwareType::Cracker | SoftwareType::Bruteforcer => {
                (hardware.cpu_mhz as f32 / 3000.0).min(2.0)
            },
            SoftwareType::AntiVirus | SoftwareType::Firewall => {
                (hardware.ram_mb as f32 / 4096.0).min(2.0)
            },
            SoftwareType::BitcoinMiner => {
                let cpu_factor = hardware.cpu_mhz as f32 / 3000.0;
                let gpu_factor = hardware.gpu_cores.unwrap_or(0) as f32 / 1000.0;
                (cpu_factor + gpu_factor * 2.0).min(3.0)
            },
            _ => 1.0
        };
        
        ((base + version_bonus) * license_mult * integrity_factor * hardware_factor) as i32
    }
    
    pub fn get_resource_usage(&self) -> (i32, i32) {
        let cpu = if self.is_running {
            (self.cpu_usage as f32 * (1.0 + self.version * 0.1)) as i32
        } else {
            0
        };
        
        let ram = if self.is_running {
            self.ram_usage
        } else {
            0
        };
        
        (cpu, ram)
    }
}

/// Software inventory management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareInventory {
    pub installed: HashMap<Uuid, Software>,
    pub running: HashSet<Uuid>,
    pub total_size_mb: i32,
    pub total_ram_usage: i32,
    pub total_cpu_usage: i32,
}

impl SoftwareInventory {
    pub fn new() -> Self {
        SoftwareInventory {
            installed: HashMap::new(),
            running: HashSet::new(),
            total_size_mb: 0,
            total_ram_usage: 0,
            total_cpu_usage: 0,
        }
    }
    
    pub fn install_software(&mut self, mut software: Software, hardware: &HardwareSpecs) -> Result<Uuid, String> {
        // Check disk space
        if self.total_size_mb + software.size_mb > hardware.hdd_gb * 1024 {
            return Err("Insufficient disk space".to_string());
        }
        
        // Check dependencies
        for dep in &software.dependencies {
            if !self.has_software_type(dep) {
                return Err(format!("Missing dependency: {:?}", dep));
            }
        }
        
        // Check incompatibilities
        for incompatible in &software.incompatible_with {
            if self.has_software_type(incompatible) {
                return Err(format!("Incompatible with installed software: {:?}", incompatible));
            }
        }
        
        software.install()?;
        let software_id = software.id;
        
        self.total_size_mb += software.size_mb;
        self.installed.insert(software_id, software);
        
        Ok(software_id)
    }
    
    pub fn uninstall_software(&mut self, software_id: Uuid) -> Result<(), String> {
        let software = self.installed.get(&software_id)
            .ok_or("Software not found")?;
        
        if self.running.contains(&software_id) {
            return Err("Cannot uninstall running software".to_string());
        }
        
        // Check if other software depends on this
        let software_type = &software.software_type;
        for other in self.installed.values() {
            if other.dependencies.contains(software_type) {
                return Err(format!("{} is required by other software", software.name));
            }
        }
        
        self.total_size_mb -= software.size_mb;
        self.installed.remove(&software_id);
        
        Ok(())
    }
    
    pub fn start_software(&mut self, software_id: Uuid, hardware: &HardwareSpecs) -> Result<(), String> {
        let software = self.installed.get_mut(&software_id)
            .ok_or("Software not found")?;
        
        let (cpu, ram) = software.get_resource_usage();
        
        // Check resource availability
        if self.total_ram_usage + ram > hardware.ram_mb {
            return Err("Insufficient RAM".to_string());
        }
        
        if self.total_cpu_usage + cpu > 100 {
            return Err("CPU at maximum capacity".to_string());
        }
        
        software.start()?;
        self.running.insert(software_id);
        self.total_ram_usage += ram;
        self.total_cpu_usage += cpu;
        
        Ok(())
    }
    
    pub fn stop_software(&mut self, software_id: Uuid) -> Result<(), String> {
        let software = self.installed.get_mut(&software_id)
            .ok_or("Software not found")?;
        
        let (cpu, ram) = software.get_resource_usage();
        
        software.stop();
        self.running.remove(&software_id);
        self.total_ram_usage -= ram;
        self.total_cpu_usage -= cpu;
        
        Ok(())
    }
    
    pub fn has_software_type(&self, software_type: &SoftwareType) -> bool {
        self.installed.values().any(|s| &s.software_type == software_type)
    }
    
    pub fn get_software_by_type(&self, software_type: &SoftwareType) -> Vec<&Software> {
        self.installed.values()
            .filter(|s| &s.software_type == software_type)
            .collect()
    }
    
    pub fn get_running_software(&self) -> Vec<&Software> {
        self.running.iter()
            .filter_map(|id| self.installed.get(id))
            .collect()
    }
    
    pub fn scan_for_viruses(&mut self) -> Vec<Uuid> {
        let mut infected = Vec::new();
        
        for (id, software) in &self.installed {
            if software.is_infected {
                infected.push(*id);
            }
        }
        
        infected
    }
    
    pub fn optimize(&mut self) {
        // Stop non-essential software if resources are tight
        let non_essential = vec![
            SoftwareType::SystemOptimizer,
            SoftwareType::NetworkMonitor,
            SoftwareType::PacketSniffer,
        ];
        
        for software_type in non_essential {
            if let Some(software) = self.get_software_by_type(&software_type).first() {
                if self.running.contains(&software.id) {
                    let _ = self.stop_software(software.id);
                }
            }
        }
    }
}

/// Calculate software effectiveness
pub fn calculate_effectiveness(software: &SoftwareInstance, hardware: &HardwareSpecs, config: &SoftwareConfig) -> i32 {
    let base_effectiveness = 70;
    let version_bonus = (software.version * 10.0) as i32;
    
    let hardware_bonus = match software.software_type.as_str() {
        "cracker" | "bruteforcer" => (hardware.cpu_mhz / 100) as i32,
        "firewall" | "antivirus" => (hardware.ram_mb / 100) as i32,
        "miner" => {
            let cpu_bonus = (hardware.cpu_mhz / 100) as i32;
            let gpu_bonus = hardware.gpu_cores.unwrap_or(0) / 10;
            cpu_bonus + gpu_bonus
        },
        _ => 0,
    };
    
    (base_effectiveness + version_bonus + hardware_bonus).min(config.max_effectiveness)
}

/// Software compatibility check
pub fn check_compatibility(software1: &Software, software2: &Software) -> Result<(), String> {
    // Check for direct incompatibilities
    if software1.incompatible_with.contains(&software2.software_type) {
        return Err(format!("{} is incompatible with {}", software1.name, software2.name));
    }
    
    if software2.incompatible_with.contains(&software1.software_type) {
        return Err(format!("{} is incompatible with {}", software2.name, software1.name));
    }
    
    // Check category conflicts
    match (&software1.software_type.category(), &software2.software_type.category()) {
        (SoftwareCategory::Defensive, SoftwareCategory::Malware) |
        (SoftwareCategory::Malware, SoftwareCategory::Defensive) => {
            return Err("Defensive software conflicts with malware".to_string());
        },
        _ => {}
    }
    
    Ok(())
}

/// Generate software dependencies
pub fn generate_dependencies(software_type: &SoftwareType) -> Vec<SoftwareType> {
    match software_type {
        SoftwareType::Exploiter => vec![SoftwareType::Analyzer],
        SoftwareType::FTPExploit => vec![SoftwareType::PortScanner],
        SoftwareType::SSHExploit => vec![SoftwareType::PortScanner, SoftwareType::Bruteforcer],
        SoftwareType::IntrusionDetection => vec![SoftwareType::Firewall],
        SoftwareType::LogForger => vec![SoftwareType::LogCleaner],
        SoftwareType::VPN => vec![SoftwareType::Proxy],
        SoftwareType::TorClient => vec![SoftwareType::Proxy],
        SoftwareType::Rootkit => vec![SoftwareType::Hidder],
        SoftwareType::Botnet => vec![SoftwareType::Trojan, SoftwareType::NetworkMonitor],
        SoftwareType::Compiler => vec![SoftwareType::Debugger],
        SoftwareType::ReverseEngineer => vec![SoftwareType::Debugger, SoftwareType::Analyzer],
        _ => Vec::new()
    }
}

/// Calculate research cost for software upgrade
pub fn calculate_research_cost(current_version: f32, target_version: f32, software_type: &SoftwareType) -> i32 {
    if target_version <= current_version {
        return 0;
    }
    
    let version_diff = target_version - current_version;
    let complexity = match software_type.category() {
        SoftwareCategory::Research => 5.0,
        SoftwareCategory::Defensive => 3.0,
        SoftwareCategory::Offensive => 3.5,
        SoftwareCategory::Malware => 4.0,
        SoftwareCategory::Specialized => 4.5,
        _ => 2.0,
    };
    
    (version_diff * version_diff * complexity * 1000.0) as i32
}

/// Software market pricing
pub fn get_market_price(software: &Software) -> i32 {
    let base_price = match software.software_type.category() {
        SoftwareCategory::Research => 10000,
        SoftwareCategory::Defensive => 5000,
        SoftwareCategory::Offensive => 7500,
        SoftwareCategory::Malware => 15000,
        SoftwareCategory::Specialized => 8000,
        _ => 2500,
    };
    
    let version_multiplier = software.version.sqrt();
    let license_multiplier = match software.license {
        LicenseType::Cracked => 0.3,
        LicenseType::Trial => 0.5,
        LicenseType::Personal => 1.0,
        LicenseType::Professional => 2.0,
        LicenseType::Enterprise => 5.0,
        LicenseType::Lifetime => 3.0,
    };
    
    (base_price as f32 * version_multiplier * license_multiplier) as i32
}

/// Apply virus infection to system
pub fn propagate_infection(inventory: &mut SoftwareInventory, virus_strength: i32) -> i32 {
    let mut infected_count = 0;
    
    for software in inventory.installed.values_mut() {
        if !software.is_infected && rand::random::<f32>() < (virus_strength as f32 / 100.0) {
            software.infect(virus_strength);
            infected_count += 1;
        }
    }
    
    infected_count
}

/// Antivirus scan and clean
pub fn antivirus_scan(inventory: &mut SoftwareInventory, av_strength: i32) -> (i32, i32) {
    let infected = inventory.scan_for_viruses();
    let mut cleaned = 0;
    
    for id in infected.iter() {
        if let Some(software) = inventory.installed.get_mut(id) {
            if rand::random::<i32>() % 100 < av_strength {
                software.disinfect();
                cleaned += 1;
            }
        }
    }
    
    (infected.len() as i32, cleaned)
}

/// Software vulnerability assessment
pub fn assess_vulnerabilities(software: &Software) -> Vec<String> {
    let mut vulnerabilities = Vec::new();
    
    if software.version < 2.0 {
        vulnerabilities.push("Outdated version with known exploits".to_string());
    }
    
    if software.integrity < 80.0 {
        vulnerabilities.push("Low integrity increases exploit risk".to_string());
    }
    
    if matches!(software.license, LicenseType::Cracked) {
        vulnerabilities.push("Cracked license may contain backdoors".to_string());
    }
    
    if software.is_infected {
        vulnerabilities.push("Active malware infection detected".to_string());
    }
    
    if software.last_updated.map(|t| SystemTime::now().duration_since(t).unwrap_or_default().as_secs() > 30 * 86400).unwrap_or(true) {
        vulnerabilities.push("No recent security updates".to_string());
    }
    
    vulnerabilities
}

/// Generate software upgrade recommendations
pub fn generate_upgrade_recommendations(inventory: &SoftwareInventory, budget: i32) -> Vec<(Software, f32, i32)> {
    let mut recommendations = Vec::new();
    
    for software in inventory.installed.values() {
        let current_version = software.version;
        let recommended_version = (current_version + 1.0).min(10.0);
        let cost = calculate_research_cost(current_version, recommended_version, &software.software_type);
        
        if cost <= budget && current_version < 5.0 {
            recommendations.push((software.clone(), recommended_version, cost));
        }
    }
    
    recommendations.sort_by_key(|r| r.2);
    recommendations.truncate(3);
    
    recommendations
}

/// Software performance benchmarking
pub fn benchmark_software(software: &Software, hardware: &HardwareSpecs) -> SoftwareBenchmark {
    let effectiveness = software.calculate_effectiveness(hardware);
    let (cpu_usage, ram_usage) = software.get_resource_usage();
    
    let efficiency = if cpu_usage + ram_usage > 0 {
        (effectiveness as f32 / (cpu_usage + ram_usage) as f32 * 100.0) as i32
    } else {
        0
    };
    
    let stability = software.integrity as i32;
    
    SoftwareBenchmark {
        software_id: software.id,
        effectiveness,
        efficiency,
        stability,
        cpu_usage,
        ram_usage,
        overall_score: (effectiveness + efficiency + stability) / 3,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareBenchmark {
    pub software_id: Uuid,
    pub effectiveness: i32,
    pub efficiency: i32,
    pub stability: i32,
    pub cpu_usage: i32,
    pub ram_usage: i32,
    pub overall_score: i32,
}

/// Software conflict resolution
pub fn resolve_conflicts(inventory: &mut SoftwareInventory) -> Vec<String> {
    let mut actions = Vec::new();
    let mut to_stop = Vec::new();
    
    // Find conflicting software
    let running_software: Vec<Software> = inventory.running.iter()
        .filter_map(|id| inventory.installed.get(id).cloned())
        .collect();
    
    for i in 0..running_software.len() {
        for j in i+1..running_software.len() {
            if let Err(conflict) = check_compatibility(&running_software[i], &running_software[j]) {
                // Stop the lower priority one
                if running_software[i].effectiveness < running_software[j].effectiveness {
                    to_stop.push(running_software[i].id);
                    actions.push(format!("Stopped {} due to conflict", running_software[i].name));
                } else {
                    to_stop.push(running_software[j].id);
                    actions.push(format!("Stopped {} due to conflict", running_software[j].name));
                }
            }
        }
    }
    
    for id in to_stop {
        let _ = inventory.stop_software(id);
    }
    
    actions
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_software_creation() {
        let software = Software::new(SoftwareType::Firewall, 2.5);
        assert_eq!(software.software_type, SoftwareType::Firewall);
        assert_eq!(software.version, 2.5);
        assert!(software.size_mb > 0);
    }
    
    #[test]
    fn test_software_installation() {
        let mut software = Software::new(SoftwareType::AntiVirus, 3.0);
        assert!(software.install().is_ok());
        assert!(software.installed_at.is_some());
    }
    
    #[test]
    fn test_license_effectiveness() {
        let mut software = Software::new(SoftwareType::Cracker, 1.0);
        software = software.with_license(LicenseType::Professional);
        
        let hardware = HardwareSpecs {
            cpu_mhz: 3000,
            ram_mb: 8192,
            hdd_gb: 500,
            net_speed: 100,
            gpu_cores: Some(1000),
        };
        
        let effectiveness = software.calculate_effectiveness(&hardware);
        assert!(effectiveness > 100); // Should be boosted by professional license
    }
    
    #[test]
    fn test_inventory_management() {
        let mut inventory = SoftwareInventory::new();
        let hardware = HardwareSpecs {
            cpu_mhz: 3000,
            ram_mb: 8192,
            hdd_gb: 500,
            net_speed: 100,
            gpu_cores: None,
        };
        
        let firewall = Software::new(SoftwareType::Firewall, 2.0);
        let firewall_id = inventory.install_software(firewall, &hardware).unwrap();
        
        assert_eq!(inventory.installed.len(), 1);
        assert!(inventory.start_software(firewall_id, &hardware).is_ok());
        assert!(inventory.running.contains(&firewall_id));
    }
    
    #[test]
    fn test_virus_infection() {
        let mut software = Software::new(SoftwareType::FileManager, 1.5);
        let initial_effectiveness = software.effectiveness;
        
        software.infect(30);
        assert!(software.is_infected);
        assert!(software.effectiveness < initial_effectiveness);
        
        software.disinfect();
        assert!(!software.is_infected);
    }
    
    #[test]
    fn test_compatibility_check() {
        let antivirus = Software::new(SoftwareType::AntiVirus, 2.0);
        let mut virus = Software::new(SoftwareType::Virus, 1.0);
        virus.incompatible_with.push(SoftwareType::AntiVirus);
        
        let result = check_compatibility(&antivirus, &virus);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_research_cost() {
        let cost = calculate_research_cost(1.0, 3.0, &SoftwareType::ResearchTool);
        assert!(cost > 0);
        
        let no_cost = calculate_research_cost(3.0, 2.0, &SoftwareType::ResearchTool);
        assert_eq!(no_cost, 0);
    }
}