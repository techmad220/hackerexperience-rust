use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, PgPool, Row, Column, TypeInfo};
use std::collections::HashMap;
use std::env;
use tracing::{info, warn, error};

/// Database configuration for runtime connection
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub main_url: String,
    pub account_url: Option<String>,
    pub server_url: Option<String>,
    pub software_url: Option<String>,
    pub network_url: Option<String>,
    pub bank_url: Option<String>,
    pub clan_url: Option<String>,
    pub mission_url: Option<String>,
    pub log_url: Option<String>,
    pub cache_url: Option<String>,
    pub universe_url: Option<String>,
    pub story_url: Option<String>,
    pub factor_url: Option<String>,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
    pub idle_timeout: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            main_url: "postgresql://localhost:5432/hackerexperience".to_string(),
            account_url: None,
            server_url: None,
            software_url: None,
            network_url: None,
            bank_url: None,
            clan_url: None,
            mission_url: None,
            log_url: None,
            cache_url: None,
            universe_url: None,
            story_url: None,
            factor_url: None,
            max_connections: 100,
            min_connections: 5,
            connect_timeout: 30,
            idle_timeout: 600,
        }
    }
}

impl DatabaseConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok(); // Load .env if present
        
        let main_url = env::var("DATABASE_URL")
            .context("DATABASE_URL environment variable required")?;
        
        Ok(Self {
            main_url,
            account_url: env::var("DATABASE_ACCOUNT_URL").ok(),
            server_url: env::var("DATABASE_SERVER_URL").ok(),
            software_url: env::var("DATABASE_SOFTWARE_URL").ok(),
            network_url: env::var("DATABASE_NETWORK_URL").ok(),
            bank_url: env::var("DATABASE_BANK_URL").ok(),
            clan_url: env::var("DATABASE_CLAN_URL").ok(),
            mission_url: env::var("DATABASE_MISSION_URL").ok(),
            log_url: env::var("DATABASE_LOG_URL").ok(),
            cache_url: env::var("DATABASE_CACHE_URL").ok(),
            universe_url: env::var("DATABASE_UNIVERSE_URL").ok(),
            story_url: env::var("DATABASE_STORY_URL").ok(),
            factor_url: env::var("DATABASE_FACTOR_URL").ok(),
            max_connections: env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            min_connections: env::var("DB_MIN_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            connect_timeout: env::var("DB_CONNECT_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            idle_timeout: env::var("DB_IDLE_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(600),
        })
    }
}

/// Multi-database manager for HackerExperience
#[derive(Debug)]
pub struct DatabaseManager {
    pub pools: HashMap<String, PgPool>,
    pub config: DatabaseConfig,
}

impl DatabaseManager {
    /// Create a new database manager with configuration
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let mut pools = HashMap::new();
        
        // Connect to main database
        info!("Connecting to main database...");
        let main_pool = create_pool(&config.main_url, &config).await
            .context("Failed to connect to main database")?;
        pools.insert("main".to_string(), main_pool);
        
        // Connect to specialized databases if configured
        let db_configs = [
            ("account", &config.account_url),
            ("server", &config.server_url),
            ("software", &config.software_url),
            ("network", &config.network_url),
            ("bank", &config.bank_url),
            ("clan", &config.clan_url),
            ("mission", &config.mission_url),
            ("log", &config.log_url),
            ("cache", &config.cache_url),
            ("universe", &config.universe_url),
            ("story", &config.story_url),
            ("factor", &config.factor_url),
        ];
        
        for (name, url_opt) in db_configs {
            if let Some(url) = url_opt {
                info!("Connecting to {} database...", name);
                match create_pool(url, &config).await {
                    Ok(pool) => {
                        pools.insert(name.to_string(), pool);
                    }
                    Err(e) => {
                        warn!("Failed to connect to {} database: {}", name, e);
                        // Use main database as fallback
                        pools.insert(name.to_string(), pools.get("main").unwrap().clone());
                    }
                }
            } else {
                // Use main database for undefined specialized databases
                pools.insert(name.to_string(), pools.get("main").unwrap().clone());
            }
        }
        
        info!("Database manager initialized with {} pools", pools.len());
        
        Ok(Self { pools, config })
    }
    
    /// Get database pool by name
    pub fn get_pool(&self, name: &str) -> Option<&PgPool> {
        self.pools.get(name)
    }
    
    /// Get main database pool
    pub fn main_pool(&self) -> &PgPool {
        self.pools.get("main").expect("Main pool must exist")
    }
    
    /// Health check for all database connections
    pub async fn health_check(&self) -> Result<HashMap<String, bool>> {
        let mut results = HashMap::new();
        
        for (name, pool) in &self.pools {
            let is_healthy = match sqlx::query("SELECT 1").fetch_one(pool).await {
                Ok(_) => true,
                Err(e) => {
                    error!("Health check failed for {} database: {}", name, e);
                    false
                }
            };
            results.insert(name.clone(), is_healthy);
        }
        
        Ok(results)
    }
    
    /// Execute a query on a specific database
    pub async fn execute_query(
        &self,
        db_name: &str,
        query: &str,
        params: &[String],
    ) -> Result<sqlx::postgres::PgQueryResult> {
        let pool = self.get_pool(db_name)
            .ok_or_else(|| anyhow::anyhow!("Database {} not found", db_name))?;
        
        let mut query_builder = sqlx::query(query);
        for param in params {
            query_builder = query_builder.bind(param);
        }
        
        query_builder.execute(pool).await
            .context(format!("Failed to execute query on {} database", db_name))
    }
    
    /// Fetch rows from a specific database
    pub async fn fetch_rows(
        &self,
        db_name: &str,
        query: &str,
        params: &[String],
    ) -> Result<Vec<sqlx::postgres::PgRow>> {
        let pool = self.get_pool(db_name)
            .ok_or_else(|| anyhow::anyhow!("Database {} not found", db_name))?;
        
        let mut query_builder = sqlx::query(query);
        for param in params {
            query_builder = query_builder.bind(param);
        }
        
        query_builder.fetch_all(pool).await
            .context(format!("Failed to fetch rows from {} database", db_name))
    }
}

