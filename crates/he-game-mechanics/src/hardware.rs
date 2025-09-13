//! Hardware system mechanics - Performance ratings, compatibility, upgrade mechanics

use crate::{HardwareSpecs, PlayerState};
use crate::config::HardwareConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Hardware component types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HardwareType {
    CPU,
    RAM,
    HDD,
    NetworkCard,
    GPU,
    PowerSupply,
    Motherboard,
    Cooler,
    ExternalHDD,
    Router,
    Firewall,
    ServerRack,
}

impl HardwareType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "cpu" | "processor" => HardwareType::CPU,
            "ram" | "memory" => HardwareType::RAM,
            "hdd" | "harddisk" | "storage" => HardwareType::HDD,
            "network" | "networkcard" | "nic" => HardwareType::NetworkCard,
            "gpu" | "graphics" | "videocard" => HardwareType::GPU,
            "psu" | "power" | "powersupply" => HardwareType::PowerSupply,
            "motherboard" | "mobo" | "mainboard" => HardwareType::Motherboard,
            "cooler" | "cooling" | "heatsink" => HardwareType::Cooler,
            "external" | "externalhdd" => HardwareType::ExternalHDD,
            "router" => HardwareType::Router,
            "firewall" => HardwareType::Firewall,
            "serverrack" | "rack" => HardwareType::ServerRack,
            _ => HardwareType::CPU, // Default fallback
        }
    }
    
    pub fn max_slots(&self) -> usize {
        match self {
            HardwareType::CPU => 1,
            HardwareType::RAM => 4,
            HardwareType::HDD => 6,
            HardwareType::NetworkCard => 2,
            HardwareType::GPU => 2,
            HardwareType::PowerSupply => 1,
            HardwareType::Motherboard => 1,
            HardwareType::Cooler => 2,
            HardwareType::ExternalHDD => 8,
            HardwareType::Router => 1,
            HardwareType::Firewall => 1,
            HardwareType::ServerRack => 4,
        }
    }
    
    pub fn power_consumption_base(&self) -> i32 {
        match self {
            HardwareType::CPU => 95,
            HardwareType::RAM => 5,
            HardwareType::HDD => 10,
            HardwareType::NetworkCard => 15,
            HardwareType::GPU => 150,
            HardwareType::PowerSupply => 0, // PSU provides power
            HardwareType::Motherboard => 25,
            HardwareType::Cooler => 5,
            HardwareType::ExternalHDD => 15,
            HardwareType::Router => 20,
            HardwareType::Firewall => 30,
            HardwareType::ServerRack => 50,
        }
    }
}

/// Hardware component quality tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareQuality {
    Basic,
    Standard,
    Advanced,
    Professional,
    Elite,
    Quantum,
}

impl HardwareQuality {
    pub fn performance_multiplier(&self) -> f32 {
        match self {
            HardwareQuality::Basic => 1.0,
            HardwareQuality::Standard => 1.5,
            HardwareQuality::Advanced => 2.0,
            HardwareQuality::Professional => 3.0,
            HardwareQuality::Elite => 5.0,
            HardwareQuality::Quantum => 10.0,
        }
    }
    
    pub fn price_multiplier(&self) -> f32 {
        match self {
            HardwareQuality::Basic => 1.0,
            HardwareQuality::Standard => 2.5,
            HardwareQuality::Advanced => 6.0,
            HardwareQuality::Professional => 15.0,
            HardwareQuality::Elite => 40.0,
            HardwareQuality::Quantum => 100.0,
        }
    }
    
    pub fn durability_multiplier(&self) -> f32 {
        match self {
            HardwareQuality::Basic => 1.0,
            HardwareQuality::Standard => 1.2,
            HardwareQuality::Advanced => 1.5,
            HardwareQuality::Professional => 2.0,
            HardwareQuality::Elite => 3.0,
            HardwareQuality::Quantum => 5.0,
        }
    }
    
