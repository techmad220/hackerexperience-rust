//! Event publishing system for producers

use crate::event::{Event, EventType, EventData, EventMetadata};
use crate::dispatcher::EventDispatcher;
use he_core::{HelixError, HelixResult, HelixId, RequestId, ProcessId};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Configuration for event publishers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishConfig {
    /// Default source name for published events
    pub default_source: Option<String>,
    /// Whether to automatically add correlation IDs
    pub auto_correlate: bool,
    /// Default metadata to add to all events
    pub default_metadata: std::collections::HashMap<String, serde_json::Value>,
    /// Batch publishing configuration
    pub batch_config: Option<BatchPublishConfig>,
}

impl Default for PublishConfig {
    fn default() -> Self {
        Self {
            default_source: None,
            auto_correlate: false,
            default_metadata: std::collections::HashMap::new(),
            batch_config: None,
        }
    }
}

/// Configuration for batch publishing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPublishConfig {
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Maximum time to wait before publishing a batch
    pub max_batch_time: std::time::Duration,
    /// Whether to enable batch publishing
    pub enabled: bool,
}

impl Default for BatchPublishConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            max_batch_time: std::time::Duration::from_millis(100),
            enabled: false,
        }
    }
}

/// Trait for event publishers
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish a single event
    async fn publish(&self, event: Event) -> HelixResult<()>;

    /// Publish multiple events
    async fn publish_batch(&self, events: Vec<Event>) -> HelixResult<()> {
        for event in events {
            self.publish(event).await?;
        }
        Ok(())
    }

    /// Create and publish an event
    async fn publish_event(
        &self,
        event_type: EventType,
        data: EventData,
    ) -> HelixResult<HelixId> {
        let event = Event::new(event_type, data);
        let event_id = event.id;
        self.publish(event).await?;
        Ok(event_id)
    }

    /// Create and publish an event with metadata
    async fn publish_event_with_metadata(
        &self,
        event_type: EventType,
        data: EventData,
        metadata: EventMetadata,
    ) -> HelixResult<HelixId> {
        let event = Event::with_metadata(event_type, data, metadata);
        let event_id = event.id;
        self.publish(event).await?;
        Ok(event_id)
    }

    /// Get the name of this publisher
    fn name(&self) -> &str {
        "UnnamedPublisher"
    }
}

/// Default event publisher that uses the event dispatcher
#[derive(Debug)]
pub struct DispatcherEventPublisher {
    /// Configuration
    config: PublishConfig,
    /// Event dispatcher
    dispatcher: Arc<EventDispatcher>,
    /// Current correlation context
    correlation_context: Arc<RwLock<Option<CorrelationContext>>>,
    /// Batch buffer if batching is enabled
    batch_buffer: Arc<RwLock<Vec<Event>>>,
}

