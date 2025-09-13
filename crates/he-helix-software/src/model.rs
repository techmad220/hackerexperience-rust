//! Software model definitions

use crate::types::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use he_helix_server::ServerId;

/// Software entity model
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Software {
    pub software_id: SoftwareId,
    pub software_type: SoftwareType,
    pub name: String,
    pub version: String,
    pub modules: Vec<SoftwareModule>,
    pub size: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// File entity model
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct File {
    pub file_id: FileId,
    pub file_type: FileType,
    pub name: String,
    pub content: Vec<u8>,
    pub server_id: ServerId,
    pub storage_id: StorageId,
    pub path: String,
    pub metadata: FileMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Virus entity model
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Virus {
    pub virus_id: VirusId,
    pub virus_type: SoftwareType,
    pub name: String,
    pub target_server_id: ServerId,
    pub state: VirusState,
    pub collection_target: Option<String>,
    pub collected_data: Vec<VirusCollectionData>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Virus {
    pub fn is_active(&self) -> bool {
        matches!(self.state, VirusState::Active | VirusState::Collecting)
    }
}

/// Crypto key entity model
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CryptoKey {
    pub key_id: CryptoKeyId,
    pub name: String,
    pub key_data: Vec<u8>,
    pub key_info: CryptoKeyInfo,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Text file entity model
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextFile {
    pub file_id: FileId,
    pub name: String,
    pub content: String,
    pub server_id: ServerId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}