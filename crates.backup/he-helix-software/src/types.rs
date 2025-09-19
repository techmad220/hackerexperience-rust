//! Type definitions for the software module

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use he_helix_server::ServerId;

/// Software unique identifier
pub type SoftwareId = Uuid;

/// File unique identifier
pub type FileId = Uuid;

/// Virus unique identifier  
pub type VirusId = Uuid;

/// Crypto key unique identifier
pub type CryptoKeyId = Uuid;

/// Storage unique identifier
pub type StorageId = Uuid;

/// Software type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoftwareType {
    /// Password cracking software
    Cracker,
    /// Network firewall software
    Firewall,
    /// Text file
    Text,
    /// System exploit software
    Exploit,
    /// Password hashing software
    Hasher,
    /// Log manipulation software
    LogForger,
    /// Log recovery software
    LogRecover,
    /// File/data encryption software
    Encryptor,
    /// File/data decryption software
    Decryptor,
    /// Network/geographic mapping software
    Anymap,
    /// Cryptographic key file
    CryptoKey,
    /// Spyware virus
    VirusSpyware,
}

impl SoftwareType {
    pub fn all_types() -> &'static [SoftwareType] {
        &[
            SoftwareType::Cracker,
            SoftwareType::Firewall,
            SoftwareType::Text,
            SoftwareType::Exploit,
            SoftwareType::Hasher,
            SoftwareType::LogForger,
            SoftwareType::LogRecover,
            SoftwareType::Encryptor,
            SoftwareType::Decryptor,
            SoftwareType::Anymap,
            SoftwareType::CryptoKey,
            SoftwareType::VirusSpyware,
        ]
    }
    
    pub fn virus_types() -> &'static [SoftwareType] {
        &[SoftwareType::VirusSpyware]
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            SoftwareType::Cracker => "cracker",
            SoftwareType::Firewall => "firewall",
            SoftwareType::Text => "text",
            SoftwareType::Exploit => "exploit",
            SoftwareType::Hasher => "hasher",
            SoftwareType::LogForger => "log_forger",
            SoftwareType::LogRecover => "log_recover",
            SoftwareType::Encryptor => "encryptor",
            SoftwareType::Decryptor => "decryptor",
            SoftwareType::Anymap => "anymap",
            SoftwareType::CryptoKey => "crypto_key",
            SoftwareType::VirusSpyware => "virus_spyware",
        }
    }
    
    pub fn is_virus(&self) -> bool {
        matches!(self, SoftwareType::VirusSpyware)
    }
    
    pub fn is_tool(&self) -> bool {
        !self.is_virus() && !matches!(self, SoftwareType::Text | SoftwareType::CryptoKey)
    }
}

impl std::fmt::Display for SoftwareType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Software extension enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoftwareExtension {
    /// Cracker extension
    Crc,
    /// Firewall extension
    Fwl,
    /// Text file extension
    Txt,
    /// Exploit extension
    Exp,
    /// Hash extension
    Hash,
    /// Log forger extension
    Logf,
    /// Log recover extension
    Logr,
    /// Encryptor extension
    Enc,
    /// Decryptor extension
    Dec,
    /// Map extension
    Map,
    /// Crypto key extension
    Key,
    /// Spyware extension
    Spy,
}

