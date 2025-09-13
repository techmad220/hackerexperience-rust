//! Core entity types shared across the Helix system

use serde::{Deserialize, Serialize};
use crate::id::EntityId;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "entity_type", rename_all = "lowercase")]
pub enum EntityType {
    Account,
    Npc,
    Clan,
}

impl EntityType {
    pub fn possible_types() -> Vec<EntityType> {
        vec![EntityType::Account, EntityType::Npc, EntityType::Clan]
    }
}

/// Trait for objects that can be converted to entities
pub trait EntitySpecialization {
    fn get_entity_id(&self) -> EntityId;
    fn get_entity_type(&self) -> EntityType;
}