//! Resource calculation and management for servers and components

use crate::types::*;
use crate::model::{Component, Motherboard};
use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub, Mul, Div};

/// Resource container for CPU, RAM, HDD, and Network capacity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Resources {
    /// CPU processing power (in units)
    pub cpu: u32,
    /// RAM memory capacity (in MB)
    pub ram: u64,
    /// Hard disk storage (in MB) 
    pub hdd: u64,
    /// Network bandwidth (in Mbps)
    pub net: u32,
}

impl Resources {
    /// Create a new resource container with zero values
    pub fn new() -> Self {
        Self {
            cpu: 0,
            ram: 0,
            hdd: 0,
            net: 0,
        }
    }

    /// Create resources with specific values
    pub fn new_with_values(cpu: u32, ram: u64, hdd: u64, net: u32) -> Self {
        Self { cpu, ram, hdd, net }
    }

    /// Check if all resource values are zero
    pub fn is_empty(&self) -> bool {
        self.cpu == 0 && self.ram == 0 && self.hdd == 0 && self.net == 0
    }

    /// Check if this resource container can satisfy the given requirements
    pub fn can_satisfy(&self, required: &Resources) -> bool {
        self.cpu >= required.cpu
            && self.ram >= required.ram
            && self.hdd >= required.hdd
            && self.net >= required.net
    }

    /// Get the remaining capacity after subtracting used resources
    pub fn remaining_after(&self, used: &Resources) -> Resources {
        Resources {
            cpu: self.cpu.saturating_sub(used.cpu),
            ram: self.ram.saturating_sub(used.ram),
            hdd: self.hdd.saturating_sub(used.hdd),
            net: self.net.saturating_sub(used.net),
        }
    }

    /// Calculate percentage usage based on total capacity
    pub fn usage_percentage(&self, total: &Resources) -> ResourceUsagePercentage {
        ResourceUsagePercentage {
            cpu: if total.cpu > 0 { (self.cpu * 100) / total.cpu } else { 0 },
            ram: if total.ram > 0 { ((self.ram * 100) / total.ram) as u32 } else { 0 },
            hdd: if total.hdd > 0 { ((self.hdd * 100) / total.hdd) as u32 } else { 0 },
            net: if total.net > 0 { (self.net * 100) / total.net } else { 0 },
        }
    }

    /// Apply a multiplier to all resource values
    pub fn multiply_by(&self, multiplier: f32) -> Resources {
        Resources {
            cpu: (self.cpu as f32 * multiplier) as u32,
            ram: (self.ram as f32 * multiplier) as u64,
            hdd: (self.hdd as f32 * multiplier) as u64,
            net: (self.net as f32 * multiplier) as u32,
        }
    }

    /// Get the maximum resource value across all types (normalized)
    pub fn max_normalized_value(&self) -> f64 {
        let cpu_normalized = self.cpu as f64;
        let ram_normalized = self.ram as f64 / 1024.0; // Convert MB to GB for normalization
        let hdd_normalized = self.hdd as f64 / 1024.0; // Convert MB to GB for normalization
        let net_normalized = self.net as f64;

        cpu_normalized.max(ram_normalized).max(hdd_normalized).max(net_normalized)
    }
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}

impl Add for Resources {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            cpu: self.cpu + other.cpu,
            ram: self.ram + other.ram,
            hdd: self.hdd + other.hdd,
            net: self.net + other.net,
        }
    }
}

impl Sub for Resources {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            cpu: self.cpu.saturating_sub(other.cpu),
            ram: self.ram.saturating_sub(other.ram),
            hdd: self.hdd.saturating_sub(other.hdd),
            net: self.net.saturating_sub(other.net),
        }
    }
}

impl Mul<f32> for Resources {
    type Output = Self;

    fn mul(self, multiplier: f32) -> Self {
        self.multiply_by(multiplier)
    }
}

/// Resource usage percentages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceUsagePercentage {
    pub cpu: u32, // 0-100%
    pub ram: u32, // 0-100%
    pub hdd: u32, // 0-100%
    pub net: u32, // 0-100%
}

/// Resource calculator for computing component and motherboard capacities
pub struct ResourceCalculator;

impl ResourceCalculator {
    /// Calculate the total resources provided by a component based on its type and specs
    pub fn calculate_component_resources(component: &Component) -> Resources {
        match component.component_type {
            ComponentType::Cpu => Resources::new_with_values(component.spec_value, 0, 0, 0),
            ComponentType::Ram => Resources::new_with_values(0, component.spec_value as u64, 0, 0),
            ComponentType::Hdd => Resources::new_with_values(0, 0, component.spec_value as u64, 0),
            ComponentType::Nic => Resources::new_with_values(0, 0, 0, component.spec_value),
            ComponentType::Mobo => Resources::new(), // Motherboards don't provide resources directly
        }
    }

