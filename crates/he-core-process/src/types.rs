//! Type definitions for the process module

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use he_core_server::ServerId;
use he_core_network::{ConnectionId, NetworkId};
use he_core_software::FileId;

/// Process unique identifier
pub type ProcessId = Uuid;

/// Entity identifier (from entity service)
pub type EntityId = Uuid;

/// Bank account identifier
pub type BankAccount = String;

/// Process type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessType {
    /// File upload process
    FileUpload,
    /// File download process
    FileDownload,
    /// Password brute force cracking
    CrackerBruteforce,
    /// Buffer overflow exploitation
    CrackerOverflow,
    /// Virus installation
    InstallVirus,
    /// Virus data collection
    VirusCollect,
    /// Bank password revelation
    BankRevealPassword,
    /// Wire transfer operation
    WireTransfer,
    /// Log manipulation/forging
    LogForger,
}

impl ProcessType {
    pub fn all_types() -> &'static [ProcessType] {
        &[
            ProcessType::FileUpload,
            ProcessType::FileDownload,
            ProcessType::CrackerBruteforce,
            ProcessType::CrackerOverflow,
            ProcessType::InstallVirus,
            ProcessType::VirusCollect,
            ProcessType::BankRevealPassword,
            ProcessType::WireTransfer,
            ProcessType::LogForger,
        ]
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessType::FileUpload => "file_upload",
            ProcessType::FileDownload => "file_download",
            ProcessType::CrackerBruteforce => "cracker_bruteforce",
            ProcessType::CrackerOverflow => "cracker_overflow",
            ProcessType::InstallVirus => "install_virus",
            ProcessType::VirusCollect => "virus_collect",
            ProcessType::BankRevealPassword => "bank_reveal_password",
            ProcessType::WireTransfer => "wire_transfer",
            ProcessType::LogForger => "log_forger",
        }
    }
    
    pub fn is_file_operation(&self) -> bool {
        matches!(self, ProcessType::FileUpload | ProcessType::FileDownload)
    }
    
    pub fn is_attack(&self) -> bool {
        matches!(
            self,
            ProcessType::CrackerBruteforce
                | ProcessType::CrackerOverflow
                | ProcessType::InstallVirus
        )
    }
    
    pub fn is_virus_related(&self) -> bool {
        matches!(self, ProcessType::InstallVirus | ProcessType::VirusCollect)
    }
    
    pub fn is_bank_operation(&self) -> bool {
        matches!(self, ProcessType::BankRevealPassword | ProcessType::WireTransfer)
    }
}

impl std::fmt::Display for ProcessType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Process state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessState {
    /// Process is queued but not yet started
    Pending,
    /// Process is currently running
    Running,
    /// Process is paused/suspended
    Paused,
    /// Process completed successfully
    Completed,
    /// Process failed with error
    Failed,
    /// Process was killed/terminated
    Killed,
}

impl ProcessState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessState::Pending => "pending",
            ProcessState::Running => "running",
            ProcessState::Paused => "paused",
            ProcessState::Completed => "completed",
            ProcessState::Failed => "failed",
            ProcessState::Killed => "killed",
        }
    }
    
    pub fn is_active(&self) -> bool {
        matches!(self, ProcessState::Running | ProcessState::Paused)
    }
    
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ProcessState::Completed | ProcessState::Failed | ProcessState::Killed
        )
    }
}

impl std::fmt::Display for ProcessState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Process priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ProcessPriority {
    Low = 1,
    Normal = 5,
    High = 10,
    Critical = 15,
}

impl ProcessPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessPriority::Low => "low",
            ProcessPriority::Normal => "normal",
            ProcessPriority::High => "high",
            ProcessPriority::Critical => "critical",
        }
    }
    
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

impl Default for ProcessPriority {
    fn default() -> Self {
        ProcessPriority::Normal
    }
}

impl std::fmt::Display for ProcessPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Process signal types for lifecycle management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProcessSignal {
    /// Process reached its objective
    Sigterm,
    /// Process should pause execution
    Sigstop,
    /// Process should resume execution
    Sigcont,
    /// Process should terminate immediately
    Sigkill,
    /// Process received updated data
    Sigupdate,
    /// Process checkpoint/save state
    Sigcheckpoint,
}

impl ProcessSignal {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessSignal::Sigterm => "SIGTERM",
            ProcessSignal::Sigstop => "SIGSTOP",
            ProcessSignal::Sigcont => "SIGCONT",
            ProcessSignal::Sigkill => "SIGKILL",
            ProcessSignal::Sigupdate => "SIGUPDATE",
            ProcessSignal::Sigcheckpoint => "SIGCHECKPOINT",
        }
    }
}

