//! Helix ID generation and management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IdError {
    #[error("Invalid ID format: {id}")]
    InvalidFormat { id: String },
    #[error("ID not found: {id}")]
    NotFound { id: String },
}

pub type IdResult<T> = Result<T, IdError>;

/// Heritage information for hierarchical IDs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heritage {
    pub grandparent: Option<Uuid>,
    pub parent: Option<Uuid>,
}

/// ID with heritage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelixId {
    pub id: Uuid,
    pub heritage: Heritage,
    pub id_type: IdType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdType {
    Account,
    Entity,
    Server,
    Process,
    Software,
    Notification,
    Log,
}

impl HelixId {
    pub fn new(id_type: IdType) -> Self {
        Self {
            id: Uuid::new_v4(),
            heritage: Heritage { grandparent: None, parent: None },
            id_type,
        }
    }

    pub fn with_heritage(id_type: IdType, grandparent: Option<Uuid>, parent: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            heritage: Heritage { grandparent, parent },
            id_type,
        }
    }
}

/// ID generation utilities
pub mod generator {
    use super::*;

    pub fn account_id() -> HelixId {
        HelixId::new(IdType::Account)
    }

    pub fn entity_id(account_id: Uuid) -> HelixId {
        HelixId::with_heritage(IdType::Entity, None, Some(account_id))
    }

    pub fn server_id(entity_id: Uuid, account_id: Uuid) -> HelixId {
        HelixId::with_heritage(IdType::Server, Some(account_id), Some(entity_id))
    }
}