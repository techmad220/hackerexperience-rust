//! Error types for the HackerExperience cron system

use thiserror::Error;

/// Result type for cron operations
pub type CronResult<T> = Result<T, CronError>;

/// Error types that can occur during cron operations
#[derive(Error, Debug)]
pub enum CronError {
    /// Database-related errors
    #[error("Database error: {0}")]
    Database(String),

    /// Runtime errors during job execution
    #[error("Runtime error: {0}")]
    Runtime(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// AWS S3 related errors
    #[error("S3 error: {0}")]
    S3(String),

    /// File system errors
    #[error("File system error: {0}")]
    FileSystem(String),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<sqlx::Error> for CronError {
    fn from(err: sqlx::Error) -> Self {
        CronError::Database(err.to_string())
    }
}

impl From<std::io::Error> for CronError {
    fn from(err: std::io::Error) -> Self {
        CronError::FileSystem(err.to_string())
    }
}

impl From<serde_json::Error> for CronError {
    fn from(err: serde_json::Error) -> Self {
        CronError::Serialization(err.to_string())
    }
}