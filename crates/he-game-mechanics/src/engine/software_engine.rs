//! Complete Software Engine - Dependencies, versioning, and installation management

use super::{EngineComponent, EngineError, EngineResult, ComponentStatus};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Software categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SoftwareCategory {
    Cracker,
    Hasher,
    Firewall,
    Hidder,
    Seeker,
    AntiVirus,
    Spam,
    Warez,
    DDoS,
    Collector,
    Miner,
    Analyzer,
    Exploit,
    Encryptor,
    Decryptor,
    LogForger,
}

/// Software information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Software {
    pub id: Uuid,
    pub name: String,
    pub category: SoftwareCategory,
    pub version: SoftwareVersion,
    pub size_mb: f32,
    pub ram_required: f32,
    pub cpu_required: f32,
    pub dependencies: Vec<SoftwareDependency>,
    pub installed: bool,
    pub hidden: bool,
    pub research_time: Duration,
    pub install_time: Duration,
}

/// Software version with semantic versioning
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SoftwareVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl SoftwareVersion {
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self { major, minor, patch }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        Some(Self {
            major: parts[0].parse().ok()?,
            minor: parts[1].parse().ok()?,
            patch: parts[2].parse().ok()?,
        })
    }

    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    pub fn is_compatible(&self, required: &Self) -> bool {
        self.major == required.major && self >= required
    }
}

/// Software dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareDependency {
    pub software_name: String,
    pub category: SoftwareCategory,
    pub min_version: SoftwareVersion,
}

impl Software {
    pub fn new(name: String, category: SoftwareCategory, version: SoftwareVersion) -> Self {
        let (size, ram, cpu) = Self::calculate_requirements(category, &version);
        let (research_time, install_time) = Self::calculate_times(category, &version);

        Self {
            id: Uuid::new_v4(),
            name,
            category,
            version,
            size_mb: size,
            ram_required: ram,
            cpu_required: cpu,
            dependencies: Vec::new(),
            installed: false,
            hidden: false,
            research_time,
            install_time,
        }
    }

    fn calculate_requirements(category: SoftwareCategory, version: &SoftwareVersion) -> (f32, f32, f32) {
        let version_factor = version.major as f32 + (version.minor as f32 / 10.0);

        let (base_size, base_ram, base_cpu) = match category {
            SoftwareCategory::Cracker => (50.0, 128.0, 200.0),
            SoftwareCategory::Hasher => (30.0, 64.0, 150.0),
            SoftwareCategory::Firewall => (100.0, 256.0, 100.0),
            SoftwareCategory::Hidder => (20.0, 32.0, 50.0),
            SoftwareCategory::Seeker => (25.0, 64.0, 100.0),
            SoftwareCategory::AntiVirus => (150.0, 512.0, 300.0),
            SoftwareCategory::Spam => (10.0, 16.0, 50.0),
            SoftwareCategory::Warez => (500.0, 128.0, 100.0),
            SoftwareCategory::DDoS => (80.0, 256.0, 400.0),
            SoftwareCategory::Collector => (40.0, 128.0, 150.0),
            SoftwareCategory::Miner => (60.0, 512.0, 800.0),
            SoftwareCategory::Analyzer => (35.0, 128.0, 200.0),
            SoftwareCategory::Exploit => (45.0, 64.0, 250.0),
            SoftwareCategory::Encryptor => (30.0, 128.0, 300.0),
            SoftwareCategory::Decryptor => (30.0, 128.0, 350.0),
            SoftwareCategory::LogForger => (15.0, 32.0, 100.0),
        };

        (
            base_size * version_factor,
            base_ram * version_factor,
            base_cpu * version_factor,
        )
    }

    fn calculate_times(category: SoftwareCategory, version: &SoftwareVersion) -> (Duration, Duration) {
        let version_factor = version.major as f32 + (version.minor as f32 / 10.0);

        let base_research = match category {
            SoftwareCategory::Cracker => 3600,
            SoftwareCategory::Firewall => 7200,
            SoftwareCategory::AntiVirus => 10800,
            SoftwareCategory::Miner => 14400,
            SoftwareCategory::DDoS => 5400,
            _ => 1800,
        };

        let research_time = Duration::from_secs((base_research as f32 * version_factor) as u64);
        let install_time = Duration::from_secs((60.0 * version_factor) as u64);

        (research_time, install_time)
    }

    pub fn add_dependency(&mut self, dep: SoftwareDependency) {
        self.dependencies.push(dep);
    }

    pub fn power_level(&self) -> f32 {
        let version_factor = self.version.major as f32 + (self.version.minor as f32 / 10.0);
        let category_multiplier = match self.category {
            SoftwareCategory::Cracker => 2.0,
            SoftwareCategory::Firewall => 1.8,
            SoftwareCategory::AntiVirus => 1.5,
            SoftwareCategory::DDoS => 2.2,
            SoftwareCategory::Miner => 1.7,
            SoftwareCategory::Exploit => 2.5,
            _ => 1.0,
        };

        version_factor * category_multiplier * 100.0
    }
}

