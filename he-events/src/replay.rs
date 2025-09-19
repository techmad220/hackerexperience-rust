//! Event replay system for debugging and recovery

use crate::event::{Event, EventType, EventCategory};
use crate::store::{EventStore, EventQuery, EventFilter, OrderBy, OrderDirection};
use crate::publisher::EventPublisher;
use he_core::{HelixError, HelixResult, HelixId};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use chrono::{DateTime, Utc};

/// Configuration for event replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayConfig {
    /// Speed multiplier for replay (1.0 = real time, 2.0 = 2x speed, etc.)
    pub speed_multiplier: f64,
    /// Maximum events to replay in a single batch
    pub batch_size: usize,
    /// Delay between batches
    pub batch_delay: Duration,
    /// Whether to preserve original timestamps during replay
    pub preserve_timestamps: bool,
    /// Maximum number of events to replay (None = no limit)
    pub max_events: Option<usize>,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            speed_multiplier: 1.0,
            batch_size: 100,
            batch_delay: Duration::from_millis(100),
            preserve_timestamps: false,
            max_events: None,
        }
    }
}

/// Event replay system
#[derive(Debug)]
pub struct EventReplaySystem {
    /// Event store to replay from
    store: Arc<EventStore>,
    /// Configuration
    config: ReplayConfig,
}

impl EventReplaySystem {
    /// Create a new replay system
    pub fn new(store: Arc<EventStore>, config: ReplayConfig) -> Self {
        Self { store, config }
    }

    /// Replay all events in the store
    pub async fn replay_all<P>(&self, publisher: Arc<P>) -> HelixResult<ReplayResult>
    where
        P: EventPublisher,
    {
        let query = EventQuery {
            order_by: OrderBy::Timestamp(OrderDirection::Ascending),
            limit: self.config.max_events,
            ..Default::default()
        };

        self.replay_query(query, publisher).await
    }

    /// Replay events matching a specific query
    pub async fn replay_query<P>(
        &self,
        query: EventQuery,
        publisher: Arc<P>,
    ) -> HelixResult<ReplayResult>
    where
        P: EventPublisher,
    {
        let events = self.store.query(&query).await?;
        self.replay_events(events, publisher).await
    }

    /// Replay events by type
    pub async fn replay_by_type<P>(
        &self,
        event_type: EventType,
        publisher: Arc<P>,
    ) -> HelixResult<ReplayResult>
    where
        P: EventPublisher,
    {
        let events = self.store.find_by_type(&event_type).await?;
        self.replay_events(events, publisher).await
    }

    /// Replay events by category
    pub async fn replay_by_category<P>(
        &self,
        category: EventCategory,
        publisher: Arc<P>,
    ) -> HelixResult<ReplayResult>
    where
        P: EventPublisher,
    {
        let events = self.store.find_by_category(&category).await?;
        self.replay_events(events, publisher).await
    }

    /// Replay events within a time range
    pub async fn replay_time_range<P>(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        publisher: Arc<P>,
    ) -> HelixResult<ReplayResult>
    where
        P: EventPublisher,
    {
        let events = self.store.find_by_time_range(start, end).await?;
        self.replay_events(events, publisher).await
    }

    /// Replay specific events by their IDs
    pub async fn replay_by_ids<P>(
        &self,
        event_ids: Vec<HelixId>,
        publisher: Arc<P>,
    ) -> HelixResult<ReplayResult>
    where
        P: EventPublisher,
    {
        let mut events = Vec::new();
        for id in event_ids {
            if let Some(event) = self.store.find_by_id(id).await? {
                events.push(event);
            }
        }

        // Sort by timestamp
        events.sort_by(|a, b| a.metadata.timestamp.cmp(&b.metadata.timestamp));
        
        self.replay_events(events, publisher).await
    }

