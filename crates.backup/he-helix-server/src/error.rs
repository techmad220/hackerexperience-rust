//! Error types for the server module

use thiserror::Error;
use crate::types::*;

/// Server-related errors
#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Server not found: {id}")]
    NotFound { id: ServerId },
    
    #[error("Server already has a motherboard attached")]
    MotherboardAlreadyAttached,
    
    #[error("Server has no motherboard attached")]
    NoMotherboardAttached,
    
    #[error("Invalid server type: {server_type}")]
    InvalidServerType { server_type: String },
    
    #[error("Invalid hostname: {hostname}")]
    InvalidHostname { hostname: String },
    
    #[error("Database error: {source}")]
    Database { source: sqlx::Error },
    
    #[error("Component error: {source}")]
    Component { source: ComponentError },
}

/// Component-related errors
#[derive(Error, Debug)]
pub enum ComponentError {
    #[error("Component not found: {id}")]
    NotFound { id: ComponentId },
    
    #[error("Invalid component type: {component_type}")]
    InvalidType { component_type: String },
    
    #[error("Component is not pluggable: {component_type:?}")]
    NotPluggable { component_type: ComponentType },
    
    #[error("Invalid component specification: {spec_value}")]
    InvalidSpec { spec_value: u32 },
    
    #[error("Database error: {source}")]
    Database { source: sqlx::Error },
}

/// Motherboard-related errors
#[derive(Error, Debug)]
pub enum MotherboardError {
    #[error("Motherboard not found: {id}")]
    NotFound { id: MotherboardId },
    
    #[error("No available slots for component type: {component_type:?}")]
    NoAvailableSlots { component_type: ComponentType },
    
    #[error("Component not found in motherboard: {component_id}")]
    ComponentNotFound { component_id: ComponentId },
    
    #[error("Invalid slot ID: {slot_id}")]
    InvalidSlotId { slot_id: SlotId },
    
    #[error("Slot already occupied: {slot_id}")]
    SlotOccupied { slot_id: SlotId },
    
    #[error("Component type mismatch for slot: expected {expected:?}, found {found:?}")]
    ComponentTypeMismatch { expected: ComponentType, found: ComponentType },
    
    #[error("Database error: {source}")]
    Database { source: sqlx::Error },
    
    #[error("Component error: {source}")]
    Component { source: ComponentError },
}

/// Resource calculation errors
#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("Insufficient resources: required {required:?}, available {available:?}")]
    InsufficientResources { required: crate::resources::Resources, available: crate::resources::Resources },
    
    #[error("Invalid resource allocation: {resource_type}")]
    InvalidAllocation { resource_type: String },
    
    #[error("Resource limit exceeded: {resource_type}")]
    LimitExceeded { resource_type: String },
    
    #[error("Component not found for resource calculation: {component_id}")]
    ComponentNotFound { component_id: ComponentId },
}

/// General server subsystem errors
#[derive(Error, Debug)]
pub enum HeliuServerError {
    #[error("Server error: {source}")]
    Server { source: ServerError },
    
    #[error("Component error: {source}")]
    Component { source: ComponentError },
    
    #[error("Motherboard error: {source}")]
    Motherboard { source: MotherboardError },
    
    #[error("Resource error: {source}")]
    Resource { source: ResourceError },
    
    #[error("Database connection error: {source}")]
    DatabaseConnection { source: sqlx::Error },
    
    #[error("Internal error: {message}")]
    Internal { message: String },
    
    #[error("Configuration error: {message}")]
    Configuration { message: String },
}

impl From<ServerError> for HeliuServerError {
    fn from(err: ServerError) -> Self {
        HeliuServerError::Server { source: err }
    }
}

impl From<ComponentError> for HeliuServerError {
    fn from(err: ComponentError) -> Self {
        HeliuServerError::Component { source: err }
    }
}

impl From<MotherboardError> for HeliuServerError {
    fn from(err: MotherboardError) -> Self {
        HeliuServerError::Motherboard { source: err }
    }
}

impl From<ResourceError> for HeliuServerError {
    fn from(err: ResourceError) -> Self {
        HeliuServerError::Resource { source: err }
    }
}

impl From<sqlx::Error> for HeliuServerError {
    fn from(err: sqlx::Error) -> Self {
        HeliuServerError::DatabaseConnection { source: err }
    }
}

impl From<sqlx::Error> for ServerError {
    fn from(err: sqlx::Error) -> Self {
        ServerError::Database { source: err }
    }
}

impl From<sqlx::Error> for ComponentError {
    fn from(err: sqlx::Error) -> Self {
        ComponentError::Database { source: err }
    }
}

impl From<sqlx::Error> for MotherboardError {
    fn from(err: sqlx::Error) -> Self {
        MotherboardError::Database { source: err }
    }
}

impl From<ComponentError> for ServerError {
    fn from(err: ComponentError) -> Self {
        ServerError::Component { source: err }
    }
}

impl From<ComponentError> for MotherboardError {
    fn from(err: ComponentError) -> Self {
        MotherboardError::Component { source: err }
    }
}

/// Result type alias for server operations
pub type ServerResult<T> = Result<T, ServerError>;

/// Result type alias for component operations
pub type ComponentResult<T> = Result<T, ComponentError>;

/// Result type alias for motherboard operations
pub type MotherboardResult<T> = Result<T, MotherboardError>;

/// Result type alias for resource operations
pub type ResourceResult<T> = Result<T, ResourceError>;

/// Result type alias for general server subsystem operations
pub type HeliuServerResult<T> = Result<T, HeliuServerError>;