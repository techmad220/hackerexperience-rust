//! Event subscription system for consumers

use crate::event::{Event, EventType, EventCategory};
use crate::stream::{EventStream, StreamFilter, EventStreamConfig};
use he_helix_core::{HelixError, HelixResult, HelixId};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio_stream::StreamExt;
use futures::Future;
use std::pin::Pin;

/// Configuration for event subscriptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionConfig {
    /// Unique name for this subscription
    pub name: String,
    /// Buffer size for the subscription
    pub buffer_size: usize,
    /// Whether to include historical events
    pub include_historical: bool,
    /// Filter for events
    pub filter: Option<StreamFilter>,
    /// Retry configuration for failed processing
    pub retry_config: Option<RetryConfig>,
}

impl SubscriptionConfig {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            buffer_size: 1000,
            include_historical: false,
            filter: None,
            retry_config: None,
        }
    }

    pub fn with_filter(mut self, filter: StreamFilter) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }
}

/// Retry configuration for failed event processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: std::time::Duration,
    pub max_delay: std::time::Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: std::time::Duration::from_millis(100),
            max_delay: std::time::Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

/// Trait for event subscribers
#[async_trait]
pub trait EventSubscriber: Send + Sync {
    /// Handle an incoming event
    async fn handle_event(&self, event: &Event) -> HelixResult<()>;

    /// Get the name of this subscriber
    fn name(&self) -> &str {
        "UnnamedSubscriber"
    }

    /// Check if this subscriber is interested in a specific event type
    fn is_interested_in(&self, event_type: &EventType) -> bool {
        // By default, subscribers are interested in all events
        true
    }

    /// Called when the subscription starts
    async fn on_start(&self) -> HelixResult<()> {
        Ok(())
    }

    /// Called when the subscription stops
    async fn on_stop(&self) -> HelixResult<()> {
        Ok(())
    }

    /// Called when an error occurs during processing
    async fn on_error(&self, error: &HelixError, event: &Event) -> HelixResult<()> {
        tracing::error!(
            subscriber = %self.name(),
            event_id = %event.id,
            error = %error,
            "Subscriber error"
        );
        Ok(())
    }
}

/// Event subscription manager
#[derive(Debug)]
pub struct EventSubscriptionManager {
    /// Active subscriptions
    subscriptions: Arc<RwLock<Vec<ActiveSubscription>>>,
    /// Broadcast sender for events
    event_sender: broadcast::Sender<Event>,
}

impl EventSubscriptionManager {
    /// Create a new subscription manager
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(10000);
        