/// Software inventory for a user/server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareInventory {
    pub owner_id: Uuid,
    pub software: HashMap<Uuid, Software>,
    pub installed: HashSet<Uuid>,
    pub hidden: HashSet<Uuid>,
    pub disk_used: f32,
    pub disk_capacity: f32,
}

impl SoftwareInventory {
    pub fn new(owner_id: Uuid, disk_capacity: f32) -> Self {
        Self {
            owner_id,
            software: HashMap::new(),
            installed: HashSet::new(),
            hidden: HashSet::new(),
            disk_used: 0.0,
            disk_capacity,
        }
    }

    pub fn add_software(&mut self, software: Software) -> EngineResult<()> {
        if self.disk_used + software.size_mb > self.disk_capacity {
            return Err(EngineError::ResourceExhausted("Insufficient disk space".into()));
        }

        self.disk_used += software.size_mb;
        self.software.insert(software.id, software);
        Ok(())
    }

    pub fn remove_software(&mut self, id: Uuid) -> EngineResult<Software> {
        match self.software.remove(&id) {
            Some(software) => {
                self.disk_used -= software.size_mb;
                self.installed.remove(&id);
                self.hidden.remove(&id);
                Ok(software)
            }
            None => Err(EngineError::NotFound("Software not found".into())),
        }
    }

    pub fn install(&mut self, id: Uuid) -> EngineResult<()> {
        match self.software.get_mut(&id) {
            Some(software) => {
                software.installed = true;
                self.installed.insert(id);
                Ok(())
            }
            None => Err(EngineError::NotFound("Software not found".into())),
        }
    }

    pub fn uninstall(&mut self, id: Uuid) -> EngineResult<()> {
        match self.software.get_mut(&id) {
            Some(software) => {
                software.installed = false;
                self.installed.remove(&id);
                Ok(())
            }
            None => Err(EngineError::NotFound("Software not found".into())),
        }
    }

    pub fn hide(&mut self, id: Uuid) -> EngineResult<()> {
        match self.software.get_mut(&id) {
            Some(software) => {
                software.hidden = true;
                self.hidden.insert(id);
                Ok(())
            }
            None => Err(EngineError::NotFound("Software not found".into())),
        }
    }

    pub fn seek_hidden(&mut self) -> Vec<Uuid> {
        let hidden: Vec<Uuid> = self.hidden.iter().cloned().collect();
        for id in &hidden {
            if let Some(software) = self.software.get_mut(&id) {
                software.hidden = false;
            }
        }
        self.hidden.clear();
        hidden
    }

    pub fn get_best(&self, category: SoftwareCategory) -> Option<&Software> {
        self.software.values()
            .filter(|s| s.category == category && s.installed)
            .max_by_key(|s| &s.version)
    }
}

/// Dependency resolver
pub struct DependencyResolver;

impl DependencyResolver {
    pub fn resolve(
        software: &Software,
        inventory: &SoftwareInventory,
    ) -> EngineResult<Vec<Uuid>> {
        let mut required = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(software);
        visited.insert(software.id);

        while let Some(current) = queue.pop_front() {
            for dep in &current.dependencies {
                let found = inventory.software.values()
                    .find(|s| {
                        s.name == dep.software_name &&
                        s.category == dep.category &&
                        s.version.is_compatible(&dep.min_version) &&
                        s.installed
                    });

                match found {
                    Some(dep_software) => {
                        if !visited.contains(&dep_software.id) {
                            visited.insert(dep_software.id);
                            queue.push_back(dep_software);
                        }
                    }
                    None => {
                        return Err(EngineError::NotFound(format!(
                            "Missing dependency: {} v{}",
                            dep.software_name,
                            dep.min_version.to_string()
                        )));
                    }
                }
            }
        }

        Ok(required)
    }

    pub fn check_conflicts(
        software: &Software,
        inventory: &SoftwareInventory,
    ) -> Vec<String> {
        let mut conflicts = Vec::new();

        // Check for conflicting software (e.g., multiple antivirus)
        if software.category == SoftwareCategory::AntiVirus {
            for existing in inventory.software.values() {
                if existing.category == SoftwareCategory::AntiVirus &&
                   existing.installed &&
                   existing.id != software.id {
                    conflicts.push(format!(
                        "Conflicts with installed antivirus: {}",
                        existing.name
                    ));
                }
            }
        }

        conflicts
    }
}

/// Main Software Engine
pub struct SoftwareEngine {
    inventories: HashMap<Uuid, SoftwareInventory>,
    software_library: HashMap<String, Software>,
    resolver: DependencyResolver,
    last_update: SystemTime,
}

