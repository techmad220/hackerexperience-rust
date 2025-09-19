//! Process Actor System
//!
//! This module provides actor implementations for process management,
//! including process lifecycle, resource allocation, and execution scheduling.

use crate::{Process, ProcessType, ProcessState, ProcessPriority, ProcessProgress, ProcessId, ProcessResources, Processable};
use he_helix_core::actors::{Actor, ActorContext, Handler, Message};
use he_helix_core::{HelixError, ProcessId as CoreProcessId};
use he_helix_server::ServerId;
use he_core::entities::EntityId;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::{Utc, Duration};
use std::sync::Arc;
use tracing::{info, error, warn, debug};
use serde::{Serialize, Deserialize};

/// Messages for Process Actor
#[derive(Debug)]
pub struct CreateProcess {
    pub gateway_id: ServerId,
    pub source_entity_id: EntityId,
    pub target_id: ServerId,
    pub process_type: ProcessType,
    pub data: Option<serde_json::Value>,
}

impl Message for CreateProcess {
    type Result = Result<Process, HelixError>;
}

#[derive(Debug)]
pub struct GetProcess {
    pub process_id: ProcessId,
}

impl Message for GetProcess {
    type Result = Result<Option<Process>, HelixError>;
}

#[derive(Debug)]
pub struct UpdateProcess {
    pub process_id: ProcessId,
    pub state: Option<ProcessState>,
    pub progress: Option<ProcessProgress>,
    pub data: Option<serde_json::Value>,
}

impl Message for UpdateProcess {
    type Result = Result<Process, HelixError>;
}

#[derive(Debug)]
pub struct DeleteProcess {
    pub process_id: ProcessId,
}

impl Message for DeleteProcess {
    type Result = Result<(), HelixError>;
}

#[derive(Debug)]
pub struct StartProcess {
    pub process_id: ProcessId,
}

impl Message for StartProcess {
    type Result = Result<(), HelixError>;
}

#[derive(Debug)]
pub struct PauseProcess {
    pub process_id: ProcessId,
}

impl Message for PauseProcess {
    type Result = Result<(), HelixError>;
}

#[derive(Debug)]
pub struct ResumeProcess {
    pub process_id: ProcessId,
}

impl Message for ResumeProcess {
    type Result = Result<(), HelixError>;
}

#[derive(Debug)]
pub struct KillProcess {
    pub process_id: ProcessId,
}

impl Message for KillProcess {
    type Result = Result<(), HelixError>;
}

#[derive(Debug)]
pub struct GetServerProcesses {
    pub server_id: ServerId,
}

impl Message for GetServerProcesses {
    type Result = Result<Vec<Process>, HelixError>;
}

#[derive(Debug)]
pub struct GetProcessesByType {
    pub process_type: ProcessType,
}

impl Message for GetProcessesByType {
    type Result = Result<Vec<Process>, HelixError>;
}

#[derive(Debug)]
pub struct AllocateResources {
    pub process_id: ProcessId,
    pub resources: ProcessResources,
}

impl Message for AllocateResources {
    type Result = Result<(), HelixError>;
}

#[derive(Debug)]
pub struct DeallocateResources {
    pub process_id: ProcessId,
}

impl Message for DeallocateResources {
    type Result = Result<ProcessResources, HelixError>;
}

/// Process execution context for running processes
#[derive(Debug, Clone)]
pub struct ProcessExecutionContext {
    pub process_id: ProcessId,
    pub allocated_resources: ProcessResources,
    pub start_time: chrono::DateTime<Utc>,
    pub last_checkpoint: chrono::DateTime<Utc>,
    pub execution_state: ProcessExecutionState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessExecutionState {
    Initializing,
    Running,
    Suspended,
    WaitingForResources,
    Completing,
    Failed(String),
}

/// Process Actor - manages process lifecycle and execution
#[derive(Debug)]
pub struct ProcessActor {
    /// In-memory process storage (in production, this would be database-backed)
    processes: Arc<RwLock<HashMap<ProcessId, Process>>>,
    /// Process resource allocations
    resource_allocations: Arc<RwLock<HashMap<ProcessId, ProcessResources>>>,
    /// Process execution contexts
    execution_contexts: Arc<RwLock<HashMap<ProcessId, ProcessExecutionContext>>>,
    /// Process hierarchy (parent -> children mapping)
    process_hierarchy: Arc<RwLock<HashMap<ProcessId, Vec<ProcessId>>>>,
    /// Server to processes mapping for efficient lookup
    server_processes: Arc<RwLock<HashMap<ServerId, Vec<ProcessId>>>>,
}

impl ProcessActor {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            resource_allocations: Arc::new(RwLock::new(HashMap::new())),
            execution_contexts: Arc::new(RwLock::new(HashMap::new())),
            process_hierarchy: Arc::new(RwLock::new(HashMap::new())),
            server_processes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a new unique process ID
    fn generate_process_id(&self) -> ProcessId {
        ProcessId::new()
    }

