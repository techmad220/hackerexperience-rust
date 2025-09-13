use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::{
    ProcessId, UserId, SoftwareId, IpAddress, ProcessAction, 
    HeResult, HeError
};

// Mapping from PHP Process.class.php
// "This is the most complex part of Legacy and HE2." - Original comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    pub id: ProcessId,
    pub creator_id: UserId,      // pCreatorID
    pub victim_id: Option<UserId>, // pVictimID  
    pub action: ProcessAction,   // pAction
    pub software_id: Option<SoftwareId>, // pSoftID
    pub target_ip: IpAddress,    // pLocal (confusingly named in original)
    pub time_left: i32,          // pTimeLeft in seconds
    pub info: Option<String>,    // pInfo - JSON or serialized data
    pub info_str: Option<String>, // pInfoStr - human readable info
    pub is_npc: bool,           // pNPC
    pub cpu_usage: i32,         // CPU required
    pub net_usage: i32,         // Network bandwidth required
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: ProcessStatus,
    pub priority: i32,          // Process priority (0-10)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessStatus {
    Pending,    // Not started yet
    Running,    // Currently executing
    Paused,     // User paused the process
    Completed,  // Successfully finished
    Failed,     // Failed due to error
    Cancelled,  // User cancelled
}

impl Process {
    pub fn new(
        creator_id: UserId,
        victim_id: Option<UserId>,
        action: ProcessAction,
        target_ip: IpAddress,
        software_id: Option<SoftwareId>,
        duration_seconds: i32,
    ) -> Self {
        Self {
            id: 0, // Set by database
            creator_id,
            victim_id,
            action,
            software_id,
            target_ip,
            time_left: duration_seconds,
            info: None,
            info_str: None,
            is_npc: false,
            cpu_usage: Self::calculate_cpu_usage(action),
            net_usage: Self::calculate_net_usage(action),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            status: ProcessStatus::Pending,
            priority: 5, // Default priority
        }
    }
    
    pub fn start(&mut self) -> HeResult<()> {
        if self.status != ProcessStatus::Pending {
            return Err(HeError::InvalidProcess(
                format!("Cannot start process in status {:?}", self.status)
            ));
        }
        
        self.status = ProcessStatus::Running;
        self.started_at = Some(Utc::now());
        Ok(())
    }
    
    pub fn pause(&mut self) -> HeResult<()> {
        if self.status != ProcessStatus::Running {
            return Err(HeError::InvalidProcess(
                format!("Cannot pause process in status {:?}", self.status)
            ));
        }
        
        self.status = ProcessStatus::Paused;
        Ok(())
    }
    
    pub fn resume(&mut self) -> HeResult<()> {
        if self.status != ProcessStatus::Paused {
            return Err(HeError::InvalidProcess(
                format!("Cannot resume process in status {:?}", self.status)
            ));
        }
        
        self.status = ProcessStatus::Running;
        Ok(())
    }
    
    pub fn complete(&mut self) -> HeResult<()> {
        if self.status != ProcessStatus::Running {
            return Err(HeError::InvalidProcess(
                format!("Cannot complete process in status {:?}", self.status)
            ));
        }
        
        self.status = ProcessStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.time_left = 0;
        Ok(())
    }
    
    pub fn fail(&mut self, reason: String) -> HeResult<()> {
        self.status = ProcessStatus::Failed;
        self.info_str = Some(reason);
        self.completed_at = Some(Utc::now());
        Ok(())
    }
    
    pub fn cancel(&mut self) -> HeResult<()> {
        if matches!(self.status, ProcessStatus::Completed | ProcessStatus::Failed) {
            return Err(HeError::InvalidProcess(
                "Cannot cancel completed or failed process".to_string()
            ));
        }
        
        self.status = ProcessStatus::Cancelled;
        self.completed_at = Some(Utc::now());
        Ok(())
    }
    
    pub fn tick(&mut self, seconds: i32) -> HeResult<bool> {
        if self.status != ProcessStatus::Running {
            return Ok(false); // Not running, no progress
        }
        
        self.time_left = (self.time_left - seconds).max(0);
        
        if self.time_left == 0 {
            self.complete()?;
            Ok(true) // Process completed
        } else {
            Ok(false) // Still running
        }
    }
    
    pub fn progress(&self) -> f64 {
        if let Some(started_at) = self.started_at {
            let elapsed = Utc::now().timestamp() - started_at.timestamp();
            let total = elapsed + self.time_left as i64;
            if total > 0 {
                elapsed as f64 / total as f64
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
    
    pub fn is_active(&self) -> bool {
        matches!(self.status, ProcessStatus::Running)
    }
    
    pub fn is_finished(&self) -> bool {
        matches!(
            self.status, 
            ProcessStatus::Completed | ProcessStatus::Failed | ProcessStatus::Cancelled
        )
    }
    
    // Calculate CPU usage based on process type
    fn calculate_cpu_usage(action: ProcessAction) -> i32 {
        match action {
            ProcessAction::Download | ProcessAction::Upload => 10,
            ProcessAction::Hack | ProcessAction::BankHack => 50,
            ProcessAction::Format => 80,
            ProcessAction::Research => 30,
            ProcessAction::Ddos => 60,
            _ => 20,
        }
    }
    
    // Calculate network usage based on process type
    fn calculate_net_usage(action: ProcessAction) -> i32 {
        match action {
            ProcessAction::Download | ProcessAction::Upload => 50,
            ProcessAction::Ddos => 90,
            ProcessAction::Nmap | ProcessAction::PortScan => 30,
            _ => 10,
        }
    }
}

// Paused process data - for processes that can be paused/resumed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PausedProcess {
    pub process_id: ProcessId,
    pub paused_at: DateTime<Utc>,
    pub time_remaining: i32,
    pub pause_data: Option<String>, // Additional data needed to resume
}

impl PausedProcess {
    pub fn new(process_id: ProcessId, time_remaining: i32) -> Self {
        Self {
            process_id,
            paused_at: Utc::now(),
            time_remaining,
            pause_data: None,
        }
    }
    
    pub fn paused_duration(&self) -> i64 {
        Utc::now().timestamp() - self.paused_at.timestamp()
    }
}

// Process queue for managing multiple processes per user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessQueue {
    pub user_id: UserId,
    pub active_processes: Vec<ProcessId>,
    pub max_concurrent: i32,
}

impl ProcessQueue {
    pub fn new(user_id: UserId, max_concurrent: i32) -> Self {
        Self {
            user_id,
            active_processes: Vec::new(),
            max_concurrent,
        }
    }
    
    pub fn can_add_process(&self) -> bool {
        self.active_processes.len() < self.max_concurrent as usize
    }
    
    pub fn add_process(&mut self, process_id: ProcessId) -> HeResult<()> {
        if !self.can_add_process() {
            return Err(HeError::InsufficientResources(
                "Too many active processes".to_string()
            ));
        }
        
        self.active_processes.push(process_id);
        Ok(())
    }
    
    pub fn remove_process(&mut self, process_id: ProcessId) {
        self.active_processes.retain(|&id| id != process_id);
    }
    
    pub fn is_full(&self) -> bool {
        self.active_processes.len() >= self.max_concurrent as usize
    }
}