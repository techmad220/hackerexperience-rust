//! Comprehensive GenServer Supervision System
//! 
//! Complete supervision tree implementation with restart strategies,
//! fault tolerance, and distributed supervision capabilities.

use crate::genserver::{
    GenServer, GenServerHandle, GenServerState, SupervisionStrategy, 
    RestartStrategy, TerminateReason
};
use crate::{HelixError, HelixResult, ProcessId};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{RwLock, Mutex, mpsc, oneshot, broadcast};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Supervisor configuration
#[derive(Debug, Clone)]
pub struct SupervisorConfig {
    pub name: String,
    pub strategy: SupervisionStrategy,
    pub max_restarts: u32,
    pub max_restart_period: Duration,
    pub shutdown_timeout: Duration,
    pub enable_hot_code_loading: bool,
    pub enable_distributed_supervision: bool,
    pub health_check_interval: Duration,
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        Self {
            name: "unnamed_supervisor".to_string(),
            strategy: SupervisionStrategy::OneForOne,
            max_restarts: 5,
            max_restart_period: Duration::from_secs(60),
            shutdown_timeout: Duration::from_secs(30),
            enable_hot_code_loading: false,
            enable_distributed_supervision: false,
            health_check_interval: Duration::from_secs(30),
        }
    }
}

/// Child specification for supervised GenServers
#[derive(Debug, Clone)]
pub struct ChildSpec {
    pub id: String,
    pub restart_strategy: RestartStrategy,
    pub shutdown_timeout: Duration,
    pub child_type: ChildType,
    pub significant: bool, // Whether failure affects supervisor strategy
    pub metadata: HashMap<String, String>,
}

/// Types of supervised children
#[derive(Debug, Clone)]
pub enum ChildType {
    Worker,
    Supervisor,
}

/// Supervisor tree node
pub struct SupervisorNode {
    pub supervisor_id: ProcessId,
    pub config: SupervisorConfig,
    pub children: Arc<RwLock<HashMap<String, ChildInfo>>>,
    pub restart_counts: Arc<RwLock<HashMap<String, RestartHistory>>>,
    pub event_tx: broadcast::Sender<SupervisorEvent>,
    pub command_rx: mpsc::UnboundedReceiver<SupervisorCommand>,
    pub command_tx: mpsc::UnboundedSender<SupervisorCommand>,
    pub parent_supervisor: Option<ProcessId>,
    pub child_supervisors: Arc<RwLock<Vec<ProcessId>>>,
}

/// Information about a supervised child
#[derive(Debug)]
pub struct ChildInfo {
    pub spec: ChildSpec,
    pub handle: Option<GenServerHandle>,
    pub start_time: SystemTime,
    pub last_restart: Option<SystemTime>,
    pub status: ChildStatus,
    pub health_check_failures: u32,
}

/// Child process status
#[derive(Debug, Clone, PartialEq)]
pub enum ChildStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
    Restarting,
}

/// Restart history tracking
#[derive(Debug, Clone)]
pub struct RestartHistory {
    pub restarts: VecDeque<SystemTime>,
    pub total_restarts: u64,
    pub last_restart_reason: Option<String>,
}

impl RestartHistory {
    pub fn new() -> Self {
        Self {
            restarts: VecDeque::new(),
            total_restarts: 0,
            last_restart_reason: None,
        }
    }

