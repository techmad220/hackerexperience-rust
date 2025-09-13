//! Repository pattern implementation for database operations

use he_helix_core::{HelixError, HelixResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use thiserror::Error;

/// Error types specific to repository operations
#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Entity not found: {0}")]
    NotFound(String),
    
    #[error("Unique constraint violation: {0}")]
    UniqueViolation(String),
    
    #[error("Foreign key constraint violation: {0}")]
    ForeignKeyViolation(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl From<RepositoryError> for HelixError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound(msg) => HelixError::not_found(msg),
            RepositoryError::ValidationError(msg) => HelixError::validation(msg),
            RepositoryError::DatabaseError(e) => HelixError::Database(e),
            RepositoryError::SerializationError(e) => HelixError::Serialization(e),
            RepositoryError::UniqueViolation(msg) => HelixError::validation(msg),
            RepositoryError::ForeignKeyViolation(msg) => HelixError::validation(msg),
        }
    }
}

/// Base repository trait for CRUD operations
#[async_trait]
pub trait Repository<T, ID>: Send + Sync
where
    T: Send + Sync + Debug,
    ID: Send + Sync + Debug,
{
    /// Find an entity by ID
    async fn find_by_id(&self, id: &ID) -> HelixResult<Option<T>>;
    
    /// Find all entities
    async fn find_all(&self) -> HelixResult<Vec<T>>;
    
    /// Find entities with pagination
    async fn find_all_paginated(&self, offset: u64, limit: u64) -> HelixResult<Vec<T>>;
    
    /// Create a new entity
    async fn create(&self, entity: &T) -> HelixResult<T>;
    
    /// Update an existing entity
    async fn update(&self, id: &ID, entity: &T) -> HelixResult<T>;
    
    /// Delete an entity by ID
    async fn delete(&self, id: &ID) -> HelixResult<bool>;
    
    /// Check if an entity exists by ID
    async fn exists(&self, id: &ID) -> HelixResult<bool>;
    
    /// Count total number of entities
    async fn count(&self) -> HelixResult<u64>;
}

/// Extended repository trait for more complex queries
#[async_trait]
pub trait ExtendedRepository<T, ID>: Repository<T, ID>
where
    T: Send + Sync + Debug,
    ID: Send + Sync + Debug,
{
    /// Find entities by a custom filter
    async fn find_by_filter(&self, filter: &str, params: &[&dyn ToString]) -> HelixResult<Vec<T>>;
    
    /// Find a single entity by a custom filter
    async fn find_one_by_filter(&self, filter: &str, params: &[&dyn ToString]) -> HelixResult<Option<T>>;
    
    /// Execute a custom query
    async fn execute_query(&self, query: &str, params: &[&dyn ToString]) -> HelixResult<u64>;
    
    /// Bulk insert entities
    async fn bulk_create(&self, entities: &[T]) -> HelixResult<Vec<T>>;
    
    /// Bulk update entities
    async fn bulk_update(&self, updates: &[(ID, T)]) -> HelixResult<u64>;
    
    /// Bulk delete entities
    async fn bulk_delete(&self, ids: &[ID]) -> HelixResult<u64>;
}

/// Transaction support trait
#[async_trait]
pub trait TransactionalRepository<T, ID>: Repository<T, ID>
where
    T: Send + Sync + Debug,
    ID: Send + Sync + Debug,
{
    /// Execute operations within a transaction
    async fn with_transaction<F, R>(&self, f: F) -> HelixResult<R>
    where
        F: FnOnce() -> futures::future::BoxFuture<'_, HelixResult<R>> + Send + 'static,
        R: Send + 'static;
}

/// Audit trail support for entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditInfo {
    /// When the entity was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When the entity was last updated
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Who created the entity
    pub created_by: Option<String>,
    /// Who last updated the entity
    pub updated_by: Option<String>,
}

impl Default for AuditInfo {
    fn default() -> Self {
        Self {
            created_at: chrono::Utc::now(),
            updated_at: None,
            created_by: None,
            updated_by: None,
        }
    }
}

impl AuditInfo {
    /// Mark as updated by a specific user
    pub fn updated_by(mut self, user: String) -> Self {
        self.updated_at = Some(chrono::Utc::now());
        self.updated_by = Some(user);
        self
    }
    
    /// Mark as created by a specific user
    pub fn created_by(mut self, user: String) -> Self {
        self.created_by = Some(user);
        self
    }
}

/// Trait for entities that support audit trails
pub trait Auditable {
    /// Get audit information
    fn audit_info(&self) -> &AuditInfo;
    
    /// Get mutable audit information
    fn audit_info_mut(&mut self) -> &mut AuditInfo;
    
    /// Update the audit information for an update operation
    fn mark_updated(&mut self, user: Option<String>) {
        let audit = self.audit_info_mut();
        audit.updated_at = Some(chrono::Utc::now());
        audit.updated_by = user;
    }
}

/// Soft delete support for entities
pub trait SoftDeletable {
    /// Check if the entity is soft deleted
    fn is_deleted(&self) -> bool;
    
    /// Mark the entity as soft deleted
    fn mark_deleted(&mut self);
    
    /// Restore a soft deleted entity
    fn restore(&mut self);
    
    /// Get the deletion timestamp
    fn deleted_at(&self) -> Option<chrono::DateTime<chrono::Utc>>;
}

/// Repository factory for creating domain-specific repositories
pub trait RepositoryFactory {
    /// Create a repository for the account domain
    fn account_repository(&self) -> Box<dyn Repository<AccountEntity, uuid::Uuid>>;
    
    /// Create a repository for the server domain
    fn server_repository(&self) -> Box<dyn Repository<ServerEntity, uuid::Uuid>>;
    
    /// Create a repository for the network domain
    fn network_repository(&self) -> Box<dyn Repository<NetworkEntity, uuid::Uuid>>;
    
    // Add more domain-specific repositories as needed
}

/// Placeholder entity types for the factory trait
/// These would be replaced with actual domain entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountEntity {
    pub id: uuid::Uuid,
    pub email: String,
    pub username: String,
    pub audit: AuditInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEntity {
    pub id: uuid::Uuid,
    pub name: String,
    pub ip_address: String,
    pub account_id: uuid::Uuid,
    pub audit: AuditInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEntity {
    pub id: uuid::Uuid,
    pub name: String,
    pub server_id: uuid::Uuid,
    pub audit: AuditInfo,
}

// Implement Auditable for example entities
impl Auditable for AccountEntity {
    fn audit_info(&self) -> &AuditInfo {
        &self.audit
    }
    
    fn audit_info_mut(&mut self) -> &mut AuditInfo {
        &mut self.audit
    }
}

impl Auditable for ServerEntity {
    fn audit_info(&self) -> &AuditInfo {
        &self.audit
    }
    
    fn audit_info_mut(&mut self) -> &mut AuditInfo {
        &mut self.audit
    }
}

impl Auditable for NetworkEntity {
    fn audit_info(&self) -> &AuditInfo {
        &self.audit
    }
    
    fn audit_info_mut(&mut self) -> &mut AuditInfo {
        &mut self.audit
    }
}