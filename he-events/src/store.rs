//! Event store for persisting and querying events

use crate::event::{Event, EventType, EventCategory};
use crate::handler::EventHandler;
use crate::{EventStoreConfig, StorageBackend};
use he_helix_core::{HelixError, HelixResult, HelixId};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Event store for persisting events
#[derive(Debug, Clone)]
pub struct EventStore {
    /// Configuration
    config: EventStoreConfig,
    /// In-memory event storage
    memory_store: Arc<RwLock<VecDeque<Event>>>,
    /// Event indices for fast querying
    indices: Arc<RwLock<EventIndices>>,
}

impl EventStore {
    /// Create a new event store
    pub async fn new(config: EventStoreConfig) -> HelixResult<Self> {
        Ok(Self {
            config,
            memory_store: Arc::new(RwLock::new(VecDeque::new())),
            indices: Arc::new(RwLock::new(EventIndices::new())),
        })
    }

    /// Store an event
    pub async fn store(&self, event: Event) -> HelixResult<()> {
        // Store in memory
        {
            let mut store = self.memory_store.write().await;
            
            // Enforce memory limit
            while store.len() >= self.config.max_memory_events {
                if let Some(old_event) = store.pop_front() {
                    // Remove from indices
                    let mut indices = self.indices.write().await;
                    indices.remove_event(&old_event);
                }
            }
            
            store.push_back(event.clone());
        }

        // Update indices
        {
            let mut indices = self.indices.write().await;
            indices.add_event(&event);
        }

        // Persist to backend if enabled
        if self.config.enable_persistence {
            self.persist_event(&event).await?;
        }

        tracing::trace!("Stored event: {}", event.id);
        Ok(())
    }