    pub fn from_tier(tier: i32) -> Self {
        match tier {
            0..=1 => HardwareQuality::Basic,
            2..=3 => HardwareQuality::Standard,
            4..=5 => HardwareQuality::Advanced,
            6..=7 => HardwareQuality::Professional,
            8..=9 => HardwareQuality::Elite,
            _ => HardwareQuality::Quantum,
        }
    }
}

/// Individual hardware component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareComponent {
    pub id: i32,
    pub component_type: HardwareType,
    pub quality: HardwareQuality,
    pub name: String,
    pub manufacturer: String,
    pub model: String,
    pub base_value: i32,
    pub current_value: i32,
    pub max_value: i32,
    pub durability: f32,
    pub max_durability: f32,
    pub temperature: f32,
    pub max_temperature: f32,
    pub power_consumption: i32,
    pub installed_at: Option<SystemTime>,
    pub last_maintained: Option<SystemTime>,
    pub overclock_level: f32,
    pub firmware_version: String,
    pub is_damaged: bool,
    pub damage_level: f32,
    pub warranty_expires: Option<SystemTime>,
    pub custom_mods: Vec<String>,
}

impl HardwareComponent {
    pub fn new(component_type: HardwareType, quality: HardwareQuality, base_value: i32) -> Self {
        let max_value = (base_value as f32 * quality.performance_multiplier()) as i32;
        let max_durability = 100.0 * quality.durability_multiplier();
        
        HardwareComponent {
            id: rand::random::<i32>().abs(),
            component_type: component_type.clone(),
            quality,
            name: format!("{:?} Component", component_type),
            manufacturer: Self::random_manufacturer(&component_type),
            model: Self::generate_model_number(&component_type, &quality),
            base_value,
            current_value: base_value,
            max_value,
            durability: max_durability,
            max_durability,
            temperature: 30.0,
            max_temperature: 90.0,
            power_consumption: component_type.power_consumption_base(),
            installed_at: None,
            last_maintained: None,
            overclock_level: 1.0,
            firmware_version: "1.0.0".to_string(),
            is_damaged: false,
            damage_level: 0.0,
            warranty_expires: Some(SystemTime::now() + Duration::from_secs(365 * 24 * 3600)),
            custom_mods: Vec::new(),
        }
    }
    
    fn random_manufacturer(component_type: &HardwareType) -> String {
        match component_type {
            HardwareType::CPU => {
                ["Intel", "AMD", "ARM", "Quantum"][rand::random::<usize>() % 4].to_string()
            },
            HardwareType::GPU => {
                ["NVIDIA", "AMD", "Intel", "Quantum Graphics"][rand::random::<usize>() % 4].to_string()
            },
            HardwareType::RAM => {
                ["Corsair", "G.Skill", "Kingston", "Crucial"][rand::random::<usize>() % 4].to_string()
            },
            HardwareType::HDD => {
                ["Seagate", "Western Digital", "Samsung", "Toshiba"][rand::random::<usize>() % 4].to_string()
            },
            _ => "Generic".to_string()
        }
    }
    
    fn generate_model_number(component_type: &HardwareType, quality: &HardwareQuality) -> String {
        let prefix = match component_type {
            HardwareType::CPU => "C",
            HardwareType::GPU => "G",
            HardwareType::RAM => "M",
            HardwareType::HDD => "D",
            HardwareType::NetworkCard => "N",
            _ => "X",
        };
        
        let tier = match quality {
            HardwareQuality::Basic => "100",
            HardwareQuality::Standard => "200",
            HardwareQuality::Advanced => "300",
            HardwareQuality::Professional => "500",
            HardwareQuality::Elite => "700",
            HardwareQuality::Quantum => "900",
        };
        
        format!("{}{}-{}", prefix, tier, rand::random::<u16>() % 1000)
    }
    
    pub fn get_effective_value(&self) -> i32 {
        let base = self.current_value as f32;
        let durability_factor = self.durability / self.max_durability;
        let overclock_bonus = (self.overclock_level - 1.0) * 0.2;
        let damage_penalty = self.damage_level / 100.0;
        let temperature_penalty = if self.temperature > 80.0 {
            ((self.temperature - 80.0) / 20.0) * 0.2
        } else {
            0.0
        };
        
        (base * durability_factor * (1.0 + overclock_bonus) * (1.0 - damage_penalty) * (1.0 - temperature_penalty)) as i32
    }
    
