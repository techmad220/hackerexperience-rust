//! Complete Hardware Engine - Performance calculations and component management

use super::{EngineComponent, EngineError, EngineResult, ComponentStatus, Resources};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Hardware component types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentType {
    CPU,
    RAM,
    HDD,
    NetworkCard,
    ExternalHDD,
    USBDevice,
}

/// Individual hardware component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareComponent {
    pub id: Uuid,
    pub component_type: ComponentType,
    pub model: String,
    pub manufacturer: String,
    pub capacity: f32,
    pub speed: f32,
    pub power_consumption: f32,
    pub health: f32,  // 0.0 to 100.0
    pub price: u32,
    pub slots_required: u32,
}

impl HardwareComponent {
    pub fn cpu(model: String, speed_mhz: f32, price: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            component_type: ComponentType::CPU,
            model,
            manufacturer: "Intel".to_string(),
            capacity: 1.0,
            speed: speed_mhz,
            power_consumption: speed_mhz / 10.0,
            health: 100.0,
            price,
            slots_required: 1,
        }
    }

    pub fn ram(model: String, capacity_mb: f32, speed_mhz: f32, price: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            component_type: ComponentType::RAM,
            model,
            manufacturer: "Kingston".to_string(),
            capacity: capacity_mb,
            speed: speed_mhz,
            power_consumption: 5.0,
            health: 100.0,
            price,
            slots_required: 1,
        }
    }

    pub fn hdd(model: String, capacity_gb: f32, speed_rpm: f32, price: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            component_type: ComponentType::HDD,
            model,
            manufacturer: "Seagate".to_string(),
            capacity: capacity_gb * 1024.0,  // Convert to MB
            speed: speed_rpm,
            power_consumption: 10.0,
            health: 100.0,
            price,
            slots_required: 1,
        }
    }

    pub fn network(model: String, speed_mbps: f32, price: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            component_type: ComponentType::NetworkCard,
            model,
            manufacturer: "Realtek".to_string(),
            capacity: 1.0,
            speed: speed_mbps,
            power_consumption: 3.0,
            health: 100.0,
            price,
            slots_required: 1,
        }
    }

    pub fn effective_performance(&self) -> f32 {
        self.speed * (self.health / 100.0)
    }
}

/// Complete hardware configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareConfiguration {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub components: HashMap<Uuid, HardwareComponent>,
    pub max_slots: u32,
    pub used_slots: u32,
    pub total_power: f32,
    pub performance_score: f32,
}

impl HardwareConfiguration {
    pub fn new(owner_id: Uuid, name: String, max_slots: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            owner_id,
            name,
            components: HashMap::new(),
            max_slots,
            used_slots: 0,
            total_power: 0.0,
            performance_score: 0.0,
        }
    }

    pub fn add_component(&mut self, component: HardwareComponent) -> EngineResult<()> {
        if self.used_slots + component.slots_required > self.max_slots {
            return Err(EngineError::ResourceExhausted("No available slots".into()));
        }

        self.used_slots += component.slots_required;
        self.total_power += component.power_consumption;
        self.components.insert(component.id, component);
        self.recalculate_performance();
        Ok(())
    }

    pub fn remove_component(&mut self, component_id: Uuid) -> EngineResult<HardwareComponent> {
        match self.components.remove(&component_id) {
            Some(component) => {
                self.used_slots -= component.slots_required;
                self.total_power -= component.power_consumption;
                self.recalculate_performance();
                Ok(component)
            }
            None => Err(EngineError::NotFound("Component not found".into())),
        }
    }

    pub fn get_resources(&self) -> Resources {
        let cpu = self.get_total_cpu();
        let ram = self.get_total_ram();
        let disk = self.get_total_disk();
        let network = self.get_total_network();
        Resources::new(cpu, ram, disk, network)
    }

    pub fn get_total_cpu(&self) -> f32 {
        self.components.values()
            .filter(|c| c.component_type == ComponentType::CPU)
            .map(|c| c.effective_performance())
            .sum()
    }

    pub fn get_total_ram(&self) -> f32 {
        self.components.values()
            .filter(|c| c.component_type == ComponentType::RAM)
            .map(|c| c.capacity * (c.health / 100.0))
            .sum()
    }

    pub fn get_total_disk(&self) -> f32 {
        self.components.values()
            .filter(|c| matches!(c.component_type, ComponentType::HDD | ComponentType::ExternalHDD))
            .map(|c| c.capacity * (c.health / 100.0))
            .sum()
    }

    pub fn get_total_network(&self) -> f32 {
        self.components.values()
            .filter(|c| c.component_type == ComponentType::NetworkCard)
            .map(|c| c.effective_performance())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    fn recalculate_performance(&mut self) {
        // Performance score formula (similar to original game)
        let cpu = self.get_total_cpu();
        let ram = self.get_total_ram();
        let hdd = self.get_total_disk();
        let net = self.get_total_network();

        self.performance_score =
            (cpu * 0.35) +
            (ram * 0.25) +
            (hdd * 0.20) +
            (net * 0.20);
    }
}

