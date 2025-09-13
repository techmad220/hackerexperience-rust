//! Listener system for tracking objects and events
//!
//! This module implements the Helix listener pattern for tracking when events
//! occur on specific objects and triggering callbacks.

use crate::{HelixError, HelixId, HelixResult, Meta};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Unique identifier for a listener
pub type ListenerId = HelixId;

/// Object identifier being tracked (can be any string-representable ID)
pub type ObjectId = String;

/// Event name that triggers the listener
pub type EventName = String;

/// Callback information for when a listener is triggered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackInfo {
    /// Module containing the callback function
    pub module: String,
    /// Method name to call
    pub method: String,
    /// Additional metadata to pass to the callback
    pub meta: Meta,
}

/// A registered listener
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Listener {
    /// Unique identifier for this listener
    pub listener_id: ListenerId,
    /// The object being tracked
    pub object_id: ObjectId,
    /// The event name that triggers this listener
    pub event: EventName,
    /// Callback information
    pub callback: CallbackInfo,
}

impl Listener {
    pub fn new(
        object_id: ObjectId,
        event: EventName,
        module: String,
        method: String,
        meta: Option<Meta>,
    ) -> Self {
        Self {
            listener_id: Uuid::new_v4(),
            object_id,
            event,
            callback: CallbackInfo {
                module,
                method,
                meta: meta.unwrap_or_default(),
            },
        }
    }
}

/// In-memory listener registry
#[derive(Debug, Default)]
pub struct ListenerRegistry {
    /// Listeners indexed by (object_id, event) for fast lookup
    listeners: Arc<RwLock<HashMap<(ObjectId, EventName), Vec<Listener>>>>,
    /// All listeners indexed by their ID
    by_id: Arc<RwLock<HashMap<ListenerId, Listener>>>,
}

impl ListenerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new listener
    pub async fn register(&self, listener: Listener) -> HelixResult<ListenerId> {
        let key = (listener.object_id.clone(), listener.event.clone());
        let listener_id = listener.listener_id;

        {
            let mut listeners = self.listeners.write().await;
            listeners.entry(key).or_default().push(listener.clone());
        }

        {
            let mut by_id = self.by_id.write().await;
            by_id.insert(listener_id, listener);
        }

        Ok(listener_id)
    }

    /// Remove a listener by ID
    pub async fn unregister(&self, listener_id: ListenerId) -> HelixResult<()> {
        let listener = {
            let mut by_id = self.by_id.write().await;
            by_id
                .remove(&listener_id)
                .ok_or_else(|| HelixError::not_found("Listener not found"))?
        };

        let key = (listener.object_id, listener.event);
        let mut listeners = self.listeners.write().await;
        
        if let Some(list) = listeners.get_mut(&key) {
            list.retain(|l| l.listener_id != listener_id);
            if list.is_empty() {
                listeners.remove(&key);
            }
        }

        Ok(())
    }

    /// Get all listeners for a specific object and event
    pub async fn get_listeners(
        &self,
        object_id: &ObjectId,
        event: &EventName,
    ) -> Vec<Listener> {
        let listeners = self.listeners.read().await;
        let key = (object_id.clone(), event.clone());
        
        listeners
            .get(&key)
            .map(|l| l.clone())
            .unwrap_or_default()
    }

    /// Get a listener by ID
    pub async fn get_listener(&self, listener_id: ListenerId) -> Option<Listener> {
        let by_id = self.by_id.read().await;
        by_id.get(&listener_id).cloned()
    }

    /// Get all listeners for an object (any event)
    pub async fn get_listeners_for_object(&self, object_id: &ObjectId) -> Vec<Listener> {
        let listeners = self.listeners.read().await;
        let mut result = Vec::new();

        for ((obj_id, _), listener_list) in listeners.iter() {
            if obj_id == object_id {
                result.extend(listener_list.iter().cloned());
            }
        }

        result
    }

    /// Trigger all listeners for a specific object and event
    pub async fn trigger(
        &self,
        object_id: &ObjectId,
        event: &EventName,
        event_data: Meta,
    ) -> HelixResult<Vec<CallbackInfo>> {
        let listeners = self.get_listeners(object_id, event).await;
        
        let mut callbacks = Vec::new();
        for listener in listeners {
            let mut callback = listener.callback;
            // Merge event data with listener meta
            for (key, value) in event_data.iter() {
                callback.meta.insert(key.clone(), value.clone());
            }
            callbacks.push(callback);
        }

        Ok(callbacks)
    }

    /// Clear all listeners
    pub async fn clear(&self) {
        let mut listeners = self.listeners.write().await;
        let mut by_id = self.by_id.write().await;
        listeners.clear();
        by_id.clear();
    }

    /// Get total number of registered listeners
    pub async fn count(&self) -> usize {
        let by_id = self.by_id.read().await;
        by_id.len()
    }
}

/// Builder for creating listeners with a fluent API
pub struct ListenerBuilder {
    object_id: Option<ObjectId>,
    event: Option<EventName>,
    module: Option<String>,
    method: Option<String>,
    meta: Meta,
}

impl ListenerBuilder {
    pub fn new() -> Self {
        Self {
            object_id: None,
            event: None,
            module: None,
            method: None,
            meta: HashMap::new(),
        }
    }

    pub fn object_id<S: Into<String>>(mut self, object_id: S) -> Self {
        self.object_id = Some(object_id.into());
        self
    }

    pub fn event<S: Into<String>>(mut self, event: S) -> Self {
        self.event = Some(event.into());
        self
    }

    pub fn callback<M, F>(mut self, module: M, method: F) -> Self
    where
        M: Into<String>,
        F: Into<String>,
    {
        self.module = Some(module.into());
        self.method = Some(method.into());
        self
    }

    pub fn meta<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<serde_json::Value>,
    {
        self.meta.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> HelixResult<Listener> {
        let object_id = self
            .object_id
            .ok_or_else(|| HelixError::validation("object_id is required"))?;
        let event = self
            .event
            .ok_or_else(|| HelixError::validation("event is required"))?;
        let module = self
            .module
            .ok_or_else(|| HelixError::validation("module is required"))?;
        let method = self
            .method
            .ok_or_else(|| HelixError::validation("method is required"))?;

        Ok(Listener::new(object_id, event, module, method, Some(self.meta)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_listener_registry() {
        let registry = ListenerRegistry::new();

        let listener = ListenerBuilder::new()
            .object_id("test_object")
            .event("test_event")
            .callback("TestModule", "test_method")
            .meta("key", "value")
            .build()
            .unwrap();

        let listener_id = registry.register(listener.clone()).await.unwrap();
        assert_eq!(listener_id, listener.listener_id);

        let found = registry.get_listener(listener_id).await;
        assert!(found.is_some());

        let listeners = registry
            .get_listeners(&"test_object".to_string(), &"test_event".to_string())
            .await;
        assert_eq!(listeners.len(), 1);

        let callbacks = registry
            .trigger(
                &"test_object".to_string(),
                &"test_event".to_string(),
                HashMap::new(),
            )
            .await
            .unwrap();
        assert_eq!(callbacks.len(), 1);

        registry.unregister(listener_id).await.unwrap();
        let found = registry.get_listener(listener_id).await;
        assert!(found.is_none());
    }
}