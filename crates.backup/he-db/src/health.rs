//! Database health monitoring

use crate::DbPool;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::{debug, error, warn};
use tokio::time::sleep;

/// Database health checker
#[derive(Debug, Clone)]
pub struct DatabaseHealthChecker {
    timeout: Duration,
    retry_attempts: usize,
    retry_delay: Duration,
}

impl Default for DatabaseHealthChecker {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            retry_attempts: 3,
            retry_delay: Duration::from_millis(500),
        }
    }
}

impl DatabaseHealthChecker {
    /// Create a new health checker
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a health checker with custom configuration
    pub fn with_config(timeout: Duration, retry_attempts: usize, retry_delay: Duration) -> Self {
        Self {
            timeout,
            retry_attempts,
            retry_delay,
        }
    }

    /// Check if a database pool is healthy
    pub async fn check_pool_health(&self, pool: &DbPool) -> bool {
        for attempt in 1..=self.retry_attempts {
            debug!("Health check attempt {} of {}", attempt, self.retry_attempts);
            
            let start_time = Instant::now();
            let health_result = self.perform_health_check(pool).await;
            let duration = start_time.elapsed();
            
            match health_result {
                Ok(health_info) => {
                    debug!("Health check passed in {:?}: {:?}", duration, health_info);
                    return true;
                }
                Err(e) => {
                    warn!("Health check attempt {} failed: {}", attempt, e);
                    
                    if attempt < self.retry_attempts {
                        debug!("Retrying health check in {:?}", self.retry_delay);
                        sleep(self.retry_delay).await;
                    }
                }
            }
        }
        
        error!("All health check attempts failed");
        false
    }

    /// Perform the actual health check
    async fn perform_health_check(&self, pool: &DbPool) -> Result<DatabaseHealthInfo> {
        let start_time = Instant::now();
        
        // Simple ping query
        let ping_result = tokio::time::timeout(
            self.timeout,
            sqlx::query("SELECT 1 as ping")
                .fetch_one(pool)
        ).await??;
        
        let ping_duration = start_time.elapsed();
        let ping_value: i32 = ping_result.get("ping");
        
        if ping_value != 1 {
            anyhow::bail!("Unexpected ping result: {}", ping_value);
        }

        // Get connection pool stats
        let pool_stats = self.get_pool_stats(pool).await?;
        
        // Get database version
        let version = self.get_database_version(pool).await?;
        
        Ok(DatabaseHealthInfo {
            ping_duration,
            pool_stats,
            version,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Get connection pool statistics
    async fn get_pool_stats(&self, pool: &DbPool) -> Result<PoolStats> {
        // SQLx doesn't expose all pool stats publicly, so we'll use available methods
        Ok(PoolStats {
            size: pool.size(),
            idle: pool.num_idle(),
            used: pool.size() - pool.num_idle(),
            max_connections: pool.size(), // This is not entirely accurate but good enough
        })
    }

    /// Get database version information
    async fn get_database_version(&self, pool: &DbPool) -> Result<String> {
        let version_result = tokio::time::timeout(
            self.timeout,
            sqlx::query("SELECT VERSION() as version")
                .fetch_one(pool)
        ).await??;
        
        Ok(version_result.get("version"))
    }

    /// Comprehensive health check with detailed information
    pub async fn detailed_health_check(&self, pool: &DbPool) -> Result<DetailedHealthInfo> {
        let start_time = Instant::now();
        
        // Basic health check
        let basic_health = self.perform_health_check(pool).await?;
        
        // Additional checks
        let table_count = self.count_tables(pool).await?;
        let connection_count = self.count_connections(pool).await?;
        let uptime = self.get_server_uptime(pool).await?;
        
        let total_duration = start_time.elapsed();
        
        Ok(DetailedHealthInfo {
            basic: basic_health,
            table_count,
            connection_count,
            uptime,
            check_duration: total_duration,
        })
    }

    /// Count the number of tables in the database
    async fn count_tables(&self, pool: &DbPool) -> Result<u64> {
        let result = sqlx::query("SELECT COUNT(*) as count FROM information_schema.tables WHERE table_schema = DATABASE()")
            .fetch_one(pool)
            .await?;
        
        Ok(result.get::<u64, _>("count"))
    }

    /// Count active connections
    async fn count_connections(&self, pool: &DbPool) -> Result<u64> {
        let result = sqlx::query("SELECT COUNT(*) as count FROM information_schema.processlist")
            .fetch_one(pool)
            .await?;
        
        Ok(result.get::<u64, _>("count"))
    }

    /// Get server uptime in seconds
    async fn get_server_uptime(&self, pool: &DbPool) -> Result<u64> {
        let result = sqlx::query("SHOW STATUS LIKE 'Uptime'")
            .fetch_one(pool)
            .await?;
        
        let uptime_str: String = result.get("Value");
        Ok(uptime_str.parse()?)
    }
}

/// Basic database health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHealthInfo {
    pub ping_duration: Duration,
    pub pool_stats: PoolStats,
    pub version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Connection pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub size: u32,
    pub idle: u32,
    pub used: u32,
    pub max_connections: u32,
}

/// Detailed health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedHealthInfo {
    pub basic: DatabaseHealthInfo,
    pub table_count: u64,
    pub connection_count: u64,
    pub uptime: u64,
    pub check_duration: Duration,
}

/// Health check result for multiple databases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiDatabaseHealthResult {
    pub overall_healthy: bool,
    pub database_results: std::collections::HashMap<String, bool>,
    pub detailed_info: Option<std::collections::HashMap<String, DetailedHealthInfo>>,
    pub check_timestamp: chrono::DateTime<chrono::Utc>,
}