/// Hardware Calculator - Performs complex calculations
pub struct HardwareCalculator;

impl HardwareCalculator {
    /// Calculate process time based on hardware
    pub fn calculate_process_time(
        process_complexity: f32,
        hardware: &HardwareConfiguration,
    ) -> Duration {
        let performance = hardware.performance_score.max(1.0);
        let base_time = 60.0; // Base time in seconds
        let time = (base_time * process_complexity) / (performance / 100.0);
        Duration::from_secs_f32(time)
    }

    /// Calculate power consumption
    pub fn calculate_power_cost(hardware: &HardwareConfiguration, hours: f32) -> f32 {
        let kwh = (hardware.total_power / 1000.0) * hours;
        kwh * 0.12  // Average cost per kWh
    }

    /// Calculate upgrade cost
    pub fn calculate_upgrade_cost(
        old_component: &HardwareComponent,
        new_component: &HardwareComponent,
    ) -> u32 {
        let depreciation = (old_component.price as f32 * (old_component.health / 100.0)) as u32;
        new_component.price.saturating_sub(depreciation / 2)
    }

    /// Calculate bottleneck component
    pub fn find_bottleneck(hardware: &HardwareConfiguration) -> Option<ComponentType> {
        let cpu = hardware.get_total_cpu();
        let ram = hardware.get_total_ram();
        let disk = hardware.get_total_disk();
        let net = hardware.get_total_network();

        let mut scores = vec![
            (ComponentType::CPU, cpu),
            (ComponentType::RAM, ram),
            (ComponentType::HDD, disk),
            (ComponentType::NetworkCard, net),
        ];

        scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        if scores[0].1 < scores[3].1 * 0.5 {
            Some(scores[0].0)
        } else {
            None
        }
    }
}

/// Main Hardware Engine
pub struct HardwareEngine {
    configurations: HashMap<Uuid, HardwareConfiguration>,
    component_catalog: HashMap<String, HardwareComponent>,
    calculator: HardwareCalculator,
    last_update: SystemTime,
}

