//! Core types used throughout the Helix system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Universal unique identifier type used throughout Helix
pub type HelixId = Uuid;

/// Generic metadata map for storing arbitrary key-value pairs
pub type Meta = HashMap<String, serde_json::Value>;

/// Request identifier for tracking operations across the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequestId(pub Uuid);

impl RequestId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for RequestId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<RequestId> for Uuid {
    fn from(id: RequestId) -> Self {
        id.0
    }
}

/// Process identifier for tracking actor processes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProcessId(pub Uuid);

impl ProcessId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ProcessId {
    fn default() -> Self {
        Self::new()
    }
}

/// Entity identifier representing game entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub Uuid);

impl EntityId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl From<AccountId> for EntityId {
    fn from(account_id: AccountId) -> Self {
        Self(account_id.0)
    }
}

/// Server identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServerId(pub Uuid);

impl ServerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Account identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountId(pub Uuid);

impl AccountId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl From<EntityId> for AccountId {
    fn from(entity_id: EntityId) -> Self {
        Self(entity_id.0)
    }
}

/// Network identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkId(pub Uuid);

impl NetworkId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}