/// Health monitoring service for continuous monitoring
pub struct HealthMonitorService {
    checker: DatabaseHealthChecker,
    check_interval: Duration,
    databases: std::collections::HashMap<String, DbPool>,
}

impl HealthMonitorService {
    /// Create a new health monitoring service
    pub fn new(
        checker: DatabaseHealthChecker,
        check_interval: Duration,
        databases: std::collections::HashMap<String, DbPool>,
    ) -> Self {
        Self {
            checker,
            check_interval,
            databases,
        }
    }

    /// Start the health monitoring service
    pub async fn start(&self) -> Result<()> {
        let mut interval = tokio::time::interval(self.check_interval);
        
        loop {
            interval.tick().await;
            
            debug!("Performing scheduled health checks");
            let health_result = self.check_all_databases().await;
            
            match health_result {
                Ok(result) => {
                    if result.overall_healthy {
                        debug!("All databases are healthy");
                    } else {
                        warn!("Some databases are unhealthy: {:?}", result.database_results);
                    }
                }
                Err(e) => {
                    error!("Failed to perform health checks: {}", e);
                }
            }
        }
    }

    /// Check health of all registered databases
    pub async fn check_all_databases(&self) -> Result<MultiDatabaseHealthResult> {
        let mut database_results = std::collections::HashMap::new();
        let mut detailed_info = std::collections::HashMap::new();
        let mut overall_healthy = true;
        
        for (name, pool) in &self.databases {
            let is_healthy = self.checker.check_pool_health(pool).await;
            database_results.insert(name.clone(), is_healthy);
            
            if !is_healthy {
                overall_healthy = false;
            }
            
            // Get detailed info for healthy databases
            if is_healthy {
                if let Ok(detail) = self.checker.detailed_health_check(pool).await {
                    detailed_info.insert(name.clone(), detail);
                }
            }
        }
        
        Ok(MultiDatabaseHealthResult {
            overall_healthy,
            database_results,
            detailed_info: if detailed_info.is_empty() { None } else { Some(detailed_info) },
            check_timestamp: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_checker_creation() {
        let checker = DatabaseHealthChecker::new();
        assert_eq!(checker.timeout, Duration::from_secs(5));
        assert_eq!(checker.retry_attempts, 3);
    }

    #[test]
    fn test_health_checker_with_config() {
        let checker = DatabaseHealthChecker::with_config(
            Duration::from_secs(10),
            5,
            Duration::from_secs(1),
        );
        
        assert_eq!(checker.timeout, Duration::from_secs(10));
        assert_eq!(checker.retry_attempts, 5);
        assert_eq!(checker.retry_delay, Duration::from_secs(1));
    }

    #[test]
    fn test_pool_stats_creation() {
        let stats = PoolStats {
            size: 10,
            idle: 5,
            used: 5,
            max_connections: 20,
        };
        
        assert_eq!(stats.size, 10);
        assert_eq!(stats.idle, 5);
        assert_eq!(stats.used, 5);
        assert_eq!(stats.max_connections, 20);
    }
}