    /// Calculate total resources for a motherboard by summing all connected components
    pub fn calculate_motherboard_resources(
        motherboard: &Motherboard,
        components: &[Component],
    ) -> Resources {
        let mut total_resources = Resources::new();

        for slot in &motherboard.slots {
            if let Some(component_id) = slot.component_id {
                if let Some(component) = components.iter().find(|c| c.component_id == component_id) {
                    total_resources = total_resources + Self::calculate_component_resources(component);
                }
            }
        }

        total_resources
    }

    /// Calculate the maximum theoretical resources for a motherboard (all slots filled with max specs)
    pub fn calculate_max_motherboard_resources(motherboard: &Motherboard, max_specs: &ComponentMaxSpecs) -> Resources {
        let mut max_resources = Resources::new();

        for component_type in ComponentType::all_types() {
            if !component_type.is_pluggable() {
                continue;
            }

            let slot_count = motherboard.slot_count_for_type(*component_type) as u32;
            let max_spec = max_specs.get_max_spec(*component_type);

            match component_type {
                ComponentType::Cpu => max_resources.cpu += slot_count * max_spec,
                ComponentType::Ram => max_resources.ram += (slot_count * max_spec) as u64,
                ComponentType::Hdd => max_resources.hdd += (slot_count * max_spec) as u64,
                ComponentType::Nic => max_resources.net += slot_count * max_spec,
                ComponentType::Mobo => {} // Skip motherboard
            }
        }

        max_resources
    }

    /// Calculate resource efficiency (actual vs theoretical maximum)
    pub fn calculate_efficiency(
        actual: &Resources,
        theoretical_max: &Resources,
    ) -> ResourceUsagePercentage {
        actual.usage_percentage(theoretical_max)
    }
}

/// Maximum specification values for different component types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMaxSpecs {
    pub cpu_max: u32,
    pub ram_max: u32,
    pub hdd_max: u32,
    pub nic_max: u32,
}

impl ComponentMaxSpecs {
    /// Create default maximum specifications
    pub fn default_specs() -> Self {
        Self {
            cpu_max: 1000,      // Max CPU power units
            ram_max: 32768,     // Max RAM in MB (32GB)
            hdd_max: 1048576,   // Max HDD in MB (1TB)
            nic_max: 1000,      // Max network speed in Mbps (1Gbps)
        }
    }

    /// Get the maximum spec value for a given component type
    pub fn get_max_spec(&self, component_type: ComponentType) -> u32 {
        match component_type {
            ComponentType::Cpu => self.cpu_max,
            ComponentType::Ram => self.ram_max,
            ComponentType::Hdd => self.hdd_max,
            ComponentType::Nic => self.nic_max,
            ComponentType::Mobo => 0, // Motherboards don't have direct specs
        }
    }
}

impl Default for ComponentMaxSpecs {
    fn default() -> Self {
        Self::default_specs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_resources_arithmetic() {
        let res1 = Resources::new_with_values(100, 1024, 512, 50);
        let res2 = Resources::new_with_values(50, 512, 256, 25);

        let sum = res1 + res2;
        assert_eq!(sum, Resources::new_with_values(150, 1536, 768, 75));

        let diff = res1 - res2;
        assert_eq!(diff, Resources::new_with_values(50, 512, 256, 25));
    }

    #[test]
    fn test_resource_can_satisfy() {
        let available = Resources::new_with_values(100, 1024, 512, 50);
        let required = Resources::new_with_values(50, 512, 256, 25);
        let excessive = Resources::new_with_values(200, 2048, 1024, 100);

        assert!(available.can_satisfy(&required));
        assert!(!available.can_satisfy(&excessive));
    }

    #[test]
    fn test_component_resource_calculation() {
        let cpu = Component::new(Uuid::new_v4(), ComponentType::Cpu, 100);
        let resources = ResourceCalculator::calculate_component_resources(&cpu);
        
        assert_eq!(resources.cpu, 100);
        assert_eq!(resources.ram, 0);
        assert_eq!(resources.hdd, 0);
        assert_eq!(resources.net, 0);
    }

    #[test]
    fn test_usage_percentage() {
        let used = Resources::new_with_values(50, 512, 256, 25);
        let total = Resources::new_with_values(100, 1024, 512, 50);
        
        let percentage = used.usage_percentage(&total);
        assert_eq!(percentage.cpu, 50);
        assert_eq!(percentage.ram, 50);
        assert_eq!(percentage.hdd, 50);
        assert_eq!(percentage.net, 50);
    }

    #[test]
    fn test_resource_multiply() {
        let resources = Resources::new_with_values(100, 1024, 512, 50);
        let doubled = resources * 2.0;
        
        assert_eq!(doubled.cpu, 200);
        assert_eq!(doubled.ram, 2048);
        assert_eq!(doubled.hdd, 1024);
        assert_eq!(doubled.net, 100);
    }
}