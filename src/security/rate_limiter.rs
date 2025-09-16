use actix_web::Error;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

/// Rate limiter implementation
pub struct RateLimiter {
    max_requests: u32,
    window_seconds: i64,
    requests: Arc<RwLock<HashMap<String, Vec<DateTime<Utc>>>>>,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_seconds: i64) -> Self {
        Self {
            max_requests,
            window_seconds,
            requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_limit(&self, identifier: &str) -> Result<(), Error> {
        let now = Utc::now();
        let window_start = now - Duration::seconds(self.window_seconds);

        let mut requests = self.requests.write().await;
        let entry = requests.entry(identifier.to_string()).or_insert_with(Vec::new);

        // Remove old requests outside the window
        entry.retain(|&timestamp| timestamp > window_start);

        // Check if limit exceeded
        if entry.len() >= self.max_requests as usize {
            return Err(actix_web::error::ErrorTooManyRequests(
                format!("Rate limit exceeded. Max {} requests per {} seconds",
                    self.max_requests, self.window_seconds)
            ));
        }

        // Add current request
        entry.push(now);
        Ok(())
    }

    pub async fn reset(&self, identifier: &str) {
        let mut requests = self.requests.write().await;
        requests.remove(identifier);
    }

    pub async fn clean_old_entries(&self) {
        let now = Utc::now();
        let window_start = now - Duration::seconds(self.window_seconds);

        let mut requests = self.requests.write().await;
        requests.retain(|_, timestamps| {
            timestamps.retain(|&timestamp| timestamp > window_start);
            !timestamps.is_empty()
        });
    }
}