use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use he_core::events::Event;
use he_core::id::{EntityId, ServerId};

use crate::models::{Log, LogId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogCreatedEvent {
    pub log_id: LogId,
    pub server_id: ServerId,
    pub entity_id: EntityId,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

impl LogCreatedEvent {
    pub fn new(log: &Log) -> Self {
        Self {
            log_id: log.log_id,
            server_id: log.server_id,
            entity_id: log.entity_id,
            message: log.message.clone(),
            timestamp: log.creation_time,
        }
    }
}

impl Event for LogCreatedEvent {
    fn event_type(&self) -> &'static str {
        "log.created"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogModifiedEvent {
    pub log_id: LogId,
    pub server_id: ServerId,
    pub entity_id: EntityId,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

impl LogModifiedEvent {
    pub fn new(log: &Log) -> Self {
        Self {
            log_id: log.log_id,
            server_id: log.server_id,
            entity_id: log.entity_id,
            message: log.message.clone(),
            timestamp: log.creation_time,
        }
    }
}

impl Event for LogModifiedEvent {
    fn event_type(&self) -> &'static str {
        "log.modified"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogDeletedEvent {
    pub log_id: LogId,
    pub server_id: ServerId,
    pub entity_id: EntityId,
    pub timestamp: DateTime<Utc>,
}

impl LogDeletedEvent {
    pub fn new(log: &Log) -> Self {
        Self {
            log_id: log.log_id,
            server_id: log.server_id,
            entity_id: log.entity_id,
            timestamp: Utc::now(),
        }
    }
}

impl Event for LogDeletedEvent {
    fn event_type(&self) -> &'static str {
        "log.deleted"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}