impl SoftwareExtension {
    pub fn as_str(&self) -> &'static str {
        match self {
            SoftwareExtension::Crc => "crc",
            SoftwareExtension::Fwl => "fwl",
            SoftwareExtension::Txt => "txt",
            SoftwareExtension::Exp => "exp",
            SoftwareExtension::Hash => "hash",
            SoftwareExtension::Logf => "logf",
            SoftwareExtension::Logr => "logr",
            SoftwareExtension::Enc => "enc",
            SoftwareExtension::Dec => "dec",
            SoftwareExtension::Map => "map",
            SoftwareExtension::Key => "key",
            SoftwareExtension::Spy => "spy",
        }
    }
    
    pub fn for_software_type(software_type: SoftwareType) -> Self {
        match software_type {
            SoftwareType::Cracker => SoftwareExtension::Crc,
            SoftwareType::Firewall => SoftwareExtension::Fwl,
            SoftwareType::Text => SoftwareExtension::Txt,
            SoftwareType::Exploit => SoftwareExtension::Exp,
            SoftwareType::Hasher => SoftwareExtension::Hash,
            SoftwareType::LogForger => SoftwareExtension::Logf,
            SoftwareType::LogRecover => SoftwareExtension::Logr,
            SoftwareType::Encryptor => SoftwareExtension::Enc,
            SoftwareType::Decryptor => SoftwareExtension::Dec,
            SoftwareType::Anymap => SoftwareExtension::Map,
            SoftwareType::CryptoKey => SoftwareExtension::Key,
            SoftwareType::VirusSpyware => SoftwareExtension::Spy,
        }
    }
}

impl std::fmt::Display for SoftwareExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Software module enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoftwareModule {
    // Cracker modules
    Bruteforce,
    Overflow,
    
    // Firewall modules
    FwlActive,
    FwlPassive,
    
    // Exploit modules
    Ftp,
    Ssh,
    
    // Hasher modules
    Password,
    
    // Log forger modules
    LogCreate,
    LogEdit,
    
    // Log recover modules
    LogRecover,
    
    // Encryptor modules
    EncFile,
    EncLog,
    EncConn,
    EncProcess,
    
    // Decryptor modules
    DecFile,
    DecLog,
    DecConn,
    DecProcess,
    
    // Anymap modules
    MapGeo,
    MapNet,
    
    // Spyware modules
    VirSpyware,
}

impl SoftwareModule {
    pub fn as_str(&self) -> &'static str {
        match self {
            SoftwareModule::Bruteforce => "bruteforce",
            SoftwareModule::Overflow => "overflow",
            SoftwareModule::FwlActive => "fwl_active",
            SoftwareModule::FwlPassive => "fwl_passive",
            SoftwareModule::Ftp => "ftp",
            SoftwareModule::Ssh => "ssh",
            SoftwareModule::Password => "password",
            SoftwareModule::LogCreate => "log_create",
            SoftwareModule::LogEdit => "log_edit",
            SoftwareModule::LogRecover => "log_recover",
            SoftwareModule::EncFile => "enc_file",
            SoftwareModule::EncLog => "enc_log",
            SoftwareModule::EncConn => "enc_conn",
            SoftwareModule::EncProcess => "enc_process",
            SoftwareModule::DecFile => "dec_file",
            SoftwareModule::DecLog => "dec_log",
            SoftwareModule::DecConn => "dec_conn",
            SoftwareModule::DecProcess => "dec_process",
            SoftwareModule::MapGeo => "map_geo",
            SoftwareModule::MapNet => "map_net",
            SoftwareModule::VirSpyware => "vir_spyware",
        }
    }
    
    pub fn for_software_type(software_type: SoftwareType) -> Vec<Self> {
        match software_type {
            SoftwareType::Cracker => vec![SoftwareModule::Bruteforce, SoftwareModule::Overflow],
            SoftwareType::Firewall => vec![SoftwareModule::FwlActive, SoftwareModule::FwlPassive],
            SoftwareType::Exploit => vec![SoftwareModule::Ftp, SoftwareModule::Ssh],
            SoftwareType::Hasher => vec![SoftwareModule::Password],
            SoftwareType::LogForger => vec![SoftwareModule::LogCreate, SoftwareModule::LogEdit],
            SoftwareType::LogRecover => vec![SoftwareModule::LogRecover],
            SoftwareType::Encryptor => vec![
                SoftwareModule::EncFile,
                SoftwareModule::EncLog,
                SoftwareModule::EncConn,
                SoftwareModule::EncProcess,
            ],
            SoftwareType::Decryptor => vec![
                SoftwareModule::DecFile,
                SoftwareModule::DecLog,
                SoftwareModule::DecConn,
                SoftwareModule::DecProcess,
            ],
            SoftwareType::Anymap => vec![SoftwareModule::MapGeo, SoftwareModule::MapNet],
            SoftwareType::VirusSpyware => vec![SoftwareModule::VirSpyware],
            SoftwareType::Text | SoftwareType::CryptoKey => vec![], // No modules
        }
    }
}

