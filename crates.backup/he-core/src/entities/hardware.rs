use serde::{Deserialize, Serialize};
use crate::{UserId, HardwareId, HeResult};

// Mapping from PHP HardwareVPC class in PC.class.php
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hardware {
    pub id: HardwareId,
    pub user_id: UserId,
    pub ram: i32,
    pub cpu: i32,
    pub hdd: i32,
    pub net: i32,
    pub is_npc: bool,
}

impl Hardware {
    pub fn new(user_id: UserId, is_npc: bool) -> Self {
        // Default starting hardware values
        Self {
            id: 0, // Will be set by database
            user_id,
            ram: if is_npc { 512 } else { 256 },      // MB
            cpu: if is_npc { 1000 } else { 500 },     // MHz
            hdd: if is_npc { 10000 } else { 5000 },   // MB
            net: if is_npc { 10 } else { 5 },         // Mbps
            is_npc,
        }
    }
    
    pub fn total_power(&self) -> i64 {
        // Simple power calculation
        (self.ram + self.cpu + self.hdd + self.net) as i64
    }
    
    pub fn can_run_process(&self, cpu_required: i32, ram_required: i32) -> bool {
        self.cpu >= cpu_required && self.ram >= ram_required
    }
    
    pub fn upgrade_component(&mut self, component: HardwareComponent, value: i32) -> HeResult<()> {
        match component {
            HardwareComponent::Ram => self.ram += value,
            HardwareComponent::Cpu => self.cpu += value,
            HardwareComponent::Hdd => self.hdd += value,
            HardwareComponent::Net => self.net += value,
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HardwareComponent {
    Ram,
    Cpu,
    Hdd,
    Net,
}

// External HD (XHD) - separate storage device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalHardware {
    pub id: i64,
    pub user_id: UserId,
    pub size: i32,          // Storage size in MB
    pub used_space: i32,    // Currently used space
    pub is_connected: bool,
}

impl ExternalHardware {
    pub fn new(user_id: UserId, size: i32) -> Self {
        Self {
            id: 0,
            user_id,
            size,
            used_space: 0,
            is_connected: false,
        }
    }
    
    pub fn available_space(&self) -> i32 {
        self.size - self.used_space
    }
    
    pub fn can_store(&self, size_required: i32) -> bool {
        self.available_space() >= size_required
    }
    
    pub fn connect(&mut self) -> HeResult<()> {
        self.is_connected = true;
        Ok(())
    }
    
    pub fn disconnect(&mut self) -> HeResult<()> {
        self.is_connected = false;
        Ok(())
    }
}

// Hardware info aggregation - matching PHP getHardwareInfo method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub total_pcs: i32,
    pub total_cpu: i32,
    pub total_ram: i32,
    pub total_hdd: i32,
    pub network_speed: i32,
}

impl HardwareInfo {
    pub fn from_hardware_list(hardware_list: &[Hardware]) -> Self {
        let total_pcs = hardware_list.len() as i32;
        let total_cpu = hardware_list.iter().map(|h| h.cpu).sum();
        let total_ram = hardware_list.iter().map(|h| h.ram).sum();
        let total_hdd = hardware_list.iter().map(|h| h.hdd).sum();
        // Network is not summed, just take the best one
        let network_speed = hardware_list.iter().map(|h| h.net).max().unwrap_or(0);
        
        Self {
            total_pcs,
            total_cpu,
            total_ram,
            total_hdd,
            network_speed,
        }
    }
    
    pub fn total_power(&self) -> i64 {
        (self.total_cpu + self.total_ram + self.total_hdd + self.network_speed) as i64
    }
}