// HackerExperience Database Layer - SQLx integration

use sqlx::{MySql, MySqlPool, Pool};
use anyhow::Result;

pub mod connection;
pub mod migrations;
pub mod repositories;

// Re-export main types
pub use connection::*;
pub use repositories::*;

// Type aliases for convenience
pub type DbPool = Pool<MySql>;

// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 3306,
            username: "hackerexperience".to_string(),
            password: "password".to_string(),
            database: "hackerexperience_rust".to_string(),
            max_connections: 20,
        }
    }
}

impl DatabaseConfig {
    pub fn connection_string(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
    
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("DB_PORT")
                .unwrap_or_else(|_| "3306".to_string())
                .parse()
                .unwrap_or(3306),
            username: std::env::var("DB_USERNAME").unwrap_or_else(|_| "hackerexperience".to_string()),
            password: std::env::var("DB_PASSWORD").unwrap_or_else(|_| "password".to_string()),
            database: std::env::var("DB_DATABASE").unwrap_or_else(|_| "hackerexperience_rust".to_string()),
            max_connections: std::env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .unwrap_or(20),
        })
    }
}