impl std::fmt::Display for SoftwareModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// File type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileType {
    /// Software executable
    Software,
    /// Text document
    Text,
    /// Cryptographic key
    CryptoKey,
    /// Virus file
    Virus,
}

impl FileType {
    pub fn all_types() -> &'static [FileType] {
        &[
            FileType::Software,
            FileType::Text,
            FileType::CryptoKey,
            FileType::Virus,
        ]
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            FileType::Software => "software",
            FileType::Text => "text",
            FileType::CryptoKey => "crypto_key",
            FileType::Virus => "virus",
        }
    }
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Virus state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VirusState {
    /// Virus is dormant/inactive
    Dormant,
    /// Virus is actively running
    Active,
    /// Virus is collecting data
    Collecting,
    /// Virus has been detected/quarantined
    Detected,
}

impl VirusState {
    pub fn as_str(&self) -> &'static str {
        match self {
            VirusState::Dormant => "dormant",
            VirusState::Active => "active",
            VirusState::Collecting => "collecting",
            VirusState::Detected => "detected",
        }
    }
}

impl std::fmt::Display for VirusState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Software creation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareCreationParams {
    pub software_type: SoftwareType,
    pub name: String,
    pub version: Option<String>,
    pub modules: Vec<SoftwareModule>,
    pub size: u64,
}

/// File creation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCreationParams {
    pub file_type: FileType,
    pub name: String,
    pub content: Vec<u8>,
    pub server_id: ServerId,
    pub storage_id: StorageId,
    pub path: String,
}

/// Virus creation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirusCreationParams {
    pub virus_type: SoftwareType,
    pub name: String,
    pub target_server_id: ServerId,
    pub collection_target: Option<String>,
}

/// Software information structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoftwareInfo {
    pub software_type: SoftwareType,
    pub extension: SoftwareExtension,
    pub modules: Vec<SoftwareModule>,
}

impl SoftwareInfo {
    pub fn for_type(software_type: SoftwareType) -> Self {
        Self {
            software_type,
            extension: SoftwareExtension::for_software_type(software_type),
            modules: SoftwareModule::for_software_type(software_type),
        }
    }
}

/// File metadata for storage and organization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMetadata {
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub accessed_at: DateTime<Utc>,
    pub size: u64,
    pub checksum: Option<String>,
    pub encrypted: bool,
    pub compressed: bool,
}

impl Default for FileMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            modified_at: now,
            accessed_at: now,
            size: 0,
            checksum: None,
            encrypted: false,
            compressed: false,
        }
    }
}

/// Storage capacity and usage information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageInfo {
    pub total_capacity: u64,
    pub used_capacity: u64,
    pub available_capacity: u64,
    pub file_count: u32,
}

impl StorageInfo {
    pub fn usage_percentage(&self) -> f64 {
        if self.total_capacity == 0 {
            0.0
        } else {
            (self.used_capacity as f64 / self.total_capacity as f64) * 100.0
        }
    }
    
    pub fn is_full(&self) -> bool {
        self.available_capacity == 0
    }
}

/// Crypto key information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CryptoKeyInfo {
    pub algorithm: String,
    pub key_size: u32,
    pub purpose: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Virus collection data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirusCollectionData {
    pub collected_at: DateTime<Utc>,
    pub data_type: String,
    pub data_size: u64,
    pub data_content: serde_json::Value,
}