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