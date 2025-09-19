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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_type_possible_types() {
        let types = EntityType::possible_types();
        assert_eq!(types.len(), 3);
        assert!(types.contains(&EntityType::Account));
        assert!(types.contains(&EntityType::Npc));
        assert!(types.contains(&EntityType::Clan));
    }

    #[test]
    fn test_entity_type_serialization() {
        let entity_type = EntityType::Account;
        let json = serde_json::to_string(&entity_type).unwrap();
        let deserialized: EntityType = serde_json::from_str(&json).unwrap();
        assert_eq!(entity_type, deserialized);

        // Test all variants
        for variant in EntityType::possible_types() {
            let json = serde_json::to_string(&variant).unwrap();
            let deserialized: EntityType = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    // Test implementation of EntitySpecialization trait
    struct TestEntity {
        id: EntityId,
        entity_type: EntityType,
    }

    impl EntitySpecialization for TestEntity {
        fn get_entity_id(&self) -> EntityId {
            self.id
        }

        fn get_entity_type(&self) -> EntityType {
            self.entity_type
        }
    }

    #[test]
    fn test_entity_specialization_trait() {
        use uuid::Uuid;

        let test_entity = TestEntity {
            id: EntityId(Uuid::new_v4()),
            entity_type: EntityType::Account,
        };

        assert_eq!(test_entity.get_entity_type(), EntityType::Account);
        assert_eq!(test_entity.get_entity_id(), test_entity.id);
    }
}