impl std::fmt::Display for ProcessSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Process signal response actions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalResponse {
    /// Delete the process
    Delete,
    /// Continue with next target
    FindNextTarget,
    /// Pause the process
    Pause,
    /// Resume the process
    Resume,
    /// Update process state
    Update(serde_json::Value),
    /// No action needed
    NoAction,
}

/// Process creation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessCreationParams {
    pub process_type: ProcessType,
    pub gateway_id: ServerId,
    pub source_entity_id: EntityId,
    pub target_id: ServerId,
    pub priority: Option<ProcessPriority>,
    pub data: Option<serde_json::Value>,
    
    // Optional foreign key relationships
    pub network_id: Option<NetworkId>,
    pub src_connection_id: Option<ConnectionId>,
    pub src_file_id: Option<FileId>,
    pub src_atm_id: Option<ServerId>,
    pub src_acc_number: Option<BankAccount>,
    pub tgt_file_id: Option<FileId>,
    pub tgt_connection_id: Option<ConnectionId>,
    pub tgt_atm_id: Option<ServerId>,
    pub tgt_acc_number: Option<BankAccount>,
    pub tgt_process_id: Option<ProcessId>,
}

/// Resource allocation limits for processes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu: u32,
    pub ram: u64,
    pub hdd: u64,
    pub net: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu: 0,
            ram: 0,
            hdd: 0,
            net: 0,
        }
    }
}

/// Dynamic resource allocation that can change over time
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicResourceAllocation {
    pub base_allocation: ResourceLimits,
    pub multiplier: f32,
    pub last_updated: DateTime<Utc>,
}

impl DynamicResourceAllocation {
    pub fn new(base_allocation: ResourceLimits) -> Self {
        Self {
            base_allocation,
            multiplier: 1.0,
            last_updated: Utc::now(),
        }
    }
    
    pub fn current_allocation(&self) -> ResourceLimits {
        ResourceLimits {
            cpu: (self.base_allocation.cpu as f32 * self.multiplier) as u32,
            ram: (self.base_allocation.ram as f32 * self.multiplier) as u64,
            hdd: (self.base_allocation.hdd as f32 * self.multiplier) as u64,
            net: (self.base_allocation.net as f32 * self.multiplier) as u32,
        }
    }
    
    pub fn update_multiplier(&mut self, multiplier: f32) {
        self.multiplier = multiplier;
        self.last_updated = Utc::now();
    }
}

/// Process execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessContext {
    pub process_id: ProcessId,
    pub gateway_id: ServerId,
    pub target_id: ServerId,
    pub source_entity_id: EntityId,
    pub allocated_resources: ResourceLimits,
    pub execution_start_time: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Process progress tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessProgress {
    pub percentage: u8, // 0-100
    pub current_step: String,
    pub steps_completed: u32,
    pub total_steps: u32,
    pub bytes_processed: Option<u64>,
    pub total_bytes: Option<u64>,
    pub last_updated: DateTime<Utc>,
}

impl ProcessProgress {
    pub fn new() -> Self {
        Self {
            percentage: 0,
            current_step: "Initializing".to_string(),
            steps_completed: 0,
            total_steps: 1,
            bytes_processed: None,
            total_bytes: None,
            last_updated: Utc::now(),
        }
    }
    
    pub fn update_percentage(&mut self, percentage: u8) {
        self.percentage = percentage.min(100);
        self.last_updated = Utc::now();
    }
    
    pub fn update_step(&mut self, step: String, completed: u32, total: u32) {
        self.current_step = step;
        self.steps_completed = completed;
        self.total_steps = total;
        self.percentage = if total > 0 { ((completed * 100) / total) as u8 } else { 0 };
        self.last_updated = Utc::now();
    }
    
    pub fn is_complete(&self) -> bool {
        self.percentage >= 100
    }
}

impl Default for ProcessProgress {
    fn default() -> Self {
        Self::new()
    }
}

/// Process statistics for monitoring and analysis
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessStats {
    pub process_id: ProcessId,
    pub cpu_usage_percent: u8,
    pub memory_usage_bytes: u64,
    pub disk_io_bytes: u64,
    pub network_io_bytes: u64,
    pub uptime_seconds: u64,
    pub last_checkpoint: Option<DateTime<Utc>>,
    pub recorded_at: DateTime<Utc>,
}