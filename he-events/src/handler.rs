//! Event handler traits and registry

use crate::event::{Event, EventType, EventCategory};
use he_helix_core::{HelixError, HelixResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Trait for handling events
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle an incoming event
    async fn handle(&self, event: &Event) -> HelixResult<()>;

    /// Get the name of this handler for debugging
    fn name(&self) -> &str {
        "UnnamedHandler"
    }

    /// Check if this handler can handle a specific event type
    fn can_handle(&self, event_type: &EventType) -> bool {
        // By default, handlers can handle any event type
        // Override this method for more specific filtering
        true
    }

    /// Get the priority of this handler (lower numbers = higher priority)
    fn priority(&self) -> u32 {
        100
    }
}

/// Registry for managing event handlers
#[derive(Debug)]
pub struct EventHandlerRegistry {
    /// Handlers by event type
    type_handlers: Arc<RwLock<HashMap<EventType, Vec<RegisteredHandler>>>>,
    /// Handlers by event category
    category_handlers: Arc<RwLock<HashMap<EventCategory, Vec<RegisteredHandler>>>>,
    /// Wildcard handlers (handle all events)
    wildcard_handlers: Arc<RwLock<Vec<RegisteredHandler>>>,
}

/// A registered handler with metadata
#[derive(Debug)]
struct RegisteredHandler {
    /// The actual handler
    handler: Arc<dyn EventHandler>,
    /// Registration ID for removal
    id: String,
    /// Priority for ordering
    priority: u32,
}

impl EventHandlerRegistry {
    /// Create a new handler registry
    pub fn new() -> Self {
        Self {
            type_handlers: Arc::new(RwLock::new(HashMap::new())),
            category_handlers: Arc::new(RwLock::new(HashMap::new())),
            wildcard_handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a handler for a specific event type
    pub async fn register_type_handler(
        &self,
        event_type: EventType,
        handler: Arc<dyn EventHandler>,
    ) -> String {
        let id = self.generate_handler_id();
        let registered = RegisteredHandler {
            handler: handler.clone(),
            id: id.clone(),
            priority: handler.priority(),
        };

        let mut handlers = self.type_handlers.write().await;
        let type_handlers = handlers.entry(event_type).or_insert_with(Vec::new);
        type_handlers.push(registered);
        
        // Sort by priority
        type_handlers.sort_by_key(|h| h.priority);

        tracing::debug!("Registered handler '{}' for type", handler.name());
        id
    }

    /// Register a handler for an event category
    pub async fn register_category_handler(
        &self,
        category: EventCategory,
        handler: Arc<dyn EventHandler>,
    ) -> String {
        let id = self.generate_handler_id();
        let registered = RegisteredHandler {
            handler: handler.clone(),
            id: id.clone(),
            priority: handler.priority(),
        };

        let mut handlers = self.category_handlers.write().await;
        let category_handlers = handlers.entry(category).or_insert_with(Vec::new);
        category_handlers.push(registered);
        
        // Sort by priority
        category_handlers.sort_by_key(|h| h.priority);

        tracing::debug!("Registered handler '{}' for category", handler.name());
        id
    }

    /// Register a wildcard handler that receives all events
    pub async fn register_wildcard_handler(&self, handler: Arc<dyn EventHandler>) -> String {
        let id = self.generate_handler_id();
        let registered = RegisteredHandler {
            handler: handler.clone(),
            id: id.clone(),
            priority: handler.priority(),
        };

        let mut handlers = self.wildcard_handlers.write().await;
        handlers.push(registered);
        
        // Sort by priority
        handlers.sort_by_key(|h| h.priority);

        tracing::debug!("Registered wildcard handler '{}'", handler.name());
        id
    }

    /// Unregister a handler by ID
    pub async fn unregister_handler(&self, handler_id: &str) -> HelixResult<()> {
        // Try to remove from type handlers
        {
            let mut handlers = self.type_handlers.write().await;
            for (_, type_handlers) in handlers.iter_mut() {
                type_handlers.retain(|h| h.id != handler_id);
            }
        }

        // Try to remove from category handlers
        {
            let mut handlers = self.category_handlers.write().await;
            for (_, category_handlers) in handlers.iter_mut() {
                category_handlers.retain(|h| h.id != handler_id);
            }
        }

        // Try to remove from wildcard handlers
        {
            let mut handlers = self.wildcard_handlers.write().await;
            handlers.retain(|h| h.id != handler_id);
        }

        tracing::debug!("Unregistered handler with ID: {}", handler_id);
        Ok(())
    }

    /// Get all handlers that should process a given event
    pub async fn get_handlers_for_event(&self, event: &Event) -> Vec<Arc<dyn EventHandler>> {
        let mut handlers = Vec::new();

        // Get type-specific handlers
        {
            let type_handlers = self.type_handlers.read().await;
            if let Some(event_handlers) = type_handlers.get(&event.event_type) {
                for registered in event_handlers {
                    if registered.handler.can_handle(&event.event_type) {
                        handlers.push(Arc::clone(&registered.handler));
                    }
                }
            }
        }

        // Get category handlers
        let category = event.event_type.category();
        {
            let category_handlers = self.category_handlers.read().await;
            if let Some(cat_handlers) = category_handlers.get(&category) {
                for registered in cat_handlers {
                    if registered.handler.can_handle(&event.event_type) {
                        handlers.push(Arc::clone(&registered.handler));
                    }
                }
            }
        }

        // Get wildcard handlers
        {
            let wildcard_handlers = self.wildcard_handlers.read().await;
            for registered in wildcard_handlers.iter() {
                if registered.handler.can_handle(&event.event_type) {
                    handlers.push(Arc::clone(&registered.handler));
                }
            }
        }

        handlers
    }

    /// Get statistics about registered handlers
    pub async fn stats(&self) -> HandlerRegistryStats {
        let type_count = {
            let handlers = self.type_handlers.read().await;
            handlers.values().map(|v| v.len()).sum()
        };

        let category_count = {
            let handlers = self.category_handlers.read().await;
            handlers.values().map(|v| v.len()).sum()
        };

        let wildcard_count = {
            let handlers = self.wildcard_handlers.read().await;
            handlers.len()
        };

        HandlerRegistryStats {
            type_handlers: type_count,
            category_handlers: category_count,
            wildcard_handlers: wildcard_count,
            total_handlers: type_count + category_count + wildcard_count,
        }
    }

    /// Generate a unique handler ID
    fn generate_handler_id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

impl Default for EventHandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the handler registry
#[derive(Debug, Clone)]
pub struct HandlerRegistryStats {
    pub type_handlers: usize,
    pub category_handlers: usize,
    pub wildcard_handlers: usize,
    pub total_handlers: usize,
}

/// A simple logging event handler for debugging
#[derive(Debug, Clone)]
pub struct LoggingEventHandler {
    pub name: String,
}

impl LoggingEventHandler {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
        }
    }
}

#[async_trait]
impl EventHandler for LoggingEventHandler {
    async fn handle(&self, event: &Event) -> HelixResult<()> {
        tracing::info!(
            handler = %self.name,
            event_id = %event.id,
            event_type = ?event.event_type,
            timestamp = %event.metadata.timestamp,
            "Handling event"
        );
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> u32 {
        1000 // Low priority for logging
    }
}

/// A metrics collection handler
#[derive(Debug, Clone)]
pub struct MetricsEventHandler {
    pub name: String,
    // In a real implementation, this would have metrics collection logic
}

impl MetricsEventHandler {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
        }
    }
}

