//! WebSocket event types for game updates

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Game events that can be sent via WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum GameEvent {
    // Process events
    ProcessStarted {
        pid: i64,
        process_type: String,
        estimated_time: u64,
    },
    ProcessCompleted {
        pid: i64,
        process_type: String,
        result: String,
    },
    ProcessCancelled {
        pid: i64,
    },
    ProcessProgress {
        pid: i64,
        progress: f32,
        remaining_time: u64,
    },

    // Hardware events
    HardwareUpgraded {
        component: String,
        old_level: u32,
        new_level: u32,
    },
    HardwareOverloaded {
        load_percentage: f32,
    },

    // Bank events
    MoneyReceived {
        amount: i64,
        from: String,
    },
    MoneySent {
        amount: i64,
        to: String,
    },
    BankHacked {
        hacker: String,
        amount: i64,
    },

    // Mission events
    MissionCompleted {
        mission_id: i64,
        reward_money: i64,
        reward_xp: i32,
    },
    MissionFailed {
        mission_id: i64,
        reason: String,
    },
    MissionProgress {
        mission_id: i64,
        progress: i32,
        total: i32,
    },

    // Attack events
    UnderAttack {
        attacker: String,
        attack_type: String,
    },
    AttackBlocked {
        attacker: String,
    },
    SystemCompromised {
        attacker: String,
        damage: String,
    },

    // Log events
    LogCreated {
        log_type: String,
        message: String,
        ip: String,
    },
    LogDeleted {
        count: usize,
    },

    // Virus events
    VirusInstalled {
        target_pc: String,
        virus_type: String,
    },
    VirusDetected {
        virus_type: String,
    },
    VirusRemoved {
        virus_type: String,
    },

    // Chat/Message events
    MessageReceived {
        from: String,
        content: String,
        timestamp: String,
    },
    ClanMessage {
        from: String,
        content: String,
        clan: String,
    },

    // System events
    ServerRestart {
        time_until: u64,
    },
    MaintenanceMode {
        enabled: bool,
        message: String,
    },
    Announcement {
        title: String,
        content: String,
        priority: String,
    },

    // Connection events
    UserOnline {
        username: String,
    },
    UserOffline {
        username: String,
    },

    // Custom event
    Custom {
        event_name: String,
        payload: Value,
    },
}

impl GameEvent {
    /// Convert to ServerMessage format
    pub fn to_server_message(&self) -> crate::ServerMessage {
        crate::ServerMessage {
            event_type: self.event_type(),
            data: serde_json::to_value(self).unwrap_or(serde_json::json!({})),
        }
    }

    /// Get the event type as string
    pub fn event_type(&self) -> String {
        match self {
            GameEvent::ProcessStarted { .. } => "process_started",
            GameEvent::ProcessCompleted { .. } => "process_completed",
            GameEvent::ProcessCancelled { .. } => "process_cancelled",
            GameEvent::ProcessProgress { .. } => "process_progress",
            GameEvent::HardwareUpgraded { .. } => "hardware_upgraded",
            GameEvent::HardwareOverloaded { .. } => "hardware_overloaded",
            GameEvent::MoneyReceived { .. } => "money_received",
            GameEvent::MoneySent { .. } => "money_sent",
            GameEvent::BankHacked { .. } => "bank_hacked",
            GameEvent::MissionCompleted { .. } => "mission_completed",
            GameEvent::MissionFailed { .. } => "mission_failed",
            GameEvent::MissionProgress { .. } => "mission_progress",
            GameEvent::UnderAttack { .. } => "under_attack",
            GameEvent::AttackBlocked { .. } => "attack_blocked",
            GameEvent::SystemCompromised { .. } => "system_compromised",
            GameEvent::LogCreated { .. } => "log_created",
            GameEvent::LogDeleted { .. } => "log_deleted",
            GameEvent::VirusInstalled { .. } => "virus_installed",
            GameEvent::VirusDetected { .. } => "virus_detected",
            GameEvent::VirusRemoved { .. } => "virus_removed",
            GameEvent::MessageReceived { .. } => "message_received",
            GameEvent::ClanMessage { .. } => "clan_message",
            GameEvent::ServerRestart { .. } => "server_restart",
            GameEvent::MaintenanceMode { .. } => "maintenance_mode",
            GameEvent::Announcement { .. } => "announcement",
            GameEvent::UserOnline { .. } => "user_online",
            GameEvent::UserOffline { .. } => "user_offline",
            GameEvent::Custom { event_name, .. } => event_name,
        }
        .to_string()
    }

    /// Check if event should be broadcast to all users
    pub fn is_broadcast(&self) -> bool {
        matches!(
            self,
            GameEvent::ServerRestart { .. }
                | GameEvent::MaintenanceMode { .. }
                | GameEvent::Announcement { .. }
        )
    }
}

/// Event builder for convenient event creation
pub struct EventBuilder;

impl EventBuilder {
    pub fn process_started(pid: i64, process_type: String, estimated_time: u64) -> GameEvent {
        GameEvent::ProcessStarted {
            pid,
            process_type,
            estimated_time,
        }
    }

    pub fn process_completed(pid: i64, process_type: String, result: String) -> GameEvent {
        GameEvent::ProcessCompleted {
            pid,
            process_type,
            result,
        }
    }

    pub fn money_received(amount: i64, from: String) -> GameEvent {
        GameEvent::MoneyReceived { amount, from }
    }

    pub fn under_attack(attacker: String, attack_type: String) -> GameEvent {
        GameEvent::UnderAttack {
            attacker,
            attack_type,
        }
    }

    pub fn announcement(title: String, content: String, priority: String) -> GameEvent {
        GameEvent::Announcement {
            title,
            content,
            priority,
        }
    }
}