    /// Query events with filters
    pub async fn query(&self, query: &EventQuery) -> HelixResult<Vec<Event>> {
        let store = self.memory_store.read().await;
        let mut results = Vec::new();

        for event in store.iter() {
            if self.matches_query(event, query) {
                results.push(event.clone());
            }
        }

        // Apply ordering
        match query.order_by {
            OrderBy::Timestamp(dir) => {
                results.sort_by(|a, b| match dir {
                    OrderDirection::Ascending => a.metadata.timestamp.cmp(&b.metadata.timestamp),
                    OrderDirection::Descending => b.metadata.timestamp.cmp(&a.metadata.timestamp),
                });
            }
            OrderBy::EventType => {
                results.sort_by(|a, b| format!("{:?}", a.event_type).cmp(&format!("{:?}", b.event_type)));
            }
        }

        // Apply pagination
        if let Some(offset) = query.offset {
            if offset < results.len() {
                results = results[offset..].to_vec();
            } else {
                results.clear();
            }
        }

        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// Find events by ID
    pub async fn find_by_id(&self, id: HelixId) -> HelixResult<Option<Event>> {
        let store = self.memory_store.read().await;
        Ok(store.iter().find(|event| event.id == id).cloned())
    }

    /// Find events by type
    pub async fn find_by_type(&self, event_type: &EventType) -> HelixResult<Vec<Event>> {
        let query = EventQuery {
            event_types: Some(vec![event_type.clone()]),
            ..Default::default()
        };
        self.query(&query).await
    }

    /// Find events by category
    pub async fn find_by_category(&self, category: &EventCategory) -> HelixResult<Vec<Event>> {
        let query = EventQuery {
            categories: Some(vec![category.clone()]),
            ..Default::default()
        };
        self.query(&query).await
    }

    /// Find events within a time range
    pub async fn find_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> HelixResult<Vec<Event>> {
        let query = EventQuery {
            time_range: Some((start, end)),
            ..Default::default()
        };
        self.query(&query).await
    }

    /// Get event statistics
    pub async fn stats(&self) -> EventStoreStats {
        let store = self.memory_store.read().await;
        let indices = self.indices.read().await;

        EventStoreStats {
            total_events: store.len(),
            events_by_type: indices.type_counts.clone(),
            events_by_category: indices.category_counts.clone(),
            oldest_event: store.front().map(|e| e.metadata.timestamp),
            newest_event: store.back().map(|e| e.metadata.timestamp),
        }
    }

    /// Clear all events
    pub async fn clear(&self) -> HelixResult<()> {
        {
            let mut store = self.memory_store.write().await;
            store.clear();
        }
        
        {
            let mut indices = self.indices.write().await;
            indices.clear();
        }

        tracing::info!("Event store cleared");
        Ok(())
    }

    /// Check if a query matches an event
    fn matches_query(&self, event: &Event, query: &EventQuery) -> bool {
        // Check event types
        if let Some(ref types) = query.event_types {
            if !types.contains(&event.event_type) {
                return false;
            }
        }

        // Check categories
        if let Some(ref categories) = query.categories {
            let event_category = event.event_type.category();
            if !categories.contains(&event_category) {
                return false;
            }
        }

        // Check time range
        if let Some((start, end)) = query.time_range {
            if event.metadata.timestamp < start || event.metadata.timestamp > end {
                return false;
            }
        }

        // Check correlation ID
        if let Some(ref correlation_id) = query.correlation_id {
            if event.metadata.correlation_id.as_ref() != Some(correlation_id) {
                return false;
            }
        }

        // Check request ID
        if let Some(ref request_id) = query.request_id {
            if event.metadata.request_id.as_ref() != Some(request_id) {
                return false;
            }
        }

        // Check process ID
        if let Some(ref process_id) = query.process_id {
            if event.metadata.process_id.as_ref() != Some(process_id) {
                return false;
            }
        }

        true
    }

    /// Persist an event to the configured backend
    async fn persist_event(&self, event: &Event) -> HelixResult<()> {
        match self.config.storage_backend {
            StorageBackend::Memory => {
                // Already stored in memory, nothing to do
                Ok(())
            }
            StorageBackend::Database => {
                // TODO: Implement database persistence
                tracing::warn!("Database persistence not yet implemented");
                Ok(())
            }
            StorageBackend::FileSystem => {
                // TODO: Implement filesystem persistence
                tracing::warn!("Filesystem persistence not yet implemented");
                Ok(())
            }
        }
    }
}

#[async_trait]
impl EventHandler for EventStore {
    async fn handle(&self, event: &Event) -> HelixResult<()> {
        self.store(event.clone()).await
    }

    fn name(&self) -> &str {
        "EventStore"
    }

