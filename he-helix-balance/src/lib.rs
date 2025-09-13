//! Helix Game Balance System

pub mod software;

use he_helix_factor::Factor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Game balance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceConfig {
    pub software: SoftwareBalance,
    pub hardware: HardwareBalance,
    pub network: NetworkBalance,
}

/// Software-related balance parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareBalance {
    pub base_execution_time: f64,
    pub complexity_multiplier: f64,
    pub skill_factor: f64,
}

/// Hardware-related balance parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareBalance {
    pub cpu_factor: f64,
    pub ram_factor: f64,
    pub storage_factor: f64,
}

/// Network-related balance parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkBalance {
    pub bandwidth_factor: f64,
    pub latency_factor: f64,
    pub connection_stability: f64,
}

impl Default for BalanceConfig {
    fn default() -> Self {
        Self {
            software: SoftwareBalance {
                base_execution_time: 1.0,
                complexity_multiplier: 1.5,
                skill_factor: 0.8,
            },
            hardware: HardwareBalance {
                cpu_factor: 1.0,
                ram_factor: 0.9,
                storage_factor: 0.7,
            },
            network: NetworkBalance {
                bandwidth_factor: 1.2,
                latency_factor: 0.8,
                connection_stability: 0.95,
            },
        }
    }
}

/// Balance calculator for game mechanics
pub struct BalanceCalculator {
    config: BalanceConfig,
}

impl BalanceCalculator {
    pub fn new(config: BalanceConfig) -> Self {
        Self { config }
    }

    /// Calculate execution time for a software process
    pub fn calculate_execution_time(
        &self,
        base_time: f64,
        complexity: f64,
        cpu_power: f64,
        skill_level: f64,
    ) -> f64 {
        let software_factor = base_time * (1.0 + complexity * self.config.software.complexity_multiplier);
        let skill_factor = 1.0 + (skill_level * self.config.software.skill_factor);
        let hardware_factor = cpu_power * self.config.hardware.cpu_factor;

        software_factor / (skill_factor * hardware_factor)
    }
}

impl Default for BalanceCalculator {
    fn default() -> Self {
        Self::new(BalanceConfig::default())
    }
}