    pub fn apply_wear(&mut self, usage_hours: f32, config: &HardwareConfig) {
        let wear_rate = config.wear_rate_per_hour * usage_hours;
        let quality_factor = 1.0 / self.quality.durability_multiplier();
        let overclock_penalty = (self.overclock_level - 1.0).max(0.0) * 2.0;
        
        let total_wear = wear_rate * quality_factor * (1.0 + overclock_penalty);
        self.durability = (self.durability - total_wear).max(0.0);
        
        if self.durability < 20.0 {
            self.is_damaged = true;
            self.damage_level = 20.0 - self.durability;
        }
        
        // Temperature increases with usage
        self.temperature = (self.temperature + usage_hours * 2.0).min(self.max_temperature);
    }
    
    pub fn repair(&mut self, repair_quality: f32) {
        let repair_amount = self.max_durability * repair_quality;
        self.durability = (self.durability + repair_amount).min(self.max_durability);
        
        if self.durability > 20.0 {
            self.is_damaged = false;
            self.damage_level = 0.0;
        }
        
        self.last_maintained = Some(SystemTime::now());
        self.temperature = 30.0; // Reset temperature after maintenance
    }
    
    pub fn overclock(&mut self, level: f32) -> Result<(), String> {
        if level < 0.5 || level > 2.0 {
            return Err("Overclock level must be between 0.5 and 2.0".to_string());
        }
        
        if self.is_damaged {
            return Err("Cannot overclock damaged hardware".to_string());
        }
        
        if self.durability < 50.0 {
            return Err("Hardware durability too low for overclocking".to_string());
        }
        
        self.overclock_level = level;
        self.power_consumption = (self.power_consumption as f32 * level) as i32;
        self.max_temperature = 90.0 - (level - 1.0) * 20.0; // Higher OC = lower safe temp
        
        Ok(())
    }
    
    pub fn upgrade_firmware(&mut self, version: String) {
        self.firmware_version = version;
        self.current_value = (self.current_value as f32 * 1.05).min(self.max_value as f32) as i32;
    }
    
    pub fn add_custom_mod(&mut self, mod_name: String) -> Result<(), String> {
        if self.custom_mods.len() >= 3 {
            return Err("Maximum custom mods reached".to_string());
        }
        
        if self.custom_mods.contains(&mod_name) {
            return Err("Mod already installed".to_string());
        }
        
        self.custom_mods.push(mod_name);
        self.max_value = (self.max_value as f32 * 1.1) as i32;
        self.current_value = (self.current_value as f32 * 1.05) as i32;
        
        Ok(())
    }
    
    pub fn calculate_resale_value(&self) -> i32 {
        let base_price = (self.base_value as f32 * self.quality.price_multiplier()) as i32;
        let durability_factor = self.durability / self.max_durability;
        let damage_penalty = if self.is_damaged { 0.5 } else { 1.0 };
        let mod_bonus = 1.0 + (self.custom_mods.len() as f32 * 0.1);
        
        (base_price as f32 * durability_factor * damage_penalty * mod_bonus * 0.6) as i32
    }
}

/// Hardware inventory management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInventory {
    pub components: HashMap<HardwareType, Vec<HardwareComponent>>,
    pub installed: HashMap<HardwareType, Vec<HardwareComponent>>,
    pub total_power_consumption: i32,
    pub total_heat_generation: f32,
    pub cooling_capacity: f32,
    pub max_slots: HashMap<HardwareType, usize>,
}

impl HardwareInventory {
    pub fn new() -> Self {
        let mut max_slots = HashMap::new();
        max_slots.insert(HardwareType::CPU, 1);
        max_slots.insert(HardwareType::RAM, 4);
        max_slots.insert(HardwareType::HDD, 6);
        max_slots.insert(HardwareType::NetworkCard, 2);
        max_slots.insert(HardwareType::GPU, 2);
        max_slots.insert(HardwareType::PowerSupply, 1);
        max_slots.insert(HardwareType::Motherboard, 1);
        max_slots.insert(HardwareType::Cooler, 2);
        
        HardwareInventory {
            components: HashMap::new(),
            installed: HashMap::new(),
            total_power_consumption: 0,
            total_heat_generation: 0.0,
            cooling_capacity: 100.0,
            max_slots,
        }
    }
    