    /// Calculate process completion time based on complexity and resources
    fn calculate_completion_time(&self, process_type: &ProcessType, resources: &ProcessResources) -> Duration {
        // Base times for different process types (in seconds)
        let base_time = match process_type {
            ProcessType::FileDownload => 30,
            ProcessType::FileUpload => 45,
            ProcessType::BruteforceLogin => 120,
            ProcessType::Hacking => 300,
            ProcessType::VirusScan => 90,
            ProcessType::LogCleaning => 60,
            _ => 180, // Default for unknown types
        };

        // Adjust based on available CPU resources
        let cpu_factor = if resources.cpu > 0.0 { 1.0 / resources.cpu } else { 2.0 };
        let adjusted_time = (base_time as f64 * cpu_factor) as i64;

        Duration::seconds(adjusted_time.max(10)) // Minimum 10 seconds
    }

    /// Start process execution in the background
    async fn start_process_execution(&self, process_id: ProcessId) {
        let processes = self.processes.clone();
        let contexts = self.execution_contexts.clone();
        let resource_allocations = self.resource_allocations.clone();

        tokio::spawn(async move {
            let execution_result = {
                let processes_guard = processes.read().await;
                let contexts_guard = contexts.read().await;
                let resources_guard = resource_allocations.read().await;

                if let (Some(process), Some(context), Some(resources)) = (
                    processes_guard.get(&process_id),
                    contexts_guard.get(&process_id),
                    resources_guard.get(&process_id)
                ) {
                    Some((process.clone(), context.clone(), resources.clone()))
                } else {
                    None
                }
            };

            if let Some((process, mut context, resources)) = execution_result {
                // Simulate process execution
                context.execution_state = ProcessExecutionState::Running;
                
                // Update context
                {
                    let mut contexts_guard = contexts.write().await;
                    contexts_guard.insert(process_id, context.clone());
                }

                // Calculate completion time
                let base_time = match process.process_type {
                    ProcessType::FileDownload => 30,
                    ProcessType::FileUpload => 45,
                    ProcessType::BruteforceLogin => 120,
                    ProcessType::Hacking => 300,
                    ProcessType::VirusScan => 90,
                    ProcessType::LogCleaning => 60,
                    _ => 180,
                };

                let cpu_factor = if resources.cpu > 0.0 { 1.0 / resources.cpu } else { 2.0 };
                let execution_time = ((base_time as f64 * cpu_factor) as u64).max(5);

                // Simulate execution time
                tokio::time::sleep(tokio::time::Duration::from_secs(execution_time)).await;

                // Mark process as completed
                {
                    let mut processes_guard = processes.write().await;
                    let mut contexts_guard = contexts.write().await;

                    if let Some(process) = processes_guard.get_mut(&process_id) {
                        process.state = ProcessState::Completed;
                        process.progress = ProcessProgress::new(1.0); // 100% complete
                        process.completion_date = Some(Utc::now());
                        process.time_left = Some(0);
                    }

                    if let Some(context) = contexts_guard.get_mut(&process_id) {
                        context.execution_state = ProcessExecutionState::Completing;
                        context.last_checkpoint = Utc::now();
                    }
                }

                info!("Process {} completed execution", process_id);
            }
        });
    }

    /// Add process to server mapping
    async fn add_process_to_server(&self, server_id: ServerId, process_id: ProcessId) {
        let mut server_processes = self.server_processes.write().await;
        server_processes.entry(server_id).or_insert_with(Vec::new).push(process_id);
    }

