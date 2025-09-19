//! # Helix Server Management System
//!
//! This crate provides the core server management functionality for the HackerExperience
//! game engine, including server entities, hardware components (motherboards, CPU, RAM, etc.),
//! and the component system architecture.
//!
//! ## Architecture
//!
//! The server system is built around several key concepts:
//! - **Server**: The main entity representing a computer system in the game
//! - **Components**: Hardware components like motherboards, CPUs, RAM, NICs, HDDs
//! - **Motherboard**: The central component that connects and manages other hardware
//! - **Resources**: CPU, RAM, HDD, and network capacity calculations
//!
//! ## Key Features
//!
//! - Async/await support using Tokio
//! - Actor-based architecture using Actix
//! - Database persistence with SeaORM
//! - Component composition and resource management
//! - Hardware specification and capacity calculations

pub mod actors;
pub mod component;
pub mod error;
pub mod model;
pub mod query;
pub mod resources;
pub mod supervisor;
pub mod types;

pub use model::{Component, Motherboard, Server, ServerType};
pub use resources::Resources;
pub use types::*;

use anyhow::Result;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Global server registry for managing server instances
pub static SERVER_REGISTRY: Lazy<Arc<RwLock<ServerRegistry>>> = 
    Lazy::new(|| Arc::new(RwLock::new(ServerRegistry::new())));

/// Server registry for tracking active server instances
#[derive(Debug, Default)]
pub struct ServerRegistry {
    servers: dashmap::DashMap<ServerId, Arc<Server>>,
}

impl ServerRegistry {
    pub fn new() -> Self {
        Self {
            servers: dashmap::DashMap::new(),
        }
    }

    pub async fn register_server(&self, server: Server) -> Arc<Server> {
        let server_arc = Arc::new(server);
        self.servers.insert(server_arc.server_id.clone(), server_arc.clone());
        server_arc
    }

    pub async fn get_server(&self, server_id: &ServerId) -> Option<Arc<Server>> {
        self.servers.get(server_id).map(|entry| entry.clone())
    }

    pub async fn remove_server(&self, server_id: &ServerId) -> Option<Arc<Server>> {
        self.servers.remove(server_id).map(|(_, server)| server)
    }

    pub async fn list_servers(&self) -> Vec<Arc<Server>> {
        self.servers.iter().map(|entry| entry.value().clone()).collect()
    }
}

/// Initialize the server subsystem
pub async fn init() -> Result<()> {
    tracing::info!("Initializing Helix Server subsystem");
    
    // Initialize the server registry
    let _registry = SERVER_REGISTRY.clone();
    
    tracing::info!("Helix Server subsystem initialized successfully");
    Ok(())
}

/// Shutdown the server subsystem gracefully
pub async fn shutdown() -> Result<()> {
    tracing::info!("Shutting down Helix Server subsystem");
    
    // Clear the server registry
    let registry = SERVER_REGISTRY.read().await;
    registry.servers.clear();
    
    tracing::info!("Helix Server subsystem shutdown complete");
    Ok(())
}