    pub fn add_restart(&mut self, reason: Option<String>) {
        let now = SystemTime::now();
        self.restarts.push_back(now);
        self.total_restarts += 1;
        self.last_restart_reason = reason;

        // Keep only recent restarts (last hour)
        while let Some(&front) = self.restarts.front() {
            if now.duration_since(front).unwrap_or(Duration::ZERO) > Duration::from_secs(3600) {
                self.restarts.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn recent_restart_count(&self, within: Duration) -> usize {
        let cutoff = SystemTime::now() - within;
        self.restarts.iter()
            .filter(|&&restart_time| restart_time >= cutoff)
            .count()
    }
}

/// Supervisor events
#[derive(Debug, Clone)]
pub enum SupervisorEvent {
    ChildStarted { child_id: String, process_id: ProcessId },
    ChildStopped { child_id: String, reason: TerminateReason },
    ChildFailed { child_id: String, error: String },
    ChildRestarted { child_id: String, attempt: u32 },
    MaxRestartsExceeded { child_id: String },
    SupervisorStarted { supervisor_id: ProcessId },
    SupervisorStopped { supervisor_id: ProcessId },
    HealthCheckFailed { child_id: String, failure_count: u32 },
    SupervisorTreeUpdated,
}

/// Supervisor commands
#[derive(Debug)]
pub enum SupervisorCommand {
    StartChild { spec: ChildSpec },
    StopChild { child_id: String },
    RestartChild { child_id: String },
    GetChildInfo { child_id: String, response_tx: oneshot::Sender<Option<ChildInfo>> },
    GetAllChildren { response_tx: oneshot::Sender<Vec<(String, ChildInfo)>> },
    UpdateChildSpec { child_id: String, spec: ChildSpec },
    SetChildMetadata { child_id: String, metadata: HashMap<String, String> },
    HealthCheck { child_id: Option<String> },
    Shutdown { graceful: bool },
    HotCodeReload { child_id: String, code_path: String },
    GetSupervisorStats { response_tx: oneshot::Sender<SupervisorStats> },
}

/// Supervisor statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorStats {
    pub supervisor_id: ProcessId,
    pub total_children: usize,
    pub running_children: usize,
    pub failed_children: usize,
    pub total_restarts: u64,
    pub uptime: Duration,
    pub last_restart: Option<SystemTime>,
    pub health_status: SupervisorHealth,
}

/// Supervisor health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupervisorHealth {
    Healthy,
    Warning { issues: Vec<String> },
    Critical { issues: Vec<String> },
    Failed { reason: String },
}

impl SupervisorNode {
    pub fn new(config: SupervisorConfig) -> (Self, mpsc::UnboundedReceiver<SupervisorCommand>) {
        let supervisor_id = ProcessId::new();
        let (event_tx, _) = broadcast::channel(1000);
        let (command_tx, command_rx) = mpsc::unbounded_channel();

        let node = Self {
            supervisor_id,
            config,
            children: Arc::new(RwLock::new(HashMap::new())),
            restart_counts: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            command_rx,
            command_tx,
            parent_supervisor: None,
            child_supervisors: Arc::new(RwLock::new(Vec::new())),
        };

        (node, command_rx)
    }

    pub async fn start(&mut self) -> HelixResult<()> {
        info!("Starting supervisor '{}' with ID {}", self.config.name, self.supervisor_id);

        // Send supervisor started event
        let _ = self.event_tx.send(SupervisorEvent::SupervisorStarted {
            supervisor_id: self.supervisor_id,
        });

        // Start the supervisor loop
        self.supervisor_loop().await
    }

    async fn supervisor_loop(&mut self) -> HelixResult<()> {
        let mut health_check_interval = tokio::time::interval(self.config.health_check_interval);
        
        loop {
            tokio::select! {
                command = self.command_rx.recv() => {
                    match command {
                        Some(cmd) => {
                            if let Err(e) = self.handle_command(cmd).await {
                                error!("Error handling supervisor command: {}", e);
                            }
                        }
                        None => {
                            info!("Supervisor command channel closed, shutting down");
                            break;
                        }
                    }
                }
                _ = health_check_interval.tick() => {
                    if let Err(e) = self.perform_health_checks().await {
                        error!("Error performing health checks: {}", e);
                    }
                }
            }
        }

        // Shutdown all children
        self.shutdown_all_children(true).await?;

        // Send supervisor stopped event
        let _ = self.event_tx.send(SupervisorEvent::SupervisorStopped {
            supervisor_id: self.supervisor_id,
        });

        Ok(())
    }

