use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use he_core::id::{EntityId, ServerId};

pub type LogId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Log {
    pub log_id: LogId,
    pub server_id: ServerId,
    pub entity_id: EntityId,
    pub message: String,
    pub crypto_version: Option<i32>,
    pub creation_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Revision {
    pub revision_id: Uuid,
    pub log_id: LogId,
    pub entity_id: EntityId,
    pub message: String,
    pub forge_version: i32,
    pub creation_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LogTouch {
    pub log_touch_id: Uuid,
    pub log_id: LogId,
    pub entity_id: EntityId,
    pub creation_time: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CreateLogParams {
    pub server_id: ServerId,
    pub entity_id: EntityId,
    pub message: String,
    pub crypto_version: Option<i32>,
    pub forge_version: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct UpdateLogParams {
    pub message: Option<String>,
    pub crypto_version: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct ReviseLogParams {
    pub entity_id: EntityId,
    pub message: String,
    pub forge_version: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogIndex {
    pub log_id: LogId,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RenderedLogIndex {
    pub log_id: String,
    pub message: String,
    pub timestamp: String,
}