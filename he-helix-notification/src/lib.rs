//! Helix Notification System
//!
//! This crate provides a comprehensive notification system for the Hacker Experience game.
//! It handles notification creation, delivery, rendering, and management across different
//! notification types and classes.
//!
//! # Features
//!
//! - Notification codes and classes system
//! - Server, Chat, and Entity-based notifications
//! - WebSocket delivery for real-time notifications
//! - Notification reading status tracking
//! - Custom notification data and rendering
//! - Event-driven notification creation

pub mod action;
pub mod event;
pub mod henforcer;
pub mod model;
pub mod public;
pub mod query;
pub mod supervisor;
pub mod websocket;

pub use model::{Notification, NotificationClass, NotificationCode};

use thiserror::Error;

/// Notification-related errors
#[derive(Debug, Error)]
pub enum NotificationError {
    #[error("Notification not found: {id}")]
    NotificationNotFound { id: String },
    
    #[error("Invalid notification class: {class}")]
    InvalidClass { class: String },
    
    #[error("Invalid notification code: {code} for class {class}")]
    InvalidCode { class: String, code: String },
    
    #[error("Notification already read: {id}")]
    AlreadyRead { id: String },
    
    #[error("Permission denied: cannot access notification {id}")]
    PermissionDenied { id: String },
    
    #[error("Failed to render notification: {reason}")]
    RenderingFailed { reason: String },
    
    #[error("Database error: {error}")]
    Database { error: String },
    
    #[error("Serialization error: {error}")]
    Serialization { error: String },
}

/// Result type for notification operations
pub type NotificationResult<T> = Result<T, NotificationError>;