    /// Replay a list of events
    async fn replay_events<P>(
        &self,
        events: Vec<Event>,
        publisher: Arc<P>,
    ) -> HelixResult<ReplayResult>
    where
        P: EventPublisher,
    {
        if events.is_empty() {
            return Ok(ReplayResult {
                total_events: 0,
                successful_events: 0,
                failed_events: 0,
                start_time: Utc::now(),
                end_time: Utc::now(),
                errors: Vec::new(),
            });
        }

        let start_time = Utc::now();
        let total_events = events.len();
        let mut successful_events = 0;
        let mut failed_events = 0;
        let mut errors = Vec::new();

        tracing::info!(
            "Starting replay of {} events with config: {:?}",
            total_events,
            self.config
        );

        // Process events in batches
        for (batch_index, batch) in events.chunks(self.config.batch_size).enumerate() {
            tracing::debug!("Processing batch {} with {} events", batch_index, batch.len());

            // Calculate timing if preserving timestamps
            if self.config.preserve_timestamps && batch.len() > 1 {
                let first_timestamp = batch[0].metadata.timestamp;
                
                for (i, event) in batch.iter().enumerate() {
                    if i > 0 {
                        let prev_timestamp = batch[i - 1].metadata.timestamp;
                        let current_timestamp = event.metadata.timestamp;
                        
                        if current_timestamp > prev_timestamp {
                            let original_delay = current_timestamp - prev_timestamp;
                            let replay_delay = Duration::from_millis(
                                (original_delay.num_milliseconds() as f64 / self.config.speed_multiplier) as u64
                            );
                            
                            if replay_delay > Duration::from_millis(0) {
                                sleep(replay_delay).await;
                            }
                        }
                    }

                    // Replay the event
                    match self.replay_single_event(event.clone(), &publisher).await {
                        Ok(()) => successful_events += 1,
                        Err(e) => {
                            failed_events += 1;
                            errors.push(ReplayError {
                                event_id: event.id,
                                event_type: event.event_type.clone(),
                                error: e.to_string(),
                                timestamp: Utc::now(),
                            });
                            tracing::error!(
                                "Failed to replay event {}: {}",
                                event.id,
                                e
                            );
                        }
                    }
                }
            } else {
                // Replay batch without timestamp preservation
                for event in batch {
                    match self.replay_single_event(event.clone(), &publisher).await {
                        Ok(()) => successful_events += 1,
                        Err(e) => {
                            failed_events += 1;
                            errors.push(ReplayError {
                                event_id: event.id,
                                event_type: event.event_type.clone(),
                                error: e.to_string(),
                                timestamp: Utc::now(),
                            });
                            tracing::error!(
                                "Failed to replay event {}: {}",
                                event.id,
                                e
                            );
                        }
                    }
                }
            }

            // Delay between batches
            if batch_index < events.chunks(self.config.batch_size).count() - 1 {
                sleep(self.config.batch_delay).await;
            }
        }

        let end_time = Utc::now();
        let result = ReplayResult {
            total_events,
            successful_events,
            failed_events,
            start_time,
            end_time,
            errors,
        };

        tracing::info!(
            "Replay completed: {}/{} events successful, {} failed, duration: {}ms",
            successful_events,
            total_events,
            failed_events,
            (end_time - start_time).num_milliseconds()
        );

        Ok(result)
    }

    /// Replay a single event
    async fn replay_single_event<P>(
        &self,
        mut event: Event,
        publisher: &P,
    ) -> HelixResult<()>
    where
        P: EventPublisher,
    {
        // Generate new ID for replayed event to avoid conflicts
        event.id = HelixId::new_v4();
        
        // Update timestamp if not preserving original
        if !self.config.preserve_timestamps {
            event.metadata.timestamp = Utc::now();
        }

        // Add replay metadata
        event.metadata.custom.insert(
            "replayed".to_string(),
            serde_json::Value::Bool(true),
        );
        event.metadata.custom.insert(
            "replay_timestamp".to_string(),
            serde_json::Value::String(Utc::now().to_rfc3339()),
        );

        publisher.publish(event).await
    }
}

/// Result of an event replay operation
#[derive(Debug, Clone)]
pub struct ReplayResult {
    /// Total number of events attempted to replay
    pub total_events: usize,
    /// Number of successfully replayed events
    pub successful_events: usize,
    /// Number of failed event replays
    pub failed_events: usize,
    /// When the replay started
    pub start_time: DateTime<Utc>,
    /// When the replay completed
    pub end_time: DateTime<Utc>,
    /// Errors that occurred during replay
    pub errors: Vec<ReplayError>,
}

impl ReplayResult {
    /// Get the success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_events == 0 {
            100.0
        } else {
            (self.successful_events as f64 / self.total_events as f64) * 100.0
        }
    }

    /// Get the duration of the replay
    pub fn duration(&self) -> chrono::Duration {
        self.end_time - self.start_time
    }

    /// Check if the replay was completely successful
    pub fn is_successful(&self) -> bool {
        self.failed_events == 0
    }
}

/// Error that occurred during event replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayError {
    /// ID of the event that failed to replay
    pub event_id: HelixId,
    /// Type of the event that failed
    pub event_type: EventType,
    /// Error message
    pub error: String,
    /// When the error occurred
    pub timestamp: DateTime<Utc>,
}