    pub fn add_component(&mut self, component: HardwareComponent) {
        self.components
            .entry(component.component_type.clone())
            .or_insert_with(Vec::new)
            .push(component);
    }
    
    pub fn install_component(&mut self, component_type: HardwareType, component_id: i32) -> Result<(), String> {
        // Check if we have the component
        let components = self.components.get_mut(&component_type)
            .ok_or("No components of this type in inventory")?;
        
        let component_index = components.iter()
            .position(|c| c.id == component_id)
            .ok_or("Component not found in inventory")?;
        
        // Check slot availability
        let installed_count = self.installed.get(&component_type).map(|v| v.len()).unwrap_or(0);
        let max_slots = self.max_slots.get(&component_type).copied().unwrap_or(1);
        
        if installed_count >= max_slots {
            return Err(format!("All {} slots are occupied", component_type.clone() as i32));
        }
        
        // Remove from inventory and install
        let mut component = components.remove(component_index);
        component.installed_at = Some(SystemTime::now());
        
        // Update power consumption
        self.total_power_consumption += component.power_consumption;
        self.total_heat_generation += component.temperature - 30.0;
        
        self.installed
            .entry(component_type)
            .or_insert_with(Vec::new)
            .push(component);
        
        Ok(())
    }
    
    pub fn uninstall_component(&mut self, component_type: HardwareType, component_id: i32) -> Result<(), String> {
        let installed = self.installed.get_mut(&component_type)
            .ok_or("No installed components of this type")?;
        
        let component_index = installed.iter()
            .position(|c| c.id == component_id)
            .ok_or("Component not found in installed hardware")?;
        
        let mut component = installed.remove(component_index);
        
        // Update power consumption
        self.total_power_consumption -= component.power_consumption;
        self.total_heat_generation -= component.temperature - 30.0;
        
        component.installed_at = None;
        self.add_component(component);
        
        Ok(())
    }
    
    pub fn get_total_value(&self, component_type: &HardwareType) -> i32 {
        self.installed
            .get(component_type)
            .map(|components| {
                components.iter()
                    .map(|c| c.get_effective_value())
                    .sum()
            })
            .unwrap_or(0)
    }
    
    pub fn apply_usage(&mut self, hours: f32, config: &HardwareConfig) {
        for components in self.installed.values_mut() {
            for component in components.iter_mut() {
                component.apply_wear(hours, config);
            }
        }
        
        // Update total heat
        self.total_heat_generation = self.installed.values()
            .flat_map(|components| components.iter())
            .map(|c| c.temperature - 30.0)
            .sum();
    }
    
    pub fn perform_maintenance(&mut self) {
        for components in self.installed.values_mut() {
            for component in components.iter_mut() {
                component.repair(0.3); // Basic maintenance restores 30% durability
            }
        }
    }
    
    pub fn check_compatibility(&self, new_component: &HardwareComponent) -> Result<(), String> {
        // Check power supply capacity
        if let Some(psu) = self.installed.get(&HardwareType::PowerSupply)
            .and_then(|v| v.first()) {
            let available_power = psu.get_effective_value();
            if self.total_power_consumption + new_component.power_consumption > available_power {
                return Err("Insufficient power supply capacity".to_string());
            }
        }
        
        // Check motherboard compatibility for CPU/RAM
        if matches!(new_component.component_type, HardwareType::CPU | HardwareType::RAM) {
            if let Some(mobo) = self.installed.get(&HardwareType::Motherboard)
                .and_then(|v| v.first()) {
                if new_component.quality as i32 > mobo.quality as i32 + 2 {
                    return Err("Component too advanced for current motherboard".to_string());
                }
            } else {
                return Err("Motherboard required for this component".to_string());
            }
        }
        
        // Check cooling capacity
        let projected_heat = self.total_heat_generation + (new_component.temperature - 30.0);
        if projected_heat > self.cooling_capacity {
            return Err("Insufficient cooling capacity".to_string());
        }
        
        Ok(())
    }
}

