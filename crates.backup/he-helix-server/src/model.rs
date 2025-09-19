//! Server model definitions

use crate::types::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Server entity model
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Server {
    pub server_id: ServerId,
    pub server_type: ServerType,
    pub motherboard_id: Option<MotherboardId>,
    pub hostname: Hostname,
    pub password: Password,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Server {
    /// Create a new server instance
    pub fn new(
        server_id: ServerId,
        server_type: ServerType,
        hostname: Hostname,
        password: Password,
    ) -> Self {
        let now = Utc::now();
        Self {
            server_id,
            server_type,
            motherboard_id: None,
            hostname,
            password,
            created_at: now,
            updated_at: now,
        }
    }

    /// Attach a motherboard to this server
    pub fn attach_motherboard(&mut self, motherboard_id: MotherboardId) {
        self.motherboard_id = Some(motherboard_id);
        self.updated_at = Utc::now();
    }

    /// Detach the motherboard from this server
    pub fn detach_motherboard(&mut self) {
        self.motherboard_id = None;
        self.updated_at = Utc::now();
    }

    /// Set a new hostname for this server
    pub fn set_hostname(&mut self, hostname: Hostname) {
        self.hostname = hostname;
        self.updated_at = Utc::now();
    }

    /// Check if server has a motherboard attached
    pub fn has_motherboard(&self) -> bool {
        self.motherboard_id.is_some()
    }
}

/// Hardware component model
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Component {
    pub component_id: ComponentId,
    pub component_type: ComponentType,
    pub spec_value: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Component {
    /// Create a new component
    pub fn new(
        component_id: ComponentId,
        component_type: ComponentType,
        spec_value: u32,
    ) -> Self {
        let now = Utc::now();
        Self {
            component_id,
            component_type,
            spec_value,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if this component can be plugged into a motherboard
    pub fn is_pluggable(&self) -> bool {
        self.component_type.is_pluggable()
    }

    /// Get the display name for this component type
    pub fn type_name(&self) -> &'static str {
        self.component_type.as_str()
    }
}

/// Motherboard model that manages component connections
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Motherboard {
    pub motherboard_id: MotherboardId,
    pub slots: Vec<Slot>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Motherboard {
    /// Create a new motherboard with empty slots
    pub fn new(motherboard_id: MotherboardId, slot_count: u8) -> Self {
        let now = Utc::now();
        let mut slots = Vec::new();
        
        // Create slots for different component types
        let mut slot_id = 0u8;
        
        // CPU slots (usually 1)
        for _ in 0..1 {
            slots.push(Slot {
                slot_id,
                component_type: ComponentType::Cpu,
                component_id: None,
            });
            slot_id += 1;
        }
        
        // RAM slots (multiple)
        for _ in 0..(slot_count / 4) {
            slots.push(Slot {
                slot_id,
                component_type: ComponentType::Ram,
                component_id: None,
            });
            slot_id += 1;
        }
        
        // HDD slots (multiple)
        for _ in 0..(slot_count / 4) {
            slots.push(Slot {
                slot_id,
                component_type: ComponentType::Hdd,
                component_id: None,
            });
            slot_id += 1;
        }
        
        // NIC slots (multiple)  
        for _ in 0..(slot_count / 4) {
            slots.push(Slot {
                slot_id,
                component_type: ComponentType::Nic,
                component_id: None,
            });
            slot_id += 1;
        }

        Self {
            motherboard_id,
            slots,
            created_at: now,
            updated_at: now,
        }
    }

    /// Plug a component into an available slot
    pub fn plug_component(&mut self, component_id: ComponentId, component_type: ComponentType) -> Result<SlotId, String> {
        // Find an available slot for this component type
        if let Some(slot) = self.slots.iter_mut().find(|s| {
            s.component_type == component_type && s.component_id.is_none()
        }) {
            slot.component_id = Some(component_id);
            self.updated_at = Utc::now();
            Ok(slot.slot_id)
        } else {
            Err(format!("No available slots for component type: {}", component_type))
        }
    }

    /// Unplug a component from its slot
    pub fn unplug_component(&mut self, component_id: ComponentId) -> Result<SlotId, String> {
        if let Some(slot) = self.slots.iter_mut().find(|s| s.component_id == Some(component_id)) {
            slot.component_id = None;
            self.updated_at = Utc::now();
            Ok(slot.slot_id)
        } else {
            Err("Component not found in any slot".to_string())
        }
    }

    /// Get all free slots grouped by component type
    pub fn get_free_slots(&self) -> FreeSlots {
        let mut free_slots: FreeSlots = HashMap::new();
        
        for slot in &self.slots {
            if slot.component_id.is_none() {
                free_slots
                    .entry(slot.component_type)
                    .or_insert_with(Vec::new)
                    .push(slot.slot_id);
            }
        }
        
        free_slots
    }

    /// Get all occupied slots
    pub fn get_occupied_slots(&self) -> Vec<&Slot> {
        self.slots.iter().filter(|s| s.component_id.is_some()).collect()
    }

    /// Check if a specific component type has available slots
    pub fn has_free_slot(&self, component_type: ComponentType) -> bool {
        self.slots.iter().any(|s| {
            s.component_type == component_type && s.component_id.is_none()
        })
    }

    /// Get total slot count for a component type
    pub fn slot_count_for_type(&self, component_type: ComponentType) -> usize {
        self.slots.iter().filter(|s| s.component_type == component_type).count()
    }

    /// Get occupied slot count for a component type
    pub fn occupied_slot_count_for_type(&self, component_type: ComponentType) -> usize {
        self.slots.iter().filter(|s| {
            s.component_type == component_type && s.component_id.is_some()
        }).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_server_creation() {
        let server_id = Uuid::new_v4();
        let server = Server::new(
            server_id,
            ServerType::Desktop,
            "test-server".to_string(),
            "password123".to_string(),
        );

        assert_eq!(server.server_id, server_id);
        assert_eq!(server.server_type, ServerType::Desktop);
        assert_eq!(server.hostname, "test-server");
        assert!(!server.has_motherboard());
    }

    #[test]
    fn test_motherboard_component_management() {
        let motherboard_id = Uuid::new_v4();
        let mut motherboard = Motherboard::new(motherboard_id, 8);
        
        assert!(motherboard.has_free_slot(ComponentType::Cpu));
        assert!(motherboard.has_free_slot(ComponentType::Ram));
        
        let cpu_id = Uuid::new_v4();
        let result = motherboard.plug_component(cpu_id, ComponentType::Cpu);
        assert!(result.is_ok());
        
        let free_cpu_slots = motherboard.get_free_slots().get(&ComponentType::Cpu).cloned().unwrap_or_default();
        assert!(free_cpu_slots.is_empty()); // Should have no free CPU slots after plugging one
    }

    #[test]
    fn test_component_is_pluggable() {
        let cpu = Component::new(Uuid::new_v4(), ComponentType::Cpu, 100);
        assert!(cpu.is_pluggable());
        
        let mobo = Component::new(Uuid::new_v4(), ComponentType::Mobo, 8);
        assert!(!mobo.is_pluggable());
    }
}