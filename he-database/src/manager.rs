//! Database manager for coordinating multiple database connections

use crate::config::{DatabaseConfig, DatabaseConnectionConfig};
use crate::connection::{DatabaseConnection, DatabaseConnectionPool};
use he_helix_core::{HelixError, HelixResult};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

/// Central database manager for the Helix system
#[derive(Debug)]
pub struct DatabaseManager {
    /// Configuration for all databases
    config: Arc<RwLock<DatabaseConfig>>,
    /// Connection pool for all databases
    pool: Arc<DatabaseConnectionPool>,
    /// Whether the manager is running
    is_running: Arc<RwLock<bool>>,
}

impl DatabaseManager {
    /// Create a new database manager
    pub fn new(config: DatabaseConfig) -> HelixResult<Self> {
        config.validate()
            .map_err(|e| HelixError::configuration(e))?;

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            pool: Arc::new(DatabaseConnectionPool::new()),
            is_running: Arc::new(RwLock::new(false)),
        })
    }

    /// Initialize all database connections
    pub async fn initialize(&self) -> HelixResult<()> {
        let config = self.config.read().await;
        
        tracing::info!("Initializing database connections...");
        
        // Initialize all configured databases
        for (name, db_config) in &config.databases {
            tracing::info!("Connecting to database: {}", name);
            
            match self.pool.add_connection(name.clone(), db_config).await {
                Ok(_) => {
                    tracing::info!("Successfully connected to database: {}", name);
                }
                Err(e) => {
                    tracing::error!("Failed to connect to database '{}': {}", name, e);
                    return Err(e);
                }
            }
        }

        // Run initial health check
        self.pool.health_check_all().await?;
        
        // Mark as running
        {
            let mut is_running = self.is_running.write().await;
            *is_running = true;
        }

        tracing::info!("Database manager initialized with {} databases", config.databases.len());
        Ok(())
    }

    /// Start the database manager with background health monitoring
    pub async fn start(&self) -> HelixResult<()> {
        // Initialize connections first
        self.initialize().await?;

        // Start health monitoring background task
        let pool = Arc::clone(&self.pool);
        let is_running = Arc::clone(&self.is_running);
        
        tokio::spawn(async move {
            let mut health_check_interval = interval(Duration::from_secs(30));
            
            loop {
                health_check_interval.tick().await;
                
                // Check if we should continue running
                {
                    let running = is_running.read().await;
                    if !*running {
                        break;
                    }
                }
                
                // Perform health check
                if let Err(e) = pool.health_check_all().await {
                    tracing::warn!("Database health check failed: {}", e);
                }
            }
            
            tracing::info!("Database health monitoring stopped");
        });

        tracing::info!("Database manager started");
        Ok(())
    }

    /// Stop the database manager
    pub async fn stop(&self) -> HelixResult<()> {
        tracing::info!("Stopping database manager...");
        
        // Mark as not running to stop background tasks
        {
            let mut is_running = self.is_running.write().await;
            *is_running = false;
        }

        // Close all connections
        self.pool.close_all().await?;

        tracing::info!("Database manager stopped");
        Ok(())
    }

    /// Get a database connection by name
    pub fn get_connection(&self, database_name: &str) -> HelixResult<DatabaseConnection> {
        self.pool
            .get_connection(database_name)
            .ok_or_else(|| HelixError::not_found(format!("Database connection not found: {}", database_name)))
    }

    /// Get the account database connection
    pub fn account_db(&self) -> HelixResult<DatabaseConnection> {
        self.get_connection("account")
    }

    /// Get the universe database connection
    pub fn universe_db(&self) -> HelixResult<DatabaseConnection> {
        self.get_connection("universe")
    }

    /// Get the server database connection
    pub fn server_db(&self) -> HelixResult<DatabaseConnection> {
        self.get_connection("server")
    }

    /// Get the network database connection
    pub fn network_db(&self) -> HelixResult<DatabaseConnection> {
        self.get_connection("network")
    }

    /// Get the entity database connection
    pub fn entity_db(&self) -> HelixResult<DatabaseConnection> {
        self.get_connection("entity")
    }

    /// Get the software database connection
    pub fn software_db(&self) -> HelixResult<DatabaseConnection> {
        self.get_connection("software")
    }

    /// Get the hardware database connection
    pub fn hardware_db(&self) -> HelixResult<DatabaseConnection> {
        self.get_connection("hardware")
    }

    /// Get the process database connection
    pub fn process_db(&self) -> HelixResult<DatabaseConnection> {
        self.get_connection("process")
    }

    /// Get the log database connection
    pub fn log_db(&self) -> HelixResult<DatabaseConnection> {
        self.get_connection("log")
    }

    /// Get the mission database connection
    pub fn mission_db(&self) -> HelixResult<DatabaseConnection> {
        self.get_connection("mission")
    }

    /// Get the story database connection
    pub fn story_db(&self) -> HelixResult<DatabaseConnection> {
        self.get_connection("story")
    }

    /// Get the economy database connection
    pub fn economy_db(&self) -> HelixResult<DatabaseConnection> {
        self.get_connection("economy")
    }

    /// Get the stats database connection
    pub fn stats_db(&self) -> HelixResult<DatabaseConnection> {
        self.get_connection("stats")
    }

    /// Get all available database names
    pub fn database_names(&self) -> Vec<String> {
        self.pool.connection_names()
    }

    /// Check if a database is healthy
    pub async fn is_database_healthy(&self, database_name: &str) -> bool {
        self.pool.is_healthy(database_name).await
    }

    /// Get health status for all databases
    pub async fn health_status(&self) -> std::collections::HashMap<String, bool> {
        let health_map = self.pool.health_status().await;
        health_map.into_iter().collect()
    }

    /// Run health check on all databases
    pub async fn health_check(&self) -> HelixResult<()> {
        self.pool.health_check_all().await
    }

    /// Add a new database connection at runtime
    pub async fn add_database(&self, name: String, config: DatabaseConnectionConfig) -> HelixResult<()> {
        // Add to pool
        self.pool.add_connection(name.clone(), &config).await?;
        
        // Add to configuration
        {
            let mut cfg = self.config.write().await;
            cfg.databases.insert(name.clone(), config);
        }
        
        tracing::info!("Added new database connection at runtime: {}", name);
        Ok(())
    }

    /// Remove a database connection at runtime
    pub async fn remove_database(&self, name: &str) -> HelixResult<()> {
        // Remove from pool
        self.pool.remove_connection(name).await?;
        
        // Remove from configuration
        {
            let mut cfg = self.config.write().await;
            cfg.databases.remove(name);
        }
        
        tracing::info!("Removed database connection at runtime: {}", name);
        Ok(())
    }

    /// Check if the manager is running
    pub async fn is_running(&self) -> bool {
        let is_running = self.is_running.read().await;
        *is_running
    }

    /// Get current configuration
    pub async fn get_config(&self) -> DatabaseConfig {
        let config = self.config.read().await;
        config.clone()
    }
}