/// Calculate system performance rating
pub fn calculate_performance_rating(specs: &HardwareSpecs, config: &HardwareConfig) -> i32 {
    let cpu_score = (specs.cpu_mhz as f32 / config.base_cpu_mhz as f32) * 25.0;
    let ram_score = (specs.ram_mb as f32 / config.base_ram_mb as f32) * 20.0;
    let hdd_score = (specs.hdd_gb as f32 / config.base_hdd_gb as f32) * 15.0;
    let net_score = (specs.net_speed as f32 / config.base_net_speed as f32) * 20.0;
    let gpu_score = specs.gpu_cores.map(|cores| (cores as f32 / 1000.0) * 20.0).unwrap_or(0.0);
    
    (cpu_score + ram_score + hdd_score + net_score + gpu_score).min(100.0) as i32
}

/// Calculate upgrade cost
pub fn calculate_upgrade_cost(
    current_value: i32,
    target_value: i32,
    component_type: &HardwareType,
    config: &HardwareConfig
) -> i32 {
    if target_value <= current_value {
        return 0;
    }
    
    let value_diff = target_value - current_value;
    let base_cost = match component_type {
        HardwareType::CPU => value_diff * config.cpu_cost_per_mhz,
        HardwareType::RAM => value_diff * config.ram_cost_per_mb,
        HardwareType::HDD => value_diff * config.hdd_cost_per_gb,
        HardwareType::NetworkCard => value_diff * config.net_cost_per_mbps,
        HardwareType::GPU => value_diff * 50,
        _ => value_diff * 10,
    };
    
    // Apply exponential scaling for large upgrades
    let scale_factor = (value_diff as f32 / current_value.max(1) as f32).powf(1.2);
    (base_cost as f32 * scale_factor) as i32
}

/// Hardware bottleneck analysis
pub fn analyze_bottlenecks(inventory: &HardwareInventory) -> Vec<String> {
    let mut bottlenecks = Vec::new();
    
    let cpu_value = inventory.get_total_value(&HardwareType::CPU);
    let ram_value = inventory.get_total_value(&HardwareType::RAM);
    let hdd_value = inventory.get_total_value(&HardwareType::HDD);
    let net_value = inventory.get_total_value(&HardwareType::NetworkCard);
    let gpu_value = inventory.get_total_value(&HardwareType::GPU);
    
    let avg_value = (cpu_value + ram_value + hdd_value + net_value + gpu_value) / 5;
    
    if cpu_value < avg_value * 70 / 100 {
        bottlenecks.push("CPU is bottlenecking system performance".to_string());
    }
    if ram_value < avg_value * 70 / 100 {
        bottlenecks.push("Insufficient RAM for optimal performance".to_string());
    }
    if hdd_value < avg_value * 70 / 100 {
        bottlenecks.push("Storage speed limiting system throughput".to_string());
    }
    if net_value < avg_value * 70 / 100 {
        bottlenecks.push("Network capacity restricting data transfer".to_string());
    }
    
    if inventory.total_heat_generation > inventory.cooling_capacity * 0.8 {
        bottlenecks.push("System running hot - additional cooling recommended".to_string());
    }
    
    if inventory.total_power_consumption > inventory.get_total_value(&HardwareType::PowerSupply) * 80 / 100 {
        bottlenecks.push("Power supply near capacity - upgrade recommended".to_string());
    }
    
    bottlenecks
}

