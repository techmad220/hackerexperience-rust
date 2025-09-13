//! Supervisor system for managing actor lifecycles
//!
//! This module provides OTP-style supervision for actors, including restart
//! strategies and fault tolerance.

use crate::{HelixError, HelixResult, ProcessId};
use crate::actors::{Actor, ActorAddress, ActorSupervisor};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

/// Restart strategy for supervised actors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestartStrategy {
    /// Never restart failed actors
    Never,
    /// Always restart failed actors
    Always,
    /// Restart failed actors up to a maximum number of times
    MaxRetries(u32),
    /// Restart failed actors with exponential backoff
    ExponentialBackoff {
        max_retries: u32,
        base_delay: Duration,
        max_delay: Duration,
    },
}

/// Supervisor strategy for handling multiple child failures
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupervisorStrategy {
    /// If one child fails, restart only that child
    OneForOne,
    /// If one child fails, restart all children
    OneForAll,
    /// If one child fails, restart all children started after it
    RestForOne,
}

/// Configuration for a supervisor
#[derive(Debug, Clone)]
pub struct SupervisorConfig {
    /// Strategy for handling child failures
    pub strategy: SupervisorStrategy,
    /// Default restart strategy for children
    pub restart_strategy: RestartStrategy,
    /// Maximum number of failures within the time window
    pub max_failures: u32,
    /// Time window for counting failures
    pub failure_window: Duration,
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        Self {
            strategy: SupervisorStrategy::OneForOne,
            restart_strategy: RestartStrategy::MaxRetries(3),
            max_failures: 5,
            failure_window: Duration::from_secs(60),
        }
    }
}

/// Information about a supervised child
#[derive(Debug)]
struct ChildInfo {
    process_id: ProcessId,
    address: ActorAddress,
    restart_strategy: RestartStrategy,
    restart_count: u32,
    last_restart: Option<Instant>,
    failure_history: Vec<Instant>,
}

impl ChildInfo {
    fn new(process_id: ProcessId, address: ActorAddress, restart_strategy: RestartStrategy) -> Self {
        Self {
            process_id,
            address,
            restart_strategy,
            restart_count: 0,
            last_restart: None,
            failure_history: Vec::new(),
        }
    }

    fn should_restart(&self, config: &SupervisorConfig) -> bool {
        match &self.restart_strategy {
            RestartStrategy::Never => false,
            RestartStrategy::Always => true,
            RestartStrategy::MaxRetries(max) => self.restart_count < *max,
            RestartStrategy::ExponentialBackoff { max_retries, .. } => {
                self.restart_count < *max_retries
            }
        }
    }

    fn calculate_restart_delay(&self) -> Duration {
        match &self.restart_strategy {
            RestartStrategy::ExponentialBackoff {
                base_delay,
                max_delay,
                ..
            } => {
                let delay = *base_delay * 2_u32.pow(self.restart_count);
                delay.min(*max_delay)
            }
            _ => Duration::ZERO,
        }
    }

    fn record_failure(&mut self) {
        let now = Instant::now();
        self.failure_history.push(now);
        
        // Keep only recent failures within the window
        // This would need the window duration, but we'll keep it simple for now
        if self.failure_history.len() > 10 {
            self.failure_history.remove(0);
        }
    }
}

/// A supervisor manages the lifecycle of child actors
pub struct Supervisor {
    config: SupervisorConfig,
    children: Arc<RwLock<HashMap<ProcessId, ChildInfo>>>,
    supervisor: Arc<RwLock<ActorSupervisor>>,
}

impl Supervisor {
    pub fn new(config: SupervisorConfig) -> Self {
        Self {
            config,
            children: Arc::new(RwLock::new(HashMap::new())),
            supervisor: Arc::new(RwLock::new(ActorSupervisor::new())),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(SupervisorConfig::default())
    }

    /// Spawn a child actor under supervision
    pub async fn spawn_child<A: Actor + 'static>(
        &self,
        actor: A,
        restart_strategy: Option<RestartStrategy>,
    ) -> HelixResult<(ProcessId, ActorAddress)> {
        let restart_strategy = restart_strategy.unwrap_or_else(|| self.config.restart_strategy.clone());
        
        let address = {
            let mut supervisor = self.supervisor.write().await;
            supervisor.spawn(actor)
        };

        let process_id = ProcessId::new();
        let child_info = ChildInfo::new(process_id, address.clone(), restart_strategy);

        {
            let mut children = self.children.write().await;
            children.insert(process_id, child_info);
        }

        Ok((process_id, address))
    }

