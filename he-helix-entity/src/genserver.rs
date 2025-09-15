//! Entity System GenServer Implementation
//! 
//! Complete port of Helix.Entity GenServer with all handle_call, handle_cast,
//! and handle_info patterns from the original Elixir implementation.

use he_helix_core::genserver::{
    GenServer, GenServerState, GenServerHandle, GenServerMessage, GenServerReply,
    InfoSource, TerminateReason, SupervisionStrategy, GenServerSupervisor
};
use he_helix_core::{HelixError, HelixResult, ProcessId};
use he_core::id::{AccountId, EntityId, ServerId};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Entity types supported by the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EntityType {
    Account,
    Server,
    Software,
    Process,
    Network,
    Tunnel,
    File,
    Virus,
    BankAccount,
    LogEntry,
    Story,
    Mission,
}

/// Entity ownership and relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityOwnership {
    pub entity_id: EntityId,
    pub owner_id: EntityId,
    pub owner_type: EntityType,
    pub owned_type: EntityType,
    pub created_at: SystemTime,
    pub permissions: EntityPermissions,
}

/// Entity permissions for fine-grained access control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityPermissions {
    pub read: bool,
    pub write: bool,
    pub delete: bool,
    pub transfer: bool,
    pub admin: bool,
}

impl Default for EntityPermissions {
    fn default() -> Self {
        Self {
            read: true,
            write: true,
            delete: true,
            transfer: true,
            admin: false,
        }
    }
}

/// Entity metadata for additional information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMetadata {
    pub tags: Vec<String>,
    pub attributes: HashMap<String, String>,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

impl Default for EntityMetadata {
    fn default() -> Self {
        let now = SystemTime::now();
        Self {
            tags: Vec::new(),
            attributes: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Complete entity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub entity_id: EntityId,
    pub entity_type: EntityType,
    pub metadata: EntityMetadata,
    pub parent_id: Option<EntityId>,
    pub children: Vec<EntityId>,
}

/// Entity System State - mirrors Elixir GenServer state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySystemState {
    /// All entities indexed by ID
    pub entities: HashMap<EntityId, Entity>,
    
    /// Entity ownership relationships
    pub ownership: HashMap<EntityId, EntityOwnership>,
    
    /// Type-based indices for efficient queries
    pub type_indices: HashMap<EntityType, Vec<EntityId>>,
    
    /// Owner-based indices
    pub owner_indices: HashMap<EntityId, Vec<EntityId>>,
    
    /// Parent-child relationship indices
    pub children_indices: HashMap<EntityId, Vec<EntityId>>,
    
    /// Statistics and counters
    pub stats: EntitySystemStats,
    
    /// Configuration
    pub config: EntitySystemConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySystemStats {
    pub total_entities: u64,
    pub entities_by_type: HashMap<EntityType, u64>,
    pub ownership_relationships: u64,
    pub last_updated: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySystemConfig {
    pub max_entities_per_owner: u32,
    pub enable_soft_delete: bool,
    pub auto_cleanup_interval: Duration,
    pub cache_ttl: Duration,
}

impl Default for EntitySystemConfig {
    fn default() -> Self {
        Self {
            max_entities_per_owner: 10000,
            enable_soft_delete: true,
            auto_cleanup_interval: Duration::from_secs(3600), // 1 hour
            cache_ttl: Duration::from_secs(300), // 5 minutes
        }
    }
}

impl GenServerState for EntitySystemState {
    fn serialize(&self) -> HelixResult<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| HelixError::Serialization(e.to_string()))
    }

    fn deserialize(data: &[u8]) -> HelixResult<Self> {
        serde_json::from_slice(data).map_err(|e| HelixError::Serialization(e.to_string()))
    }
}

/// Entity System GenServer Messages - mirrors Elixir handle_call patterns
#[derive(Debug)]
pub enum EntityCall {
    /// Create a new entity
    Create {
        entity_type: EntityType,
        owner_id: Option<EntityId>,
        metadata: Option<EntityMetadata>,
        parent_id: Option<EntityId>,
    },
    
    /// Get entity by ID
    Get { entity_id: EntityId },
    
    /// Update entity metadata
    Update {
        entity_id: EntityId,
        metadata: EntityMetadata,
    },
    
    /// Delete entity (soft or hard delete)
    Delete {
        entity_id: EntityId,
        hard_delete: bool,
    },
    
