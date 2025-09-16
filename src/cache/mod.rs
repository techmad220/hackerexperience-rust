use redis::{aio::ConnectionManager, AsyncCommands, RedisError};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Cache service for Redis operations
#[derive(Clone)]
pub struct CacheService {
    conn: ConnectionManager,
    default_ttl: u64,
}

impl CacheService {
    /// Create new cache service
    pub async fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = redis::Client::open(redis_url)?;
        let conn = ConnectionManager::new(client).await?;

        Ok(Self {
            conn,
            default_ttl: 3600, // 1 hour default
        })
    }

    /// Get value from cache
    pub async fn get<T>(&mut self, key: &str) -> Result<Option<T>, RedisError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let value: Option<String> = self.conn.get(key).await?;

        match value {
            Some(json) => {
                let deserialized = serde_json::from_str(&json)
                    .map_err(|e| RedisError::from((redis::ErrorKind::TypeError, "Deserialization failed", e.to_string())))?;
                Ok(Some(deserialized))
            }
            None => Ok(None)
        }
    }

    /// Set value in cache with TTL
    pub async fn set<T>(&mut self, key: &str, value: &T, ttl_seconds: Option<u64>) -> Result<(), RedisError>
    where
        T: Serialize,
    {
        let json = serde_json::to_string(value)
            .map_err(|e| RedisError::from((redis::ErrorKind::TypeError, "Serialization failed", e.to_string())))?;

        let ttl = ttl_seconds.unwrap_or(self.default_ttl);
        self.conn.set_ex(key, json, ttl).await?;
        Ok(())
    }

    /// Delete key from cache
    pub async fn delete(&mut self, key: &str) -> Result<(), RedisError> {
        self.conn.del(key).await?;
        Ok(())
    }

    /// Check if key exists
    pub async fn exists(&mut self, key: &str) -> Result<bool, RedisError> {
        self.conn.exists(key).await
    }

    /// Set with expiration
    pub async fn set_with_expiry<T>(&mut self, key: &str, value: &T, duration: Duration) -> Result<(), RedisError>
    where
        T: Serialize,
    {
        let json = serde_json::to_string(value)
            .map_err(|e| RedisError::from((redis::ErrorKind::TypeError, "Serialization failed", e.to_string())))?;

        self.conn.set_ex(key, json, duration.as_secs()).await?;
        Ok(())
    }

    /// Increment counter
    pub async fn increment(&mut self, key: &str) -> Result<i64, RedisError> {
        self.conn.incr(key, 1).await
    }

    /// Decrement counter
    pub async fn decrement(&mut self, key: &str) -> Result<i64, RedisError> {
        self.conn.decr(key, 1).await
    }

    /// Get multiple keys at once
    pub async fn get_multiple<T>(&mut self, keys: &[String]) -> Result<Vec<Option<T>>, RedisError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let values: Vec<Option<String>> = self.conn.get(keys).await?;

        let mut results = Vec::new();
        for value in values {
            match value {
                Some(json) => {
                    let deserialized = serde_json::from_str(&json)
                        .map_err(|e| RedisError::from((redis::ErrorKind::TypeError, "Deserialization failed", e.to_string())))?;
                    results.push(Some(deserialized));
                }
                None => results.push(None)
            }
        }

        Ok(results)
    }

    /// Set hash field
    pub async fn hset(&mut self, key: &str, field: &str, value: &str) -> Result<(), RedisError> {
        self.conn.hset(key, field, value).await
    }

    /// Get hash field
    pub async fn hget(&mut self, key: &str, field: &str) -> Result<Option<String>, RedisError> {
        self.conn.hget(key, field).await
    }

    /// Get all hash fields
    pub async fn hgetall(&mut self, key: &str) -> Result<std::collections::HashMap<String, String>, RedisError> {
        self.conn.hgetall(key).await
    }

    /// Add to set
    pub async fn sadd(&mut self, key: &str, member: &str) -> Result<(), RedisError> {
        self.conn.sadd(key, member).await
    }

    /// Check if member exists in set
    pub async fn sismember(&mut self, key: &str, member: &str) -> Result<bool, RedisError> {
        self.conn.sismember(key, member).await
    }

    /// Get all set members
    pub async fn smembers(&mut self, key: &str) -> Result<Vec<String>, RedisError> {
        self.conn.smembers(key).await
    }

    /// Push to list
    pub async fn lpush(&mut self, key: &str, value: &str) -> Result<(), RedisError> {
        self.conn.lpush(key, value).await
    }

    /// Get list range
    pub async fn lrange(&mut self, key: &str, start: isize, stop: isize) -> Result<Vec<String>, RedisError> {
        self.conn.lrange(key, start, stop).await
    }

    /// Set TTL on existing key
    pub async fn expire(&mut self, key: &str, seconds: i64) -> Result<bool, RedisError> {
        self.conn.expire(key, seconds).await
    }

    /// Get TTL of key
    pub async fn ttl(&mut self, key: &str) -> Result<i64, RedisError> {
        self.conn.ttl(key).await
    }

    /// Clear all cache (use with caution)
    pub async fn flush_all(&mut self) -> Result<(), RedisError> {
        redis::cmd("FLUSHALL").query_async(&mut self.conn).await
    }
}