    fn priority(&self) -> u32 {
        1 // High priority for persistence
    }
}

/// Query for filtering events
#[derive(Debug, Clone, Default)]
pub struct EventQuery {
    /// Filter by event types
    pub event_types: Option<Vec<EventType>>,
    /// Filter by event categories
    pub categories: Option<Vec<EventCategory>>,
    /// Filter by time range
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    /// Filter by correlation ID
    pub correlation_id: Option<HelixId>,
    /// Filter by request ID
    pub request_id: Option<he_helix_core::RequestId>,
    /// Filter by process ID
    pub process_id: Option<he_helix_core::ProcessId>,
    /// Result ordering
    pub order_by: OrderBy,
    /// Result offset for pagination
    pub offset: Option<usize>,
    /// Result limit for pagination
    pub limit: Option<usize>,
}

/// Event filtering criteria
#[derive(Debug, Clone)]
pub struct EventFilter {
    /// Event types to include
    pub include_types: Option<Vec<EventType>>,
    /// Event types to exclude
    pub exclude_types: Option<Vec<EventType>>,
    /// Categories to include
    pub include_categories: Option<Vec<EventCategory>>,
    /// Categories to exclude
    pub exclude_categories: Option<Vec<EventCategory>>,
    /// Only include events newer than this
    pub min_age: Option<chrono::Duration>,
    /// Only include events older than this
    pub max_age: Option<chrono::Duration>,
}

impl Default for EventFilter {
    fn default() -> Self {
        Self {
            include_types: None,
            exclude_types: None,
            include_categories: None,
            exclude_categories: None,
            min_age: None,
            max_age: None,
        }
    }
}

/// Ordering options for query results
#[derive(Debug, Clone)]
pub enum OrderBy {
    Timestamp(OrderDirection),
    EventType,
}

impl Default for OrderBy {
    fn default() -> Self {
        Self::Timestamp(OrderDirection::Descending)
    }
}

/// Order direction
#[derive(Debug, Clone)]
pub enum OrderDirection {
    Ascending,
    Descending,
}

/// Event store statistics
#[derive(Debug, Clone)]
pub struct EventStoreStats {
    pub total_events: usize,
    pub events_by_type: std::collections::HashMap<EventType, usize>,
    pub events_by_category: std::collections::HashMap<EventCategory, usize>,
    pub oldest_event: Option<DateTime<Utc>>,
    pub newest_event: Option<DateTime<Utc>>,
}

/// Indices for fast event querying
#[derive(Debug)]
struct EventIndices {
    /// Count of events by type
    type_counts: std::collections::HashMap<EventType, usize>,
    /// Count of events by category
    category_counts: std::collections::HashMap<EventCategory, usize>,
    /// Events by correlation ID
    correlation_index: std::collections::HashMap<HelixId, Vec<HelixId>>,
}

impl EventIndices {
    fn new() -> Self {
        Self {
            type_counts: std::collections::HashMap::new(),
            category_counts: std::collections::HashMap::new(),
            correlation_index: std::collections::HashMap::new(),
        }
    }

    fn add_event(&mut self, event: &Event) {
        // Update type counts
        *self.type_counts.entry(event.event_type.clone()).or_insert(0) += 1;

        // Update category counts
        let category = event.event_type.category();
        *self.category_counts.entry(category).or_insert(0) += 1;

        // Update correlation index
        if let Some(correlation_id) = event.metadata.correlation_id {
            self.correlation_index
                .entry(correlation_id)
                .or_insert_with(Vec::new)
                .push(event.id);
        }
    }

    fn remove_event(&mut self, event: &Event) {
        // Update type counts
        if let Some(count) = self.type_counts.get_mut(&event.event_type) {
            if *count > 1 {
                *count -= 1;
            } else {
                self.type_counts.remove(&event.event_type);
            }
        }

        // Update category counts
        let category = event.event_type.category();
        if let Some(count) = self.category_counts.get_mut(&category) {
            if *count > 1 {
                *count -= 1;
            } else {
                self.category_counts.remove(&category);
            }
        }

        // Update correlation index
        if let Some(correlation_id) = event.metadata.correlation_id {
            if let Some(event_ids) = self.correlation_index.get_mut(&correlation_id) {
                event_ids.retain(|id| *id != event.id);
                if event_ids.is_empty() {
                    self.correlation_index.remove(&correlation_id);
                }
            }
        }
    }

    fn clear(&mut self) {
        self.type_counts.clear();
        self.category_counts.clear();
        self.correlation_index.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Event, EventType, EventData};

    #[tokio::test]
    async fn test_event_store() {
        let config = EventStoreConfig::default();
        let store = EventStore::new(config).await.unwrap();

        let event = Event::new(
            EventType::SystemStarted,
            EventData::SystemStatus {
                status: "started".to_string(),
                details: serde_json::Value::Null,
            },
        );

        // Store event
        store.store(event.clone()).await.unwrap();

        // Find by ID
        let found = store.find_by_id(event.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, event.id);

        // Find by type
        let events = store.find_by_type(&EventType::SystemStarted).await.unwrap();
        assert_eq!(events.len(), 1);

        // Get stats
        let stats = store.stats().await;
        assert_eq!(stats.total_events, 1);
    }
}