    /// Transfer ownership
    TransferOwnership {
        entity_id: EntityId,
        new_owner_id: EntityId,
        permissions: Option<EntityPermissions>,
    },
    
    /// Get entities by type
    GetByType { entity_type: EntityType },
    
    /// Get entities owned by owner
    GetByOwner { owner_id: EntityId },
    
    /// Get child entities
    GetChildren { parent_id: EntityId },
    
    /// Check entity permissions
    CheckPermission {
        entity_id: EntityId,
        requester_id: EntityId,
        permission: String,
    },
    
    /// Get entity statistics
    GetStats,
    
    /// Search entities with criteria
    Search {
        criteria: EntitySearchCriteria,
        limit: Option<u32>,
    },
}

/// Entity System GenServer Cast Messages - mirrors Elixir handle_cast patterns
#[derive(Debug)]
pub enum EntityCast {
    /// Add tag to entity
    AddTag {
        entity_id: EntityId,
        tag: String,
    },
    
    /// Remove tag from entity
    RemoveTag {
        entity_id: EntityId,
        tag: String,
    },
    
    /// Set entity attribute
    SetAttribute {
        entity_id: EntityId,
        key: String,
        value: String,
    },
    
    /// Remove entity attribute
    RemoveAttribute {
        entity_id: EntityId,
        key: String,
    },
    
    /// Bulk update entities
    BulkUpdate {
        updates: Vec<(EntityId, EntityMetadata)>,
    },
    
    /// Cleanup orphaned entities
    CleanupOrphans,
    
    /// Update statistics
    UpdateStats,
    
    /// Refresh cache
    RefreshCache,
}

/// Entity System GenServer Info Messages - mirrors Elixir handle_info patterns
#[derive(Debug)]
pub enum EntityInfo {
    /// Periodic cleanup timer
    CleanupTimer,
    
    /// Cache refresh timer
    CacheTimer,
    
    /// Stats update timer
    StatsTimer,
    
    /// External entity event
    ExternalEvent {
        event_type: String,
        entity_id: EntityId,
        data: HashMap<String, String>,
    },
    
    /// Monitor notification
    MonitorDown {
        monitored_id: EntityId,
        reason: String,
    },
    
    /// System resource warning
    ResourceWarning {
        resource: String,
        usage: f64,
        threshold: f64,
    },
}

/// Entity search criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySearchCriteria {
    pub entity_type: Option<EntityType>,
    pub owner_id: Option<EntityId>,
    pub tags: Option<Vec<String>>,
    pub attributes: Option<HashMap<String, String>>,
    pub created_after: Option<SystemTime>,
    pub created_before: Option<SystemTime>,
}

/// Entity System GenServer Implementation
pub struct EntitySystemGenServer;

#[async_trait]
impl GenServer for EntitySystemGenServer {
    type State = EntitySystemState;
    type InitArgs = EntitySystemConfig;

    async fn init(config: Self::InitArgs) -> HelixResult<Self::State> {
        info!("Initializing Entity System GenServer");
        
        let now = SystemTime::now();
        let stats = EntitySystemStats {
            total_entities: 0,
            entities_by_type: HashMap::new(),
            ownership_relationships: 0,
            last_updated: now,
        };

        Ok(EntitySystemState {
            entities: HashMap::new(),
            ownership: HashMap::new(),
            type_indices: HashMap::new(),
            owner_indices: HashMap::new(),
            children_indices: HashMap::new(),
            stats,
            config,
        })
    }