        Self {
            subscriptions: Arc::new(RwLock::new(Vec::new())),
            event_sender,
        }
    }

    /// Subscribe with a subscriber and configuration
    pub async fn subscribe<S>(&self, subscriber: S, config: SubscriptionConfig) -> HelixResult<SubscriptionHandle>
    where
        S: EventSubscriber + 'static,
    {
        let subscription_id = HelixId::new_v4();
        let receiver = self.event_sender.subscribe();
        
        // Create event stream
        let stream_config = EventStreamConfig {
            buffer_size: config.buffer_size,
            include_historical: config.include_historical,
            filter: config.filter.clone(),
        };

        let stream = if let Some(filter) = config.filter {
            EventStream::filtered(stream_config, receiver, filter)
        } else {
            EventStream::new(stream_config, receiver)
        };

        // Create active subscription
        let subscription = ActiveSubscription {
            id: subscription_id,
            name: config.name.clone(),
            subscriber: Arc::new(subscriber),
            config: config.clone(),
            is_active: Arc::new(RwLock::new(false)),
        };

        // Start the subscription processing loop
        self.start_subscription_loop(subscription.clone(), stream).await?;

        // Add to active subscriptions
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.push(subscription);
        }

        tracing::info!("Started subscription: {} ({})", config.name, subscription_id);

        Ok(SubscriptionHandle {
            id: subscription_id,
            name: config.name,
            manager: self.subscriptions.clone(),
        })
    }

    /// Publish an event to all subscriptions
    pub async fn publish(&self, event: Event) -> HelixResult<()> {
        let _ = self.event_sender.send(event);
        Ok(())
    }

    /// Get active subscription count
    pub async fn subscription_count(&self) -> usize {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.len()
    }

    /// Get subscription statistics
    pub async fn stats(&self) -> SubscriptionStats {
        let subscriptions = self.subscriptions.read().await;
        
        SubscriptionStats {
            total_subscriptions: subscriptions.len(),
            active_subscriptions: {
                let mut count = 0;
                for sub in subscriptions.iter() {
                    if *sub.is_active.read().await {
                        count += 1;
                    }
                }
                count
            },
        }
    }

    /// Start the event processing loop for a subscription
    async fn start_subscription_loop(
        &self,
        subscription: ActiveSubscription,
        mut stream: impl futures::Stream<Item = HelixResult<Event>> + Send + Unpin + 'static,
    ) -> HelixResult<()> {
        let subscriber = Arc::clone(&subscription.subscriber);
        let is_active = Arc::clone(&subscription.is_active);
        let config = subscription.config.clone();

        // Mark as active
        {
            let mut active = is_active.write().await;
            *active = true;
        }

        // Call on_start
        subscriber.on_start().await?;

        tokio::spawn(async move {
            while *is_active.read().await {
                match stream.next().await {
                    Some(Ok(event)) => {
                        // Check if subscriber is interested
                        if subscriber.is_interested_in(&event.event_type) {
                            if let Err(error) = Self::process_event_with_retry(
                                &subscriber,
                                &event,
                                config.retry_config.as_ref(),
                            ).await {
                                if let Err(e) = subscriber.on_error(&error, &event).await {
                                    tracing::error!(
                                        "Error in subscriber error handler: {}",
                                        e
                                    );
                                }
                            }
                        }
                    }
                    Some(Err(error)) => {
                        tracing::error!(
                            subscription = %subscription.name,
                            error = %error,
                            "Stream error in subscription"
                        );
                    }
                    None => {
                        tracing::info!(
                            subscription = %subscription.name,
                            "Event stream ended"
                        );
                        break;
                    }
                }
            }

            // Call on_stop
            if let Err(e) = subscriber.on_stop().await {
                tracing::error!("Error in subscriber on_stop: {}", e);
            }

            tracing::info!(
                subscription = %subscription.name,
                "Subscription processing stopped"
            );
        });

        Ok(())
    }

    /// Process an event with retry logic
    async fn process_event_with_retry(
        subscriber: &Arc<dyn EventSubscriber>,
        event: &Event,
        retry_config: Option<&RetryConfig>,
    ) -> HelixResult<()> {
        let retry_config = retry_config.unwrap_or(&RetryConfig::default());
        let mut attempts = 0;
        let mut delay = retry_config.initial_delay;

        loop {
            attempts += 1;
            
            match subscriber.handle_event(event).await {
                Ok(()) => return Ok(()),
                Err(error) => {
                    if attempts >= retry_config.max_attempts {
                        return Err(error);
                    }

                    tracing::warn!(
                        subscriber = %subscriber.name(),
                        event_id = %event.id,
                        attempt = attempts,
                        error = %error,
                        "Event processing failed, retrying"
                    );

                    tokio::time::sleep(delay).await;
                    
                    // Exponential backoff
                    delay = std::cmp::min(
                        std::time::Duration::from_millis(
                            (delay.as_millis() as f64 * retry_config.backoff_multiplier) as u64
                        ),
                        retry_config.max_delay,
                    );
                }
            }
        }
    }

    /// Stop a specific subscription
    async fn stop_subscription(&self, subscription_id: HelixId) -> HelixResult<()> {
        let mut subscriptions = self.subscriptions.write().await;
        
        if let Some(pos) = subscriptions.iter().position(|s| s.id == subscription_id) {
            let subscription = subscriptions.remove(pos);
            
            // Mark as inactive
            {
                let mut active = subscription.is_active.write().await;
                *active = false;
            }

            tracing::info!("Stopped subscription: {} ({})", subscription.name, subscription_id);
            Ok(())
        } else {
            Err(HelixError::not_found(format!(
                "Subscription not found: {}",
                subscription_id
            )))
        }
    }
}

