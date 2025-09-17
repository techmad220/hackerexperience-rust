//! Database connection and models for HackerExperience

use anyhow::Result;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use tracing::info;

pub mod models;
pub mod queries;
pub mod queries_optimized;
pub mod cache;

#[cfg(test)]
mod tests;

pub use models::*;
pub use queries::*;
pub use queries_optimized::*;
pub use cache::*;

/// Database connection pool
#[derive(Debug, Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    /// Create a new database connection pool
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .min_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(database_url)
            .await?;

        info!("Database connection pool established");

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