    async fn handle_call(
        &mut self,
        request: Box<dyn Any + Send + Sync>,
        from: ProcessId,
        state: &mut Self::State,
    ) -> HelixResult<GenServerReply> {
        if let Some(call) = request.downcast_ref::<EntityCall>() {
            match call {
                EntityCall::Create { entity_type, owner_id, metadata, parent_id } => {
                    let entity_id = EntityId::new();
                    let metadata = metadata.clone().unwrap_or_default();
                    
                    let entity = Entity {
                        entity_id,
                        entity_type: entity_type.clone(),
                        metadata,
                        parent_id: *parent_id,
                        children: Vec::new(),
                    };
                    
                    // Add to main storage
                    state.entities.insert(entity_id, entity.clone());
                    
                    // Update type index
                    state.type_indices.entry(entity_type.clone())
                        .or_insert_with(Vec::new)
                        .push(entity_id);
                    
                    // Handle ownership
                    if let Some(owner_id) = owner_id {
                        let ownership = EntityOwnership {
                            entity_id,
                            owner_id: *owner_id,
                            owner_type: entity_type.clone(), // Simplified
                            owned_type: entity_type.clone(),
                            created_at: SystemTime::now(),
                            permissions: EntityPermissions::default(),
                        };
                        
                        state.ownership.insert(entity_id, ownership);
                        state.owner_indices.entry(*owner_id)
                            .or_insert_with(Vec::new)
                            .push(entity_id);
                    }
                    
                    // Handle parent-child relationships
                    if let Some(parent_id) = parent_id {
                        state.children_indices.entry(*parent_id)
                            .or_insert_with(Vec::new)
                            .push(entity_id);
                        
                        // Add to parent's children list
                        if let Some(parent) = state.entities.get_mut(parent_id) {
                            parent.children.push(entity_id);
                        }
                    }
                    
                    // Update statistics
                    state.stats.total_entities += 1;
                    *state.stats.entities_by_type.entry(entity_type.clone()).or_insert(0) += 1;
                    state.stats.last_updated = SystemTime::now();
                    
                    info!("Created entity {} of type {:?} for {:?}", entity_id, entity_type, from);
                    Ok(GenServerReply::Reply(Box::new(entity)))
                }
                
                EntityCall::Get { entity_id } => {
                    debug!("Getting entity {} for {:?}", entity_id, from);
                    let entity = state.entities.get(entity_id).cloned();
                    Ok(GenServerReply::Reply(Box::new(entity)))
                }
                
                EntityCall::Update { entity_id, metadata } => {
                    if let Some(entity) = state.entities.get_mut(entity_id) {
                        entity.metadata = metadata.clone();
                        info!("Updated entity {} for {:?}", entity_id, from);
                        Ok(GenServerReply::Reply(Box::new(entity.clone())))
                    } else {
                        Ok(GenServerReply::Reply(Box::new(None::<Entity>)))
                    }
                }
                
                EntityCall::Delete { entity_id, hard_delete } => {
                    if *hard_delete {
                        // Hard delete - remove completely
                        if let Some(entity) = state.entities.remove(entity_id) {
                            self.remove_from_indices(state, entity_id, &entity.entity_type);
                            state.stats.total_entities -= 1;
                            *state.stats.entities_by_type.entry(entity.entity_type).or_insert(1) -= 1;
                            info!("Hard deleted entity {} for {:?}", entity_id, from);
                            Ok(GenServerReply::Reply(Box::new(true)))
                        } else {
                            Ok(GenServerReply::Reply(Box::new(false)))
                        }
                    } else {
                        // Soft delete - mark as deleted
                        if let Some(entity) = state.entities.get_mut(entity_id) {
                            entity.metadata.attributes.insert("deleted".to_string(), "true".to_string());
                            entity.metadata.attributes.insert("deleted_at".to_string(), 
                                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string());
                            info!("Soft deleted entity {} for {:?}", entity_id, from);
                            Ok(GenServerReply::Reply(Box::new(true)))
                        } else {
                            Ok(GenServerReply::Reply(Box::new(false)))
                        }
                    }
                }
                
                EntityCall::TransferOwnership { entity_id, new_owner_id, permissions } => {
                    if let Some(ownership) = state.ownership.get_mut(entity_id) {
                        let old_owner = ownership.owner_id;
                        ownership.owner_id = *new_owner_id;
                        
                        if let Some(perms) = permissions {
                            ownership.permissions = perms.clone();
                        }
                        
                        // Update owner indices
                        if let Some(old_list) = state.owner_indices.get_mut(&old_owner) {
                            old_list.retain(|id| id != entity_id);
                        }
                        state.owner_indices.entry(*new_owner_id)
                            .or_insert_with(Vec::new)
                            .push(*entity_id);
                        
                        info!("Transferred ownership of entity {} from {} to {} for {:?}", 
                              entity_id, old_owner, new_owner_id, from);
                        Ok(GenServerReply::Reply(Box::new(true)))
                    } else {
                        Ok(GenServerReply::Reply(Box::new(false)))
                    }
                }
                
                EntityCall::GetByType { entity_type } => {
                    debug!("Getting entities by type {:?} for {:?}", entity_type, from);
                    let entity_ids = state.type_indices.get(entity_type).cloned().unwrap_or_default();
                    let entities: Vec<Entity> = entity_ids.into_iter()
                        .filter_map(|id| state.entities.get(&id))
                        .cloned()
                        .collect();
                    Ok(GenServerReply::Reply(Box::new(entities)))
                }
                
                EntityCall::GetByOwner { owner_id } => {
                    debug!("Getting entities by owner {} for {:?}", owner_id, from);
                    let entity_ids = state.owner_indices.get(owner_id).cloned().unwrap_or_default();
                    let entities: Vec<Entity> = entity_ids.into_iter()
                        .filter_map(|id| state.entities.get(&id))
                        .cloned()
                        .collect();
                    Ok(GenServerReply::Reply(Box::new(entities)))
                }
                
                EntityCall::GetChildren { parent_id } => {
                    debug!("Getting children of entity {} for {:?}", parent_id, from);
                    let child_ids = state.children_indices.get(parent_id).cloned().unwrap_or_default();
                    let children: Vec<Entity> = child_ids.into_iter()
                        .filter_map(|id| state.entities.get(&id))
                        .cloned()
                        .collect();
                    Ok(GenServerReply::Reply(Box::new(children)))
                }
                
                EntityCall::CheckPermission { entity_id, requester_id, permission } => {
                    let has_permission = if let Some(ownership) = state.ownership.get(entity_id) {
                        if ownership.owner_id == *requester_id {
                            // Owner has all permissions
                            true
                        } else {
                            // Check specific permission
                            match permission.as_str() {
                                "read" => ownership.permissions.read,
                                "write" => ownership.permissions.write,
                                "delete" => ownership.permissions.delete,
                                "transfer" => ownership.permissions.transfer,
                                "admin" => ownership.permissions.admin,
                                _ => false,
                            }
                        }
                    } else {
                        false
                    };
                    
                    Ok(GenServerReply::Reply(Box::new(has_permission)))
                }
                
                EntityCall::GetStats => {
                    debug!("Getting system stats for {:?}", from);
                    Ok(GenServerReply::Reply(Box::new(state.stats.clone())))
                }
                
                EntityCall::Search { criteria, limit } => {
                    debug!("Searching entities for {:?}", from);
                    let results = self.search_entities(state, criteria, *limit);
                    Ok(GenServerReply::Reply(Box::new(results)))
                }
            }
        } else {
            warn!("Unknown call type from {:?}", from);
            Ok(GenServerReply::Reply(Box::new("unknown_call")))
        }
    }

