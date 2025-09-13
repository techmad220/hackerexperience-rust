use anyhow::Result;
use redis::AsyncCommands;
use std::sync::Arc;
use tracing::{info, warn, error};

use he_core::id::{NetworkId, ServerId, StorageId, ComponentId};

use crate::models::{
    CacheError, CacheKey, CacheResult, CacheValue, NetworkCache, ServerCache, StorageCache, WebCache,
};
use crate::queries::CacheQueries;

/// The Cache Action module provides the main API for cache operations.
/// 
/// Key principles:
/// - **Update** data when the underlying object has been modified
/// - **Purge** data when the underlying object has been deleted
/// - Updates will create related entries and handle cascading updates
/// - Purges will remove the entry but won't cascade to related objects
pub struct CacheActions {
    queries: CacheQueries,
    redis_pool: redis::aio::ConnectionManager,
}

impl CacheActions {
    pub async fn new(
        queries: CacheQueries, 
        redis_url: &str
    ) -> Result<Self, CacheError> {
        let client = redis::Client::open(redis_url)?;
        let redis_pool = redis::aio::ConnectionManager::new(client).await?;
        
        Ok(Self {
            queries,
            redis_pool,
        })
    }

    /// Purges the server entry from the cache.
    /// 
    /// If there is a motherboard attached to it, it will purge all related
    /// objects as well (components, storages and networks).
    pub async fn purge_server(&mut self, server_id: ServerId) -> Result<()> {
        // First, get the server cache to find related entries
        if let Ok(Some(server_cache)) = self.queries.get_server_cache(server_id).await {
            // Purge related storages
            for storage_id in &server_cache.storages {
                self.purge_storage_internal(*storage_id).await?;
            }

            // Purge related networks
            for network_info in &server_cache.networks {
                self.purge_network_internal(network_info.network_id, &network_info.ip).await?;
            }
        }

        // Purge the server itself
        self.purge_server_internal(server_id).await?;
        Ok(())
    }

    /// Updates a server entry.
    /// 
    /// If the server has a motherboard attached to it, it will update all related
    /// components as well (components, storages and networks).
    pub async fn update_server(&mut self, server_id: ServerId) -> Result<()> {
        // Fetch fresh server data from the database
        if let Some(server_data) = self.queries.fetch_server_data(server_id).await? {
            // Update related storages
            for storage_id in &server_data.storages {
                self.update_storage_internal(*storage_id).await?;
            }

            // Update related networks
            for network_info in &server_data.networks {
                self.update_network_internal(network_info.network_id, &network_info.ip).await?;
            }

            // Update the server cache
            self.update_server_internal(server_id).await?;
        }

        Ok(())
    }

    /// Given a storage, update its corresponding server.
    pub async fn update_server_by_storage(&mut self, storage_id: StorageId) -> Result<()> {
        if let Some(server_id) = self.queries.get_server_by_storage(storage_id).await? {
            self.update_server(server_id).await?;
        }
        Ok(())
    }

    /// Given a Network IP (NIP), update its corresponding server.
    pub async fn update_server_by_nip(&mut self, network_id: NetworkId, ip: &str) -> Result<()> {
        if let Some(server_id) = self.queries.get_server_by_nip(network_id, ip).await? {
            self.update_server(server_id).await?;
        }
        Ok(())
    }

    /// Given a component, locate its server and update it.
    pub async fn update_server_by_component(&mut self, component_id: ComponentId) -> Result<()> {
        if let Some(server_id) = self.queries.get_server_by_component(component_id).await? {
            self.update_server(server_id).await?;
        }
        Ok(())
    }

    /// Updates a storage entry from the cache.
    /// It will also update the underlying server.
    pub async fn update_storage(&mut self, storage_id: StorageId) -> Result<()> {
        if let Some(server_id) = self.queries.get_server_by_storage(storage_id).await? {
            self.update_server(server_id).await?;
        }
        self.update_storage_internal(storage_id).await?;
        Ok(())
    }

    /// Purges a storage entry.
    /// It does not purge/update the server.
    pub async fn purge_storage(&mut self, storage_id: StorageId) -> Result<()> {
        self.purge_storage_internal(storage_id).await
    }

    /// Updates the network/IP entry on the cache.
    /// It will also update the underlying server.
    pub async fn update_network(&mut self, network_id: NetworkId, ip: &str) -> Result<()> {
        if let Some(server_id) = self.queries.get_server_by_nip(network_id, ip).await? {
            self.update_server(server_id).await?;
        }
        self.update_network_internal(network_id, ip).await?;
        Ok(())
    }

