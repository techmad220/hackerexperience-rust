//! Software-specific balance calculations

use serde::{Deserialize, Serialize};

/// Software types and their balance parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoftwareType {
    Cracker,
    Virus,
    Firewall,
    LogForge,
    Encryptor,
    Decryptor,
}

/// Software balance parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareBalanceParams {
    pub base_complexity: f64,
    pub resource_usage: ResourceUsage,
    pub effectiveness_factors: EffectivenessFactors,
}

/// Resource usage parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_usage: f64,
    pub ram_usage: f64,
    pub network_usage: f64,
}

/// Effectiveness factors for different scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectivenessFactors {
    pub skill_multiplier: f64,
    pub hardware_dependency: f64,
    pub network_dependency: f64,
}

impl SoftwareType {
    /// Get default balance parameters for this software type
    pub fn balance_params(&self) -> SoftwareBalanceParams {
        match self {
            SoftwareType::Cracker => SoftwareBalanceParams {
                base_complexity: 1.5,
                resource_usage: ResourceUsage {
                    cpu_usage: 0.8,
                    ram_usage: 0.6,
                    network_usage: 0.3,
                },
                effectiveness_factors: EffectivenessFactors {
                    skill_multiplier: 1.4,
                    hardware_dependency: 0.7,
                    network_dependency: 0.3,
                },
            },
            SoftwareType::Virus => SoftwareBalanceParams {
                base_complexity: 2.0,
                resource_usage: ResourceUsage {
                    cpu_usage: 0.6,
                    ram_usage: 0.4,
                    network_usage: 0.8,
                },
                effectiveness_factors: EffectivenessFactors {
                    skill_multiplier: 1.6,
                    hardware_dependency: 0.5,
                    network_dependency: 0.9,
                },
            },
            SoftwareType::Firewall => SoftwareBalanceParams {
                base_complexity: 1.8,
                resource_usage: ResourceUsage {
                    cpu_usage: 0.7,
                    ram_usage: 0.8,
                    network_usage: 0.9,
                },
                effectiveness_factors: EffectivenessFactors {
                    skill_multiplier: 1.2,
                    hardware_dependency: 0.8,
                    network_dependency: 0.6,
                },
            },
            _ => SoftwareBalanceParams {
                base_complexity: 1.0,
                resource_usage: ResourceUsage {
                    cpu_usage: 0.5,
                    ram_usage: 0.5,
                    network_usage: 0.5,
                },
                effectiveness_factors: EffectivenessFactors {
                    skill_multiplier: 1.0,
                    hardware_dependency: 0.6,
                    network_dependency: 0.4,
                },
            },
        }
    }
}