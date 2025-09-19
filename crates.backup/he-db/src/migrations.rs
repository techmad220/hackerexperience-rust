// Migration utilities for HackerExperience database schema

use sqlx::{MySql, Pool, migrate::MigrateDatabase};
use anyhow::Result;

// Create database if it doesn't exist
pub async fn create_database_if_not_exists(database_url: &str) -> Result<()> {
    if !MySql::database_exists(database_url).await.unwrap_or(false) {
        tracing::info!("Creating database...");
        MySql::create_database(database_url).await?;
    }
    Ok(())
}

// Run all migrations
pub async fn run_migrations(pool: &Pool<MySql>) -> Result<()> {
    tracing::info!("Running database migrations...");
    
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await?;
        
    tracing::info!("Database migrations completed successfully");
    Ok(())
}

// Check if database is ready
pub async fn check_database_health(pool: &Pool<MySql>) -> Result<bool> {
    let result = sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await;
        
    Ok(result.is_ok())
}