impl DispatcherEventPublisher {
    /// Create a new dispatcher-based publisher
    pub fn new(config: PublishConfig, dispatcher: Arc<EventDispatcher>) -> Self {
        Self {
            config,
            dispatcher,
            correlation_context: Arc::new(RwLock::new(None)),
            batch_buffer: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Set correlation context for automatic correlation
    pub async fn set_correlation_context(&self, context: CorrelationContext) {
        let mut ctx = self.correlation_context.write().await;
        *ctx = Some(context);
    }

    /// Clear correlation context
    pub async fn clear_correlation_context(&self) {
        let mut ctx = self.correlation_context.write().await;
        *ctx = None;
    }

    /// Get current correlation context
    pub async fn get_correlation_context(&self) -> Option<CorrelationContext> {
        let ctx = self.correlation_context.read().await;
        ctx.clone()
    }

    /// Enhance event with configured defaults and correlation
    async fn enhance_event(&self, mut event: Event) -> Event {
        // Add default source
        if let Some(ref source) = self.config.default_source {
            if event.metadata.source.is_none() {
                event.metadata.source = Some(source.clone());
            }
        }

        // Add default metadata
        for (key, value) in &self.config.default_metadata {
            event.metadata.custom.entry(key.clone()).or_insert_with(|| value.clone());
        }

        // Apply correlation context
        if self.config.auto_correlate {
            if let Some(ref context) = *self.correlation_context.read().await {
                if event.metadata.correlation_id.is_none() {
                    event.metadata.correlation_id = context.correlation_id;
                }
                if event.metadata.causation_id.is_none() {
                    event.metadata.causation_id = context.causation_id;
                }
                if event.metadata.request_id.is_none() {
                    event.metadata.request_id = context.request_id;
                }
                if event.metadata.process_id.is_none() {
                    event.metadata.process_id = context.process_id;
                }
            }
        }

        event
    }
}

#[async_trait]
impl EventPublisher for DispatcherEventPublisher {
    async fn publish(&self, event: Event) -> HelixResult<()> {
        let enhanced_event = self.enhance_event(event).await;

        // Check if batching is enabled
        if let Some(ref batch_config) = self.config.batch_config {
            if batch_config.enabled {
                let mut buffer = self.batch_buffer.write().await;
                buffer.push(enhanced_event);

                // Check if we should flush the batch
                if buffer.len() >= batch_config.max_batch_size {
                    let events = std::mem::take(&mut *buffer);
                    drop(buffer); // Release the lock before dispatching
                    return self.publish_batch_internal(events).await;
                }
                
                return Ok(());
            }
        }

        // Direct publish
        self.dispatcher.dispatch(enhanced_event).await
    }

    async fn publish_batch(&self, events: Vec<Event>) -> HelixResult<()> {
        if events.is_empty() {
            return Ok(());
        }

        let mut enhanced_events = Vec::with_capacity(events.len());
        for event in events {
            enhanced_events.push(self.enhance_event(event).await);
        }

        self.publish_batch_internal(enhanced_events).await
    }

    fn name(&self) -> &str {
        "DispatcherEventPublisher"
    }
}

impl DispatcherEventPublisher {
    /// Internal batch publishing
    async fn publish_batch_internal(&self, events: Vec<Event>) -> HelixResult<()> {
        for event in events {
            self.dispatcher.dispatch(event).await?;
        }
        Ok(())
    }
}

/// Correlation context for automatic event correlation
#[derive(Debug, Clone)]
pub struct CorrelationContext {
    /// Correlation ID to group related events
    pub correlation_id: Option<HelixId>,
    /// Causation ID for event chains
    pub causation_id: Option<HelixId>,
    /// Request ID if within a request
    pub request_id: Option<RequestId>,
    /// Process ID that's generating events
    pub process_id: Option<ProcessId>,
}

impl CorrelationContext {
    /// Create a new correlation context
    pub fn new() -> Self {
        Self {
            correlation_id: Some(Uuid::new_v4()),
            causation_id: None,
            request_id: None,
            process_id: None,
        }
    }

    /// Create context with specific correlation ID
    pub fn with_correlation_id(correlation_id: HelixId) -> Self {
        Self {
            correlation_id: Some(correlation_id),
            causation_id: None,
            request_id: None,
            process_id: None,
        }
    }

    /// Add causation ID
    pub fn with_causation_id(mut self, causation_id: HelixId) -> Self {
        self.causation_id = Some(causation_id);
        self
    }

    /// Add request ID
    pub fn with_request_id(mut self, request_id: RequestId) -> Self {
        self.request_id = Some(request_id);
        self
    }

    /// Add process ID
    pub fn with_process_id(mut self, process_id: ProcessId) -> Self {
        self.process_id = Some(process_id);
        self
    }
}

impl Default for CorrelationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Event publisher builder for easy configuration
pub struct EventPublisherBuilder {
    config: PublishConfig,
}

impl EventPublisherBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: PublishConfig::default(),
        }
    }

    /// Set default source
    pub fn with_source<S: Into<String>>(mut self, source: S) -> Self {
        self.config.default_source = Some(source.into());
        self
    }

    /// Enable automatic correlation
    pub fn with_auto_correlation(mut self, enabled: bool) -> Self {
        self.config.auto_correlate = enabled;
        self
    }

    /// Add default metadata
    pub fn with_metadata<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<serde_json::Value>,
    {
        self.config.default_metadata.insert(key.into(), value.into());
        self
    }

    /// Enable batch publishing
    pub fn with_batching(mut self, config: BatchPublishConfig) -> Self {
        self.config.batch_config = Some(config);
        self
    }

    /// Build the publisher with a dispatcher
    pub fn build(self, dispatcher: Arc<EventDispatcher>) -> DispatcherEventPublisher {
        DispatcherEventPublisher::new(self.config, dispatcher)
    }
}