/// Event replay builder for easy configuration
pub struct EventReplayBuilder {
    config: ReplayConfig,
}

impl EventReplayBuilder {
    /// Create a new replay builder
    pub fn new() -> Self {
        Self {
            config: ReplayConfig::default(),
        }
    }

    /// Set replay speed multiplier
    pub fn speed(mut self, multiplier: f64) -> Self {
        self.config.speed_multiplier = multiplier;
        self
    }

    /// Set batch size
    pub fn batch_size(mut self, size: usize) -> Self {
        self.config.batch_size = size;
        self
    }

    /// Set batch delay
    pub fn batch_delay(mut self, delay: Duration) -> Self {
        self.config.batch_delay = delay;
        self
    }

    /// Preserve original timestamps
    pub fn preserve_timestamps(mut self, preserve: bool) -> Self {
        self.config.preserve_timestamps = preserve;
        self
    }

    /// Set maximum number of events to replay
    pub fn max_events(mut self, max: usize) -> Self {
        self.config.max_events = Some(max);
        self
    }

    /// Build the replay system
    pub fn build(self, store: Arc<EventStore>) -> EventReplaySystem {
        EventReplaySystem::new(store, self.config)
    }
}

impl Default for EventReplayBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot replay - replay events to rebuild system state
pub struct SnapshotReplay {
    replay_system: EventReplaySystem,
}

impl SnapshotReplay {
    /// Create a new snapshot replay
    pub fn new(store: Arc<EventStore>) -> Self {
        let config = ReplayConfig {
            speed_multiplier: f64::INFINITY, // As fast as possible
            batch_size: 1000,
            batch_delay: Duration::from_millis(0),
            preserve_timestamps: false,
            max_events: None,
        };

        Self {
            replay_system: EventReplaySystem::new(store, config),
        }
    }

    /// Replay events up to a specific point in time
    pub async fn replay_to_time<P>(
        &self,
        target_time: DateTime<Utc>,
        publisher: Arc<P>,
    ) -> HelixResult<ReplayResult>
    where
        P: EventPublisher,
    {
        // Get all events up to the target time
        let query = EventQuery {
            time_range: Some((DateTime::UNIX_EPOCH, target_time)),
            order_by: OrderBy::Timestamp(OrderDirection::Ascending),
            ..Default::default()
        };

        self.replay_system.replay_query(query, publisher).await
    }

    /// Replay events up to a specific event
    pub async fn replay_to_event<P>(
        &self,
        target_event_id: HelixId,
        publisher: Arc<P>,
    ) -> HelixResult<ReplayResult>
    where
        P: EventPublisher,
    {
        // Find the target event to get its timestamp
        if let Some(target_event) = self.replay_system.store.find_by_id(target_event_id).await? {
            self.replay_to_time(target_event.metadata.timestamp, publisher).await
        } else {
            Err(HelixError::not_found(format!(
                "Target event not found: {}",
                target_event_id
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Event, EventType, EventData};
    use crate::store::{EventStore, EventStoreConfig};
    use crate::publisher::{EventPublisher, DispatcherEventPublisher, PublishConfig};
    use crate::dispatcher::{EventDispatcher, DispatchConfig};

    #[tokio::test]
    async fn test_event_replay() {
        let store_config = EventStoreConfig::default();
        let store = Arc::new(EventStore::new(store_config).await.unwrap());

        // Store some events
        let event1 = Event::new(
            EventType::SystemStarted,
            EventData::SystemStatus {
                status: "started".to_string(),
                details: serde_json::Value::Null,
            },
        );
        let event2 = Event::new(
            EventType::SystemStopped,
            EventData::SystemStatus {
                status: "stopped".to_string(),
                details: serde_json::Value::Null,
            },
        );

        store.store(event1).await.unwrap();
        store.store(event2).await.unwrap();

        // Create replay system
        let replay_config = ReplayConfig::default();
        let replay_system = EventReplaySystem::new(Arc::clone(&store), replay_config);

        // Create publisher for replay
        let dispatcher = Arc::new(EventDispatcher::new(DispatchConfig::default()).await.unwrap());
        let publish_config = PublishConfig::default();
        let publisher = Arc::new(DispatcherEventPublisher::new(publish_config, dispatcher));

        // Replay all events
        let result = replay_system.replay_all(publisher).await.unwrap();
        
        assert_eq!(result.total_events, 2);
        assert_eq!(result.successful_events, 2);
        assert_eq!(result.failed_events, 0);
        assert!(result.is_successful());
    }
}