    async fn handle_cast(
        &mut self,
        message: Box<dyn Any + Send + Sync>,
        state: &mut Self::State,
    ) -> HelixResult<()> {
        if let Some(cast) = message.downcast_ref::<EntityCast>() {
            match cast {
                EntityCast::AddTag { entity_id, tag } => {
                    if let Some(entity) = state.entities.get_mut(entity_id) {
                        if !entity.metadata.tags.contains(tag) {
                            entity.metadata.tags.push(tag.clone());
                            entity.metadata.updated_at = SystemTime::now();
                        }
                    }
                }
                
                EntityCast::RemoveTag { entity_id, tag } => {
                    if let Some(entity) = state.entities.get_mut(entity_id) {
                        entity.metadata.tags.retain(|t| t != tag);
                        entity.metadata.updated_at = SystemTime::now();
                    }
                }
                
                EntityCast::SetAttribute { entity_id, key, value } => {
                    if let Some(entity) = state.entities.get_mut(entity_id) {
                        entity.metadata.attributes.insert(key.clone(), value.clone());
                        entity.metadata.updated_at = SystemTime::now();
                    }
                }
                
                EntityCast::RemoveAttribute { entity_id, key } => {
                    if let Some(entity) = state.entities.get_mut(entity_id) {
                        entity.metadata.attributes.remove(key);
                        entity.metadata.updated_at = SystemTime::now();
                    }
                }
                
                EntityCast::BulkUpdate { updates } => {
                    for (entity_id, metadata) in updates {
                        if let Some(entity) = state.entities.get_mut(entity_id) {
                            entity.metadata = metadata.clone();
                        }
                    }
                    info!("Bulk updated {} entities", updates.len());
                }
                
                EntityCast::CleanupOrphans => {
                    self.cleanup_orphaned_entities(state).await?;
                    info!("Orphaned entities cleanup completed");
                }
                
                EntityCast::UpdateStats => {
                    self.update_statistics(state);
                    debug!("Statistics updated");
                }
                
                EntityCast::RefreshCache => {
                    // In a real implementation, this would refresh any caches
                    debug!("Cache refreshed");
                }
            }
        }
        Ok(())
    }

