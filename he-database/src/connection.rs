//! Database connection management for Helix

use crate::config::{DatabaseConnectionConfig, DatabaseType, ConnectionPoolConfig};
use he_helix_core::{HelixError, HelixResult};
use sqlx::{Pool, Postgres, MySql, Sqlite};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use dashmap::DashMap;

/// Unified database connection wrapper
#[derive(Debug, Clone)]
pub enum DatabaseConnection {
    PostgreSQL(sqlx::PgPool),
    MySQL(sqlx::MySqlPool),
    SQLite(sqlx::SqlitePool),
}

impl DatabaseConnection {
    /// Create a new database connection from configuration
    pub async fn new(config: &DatabaseConnectionConfig) -> HelixResult<Self> {
        let pool_config = config.pool.as_ref().unwrap_or(&ConnectionPoolConfig::default());
        
        match config.database_type {
            DatabaseType::PostgreSQL => {
                let pool = sqlx::postgres::PgPoolOptions::new()
                    .max_connections(pool_config.max_connections)
                    .min_connections(pool_config.min_connections)
                    .acquire_timeout(Duration::from_secs(pool_config.connect_timeout))
                    .idle_timeout(Duration::from_secs(pool_config.idle_timeout))
                    .max_lifetime(Duration::from_secs(pool_config.max_lifetime))
                    .connect(&config.url)
                    .await
                    .map_err(|e| HelixError::Database(e))?;
                    
                Ok(DatabaseConnection::PostgreSQL(pool))
            }
            DatabaseType::MySQL => {
                let pool = sqlx::mysql::MySqlPoolOptions::new()
                    .max_connections(pool_config.max_connections)
                    .min_connections(pool_config.min_connections)
                    .acquire_timeout(Duration::from_secs(pool_config.connect_timeout))
                    .idle_timeout(Duration::from_secs(pool_config.idle_timeout))
                    .max_lifetime(Duration::from_secs(pool_config.max_lifetime))
                    .connect(&config.url)
                    .await
                    .map_err(|e| HelixError::Database(e))?;
                    
                Ok(DatabaseConnection::MySQL(pool))
            }
            DatabaseType::SQLite => {
                let pool = sqlx::sqlite::SqlitePoolOptions::new()
                    .max_connections(pool_config.max_connections)
                    .min_connections(pool_config.min_connections)
                    .acquire_timeout(Duration::from_secs(pool_config.connect_timeout))
                    .idle_timeout(Duration::from_secs(pool_config.idle_timeout))
                    .max_lifetime(Duration::from_secs(pool_config.max_lifetime))
                    .connect(&config.url)
                    .await
                    .map_err(|e| HelixError::Database(e))?;
                    
                Ok(DatabaseConnection::SQLite(pool))
            }
        }
    }

    /// Get the database type
    pub fn database_type(&self) -> DatabaseType {
        match self {
            DatabaseConnection::PostgreSQL(_) => DatabaseType::PostgreSQL,
            DatabaseConnection::MySQL(_) => DatabaseType::MySQL,
            DatabaseConnection::SQLite(_) => DatabaseType::SQLite,
        }
    }

    /// Check if the connection is healthy
    pub async fn health_check(&self) -> HelixResult<()> {
        match self {
            DatabaseConnection::PostgreSQL(pool) => {
                sqlx::query("SELECT 1").execute(pool).await
                    .map_err(|e| HelixError::Database(e))?;
            }
            DatabaseConnection::MySQL(pool) => {
                sqlx::query("SELECT 1").execute(pool).await
                    .map_err(|e| HelixError::Database(e))?;
            }
            DatabaseConnection::SQLite(pool) => {
                sqlx::query("SELECT 1").execute(pool).await
                    .map_err(|e| HelixError::Database(e))?;
            }
        }
        Ok(())
    }

    /// Close the connection pool
    pub async fn close(&self) {
        match self {
            DatabaseConnection::PostgreSQL(pool) => pool.close().await,
            DatabaseConnection::MySQL(pool) => pool.close().await,
            DatabaseConnection::SQLite(pool) => pool.close().await,
        }
    }
}

/// Database connection pool manager for multiple databases
#[derive(Debug)]
pub struct DatabaseConnectionPool {
    /// Map of database name to connection
    connections: Arc<DashMap<String, DatabaseConnection>>,
    /// Connection health check status
    health_status: Arc<RwLock<DashMap<String, bool>>>,
}

impl DatabaseConnectionPool {
    /// Create a new connection pool
    pub fn new() -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
            health_status: Arc::new(RwLock::new(DashMap::new())),
        }
    }

    /// Add a database connection to the pool
    pub async fn add_connection(&self, name: String, config: &DatabaseConnectionConfig) -> HelixResult<()> {
        let connection = DatabaseConnection::new(config).await?;
        
        // Test the connection
        connection.health_check().await?;
        
        // Store the connection and mark as healthy
        self.connections.insert(name.clone(), connection);
        {
            let health_status = self.health_status.read().await;
            health_status.insert(name.clone(), true);
        }
        
        tracing::info!("Added database connection: {}", name);
        Ok(())
    }

    /// Get a database connection by name
    pub fn get_connection(&self, name: &str) -> Option<DatabaseConnection> {
        self.connections.get(name).map(|entry| entry.clone())
    }

    /// Get all connection names
    pub fn connection_names(&self) -> Vec<String> {
        self.connections.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Check health of all connections
    pub async fn health_check_all(&self) -> HelixResult<()> {
        let mut health_results = Vec::new();
        
        for entry in self.connections.iter() {
            let name = entry.key().clone();
            let connection = entry.value().clone();
            
            let health_result = connection.health_check().await;
            let is_healthy = health_result.is_ok();
            
            if let Err(ref e) = health_result {
                tracing::error!("Health check failed for database '{}': {}", name, e);
            }
            
            {
                let health_status = self.health_status.read().await;
                health_status.insert(name.clone(), is_healthy);
            }
            
            health_results.push((name, health_result));
        }
        
        // Return error if any connection is unhealthy
        let failed_connections: Vec<_> = health_results
            .into_iter()
            .filter_map(|(name, result)| result.err().map(|e| (name, e)))
            .collect();
            
        if !failed_connections.is_empty() {
            let error_msg = format!(
                "Health check failed for databases: {}",
                failed_connections
                    .iter()
                    .map(|(name, _)| name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            return Err(HelixError::Database(sqlx::Error::Configuration(error_msg.into())));
        }
        
        Ok(())
    }

    /// Get health status for a specific database
    pub async fn is_healthy(&self, name: &str) -> bool {
        let health_status = self.health_status.read().await;
        health_status.get(name).map_or(false, |status| *status)
    }

    /// Get health status for all databases
    pub async fn health_status(&self) -> DashMap<String, bool> {
        let health_status = self.health_status.read().await;
        health_status.clone()
    }

    /// Close all connections
    pub async fn close_all(&self) -> HelixResult<()> {
        for entry in self.connections.iter() {
            let name = entry.key();
            let connection = entry.value();
            
            tracing::info!("Closing database connection: {}", name);
            connection.close().await;
        }
        
        self.connections.clear();
        {
            let health_status = self.health_status.read().await;
            health_status.clear();
        }
        
        Ok(())
    }

    /// Remove a specific connection
    pub async fn remove_connection(&self, name: &str) -> HelixResult<()> {
        if let Some((_, connection)) = self.connections.remove(name) {
            connection.close().await;
            
            let health_status = self.health_status.read().await;
            health_status.remove(name);
            
            tracing::info!("Removed database connection: {}", name);
        }
        
        Ok(())
    }
}