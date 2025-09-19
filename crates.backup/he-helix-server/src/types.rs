//! Type definitions for the server module

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Server unique identifier
pub type ServerId = Uuid;

/// Component unique identifier  
pub type ComponentId = Uuid;

/// Motherboard identifier (same as ComponentId)
pub type MotherboardId = ComponentId;

/// Entity identifier (from entity service)
pub type EntityId = Uuid;

/// Server hostname/name
pub type Hostname = String;

/// Server password
pub type Password = String;

/// Server type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerType {
    /// Player's personal computer
    Desktop,
    /// NPCs (Non-Player Characters) servers  
    Npc,
    /// Story mission servers
    Story,
}

impl ServerType {
    pub fn possible_types() -> &'static [ServerType] {
        &[ServerType::Desktop, ServerType::Npc, ServerType::Story]
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            ServerType::Desktop => "desktop",
            ServerType::Npc => "npc", 
            ServerType::Story => "story",
        }
    }
}

impl std::fmt::Display for ServerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Component type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentType {
    /// Central Processing Unit
    Cpu,
    /// Random Access Memory
    Ram,
    /// Hard Disk Drive
    Hdd,
    /// Network Interface Card
    Nic,
    /// Motherboard (connects all components)
    Mobo,
}

impl ComponentType {
    pub fn all_types() -> &'static [ComponentType] {
        &[
            ComponentType::Cpu,
            ComponentType::Ram, 
            ComponentType::Hdd,
            ComponentType::Nic,
            ComponentType::Mobo,
        ]
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            ComponentType::Cpu => "cpu",
            ComponentType::Ram => "ram",
            ComponentType::Hdd => "hdd", 
            ComponentType::Nic => "nic",
            ComponentType::Mobo => "mobo",
        }
    }
    
    /// Check if this component type is pluggable into a motherboard
    pub fn is_pluggable(&self) -> bool {
        matches!(self, ComponentType::Cpu | ComponentType::Ram | ComponentType::Hdd | ComponentType::Nic)
    }
}

impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Motherboard slot identifier
pub type SlotId = u8;

/// Component slot on motherboard
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Slot {
    pub slot_id: SlotId,
    pub component_type: ComponentType,
    pub component_id: Option<ComponentId>,
}

/// Free slots available on motherboard
pub type FreeSlots = std::collections::HashMap<ComponentType, Vec<SlotId>>;

/// Resource allocation limits
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu: u32,
    pub ram: u64,
    pub hdd: u64,
    pub net: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu: 0,
            ram: 0,
            hdd: 0,
            net: 0,
        }
    }
}

/// Custom component specifications
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentSpecs {
    pub component_type: ComponentType,
    pub spec_value: u32,
}

/// Process priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProcessPriority {
    Low = 1,
    Normal = 5, 
    High = 10,
}

impl Default for ProcessPriority {
    fn default() -> Self {
        ProcessPriority::Normal
    }
}

/// Creation parameters for new servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCreationParams {
    pub server_type: ServerType,
    pub motherboard_id: Option<MotherboardId>,
    pub hostname: Option<Hostname>,
}

/// Update parameters for existing servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerUpdateParams {
    pub motherboard_id: Option<MotherboardId>,
    pub hostname: Option<Hostname>,
}

/// Component creation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentCreationParams {
    pub component_type: ComponentType,
    pub spec_value: u32,
}

/// Motherboard creation parameters with initial components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotherboardCreationParams {
    pub initial_components: Vec<(ComponentId, SlotId)>,
}