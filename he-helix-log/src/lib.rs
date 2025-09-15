//! # Helix Logging and Monitoring Infrastructure
//!
//! Comprehensive logging system for HackerExperience with structured logging,
//! metrics collection, performance monitoring, and health checks.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, error, debug};

pub mod models;
pub mod actors;
pub mod events;
pub mod queries;
pub mod actions;
pub mod genserver;
pub mod logger;
pub mod metrics;
pub mod monitoring;
pub mod health;
pub mod tracing_setup;

// Re-export main types
pub use models::*;
pub use actors::*;
pub use events::*;
pub use queries::*;
pub use actions::*;
pub use genserver::*;
pub use logger::*;
pub use metrics::*;
pub use monitoring::*;
pub use health::*;
pub use tracing_setup::*;

/// Central logging and monitoring system
#[derive(Debug)]
pub struct LoggingSystem {
    logger: Arc<HelixLogger>,
    metrics: Arc<MetricsCollector>,
    monitor: Arc<SystemMonitor>,
    health_checker: Arc<HealthChecker>,
}

impl LoggingSystem {
    /// Initialize the logging system
    pub async fn new(config: LoggingConfig) -> Result<Self> {
        // Initialize tracing
        init_tracing(&config)?;
        
        // Create components
        let logger = Arc::new(HelixLogger::new(config.clone()).await?);
        let metrics = Arc::new(MetricsCollector::new());
        let monitor = Arc::new(SystemMonitor::new(config.clone()));
        let health_checker = Arc::new(HealthChecker::new());
        
        let system = Self {
            logger,
            metrics,
            monitor,
            health_checker,
        };
        
        info!("Helix logging system initialized successfully");
        Ok(system)
    }
    
    /// Start the logging system
    pub async fn start(&self) -> Result<()> {
        info!("Starting Helix logging system");
        
        // Start monitoring
        self.monitor.start().await?;
        
        // Start health checks
        self.health_checker.start().await?;
        
        info!("Helix logging system started");
        Ok(())
    }
    
    /// Get the logger instance
    pub fn logger(&self) -> &Arc<HelixLogger> {
        &self.logger
    }
    
    /// Get the metrics collector
    pub fn metrics(&self) -> &Arc<MetricsCollector> {
        &self.metrics
    }
    
    /// Get the system monitor
    pub fn monitor(&self) -> &Arc<SystemMonitor> {
        &self.monitor
    }
    
    /// Get the health checker
    pub fn health_checker(&self) -> &Arc<HealthChecker> {
        &self.health_checker
    }
    
    /// Shutdown the logging system
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down Helix logging system");
        
        self.monitor.stop().await?;
        self.health_checker.stop().await?;
        self.logger.flush().await?;
        
        info!("Helix logging system shut down successfully");
        Ok(())
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Log format (json, pretty, compact)
    pub format: LogFormat,
    /// Enable file logging
    pub enable_file: bool,
    /// Log file path
    pub file_path: Option<String>,
    /// Enable console logging
    pub enable_console: bool,
    /// Enable structured logging
    pub structured: bool,
    /// Log rotation settings
    pub rotation: LogRotationConfig,
    /// Metrics configuration
    pub metrics: MetricsConfig,
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
}

/// Log format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

/// Log rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    /// Maximum file size in MB
    pub max_size_mb: u64,
    /// Maximum number of files to keep
    pub max_files: u32,
    /// Enable compression of old files
    pub compress: bool,
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,
    /// Metrics collection interval in seconds
    pub interval: u64,
    /// Enable Prometheus metrics endpoint
    pub enable_prometheus: bool,
    /// Prometheus metrics port
    pub prometheus_port: u16,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable system monitoring
    pub enabled: bool,
    /// Monitoring interval in seconds
    pub interval: u64,
    /// Alert thresholds
    pub thresholds: AlertThresholds,
}

/// Alert threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// CPU usage threshold (percentage)
    pub cpu_threshold: f64,
    /// Memory usage threshold (percentage)
    pub memory_threshold: f64,
    /// Disk usage threshold (percentage)
    pub disk_threshold: f64,
    /// Response time threshold (milliseconds)
    pub response_time_threshold: u64,
    /// Error rate threshold (percentage)
    pub error_rate_threshold: f64,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Pretty,
            enable_file: true,
            file_path: Some("logs/helix.log".to_string()),
            enable_console: true,
            structured: false,
            rotation: LogRotationConfig {
                max_size_mb: 100,
                max_files: 10,
                compress: true,
            },
            metrics: MetricsConfig {
                enabled: true,
                interval: 60,
                enable_prometheus: false,
                prometheus_port: 9090,
            },
            monitoring: MonitoringConfig {
                enabled: true,
                interval: 30,
                thresholds: AlertThresholds {
                    cpu_threshold: 80.0,
                    memory_threshold: 85.0,
                    disk_threshold: 90.0,
                    response_time_threshold: 5000,
                    error_rate_threshold: 5.0,
                },
            },
        }
    }
}

/// Initialize logging for the entire Helix system
pub async fn init_helix_logging(config: LoggingConfig) -> Result<LoggingSystem> {
    LoggingSystem::new(config).await
}

/// Convenience macro for structured logging
#[macro_export]
macro_rules! helix_info {
    ($($key:expr => $value:expr),* $(,)? ; $msg:expr) => {
        tracing::info!($($key = %$value),*; $msg)
    };
    ($msg:expr) => {
        tracing::info!($msg)
    };
}

#[macro_export]
macro_rules! helix_warn {
    ($($key:expr => $value:expr),* $(,)? ; $msg:expr) => {
        tracing::warn!($($key = %$value),*; $msg)
    };
    ($msg:expr) => {
        tracing::warn!($msg)
    };
}

#[macro_export]
macro_rules! helix_error {
    ($($key:expr => $value:expr),* $(,)? ; $msg:expr) => {
        tracing::error!($($key = %$value),*; $msg)
    };
    ($msg:expr) => {
        tracing::error!($msg)
    };
}

#[macro_export]
macro_rules! helix_debug {
    ($($key:expr => $value:expr),* $(,)? ; $msg:expr) => {
        tracing::debug!($($key = %$value),*; $msg)
    };
    ($msg:expr) => {
        tracing::debug!($msg)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_logging_system_creation() {
        let config = LoggingConfig::default();
        let result = LoggingSystem::new(config).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_config() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert!(config.enable_console);
        assert!(config.enable_file);
    }
}