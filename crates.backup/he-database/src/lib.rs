//! Database connection and models for HackerExperience

use anyhow::Result;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use tracing::info;

pub mod models;
pub mod queries;
pub mod queries_optimized;
pub mod cache;
pub mod batch_queries;
pub mod redis_cache;

#[cfg(test)]
mod tests;

pub use models::*;
pub use queries::*;
pub use queries_optimized::*;
pub use cache::*;
pub use batch_queries::*;
pub use redis_cache::{RedisCache, RedisCacheConfig, QueryCache};

/// Database connection pool
#[derive(Debug, Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    /// Create a new database connection pool with production-ready settings
    pub async fn new(database_url: &str) -> Result<Self> {
        // Scale connection pool for production workloads
        let max_connections = std::env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .unwrap_or(100);

        let min_connections = std::env::var("DB_MIN_CONNECTIONS")
            .unwrap_or_else(|_| "20".to_string())
            .parse()
            .unwrap_or(20);

        let pool = PgPoolOptions::new()
            .max_connections(max_connections)  // Increased from 20 to 100
            .min_connections(min_connections)   // Increased from 5 to 20
            .acquire_timeout(Duration::from_secs(5))  // Increased from 3 to 5 seconds
            .idle_timeout(Duration::from_secs(300))   // Connection idle for 5 minutes
            .max_lifetime(Duration::from_secs(3600))  // Connection lifetime of 1 hour
            .test_before_acquire(true)  // Test connection health before use
            .connect(database_url)
            .await?;

        info!(
            "Database connection pool established (max: {}, min: {})",
            max_connections, min_connections
        );

        Ok(Self { pool })
    }

    /// Create a database connection pool with custom configuration
    pub async fn new_with_config(
        database_url: &str,
        max_connections: u32,
        min_connections: u32,
    ) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .min_connections(min_connections)
            .acquire_timeout(Duration::from_secs(5))
            .idle_timeout(Duration::from_secs(300))
            .max_lifetime(Duration::from_secs(3600))
            .test_before_acquire(true)
            .connect(database_url)
            .await?;

        info!(
            "Database connection pool established (max: {}, min: {})",
            max_connections, min_connections
        );

        Ok(Self { pool })
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("../../migrations-postgres")
            .run(&self.pool)
            .await?;

        info!("Database migrations completed");
        Ok(())
    }

    /// Get a reference to the pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}