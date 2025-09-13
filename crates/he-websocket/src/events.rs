use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Types of events that can be sent through WebSocket channels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload")]
pub enum GameEvent {
    /// Process-related events
    ProcessStarted {
        process_id: Uuid,
        process_type: String,
        target_id: Option<String>,
        estimated_completion: i64,
    },
    ProcessCompleted {
        process_id: Uuid,
        result: ProcessResult,
    },
    ProcessFailed {
        process_id: Uuid,
        error: String,
    },
    ProcessProgress {
        process_id: Uuid,
        progress: f32,
        remaining_time: i64,
    },

    /// Network/Server events
    ServerConnected {
        server_id: String,
        hostname: String,
        ip: String,
    },
    ServerDisconnected {
        server_id: String,
        reason: String,
    },
    NetworkScanResult {
        target_ip: String,
        open_ports: Vec<u16>,
        services: HashMap<u16, String>,
    },

    /// Log events
    LogEntryAdded {
        server_id: String,
        log_id: Uuid,
        log_type: String,
        message: String,
        timestamp: i64,
    },
    LogCleared {
        server_id: String,
        log_type: String,
    },

    /// File system events
    FileCreated {
        server_id: String,
        path: String,
        file_type: String,
        size: u64,
    },
    FileDeleted {
        server_id: String,
        path: String,
    },
    FileModified {
        server_id: String,
        path: String,
        new_size: u64,
    },
    DirectoryListed {
        server_id: String,
        path: String,
        contents: Vec<FileInfo>,
    },

    /// Mission/Campaign events
    MissionStarted {
        mission_id: Uuid,
        title: String,
        objectives: Vec<String>,
    },
    MissionCompleted {
        mission_id: Uuid,
        rewards: MissionRewards,
    },
    ObjectiveCompleted {
        mission_id: Uuid,
        objective_id: String,
    },

    /// Player/Account events
    ExperienceGained {
        skill: String,
        amount: u32,
        new_level: Option<u32>,
    },
    MoneyChanged {
        old_amount: u64,
        new_amount: u64,
        reason: String,
    },
    NotificationReceived {
        notification_id: Uuid,
        title: String,
        message: String,
        notification_type: String,
        expires_at: Option<i64>,
    },

    /// System events
    SystemStatus {
        cpu_usage: f32,
        memory_usage: f32,
        active_processes: u32,
        uptime: u64,
    },
    Maintenance {
        message: String,
        scheduled_time: i64,
        duration: u64,
    },

    /// Chat/Social events
    ChatMessage {
        channel: String,
        sender: String,
        message: String,
        timestamp: i64,
    },
    UserOnline {
        username: String,
    },
    UserOffline {
        username: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub file_type: String,
    pub size: u64,
    pub modified: i64,
    pub permissions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionRewards {
    pub experience: HashMap<String, u32>,
    pub money: u64,
    pub items: Vec<String>,
}

/// WebSocket message structure similar to Phoenix channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub topic: String,
    pub event: String,
    pub payload: serde_json::Value,
    pub ref_id: Option<String>,
}

/// Channel events for managing subscriptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelEvent {
    Join { topic: String },
    Leave { topic: String },
    HeartBeat,
    HeartBeatReply,
    Error { reason: String },
    Close { reason: String },
}

/// Response types for channel operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelResponse {
    pub status: ChannelStatus,
    pub response: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelStatus {
    Ok,
    Error,
    Timeout,
}

/// Topic patterns for organizing channels
pub mod topics {
    use uuid::Uuid;

    /// User-specific topics
    pub fn user_channel(user_id: Uuid) -> String {
        format!("user:{}", user_id)
    }

    /// Server-specific topics
    pub fn server_channel(server_id: &str) -> String {
        format!("server:{}", server_id)
    }

    /// Process-specific topics
    pub fn process_channel(process_id: Uuid) -> String {
        format!("process:{}", process_id)
    }

    /// Mission-specific topics
    pub fn mission_channel(mission_id: Uuid) -> String {
        format!("mission:{}", mission_id)
    }

    /// Global channels
    pub const LOBBY: &str = "lobby:global";
    pub const SYSTEM: &str = "system:announcements";
    
    /// Chat channels
    pub fn chat_channel(channel_name: &str) -> String {
        format!("chat:{}", channel_name)
    }

    /// Network/faction channels
    pub fn network_channel(network_id: &str) -> String {
        format!("network:{}", network_id)
    }
}

impl WebSocketMessage {
    pub fn new(topic: String, event: String, payload: serde_json::Value) -> Self {
        Self {
            topic,
            event,
            payload,
            ref_id: None,
        }
    }

    pub fn with_ref(mut self, ref_id: String) -> Self {
        self.ref_id = Some(ref_id);
        self
    }

    pub fn from_game_event(topic: String, event: GameEvent) -> Result<Self, serde_json::Error> {
        let payload = serde_json::to_value(event.clone())?;
        let event_name = match event {
            GameEvent::ProcessStarted { .. } => "process_started",
            GameEvent::ProcessCompleted { .. } => "process_completed",
            GameEvent::ProcessFailed { .. } => "process_failed",
            GameEvent::ProcessProgress { .. } => "process_progress",
            GameEvent::ServerConnected { .. } => "server_connected",
            GameEvent::ServerDisconnected { .. } => "server_disconnected",
            GameEvent::NetworkScanResult { .. } => "network_scan_result",
            GameEvent::LogEntryAdded { .. } => "log_entry_added",
            GameEvent::LogCleared { .. } => "log_cleared",
            GameEvent::FileCreated { .. } => "file_created",
            GameEvent::FileDeleted { .. } => "file_deleted",
            GameEvent::FileModified { .. } => "file_modified",
            GameEvent::DirectoryListed { .. } => "directory_listed",
            GameEvent::MissionStarted { .. } => "mission_started",
            GameEvent::MissionCompleted { .. } => "mission_completed",
            GameEvent::ObjectiveCompleted { .. } => "objective_completed",
            GameEvent::ExperienceGained { .. } => "experience_gained",
            GameEvent::MoneyChanged { .. } => "money_changed",
            GameEvent::NotificationReceived { .. } => "notification_received",
            GameEvent::SystemStatus { .. } => "system_status",
            GameEvent::Maintenance { .. } => "maintenance",
            GameEvent::ChatMessage { .. } => "chat_message",
            GameEvent::UserOnline { .. } => "user_online",
            GameEvent::UserOffline { .. } => "user_offline",
        };

        Ok(Self::new(topic, event_name.to_string(), payload))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_message_creation() {
        let event = GameEvent::ProcessStarted {
            process_id: Uuid::new_v4(),
            process_type: "hack".to_string(),
            target_id: Some("192.168.1.1".to_string()),
            estimated_completion: 1234567890,
        };

        let message = WebSocketMessage::from_game_event(
            topics::user_channel(Uuid::new_v4()),
            event,
        ).unwrap();

        assert_eq!(message.event, "process_started");
        assert!(message.topic.starts_with("user:"));
    }

    #[test]
    fn test_topic_generation() {
        let user_id = Uuid::new_v4();
        let topic = topics::user_channel(user_id);
        assert_eq!(topic, format!("user:{}", user_id));

        let server_topic = topics::server_channel("test-server");
        assert_eq!(server_topic, "server:test-server");
    }
}