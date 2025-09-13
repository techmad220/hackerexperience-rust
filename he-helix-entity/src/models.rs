use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use he_core::id::{ComponentId, EntityId, ServerId};
use he_core::{EntityType, EntitySpecialization};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Entity {
    pub entity_id: EntityId,
    pub entity_type: EntityType,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EntityComponent {
    pub entity_component_id: Uuid,
    pub entity_id: EntityId,
    pub component_id: ComponentId,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EntityServer {
    pub entity_server_id: Uuid,
    pub entity_id: EntityId,
    pub server_id: ServerId,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Database {
    pub database_id: Uuid,
    pub entity_id: EntityId,
    pub server_id: ServerId,
    pub hacked_by_id: Option<EntityId>,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CreateEntityParams {
    pub entity_id: EntityId,
    pub entity_type: EntityType,
}

#[derive(Debug, Clone)]
pub struct LinkComponentParams {
    pub entity_id: EntityId,
    pub component_id: ComponentId,
}

#[derive(Debug, Clone)]
pub struct LinkServerParams {
    pub entity_id: EntityId,
    pub server_id: ServerId,
}

#[derive(Debug, Clone)]
pub struct CreateDatabaseParams {
    pub entity_id: EntityId,
    pub server_id: ServerId,
    pub hacked_by_id: Option<EntityId>,
}