impl Default for EventPublisherBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A scoped event publisher that automatically manages correlation context
#[derive(Debug)]
pub struct ScopedEventPublisher {
    inner: Arc<dyn EventPublisher>,
    context: CorrelationContext,
}

impl ScopedEventPublisher {
    /// Create a new scoped publisher
    pub fn new(inner: Arc<dyn EventPublisher>, context: CorrelationContext) -> Self {
        Self { inner, context }
    }

    /// Create a scoped publisher with automatic correlation ID
    pub fn with_correlation(inner: Arc<dyn EventPublisher>) -> Self {
        Self::new(inner, CorrelationContext::new())
    }

    /// Get the correlation context
    pub fn context(&self) -> &CorrelationContext {
        &self.context
    }

    /// Publish an event that caused another event (sets causation_id)
    pub async fn publish_caused_by(&self, event: Event, caused_by: HelixId) -> HelixResult<()> {
        let mut enhanced_event = event;
        enhanced_event.metadata.causation_id = Some(caused_by);
        self.publish(enhanced_event).await
    }
}

#[async_trait]
impl EventPublisher for ScopedEventPublisher {
    async fn publish(&self, mut event: Event) -> HelixResult<()> {
        // Apply scoped context
        if event.metadata.correlation_id.is_none() {
            event.metadata.correlation_id = self.context.correlation_id;
        }
        if event.metadata.request_id.is_none() {
            event.metadata.request_id = self.context.request_id;
        }
        if event.metadata.process_id.is_none() {
            event.metadata.process_id = self.context.process_id;
        }

        self.inner.publish(event).await
    }

    async fn publish_batch(&self, mut events: Vec<Event>) -> HelixResult<()> {
        // Apply scoped context to all events
        for event in &mut events {
            if event.metadata.correlation_id.is_none() {
                event.metadata.correlation_id = self.context.correlation_id;
            }
            if event.metadata.request_id.is_none() {
                event.metadata.request_id = self.context.request_id;
            }
            if event.metadata.process_id.is_none() {
                event.metadata.process_id = self.context.process_id;
            }
        }

        self.inner.publish_batch(events).await
    }

    fn name(&self) -> &str {
        self.inner.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Event, EventType, EventData};
    use crate::dispatcher::{EventDispatcher, DispatchConfig};

    #[tokio::test]
    async fn test_event_publisher() {
        let dispatcher = Arc::new(EventDispatcher::new(DispatchConfig::default()).await.unwrap());
        let config = PublishConfig::default();
        let publisher = DispatcherEventPublisher::new(config, dispatcher);

        let event = Event::new(
            EventType::SystemStarted,
            EventData::SystemStatus {
                status: "started".to_string(),
                details: serde_json::Value::Null,
            },
        );

        let result = publisher.publish(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_scoped_publisher() {
        let dispatcher = Arc::new(EventDispatcher::new(DispatchConfig::default()).await.unwrap());
        let config = PublishConfig::default();
        let base_publisher = Arc::new(DispatcherEventPublisher::new(config, dispatcher));
        
        let context = CorrelationContext::new();
        let correlation_id = context.correlation_id.unwrap();
        
        let scoped = ScopedEventPublisher::new(base_publisher, context);

        let event = Event::new(
            EventType::SystemStarted,
            EventData::SystemStatus {
                status: "started".to_string(),
                details: serde_json::Value::Null,
            },
        );

        scoped.publish(event).await.unwrap();
        
        assert_eq!(scoped.context().correlation_id.unwrap(), correlation_id);
    }
}
