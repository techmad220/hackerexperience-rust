//! Helix Logger implementation

use crate::{LoggingConfig, LogFormat, LogRotationConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Helix Logger with advanced features
#[derive(Debug)]
pub struct HelixLogger {
    config: LoggingConfig,
    log_buffer: Arc<RwLock<Vec<LogEntry>>>,
    stats: Arc<RwLock<LoggerStats>>,
}

/// Log entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub message: String,
    pub module: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub fields: std::collections::HashMap<String, String>,
    pub user_id: Option<uuid::Uuid>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
}

/// Log levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// Logger statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerStats {
    pub total_logs: u64,
    pub logs_by_level: std::collections::HashMap<LogLevel, u64>,
    pub errors_count: u64,
    pub warnings_count: u64,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub last_log_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for LoggerStats {
    fn default() -> Self {
        Self {
            total_logs: 0,
            logs_by_level: std::collections::HashMap::new(),
            errors_count: 0,
            warnings_count: 0,
            start_time: chrono::Utc::now(),
            last_log_time: None,
        }
    }
}

impl HelixLogger {
    /// Create a new Helix logger
    pub async fn new(config: LoggingConfig) -> Result<Self> {
        // Ensure log directory exists
        if let Some(file_path) = &config.file_path {
            if let Some(parent) = Path::new(file_path).parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }

        let logger = Self {
            config,
            log_buffer: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(LoggerStats::default())),
        };

        // Start background tasks
        logger.start_background_tasks().await;

        debug!(\"Helix logger initialized\");
        Ok(logger)
    }

    /// Log an entry
    pub async fn log(&self, entry: LogEntry) -> Result<()> {
        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_logs += 1;
            *stats.logs_by_level.entry(entry.level.clone()).or_insert(0) += 1;
            
            match entry.level {
                LogLevel::Error => stats.errors_count += 1,
                LogLevel::Warn => stats.warnings_count += 1,
                _ => {}
            }
            
            stats.last_log_time = Some(entry.timestamp);
        }

        // Add to buffer
        if self.config.enable_file {
            let mut buffer = self.log_buffer.write().await;
            buffer.push(entry.clone());
        }

        // Console output
        if self.config.enable_console {
            self.print_to_console(&entry).await;
        }