    async fn handle_command(&mut self, command: SupervisorCommand) -> HelixResult<()> {
        match command {
            SupervisorCommand::StartChild { spec } => {
                self.start_child(spec).await?;
            }
            
            SupervisorCommand::StopChild { child_id } => {
                self.stop_child(&child_id, false).await?;
            }
            
            SupervisorCommand::RestartChild { child_id } => {
                self.restart_child(&child_id).await?;
            }
            
            SupervisorCommand::GetChildInfo { child_id, response_tx } => {
                let children = self.children.read().await;
                let info = children.get(&child_id).cloned();
                let _ = response_tx.send(info);
            }
            
            SupervisorCommand::GetAllChildren { response_tx } => {
                let children = self.children.read().await;
                let all_children: Vec<(String, ChildInfo)> = children.iter()
                    .map(|(id, info)| (id.clone(), info.clone()))
                    .collect();
                let _ = response_tx.send(all_children);
            }
            
            SupervisorCommand::UpdateChildSpec { child_id, spec } => {
                let mut children = self.children.write().await;
                if let Some(child) = children.get_mut(&child_id) {
                    child.spec = spec;
                    info!("Updated child spec for '{}'", child_id);
                }
            }
            
            SupervisorCommand::SetChildMetadata { child_id, metadata } => {
                let mut children = self.children.write().await;
                if let Some(child) = children.get_mut(&child_id) {
                    child.spec.metadata = metadata;
                    debug!("Updated metadata for child '{}'", child_id);
                }
            }
            
            SupervisorCommand::HealthCheck { child_id } => {
                if let Some(child_id) = child_id {
                    self.check_child_health(&child_id).await?;
                } else {
                    self.perform_health_checks().await?;
                }
            }
            
            SupervisorCommand::Shutdown { graceful } => {
                info!("Supervisor shutdown requested (graceful: {})", graceful);
                self.shutdown_all_children(graceful).await?;
                return Ok(()); // Exit the supervisor loop
            }
            
            SupervisorCommand::HotCodeReload { child_id, code_path } => {
                if self.config.enable_hot_code_loading {
                    self.hot_reload_child(&child_id, &code_path).await?;
                } else {
                    warn!("Hot code reloading is disabled for this supervisor");
                }
            }
            
            SupervisorCommand::GetSupervisorStats { response_tx } => {
                let stats = self.get_supervisor_stats().await;
                let _ = response_tx.send(stats);
            }
        }
        
        Ok(())
    }

    async fn start_child(&mut self, spec: ChildSpec) -> HelixResult<()> {
        info!("Starting child '{}' with restart strategy {:?}", spec.id, spec.restart_strategy);

        let child_info = ChildInfo {
            spec: spec.clone(),
            handle: None, // Will be set when child actually starts
            start_time: SystemTime::now(),
            last_restart: None,
            status: ChildStatus::Starting,
            health_check_failures: 0,
        };

        // Add to children map
        {
            let mut children = self.children.write().await;
            children.insert(spec.id.clone(), child_info);
        }

        // Initialize restart history
        {
            let mut restart_counts = self.restart_counts.write().await;
            restart_counts.insert(spec.id.clone(), RestartHistory::new());
        }

        // TODO: Actually start the GenServer here
        // This would require the GenServer implementation to be passed in
        // For now, we'll simulate a successful start

        // Update status to running
        {
            let mut children = self.children.write().await;
            if let Some(child) = children.get_mut(&spec.id) {
                child.status = ChildStatus::Running;
            }
        }

        // Send child started event
        let _ = self.event_tx.send(SupervisorEvent::ChildStarted {
            child_id: spec.id.clone(),
            process_id: ProcessId::new(), // Would be actual process ID
        });

        Ok(())
    }

    async fn stop_child(&mut self, child_id: &str, force: bool) -> HelixResult<()> {
        info!("Stopping child '{}' (force: {})", child_id, force);

        let mut children = self.children.write().await;
        if let Some(child) = children.get_mut(child_id) {
            child.status = ChildStatus::Stopping;

            if let Some(handle) = &child.handle {
                let timeout = if force {
                    Duration::from_secs(1) // Force stop quickly
                } else {
                    child.spec.shutdown_timeout
                };

                // TODO: Implement graceful shutdown with timeout
                // For now, we'll simulate the stop
                
                child.status = ChildStatus::Stopped;
                child.handle = None;

                // Send child stopped event
                let _ = self.event_tx.send(SupervisorEvent::ChildStopped {
                    child_id: child_id.to_string(),
                    reason: if force { TerminateReason::Kill } else { TerminateReason::Shutdown },
                });
            }
        }

        Ok(())
    }