impl Default for EventSubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle for managing a subscription
pub struct SubscriptionHandle {
    id: HelixId,
    name: String,
    manager: Arc<RwLock<Vec<ActiveSubscription>>>,
}

impl SubscriptionHandle {
    /// Get the subscription ID
    pub fn id(&self) -> HelixId {
        self.id
    }

    /// Get the subscription name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Stop this subscription
    pub async fn stop(self) -> HelixResult<()> {
        let mut subscriptions = self.manager.write().await;
        
        if let Some(pos) = subscriptions.iter().position(|s| s.id == self.id) {
            let subscription = subscriptions.remove(pos);
            
            // Mark as inactive
            {
                let mut active = subscription.is_active.write().await;
                *active = false;
            }

            tracing::info!("Stopped subscription: {} ({})", self.name, self.id);
            Ok(())
        } else {
            Err(HelixError::not_found(format!(
                "Subscription not found: {}",
                self.id
            )))
        }
    }

    /// Check if the subscription is active
    pub async fn is_active(&self) -> bool {
        let subscriptions = self.manager.read().await;
        if let Some(subscription) = subscriptions.iter().find(|s| s.id == self.id) {
            *subscription.is_active.read().await
        } else {
            false
        }
    }
}

/// Active subscription
#[derive(Debug)]
struct ActiveSubscription {
    id: HelixId,
    name: String,
    subscriber: Arc<dyn EventSubscriber>,
    config: SubscriptionConfig,
    is_active: Arc<RwLock<bool>>,
}

/// Subscription statistics
#[derive(Debug, Clone)]
pub struct SubscriptionStats {
    pub total_subscriptions: usize,
    pub active_subscriptions: usize,
}

/// A simple logging subscriber for debugging
#[derive(Debug)]
pub struct LoggingSubscriber {
    name: String,
}

impl LoggingSubscriber {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
        }
    }
}

#[async_trait]
impl EventSubscriber for LoggingSubscriber {
    async fn handle_event(&self, event: &Event) -> HelixResult<()> {
        tracing::info!(
            subscriber = %self.name,
            event_id = %event.id,
            event_type = ?event.event_type,
            timestamp = %event.metadata.timestamp,
            "Received event"
        );
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// A function-based subscriber
pub struct FunctionSubscriber<F>
where
    F: Fn(&Event) -> Pin<Box<dyn Future<Output = HelixResult<()>> + Send>> + Send + Sync,
{
    name: String,
    handler: F,
}

impl<F> FunctionSubscriber<F>
where
    F: Fn(&Event) -> Pin<Box<dyn Future<Output = HelixResult<()>> + Send>> + Send + Sync,
{
    pub fn new<S: Into<String>>(name: S, handler: F) -> Self {
        Self {
            name: name.into(),
            handler,
        }
    }
}

#[async_trait]
impl<F> EventSubscriber for FunctionSubscriber<F>
where
    F: Fn(&Event) -> Pin<Box<dyn Future<Output = HelixResult<()>> + Send>> + Send + Sync,
{
    async fn handle_event(&self, event: &Event) -> HelixResult<()> {
        (self.handler)(event).await
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Event, EventType, EventData};

    #[tokio::test]
    async fn test_subscription_manager() {
        let manager = EventSubscriptionManager::new();
        let subscriber = LoggingSubscriber::new("test");
        let config = SubscriptionConfig::new("test-subscription");

        let handle = manager.subscribe(subscriber, config).await.unwrap();
        assert!(handle.is_active().await);

        let stats = manager.stats().await;
        assert_eq!(stats.total_subscriptions, 1);
        assert_eq!(stats.active_subscriptions, 1);

        handle.stop().await.unwrap();

        let stats = manager.stats().await;
        assert_eq!(stats.total_subscriptions, 0);
    }
}