/// Generate hardware recommendations
pub fn generate_upgrade_recommendations(
    player: &PlayerState,
    inventory: &HardwareInventory,
    budget: i32,
    config: &HardwareConfig
) -> Vec<(HardwareType, String, i32)> {
    let mut recommendations = Vec::new();
    
    // Analyze current setup
    let bottlenecks = analyze_bottlenecks(inventory);
    
    // Check CPU upgrade
    let cpu_value = inventory.get_total_value(&HardwareType::CPU);
    let cpu_target = (cpu_value as f32 * 1.5) as i32;
    let cpu_cost = calculate_upgrade_cost(cpu_value, cpu_target, &HardwareType::CPU, config);
    
    if cpu_cost <= budget && bottlenecks.iter().any(|b| b.contains("CPU")) {
        recommendations.push((
            HardwareType::CPU,
            format!("Upgrade CPU from {} to {} MHz", cpu_value, cpu_target),
            cpu_cost
        ));
    }
    
    // Check RAM upgrade
    let ram_value = inventory.get_total_value(&HardwareType::RAM);
    let ram_target = (ram_value as f32 * 2.0) as i32;
    let ram_cost = calculate_upgrade_cost(ram_value, ram_target, &HardwareType::RAM, config);
    
    if ram_cost <= budget && bottlenecks.iter().any(|b| b.contains("RAM")) {
        recommendations.push((
            HardwareType::RAM,
            format!("Upgrade RAM from {} to {} MB", ram_value, ram_target),
            ram_cost
        ));
    }
    
    // Check cooling upgrade if running hot
    if inventory.total_heat_generation > inventory.cooling_capacity * 0.8 {
        let cooling_cost = 500;
        if cooling_cost <= budget {
            recommendations.push((
                HardwareType::Cooler,
                "Add additional cooling system".to_string(),
                cooling_cost
            ));
        }
    }
    
    // Sort by cost-effectiveness
    recommendations.sort_by_key(|r| r.2);
    recommendations.truncate(3); // Return top 3 recommendations
    
    recommendations
}

/// Hardware failure simulation
pub fn simulate_failure(component: &mut HardwareComponent, failure_type: &str) -> String {
    match failure_type {
        "overheat" => {
            component.temperature = component.max_temperature + 10.0;
            component.is_damaged = true;
            component.damage_level = 50.0;
            component.durability = (component.durability * 0.5).max(0.0);
            "Component overheated and sustained thermal damage".to_string()
        },
        "power_surge" => {
            component.is_damaged = true;
            component.damage_level = 75.0;
            component.current_value = (component.current_value as f32 * 0.3) as i32;
            "Power surge damaged component circuits".to_string()
        },
        "mechanical" => {
            component.is_damaged = true;
            component.damage_level = 100.0;
            component.current_value = 0;
            "Mechanical failure - component non-functional".to_string()
        },
        "degradation" => {
            component.durability = 0.0;
            component.is_damaged = true;
            component.damage_level = 30.0;
            "Component degraded beyond operational limits".to_string()
        },
        _ => "Unknown failure type".to_string()
    }
}

/// Calculate total system specifications
pub fn calculate_system_specs(inventory: &HardwareInventory) -> HardwareSpecs {
    HardwareSpecs {
        cpu_mhz: inventory.get_total_value(&HardwareType::CPU),
        ram_mb: inventory.get_total_value(&HardwareType::RAM),
        hdd_gb: inventory.get_total_value(&HardwareType::HDD),
        net_speed: inventory.get_total_value(&HardwareType::NetworkCard),
        gpu_cores: if inventory.get_total_value(&HardwareType::GPU) > 0 {
            Some(inventory.get_total_value(&HardwareType::GPU))
        } else {
            None
        },
    }
}

/// Hardware market pricing
pub fn get_market_price(component_type: &HardwareType, quality: &HardwareQuality, base_value: i32) -> i32 {
    let base_price = match component_type {
        HardwareType::CPU => base_value * 2,
        HardwareType::GPU => base_value * 3,
        HardwareType::RAM => base_value / 2,
        HardwareType::HDD => base_value / 10,
        HardwareType::NetworkCard => base_value,
        _ => base_value,
    };
    
    (base_price as f32 * quality.price_multiplier()) as i32
}

