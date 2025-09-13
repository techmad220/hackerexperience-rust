//! # Helix Software and Virus System
//!
//! This crate provides the software management functionality for the HackerExperience
//! game engine, including software types, file systems, viruses, and related tools.
//!
//! ## Architecture
//!
//! The software system is built around several key concepts:
//! - **Software**: Various types of software tools (crackers, firewalls, etc.)
//! - **Files**: Individual file instances with metadata and content
//! - **Viruses**: Malicious software with spreading and collection capabilities
//! - **Storage**: File system and storage management
//! - **Modules**: Software modules providing specific capabilities
//!
//! ## Key Features
//!
//! - Async/await support using Tokio
//! - Actor-based architecture using Actix
//! - Database persistence with SeaORM
//! - Software type system with extensions and modules
//! - File system management and organization
//! - Virus propagation and collection systems

pub mod actors;
pub mod crypto;
pub mod error;
pub mod file;
pub mod model;
pub mod modules;
pub mod query;
pub mod software;
pub mod storage;
pub mod types;
pub mod virus;

pub use model::{CryptoKey, File, Software, SoftwareType, TextFile, Virus};
pub use types::*;

use anyhow::Result;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Global software registry for managing software instances and types
pub static SOFTWARE_REGISTRY: Lazy<Arc<RwLock<SoftwareRegistry>>> = 
    Lazy::new(|| Arc::new(RwLock::new(SoftwareRegistry::new())));

/// Software registry for tracking software instances, files, and viruses
#[derive(Debug, Default)]
pub struct SoftwareRegistry {
    software: dashmap::DashMap<SoftwareId, Arc<Software>>,
    files: dashmap::DashMap<FileId, Arc<File>>,
    viruses: dashmap::DashMap<VirusId, Arc<Virus>>,
    crypto_keys: dashmap::DashMap<CryptoKeyId, Arc<CryptoKey>>,
}

impl SoftwareRegistry {
    pub fn new() -> Self {
        Self {
            software: dashmap::DashMap::new(),
            files: dashmap::DashMap::new(),
            viruses: dashmap::DashMap::new(),
            crypto_keys: dashmap::DashMap::new(),
        }
    }

    pub async fn register_software(&self, software: Software) -> Arc<Software> {
        let software_arc = Arc::new(software);
        self.software.insert(software_arc.software_id.clone(), software_arc.clone());
        software_arc
    }

    pub async fn register_file(&self, file: File) -> Arc<File> {
        let file_arc = Arc::new(file);
        self.files.insert(file_arc.file_id.clone(), file_arc.clone());
        file_arc
    }

    pub async fn register_virus(&self, virus: Virus) -> Arc<Virus> {
        let virus_arc = Arc::new(virus);
        self.viruses.insert(virus_arc.virus_id.clone(), virus_arc.clone());
        virus_arc
    }

    pub async fn register_crypto_key(&self, crypto_key: CryptoKey) -> Arc<CryptoKey> {
        let crypto_key_arc = Arc::new(crypto_key);
        self.crypto_keys.insert(crypto_key_arc.key_id.clone(), crypto_key_arc.clone());
        crypto_key_arc
    }

    pub async fn get_software(&self, software_id: &SoftwareId) -> Option<Arc<Software>> {
        self.software.get(software_id).map(|entry| entry.clone())
    }

    pub async fn get_file(&self, file_id: &FileId) -> Option<Arc<File>> {
        self.files.get(file_id).map(|entry| entry.clone())
    }

    pub async fn get_virus(&self, virus_id: &VirusId) -> Option<Arc<Virus>> {
        self.viruses.get(virus_id).map(|entry| entry.clone())
    }

    pub async fn get_crypto_key(&self, crypto_key_id: &CryptoKeyId) -> Option<Arc<CryptoKey>> {
        self.crypto_keys.get(crypto_key_id).map(|entry| entry.clone())
    }

    pub async fn remove_software(&self, software_id: &SoftwareId) -> Option<Arc<Software>> {
        self.software.remove(software_id).map(|(_, software)| software)
    }

    pub async fn remove_file(&self, file_id: &FileId) -> Option<Arc<File>> {
        self.files.remove(file_id).map(|(_, file)| file)
    }

    pub async fn remove_virus(&self, virus_id: &VirusId) -> Option<Arc<Virus>> {
        self.viruses.remove(virus_id).map(|(_, virus)| virus)
    }

    pub async fn remove_crypto_key(&self, crypto_key_id: &CryptoKeyId) -> Option<Arc<CryptoKey>> {
        self.crypto_keys.remove(crypto_key_id).map(|(_, key)| key)
    }

    pub async fn list_software(&self) -> Vec<Arc<Software>> {
        self.software.iter().map(|entry| entry.value().clone()).collect()
    }

    pub async fn list_files(&self) -> Vec<Arc<File>> {
        self.files.iter().map(|entry| entry.value().clone()).collect()
    }

    pub async fn list_viruses(&self) -> Vec<Arc<Virus>> {
        self.viruses.iter().map(|entry| entry.value().clone()).collect()
    }

    pub async fn list_crypto_keys(&self) -> Vec<Arc<CryptoKey>> {
        self.crypto_keys.iter().map(|entry| entry.value().clone()).collect()
    }

    /// Get software instances by type
    pub async fn get_software_by_type(&self, software_type: SoftwareType) -> Vec<Arc<Software>> {
        self.software
            .iter()
            .filter(|entry| entry.software_type == software_type)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get files by type
    pub async fn get_files_by_type(&self, file_type: FileType) -> Vec<Arc<File>> {
        self.files
            .iter()
            .filter(|entry| entry.file_type == file_type)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get active viruses
    pub async fn get_active_viruses(&self) -> Vec<Arc<Virus>> {
        self.viruses
            .iter()
            .filter(|entry| entry.is_active())
            .map(|entry| entry.value().clone())
            .collect()
    }
}

/// Initialize the software subsystem
pub async fn init() -> Result<()> {
    tracing::info!("Initializing Helix Software subsystem");
    
    // Initialize the software registry
    let _registry = SOFTWARE_REGISTRY.clone();
    
    tracing::info!("Helix Software subsystem initialized successfully");
    Ok(())
}

/// Shutdown the software subsystem gracefully
pub async fn shutdown() -> Result<()> {
    tracing::info!("Shutting down Helix Software subsystem");
    
    // Clear all registries
    let registry = SOFTWARE_REGISTRY.read().await;
    registry.software.clear();
    registry.files.clear();
    registry.viruses.clear();
    registry.crypto_keys.clear();
    
    tracing::info!("Helix Software subsystem shutdown complete");
    Ok(())
}