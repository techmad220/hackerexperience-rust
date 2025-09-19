use anyhow::{anyhow, Result};
//! Improved game server with safe resource management and idempotent operations

use actix_web::{web, App, HttpResponse, HttpServer, Result, middleware, HttpRequest};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uuid::Uuid;
use std::collections::HashMap;
use tracing::{info, warn, error};

mod safe_resources;
use safe_resources::{Units, ResourceCaps, ProcessState, allocate, deallocate};

#[derive(Debug, Clone)]
pub struct Process {
    pub id: Uuid,
    pub process_type: String,
    pub state: ProcessState,
    pub priority: String,
    pub progress: f32,
    pub cpu_usage: Units,
    pub ram_usage: Units,
    pub time_total: Duration,
    pub time_elapsed: Duration,
    pub target: Option<String>,
}

impl Process {
    fn new(process_type: String, priority: String, target: Option<String>) -> Self {
        let (cpu, ram, duration) = match process_type.as_str() {
            "Crack" => (Units(350), Units(128), Duration::from_secs(10)),
            "Download" => (Units(100), Units(64), Duration::from_secs(5)),
            "Scan" => (Units(150), Units(32), Duration::from_secs(3)),
            "Install" => (Units(200), Units(256), Duration::from_secs(7)),
            "DDoS" => (Units(400), Units(512), Duration::from_secs(15)),
            "Mine" => (Units(800), Units(1024), Duration::from_secs(30)),
            _ => (Units(100), Units(64), Duration::from_secs(5)),
        };

        Self {
            id: Uuid::new_v4(),
            process_type,
            state: ProcessState::Queued,
            priority,
            progress: 0.0,
            cpu_usage: cpu,
            ram_usage: ram,
            time_total: duration,
            time_elapsed: Duration::ZERO,
            target,
        }
    }

    fn update(&mut self, delta: Duration) -> bool {
        if self.state.is_terminal() {
            return false; // No updates for terminal states
        }

        if self.state == ProcessState::Running {
            self.time_elapsed = self.time_elapsed.saturating_add(delta);
            self.progress = (self.time_elapsed.as_secs_f32() / self.time_total.as_secs_f32() * 100.0).min(100.0);

            if self.progress >= 100.0 {
                self.state = ProcessState::Completed;
                return true; // Resources can be freed
            }
        }

        false
    }
}

#[derive(Debug, Clone)]
pub struct GameEngine {
    processes: HashMap<Uuid, Process>,
    hardware: ResourceCaps,
    cpu_available: Units,
    ram_available: Units,
    last_update: Instant,
}

impl GameEngine {
    pub fn new() -> Self {
        let hardware = ResourceCaps {
            cpu: Units(2000),
            ram: Units(4096),
        };

        Self {
            processes: HashMap::new(),
            hardware,
            cpu_available: hardware.cpu,
            ram_available: hardware.ram,
            last_update: Instant::now(),
        }
    }

    pub fn start_process(&mut self, process_type: String, priority: String, target: Option<String>)
        -> Result<Uuid, String>
    {
        let mut process = Process::new(process_type.clone(), priority, target);

        // Try to allocate resources safely
        let current_used = (
            Units(self.hardware.cpu.0 - self.cpu_available.0),
            Units(self.hardware.ram.0 - self.ram_available.0),
        );

        match allocate(process.cpu_usage, process.ram_usage, self.hardware, current_used) {
            Ok((allocated_cpu, allocated_ram)) => {
                // Update available resources with actual allocated amounts
                self.cpu_available = self.cpu_available.saturating_sub(allocated_cpu);
                self.ram_available = self.ram_available.saturating_sub(allocated_ram);

                // Store actual allocated amounts (might be less than requested)
                process.cpu_usage = allocated_cpu;
                process.ram_usage = allocated_ram;
                process.state = ProcessState::Running;

                let id = process.id;
                self.processes.insert(id, process);

                info!("Started process {} with cpu={} ram={}", id, allocated_cpu.0, allocated_ram.0);
                Ok(id)
            }
            Err(e) => {
                warn!("Failed to allocate resources for {}: {}", process_type, e);
                Err(format!("Resource allocation failed: {}", e))
            }
        }
    }