#[async_trait]
impl EventHandler for MetricsEventHandler {
    async fn handle(&self, event: &Event) -> HelixResult<()> {
        // Collect metrics about the event
        // This is a placeholder implementation
        tracing::trace!(
            handler = %self.name,
            event_type = ?event.event_type,
            category = ?event.event_type.category(),
            "Collecting event metrics"
        );
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> u32 {
        10 // High priority for metrics
    }
}

/// A filtering handler that only processes events matching criteria
#[derive(Debug)]
pub struct FilteringEventHandler<F>
where
    F: Fn(&Event) -> bool + Send + Sync,
{
    pub name: String,
    pub filter: F,
    pub inner_handler: Arc<dyn EventHandler>,
}

impl<F> FilteringEventHandler<F>
where
    F: Fn(&Event) -> bool + Send + Sync,
{
    pub fn new<S: Into<String>>(
        name: S,
        filter: F,
        inner_handler: Arc<dyn EventHandler>,
    ) -> Self {
        Self {
            name: name.into(),
            filter,
            inner_handler,
        }
    }
}

#[async_trait]
impl<F> EventHandler for FilteringEventHandler<F>
where
    F: Fn(&Event) -> bool + Send + Sync,
{
    async fn handle(&self, event: &Event) -> HelixResult<()> {
        if (self.filter)(event) {
            self.inner_handler.handle(event).await
        } else {
            Ok(()) // Skip this event
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> u32 {
        self.inner_handler.priority()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Event, EventType, EventData, EventMetadata};

    #[tokio::test]
    async fn test_handler_registry() {
        let registry = EventHandlerRegistry::new();
        let handler = Arc::new(LoggingEventHandler::new("test"));
        
        let id = registry
            .register_type_handler(EventType::SystemStarted, handler)
            .await;
        
        let event = Event::new(
            EventType::SystemStarted,
            EventData::SystemStatus {
                status: "started".to_string(),
                details: serde_json::Value::Null,
            },
        );
        
        let handlers = registry.get_handlers_for_event(&event).await;
        assert_eq!(handlers.len(), 1);
        
        registry.unregister_handler(&id).await.unwrap();
        let handlers = registry.get_handlers_for_event(&event).await;
        assert_eq!(handlers.len(), 0);
    }
}