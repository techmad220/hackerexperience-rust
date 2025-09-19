//! Event dispatcher for real-time event distribution

use crate::event::{Event, EventType, EventCategory};
use crate::handler::EventHandler;
use he_core::{HelixError, HelixResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, broadcast};
use tokio::time::{interval, Duration};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// Event dispatcher configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchConfig {
    /// Buffer size for the event queue
    pub queue_size: usize,
    /// Maximum number of concurrent event handlers
    pub max_concurrent_handlers: usize,
    /// Retry configuration for failed events
    pub retry_config: RetryConfig,
    /// Dead letter queue configuration
    pub dead_letter_config: DeadLetterConfig,
    /// Event batching configuration
    pub batch_config: BatchConfig,
}

impl Default for DispatchConfig {
    fn default() -> Self {
        Self {
            queue_size: 10000,
            max_concurrent_handlers: 100,
            retry_config: RetryConfig::default(),
            dead_letter_config: DeadLetterConfig::default(),
            batch_config: BatchConfig::default(),
        }
    }
}

/// Retry configuration for failed event processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

/// Dead letter queue configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterConfig {
    /// Whether to enable dead letter queue
    pub enabled: bool,
    /// Maximum size of dead letter queue
    pub max_size: usize,
}

impl Default for DeadLetterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_size: 1000,
        }
    }
}

/// Event batching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Whether to enable event batching
    pub enabled: bool,
    /// Maximum batch size
    pub max_size: usize,
    /// Maximum time to wait before flushing a batch
    pub flush_interval: Duration,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_size: 100,
            flush_interval: Duration::from_millis(100),
        }
    }
}

/// Central event dispatcher
pub struct EventDispatcher {
    /// Configuration
    config: DispatchConfig,
    /// Event queue sender
    event_tx: mpsc::Sender<Event>,
    /// Event queue receiver
    event_rx: Arc<RwLock<Option<mpsc::Receiver<Event>>>>,
    /// Broadcast channel for real-time subscribers
    broadcast_tx: broadcast::Sender<Event>,
    /// Event handlers by type
    handlers: Arc<DashMap<EventType, Vec<Arc<dyn EventHandler>>>>,
    /// Event handlers by category
    category_handlers: Arc<DashMap<EventCategory, Vec<Arc<dyn EventHandler>>>>,
    /// Wildcard handlers (receive all events)
    wildcard_handlers: Arc<RwLock<Vec<Arc<dyn EventHandler>>>>,
    /// Dead letter queue
    dead_letter_queue: Arc<RwLock<Vec<Event>>>,
    /// Whether the dispatcher is running
    is_running: Arc<RwLock<bool>>,
    /// Metrics
    metrics: Arc<DispatcherMetrics>,
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub async fn new(config: DispatchConfig) -> HelixResult<Self> {
        let (event_tx, event_rx) = mpsc::channel(config.queue_size);
        let (broadcast_tx, _) = broadcast::channel(config.queue_size);

        Ok(Self {
            config,
            event_tx,
            event_rx: Arc::new(RwLock::new(Some(event_rx))),
            broadcast_tx,
            handlers: Arc::new(DashMap::new()),
            category_handlers: Arc::new(DashMap::new()),
            wildcard_handlers: Arc::new(RwLock::new(Vec::new())),
            dead_letter_queue: Arc::new(RwLock::new(Vec::new())),
            is_running: Arc::new(RwLock::new(false)),
            metrics: Arc::new(DispatcherMetrics::new()),
        })
    }

