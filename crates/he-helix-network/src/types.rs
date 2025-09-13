//! Type definitions for the network module

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use uuid::Uuid;
use he_helix_server::ServerId;

/// Network unique identifier
pub type NetworkId = Uuid;

/// Tunnel unique identifier
pub type TunnelId = Uuid;

/// Connection unique identifier
pub type ConnectionId = Uuid;

/// Bounce unique identifier  
pub type BounceId = Uuid;

/// Link unique identifier
pub type LinkId = Uuid;

/// IP address type
pub type IpAddress = Ipv4Addr;

/// Network type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkType {
    /// Global internet network
    Internet,
    /// Story-specific networks
    Story,
    /// Mission-specific networks
    Mission,
    /// Local Area Networks
    Lan,
}

impl NetworkType {
    pub fn possible_types() -> &'static [NetworkType] {
        &[
            NetworkType::Internet,
            NetworkType::Story,
            NetworkType::Mission,
            NetworkType::Lan,
        ]
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            NetworkType::Internet => "internet",
            NetworkType::Story => "story",
            NetworkType::Mission => "mission",
            NetworkType::Lan => "lan",
        }
    }
}

impl std::fmt::Display for NetworkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Connection type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionType {
    /// Secure Shell connection
    Ssh,
    /// File Transfer Protocol
    Ftp,
    /// Public FTP access
    PublicFtp,
    /// Bank login connection
    BankLogin,
    /// Wire transfer connection
    WireTransfer,
    /// Virus collection connection
    VirusCollect,
    /// Brute force cracking connection
    CrackerBruteforce,
}

impl ConnectionType {
    pub fn all_types() -> &'static [ConnectionType] {
        &[
            ConnectionType::Ssh,
            ConnectionType::Ftp,
            ConnectionType::PublicFtp,
            ConnectionType::BankLogin,
            ConnectionType::WireTransfer,
            ConnectionType::VirusCollect,
            ConnectionType::CrackerBruteforce,
        ]
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            ConnectionType::Ssh => "ssh",
            ConnectionType::Ftp => "ftp",
            ConnectionType::PublicFtp => "public_ftp",
            ConnectionType::BankLogin => "bank_login",
            ConnectionType::WireTransfer => "wire_transfer",
            ConnectionType::VirusCollect => "virus_collect",
            ConnectionType::CrackerBruteforce => "cracker_bruteforce",
        }
    }
}

impl std::fmt::Display for ConnectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Connection close reasons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloseReason {
    /// Normal connection termination
    Normal,
    /// Forced connection termination
    Force,
}

impl CloseReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            CloseReason::Normal => "normal",
            CloseReason::Force => "force",
        }
    }
}

/// Network creation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkCreationParams {
    pub name: String,
    pub network_type: NetworkType,
}

/// Tunnel creation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelCreationParams {
    pub network_id: NetworkId,
    pub gateway_id: ServerId,
    pub target_id: ServerId,
    pub bounce_id: Option<BounceId>,
}

/// Connection creation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionCreationParams {
    pub tunnel_id: TunnelId,
    pub connection_type: ConnectionType,
    pub meta: Option<ConnectionMeta>,
}

/// Connection metadata for storing type-specific information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionMeta {
    pub data: std::collections::HashMap<String, serde_json::Value>,
}

impl ConnectionMeta {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }

    pub fn with_data(data: std::collections::HashMap<String, serde_json::Value>) -> Self {
        Self { data }
    }

    pub fn insert<V: serde::Serialize>(&mut self, key: String, value: V) -> Result<(), serde_json::Error> {
        let json_value = serde_json::to_value(value)?;
        self.data.insert(key, json_value);
        Ok(())
    }

    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>, serde_json::Error> {
        match self.data.get(key) {
            Some(value) => Ok(Some(serde_json::from_value(value.clone())?)),
            None => Ok(None),
        }
    }
}

impl Default for ConnectionMeta {
    fn default() -> Self {
        Self::new()
    }
}

/// Bounce link representing a single hop in a bounce chain
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BounceLink {
    pub link_id: LinkId,
    pub server_id: ServerId,
    pub network_id: NetworkId,
    pub ip: IpAddress,
    /// Order in the bounce chain (0-based)
    pub sequence: u32,
}

/// Network and IP address pairing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkIp {
    pub network_id: NetworkId,
    pub ip: IpAddress,
}

/// Tunnel creation errors
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TunnelCreationError {
    /// Tunnel would create a cycle
    CyclicTunnel,
    /// Internal error during creation
    Internal,
}

/// Gateway endpoint mapping for tunnels
pub type GatewayEndpoints = std::collections::HashMap<ServerId, TunnelId>;

/// DNS record types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DnsRecordType {
    A,
    Aaaa,
    Cname,
    Mx,
    Ns,
    Txt,
}

impl DnsRecordType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DnsRecordType::A => "A",
            DnsRecordType::Aaaa => "AAAA",
            DnsRecordType::Cname => "CNAME",
            DnsRecordType::Mx => "MX",
            DnsRecordType::Ns => "NS",
            DnsRecordType::Txt => "TXT",
        }
    }
}

/// DNS record for domain name resolution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DnsRecord {
    pub domain: String,
    pub record_type: DnsRecordType,
    pub value: String,
    pub ttl: u32,
}

/// Network statistics and metrics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkStats {
    pub network_id: NetworkId,
    pub total_servers: u64,
    pub active_tunnels: u64,
    pub active_connections: u64,
    pub total_bandwidth_usage: u64,
    pub last_updated: DateTime<Utc>,
}

/// Connection bandwidth usage information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BandwidthUsage {
    pub connection_id: ConnectionId,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub start_time: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}