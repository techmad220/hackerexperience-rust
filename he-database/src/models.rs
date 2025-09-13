//! Database models and entities for the Helix system

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::repository::{AuditInfo, Auditable};

/// Base trait for all database entities
pub trait Entity {
    /// The type of the entity's ID
    type Id;
    
    /// Get the entity's ID
    fn id(&self) -> &Self::Id;
    
    /// Set the entity's ID
    fn set_id(&mut self, id: Self::Id);
}

/// Account entity representing user accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub is_active: bool,
    pub email_verified: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub preferences: serde_json::Value,
    pub audit: AuditInfo,
}

impl Entity for Account {
    type Id = Uuid;
    
    fn id(&self) -> &Self::Id {
        &self.id
    }
    
    fn set_id(&mut self, id: Self::Id) {
        self.id = id;
    }
}

impl Auditable for Account {
    fn audit_info(&self) -> &AuditInfo {
        &self.audit
    }
    
    fn audit_info_mut(&mut self) -> &mut AuditInfo {
        &mut self.audit
    }
}

/// Server entity representing game servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub hostname: Option<String>,
    pub ip_address: String,
    pub server_type: ServerType,
    pub operating_system: String,
    pub hardware_specs: serde_json::Value,
    pub location: Option<String>,
    pub is_online: bool,
    pub last_seen: Option<DateTime<Utc>>,
    pub reputation: i32,
    pub audit: AuditInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerType {
    Desktop,
    Laptop,
    Server,
    Mainframe,
    Mobile,
    IoT,
}

impl Entity for Server {
    type Id = Uuid;
    
    fn id(&self) -> &Self::Id {
        &self.id
    }
    
    fn set_id(&mut self, id: Self::Id) {
        self.id = id;
    }
}

impl Auditable for Server {
    fn audit_info(&self) -> &AuditInfo {
        &self.audit
    }
    
    fn audit_info_mut(&mut self) -> &mut AuditInfo {
        &mut self.audit
    }
}

/// Network entity representing network connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub id: Uuid,
    pub server_id: Uuid,
    pub name: String,
    pub network_type: NetworkType,
    pub ip_range: String,
    pub gateway: String,
    pub dns_servers: Vec<String>,
    pub security_level: i32,
    pub encryption_level: i32,
    pub is_active: bool,
    pub audit: AuditInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkType {
    LAN,
    WAN,
    VPN,
    Tor,
    Satellite,
}

impl Entity for Network {
    type Id = Uuid;
    
    fn id(&self) -> &Self::Id {
        &self.id
    }
    
    fn set_id(&mut self, id: Self::Id) {
        self.id = id;
    }
}

impl Auditable for Network {
    fn audit_info(&self) -> &AuditInfo {
        &self.audit
    }
    
    fn audit_info_mut(&mut self) -> &mut AuditInfo {
        &mut self.audit
    }
}