    /// Start the event dispatcher
    pub async fn start(&self) -> HelixResult<()> {
        {
            let mut is_running = self.is_running.write().await;
            if *is_running {
                return Err(HelixError::validation("Dispatcher is already running"));
            }
            *is_running = true;
        }

        // Take the receiver from the option
        let event_rx = {
            let mut rx_opt = self.event_rx.write().await;
            rx_opt.take().ok_or_else(|| HelixError::internal("Event receiver already taken"))?
        };

        // Start the main event processing loop
        let handlers = Arc::clone(&self.handlers);
        let category_handlers = Arc::clone(&self.category_handlers);
        let wildcard_handlers = Arc::clone(&self.wildcard_handlers);
        let dead_letter_queue = Arc::clone(&self.dead_letter_queue);
        let is_running = Arc::clone(&self.is_running);
        let metrics = Arc::clone(&self.metrics);
        let config = self.config.clone();
        let broadcast_tx = self.broadcast_tx.clone();

        tokio::spawn(async move {
            Self::event_loop(
                event_rx,
                handlers,
                category_handlers,
                wildcard_handlers,
                dead_letter_queue,
                is_running,
                metrics,
                config,
                broadcast_tx,
            ).await;
        });

        // Start metrics reporting
        self.start_metrics_reporting().await;

        tracing::info!("Event dispatcher started");
        Ok(())
    }

    /// Stop the event dispatcher
    pub async fn stop(&self) -> HelixResult<()> {
        {
            let mut is_running = self.is_running.write().await;
            *is_running = false;
        }

        tracing::info!("Event dispatcher stopped");
        Ok(())
    }

    /// Dispatch an event
    pub async fn dispatch(&self, event: Event) -> HelixResult<()> {
        self.metrics.increment_events_received();

        // Send to broadcast channel for real-time subscribers
        if self.broadcast_tx.receiver_count() > 0 {
            let _ = self.broadcast_tx.send(event.clone());
        }

        // Send to processing queue
        self.event_tx
            .send(event)
            .await
            .map_err(|_| HelixError::internal("Failed to send event to dispatcher"))?;

        Ok(())
    }

    /// Add an event handler for a specific event type
    pub async fn add_handler(&self, event_type: EventType, handler: Arc<dyn EventHandler>) {
        self.handlers
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(handler);
    }

    /// Add an event handler for an event category
    pub async fn add_category_handler(&self, category: EventCategory, handler: Arc<dyn EventHandler>) {
        self.category_handlers
            .entry(category)
            .or_insert_with(Vec::new)
            .push(handler);
    }

    /// Add a wildcard handler that receives all events
    pub async fn add_wildcard_handler(&self, handler: Arc<dyn EventHandler>) {
        let mut handlers = self.wildcard_handlers.write().await;
        handlers.push(handler);
    }

    /// Add a persistent handler that stores events
    pub async fn add_persistent_handler<H>(&self, handler: &H) -> HelixResult<()>
    where
        H: EventHandler + Clone + 'static,
    {
        let handler = Arc::new(handler.clone());
        self.add_wildcard_handler(handler).await;
        Ok(())
    }

