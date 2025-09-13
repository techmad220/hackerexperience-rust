//! Helix Henforcer - Authorization and Validation System
//! 
//! The Henforcer Architecture provides a sophisticated building-block system for
//! authorization and validation checks. Each henforcer function verifies specific
//! assumptions about objects/elements and can be composed with other henforcers.
//!
//! Key principles:
//! - Each henforcer verifies one specific thing
//! - Henforcers can be composed as building blocks
//! - Efficient relay system prevents redundant database queries
//! - Consistent return types for composability
//!
//! # Architecture
//!
//! ## Input Types
//! Henforcers accept either:
//! - Object ID (`ObjectId`) - assumes object needs verification
//! - Object itself (`Object`) - assumes object already verified
//!
//! ## Return Types
//! Success: `HenforcerResult::Ok(relay)`
//! Failure: `HenforcerResult::Err(reason, relay)`
//!
//! The `relay` contains related data fetched during verification,
//! passed upstream to prevent redundant queries.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Result type for henforcer operations
#[derive(Debug, Clone, PartialEq)]
pub enum HenforcerResult<T, E> {
    /// Success with relay data
    Ok(T),
    /// Failure with reason and partial relay data
    Err(E, T),
}

impl<T, E> HenforcerResult<T, E> {
    /// Returns true if the result is Ok
    pub fn is_ok(&self) -> bool {
        matches!(self, HenforcerResult::Ok(_))
    }

    /// Returns true if the result is Err
    pub fn is_err(&self) -> bool {
        matches!(self, HenforcerResult::Err(_, _))
    }

    /// Unwrap the relay data regardless of success/failure
    pub fn relay(self) -> T {
        match self {
            HenforcerResult::Ok(relay) | HenforcerResult::Err(_, relay) => relay,
        }
    }

    /// Map the relay type
    pub fn map_relay<U, F>(self, f: F) -> HenforcerResult<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            HenforcerResult::Ok(relay) => HenforcerResult::Ok(f(relay)),
            HenforcerResult::Err(err, relay) => HenforcerResult::Err(err, f(relay)),
        }
    }

    /// Chain another henforcer operation
    pub fn and_then<U, F>(self, f: F) -> HenforcerResult<U, E>
    where
        F: FnOnce(T) -> HenforcerResult<U, E>,
    {
        match self {
            HenforcerResult::Ok(relay) => f(relay),
            HenforcerResult::Err(err, relay) => match f(relay) {
                HenforcerResult::Ok(new_relay) => HenforcerResult::Err(err, new_relay),
                HenforcerResult::Err(_, new_relay) => HenforcerResult::Err(err, new_relay),
            },
        }
    }
}

/// Relay data type - used to pass related objects between henforcers
pub type Relay = HashMap<String, serde_json::Value>;

/// Henforcer error reasons
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Error)]
pub enum HenforcerError {
    #[error("Object not found: {object_type} {id}")]
    NotFound { object_type: String, id: String },
    
    #[error("Access denied: {reason}")]
    AccessDenied { reason: String },
    
    #[error("Invalid state: {reason}")]
    InvalidState { reason: String },
    
    #[error("Insufficient resources: {resource} required: {required}, available: {available}")]
    InsufficientResources {
        resource: String,
        required: u64,
        available: u64,
    },
    
    #[error("Validation failed: {field} - {reason}")]
    ValidationFailed { field: String, reason: String },
    
    #[error("Custom error: {reason}")]
    Custom { reason: String },
}

/// Standard henforcer result type
pub type StandardResult = HenforcerResult<Relay, HenforcerError>;

/// Trait for henforcer operations
#[async_trait]
pub trait Henforcer {
    type Input;
    type Error: Into<HenforcerError>;
    
    /// Perform the henforcer check
    async fn check(&self, input: Self::Input) -> HenforcerResult<Relay, Self::Error>;
}

/// Henforcer macro-like function for chaining operations
pub async fn henforce<F, Fut, T, E>(
    henforcer_fn: F,
    relay: Relay,
) -> HenforcerResult<Relay, E>
where
    F: FnOnce(Relay) -> Fut,
    Fut: Future<Output = HenforcerResult<T, E>>,
    T: Into<Relay>,
{
    match henforcer_fn(relay).await {
        HenforcerResult::Ok(new_relay) => HenforcerResult::Ok(new_relay.into()),
        HenforcerResult::Err(err, partial_relay) => HenforcerResult::Err(err, partial_relay.into()),
    }
}

/// Helper to create success result
pub fn reply_ok(relay: Relay) -> StandardResult {
    HenforcerResult::Ok(relay)
}

/// Helper to create error result
pub fn reply_error(error: HenforcerError, relay: Relay) -> StandardResult {
    HenforcerResult::Err(error, relay)
}

/// Merge multiple relay maps
pub fn merge_relays(relays: Vec<Relay>) -> Relay {
    relays.into_iter().fold(HashMap::new(), |mut acc, relay| {
        acc.extend(relay);
        acc
    })
}

/// Add a key-value pair to relay
pub fn add_to_relay(mut relay: Relay, key: impl Into<String>, value: impl Serialize) -> Relay {
    let serialized_value = serde_json::to_value(value)
        .unwrap_or_else(|_| serde_json::Value::Null);
    relay.insert(key.into(), serialized_value);
    relay
}

/// Get a value from relay and remove it
pub fn get_and_drop<T>(mut relay: Relay, key: &str) -> Result<(Relay, T), serde_json::Error>
where
    T: for<'de> Deserialize<'de>,
{
    let value = relay.remove(key)
        .ok_or_else(|| serde_json::Error::custom(format!("Key '{}' not found in relay", key)))?;
    let typed_value = serde_json::from_value(value)?;
    Ok((relay, typed_value))
}

