//! Process system for managing long-running operations
//!
//! This module provides infrastructure for managing processes that represent
//! long-running game operations like file transfers, hacking attempts, etc.

use crate::{HelixError, HelixResult, ProcessId, EntityId, ServerId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

/// Process status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessStatus {
    /// Process is currently running
    Running,
    /// Process completed successfully
    Completed,
    /// Process failed with an error
    Failed,
    /// Process was cancelled
    Cancelled,
    /// Process is paused
    Paused,
}

/// Process type enumeration for different kinds of operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessType {
    /// File transfer process
    FileTransfer,
    /// Cracker/hacking process
    Cracker,
    /// Log forge process
    LogForge,
    /// Virus collection process
    VirusCollect,
    /// Software installation
    SoftwareInstall,
    /// Bank transfer process
    BankTransfer,
    /// Generic process
    Generic(String),
}

/// Process metadata and data
pub type ProcessData = HashMap<String, serde_json::Value>;

/// A process represents a long-running operation in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    /// Unique process identifier
    pub process_id: ProcessId,
    /// Entity that owns this process
    pub entity_id: EntityId,
    /// Server where the process is running
    pub server_id: ServerId,
    /// Type of process
    pub process_type: ProcessType,
    /// Current status
    pub status: ProcessStatus,
    /// When the process started
    pub started_at: DateTime<Utc>,
    /// When the process should complete (if scheduled)
    pub scheduled_completion: Option<DateTime<Utc>>,
    /// When the process actually completed
    pub completed_at: Option<DateTime<Utc>>,
    /// Process-specific data
    pub data: ProcessData,
    /// Priority level (higher = more important)
    pub priority: i32,
    /// Progress percentage (0-100)
    pub progress: u8,
}

impl Process {
    pub fn new(
        entity_id: EntityId,
        server_id: ServerId,
        process_type: ProcessType,
        data: ProcessData,
    ) -> Self {
        Self {
            process_id: ProcessId::new(),
            entity_id,
            server_id,
            process_type,
            status: ProcessStatus::Running,
            started_at: Utc::now(),
            scheduled_completion: None,
            completed_at: None,
            data,
            priority: 0,
            progress: 0,
        }
    }

    /// Mark the process as completed
    pub fn complete(&mut self) {
        self.status = ProcessStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.progress = 100;
    }

    /// Mark the process as failed
    pub fn fail(&mut self) {
        self.status = ProcessStatus::Failed;
        self.completed_at = Some(Utc::now());
    }

    /// Cancel the process
    pub fn cancel(&mut self) {
        self.status = ProcessStatus::Cancelled;
        self.completed_at = Some(Utc::now());
    }

    /// Update process progress
    pub fn update_progress(&mut self, progress: u8) {
        self.progress = progress.min(100);
    }

    /// Check if the process is finished (completed, failed, or cancelled)
    pub fn is_finished(&self) -> bool {
        matches!(
            self.status,
            ProcessStatus::Completed | ProcessStatus::Failed | ProcessStatus::Cancelled
        )
    }

    /// Check if the process is active (running or paused)
    pub fn is_active(&self) -> bool {
        matches!(self.status, ProcessStatus::Running | ProcessStatus::Paused)
    }

    /// Get the duration the process has been running
    pub fn duration(&self) -> chrono::Duration {
        match self.completed_at {
            Some(completed) => completed - self.started_at,
            None => Utc::now() - self.started_at,
        }
    }
}

/// Process registry for managing active processes
#[derive(Debug, Default)]
pub struct ProcessRegistry {
    /// All processes indexed by ID
    processes: Arc<RwLock<HashMap<ProcessId, Process>>>,
    /// Processes indexed by entity ID
    by_entity: Arc<RwLock<HashMap<EntityId, Vec<ProcessId>>>>,
    /// Processes indexed by server ID  
    by_server: Arc<RwLock<HashMap<ServerId, Vec<ProcessId>>>>,
}

