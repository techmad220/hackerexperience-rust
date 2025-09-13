//! Database configuration for the Helix multi-database architecture

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for all databases in the Helix system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Main game database configurations (13 databases)
    pub databases: HashMap<String, DatabaseConnectionConfig>,
    /// Default connection pool settings
    pub pool: ConnectionPoolConfig,
}

/// Configuration for a single database connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConnectionConfig {
    /// Database type
    pub database_type: DatabaseType,
    /// Connection URL
    pub url: String,
    /// Database name
    pub name: String,
    /// Optional custom pool settings
    pub pool: Option<ConnectionPoolConfig>,
    /// Whether migrations should run on startup
    pub auto_migrate: Option<bool>,
}

/// Database type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DatabaseType {
    /// PostgreSQL database
    PostgreSQL,
    /// MySQL database  
    MySQL,
    /// SQLite database
    SQLite,
}

impl DatabaseType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseType::PostgreSQL => "postgresql",
            DatabaseType::MySQL => "mysql",
            DatabaseType::SQLite => "sqlite",
        }
    }
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections to maintain
    pub min_connections: u32,
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    /// Idle timeout in seconds
    pub idle_timeout: u64,
    /// Maximum lifetime of a connection in seconds
    pub max_lifetime: u64,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 1,
            connect_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 3600,
        }
    }
}

impl DatabaseConfig {
    /// Create a new database configuration with the 13 game databases
    pub fn new() -> Self {
        let mut databases = HashMap::new();
        
        // Add the 13 core game databases
        let database_names = [
            "account",
            "universe", 
            "server",
            "network",
            "entity",
            "software",
            "hardware",
            "process",
            "log",
            "mission",
            "story",
            "economy",
            "stats"
        ];

        for name in database_names {
            databases.insert(
                name.to_string(),
                DatabaseConnectionConfig {
                    database_type: DatabaseType::PostgreSQL,
                    url: format!("postgresql://localhost:5432/helix_{}", name),
                    name: name.to_string(),
                    pool: None,
                    auto_migrate: Some(true),
                }
            );
        }

        Self {
            databases,
            pool: ConnectionPoolConfig::default(),
        }
    }

    /// Get configuration for a specific database
    pub fn get_database_config(&self, name: &str) -> Option<&DatabaseConnectionConfig> {
        self.databases.get(name)
    }

    /// Get all database names
    pub fn database_names(&self) -> Vec<String> {
        self.databases.keys().cloned().collect()
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.databases.is_empty() {
            return Err("No databases configured".to_string());
        }

        for (name, config) in &self.databases {
            if config.url.is_empty() {
                return Err(format!("Empty URL for database '{}'", name));
            }
            if config.name.is_empty() {
                return Err(format!("Empty name for database '{}'", name));
            }
        }

        Ok(())
    }
}