impl HardwareEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            configurations: HashMap::new(),
            component_catalog: HashMap::new(),
            calculator: HardwareCalculator,
            last_update: SystemTime::now(),
        };
        engine.initialize_catalog();
        engine
    }

    fn initialize_catalog(&mut self) {
        // CPU catalog
        self.add_to_catalog(HardwareComponent::cpu("Pentium III".to_string(), 500.0, 50));
        self.add_to_catalog(HardwareComponent::cpu("Pentium 4".to_string(), 1500.0, 150));
        self.add_to_catalog(HardwareComponent::cpu("Core 2 Duo".to_string(), 2400.0, 300));
        self.add_to_catalog(HardwareComponent::cpu("Core i3".to_string(), 3200.0, 500));
        self.add_to_catalog(HardwareComponent::cpu("Core i5".to_string(), 4500.0, 800));
        self.add_to_catalog(HardwareComponent::cpu("Core i7".to_string(), 5800.0, 1200));
        self.add_to_catalog(HardwareComponent::cpu("Core i9".to_string(), 8000.0, 2000));

        // RAM catalog
        self.add_to_catalog(HardwareComponent::ram("DDR 256MB".to_string(), 256.0, 266.0, 20));
        self.add_to_catalog(HardwareComponent::ram("DDR 512MB".to_string(), 512.0, 333.0, 40));
        self.add_to_catalog(HardwareComponent::ram("DDR2 1GB".to_string(), 1024.0, 667.0, 80));
        self.add_to_catalog(HardwareComponent::ram("DDR3 2GB".to_string(), 2048.0, 1333.0, 150));
        self.add_to_catalog(HardwareComponent::ram("DDR3 4GB".to_string(), 4096.0, 1600.0, 250));
        self.add_to_catalog(HardwareComponent::ram("DDR4 8GB".to_string(), 8192.0, 2400.0, 400));
        self.add_to_catalog(HardwareComponent::ram("DDR4 16GB".to_string(), 16384.0, 3200.0, 700));

        // HDD catalog
        self.add_to_catalog(HardwareComponent::hdd("10GB IDE".to_string(), 10.0, 5400.0, 30));
        self.add_to_catalog(HardwareComponent::hdd("40GB IDE".to_string(), 40.0, 7200.0, 60));
        self.add_to_catalog(HardwareComponent::hdd("120GB SATA".to_string(), 120.0, 7200.0, 100));
        self.add_to_catalog(HardwareComponent::hdd("250GB SATA".to_string(), 250.0, 7200.0, 150));
        self.add_to_catalog(HardwareComponent::hdd("500GB SATA".to_string(), 500.0, 7200.0, 200));
        self.add_to_catalog(HardwareComponent::hdd("1TB SATA".to_string(), 1000.0, 7200.0, 300));
        self.add_to_catalog(HardwareComponent::hdd("2TB SATA".to_string(), 2000.0, 7200.0, 500));

        // Network catalog
        self.add_to_catalog(HardwareComponent::network("10Mbps Ethernet".to_string(), 10.0, 20));
        self.add_to_catalog(HardwareComponent::network("100Mbps Fast Ethernet".to_string(), 100.0, 50));
        self.add_to_catalog(HardwareComponent::network("1Gbps Gigabit".to_string(), 1000.0, 100));
        self.add_to_catalog(HardwareComponent::network("10Gbps 10-Gigabit".to_string(), 10000.0, 500));
    }

    fn add_to_catalog(&mut self, component: HardwareComponent) {
        self.component_catalog.insert(component.model.clone(), component);
    }

    pub fn create_configuration(&mut self, owner_id: Uuid, name: String) -> Uuid {
        let config = HardwareConfiguration::new(owner_id, name, 10);
        let id = config.id;
        self.configurations.insert(id, config);
        id
    }

    pub fn get_configuration(&self, id: Uuid) -> Option<&HardwareConfiguration> {
        self.configurations.get(&id)
    }

    pub fn get_configuration_mut(&mut self, id: Uuid) -> Option<&mut HardwareConfiguration> {
        self.configurations.get_mut(&id)
    }

    pub fn purchase_component(&mut self, config_id: Uuid, model: &str) -> EngineResult<()> {
        let component = self.component_catalog.get(model)
            .ok_or_else(|| EngineError::NotFound("Component not in catalog".into()))?
            .clone();

        let config = self.configurations.get_mut(&config_id)
            .ok_or_else(|| EngineError::NotFound("Configuration not found".into()))?;

        config.add_component(component)
    }

    pub fn calculate_process_time(&self, complexity: f32, config_id: Uuid) -> EngineResult<Duration> {
        let config = self.configurations.get(&config_id)
            .ok_or_else(|| EngineError::NotFound("Configuration not found".into()))?;

        Ok(HardwareCalculator::calculate_process_time(complexity, config))
    }

    pub fn get_catalog(&self) -> Vec<&HardwareComponent> {
        self.component_catalog.values().collect()
    }
}

impl EngineComponent for HardwareEngine {
    fn initialize(&mut self) -> EngineResult<()> {
        Ok(())
    }

    fn update(&mut self, _delta: Duration) -> EngineResult<()> {
        // Degrade hardware health over time
        for config in self.configurations.values_mut() {
            for component in config.components.values_mut() {
                // Degrade by 0.01% per update
                component.health = (component.health - 0.01).max(0.0);
            }
            config.recalculate_performance();
        }
        self.last_update = SystemTime::now();
        Ok(())
    }

    fn status(&self) -> ComponentStatus {
        ComponentStatus {
            name: "HardwareEngine".to_string(),
            healthy: true,
            last_update: self.last_update,
            metrics: vec![
                ("configurations".to_string(), self.configurations.len() as f64),
                ("catalog_items".to_string(), self.component_catalog.len() as f64),
            ],
        }
    }

    fn reset(&mut self) -> EngineResult<()> {
        self.configurations.clear();
        self.initialize_catalog();
        self.last_update = SystemTime::now();
        Ok(())
    }
}