//! Error handling for the Helix system

use thiserror::Error;

/// Result type used throughout Helix
pub type HelixResult<T> = Result<T, HelixError>;

/// Main error type for the Helix system
#[derive(Error, Debug)]
pub enum HelixError {
    /// Database-related errors
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Event system errors
    #[error("Event error: {0}")]
    Event(String),

    /// Actor system errors
    #[error("Actor error: {0}")]
    Actor(String),

    /// Process errors
    #[error("Process error: {0}")]
    Process(String),

    /// Listener errors
    #[error("Listener error: {0}")]
    Listener(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Network errors
    #[error("Network error: {0}")]
    Network(String),

    /// Internal system errors
    #[error("Internal error: {0}")]
    Internal(String),

    /// Not found errors
    #[error("Not found: {0}")]
    NotFound(String),

    /// Permission denied errors
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Timeout errors
    #[error("Timeout: {0}")]
    Timeout(String),
}

impl HelixError {
    /// Create a new event error
    pub fn event<S: Into<String>>(msg: S) -> Self {
        Self::Event(msg.into())
    }

    /// Create a new actor error
    pub fn actor<S: Into<String>>(msg: S) -> Self {
        Self::Actor(msg.into())
    }

    /// Create a new process error
    pub fn process<S: Into<String>>(msg: S) -> Self {
        Self::Process(msg.into())
    }

    /// Create a new listener error
    pub fn listener<S: Into<String>>(msg: S) -> Self {
        Self::Listener(msg.into())
    }

    /// Create a new validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a new configuration error
    pub fn configuration<S: Into<String>>(msg: S) -> Self {
        Self::Configuration(msg.into())
    }

    /// Create a new network error
    pub fn network<S: Into<String>>(msg: S) -> Self {
        Self::Network(msg.into())
    }

    /// Create a new internal error
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::Internal(msg.into())
    }

    /// Create a new not found error
    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create a new permission denied error
    pub fn permission_denied<S: Into<String>>(msg: S) -> Self {
        Self::PermissionDenied(msg.into())
    }

    /// Create a new timeout error
    pub fn timeout<S: Into<String>>(msg: S) -> Self {
        Self::Timeout(msg.into())
    }
}