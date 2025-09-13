//! # Helix Event System
//!
//! This crate provides a comprehensive event-driven architecture for Helix,
//! supporting real-time event dispatch, persistence, and replay capabilities.

pub mod dispatcher;
pub mod event;
pub mod handler;
pub mod store;
pub mod stream;
pub mod subscriber;
pub mod publisher;
pub mod replay;

// Event behavior modules
pub mod loggable;
pub mod notificable;
pub mod listenable;
pub mod publishable;
pub mod state;

// Re-export commonly used types
pub use dispatcher::{EventDispatcher, DispatchConfig};
pub use event::{Event, EventData, EventMetadata, EventType};
pub use handler::{EventHandler, EventHandlerRegistry};
pub use store::{EventStore, EventFilter, EventQuery};
pub use stream::{EventStream, EventStreamConfig};
pub use subscriber::{EventSubscriber, SubscriptionConfig};
pub use publisher::{EventPublisher, PublishConfig};

use he_helix_core::HelixResult;

/// Initialize the event system with default configuration
pub async fn init() -> HelixResult<EventSystem> {
    EventSystem::new().await
}

/// Initialize the event system with custom configuration  
pub async fn init_with_config(config: EventSystemConfig) -> HelixResult<EventSystem> {
    EventSystem::with_config(config).await
}

/// Central event system coordinator
#[derive(Debug)]
pub struct EventSystem {
    dispatcher: EventDispatcher,
    store: EventStore,
    handler_registry: EventHandlerRegistry,
}

impl EventSystem {
    /// Create a new event system with default configuration
    pub async fn new() -> HelixResult<Self> {
        let config = EventSystemConfig::default();
        Self::with_config(config).await
    }

    /// Create a new event system with custom configuration
    pub async fn with_config(config: EventSystemConfig) -> HelixResult<Self> {
        let dispatcher = EventDispatcher::new(config.dispatch).await?;
        let store = EventStore::new(config.store).await?;
        let handler_registry = EventHandlerRegistry::new();

        Ok(Self {
            dispatcher,
            store,
            handler_registry,
        })
    }

    /// Get the event dispatcher
    pub fn dispatcher(&self) -> &EventDispatcher {
        &self.dispatcher
    }

    /// Get the event store
    pub fn store(&self) -> &EventStore {
        &self.store
    }

    /// Get the handler registry
    pub fn handler_registry(&self) -> &EventHandlerRegistry {
        &self.handler_registry
    }

    /// Start the event system
    pub async fn start(&mut self) -> HelixResult<()> {
        tracing::info!("Starting Helix event system");
        
        // Start the dispatcher
        self.dispatcher.start().await?;
        
        // Connect the dispatcher to the store for persistence
        self.dispatcher.add_persistent_handler(&self.store).await?;
        
        tracing::info!("Helix event system started successfully");
        Ok(())
    }

    /// Stop the event system
    pub async fn stop(&mut self) -> HelixResult<()> {
        tracing::info!("Stopping Helix event system");
        
        self.dispatcher.stop().await?;
        
        tracing::info!("Helix event system stopped successfully");
        Ok(())
    }
}

/// Configuration for the entire event system
#[derive(Debug, Clone)]
pub struct EventSystemConfig {
    /// Event dispatcher configuration
    pub dispatch: DispatchConfig,
    /// Event store configuration
    pub store: EventStoreConfig,
}

impl Default for EventSystemConfig {
    fn default() -> Self {
        Self {
            dispatch: DispatchConfig::default(),
            store: EventStoreConfig::default(),
        }
    }
}

/// Configuration for the event store
#[derive(Debug, Clone)]
pub struct EventStoreConfig {
    /// Maximum number of events to keep in memory
    pub max_memory_events: usize,
    /// Whether to persist events to disk
    pub enable_persistence: bool,
    /// Storage backend type
    pub storage_backend: StorageBackend,
    /// Database connection for persistent storage
    pub database_connection: Option<String>,
}

impl Default for EventStoreConfig {
    fn default() -> Self {
        Self {
            max_memory_events: 10000,
            enable_persistence: true,
            storage_backend: StorageBackend::Memory,
            database_connection: None,
        }
    }
}

/// Storage backend options for event persistence
#[derive(Debug, Clone)]
pub enum StorageBackend {
    Memory,
    Database,
    FileSystem,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_system_creation() {
        let system = EventSystem::new().await;
        assert!(system.is_ok());
    }

    #[tokio::test]
    async fn test_event_system_with_config() {
        let config = EventSystemConfig::default();
        let system = EventSystem::with_config(config).await;
        assert!(system.is_ok());
    }
}