/// Software entity representing installed software
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Software {
    pub id: Uuid,
    pub server_id: Uuid,
    pub name: String,
    pub version: String,
    pub software_type: SoftwareType,
    pub size: u64,
    pub install_path: String,
    pub is_running: bool,
    pub permissions: Vec<String>,
    pub dependencies: Vec<Uuid>,
    pub configuration: serde_json::Value,
    pub audit: AuditInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SoftwareType {
    OS,
    Firewall,
    Antivirus,
    Cracker,
    LogDeleter,
    LogForger,
    Encryptor,
    Decryptor,
    ProxyServer,
    GameClient,
    Custom,
}

impl Entity for Software {
    type Id = Uuid;
    
    fn id(&self) -> &Self::Id {
        &self.id
    }
    
    fn set_id(&mut self, id: Self::Id) {
        self.id = id;
    }
}

impl Auditable for Software {
    fn audit_info(&self) -> &AuditInfo {
        &self.audit
    }
    
    fn audit_info_mut(&mut self) -> &mut AuditInfo {
        &mut self.audit
    }
}

/// Hardware entity representing server hardware components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hardware {
    pub id: Uuid,
    pub server_id: Uuid,
    pub component_type: HardwareType,
    pub name: String,
    pub model: String,
    pub specifications: serde_json::Value,
    pub performance_rating: i32,
    pub power_consumption: u32,
    pub is_functional: bool,
    pub wear_level: f32,
    pub audit: AuditInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HardwareType {
    CPU,
    RAM,
    HDD,
    SSD,
    NetworkCard,
    GPU,
    Motherboard,
    PowerSupply,
    CoolingSystem,
}

impl Entity for Hardware {
    type Id = Uuid;
    
    fn id(&self) -> &Self::Id {
        &self.id
    }
    
    fn set_id(&mut self, id: Self::Id) {
        self.id = id;
    }
}

impl Auditable for Hardware {
    fn audit_info(&self) -> &AuditInfo {
        &self.audit
    }
    
    fn audit_info_mut(&mut self) -> &mut AuditInfo {
        &mut self.audit
    }
}

/// Process entity representing running processes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    pub id: Uuid,
    pub server_id: Uuid,
    pub software_id: Option<Uuid>,
    pub process_name: String,
    pub process_type: ProcessType,
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub status: ProcessStatus,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub priority: i32,
    pub parameters: serde_json::Value,
    pub audit: AuditInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessType {
    System,
    User,
    Game,
    Network,
    Security,
    Maintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessStatus {
    Running,
    Sleeping,
    Waiting,
    Zombie,
    Stopped,
    Completed,
    Failed,
}

impl Entity for Process {
    type Id = Uuid;
    
    fn id(&self) -> &Self::Id {
        &self.id
    }
    
    fn set_id(&mut self, id: Self::Id) {
        self.id = id;
    }
}

impl Auditable for Process {
    fn audit_info(&self) -> &AuditInfo {
        &self.audit
    }
    
    fn audit_info_mut(&mut self) -> &mut AuditInfo {
        &mut self.audit
    }
}

/// Log entry entity for system and security logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: Uuid,
    pub server_id: Uuid,
    pub log_type: LogType,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
    pub is_forged: bool,
    pub is_deleted: bool,
    pub audit: AuditInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogType {
    System,
    Security,
    Access,
    Error,
    Network,
    Process,
    Game,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

impl Entity for LogEntry {
    type Id = Uuid;
    
    fn id(&self) -> &Self::Id {
        &self.id
    }
    
    fn set_id(&mut self, id: Self::Id) {
        self.id = id;
    }
}

impl Auditable for LogEntry {
    fn audit_info(&self) -> &AuditInfo {
        &self.audit
    }
    
    fn audit_info_mut(&mut self) -> &mut AuditInfo {
        &mut self.audit
    }
}

/// Mission entity representing game missions/quests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub description: String,
    pub mission_type: MissionType,
    pub difficulty: i32,
    pub status: MissionStatus,
    pub objectives: Vec<MissionObjective>,
    pub rewards: serde_json::Value,
    pub prerequisites: Vec<Uuid>,
    pub start_time: Option<DateTime<Utc>>,
    pub completion_time: Option<DateTime<Utc>>,
    pub deadline: Option<DateTime<Utc>>,
    pub audit: AuditInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MissionType {
    Tutorial,
    Story,
    Side,
    Daily,
    Weekly,
    Event,
    PvP,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MissionStatus {
    Available,
    Active,
    Completed,
    Failed,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionObjective {
    pub id: Uuid,
    pub description: String,
    pub target: String,
    pub completed: bool,
    pub progress: f32,
}

impl Entity for Mission {
    type Id = Uuid;
    
    fn id(&self) -> &Self::Id {
        &self.id
    }
    
    fn set_id(&mut self, id: Self::Id) {
        self.id = id;
    }
}

impl Auditable for Mission {
    fn audit_info(&self) -> &AuditInfo {
        &self.audit
    }
    
    fn audit_info_mut(&mut self) -> &mut AuditInfo {
        &mut self.audit
    }
}