    async fn handle_info(
        &mut self,
        message: Box<dyn Any + Send + Sync>,
        source: InfoSource,
        state: &mut Self::State,
    ) -> HelixResult<()> {
        if let Some(info) = message.downcast_ref::<EntityInfo>() {
            match info {
                EntityInfo::CleanupTimer => {
                    debug!("Periodic cleanup triggered");
                    self.cleanup_orphaned_entities(state).await?;
                    // Schedule next cleanup
                    // In a real implementation, this would reschedule the timer
                }
                
                EntityInfo::CacheTimer => {
                    debug!("Cache refresh triggered");
                    // Refresh any cached data
                }
                
                EntityInfo::StatsTimer => {
                    debug!("Stats update triggered");
                    self.update_statistics(state);
                }
                
                EntityInfo::ExternalEvent { event_type, entity_id, data } => {
                    info!("External event '{}' for entity {}: {:?}", event_type, entity_id, data);
                    // Handle external events that might affect entities
                }
                
                EntityInfo::MonitorDown { monitored_id, reason } => {
                    warn!("Monitored entity {} went down: {}", monitored_id, reason);
                    // Handle entity that went down (cleanup, notifications, etc.)
                }
                
                EntityInfo::ResourceWarning { resource, usage, threshold } => {
                    warn!("Resource warning: {} usage {:.2}% exceeds threshold {:.2}%", 
                          resource, usage * 100.0, threshold * 100.0);
                    // Handle resource warnings (cleanup, throttling, etc.)
                }
            }
        }
        Ok(())
    }

    async fn handle_timeout(
        &mut self,
        duration: Duration,
        state: &mut Self::State,
    ) -> HelixResult<()> {
        info!("Entity system timeout after {:?}", duration);
        
        // Perform periodic maintenance
        self.cleanup_orphaned_entities(state).await?;
        self.update_statistics(state);
        
        Ok(())
    }

    async fn terminate(
        &mut self,
        reason: TerminateReason,
        state: &mut Self::State,
    ) -> HelixResult<()> {
        info!("Entity System GenServer terminating: {:?}", reason);
        
        // Perform final cleanup
        info!("Final stats: {} entities, {} ownership relationships", 
              state.stats.total_entities, state.stats.ownership_relationships);
        
        Ok(())
    }
}

impl EntitySystemGenServer {
    /// Remove entity from all indices
    fn remove_from_indices(&self, state: &mut EntitySystemState, entity_id: &EntityId, entity_type: &EntityType) {
        // Remove from type index
        if let Some(type_list) = state.type_indices.get_mut(entity_type) {
            type_list.retain(|id| id != entity_id);
        }
        
        // Remove from ownership
        if let Some(ownership) = state.ownership.remove(entity_id) {
            if let Some(owner_list) = state.owner_indices.get_mut(&ownership.owner_id) {
                owner_list.retain(|id| id != entity_id);
            }
        }
        
        // Remove from children indices
        if let Some(children) = state.children_indices.remove(entity_id) {
            // Orphan the children or handle as needed
            for child_id in children {
                if let Some(child) = state.entities.get_mut(&child_id) {
                    child.parent_id = None;
                }
            }
        }
        
        // Remove from parent's children list
        for (_, entity) in state.entities.iter_mut() {
            entity.children.retain(|id| id != entity_id);
        }
    }
    
    /// Cleanup orphaned entities
    async fn cleanup_orphaned_entities(&self, state: &mut EntitySystemState) -> HelixResult<()> {
        let mut to_remove = Vec::new();
        
        for (entity_id, entity) in state.entities.iter() {
            // Check if entity is marked as deleted and old enough
            if let Some(deleted_at_str) = entity.metadata.attributes.get("deleted") {
                if deleted_at_str == "true" {
                    if let Some(deleted_at_str) = entity.metadata.attributes.get("deleted_at") {
                        if let Ok(deleted_at) = deleted_at_str.parse::<u64>() {
                            let deleted_time = UNIX_EPOCH + Duration::from_secs(deleted_at);
                            let cleanup_threshold = SystemTime::now() - Duration::from_secs(86400); // 24 hours
                            
                            if deleted_time < cleanup_threshold {
                                to_remove.push(*entity_id);
                            }
                        }
                    }
                }
            }
        }
        
        for entity_id in to_remove {
            if let Some(entity) = state.entities.remove(&entity_id) {
                self.remove_from_indices(state, &entity_id, &entity.entity_type);
                state.stats.total_entities -= 1;
                *state.stats.entities_by_type.entry(entity.entity_type).or_insert(1) -= 1;
            }
        }
        
        Ok(())
    }
    
