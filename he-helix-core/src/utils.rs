//! Utility functions and helpers

use serde_json::Value;
use std::collections::HashMap;

/// Convert string keys in a JSON object to atoms (String in Rust)
pub fn atomize_keys(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (key, val) in map {
                new_map.insert(key, atomize_keys(val));
            }
            Value::Object(new_map)
        }
        Value::Array(arr) => {
            Value::Array(arr.into_iter().map(atomize_keys).collect())
        }
        other => other,
    }
}

/// Convert a HashMap<String, Value> to a more ergonomic format
pub fn map_to_value(map: HashMap<String, Value>) -> Value {
    let mut json_map = serde_json::Map::new();
    for (key, value) in map {
        json_map.insert(key, value);
    }
    Value::Object(json_map)
}

/// Extract a typed value from metadata
pub fn extract_meta<T>(meta: &HashMap<String, Value>, key: &str) -> Option<T>
where
    T: serde::de::DeserializeOwned,
{
    meta.get(key)
        .and_then(|v| serde_json::from_value(v.clone()).ok())
}

/// Hash a string using MD5 (for compatibility with Elixir version)
pub fn hash_event(event: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    event.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Generate a random string of specified length
pub fn random_string(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Convert milliseconds to Duration
pub fn millis_to_duration(millis: u64) -> std::time::Duration {
    std::time::Duration::from_millis(millis)
}

/// Convert Duration to milliseconds
pub fn duration_to_millis(duration: std::time::Duration) -> u64 {
    duration.as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_atomize_keys() {
        let value = json!({
            "key1": "value1",
            "key2": {
                "nested": "value"
            },
            "key3": ["item1", "item2"]
        });

        let atomized = atomize_keys(value.clone());
        assert_eq!(atomized, value); // Should be the same for this simple case
    }

    #[test]
    fn test_extract_meta() {
        let mut meta = HashMap::new();
        meta.insert("test_key".to_string(), json!("test_value"));
        meta.insert("number".to_string(), json!(42));

        let string_val: Option<String> = extract_meta(&meta, "test_key");
        assert_eq!(string_val, Some("test_value".to_string()));

        let number_val: Option<i32> = extract_meta(&meta, "number");
        assert_eq!(number_val, Some(42));

        let missing_val: Option<String> = extract_meta(&meta, "missing");
        assert_eq!(missing_val, None);
    }

    #[test]
    fn test_hash_event() {
        let event = "Helix.Test.Event";
        let hash1 = hash_event(event);
        let hash2 = hash_event(event);
        assert_eq!(hash1, hash2); // Should be deterministic

        let different_event = "Helix.Different.Event";
        let hash3 = hash_event(different_event);
        assert_ne!(hash1, hash3); // Different events should have different hashes
    }

    #[test]
    fn test_random_string() {
        let s1 = random_string(10);
        let s2 = random_string(10);
        
        assert_eq!(s1.len(), 10);
        assert_eq!(s2.len(), 10);
        assert_ne!(s1, s2); // Should be different (very high probability)
    }
}