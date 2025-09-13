//! Process model definitions

use crate::types::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use he_helix_server::ServerId;
use he_helix_network::{ConnectionId, NetworkId};
use he_helix_software::FileId;

/// Process entity model
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Process {
    pub process_id: ProcessId,
    pub gateway_id: ServerId,
    pub source_entity_id: EntityId,
    pub target_id: ServerId,
    pub process_type: ProcessType,
    pub data: Option<serde_json::Value>,
    
    // Optional foreign keys
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
    
    // State and progress
    pub state: ProcessState,
    pub priority: ProcessPriority,
    pub progress: ProcessProgress,
    
    // Resource allocation
    pub l_allocated: Option<ResourceLimits>,
    pub r_allocated: Option<ResourceLimits>,
    pub l_limit: ResourceLimits,
    pub r_limit: ResourceLimits,
    pub l_reserved: ResourceLimits,
    pub r_reserved: ResourceLimits,
    
    // Timing
    pub creation_time: DateTime<Utc>,
    pub last_checkpoint_time: DateTime<Utc>,
    pub completion_date: Option<DateTime<Utc>>,
    pub time_left: Option<u64>, // seconds
    
    // Flags
    pub local: Option<bool>,
    
    // Dynamic allocation
    pub next_allocation: Option<ResourceLimits>,
    pub l_dynamic: Option<DynamicResourceAllocation>,
    pub r_dynamic: Option<DynamicResourceAllocation>,
    
    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Process {
    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }
    
    pub fn is_completed(&self) -> bool {
        matches!(self.state, ProcessState::Completed)
    }
    
    pub fn is_failed(&self) -> bool {
        matches!(self.state, ProcessState::Failed)
    }
}

/// Processable type enumeration for different process implementations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProcessableType {
    FileTransfer,
    Cracker,
    Virus,
    BankOperation,
    LogOperation,
}