impl SoftwareEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            inventories: HashMap::new(),
            software_library: HashMap::new(),
            resolver: DependencyResolver,
            last_update: SystemTime::now(),
        };
        engine.initialize_library();
        engine
    }

    fn initialize_library(&mut self) {
        // Crackers
        self.add_to_library(Software::new(
            "Basic Cracker".to_string(),
            SoftwareCategory::Cracker,
            SoftwareVersion::new(1, 0, 0),
        ));
        self.add_to_library(Software::new(
            "Advanced Cracker".to_string(),
            SoftwareCategory::Cracker,
            SoftwareVersion::new(2, 0, 0),
        ));
        self.add_to_library(Software::new(
            "Elite Cracker".to_string(),
            SoftwareCategory::Cracker,
            SoftwareVersion::new(3, 0, 0),
        ));

        // Firewalls
        self.add_to_library(Software::new(
            "Basic Firewall".to_string(),
            SoftwareCategory::Firewall,
            SoftwareVersion::new(1, 0, 0),
        ));
        self.add_to_library(Software::new(
            "Advanced Firewall".to_string(),
            SoftwareCategory::Firewall,
            SoftwareVersion::new(2, 0, 0),
        ));

        // AntiVirus
        self.add_to_library(Software::new(
            "Norton AntiVirus".to_string(),
            SoftwareCategory::AntiVirus,
            SoftwareVersion::new(1, 5, 0),
        ));
        self.add_to_library(Software::new(
            "Kaspersky AntiVirus".to_string(),
            SoftwareCategory::AntiVirus,
            SoftwareVersion::new(2, 0, 0),
        ));

        // Exploits
        self.add_to_library(Software::new(
            "FTP Exploit".to_string(),
            SoftwareCategory::Exploit,
            SoftwareVersion::new(1, 0, 0),
        ));
        self.add_to_library(Software::new(
            "SSH Exploit".to_string(),
            SoftwareCategory::Exploit,
            SoftwareVersion::new(1, 2, 0),
        ));

        // Miners
        self.add_to_library(Software::new(
            "Bitcoin Miner".to_string(),
            SoftwareCategory::Miner,
            SoftwareVersion::new(1, 0, 0),
        ));
        self.add_to_library(Software::new(
            "Ethereum Miner".to_string(),
            SoftwareCategory::Miner,
            SoftwareVersion::new(1, 5, 0),
        ));

        // Add dependencies
        if let Some(ssh_exploit) = self.software_library.get_mut("SSH Exploit") {
            ssh_exploit.add_dependency(SoftwareDependency {
                software_name: "Basic Cracker".to_string(),
                category: SoftwareCategory::Cracker,
                min_version: SoftwareVersion::new(1, 0, 0),
            });
        }
    }

    fn add_to_library(&mut self, software: Software) {
        self.software_library.insert(software.name.clone(), software);
    }

    pub fn create_inventory(&mut self, owner_id: Uuid, disk_capacity: f32) -> Uuid {
        let inventory = SoftwareInventory::new(owner_id, disk_capacity);
        self.inventories.insert(owner_id, inventory);
        owner_id
    }

    pub fn get_inventory(&self, owner_id: Uuid) -> Option<&SoftwareInventory> {
        self.inventories.get(&owner_id)
    }

    pub fn get_inventory_mut(&mut self, owner_id: Uuid) -> Option<&mut SoftwareInventory> {
        self.inventories.get_mut(&owner_id)
    }

    pub fn research_software(
        &mut self,
        owner_id: Uuid,
        software_name: &str,
    ) -> EngineResult<Software> {
        let template = self.software_library.get(software_name)
            .ok_or_else(|| EngineError::NotFound("Software not in library".into()))?
            .clone();

        let inventory = self.inventories.get_mut(&owner_id)
            .ok_or_else(|| EngineError::NotFound("Inventory not found".into()))?;

        inventory.add_software(template.clone())?;
        Ok(template)
    }

    pub fn install_software(
        &mut self,
        owner_id: Uuid,
        software_id: Uuid,
    ) -> EngineResult<Vec<Uuid>> {
        let inventory = self.inventories.get(&owner_id)
            .ok_or_else(|| EngineError::NotFound("Inventory not found".into()))?;

        let software = inventory.software.get(&software_id)
            .ok_or_else(|| EngineError::NotFound("Software not found".into()))?;

        // Check dependencies
        let deps = DependencyResolver::resolve(software, inventory)?;

        // Check conflicts
        let conflicts = DependencyResolver::check_conflicts(software, inventory);
        if !conflicts.is_empty() {
            return Err(EngineError::InvalidOperation(conflicts.join(", ")));
        }

        // Install
        let inventory = self.inventories.get_mut(&owner_id).unwrap();
        inventory.install(software_id)?;

        Ok(deps)
    }

    pub fn get_library(&self) -> Vec<&Software> {
        self.software_library.values().collect()
    }
}

impl EngineComponent for SoftwareEngine {
    fn initialize(&mut self) -> EngineResult<()> {
        Ok(())
    }

    fn update(&mut self, _delta: Duration) -> EngineResult<()> {
        self.last_update = SystemTime::now();
        Ok(())
    }

    fn status(&self) -> ComponentStatus {
        ComponentStatus {
            name: "SoftwareEngine".to_string(),
            healthy: true,
            last_update: self.last_update,
            metrics: vec![
                ("inventories".to_string(), self.inventories.len() as f64),
                ("library_items".to_string(), self.software_library.len() as f64),
            ],
        }
    }

    fn reset(&mut self) -> EngineResult<()> {
        self.inventories.clear();
        self.initialize_library();
        self.last_update = SystemTime::now();
        Ok(())
    }
}