/// Cache keys structure for consistency
pub struct CacheKeys;

impl CacheKeys {
    /// User cache keys
    pub fn user(user_id: u64) -> String {
        format!("user:{}", user_id)
    }

    pub fn user_session(session_id: &str) -> String {
        format!("session:{}", session_id)
    }

    pub fn user_processes(user_id: u64) -> String {
        format!("user:{}:processes", user_id)
    }

    /// Server cache keys
    pub fn server(server_id: u64) -> String {
        format!("server:{}", server_id)
    }

    pub fn server_by_ip(ip: &str) -> String {
        format!("server:ip:{}", ip)
    }

    pub fn server_logs(server_id: u64) -> String {
        format!("server:{}:logs", server_id)
    }

    /// Process cache keys
    pub fn process(process_id: u64) -> String {
        format!("process:{}", process_id)
    }

    pub fn process_queue() -> String {
        "process:queue".to_string()
    }

    /// Mission cache keys
    pub fn mission(mission_id: u64) -> String {
        format!("mission:{}", mission_id)
    }

    pub fn mission_progress(user_id: u64, mission_id: u64) -> String {
        format!("mission:{}:{}", user_id, mission_id)
    }

    /// Clan cache keys
    pub fn clan(clan_id: u64) -> String {
        format!("clan:{}", clan_id)
    }

    pub fn clan_members(clan_id: u64) -> String {
        format!("clan:{}:members", clan_id)
    }

    pub fn clan_war(war_id: u64) -> String {
        format!("clan:war:{}", war_id)
    }

    /// Ranking cache keys
    pub fn ranking_global() -> String {
        "ranking:global".to_string()
    }

    pub fn ranking_clan() -> String {
        "ranking:clan".to_string()
    }

    pub fn ranking_category(category: &str) -> String {
        format!("ranking:{}", category)
    }

    /// Rate limiting keys
    pub fn rate_limit(identifier: &str, action: &str) -> String {
        format!("ratelimit:{}:{}", identifier, action)
    }

    /// Lock keys for distributed locking
    pub fn lock(resource: &str) -> String {
        format!("lock:{}", resource)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_operations() {
        // This would require a Redis instance to run
        // Placeholder for testing cache operations
    }

    #[test]
    fn test_cache_key_generation() {
        assert_eq!(CacheKeys::user(123), "user:123");
        assert_eq!(CacheKeys::server_by_ip("192.168.1.1"), "server:ip:192.168.1.1");
        assert_eq!(CacheKeys::process_queue(), "process:queue");
    }
}