    /// Subscribe to real-time events
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.broadcast_tx.subscribe()
    }

    /// Get dispatcher metrics
    pub fn metrics(&self) -> Arc<DispatcherMetrics> {
        Arc::clone(&self.metrics)
    }

    /// Get dead letter queue contents
    pub async fn dead_letter_queue(&self) -> Vec<Event> {
        let dlq = self.dead_letter_queue.read().await;
        dlq.clone()
    }

    /// Clear dead letter queue
    pub async fn clear_dead_letter_queue(&self) -> HelixResult<()> {
        let mut dlq = self.dead_letter_queue.write().await;
        dlq.clear();
        Ok(())
    }

    /// Main event processing loop
    async fn event_loop(
        mut event_rx: mpsc::Receiver<Event>,
        handlers: Arc<DashMap<EventType, Vec<Arc<dyn EventHandler>>>>,
        category_handlers: Arc<DashMap<EventCategory, Vec<Arc<dyn EventHandler>>>>,
        wildcard_handlers: Arc<RwLock<Vec<Arc<dyn EventHandler>>>>,
        dead_letter_queue: Arc<RwLock<Vec<Event>>>,
        is_running: Arc<RwLock<bool>>,
        metrics: Arc<DispatcherMetrics>,
        config: DispatchConfig,
        _broadcast_tx: broadcast::Sender<Event>,
    ) {
        while *is_running.read().await {
            if let Some(event) = event_rx.recv().await {
                metrics.increment_events_processed();

                // Process the event
                let result = Self::process_event(
                    &event,
                    &handlers,
                    &category_handlers,
                    &wildcard_handlers,
                    &config,
                ).await;

                match result {
                    Ok(_) => {
                        metrics.increment_events_successful();
                    }
                    Err(e) => {
                        metrics.increment_events_failed();
                        tracing::error!("Event processing failed: {}", e);

                        // Add to dead letter queue if enabled
                        if config.dead_letter_config.enabled {
                            let mut dlq = dead_letter_queue.write().await;
                            if dlq.len() < config.dead_letter_config.max_size {
                                dlq.push(event);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Process a single event
    async fn process_event(
        event: &Event,
        handlers: &DashMap<EventType, Vec<Arc<dyn EventHandler>>>,
        category_handlers: &DashMap<EventCategory, Vec<Arc<dyn EventHandler>>>,
        wildcard_handlers: &RwLock<Vec<Arc<dyn EventHandler>>>,
        _config: &DispatchConfig,
    ) -> HelixResult<()> {
        let mut handler_results = Vec::new();

        // Get handlers for specific event type
        if let Some(type_handlers) = handlers.get(&event.event_type) {
            for handler in type_handlers.iter() {
                let result = handler.handle(event).await;
                handler_results.push(result);
            }
        }

        // Get handlers for event category
        let category = event.event_type.category();
        if let Some(cat_handlers) = category_handlers.get(&category) {
            for handler in cat_handlers.iter() {
                let result = handler.handle(event).await;
                handler_results.push(result);
            }
        }

        // Get wildcard handlers
        {
            let wildcard = wildcard_handlers.read().await;
            for handler in wildcard.iter() {
                let result = handler.handle(event).await;
                handler_results.push(result);
            }
        }

        // Check if any handler failed
        for result in handler_results {
            if let Err(e) = result {
                return Err(e);
            }
        }

        Ok(())
    }

    /// Start metrics reporting
    async fn start_metrics_reporting(&self) {
        let metrics = Arc::clone(&self.metrics);
        let is_running = Arc::clone(&self.is_running);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                if !*is_running.read().await {
                    break;
                }

                let snapshot = metrics.snapshot();
                tracing::info!(
                    "Event dispatcher metrics: received={}, processed={}, successful={}, failed={}",
                    snapshot.events_received,
                    snapshot.events_processed,
                    snapshot.events_successful,
                    snapshot.events_failed
                );
            }
        });
    }
}

/// Event dispatcher metrics
#[derive(Debug)]
pub struct DispatcherMetrics {
    events_received: Arc<RwLock<u64>>,
    events_processed: Arc<RwLock<u64>>,
    events_successful: Arc<RwLock<u64>>,
    events_failed: Arc<RwLock<u64>>,
}

impl DispatcherMetrics {
    pub fn new() -> Self {
        Self {
            events_received: Arc::new(RwLock::new(0)),
            events_processed: Arc::new(RwLock::new(0)),
            events_successful: Arc::new(RwLock::new(0)),
            events_failed: Arc::new(RwLock::new(0)),
        }
    }

    pub fn increment_events_received(&self) {
        tokio::spawn({
            let counter = Arc::clone(&self.events_received);
            async move {
                let mut count = counter.write().await;
                *count += 1;
            }
        });
    }

    pub fn increment_events_processed(&self) {
        tokio::spawn({
            let counter = Arc::clone(&self.events_processed);
            async move {
                let mut count = counter.write().await;
                *count += 1;
            }
        });
    }

    pub fn increment_events_successful(&self) {
        tokio::spawn({
            let counter = Arc::clone(&self.events_successful);
            async move {
                let mut count = counter.write().await;
                *count += 1;
            }
        });
    }

    pub fn increment_events_failed(&self) {
        tokio::spawn({
            let counter = Arc::clone(&self.events_failed);
            async move {
                let mut count = counter.write().await;
                *count += 1;
            }
        });
    }

    pub async fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            events_received: *self.events_received.read().await,
            events_processed: *self.events_processed.read().await,
            events_successful: *self.events_successful.read().await,
            events_failed: *self.events_failed.read().await,
        }
    }
}

/// Snapshot of dispatcher metrics
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub events_received: u64,
    pub events_processed: u64,
    pub events_successful: u64,
    pub events_failed: u64,
}
