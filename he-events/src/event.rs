//! Core event types and structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use he_core::{HelixId, RequestId, ProcessId};
use he_core::id::{EntityId, ServerId, AccountId, NetworkId};

/// Core event structure that flows through the Helix system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique identifier for this event
    pub id: HelixId,
    /// Type of event
    pub event_type: EventType,
    /// Event payload data
    pub data: EventData,
    /// Event metadata
    pub metadata: EventMetadata,
}

impl Event {
    /// Create a new event
    pub fn new(event_type: EventType, data: EventData) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            data,
            metadata: EventMetadata::new(),
        }
    }

    /// Create a new event with specific metadata
    pub fn with_metadata(event_type: EventType, data: EventData, metadata: EventMetadata) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            data,
            metadata,
        }
    }

    /// Add correlation ID to link related events
    pub fn with_correlation_id(mut self, correlation_id: HelixId) -> Self {
        self.metadata.correlation_id = Some(correlation_id);
        self
    }

    /// Add causation ID to track event chains
    pub fn with_causation_id(mut self, causation_id: HelixId) -> Self {
        self.metadata.causation_id = Some(causation_id);
        self
    }

    /// Add request ID to track events within a request
    pub fn with_request_id(mut self, request_id: RequestId) -> Self {
        self.metadata.request_id = Some(request_id);
        self
    }

    /// Add process ID to track events from a specific process
    pub fn with_process_id(mut self, process_id: ProcessId) -> Self {
        self.metadata.process_id = Some(process_id);
        self
    }

    /// Add custom metadata
    pub fn with_custom_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.custom.insert(key, value);
        self
    }

    /// Get the age of this event
    pub fn age(&self) -> chrono::Duration {
        Utc::now() - self.metadata.timestamp
    }

    /// Check if this event is older than a given duration
    pub fn is_older_than(&self, duration: chrono::Duration) -> bool {
        self.age() > duration
    }
}

/// Event metadata for tracking and debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// When the event was created
    pub timestamp: DateTime<Utc>,
    /// Version of the event schema
    pub version: u32,
    /// Source that generated the event
    pub source: Option<String>,
    /// Correlation ID for grouping related events
    pub correlation_id: Option<HelixId>,
    /// Causation ID for tracking event chains
    pub causation_id: Option<HelixId>,
    /// Request ID if this event is part of a request
    pub request_id: Option<RequestId>,
    /// Process ID that generated the event
    pub process_id: Option<ProcessId>,
    /// Custom metadata fields
    pub custom: HashMap<String, serde_json::Value>,
}

impl EventMetadata {
    pub fn new() -> Self {
        Self {
            timestamp: Utc::now(),
            version: 1,
            source: None,
            correlation_id: None,
            causation_id: None,
            request_id: None,
            process_id: None,
            custom: HashMap::new(),
        }
    }

    pub fn with_source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }
}

impl Default for EventMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Event type enumeration for categorizing events
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    // System events
    SystemStarted,
    SystemStopped,
    SystemError,
    
    // Account events
    AccountCreated,
    AccountUpdated,
    AccountDeleted,
    AccountLoggedIn,
    AccountLoggedOut,
    
    // Server events
    ServerCreated,
    ServerUpdated,
    ServerDeleted,
    ServerOnline,
    ServerOffline,
    ServerCompromised,
    
    // Network events
    NetworkCreated,
    NetworkUpdated,
    NetworkDeleted,
    NetworkConnected,
    NetworkDisconnected,
    
    // Process events
    ProcessStarted,
    ProcessCompleted,
    ProcessFailed,
    ProcessKilled,
    
    // Security events
    IntrusionDetected,
    AccessGranted,
    AccessDenied,
    SecurityBreach,
    
    // Game events
    MissionStarted,
    MissionCompleted,
    MissionFailed,
    
    // Log events
    LogCreated,
    LogModified,
    LogDeleted,
    
    // Custom events
    Custom(String),
}

