//! # HackerExperience Database Infrastructure
//!
//! Comprehensive database layer supporting multiple databases with connection pooling,
//! migrations, query builders, and repository patterns.
//!
//! ## Features
//!
//! - **Multi-Database Support**: Manages 13 game databases
//! - **Connection Pooling**: Efficient connection management with SQLx
//! - **Migration Management**: Automated schema migrations
//! - **Query Builders**: Type-safe query construction
//! - **Repository Pattern**: Domain-specific data access patterns
//! - **Transaction Support**: ACID transaction management
//! - **Health Monitoring**: Database health checks and metrics

use sqlx::{MySql, MySqlPool, Pool, Transaction};
use anyhow::Result;
use std::collections::HashMap;
use tracing::{info, error, warn};
use tokio::sync::RwLock;
use std::sync::Arc;

pub mod connection;
pub mod migrations;
pub mod repositories;
pub mod query_builder;
pub mod transactions;
pub mod health;
pub mod metrics;

// Re-export main types
pub use connection::*;
pub use repositories::*;
pub use query_builder::*;
pub use transactions::*;
pub use health::*;
pub use metrics::*;

// Type aliases for convenience
pub type DbPool = Pool<MySql>;
pub type DbTransaction<'a> = Transaction<'a, MySql>;

/// Database configuration with support for multiple databases
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: std::time::Duration,
    pub idle_timeout: std::time::Duration,
    pub max_lifetime: std::time::Duration,
    pub enable_logging: bool,
    pub ssl_mode: SslMode,
}

/// SSL configuration modes
#[derive(Debug, Clone)]
pub enum SslMode {
    Disabled,
    Preferred,
    Required,
    VerifyCA,
    VerifyIdentity,
}

/// Multi-database configuration for HackerExperience's 13 databases
#[derive(Debug, Clone)]
pub struct MultiDatabaseConfig {
    pub main: DatabaseConfig,
    pub cache: DatabaseConfig,
    pub logs: DatabaseConfig,
    pub admin: DatabaseConfig,
    pub process: DatabaseConfig,
    pub network: DatabaseConfig,
    pub factor: DatabaseConfig,
    pub id: DatabaseConfig,
    pub balance: DatabaseConfig,
    pub client: DatabaseConfig,
    pub story: DatabaseConfig,
    pub account: DatabaseConfig,
    pub universe: DatabaseConfig,
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
            min_connections: 5,
            connection_timeout: std::time::Duration::from_secs(30),
            idle_timeout: std::time::Duration::from_secs(600),
            max_lifetime: std::time::Duration::from_secs(3600),
            enable_logging: true,
            ssl_mode: SslMode::Preferred,
        }
    }
}

impl Default for MultiDatabaseConfig {
    fn default() -> Self {
        let base_config = DatabaseConfig::default();
        
        Self {
            main: DatabaseConfig { database: "helix_main".to_string(), ..base_config.clone() },
            cache: DatabaseConfig { database: "helix_cache".to_string(), ..base_config.clone() },
            logs: DatabaseConfig { database: "helix_log".to_string(), ..base_config.clone() },
            admin: DatabaseConfig { database: "helix_henforcer".to_string(), ..base_config.clone() },
            process: DatabaseConfig { database: "helix_process".to_string(), ..base_config.clone() },
            network: DatabaseConfig { database: "helix_network".to_string(), ..base_config.clone() },
            factor: DatabaseConfig { database: "helix_factor".to_string(), ..base_config.clone() },
            id: DatabaseConfig { database: "helix_id".to_string(), ..base_config.clone() },
            balance: DatabaseConfig { database: "helix_balance".to_string(), ..base_config.clone() },
            client: DatabaseConfig { database: "helix_client".to_string(), ..base_config.clone() },
            story: DatabaseConfig { database: "helix_story".to_string(), ..base_config.clone() },
            account: DatabaseConfig { database: "helix_account".to_string(), ..base_config.clone() },
            universe: DatabaseConfig { database: "helix_universe".to_string(), ..base_config },
        }
    }
}

