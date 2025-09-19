//! Event streaming utilities for real-time event processing

use crate::event::{Event, EventType, EventCategory};
use he_core::{HelixError, HelixResult};
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

/// Configuration for event streams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStreamConfig {
    /// Buffer size for the stream
    pub buffer_size: usize,
    /// Whether to include historical events
    pub include_historical: bool,
    /// Filter configuration
    pub filter: Option<StreamFilter>,
}

impl Default for EventStreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            include_historical: false,
            filter: None,
        }
    }
}

/// Filter for event streams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamFilter {
    /// Event types to include
    pub include_types: Option<Vec<EventType>>,
    /// Event types to exclude
    pub exclude_types: Option<Vec<EventType>>,
    /// Categories to include
    pub include_categories: Option<Vec<EventCategory>>,
    /// Categories to exclude
    pub exclude_categories: Option<Vec<EventCategory>>,
    /// Only critical events
    pub critical_only: bool,
}

impl StreamFilter {
    /// Check if an event passes this filter
    pub fn matches(&self, event: &Event) -> bool {
        // Check include types
        if let Some(ref include_types) = self.include_types {
            if !include_types.contains(&event.event_type) {
                return false;
            }
        }

        // Check exclude types
        if let Some(ref exclude_types) = self.exclude_types {
            if exclude_types.contains(&event.event_type) {
                return false;
            }
        }

        // Check include categories
        if let Some(ref include_categories) = self.include_categories {
            let category = event.event_type.category();
            if !include_categories.contains(&category) {
                return false;
            }
        }

        // Check exclude categories
        if let Some(ref exclude_categories) = self.exclude_categories {
            let category = event.event_type.category();
            if exclude_categories.contains(&category) {
                return false;
            }
        }

        // Check critical only
        if self.critical_only && !event.event_type.is_critical() {
            return false;
        }

        true
    }
}

/// Event stream for real-time event processing
pub struct EventStream {
    /// Configuration
    config: EventStreamConfig,
    /// Inner broadcast stream
    inner: Pin<Box<dyn Stream<Item = Result<Event, broadcast::error::RecvError>> + Send>>,
}

impl EventStream {
    /// Create a new event stream from a broadcast receiver
    pub fn new(config: EventStreamConfig, receiver: broadcast::Receiver<Event>) -> Self {
        let stream = BroadcastStream::new(receiver);
        
        Self {
            config,
            inner: Box::pin(stream),
        }
    }

    /// Create a filtered event stream
    pub fn filtered(
        config: EventStreamConfig,
        receiver: broadcast::Receiver<Event>,
        filter: StreamFilter,
    ) -> FilteredEventStream {
        let stream = Self::new(config, receiver);
        FilteredEventStream::new(stream, filter)
    }

    /// Convert to a filtered stream
    pub fn with_filter(self, filter: StreamFilter) -> FilteredEventStream {
        FilteredEventStream::new(self, filter)
    }

    /// Map events to another type
    pub fn map<T, F>(self, f: F) -> MappedEventStream<T, F>
    where
        F: Fn(Event) -> T + Send + 'static,
        T: Send + 'static,
    {
        MappedEventStream::new(self, f)
    }

    /// Take only the first N events
    pub fn take(self, count: usize) -> TakeEventStream {
        TakeEventStream::new(self, count)
    }

    /// Skip the first N events
    pub fn skip(self, count: usize) -> SkipEventStream {
        SkipEventStream::new(self, count)
    }
}