/// Hardware compatibility matrix
pub fn check_component_compatibility(
    component1: &HardwareComponent,
    component2: &HardwareComponent
) -> Result<(), String> {
    // CPU and Motherboard compatibility
    if component1.component_type == HardwareType::CPU && component2.component_type == HardwareType::Motherboard {
        if component1.quality as i32 > component2.quality as i32 + 1 {
            return Err("CPU too advanced for motherboard".to_string());
        }
    }
    
    // RAM and Motherboard compatibility
    if component1.component_type == HardwareType::RAM && component2.component_type == HardwareType::Motherboard {
        if component1.quality as i32 > component2.quality as i32 + 2 {
            return Err("RAM incompatible with motherboard".to_string());
        }
    }
    
    // GPU and Power Supply compatibility
    if component1.component_type == HardwareType::GPU && component2.component_type == HardwareType::PowerSupply {
        if component1.power_consumption > component2.get_effective_value() / 2 {
            return Err("GPU requires more powerful PSU".to_string());
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hardware_component_creation() {
        let component = HardwareComponent::new(HardwareType::CPU, HardwareQuality::Advanced, 3000);
        assert_eq!(component.component_type, HardwareType::CPU);
        assert_eq!(component.quality, HardwareQuality::Advanced);
        assert_eq!(component.base_value, 3000);
        assert_eq!(component.max_value, 6000); // 3000 * 2.0 (Advanced multiplier)
    }
    
    #[test]
    fn test_component_wear_and_repair() {
        let config = HardwareConfig::default();
        let mut component = HardwareComponent::new(HardwareType::RAM, HardwareQuality::Standard, 4096);
        
        let initial_durability = component.durability;
        component.apply_wear(10.0, &config);
        assert!(component.durability < initial_durability);
        
        component.repair(0.5);
        assert!(component.durability > initial_durability * 0.4);
    }
    
    #[test]
    fn test_inventory_management() {
        let mut inventory = HardwareInventory::new();
        let cpu = HardwareComponent::new(HardwareType::CPU, HardwareQuality::Professional, 4000);
        let cpu_id = cpu.id;
        
        inventory.add_component(cpu);
        assert_eq!(inventory.components.get(&HardwareType::CPU).unwrap().len(), 1);
        
        inventory.install_component(HardwareType::CPU, cpu_id).unwrap();
        assert_eq!(inventory.installed.get(&HardwareType::CPU).unwrap().len(), 1);
        assert_eq!(inventory.components.get(&HardwareType::CPU).map(|v| v.len()).unwrap_or(0), 0);
    }
    
    #[test]
    fn test_overclock() {
        let mut component = HardwareComponent::new(HardwareType::CPU, HardwareQuality::Elite, 5000);
        
        assert!(component.overclock(1.5).is_ok());
        assert_eq!(component.overclock_level, 1.5);
        
        component.is_damaged = true;
        assert!(component.overclock(1.2).is_err());
    }
    
    #[test]
    fn test_bottleneck_analysis() {
        let mut inventory = HardwareInventory::new();
        
        // Add unbalanced components
        let weak_cpu = HardwareComponent::new(HardwareType::CPU, HardwareQuality::Basic, 1000);
        let strong_ram = HardwareComponent::new(HardwareType::RAM, HardwareQuality::Elite, 16384);
        
        inventory.installed.insert(HardwareType::CPU, vec![weak_cpu]);
        inventory.installed.insert(HardwareType::RAM, vec![strong_ram]);
        
        let bottlenecks = analyze_bottlenecks(&inventory);
        assert!(bottlenecks.iter().any(|b| b.contains("CPU")));
    }
    
    #[test]
    fn test_upgrade_cost_calculation() {
        let config = HardwareConfig::default();
        let cost = calculate_upgrade_cost(2000, 4000, &HardwareType::CPU, &config);
        assert!(cost > 0);
        
        let no_cost = calculate_upgrade_cost(4000, 2000, &HardwareType::CPU, &config);
        assert_eq!(no_cost, 0);
    }
    
    #[test]
    fn test_compatibility_check() {
        let cpu = HardwareComponent::new(HardwareType::CPU, HardwareQuality::Quantum, 10000);
        let mobo = HardwareComponent::new(HardwareType::Motherboard, HardwareQuality::Basic, 100);
        
        let result = check_component_compatibility(&cpu, &mobo);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too advanced"));
    }
}