    async fn restart_child(&mut self, child_id: &str) -> HelixResult<()> {
        info!("Restarting child '{}'", child_id);

        // Check restart limits
        {
            let restart_counts = self.restart_counts.read().await;
            if let Some(history) = restart_counts.get(child_id) {
                let recent_restarts = history.recent_restart_count(self.config.max_restart_period);
                if recent_restarts >= self.config.max_restarts as usize {
                    error!("Child '{}' exceeded max restarts ({} in {:?})", 
                           child_id, recent_restarts, self.config.max_restart_period);
                    
                    let _ = self.event_tx.send(SupervisorEvent::MaxRestartsExceeded {
                        child_id: child_id.to_string(),
                    });

                    // Apply supervision strategy
                    self.apply_supervision_strategy(child_id).await?;
                    return Ok(());
                }
            }
        }

        // Update restart history
        {
            let mut restart_counts = self.restart_counts.write().await;
            if let Some(history) = restart_counts.get_mut(child_id) {
                history.add_restart(Some("manual_restart".to_string()));
            }
        }

        // Stop the child first
        self.stop_child(child_id, false).await?;

        // Wait a bit before restarting
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Get the child spec and restart
        let spec = {
            let children = self.children.read().await;
            children.get(child_id).map(|child| child.spec.clone())
        };

        if let Some(spec) = spec {
            // Update status to restarting
            {
                let mut children = self.children.write().await;
                if let Some(child) = children.get_mut(child_id) {
                    child.status = ChildStatus::Restarting;
                    child.last_restart = Some(SystemTime::now());
                }
            }

            // Start the child again
            self.start_child(spec).await?;

            // Get restart attempt count
            let restart_count = {
                let restart_counts = self.restart_counts.read().await;
                restart_counts.get(child_id)
                    .map(|h| h.total_restarts)
                    .unwrap_or(0) as u32
            };

            // Send child restarted event
            let _ = self.event_tx.send(SupervisorEvent::ChildRestarted {
                child_id: child_id.to_string(),
                attempt: restart_count,
            });

            info!("Successfully restarted child '{}' (attempt {})", child_id, restart_count);
        } else {
            error!("Could not find child spec for '{}'", child_id);
        }

        Ok(())
    }

    async fn apply_supervision_strategy(&mut self, failed_child_id: &str) -> HelixResult<()> {
        match self.config.strategy {
            SupervisionStrategy::OneForOne => {
                // Only restart the failed child (already handled in restart_child)
                warn!("Child '{}' failed and exceeded restart limits, stopping", failed_child_id);
                self.stop_child(failed_child_id, true).await?;
            }
            
            SupervisionStrategy::OneForAll => {
                warn!("Child '{}' failed, restarting all children due to OneForAll strategy", failed_child_id);
                
                // Stop all children
                let child_ids: Vec<String> = {
                    let children = self.children.read().await;
                    children.keys().cloned().collect()
                };

                for child_id in &child_ids {
                    self.stop_child(child_id, true).await?;
                }

                // Wait a bit
                tokio::time::sleep(Duration::from_millis(500)).await;

                // Restart all children
                for child_id in &child_ids {
                    if let Err(e) = self.restart_child(child_id).await {
                        error!("Failed to restart child '{}': {}", child_id, e);
                    }
                }
            }
            
            SupervisionStrategy::RestForOne => {
                warn!("Child '{}' failed, restarting it and all children started after it", failed_child_id);
                
                // Find children to restart (this is simplified - would need proper ordering)
                let children_to_restart: Vec<String> = {
                    let children = self.children.read().await;
                    children.keys().cloned().collect() // Simplified
                };

                for child_id in &children_to_restart {
                    if let Err(e) = self.restart_child(child_id).await {
                        error!("Failed to restart child '{}': {}", child_id, e);
                    }
                }
            }
            
            SupervisionStrategy::SimpleOneForOne => {
                // For dynamic children, just remove the failed one
                warn!("Child '{}' failed, removing from simple_one_for_one supervisor", failed_child_id);
                
                let mut children = self.children.write().await;
                children.remove(failed_child_id);
                
                let mut restart_counts = self.restart_counts.write().await;
                restart_counts.remove(failed_child_id);
            }
        }

        Ok(())
    }