impl ProcessRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new process
    pub async fn register(&self, process: Process) -> HelixResult<ProcessId> {
        let process_id = process.process_id;
        let entity_id = process.entity_id;
        let server_id = process.server_id;

        {
            let mut processes = self.processes.write().await;
            processes.insert(process_id, process);
        }

        {
            let mut by_entity = self.by_entity.write().await;
            by_entity.entry(entity_id).or_default().push(process_id);
        }

        {
            let mut by_server = self.by_server.write().await;
            by_server.entry(server_id).or_default().push(process_id);
        }

        Ok(process_id)
    }

    /// Get a process by ID
    pub async fn get(&self, process_id: ProcessId) -> Option<Process> {
        let processes = self.processes.read().await;
        processes.get(&process_id).cloned()
    }

    /// Update a process
    pub async fn update(&self, process: Process) -> HelixResult<()> {
        let mut processes = self.processes.write().await;
        processes.insert(process.process_id, process);
        Ok(())
    }

    /// Remove a process
    pub async fn remove(&self, process_id: ProcessId) -> HelixResult<()> {
        let process = {
            let mut processes = self.processes.write().await;
            processes
                .remove(&process_id)
                .ok_or_else(|| HelixError::not_found("Process not found"))?
        };

        // Remove from entity index
        {
            let mut by_entity = self.by_entity.write().await;
            if let Some(entity_processes) = by_entity.get_mut(&process.entity_id) {
                entity_processes.retain(|&id| id != process_id);
                if entity_processes.is_empty() {
                    by_entity.remove(&process.entity_id);
                }
            }
        }

        // Remove from server index
        {
            let mut by_server = self.by_server.write().await;
            if let Some(server_processes) = by_server.get_mut(&process.server_id) {
                server_processes.retain(|&id| id != process_id);
                if server_processes.is_empty() {
                    by_server.remove(&process.server_id);
                }
            }
        }

        Ok(())
    }

    /// Get all processes for an entity
    pub async fn get_by_entity(&self, entity_id: EntityId) -> Vec<Process> {
        let by_entity = self.by_entity.read().await;
        let processes = self.processes.read().await;
        
        by_entity
            .get(&entity_id)
            .map(|process_ids| {
                process_ids
                    .iter()
                    .filter_map(|&id| processes.get(&id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all processes on a server
    pub async fn get_by_server(&self, server_id: ServerId) -> Vec<Process> {
        let by_server = self.by_server.read().await;
        let processes = self.processes.read().await;
        
        by_server
            .get(&server_id)
            .map(|process_ids| {
                process_ids
                    .iter()
                    .filter_map(|&id| processes.get(&id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all active processes
    pub async fn get_active(&self) -> Vec<Process> {
        let processes = self.processes.read().await;
        processes
            .values()
            .filter(|p| p.is_active())
            .cloned()
            .collect()
    }

    /// Get all processes by status
    pub async fn get_by_status(&self, status: ProcessStatus) -> Vec<Process> {
        let processes = self.processes.read().await;
        processes
            .values()
            .filter(|p| p.status == status)
            .cloned()
            .collect()
    }

    /// Clear all processes
    pub async fn clear(&self) {
        let mut processes = self.processes.write().await;
        let mut by_entity = self.by_entity.write().await;
        let mut by_server = self.by_server.write().await;
        
        processes.clear();
        by_entity.clear();
        by_server.clear();
    }

    /// Get total number of processes
    pub async fn count(&self) -> usize {
        let processes = self.processes.read().await;
        processes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_registry() {
        let registry = ProcessRegistry::new();
        let entity_id = EntityId::new();
        let server_id = ServerId::new();

        let mut process = Process::new(
            entity_id,
            server_id,
            ProcessType::FileTransfer,
            HashMap::new(),
        );

        let process_id = registry.register(process.clone()).await.unwrap();
        assert_eq!(process_id, process.process_id);

        let found = registry.get(process_id).await;
        assert!(found.is_some());

        process.complete();
        registry.update(process.clone()).await.unwrap();

        let updated = registry.get(process_id).await.unwrap();
        assert_eq!(updated.status, ProcessStatus::Completed);

        let entity_processes = registry.get_by_entity(entity_id).await;
        assert_eq!(entity_processes.len(), 1);

        let server_processes = registry.get_by_server(server_id).await;
        assert_eq!(server_processes.len(), 1);

        registry.remove(process_id).await.unwrap();
        let found = registry.get(process_id).await;
        assert!(found.is_none());
    }
}