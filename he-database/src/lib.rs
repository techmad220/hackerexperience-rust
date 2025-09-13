//! # Helix Database Infrastructure
//!
//! This crate provides the database layer for Helix, supporting the multi-database
//! architecture with domain-specific databases.

pub mod config;
pub mod connection;
pub mod manager;
pub mod migrations;
pub mod models;
pub mod repository;

// Re-export commonly used types
pub use config::{DatabaseConfig, DatabaseType};
pub use connection::{DatabaseConnection, DatabaseConnectionPool};
pub use manager::DatabaseManager;
pub use repository::{Repository, RepositoryError};

use he_helix_core::HelixResult;