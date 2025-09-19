//! # Core Process Execution Engine
//!
//! This crate provides the process management functionality for the HackerExperience
//! game engine, including process lifecycle, resource allocation, and execution scheduling.
//!
//! ## Architecture
//!
//! The process system is built around several key concepts:
//! - **Process**: Individual executable tasks with resource requirements
//! - **Processable**: Trait for defining process behavior and lifecycle
//! - **Resource Allocation**: Dynamic CPU, RAM, HDD, and network resource management
//! - **Scheduler**: Task scheduling and execution coordination
//! - **TOP**: System monitoring and process listing functionality
//!
//! ## Key Features
//!
//! - Async/await support using Tokio
//! - Actor-based architecture using Actix
//! - Database persistence with SeaORM
//! - Dynamic resource allocation and monitoring
//! - Process signal handling and lifecycle management
//! - Real-time process scheduling and execution

pub mod actors;
pub mod allocator;
pub mod error;
pub mod model;
pub mod processable;
pub mod query;
pub mod resources;
pub mod scheduler;
pub mod signals;
pub mod top;
pub mod types;

pub use model::{Process, ProcessableType};
pub use processable::Processable;
pub use resources::ProcessResources;
pub use types::*;

use anyhow::Result;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Global process registry for managing active processes
pub static PROCESS_REGISTRY: Lazy<Arc<RwLock<ProcessRegistry>>> = 
    Lazy::new(|| Arc::new(RwLock::new(ProcessRegistry::new())));

/// Process registry for tracking active processes and resource allocation
#[derive(Debug, Default)]
pub struct ProcessRegistry {
    processes: dashmap::DashMap<ProcessId, Arc<Process>>,
    resource_allocations: dashmap::DashMap<ProcessId, ProcessResources>,
    process_hierarchy: dashmap::DashMap<ProcessId, Vec<ProcessId>>, // parent -> children
}

impl ProcessRegistry {
    pub fn new() -> Self {
        Self {
            processes: dashmap::DashMap::new(),
            resource_allocations: dashmap::DashMap::new(),
            process_hierarchy: dashmap::DashMap::new(),
        }
    }

    pub async fn register_process(&self, process: Process) -> Arc<Process> {
        let process_arc = Arc::new(process);
        self.processes.insert(process_arc.process_id.clone(), process_arc.clone());
        process_arc
    }

    pub async fn allocate_resources(&self, process_id: ProcessId, resources: ProcessResources) {
        self.resource_allocations.insert(process_id, resources);
    }

    pub async fn deallocate_resources(&self, process_id: &ProcessId) -> Option<ProcessResources> {
        self.resource_allocations.remove(process_id).map(|(_, resources)| resources)
    }

    pub async fn get_process(&self, process_id: &ProcessId) -> Option<Arc<Process>> {
        self.processes.get(process_id).map(|entry| entry.clone())
    }

    pub async fn get_resource_allocation(&self, process_id: &ProcessId) -> Option<ProcessResources> {
        self.resource_allocations.get(process_id).map(|entry| entry.clone())
    }

    pub async fn remove_process(&self, process_id: &ProcessId) -> Option<Arc<Process>> {
        // Remove from hierarchy
        self.process_hierarchy.remove(process_id);
        
        // Remove resource allocation
        self.resource_allocations.remove(process_id);
        
        // Remove the process itself
        self.processes.remove(process_id).map(|(_, process)| process)
    }

    pub async fn list_processes(&self) -> Vec<Arc<Process>> {
        self.processes.iter().map(|entry| entry.value().clone()).collect()
    }

    pub async fn list_active_processes(&self) -> Vec<Arc<Process>> {
        self.processes
            .iter()
            .filter(|entry| entry.is_active())
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get processes by type
    pub async fn get_processes_by_type(&self, process_type: ProcessType) -> Vec<Arc<Process>> {
        self.processes
            .iter()
            .filter(|entry| entry.process_type == process_type)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get processes running on a specific server
    pub async fn get_server_processes(&self, server_id: &he_core_server::ServerId) -> Vec<Arc<Process>> {
        self.processes
            .iter()
            .filter(|entry| &entry.gateway_id == server_id || &entry.target_id == server_id)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get child processes of a parent process
    pub async fn get_child_processes(&self, parent_id: &ProcessId) -> Vec<Arc<Process>> {
        if let Some(children) = self.process_hierarchy.get(parent_id) {
            children
                .iter()
                .filter_map(|child_id| self.processes.get(child_id).map(|entry| entry.value().clone()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Add a child process to a parent
    pub async fn add_child_process(&self, parent_id: ProcessId, child_id: ProcessId) {
        self.process_hierarchy
            .entry(parent_id)
            .or_insert_with(Vec::new)
            .push(child_id);
    }

    /// Remove a child process from a parent
    pub async fn remove_child_process(&self, parent_id: &ProcessId, child_id: &ProcessId) {
        if let Some(mut children) = self.process_hierarchy.get_mut(parent_id) {
            children.retain(|id| id != child_id);
        }
    }

    /// Get total resource usage across all processes
    pub async fn get_total_resource_usage(&self) -> ProcessResources {
        self.resource_allocations
            .iter()
            .fold(ProcessResources::new(), |acc, entry| acc + entry.value().clone())
    }

    /// Get resource usage for a specific server
    pub async fn get_server_resource_usage(&self, server_id: &he_core_server::ServerId) -> ProcessResources {
        self.processes
            .iter()
            .filter(|entry| &entry.gateway_id == server_id || &entry.target_id == server_id)
            .filter_map(|entry| self.resource_allocations.get(&entry.process_id))
            .fold(ProcessResources::new(), |acc, entry| acc + entry.value().clone())
    }
}

/// Initialize the process subsystem
pub async fn init() -> Result<()> {
    tracing::info!("Initializing Core Process subsystem");
    
    // Initialize the process registry
    let _registry = PROCESS_REGISTRY.clone();
    
    // Start the process scheduler
    scheduler::start_scheduler().await?;
    
    tracing::info!("Core Process subsystem initialized successfully");
    Ok(())
}

/// Shutdown the process subsystem gracefully
pub async fn shutdown() -> Result<()> {
    tracing::info!("Shutting down Core Process subsystem");
    
    // Stop the process scheduler
    scheduler::stop_scheduler().await?;
    
    // Clear all registries
    let registry = PROCESS_REGISTRY.read().await;
    registry.processes.clear();
    registry.resource_allocations.clear();
    registry.process_hierarchy.clear();
    
    tracing::info!("Core Process subsystem shutdown complete");
    Ok(())
}