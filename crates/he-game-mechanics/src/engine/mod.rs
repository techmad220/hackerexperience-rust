//! Core Game Engine - The heart of HackerExperience game mechanics
//!
//! This module contains the complete implementation of all core game systems:
//! - Process scheduling and execution
//! - Hardware performance calculations
//! - Software dependency management
//! - Network topology and routing

pub mod process_engine;
pub mod hardware_engine;
pub mod software_engine;
pub mod network_engine;
pub mod game_engine;

pub use process_engine::{ProcessEngine, ProcessScheduler, ProcessExecutor};
pub use hardware_engine::{HardwareEngine, HardwareCalculator};
pub use software_engine::{SoftwareEngine, DependencyResolver};
pub use network_engine::{NetworkEngine, NetworkTopology};
pub use game_engine::{GameEngine, GameState};

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Result type for engine operations
pub type EngineResult<T> = Result<T, EngineError>;

/// Engine errors
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Process error: {0}")]
    ProcessError(String),

    #[error("Hardware error: {0}")]
    HardwareError(String),

    #[error("Software error: {0}")]
    SoftwareError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

/// Common trait for all engine components
pub trait EngineComponent {
    /// Initialize the component
    fn initialize(&mut self) -> EngineResult<()>;

    /// Update the component state (called each tick)
    fn update(&mut self, delta: Duration) -> EngineResult<()>;

    /// Get component status
    fn status(&self) -> ComponentStatus;

    /// Reset component to initial state
    fn reset(&mut self) -> EngineResult<()>;
}

/// Component status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStatus {
    pub name: String,
    pub healthy: bool,
    pub last_update: SystemTime,
    pub metrics: Vec<(String, f64)>,
}

/// Resource types used by processes
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Resources {
    pub cpu: f32,      // MHz used
    pub ram: f32,      // MB used
    pub disk: f32,     // MB used
    pub network: f32,  // Mbps used
}

impl Resources {
    pub fn new(cpu: f32, ram: f32, disk: f32, network: f32) -> Self {
        Self { cpu, ram, disk, network }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    pub fn can_allocate(&self, required: &Resources, available: &Resources) -> bool {
        self.cpu + required.cpu <= available.cpu &&
        self.ram + required.ram <= available.ram &&
        self.disk + required.disk <= available.disk &&
        self.network + required.network <= available.network
    }

    pub fn allocate(&mut self, amount: &Resources) {
        self.cpu += amount.cpu;
        self.ram += amount.ram;
        self.disk += amount.disk;
        self.network += amount.network;
    }

    pub fn deallocate(&mut self, amount: &Resources) {
        self.cpu = (self.cpu - amount.cpu).max(0.0);
        self.ram = (self.ram - amount.ram).max(0.0);
        self.disk = (self.disk - amount.disk).max(0.0);
        self.network = (self.network - amount.network).max(0.0);
    }
}