    pub fn cancel_process(&mut self, id: Uuid) -> Result<(), String> {
        match self.processes.get_mut(&id) {
            Some(process) => {
                // Idempotent: if already terminal, succeed silently
                if process.state.is_terminal() {
                    info!("Process {} already in terminal state {:?}", id, process.state);
                    return Ok(());
                }

                // Validate state transition
                if !process.state.can_transition_to(ProcessState::Cancelling) {
                    warn!("Invalid transition from {:?} to Cancelling for process {}", process.state, id);
                    return Err("Invalid state transition".to_string());
                }

                // Mark as cancelling
                process.state = ProcessState::Cancelling;
                info!("Process {} marked for cancellation", id);
                Ok(())
            }
            None => {
                // Idempotent: if process doesn't exist, succeed silently (might have been cleaned up)
                info!("Process {} not found (may have completed)", id);
                Ok(())
            }
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let delta = now - self.last_update;
        self.last_update = now;

        let mut to_remove = Vec::new();

        // Update all processes
        for (id, process) in &mut self.processes {
            // Handle cancelling state
            if process.state == ProcessState::Cancelling {
                process.state = ProcessState::Cancelled;
                to_remove.push(*id);
                continue;
            }

            // Update running processes
            if process.update(delta) {
                // Process completed
                to_remove.push(*id);
            }
        }

        // Remove terminal processes and free resources
        for id in to_remove {
            if let Some(process) = self.processes.remove(&id) {
                // Safely deallocate resources
                let current_free = (self.cpu_available, self.ram_available);
                let (new_cpu, new_ram) = deallocate(
                    process.cpu_usage,
                    process.ram_usage,
                    self.hardware,
                    current_free,
                );

                self.cpu_available = new_cpu;
                self.ram_available = new_ram;

                info!("Process {} terminated, freed cpu={} ram={}",
                    id, process.cpu_usage.0, process.ram_usage.0);
            }
        }
    }

    pub fn get_state(&self) -> GameState {
        GameState {
            processes: self.processes.values().cloned().collect(),
            hardware: self.hardware,
            cpu_available: self.cpu_available,
            ram_available: self.ram_available,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GameState {
    pub processes: Vec<Process>,
    pub hardware: ResourceCaps,
    pub cpu_available: Units,
    pub ram_available: Units,
}

// Serialize implementations for API responses
impl Serialize for Units {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl Serialize for ResourceCaps {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ResourceCaps", 2)?;
        state.serialize_field("cpu", &self.cpu.0)?;
        state.serialize_field("ram", &self.ram.0)?;
        state.end()
    }
}

impl Serialize for ProcessState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let state_str = match self {
            ProcessState::Queued => "queued",
            ProcessState::Running => "running",
            ProcessState::Cancelling => "cancelling",
            ProcessState::Cancelled => "cancelled",
            ProcessState::Completed => "completed",
            ProcessState::Failed => "failed",
        };
        state_str.serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idempotent_cancel() {
        let mut engine = GameEngine::new();

        // Start a process
        let id = engine.start_process("Mine".to_string(), "normal".to_string(), None).map_err(|e| anyhow::anyhow!("Error: {}", e))?;

        // Cancel it multiple times - should all succeed
        assert!(engine.cancel_process(id).is_ok());
        assert!(engine.cancel_process(id).is_ok()); // Idempotent
        assert!(engine.cancel_process(id).is_ok()); // Still ok

        // Update to process cancellation
        engine.update();

        // Cancel non-existent process - should succeed (idempotent)
        let fake_id = Uuid::new_v4();
        assert!(engine.cancel_process(fake_id).is_ok());
    }

    #[test]
    fn test_resource_overflow_protection() {
        let mut engine = GameEngine::new();

        // Try to allocate more than available
        let result = engine.start_process("Mine".to_string(), "high".to_string(), None);
        assert!(result.is_ok());

        // Try to start another huge process - should be rejected or clamped
        let result2 = engine.start_process("Mine".to_string(), "high".to_string(), None);
        assert!(result2.is_ok()); // Gets clamped allocation

        // Try to start when exhausted
        let result3 = engine.start_process("Mine".to_string(), "high".to_string(), None);
        assert!(result3.is_err()); // Should fail cleanly
    }

    #[test]
    fn test_concurrent_cancellation() {
        let mut engine = GameEngine::new();

        // Start multiple processes
        let id1 = engine.start_process("Scan".to_string(), "normal".to_string(), None).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let id2 = engine.start_process("Download".to_string(), "normal".to_string(), None).map_err(|e| anyhow::anyhow!("Error: {}", e))?;

        // Cancel both
        assert!(engine.cancel_process(id1).is_ok());
        assert!(engine.cancel_process(id2).is_ok());

        // Let one complete naturally while other is cancelling
        for _ in 0..10 {
            engine.update();
        }

        // Both should be gone, resources freed
        assert!(engine.processes.is_empty() ||
                engine.processes.values().all(|p| p.state.is_terminal()));
    }
}