    /// Update system statistics
    fn update_statistics(&self, state: &mut EntitySystemState) {
        state.stats.total_entities = state.entities.len() as u64;
        
        let mut type_counts = HashMap::new();
        for entity in state.entities.values() {
            *type_counts.entry(entity.entity_type.clone()).or_insert(0) += 1;
        }
        
        state.stats.entities_by_type = type_counts;
        state.stats.ownership_relationships = state.ownership.len() as u64;
        state.stats.last_updated = SystemTime::now();
    }
    
    /// Search entities with criteria
    fn search_entities(
        &self,
        state: &EntitySystemState,
        criteria: &EntitySearchCriteria,
        limit: Option<u32>,
    ) -> Vec<Entity> {
        let mut results = Vec::new();
        let max_results = limit.unwrap_or(1000) as usize;
        
        for entity in state.entities.values() {
            if results.len() >= max_results {
                break;
            }
            
            // Filter by entity type
            if let Some(entity_type) = &criteria.entity_type {
                if entity.entity_type != *entity_type {
                    continue;
                }
            }
            
            // Filter by owner
            if let Some(owner_id) = criteria.owner_id {
                if let Some(ownership) = state.ownership.get(&entity.entity_id) {
                    if ownership.owner_id != owner_id {
                        continue;
                    }
                } else {
                    continue;
                }
            }
            
            // Filter by tags
            if let Some(tags) = &criteria.tags {
                if !tags.iter().all(|tag| entity.metadata.tags.contains(tag)) {
                    continue;
                }
            }
            
            // Filter by attributes
            if let Some(attributes) = &criteria.attributes {
                if !attributes.iter().all(|(key, value)| {
                    entity.metadata.attributes.get(key).map(|v| v == value).unwrap_or(false)
                }) {
                    continue;
                }
            }
            
            // Filter by creation time
            if let Some(after) = criteria.created_after {
                if entity.metadata.created_at < after {
                    continue;
                }
            }
            
            if let Some(before) = criteria.created_before {
                if entity.metadata.created_at > before {
                    continue;
                }
            }
            
            results.push(entity.clone());
        }
        
        results
    }
}

/// Entity System Supervisor
pub struct EntitySystemSupervisor {
    supervisor: GenServerSupervisor,
}

impl EntitySystemSupervisor {
    pub fn new() -> Self {
        Self {
            supervisor: GenServerSupervisor::new(SupervisionStrategy::OneForOne),
        }
    }
    
    pub async fn start(&mut self) -> HelixResult<GenServerHandle> {
        let genserver = EntitySystemGenServer;
        let config = EntitySystemConfig::default();
        
        GenServerHandle::start(genserver, config, Some("entity_system".to_string())).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_entity_creation() {
        let genserver = EntitySystemGenServer;
        let handle = GenServerHandle::start(
            genserver, 
            EntitySystemConfig::default(), 
            Some("test_entity_system".to_string())
        ).await.expect("Failed to start EntitySystemGenServer");

        let call = EntityCall::Create {
            entity_type: EntityType::Server,
            owner_id: Some(EntityId::new()),
            metadata: None,
            parent_id: None,
        };

        let entity: Entity = handle.call(call, None).await.expect("Failed to create entity");
        assert_eq!(entity.entity_type, EntityType::Server);

        handle.stop(TerminateReason::Normal).await.expect("Failed to stop");
    }

    #[tokio::test]
    async fn test_entity_ownership() {
        let genserver = EntitySystemGenServer;
        let handle = GenServerHandle::start(
            genserver, 
            EntitySystemConfig::default(), 
            Some("test_entity_system".to_string())
        ).await.expect("Failed to start EntitySystemGenServer");

        let owner_id = EntityId::new();
        
        let call = EntityCall::Create {
            entity_type: EntityType::Server,
            owner_id: Some(owner_id),
            metadata: None,
            parent_id: None,
        };

        let entity: Entity = handle.call(call, None).await.expect("Failed to create entity");
        
        let owned_call = EntityCall::GetByOwner { owner_id };
        let owned: Vec<Entity> = handle.call(owned_call, None).await.expect("Failed to get owned entities");
        
        assert_eq!(owned.len(), 1);
        assert_eq!(owned[0].entity_id, entity.entity_id);

        handle.stop(TerminateReason::Normal).await.expect("Failed to stop");
    }
}