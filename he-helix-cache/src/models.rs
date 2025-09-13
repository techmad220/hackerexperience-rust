use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;

use he_core::id::{NetworkId, ServerId, StorageId, ComponentId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheNetworkInfo {
    pub network_id: NetworkId,
    pub ip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ServerCache {
    pub server_id: ServerId,
    pub networks: Vec<CacheNetworkInfo>,
    pub storages: Vec<StorageId>,
    pub expiration_date: DateTime<Utc>,
}

impl ServerCache {
    pub fn new(server_id: ServerId, networks: Vec<CacheNetworkInfo>, storages: Vec<StorageId>) -> Self {
        let expiration_date = Utc::now() + Duration::days(1); // 24 hours cache duration
        
        Self {
            server_id,
            networks,
            storages,
            expiration_date,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expiration_date
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StorageCache {
    pub storage_id: StorageId,
    pub server_id: ServerId,
    pub expiration_date: DateTime<Utc>,
}

impl StorageCache {
    pub fn new(storage_id: StorageId, server_id: ServerId) -> Self {
        let expiration_date = Utc::now() + Duration::days(1);
        
        Self {
            storage_id,
            server_id,
            expiration_date,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expiration_date
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NetworkCache {
    pub network_id: NetworkId,
    pub ip: String,
    pub server_id: Option<ServerId>,
    pub expiration_date: DateTime<Utc>,
}

impl NetworkCache {
    pub fn new(network_id: NetworkId, ip: String, server_id: Option<ServerId>) -> Self {
        let expiration_date = Utc::now() + Duration::days(1);
        
        Self {
            network_id,
            ip,
            server_id,
            expiration_date,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expiration_date
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebCache {
    pub network_id: NetworkId,
    pub ip: String,
    pub server_id: Option<ServerId>,
    pub expiration_date: DateTime<Utc>,
}

impl WebCache {
    pub fn new(network_id: NetworkId, ip: String, server_id: Option<ServerId>) -> Self {
        let expiration_date = Utc::now() + Duration::days(1);
        
        Self {
            network_id,
            ip,
            server_id,
            expiration_date,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expiration_date
    }
}

#[derive(Debug, Clone)]
pub enum CacheKey {
    Server(ServerId),
    Storage(StorageId),
    Network(NetworkId, String),
    Web(NetworkId, String),
}

impl CacheKey {
    pub fn to_string(&self) -> String {
        match self {
            CacheKey::Server(id) => format!("server:{}", id),
            CacheKey::Storage(id) => format!("storage:{}", id),
            CacheKey::Network(network_id, ip) => format!("network:{}:{}", network_id, ip),
            CacheKey::Web(network_id, ip) => format!("web:{}:{}", network_id, ip),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CacheValue {
    Server(ServerCache),
    Storage(StorageCache),
    Network(NetworkCache),
    Web(WebCache),
}

#[derive(Debug, Clone)]
pub enum CacheResult<T> {
    Hit(T),
    Miss,
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Entry not found")]
    NotFound,
}