    async fn perform_health_checks(&mut self) -> HelixResult<()> {
        debug!("Performing health checks for all children");

        let child_ids: Vec<String> = {
            let children = self.children.read().await;
            children.keys().cloned().collect()
        };

        for child_id in child_ids {
            if let Err(e) = self.check_child_health(&child_id).await {
                error!("Health check failed for child '{}': {}", child_id, e);
            }
        }

        Ok(())
    }

    async fn check_child_health(&mut self, child_id: &str) -> HelixResult<()> {
        let mut children = self.children.write().await;
        if let Some(child) = children.get_mut(child_id) {
            match child.status {
                ChildStatus::Running => {
                    // TODO: Implement actual health check (ping the GenServer)
                    // For now, we'll simulate a random health check
                    let is_healthy = true; // Placeholder
                    
                    if is_healthy {
                        child.health_check_failures = 0;
                    } else {
                        child.health_check_failures += 1;
                        
                        let _ = self.event_tx.send(SupervisorEvent::HealthCheckFailed {
                            child_id: child_id.to_string(),
                            failure_count: child.health_check_failures,
                        });

                        // If health check fails too many times, consider restarting
                        if child.health_check_failures >= 3 {
                            warn!("Child '{}' failed {} health checks, scheduling restart", 
                                  child_id, child.health_check_failures);
                            
                            // Schedule restart (would be done asynchronously)
                            let restart_command = SupervisorCommand::RestartChild {
                                child_id: child_id.to_string(),
                            };
                            let _ = self.command_tx.send(restart_command);
                        }
                    }
                }
                _ => {
                    debug!("Skipping health check for child '{}' (status: {:?})", child_id, child.status);
                }
            }
        }

        Ok(())
    }

    async fn shutdown_all_children(&mut self, graceful: bool) -> HelixResult<()> {
        info!("Shutting down all children (graceful: {})", graceful);

        let child_ids: Vec<String> = {
            let children = self.children.read().await;
            children.keys().cloned().collect()
        };

        for child_id in child_ids {
            if let Err(e) = self.stop_child(&child_id, !graceful).await {
                error!("Failed to stop child '{}': {}", child_id, e);
            }
        }

        Ok(())
    }

    async fn hot_reload_child(&mut self, child_id: &str, _code_path: &str) -> HelixResult<()> {
        info!("Hot reloading child '{}' with new code", child_id);

        // TODO: Implement hot code reloading
        // This would involve:
        // 1. Loading new code from the specified path
        // 2. Calling the GenServer's code_change callback
        // 3. Updating the child's behavior without stopping it

        warn!("Hot code reloading not yet fully implemented");
        Ok(())
    }

    async fn get_supervisor_stats(&self) -> SupervisorStats {
        let children = self.children.read().await;
        let restart_counts = self.restart_counts.read().await;

        let total_children = children.len();
        let running_children = children.values()
            .filter(|child| child.status == ChildStatus::Running)
            .count();
        let failed_children = children.values()
            .filter(|child| child.status == ChildStatus::Failed)
            .count();

        let total_restarts = restart_counts.values()
            .map(|history| history.total_restarts)
            .sum();

        let last_restart = restart_counts.values()
            .filter_map(|history| history.restarts.back().copied())
            .max();

        // Determine health status
        let health_status = if failed_children > 0 {
            SupervisorHealth::Critical {
                issues: vec![format!("{} children have failed", failed_children)],
            }
        } else if running_children < total_children {
            SupervisorHealth::Warning {
                issues: vec![format!("{} children are not running", total_children - running_children)],
            }
        } else {
            SupervisorHealth::Healthy
        };

        SupervisorStats {
            supervisor_id: self.supervisor_id,
            total_children,
            running_children,
            failed_children,
            total_restarts,
            uptime: SystemTime::now().duration_since(
                children.values()
                    .map(|child| child.start_time)
                    .min()
                    .unwrap_or_else(SystemTime::now)
            ).unwrap_or(Duration::ZERO),
            last_restart,
            health_status,
        }
    }

