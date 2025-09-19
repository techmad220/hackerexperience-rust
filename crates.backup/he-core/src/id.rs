//! ID types re-exported from he-helix-core

pub use he_helix_core::types::{EntityId, AccountId, ServerId, NetworkId, ProcessId, RequestId, HelixId};

// Component ID type - define it here since it's not in he-helix-core
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Component identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(pub Uuid);

impl ComponentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ComponentId {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_id_creation() {
        let id1 = ComponentId::new();
        let id2 = ComponentId::new();

        // Each new ID should be unique
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_component_id_default() {
        let id1 = ComponentId::default();
        let id2 = ComponentId::default();

        // Default should create unique IDs
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_component_id_equality() {
        let uuid = Uuid::new_v4();
        let id1 = ComponentId(uuid);
        let id2 = ComponentId(uuid);

        // Same UUID should produce equal ComponentIds
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_component_id_serialization() {
        let id = ComponentId::new();
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: ComponentId = serde_json::from_str(&json).unwrap();

        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_component_id_hash() {
        use std::collections::HashSet;

        let id1 = ComponentId::new();
        let id2 = ComponentId::new();

        let mut set = HashSet::new();
        set.insert(id1);
        set.insert(id2);

        assert_eq!(set.len(), 2);
        assert!(set.contains(&id1));
        assert!(set.contains(&id2));
    }
}