    /// Remove a child from supervision
    pub async fn remove_child(&self, process_id: ProcessId) -> HelixResult<()> {
        let mut children = self.children.write().await;
        children
            .remove(&process_id)
            .ok_or_else(|| HelixError::not_found("Child not found"))?;
        Ok(())
    }

    /// Handle child failure and potentially restart
    pub async fn handle_child_failure(&self, process_id: ProcessId, error: HelixError) -> HelixResult<bool> {
        let mut children = self.children.write().await;
        
        let child = children
            .get_mut(&process_id)
            .ok_or_else(|| HelixError::not_found("Child not found"))?;

        child.record_failure();

        if !child.should_restart(&self.config) {
            tracing::warn!(
                "Child {} failed and will not be restarted: {}",
                process_id.0,
                error
            );
            return Ok(false);
        }

        let delay = child.calculate_restart_delay();
        child.restart_count += 1;
        child.last_restart = Some(Instant::now());

        tracing::info!(
            "Restarting child {} (attempt {}) after {:?}",
            process_id.0,
            child.restart_count,
            delay
        );

        if delay > Duration::ZERO {
            tokio::time::sleep(delay).await;
        }

        // TODO: Actually restart the actor
        // This would require storing the actor factory function
        Ok(true)
    }

    /// Get information about all supervised children
    pub async fn get_children(&self) -> Vec<ProcessId> {
        let children = self.children.read().await;
        children.keys().copied().collect()
    }

    /// Get child count
    pub async fn child_count(&self) -> usize {
        let children = self.children.read().await;
        children.len()
    }

    /// Stop all supervised children
    pub async fn stop_all_children(&self) -> HelixResult<()> {
        let mut supervisor = self.supervisor.write().await;
        supervisor.stop_all().await?;
        
        let mut children = self.children.write().await;
        children.clear();
        
        Ok(())
    }

    /// Check if a child is under supervision
    pub async fn is_supervised(&self, process_id: ProcessId) -> bool {
        let children = self.children.read().await;
        children.contains_key(&process_id)
    }

    /// Get supervisor configuration
    pub fn config(&self) -> &SupervisorConfig {
        &self.config
    }
}

/// Builder for supervisor configuration
pub struct SupervisorConfigBuilder {
    strategy: SupervisorStrategy,
    restart_strategy: RestartStrategy,
    max_failures: u32,
    failure_window: Duration,
}

impl SupervisorConfigBuilder {
    pub fn new() -> Self {
        Self {
            strategy: SupervisorStrategy::OneForOne,
            restart_strategy: RestartStrategy::MaxRetries(3),
            max_failures: 5,
            failure_window: Duration::from_secs(60),
        }
    }

    pub fn strategy(mut self, strategy: SupervisorStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn restart_strategy(mut self, restart_strategy: RestartStrategy) -> Self {
        self.restart_strategy = restart_strategy;
        self
    }

    pub fn max_failures(mut self, max_failures: u32) -> Self {
        self.max_failures = max_failures;
        self
    }

    pub fn failure_window(mut self, failure_window: Duration) -> Self {
        self.failure_window = failure_window;
        self
    }

    pub fn build(self) -> SupervisorConfig {
        SupervisorConfig {
            strategy: self.strategy,
            restart_strategy: self.restart_strategy,
            max_failures: self.max_failures,
            failure_window: self.failure_window,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actors::{Actor, ActorContext};

    struct TestActor;

    impl Actor for TestActor {}

    #[tokio::test]
    async fn test_supervisor() {
        let config = SupervisorConfigBuilder::new()
            .strategy(SupervisorStrategy::OneForOne)
            .restart_strategy(RestartStrategy::MaxRetries(2))
            .build();

        let supervisor = Supervisor::new(config);

        let (process_id, _address) = supervisor
            .spawn_child(TestActor, None)
            .await
            .unwrap();

        assert!(supervisor.is_supervised(process_id).await);
        assert_eq!(supervisor.child_count().await, 1);

        supervisor.remove_child(process_id).await.unwrap();
        assert!(!supervisor.is_supervised(process_id).await);
        assert_eq!(supervisor.child_count().await, 0);
    }
}