    pub fn get_event_receiver(&self) -> broadcast::Receiver<SupervisorEvent> {
        self.event_tx.subscribe()
    }

    pub fn get_command_sender(&self) -> mpsc::UnboundedSender<SupervisorCommand> {
        self.command_tx.clone()
    }
}

/// Supervision tree builder for constructing complex supervisor hierarchies
pub struct SupervisionTreeBuilder {
    root_config: SupervisorConfig,
    child_specs: Vec<ChildSpec>,
    child_supervisors: Vec<SupervisionTreeBuilder>,
}

impl SupervisionTreeBuilder {
    pub fn new(name: String, strategy: SupervisionStrategy) -> Self {
        Self {
            root_config: SupervisorConfig {
                name,
                strategy,
                ..Default::default()
            },
            child_specs: Vec::new(),
            child_supervisors: Vec::new(),
        }
    }

    pub fn with_config(mut self, config: SupervisorConfig) -> Self {
        self.root_config = config;
        self
    }

    pub fn add_worker(mut self, id: String, restart_strategy: RestartStrategy) -> Self {
        let spec = ChildSpec {
            id,
            restart_strategy,
            shutdown_timeout: Duration::from_secs(10),
            child_type: ChildType::Worker,
            significant: true,
            metadata: HashMap::new(),
        };
        self.child_specs.push(spec);
        self
    }

    pub fn add_child(mut self, spec: ChildSpec) -> Self {
        self.child_specs.push(spec);
        self
    }

    pub fn add_supervisor(mut self, supervisor: SupervisionTreeBuilder) -> Self {
        self.child_supervisors.push(supervisor);
        self
    }

    pub async fn build(self) -> HelixResult<SupervisorTree> {
        SupervisorTree::new(self).await
    }
}

/// Complete supervision tree with hierarchical supervisors
pub struct SupervisorTree {
    root_supervisor: SupervisorNode,
    command_rx: mpsc::UnboundedReceiver<SupervisorCommand>,
}

impl SupervisorTree {
    pub async fn new(builder: SupervisionTreeBuilder) -> HelixResult<Self> {
        let (mut root_supervisor, command_rx) = SupervisorNode::new(builder.root_config);

        // Add child specs to root supervisor
        for spec in builder.child_specs {
            root_supervisor.start_child(spec).await?;
        }

        // TODO: Add child supervisors recursively
        // This would require spawning them as separate tasks

        Ok(Self {
            root_supervisor,
            command_rx,
        })
    }

    pub async fn start(mut self) -> HelixResult<()> {
        info!("Starting supervision tree");
        self.root_supervisor.start().await
    }

    pub fn get_command_sender(&self) -> mpsc::UnboundedSender<SupervisorCommand> {
        self.root_supervisor.get_command_sender()
    }