impl Stream for EventStream {
    type Item = HelixResult<Event>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(event))) => Poll::Ready(Some(Ok(event))),
            Poll::Ready(Some(Err(broadcast::error::RecvError::Lagged(count)))) => {
                tracing::warn!("Event stream lagged, skipped {} events", count);
                // Continue polling for the next event
                self.poll_next(cx)
            }
            Poll::Ready(Some(Err(broadcast::error::RecvError::Closed))) => {
                Poll::Ready(Some(Err(HelixError::internal("Event stream closed"))))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Filtered event stream
pub struct FilteredEventStream {
    inner: EventStream,
    filter: StreamFilter,
}

impl FilteredEventStream {
    fn new(inner: EventStream, filter: StreamFilter) -> Self {
        Self { inner, filter }
    }
}

impl Stream for FilteredEventStream {
    type Item = HelixResult<Event>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match Pin::new(&mut self.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(event))) => {
                    if self.filter.matches(&event) {
                        return Poll::Ready(Some(Ok(event)));
                    }
                    // Event filtered out, continue polling
                    continue;
                }
                Poll::Ready(Some(Err(e))) => return Poll::Ready(Some(Err(e))),
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Mapped event stream
pub struct MappedEventStream<T, F>
where
    F: Fn(Event) -> T,
{
    inner: EventStream,
    mapper: F,
}

impl<T, F> MappedEventStream<T, F>
where
    F: Fn(Event) -> T,
{
    fn new(inner: EventStream, mapper: F) -> Self {
        Self { inner, mapper }
    }
}

impl<T, F> Stream for MappedEventStream<T, F>
where
    F: Fn(Event) -> T + Unpin,
    T: Send + 'static,
{
    type Item = HelixResult<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(event))) => {
                let mapped = (self.mapper)(event);
                Poll::Ready(Some(Ok(mapped)))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Take event stream
pub struct TakeEventStream {
    inner: EventStream,
    remaining: usize,
}

impl TakeEventStream {
    fn new(inner: EventStream, count: usize) -> Self {
        Self {
            inner,
            remaining: count,
        }
    }
}

impl Stream for TakeEventStream {
    type Item = HelixResult<Event>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.remaining == 0 {
            return Poll::Ready(None);
        }

        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(result)) => {
                self.remaining -= 1;
                Poll::Ready(Some(result))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Skip event stream
pub struct SkipEventStream {
    inner: EventStream,
    remaining: usize,
}

impl SkipEventStream {
    fn new(inner: EventStream, count: usize) -> Self {
        Self {
            inner,
            remaining: count,
        }
    }
}

impl Stream for SkipEventStream {
    type Item = HelixResult<Event>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match Pin::new(&mut self.inner).poll_next(cx) {
                Poll::Ready(Some(result)) => {
                    if self.remaining > 0 {
                        self.remaining -= 1;
                        continue; // Skip this event
                    } else {
                        return Poll::Ready(Some(result));
                    }
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Event stream builder for easy configuration
pub struct EventStreamBuilder {
    config: EventStreamConfig,
    filter: Option<StreamFilter>,
}

impl EventStreamBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: EventStreamConfig::default(),
            filter: None,
        }
    }

    /// Set buffer size
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.config.buffer_size = size;
        self
    }

    /// Include historical events
    pub fn include_historical(mut self, include: bool) -> Self {
        self.config.include_historical = include;
        self
    }

    /// Add event type filter
    pub fn include_types(mut self, types: Vec<EventType>) -> Self {
        let filter = self.filter.get_or_insert_with(|| StreamFilter {
            include_types: None,
            exclude_types: None,
            include_categories: None,
            exclude_categories: None,
            critical_only: false,
        });
        filter.include_types = Some(types);
        self
    }

    /// Exclude event types
    pub fn exclude_types(mut self, types: Vec<EventType>) -> Self {
        let filter = self.filter.get_or_insert_with(|| StreamFilter {
            include_types: None,
            exclude_types: None,
            include_categories: None,
            exclude_categories: None,
            critical_only: false,
        });
        filter.exclude_types = Some(types);
        self
    }

    /// Include event categories
    pub fn include_categories(mut self, categories: Vec<EventCategory>) -> Self {
        let filter = self.filter.get_or_insert_with(|| StreamFilter {
            include_types: None,
            exclude_types: None,
            include_categories: None,
            exclude_categories: None,
            critical_only: false,
        });
        filter.include_categories = Some(categories);
        self
    }

    /// Only critical events
    pub fn critical_only(mut self, critical: bool) -> Self {
        let filter = self.filter.get_or_insert_with(|| StreamFilter {
            include_types: None,
            exclude_types: None,
            include_categories: None,
            exclude_categories: None,
            critical_only: false,
        });
        filter.critical_only = critical;
        self
    }

    /// Build the stream from a broadcast receiver
    pub fn build(mut self, receiver: broadcast::Receiver<Event>) -> EventStream {
        self.config.filter = self.filter;
        EventStream::new(self.config, receiver)
    }

    /// Build a filtered stream from a broadcast receiver
    pub fn build_filtered(self, receiver: broadcast::Receiver<Event>) -> HelixResult<FilteredEventStream> {
        let filter = self.filter.ok_or_else(|| HelixError::validation("No filter specified"))?;
        let stream = EventStream::new(self.config, receiver);
        Ok(FilteredEventStream::new(stream, filter))
    }
}

impl Default for EventStreamBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Event, EventType, EventData};
    use tokio_stream::StreamExt as TokioStreamExt;

    #[tokio::test]
    async fn test_event_stream_filtering() {
        let (tx, rx) = broadcast::channel(10);
        
        let filter = StreamFilter {
            include_types: Some(vec![EventType::SystemStarted]),
            exclude_types: None,
            include_categories: None,
            exclude_categories: None,
            critical_only: false,
        };

        let config = EventStreamConfig::default();
        let mut stream = EventStream::filtered(config, rx, filter);

        // Send matching event
        let event1 = Event::new(
            EventType::SystemStarted,
            EventData::SystemStatus {
                status: "started".to_string(),
                details: serde_json::Value::Null,
            },
        );
        tx.send(event1.clone()).unwrap();

        // Send non-matching event
        let event2 = Event::new(
            EventType::SystemStopped,
            EventData::SystemStatus {
                status: "stopped".to_string(),
                details: serde_json::Value::Null,
            },
        );
        tx.send(event2).unwrap();

        // Should only receive the matching event
        if let Some(Ok(received)) = stream.next().await {
            assert_eq!(received.id, event1.id);
        } else {
            panic!("Expected to receive an event");
        }
    }
}
