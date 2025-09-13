//! Event state management and supervision

use tokio::sync::RwLock;
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;

/// Event state supervisor
pub struct EventStateSupervisor {
    state: Arc<RwLock<EventState>>,
    timers: Arc<RwLock<HashMap<Uuid, EventTimer>>>,
}

/// Event system state
#[derive(Default)]
struct EventState {
    active_events: u64,
    processed_events: u64,
    error_count: u64,
}

/// Event timer for delayed/scheduled events
pub struct EventTimer {
    pub timer_id: Uuid,
    pub event_type: String,
    pub scheduled_at: chrono::DateTime<chrono::Utc>,
    pub data: serde_json::Value,
}

impl EventStateSupervisor {
    /// Create a new event state supervisor
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(EventState::default())),
            timers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get event statistics
    pub async fn get_stats(&self) -> (u64, u64, u64) {
        let state = self.state.read().await;
        (state.active_events, state.processed_events, state.error_count)
    }

    /// Schedule an event timer
    pub async fn schedule_event(&self, timer: EventTimer) {
        let mut timers = self.timers.write().await;
        timers.insert(timer.timer_id, timer);
    }
}

impl Default for EventStateSupervisor {
    fn default() -> Self {
        Self::new()
    }
}