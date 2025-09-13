//! Common Helix types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Client Network IP type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientNip {
    pub network_id: Uuid,
    pub ip: String,
}

/// HE Types for various game entities
pub mod he_types {
    use super::*;

    pub type ClientNip = super::ClientNip;

    /// Server identifier with IP
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ServerNip {
        pub server_id: Uuid,
        pub network_id: Uuid, 
        pub ip: String,
    }

    /// Process identifier
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ProcessId {
        pub process_id: Uuid,
        pub server_id: Uuid,
    }

    /// File identifier
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FileId {
        pub file_id: Uuid,
        pub server_id: Uuid,
        pub path: String,
    }
}