/// Create a connection pool with configuration
async fn create_pool(url: &str, config: &DatabaseConfig) -> Result<PgPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(config.connect_timeout))
        .idle_timeout(std::time::Duration::from_secs(config.idle_timeout))
        .connect(url)
        .await
        .context("Failed to create database pool")?;
    
    Ok(pool)
}

/// Simplified database operations without compile-time query checking
#[derive(Debug)]
pub struct SimpleDatabase {
    pool: PgPool,
}

impl SimpleDatabase {
    /// Create a simple database connection from URL
    pub async fn connect(url: &str) -> Result<Self> {
        let pool = PgPool::connect(url).await
            .context("Failed to connect to database")?;
        
        Ok(Self { pool })
    }
    
    /// Execute a simple query with parameters
    pub async fn execute(&self, query: &str, params: &[String]) -> Result<u64> {
        let mut sqlx_query = sqlx::query(query);
        
        for param in params {
            sqlx_query = sqlx_query.bind(param);
        }
        
        let result = sqlx_query.execute(&self.pool).await
            .context("Failed to execute query")?;
        
        Ok(result.rows_affected())
    }
    
    /// Fetch rows as JSON values
    pub async fn fetch_json(&self, query: &str, params: &[String]) -> Result<Vec<serde_json::Value>> {
        let mut sqlx_query = sqlx::query(query);
        
        for param in params {
            sqlx_query = sqlx_query.bind(param);
        }
        
        let rows = sqlx_query.fetch_all(&self.pool).await
            .context("Failed to fetch rows")?;
        
        let mut results = Vec::new();
        for row in rows {
            let mut json_obj = serde_json::Map::new();
            
            for (i, column) in row.columns().iter().enumerate() {
                let column_name = column.name();
                
                // Convert different PostgreSQL types to JSON values
                let value = match column.type_info().name() {
                    "INT4" => {
                        if let Ok(val) = row.try_get::<i32, _>(i) {
                            serde_json::Value::Number(val.into())
                        } else {
                            serde_json::Value::Null
                        }
                    }
                    "INT8" => {
                        if let Ok(val) = row.try_get::<i64, _>(i) {
                            serde_json::Value::Number(val.into())
                        } else {
                            serde_json::Value::Null
                        }
                    }
                    "VARCHAR" | "TEXT" => {
                        if let Ok(val) = row.try_get::<String, _>(i) {
                            serde_json::Value::String(val)
                        } else {
                            serde_json::Value::Null
                        }
                    }
                    "BOOL" => {
                        if let Ok(val) = row.try_get::<bool, _>(i) {
                            serde_json::Value::Bool(val)
                        } else {
                            serde_json::Value::Null
                        }
                    }
                    "TIMESTAMP" | "TIMESTAMPTZ" => {
                        if let Ok(val) = row.try_get::<chrono::DateTime<chrono::Utc>, _>(i) {
                            serde_json::Value::String(val.to_rfc3339())
                        } else {
                            serde_json::Value::Null
                        }
                    }
                    _ => {
                        // Try to get as string for unknown types
                        if let Ok(val) = row.try_get::<String, _>(i) {
                            serde_json::Value::String(val)
                        } else {
                            serde_json::Value::Null
                        }
                    }
                };
                
                json_obj.insert(column_name.to_string(), value);
            }
            
            results.push(serde_json::Value::Object(json_obj));
        }
        
        Ok(results)
    }
    
    /// Health check
    pub async fn health_check(&self) -> Result<bool> {
        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => Ok(true),
            Err(e) => {
                error!("Database health check failed: {}", e);
                Ok(false)
            }
        }
    }
    
    /// Get the underlying pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

/// Database operations trait for runtime queries
pub trait DatabaseOps {
    /// Execute a query and return affected rows
    async fn execute_query(&self, query: &str, params: Vec<String>) -> Result<u64>;
    
    /// Fetch results as JSON
    async fn fetch_json(&self, query: &str, params: Vec<String>) -> Result<Vec<serde_json::Value>>;
    
    /// Check if database is healthy
    async fn health_check(&self) -> Result<bool>;
}

impl DatabaseOps for SimpleDatabase {
    async fn execute_query(&self, query: &str, params: Vec<String>) -> Result<u64> {
        let param_refs: Vec<String> = params;
        self.execute(query, &param_refs).await
    }
    
    async fn fetch_json(&self, query: &str, params: Vec<String>) -> Result<Vec<serde_json::Value>> {
        let param_refs: Vec<String> = params;
        self.fetch_json(query, &param_refs).await
    }
    
    async fn health_check(&self) -> Result<bool> {
        self.health_check().await
    }
}