        Ok(())
    }

    /// Log with context
    pub async fn log_with_context(
        &self,
        level: LogLevel,
        message: String,
        context: LogContext,
    ) -> Result<()> {
        let entry = LogEntry {
            timestamp: chrono::Utc::now(),
            level,
            message,
            module: context.module,
            file: context.file,
            line: context.line,
            fields: context.fields,
            user_id: context.user_id,
            session_id: context.session_id,
            request_id: context.request_id,
        };

        self.log(entry).await
    }

    /// Print log entry to console
    async fn print_to_console(&self, entry: &LogEntry) {
        match self.config.format {
            LogFormat::Json => {
                if let Ok(json) = serde_json::to_string(entry) {
                    println!(\"{}\", json);
                }
            }
            LogFormat::Pretty => {
                let timestamp = entry.timestamp.format(\"%Y-%m-%d %H:%M:%S UTC\");
                let level_str = match entry.level {
                    LogLevel::Error => format!(\"\\x1b[31m{}\\x1b[0m\", entry.level), // Red
                    LogLevel::Warn => format!(\"\\x1b[33m{}\\x1b[0m\", entry.level),  // Yellow
                    LogLevel::Info => format!(\"\\x1b[32m{}\\x1b[0m\", entry.level),  // Green
                    LogLevel::Debug => format!(\"\\x1b[36m{}\\x1b[0m\", entry.level), // Cyan
                    LogLevel::Trace => format!(\"\\x1b[37m{}\\x1b[0m\", entry.level), // White
                };

                let mut output = format!(\"[{}] {} {}\", timestamp, level_str, entry.message);

                // Add context if available
                if let Some(module) = &entry.module {
                    output.push_str(&format!(\" module={}\", module));
                }
                if let Some(user_id) = &entry.user_id {
                    output.push_str(&format!(\" user_id={}\", user_id));
                }
                if let Some(request_id) = &entry.request_id {
                    output.push_str(&format!(\" request_id={}\", request_id));
                }

                // Add fields
                for (key, value) in &entry.fields {
                    output.push_str(&format!(\" {}={}\", key, value));
                }

                println!(\"{}\", output);
            }
            LogFormat::Compact => {
                let timestamp = entry.timestamp.format(\"%H:%M:%S\");
                println!(\"[{}] {} {}\", timestamp, entry.level, entry.message);
            }
        }
    }

    /// Flush log buffer to file
    pub async fn flush(&self) -> Result<()> {
        if !self.config.enable_file {
            return Ok(());
        }

        let buffer = {
            let mut buffer_guard = self.log_buffer.write().await;
            std::mem::take(&mut *buffer_guard)
        };

        if buffer.is_empty() {
            return Ok(());
        }

        if let Some(file_path) = &self.config.file_path {
            self.write_to_file(&buffer, file_path).await?;
        }

        debug!(\"Flushed {} log entries to file\", buffer.len());
        Ok(())
    }

    /// Write entries to file
    async fn write_to_file(&self, entries: &[LogEntry], file_path: &str) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        // Check if rotation is needed
        if self.should_rotate_file(file_path).await? {
            self.rotate_log_file(file_path).await?;
        }

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .await?;

        for entry in entries {
            let line = match self.config.format {
                LogFormat::Json => {
                    format!(\"{}\n\", serde_json::to_string(entry)?)
                }
                LogFormat::Pretty | LogFormat::Compact => {
                    let timestamp = entry.timestamp.format(\"%Y-%m-%d %H:%M:%S UTC\");
                    format!(\"[{}] {} {}\\n\", timestamp, entry.level, entry.message)
                }
            };

            file.write_all(line.as_bytes()).await?;
        }

        file.flush().await?;
        Ok(())
    }

    /// Check if log file should be rotated
    async fn should_rotate_file(&self, file_path: &str) -> Result<bool> {
        match tokio::fs::metadata(file_path).await {
            Ok(metadata) => {
                let size_mb = metadata.len() / (1024 * 1024);
                Ok(size_mb >= self.config.rotation.max_size_mb)
            }
            Err(_) => Ok(false), // File doesn't exist yet
        }
    }

    /// Rotate log file
    async fn rotate_log_file(&self, file_path: &str) -> Result<()> {
        let path = Path::new(file_path);
        let stem = path.file_stem().unwrap().to_string_lossy();
        let extension = path.extension().map(|s| s.to_string_lossy()).unwrap_or_default();
        let parent = path.parent().unwrap();

        // Rotate existing files
        for i in (1..self.config.rotation.max_files).rev() {
            let old_name = if extension.is_empty() {
                format!(\"{}.{}\", stem, i)
            } else {
                format!(\"{}.{}.{}\", stem, i, extension)
            };
            let new_name = if extension.is_empty() {
                format!(\"{}.{}\", stem, i + 1)
            } else {
                format!(\"{}.{}.{}\", stem, i + 1, extension)
            };

            let old_path = parent.join(&old_name);
            let new_path = parent.join(&new_name);

            if old_path.exists() {
                let _ = tokio::fs::rename(&old_path, &new_path).await;
            }
        }

        // Move current file to .1
        let rotated_name = if extension.is_empty() {
            format!(\"{}.1\", stem)
        } else {
            format!(\"{}.1.{}\", stem, extension)
        };
        let rotated_path = parent.join(&rotated_name);

        if path.exists() {
            tokio::fs::rename(file_path, &rotated_path).await?;

            // Compress if enabled
            if self.config.rotation.compress {
                self.compress_file(&rotated_path).await?;
            }
        }

        info!(\"Log file rotated: {}\", file_path);
        Ok(())
    }

    /// Compress a log file
    async fn compress_file(&self, file_path: &Path) -> Result<()> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let input_data = tokio::fs::read(file_path).await?;
        let compressed_path = file_path.with_extension(\"gz\");

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&input_data)?;
        let compressed_data = encoder.finish()?;

        tokio::fs::write(&compressed_path, compressed_data).await?;
        tokio::fs::remove_file(file_path).await?;

        debug!(\"Compressed log file: {:?}\", compressed_path);
        Ok(())
    }

    /// Start background tasks
    async fn start_background_tasks(&self) {
        // Flush task
        if self.config.enable_file {
            let logger = self.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
                loop {
                    interval.tick().await;
                    if let Err(e) = logger.flush().await {
                        eprintln!(\"Failed to flush logs: {}\", e);
                    }
                }
            });
        }
    }

    /// Get logger statistics
    pub async fn get_stats(&self) -> LoggerStats {
        self.stats.read().await.clone()
    }

    /// Search logs
    pub async fn search_logs(
        &self,
        query: LogQuery,
    ) -> Result<Vec<LogEntry>> {
        // This is a simplified implementation
        // In production, you might want to use a proper search engine
        let buffer = self.log_buffer.read().await;
        
        let mut results = Vec::new();
        
        for entry in buffer.iter() {
            if self.matches_query(entry, &query) {
                results.push(entry.clone());
            }
        }
        
        // Sort by timestamp (newest first)
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        // Apply limit
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }
        
        Ok(results)
    }

    /// Check if log entry matches query
    fn matches_query(&self, entry: &LogEntry, query: &LogQuery) -> bool {
        // Level filter
        if let Some(ref level) = query.level {
            if &entry.level != level {
                return false;
            }
        }

        // Time range filter
        if let Some(start_time) = query.start_time {
            if entry.timestamp < start_time {
                return false;
            }
        }
        
        if let Some(end_time) = query.end_time {
            if entry.timestamp > end_time {
                return false;
            }
        }

        // Text search
        if let Some(ref text) = query.text {
            if !entry.message.contains(text) {
                return false;
            }
        }

        // User ID filter
        if let Some(user_id) = query.user_id {
            if entry.user_id != Some(user_id) {
                return false;
            }
        }

        true
    }
}

impl Clone for HelixLogger {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            log_buffer: self.log_buffer.clone(),
            stats: self.stats.clone(),
        }
    }
}

/// Log context for structured logging
#[derive(Debug, Clone, Default)]
pub struct LogContext {
    pub module: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub fields: std::collections::HashMap<String, String>,
    pub user_id: Option<uuid::Uuid>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
}

/// Log query for searching
#[derive(Debug, Clone)]
pub struct LogQuery {
    pub level: Option<LogLevel>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub text: Option<String>,
    pub user_id: Option<uuid::Uuid>,
    pub limit: Option<usize>,
}

/// Convenience functions for logging
impl HelixLogger {
    pub async fn info(&self, message: &str) -> Result<()> {
        self.log_with_context(
            LogLevel::Info,
            message.to_string(),
            LogContext::default(),
        ).await
    }

    pub async fn warn(&self, message: &str) -> Result<()> {
        self.log_with_context(
            LogLevel::Warn,
            message.to_string(),
            LogContext::default(),
        ).await
    }

    pub async fn error(&self, message: &str) -> Result<()> {
        self.log_with_context(
            LogLevel::Error,
            message.to_string(),
            LogContext::default(),
        ).await
    }

    pub async fn debug(&self, message: &str) -> Result<()> {
        self.log_with_context(
            LogLevel::Debug,
            message.to_string(),
            LogContext::default(),
        ).await
    }

    pub async fn trace(&self, message: &str) -> Result<()> {
        self.log_with_context(
            LogLevel::Trace,
            message.to_string(),
            LogContext::default(),
        ).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_helix_logger_creation() {
        let config = LoggingConfig::default();
        let logger = HelixLogger::new(config).await;
        assert!(logger.is_ok());
    }

    #[tokio::test]
    async fn test_logging() {
        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path().join(\"test.log\");
        
        let config = LoggingConfig {
            enable_file: true,
            file_path: Some(log_path.to_string_lossy().to_string()),
            enable_console: false,
            ..Default::default()
        };

        let logger = HelixLogger::new(config).await.unwrap();
        
        logger.info(\"Test message\").await.unwrap();
        logger.flush().await.unwrap();

        let content = tokio::fs::read_to_string(&log_path).await.unwrap();
        assert!(content.contains(\"Test message\"));
    }

    #[tokio::test]
    async fn test_log_search() {
        let config = LoggingConfig {
            enable_file: false,
            enable_console: false,
            ..Default::default()
        };

        let logger = HelixLogger::new(config).await.unwrap();
        
        logger.info(\"Test info message\").await.unwrap();
        logger.error(\"Test error message\").await.unwrap();

        let query = LogQuery {
            level: Some(LogLevel::Error),
            start_time: None,
            end_time: None,
            text: None,
            user_id: None,
            limit: None,
        };

        let results = logger.search_logs(query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].level, LogLevel::Error);
    }
}