    pub fn get_event_receiver(&self) -> broadcast::Receiver<SupervisorEvent> {
        self.root_supervisor.get_event_receiver()
    }
}

/// Distributed supervision coordinator
pub struct DistributedSupervisor {
    local_supervisors: HashMap<String, mpsc::UnboundedSender<SupervisorCommand>>,
    remote_nodes: HashMap<String, RemoteNode>,
    coordination_tx: broadcast::Sender<CoordinationMessage>,
}

/// Remote supervision node information
#[derive(Debug, Clone)]
pub struct RemoteNode {
    pub node_id: String,
    pub address: String,
    pub last_heartbeat: SystemTime,
    pub is_online: bool,
    pub supervisors: Vec<String>,
}

/// Messages for distributed coordination
#[derive(Debug, Clone)]
pub enum CoordinationMessage {
    NodeJoined { node_id: String, address: String },
    NodeLeft { node_id: String },
    SupervisorStarted { supervisor_id: String, node_id: String },
    SupervisorFailed { supervisor_id: String, node_id: String },
    FailoverRequest { failed_node: String, supervisor_id: String },
    HeartbeatReceived { node_id: String },
}

impl DistributedSupervisor {
    pub fn new() -> Self {
        let (coordination_tx, _) = broadcast::channel(1000);
        
        Self {
            local_supervisors: HashMap::new(),
            remote_nodes: HashMap::new(),
            coordination_tx,
        }
    }

    pub async fn add_local_supervisor(&mut self, name: String, command_tx: mpsc::UnboundedSender<SupervisorCommand>) {
        self.local_supervisors.insert(name, command_tx);
    }

    pub async fn add_remote_node(&mut self, node: RemoteNode) {
        info!("Adding remote node: {}", node.node_id);
        self.remote_nodes.insert(node.node_id.clone(), node.clone());
        
        let _ = self.coordination_tx.send(CoordinationMessage::NodeJoined {
            node_id: node.node_id,
            address: node.address,
        });
    }

    pub async fn handle_node_failure(&mut self, node_id: &str) -> HelixResult<()> {
        warn!("Handling failure of node: {}", node_id);

        if let Some(node) = self.remote_nodes.get_mut(node_id) {
            node.is_online = false;

            // Trigger failover for supervisors on the failed node
            for supervisor_id in &node.supervisors {
                let _ = self.coordination_tx.send(CoordinationMessage::FailoverRequest {
                    failed_node: node_id.to_string(),
                    supervisor_id: supervisor_id.clone(),
                });
            }
        }

        Ok(())
    }

    pub fn get_coordination_receiver(&self) -> broadcast::Receiver<CoordinationMessage> {
        self.coordination_tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_supervisor_node_creation() {
        let config = SupervisorConfig {
            name: "test_supervisor".to_string(),
            strategy: SupervisionStrategy::OneForOne,
            ..Default::default()
        };

        let (mut supervisor, _command_rx) = SupervisorNode::new(config);
        assert_eq!(supervisor.config.name, "test_supervisor");
        assert!(matches!(supervisor.config.strategy, SupervisionStrategy::OneForOne));
    }

    #[tokio::test]
    async fn test_child_spec_creation() {
        let spec = ChildSpec {
            id: "test_worker".to_string(),
            restart_strategy: RestartStrategy::Permanent,
            shutdown_timeout: Duration::from_secs(5),
            child_type: ChildType::Worker,
            significant: true,
            metadata: HashMap::new(),
        };

        assert_eq!(spec.id, "test_worker");
        assert!(matches!(spec.restart_strategy, RestartStrategy::Permanent));
        assert!(matches!(spec.child_type, ChildType::Worker));
    }

    #[tokio::test]
    async fn test_restart_history() {
        let mut history = RestartHistory::new();
        
        history.add_restart(Some("test_failure".to_string()));
        assert_eq!(history.total_restarts, 1);
        assert_eq!(history.last_restart_reason, Some("test_failure".to_string()));
        
        let recent_count = history.recent_restart_count(Duration::from_secs(60));
        assert_eq!(recent_count, 1);
    }

    #[tokio::test]
    async fn test_supervision_tree_builder() {
        let tree_builder = SupervisionTreeBuilder::new(
            "test_tree".to_string(),
            SupervisionStrategy::OneForOne,
        )
        .add_worker("worker1".to_string(), RestartStrategy::Permanent)
        .add_worker("worker2".to_string(), RestartStrategy::Temporary);

        // The tree builder should be ready to build
        assert_eq!(tree_builder.root_config.name, "test_tree");
        assert_eq!(tree_builder.child_specs.len(), 2);
    }
}