    /// Remove process from server mapping
    async fn remove_process_from_server(&self, server_id: &ServerId, process_id: &ProcessId) {
        let mut server_processes = self.server_processes.write().await;
        if let Some(processes) = server_processes.get_mut(server_id) {
            processes.retain(|id| id != process_id);
        }
    }
}

impl Actor for ProcessActor {
    fn started(&mut self, ctx: &mut ActorContext) {
        info!("ProcessActor started with process_id: {}", ctx.process_id);
    }

    fn stopping(&mut self, ctx: &mut ActorContext) {
        info!("ProcessActor stopping with process_id: {}", ctx.process_id);
    }

    fn error(&mut self, err: HelixError, ctx: &mut ActorContext) {
        error!("ProcessActor error on process_id {}: {}", ctx.process_id, err);
    }
}

#[async_trait]
impl Handler<CreateProcess> for ProcessActor {
    async fn handle(&mut self, msg: CreateProcess, _ctx: &mut ActorContext) -> Result<Process, HelixError> {
        info!("Creating process type: {:?} from {} to {}", msg.process_type, msg.gateway_id, msg.target_id);
        
        let mut processes = self.processes.write().await;
        let mut server_processes = self.server_processes.write().await;
        
        let process_id = self.generate_process_id();
        let now = Utc::now();
        
        let process = Process {
            process_id,
            gateway_id: msg.gateway_id,
            source_entity_id: msg.source_entity_id,
            target_id: msg.target_id,
            process_type: msg.process_type,
            data: msg.data,
            network_id: None,
            src_connection_id: None,
            src_file_id: None,
            src_atm_id: None,
            src_acc_number: None,
            tgt_file_id: None,
            tgt_connection_id: None,
            tgt_atm_id: None,
            tgt_acc_number: None,
            tgt_process_id: None,
            state: ProcessState::Waiting,
            priority: ProcessPriority::Normal,
            progress: ProcessProgress::new(0.0),
            l_allocated: None,
            r_allocated: None,
            l_limit: Default::default(),
            r_limit: Default::default(),
            l_reserved: Default::default(),
            r_reserved: Default::default(),
            creation_time: now,
            last_checkpoint_time: now,
            completion_date: None,
            time_left: None,
        };
        
        // Store process
        processes.insert(process_id, process.clone());
        
        // Add to server mapping
        server_processes.entry(msg.gateway_id).or_insert_with(Vec::new).push(process_id);
        if msg.gateway_id != msg.target_id {
            server_processes.entry(msg.target_id).or_insert_with(Vec::new).push(process_id);
        }
        
        info!("Process created successfully: {}", process_id);
        Ok(process)
    }
}

#[async_trait]
impl Handler<GetProcess> for ProcessActor {
    async fn handle(&mut self, msg: GetProcess, _ctx: &mut ActorContext) -> Result<Option<Process>, HelixError> {
        let processes = self.processes.read().await;
        Ok(processes.get(&msg.process_id).cloned())
    }
}

#[async_trait]
impl Handler<UpdateProcess> for ProcessActor {
    async fn handle(&mut self, msg: UpdateProcess, _ctx: &mut ActorContext) -> Result<Process, HelixError> {
        let mut processes = self.processes.write().await;
        
        let process = processes.get_mut(&msg.process_id)
            .ok_or_else(|| HelixError::not_found("Process not found"))?;
        
        // Update fields if provided
        if let Some(state) = msg.state {
            process.state = state;
        }
        
        if let Some(progress) = msg.progress {
            process.progress = progress;
        }
        
        if let Some(data) = msg.data {
            process.data = Some(data);
        }
        
        process.last_checkpoint_time = Utc::now();
        
        debug!("Process updated: {}", msg.process_id);
        Ok(process.clone())
    }
}

#[async_trait]
impl Handler<StartProcess> for ProcessActor {
    async fn handle(&mut self, msg: StartProcess, _ctx: &mut ActorContext) -> Result<(), HelixError> {
        let mut processes = self.processes.write().await;
        let mut contexts = self.execution_contexts.write().await;
        let resource_allocations = self.resource_allocations.read().await;
        
        let process = processes.get_mut(&msg.process_id)
            .ok_or_else(|| HelixError::not_found("Process not found"))?;
        
        // Check if process can be started
        if !matches!(process.state, ProcessState::Waiting | ProcessState::Paused) {
            return Err(HelixError::validation("Process cannot be started in current state"));
        }
        
        // Get allocated resources
        let resources = resource_allocations.get(&msg.process_id)
            .cloned()
            .unwrap_or_default();
        
        // Create execution context
        let context = ProcessExecutionContext {
            process_id: msg.process_id,
            allocated_resources: resources.clone(),
            start_time: Utc::now(),
            last_checkpoint: Utc::now(),
            execution_state: ProcessExecutionState::Initializing,
        };
        
        contexts.insert(msg.process_id, context);
        
        // Update process state
        process.state = ProcessState::Running;
        process.last_checkpoint_time = Utc::now();
        
        // Calculate estimated completion time
        let completion_time = self.calculate_completion_time(&process.process_type, &resources);
        process.time_left = Some(completion_time.num_seconds() as u64);
        
        drop(processes);
        drop(contexts);
        drop(resource_allocations);
        
        // Start background execution
        self.start_process_execution(msg.process_id).await;
        
        info!("Process started: {}", msg.process_id);
        Ok(())
    }
}

#[async_trait]
impl Handler<PauseProcess> for ProcessActor {
    async fn handle(&mut self, msg: PauseProcess, _ctx: &mut ActorContext) -> Result<(), HelixError> {
        let mut processes = self.processes.write().await;
        let mut contexts = self.execution_contexts.write().await;
        
        let process = processes.get_mut(&msg.process_id)
            .ok_or_else(|| HelixError::not_found("Process not found"))?;
        
        if !matches!(process.state, ProcessState::Running) {
            return Err(HelixError::validation("Process is not running"));
        }
        
        process.state = ProcessState::Paused;
        process.last_checkpoint_time = Utc::now();
        
        // Update execution context
        if let Some(context) = contexts.get_mut(&msg.process_id) {
            context.execution_state = ProcessExecutionState::Suspended;
            context.last_checkpoint = Utc::now();
        }
        
        info!("Process paused: {}", msg.process_id);
        Ok(())
    }
}

#[async_trait]
impl Handler<ResumeProcess> for ProcessActor {
    async fn handle(&mut self, msg: ResumeProcess, _ctx: &mut ActorContext) -> Result<(), HelixError> {
        let mut processes = self.processes.write().await;
        let mut contexts = self.execution_contexts.write().await;
        
        let process = processes.get_mut(&msg.process_id)
            .ok_or_else(|| HelixError::not_found("Process not found"))?;
        
        if !matches!(process.state, ProcessState::Paused) {
            return Err(HelixError::validation("Process is not paused"));
        }
        
        process.state = ProcessState::Running;
        process.last_checkpoint_time = Utc::now();
        
        // Update execution context
        if let Some(context) = contexts.get_mut(&msg.process_id) {
            context.execution_state = ProcessExecutionState::Running;
            context.last_checkpoint = Utc::now();
        }
        
        info!("Process resumed: {}", msg.process_id);
        Ok(())
    }
}

#[async_trait]
impl Handler<KillProcess> for ProcessActor {
    async fn handle(&mut self, msg: KillProcess, _ctx: &mut ActorContext) -> Result<(), HelixError> {
        let mut processes = self.processes.write().await;
        let mut contexts = self.execution_contexts.write().await;
        let mut resource_allocations = self.resource_allocations.write().await;
        
        if let Some(process) = processes.get_mut(&msg.process_id) {
            process.state = ProcessState::Killed;
            process.completion_date = Some(Utc::now());
            process.time_left = Some(0);
            
            // Remove execution context and resources
            contexts.remove(&msg.process_id);
            resource_allocations.remove(&msg.process_id);
            
            // Remove from server mappings
            self.remove_process_from_server(&process.gateway_id, &msg.process_id).await;
            if process.gateway_id != process.target_id {
                self.remove_process_from_server(&process.target_id, &msg.process_id).await;
            }
            
            info!("Process killed: {}", msg.process_id);
            Ok(())
        } else {
            Err(HelixError::not_found("Process not found"))
        }
    }
}

#[async_trait]
impl Handler<DeleteProcess> for ProcessActor {
    async fn handle(&mut self, msg: DeleteProcess, _ctx: &mut ActorContext) -> Result<(), HelixError> {
        let mut processes = self.processes.write().await;
        let mut contexts = self.execution_contexts.write().await;
        let mut resource_allocations = self.resource_allocations.write().await;
        let mut process_hierarchy = self.process_hierarchy.write().await;
        
        if let Some(process) = processes.remove(&msg.process_id) {
            // Clean up all related data
            contexts.remove(&msg.process_id);
            resource_allocations.remove(&msg.process_id);
            process_hierarchy.remove(&msg.process_id);
            
            // Remove from server mappings
            self.remove_process_from_server(&process.gateway_id, &msg.process_id).await;
            if process.gateway_id != process.target_id {
                self.remove_process_from_server(&process.target_id, &msg.process_id).await;
            }
            
            info!("Process deleted: {}", msg.process_id);
            Ok(())
        } else {
            Err(HelixError::not_found("Process not found"))
        }
    }
}

#[async_trait]
impl Handler<GetServerProcesses> for ProcessActor {
    async fn handle(&mut self, msg: GetServerProcesses, _ctx: &mut ActorContext) -> Result<Vec<Process>, HelixError> {
        let processes = self.processes.read().await;
        let server_processes = self.server_processes.read().await;
        
        if let Some(process_ids) = server_processes.get(&msg.server_id) {
            Ok(process_ids.iter()
                .filter_map(|id| processes.get(id).cloned())
                .collect())
        } else {
            Ok(Vec::new())
        }
    }
}

#[async_trait]
impl Handler<GetProcessesByType> for ProcessActor {
    async fn handle(&mut self, msg: GetProcessesByType, _ctx: &mut ActorContext) -> Result<Vec<Process>, HelixError> {
        let processes = self.processes.read().await;
        
        Ok(processes.values()
            .filter(|p| p.process_type == msg.process_type)
            .cloned()
            .collect())
    }
}

#[async_trait]
impl Handler<AllocateResources> for ProcessActor {
    async fn handle(&mut self, msg: AllocateResources, _ctx: &mut ActorContext) -> Result<(), HelixError> {
        let mut resource_allocations = self.resource_allocations.write().await;
        let processes = self.processes.read().await;
        
        // Verify process exists
        if !processes.contains_key(&msg.process_id) {
            return Err(HelixError::not_found("Process not found"));
        }
        
        resource_allocations.insert(msg.process_id, msg.resources);
        debug!("Resources allocated to process: {}", msg.process_id);
        Ok(())
    }
}

#[async_trait]
impl Handler<DeallocateResources> for ProcessActor {
    async fn handle(&mut self, msg: DeallocateResources, _ctx: &mut ActorContext) -> Result<ProcessResources, HelixError> {
        let mut resource_allocations = self.resource_allocations.write().await;
        
        if let Some(resources) = resource_allocations.remove(&msg.process_id) {
            debug!("Resources deallocated from process: {}", msg.process_id);
            Ok(resources)
        } else {
            Err(HelixError::not_found("Process resource allocation not found"))
        }
    }
}

/// Process Supervisor - manages process actors and provides supervision
#[derive(Debug)]
pub struct ProcessSupervisor {
    process_actor: Option<he_helix_core::actors::ActorAddress>,
}

impl ProcessSupervisor {
    pub fn new() -> Self {
        Self {
            process_actor: None,
        }
    }
    
    pub async fn start(&mut self) -> Result<he_helix_core::actors::ActorAddress, HelixError> {
        let mut supervisor = he_helix_core::actors::ActorSupervisor::new();
        let process_actor = ProcessActor::new();
        let address = supervisor.spawn(process_actor);
        
        self.process_actor = Some(address.clone());
        info!("ProcessSupervisor started successfully");
        
        Ok(address)
    }
    
    pub fn get_process_actor(&self) -> Option<&he_helix_core::actors::ActorAddress> {
        self.process_actor.as_ref()
    }
}

impl Default for ProcessSupervisor {
    fn default() -> Self {
        Self::new()
    }
}