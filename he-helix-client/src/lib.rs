//! Helix Client - Client Connection and Rendering System
//!
//! This crate handles client connections, rendering, and client-specific operations
//! for the Hacker Experience game. It supports multiple client types including
//! web clients and mobile clients.
//!
//! # Features
//!
//! - Client type identification and validation
//! - Client-specific rendering pipeline
//! - Client action tracking and events
//! - WebSocket request handling for clients
//! - Client setup and bootstrap operations

pub mod event;
pub mod model;
pub mod renderer;
pub mod supervisor;
pub mod web1;
pub mod websocket;

pub use model::Client;
pub use renderer::Renderer;

use thiserror::Error;

/// Client-related errors
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Invalid client type: {client_type}")]
    InvalidClientType { client_type: String },
    
    #[error("Client not found: {client_id}")]
    ClientNotFound { client_id: String },
    
    #[error("Client not connected: {client_id}")]
    ClientNotConnected { client_id: String },
    
    #[error("Client action not supported: {action}")]
    UnsupportedAction { action: String },
    
    #[error("Client setup failed: {reason}")]
    SetupFailed { reason: String },
    
    #[error("Rendering failed: {reason}")]
    RenderingFailed { reason: String },
}

/// Result type for client operations
pub type ClientResult<T> = Result<T, ClientError>;