impl EventType {
    /// Get the category of this event type
    pub fn category(&self) -> EventCategory {
        match self {
            EventType::SystemStarted | EventType::SystemStopped | EventType::SystemError => {
                EventCategory::System
            }
            EventType::AccountCreated | EventType::AccountUpdated | EventType::AccountDeleted 
            | EventType::AccountLoggedIn | EventType::AccountLoggedOut => {
                EventCategory::Account
            }
            EventType::ServerCreated | EventType::ServerUpdated | EventType::ServerDeleted 
            | EventType::ServerOnline | EventType::ServerOffline | EventType::ServerCompromised => {
                EventCategory::Server
            }
            EventType::NetworkCreated | EventType::NetworkUpdated | EventType::NetworkDeleted 
            | EventType::NetworkConnected | EventType::NetworkDisconnected => {
                EventCategory::Network
            }
            EventType::ProcessStarted | EventType::ProcessCompleted | EventType::ProcessFailed 
            | EventType::ProcessKilled => {
                EventCategory::Process
            }
            EventType::IntrusionDetected | EventType::AccessGranted | EventType::AccessDenied 
            | EventType::SecurityBreach => {
                EventCategory::Security
            }
            EventType::MissionStarted | EventType::MissionCompleted | EventType::MissionFailed => {
                EventCategory::Game
            }
            EventType::LogCreated | EventType::LogModified | EventType::LogDeleted => {
                EventCategory::Log
            }
            EventType::Custom(_) => EventCategory::Custom,
        }
    }

    /// Check if this is a critical event that requires immediate attention
    pub fn is_critical(&self) -> bool {
        matches!(self, 
            EventType::SystemError 
            | EventType::ServerCompromised 
            | EventType::SecurityBreach
            | EventType::IntrusionDetected
        )
    }
}

/// Event category for high-level grouping
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventCategory {
    System,
    Account,
    Server,
    Network,
    Process,
    Security,
    Game,
    Log,
    Custom,
}

/// Event data payload - the actual content of the event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum EventData {
    // System data
    SystemStatus {
        status: String,
        details: serde_json::Value,
    },
    SystemError {
        error: String,
        stack_trace: Option<String>,
    },
    
    // Account data
    AccountData {
        account_id: AccountId,
        email: Option<String>,
        username: Option<String>,
        details: serde_json::Value,
    },
    
    // Server data
    ServerData {
        server_id: ServerId,
        account_id: AccountId,
        name: Option<String>,
        ip_address: Option<String>,
        details: serde_json::Value,
    },
    
    // Network data
    NetworkData {
        network_id: NetworkId,
        server_id: ServerId,
        name: Option<String>,
        network_type: Option<String>,
        details: serde_json::Value,
    },
    
    // Process data
    ProcessData {
        process_id: ProcessId,
        server_id: ServerId,
        process_name: String,
        status: String,
        details: serde_json::Value,
    },
    
    // Security data
    SecurityData {
        source_ip: Option<String>,
        target_ip: Option<String>,
        threat_level: u8,
        description: String,
        details: serde_json::Value,
    },
    
    // Game data
    GameData {
        entity_id: EntityId,
        game_action: String,
        result: String,
        details: serde_json::Value,
    },
    
    // Log data
    LogData {
        log_id: HelixId,
        level: String,
        message: String,
        source: String,
        details: serde_json::Value,
    },
    
    // Custom data for extensibility
    Custom {
        data_type: String,
        payload: serde_json::Value,
    },
}

impl EventData {
    /// Get the primary entity ID associated with this event data
    pub fn primary_entity_id(&self) -> Option<HelixId> {
        match self {
            EventData::AccountData { account_id, .. } => Some(account_id.0),
            EventData::ServerData { server_id, .. } => Some(server_id.0),
            EventData::NetworkData { network_id, .. } => Some(network_id.0),
            EventData::ProcessData { process_id, .. } => Some(process_id.0),
            EventData::GameData { entity_id, .. } => Some(entity_id.0),
            EventData::LogData { log_id, .. } => Some(*log_id),
            _ => None,
        }
    }

    /// Extract account ID if present in the event data
    pub fn account_id(&self) -> Option<AccountId> {
        match self {
            EventData::AccountData { account_id, .. } => Some(*account_id),
            EventData::ServerData { account_id, .. } => Some(*account_id),
            _ => None,
        }
    }

    /// Extract server ID if present in the event data
    pub fn server_id(&self) -> Option<ServerId> {
        match self {
            EventData::ServerData { server_id, .. } => Some(*server_id),
            EventData::NetworkData { server_id, .. } => Some(*server_id),
            EventData::ProcessData { server_id, .. } => Some(*server_id),
            _ => None,
        }
    }
}