impl DatabaseConfig {
    pub fn connection_string(&self) -> String {
        let ssl_param = match self.ssl_mode {
            SslMode::Disabled => "ssl-mode=DISABLED",
            SslMode::Preferred => "ssl-mode=PREFERRED",
            SslMode::Required => "ssl-mode=REQUIRED",
            SslMode::VerifyCA => "ssl-mode=VERIFY_CA",
            SslMode::VerifyIdentity => "ssl-mode=VERIFY_IDENTITY",
        };
        
        format!(
            "mysql://{}:{}@{}:{}/{}?{}",
            self.username, self.password, self.host, self.port, self.database, ssl_param
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
            min_connections: std::env::var("DB_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            connection_timeout: std::time::Duration::from_secs(
                std::env::var("DB_CONNECTION_TIMEOUT")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30)
            ),
            idle_timeout: std::time::Duration::from_secs(
                std::env::var("DB_IDLE_TIMEOUT")
                    .unwrap_or_else(|_| "600".to_string())
                    .parse()
                    .unwrap_or(600)
            ),
            max_lifetime: std::time::Duration::from_secs(
                std::env::var("DB_MAX_LIFETIME")
                    .unwrap_or_else(|_| "3600".to_string())
                    .parse()
                    .unwrap_or(3600)
            ),
            enable_logging: std::env::var("DB_ENABLE_LOGGING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            ssl_mode: match std::env::var("DB_SSL_MODE").unwrap_or_else(|_| "preferred".to_string()).as_str() {
                "disabled" => SslMode::Disabled,
                "required" => SslMode::Required,
                "verify-ca" => SslMode::VerifyCA,
                "verify-identity" => SslMode::VerifyIdentity,
                _ => SslMode::Preferred,
            },
        })
    }
    
    pub fn for_database(database_name: &str) -> Result<Self> {
        let mut config = Self::from_env()?;
        config.database = database_name.to_string();
        Ok(config)
    }
}

/// Multi-database manager for HackerExperience's distributed database architecture
#[derive(Debug)]
pub struct DatabaseManager {
    pools: Arc<RwLock<HashMap<String, DbPool>>>,
    config: MultiDatabaseConfig,
    health_checker: DatabaseHealthChecker,
    metrics: DatabaseMetrics,
}

impl DatabaseManager {
    /// Create a new database manager
    pub async fn new(config: MultiDatabaseConfig) -> Result<Self> {
        let pools = Arc::new(RwLock::new(HashMap::new()));
        let health_checker = DatabaseHealthChecker::new();
        let metrics = DatabaseMetrics::new();
        
        let manager = Self {
            pools,
            config,
            health_checker,
            metrics,
        };
        
        // Initialize all database connections
        manager.initialize_all_pools().await?;
        
        Ok(manager)
    }
    
    /// Initialize all database pools
    async fn initialize_all_pools(&self) -> Result<()> {
        let databases = [
            ("main", &self.config.main),
            ("cache", &self.config.cache),
            ("logs", &self.config.logs),
            ("admin", &self.config.admin),
            ("process", &self.config.process),
            ("network", &self.config.network),
            ("factor", &self.config.factor),
            ("id", &self.config.id),
            ("balance", &self.config.balance),
            ("client", &self.config.client),
            ("story", &self.config.story),
            ("account", &self.config.account),
            ("universe", &self.config.universe),
        ];
        
        for (name, config) in databases {
            match self.create_pool(config).await {
                Ok(pool) => {
                    let mut pools = self.pools.write().await;
                    pools.insert(name.to_string(), pool);
                    info!("Initialized database pool for: {}", name);
                }
                Err(e) => {
                    error!("Failed to initialize database pool for {}: {}", name, e);
                    return Err(e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Create a database pool for a specific configuration
    async fn create_pool(&self, config: &DatabaseConfig) -> Result<DbPool> {
        use sqlx::mysql::MySqlPoolOptions;
        
        let pool = MySqlPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(config.connection_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .connect(&config.connection_string())
            .await?;
            
        Ok(pool)
    }
    
    /// Get a database pool by name
    pub async fn get_pool(&self, database_name: &str) -> Option<DbPool> {
        let pools = self.pools.read().await;
        pools.get(database_name).cloned()
    }
    
    /// Get the main database pool
    pub async fn main_pool(&self) -> Option<DbPool> {
        self.get_pool("main").await
    }
    
    /// Get database health status
    pub async fn health_status(&self) -> HashMap<String, bool> {
        let mut status = HashMap::new();
        let pools = self.pools.read().await;
        
        for (name, pool) in pools.iter() {
            let is_healthy = self.health_checker.check_pool_health(pool).await;
            status.insert(name.clone(), is_healthy);
        }
        
        status
    }
    
    /// Get database metrics
    pub async fn get_metrics(&self) -> DatabaseMetrics {
        self.metrics.clone()
    }
    
    /// Close all database connections
    pub async fn close_all(&self) {
        let mut pools = self.pools.write().await;
        
        for (name, pool) in pools.drain() {
            pool.close().await;
            info!("Closed database pool: {}", name);
        }
    }
}