    /// Purges the network/IP entry from the cache.
    /// It does not purge/update the server.
    pub async fn purge_network(&mut self, network_id: NetworkId, ip: &str) -> Result<()> {
        self.purge_network_internal(network_id, ip).await
    }

    /// Updates web cache entry.
    pub async fn update_web(&mut self, network_id: NetworkId, ip: &str) -> Result<()> {
        self.update_web_internal(network_id, ip).await
    }

    /// Purges web cache entry.
    pub async fn purge_web(&mut self, network_id: NetworkId, ip: &str) -> Result<()> {
        self.purge_web_internal(network_id, ip).await
    }

    // Internal methods for direct cache operations

    async fn update_server_internal(&mut self, server_id: ServerId) -> Result<()> {
        if let Some(server_data) = self.queries.fetch_server_data(server_id).await? {
            let cache_data = ServerCache::new(server_id, server_data.networks, server_data.storages);
            self.set_cache(CacheKey::Server(server_id), CacheValue::Server(cache_data)).await?;
        }
        Ok(())
    }

    async fn purge_server_internal(&mut self, server_id: ServerId) -> Result<()> {
        self.delete_cache(CacheKey::Server(server_id)).await
    }

    async fn update_storage_internal(&mut self, storage_id: StorageId) -> Result<()> {
        if let Some(server_id) = self.queries.get_server_by_storage(storage_id).await? {
            let cache_data = StorageCache::new(storage_id, server_id);
            self.set_cache(CacheKey::Storage(storage_id), CacheValue::Storage(cache_data)).await?;
        }
        Ok(())
    }

    async fn purge_storage_internal(&mut self, storage_id: StorageId) -> Result<()> {
        self.delete_cache(CacheKey::Storage(storage_id)).await
    }

    async fn update_network_internal(&mut self, network_id: NetworkId, ip: &str) -> Result<()> {
        let server_id = self.queries.get_server_by_nip(network_id, ip).await?;
        let cache_data = NetworkCache::new(network_id, ip.to_string(), server_id);
        self.set_cache(
            CacheKey::Network(network_id, ip.to_string()), 
            CacheValue::Network(cache_data)
        ).await?;
        Ok(())
    }

    async fn purge_network_internal(&mut self, network_id: NetworkId, ip: &str) -> Result<()> {
        self.delete_cache(CacheKey::Network(network_id, ip.to_string())).await
    }

    async fn update_web_internal(&mut self, network_id: NetworkId, ip: &str) -> Result<()> {
        let server_id = self.queries.get_server_by_nip(network_id, ip).await?;
        let cache_data = WebCache::new(network_id, ip.to_string(), server_id);
        self.set_cache(
            CacheKey::Web(network_id, ip.to_string()), 
            CacheValue::Web(cache_data)
        ).await?;
        Ok(())
    }

    async fn purge_web_internal(&mut self, network_id: NetworkId, ip: &str) -> Result<()> {
        self.delete_cache(CacheKey::Web(network_id, ip.to_string())).await
    }

    // Direct cache operations

    async fn set_cache(&mut self, key: CacheKey, value: CacheValue) -> Result<()> {
        let key_str = key.to_string();
        let value_json = serde_json::to_string(&value)?;
        
        let mut conn = self.redis_pool.clone();
        conn.set(&key_str, &value_json).await?;
        
        // Set expiration (24 hours)
        conn.expire(&key_str, 86400).await?;
        
        Ok(())
    }

    async fn delete_cache(&mut self, key: CacheKey) -> Result<()> {
        let key_str = key.to_string();
        let mut conn = self.redis_pool.clone();
        conn.del(&key_str).await?;
        Ok(())
    }

    /// Direct cache query that doesn't populate cache on miss
    pub async fn direct_query<T>(&mut self, key: CacheKey) -> Result<CacheResult<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let key_str = key.to_string();
        let mut conn = self.redis_pool.clone();
        
        match conn.get::<_, Option<String>>(&key_str).await? {
            Some(value_json) => {
                match serde_json::from_str::<T>(&value_json) {
                    Ok(value) => Ok(CacheResult::Hit(value)),
                    Err(e) => {
                        error!("Failed to deserialize cache value for key {}: {}", key_str, e);
                        Ok(CacheResult::Miss)
                    }
                }
            }
            None => Ok(CacheResult::Miss),
        }
    }
}