/// Replace a key in relay with a new key
pub fn replace_key(mut relay: Relay, old_key: &str, new_key: impl Into<String>) -> Relay {
    if let Some(value) = relay.remove(old_key) {
        relay.insert(new_key.into(), value);
    }
    relay
}

/// Drop a key from relay
pub fn drop_key(mut relay: Relay, key: &str) -> Relay {
    relay.remove(key);
    relay
}

/// Negate a henforcer result
pub fn henforce_not<T, E>(
    result: HenforcerResult<T, E>,
    error: E,
) -> HenforcerResult<T, E> {
    match result {
        HenforcerResult::Ok(relay) => HenforcerResult::Err(error, relay),
        HenforcerResult::Err(_, relay) => HenforcerResult::Ok(relay),
    }
}

/// Common henforcer implementations
pub mod common {
    use super::*;

    /// Check if an object exists by ID
    pub async fn object_exists<T>(
        object_type: &str,
        id: Uuid,
        fetch_fn: impl Fn(Uuid) -> Pin<Box<dyn Future<Output = Option<T>> + Send>>,
    ) -> StandardResult
    where
        T: Serialize + Send,
    {
        match fetch_fn(id).await {
            Some(obj) => {
                let mut relay = Relay::new();
                relay.insert(object_type.to_string(), serde_json::to_value(obj).unwrap());
                reply_ok(relay)
            }
            None => reply_error(
                HenforcerError::NotFound {
                    object_type: object_type.to_string(),
                    id: id.to_string(),
                },
                Relay::new(),
            ),
        }
    }

    /// Check if user has permission
    pub async fn has_permission(
        user_id: Uuid,
        resource: &str,
        action: &str,
        permission_check: impl Fn(Uuid, &str, &str) -> Pin<Box<dyn Future<Output = bool> + Send>>,
    ) -> StandardResult {
        if permission_check(user_id, resource, action).await {
            let mut relay = Relay::new();
            relay.insert("user_id".to_string(), serde_json::to_value(user_id).unwrap());
            reply_ok(relay)
        } else {
            reply_error(
                HenforcerError::AccessDenied {
                    reason: format!("User {} cannot {} on {}", user_id, action, resource),
                },
                Relay::new(),
            )
        }
    }

    /// Check if resource has sufficient capacity
    pub async fn has_sufficient_resources(
        resource_name: &str,
        required: u64,
        available: u64,
    ) -> StandardResult {
        if available >= required {
            let mut relay = Relay::new();
            relay.insert("available".to_string(), serde_json::to_value(available).unwrap());
            relay.insert("required".to_string(), serde_json::to_value(required).unwrap());
            reply_ok(relay)
        } else {
            reply_error(
                HenforcerError::InsufficientResources {
                    resource: resource_name.to_string(),
                    required,
                    available,
                },
                Relay::new(),
            )
        }
    }
}

/// Macro for easy henforcer chaining (similar to Elixir's `henforce` macro)
#[macro_export]
macro_rules! henforce {
    ($henforcer:expr, $block:block) => {
        match $henforcer.await {
            $crate::HenforcerResult::Ok(relay) => {
                let result = {
                    let _ = &relay; // Make relay available in block
                    $block
                };
                match result {
                    $crate::HenforcerResult::Ok(new_relay) => {
                        $crate::HenforcerResult::Ok($crate::merge_relays(vec![relay, new_relay]))
                    }
                    $crate::HenforcerResult::Err(err, partial_relay) => {
                        $crate::HenforcerResult::Err(err, $crate::merge_relays(vec![relay, partial_relay]))
                    }
                }
            }
            $crate::HenforcerResult::Err(err, relay) => $crate::HenforcerResult::Err(err, relay),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_henforcer_result_ok() {
        let result: StandardResult = reply_ok(Relay::new());
        assert!(result.is_ok());
        assert!(!result.is_err());
    }

    #[tokio::test]
    async fn test_henforcer_result_error() {
        let error = HenforcerError::Custom {
            reason: "test error".to_string(),
        };
        let result = reply_error(error, Relay::new());
        assert!(result.is_err());
        assert!(!result.is_ok());
    }

    #[tokio::test]
    async fn test_merge_relays() {
        let mut relay1 = Relay::new();
        relay1.insert("key1".to_string(), serde_json::Value::String("value1".to_string()));
        
        let mut relay2 = Relay::new();
        relay2.insert("key2".to_string(), serde_json::Value::String("value2".to_string()));
        
        let merged = merge_relays(vec![relay1, relay2]);
        
        assert_eq!(merged.len(), 2);
        assert!(merged.contains_key("key1"));
        assert!(merged.contains_key("key2"));
    }

    #[tokio::test]
    async fn test_add_to_relay() {
        let relay = Relay::new();
        let updated = add_to_relay(relay, "test_key", "test_value");
        
        assert_eq!(updated.len(), 1);
        assert_eq!(
            updated.get("test_key"),
            Some(&serde_json::Value::String("test_value".to_string()))
        );
    }

    #[tokio::test]
    async fn test_replace_key() {
        let mut relay = Relay::new();
        relay.insert("old_key".to_string(), serde_json::Value::String("value".to_string()));
        
        let updated = replace_key(relay, "old_key", "new_key");
        
        assert!(!updated.contains_key("old_key"));
        assert!(updated.contains_key("new_key"));
        assert_eq!(
            updated.get("new_key"),
            Some(&serde_json::Value::String("value".to_string()))
        );
    }
}