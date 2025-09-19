use sqlx::{MySqlPool, mysql::MySqlPoolOptions};
use anyhow::Result;
use crate::{DatabaseConfig, DbPool};

// Database connection management - replacing PHP PDO.class.php
pub struct Database {
    pool: DbPool,
}

impl Database {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let pool = MySqlPoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&config.connection_string())
            .await?;
            
        Ok(Self { pool })
    }
    
    pub async fn from_env() -> Result<Self> {
        let config = DatabaseConfig::from_env()?;
        Self::new(&config).await
    }
    
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }
    
    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::migrate!("../../migrations")
            .run(&self.pool)
            .await?;
        Ok(())
    }
    
    pub async fn close(self) {
        self.pool.close().await;
    }
    
    // Health check - equivalent to original PDO connection test
    pub async fn ping(&self) -> Result<bool> {
        let result = sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await;
            
        Ok(result.is_ok())
    }
}

// Factory method like original PDO_DB::factory()
pub async fn create_database_pool() -> Result<DbPool> {
    let config = DatabaseConfig::from_env()?;
    let pool = MySqlPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.connection_string())
        .await?;
    Ok(pool)
}