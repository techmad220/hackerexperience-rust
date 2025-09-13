//! Traits for cron job implementations

use crate::error::CronResult;
use async_trait::async_trait;
use sqlx::MySqlPool;
use std::sync::Arc;

/// Trait for implementing cron jobs
#[async_trait]
pub trait CronJob {
    /// Execute the cron job with the given database pool
    async fn execute(db_pool: Arc<MySqlPool>) -> CronResult<()>;
    
    /// Get the name of the cron job for logging
    fn name() -> &'static str;
    
    /// Get the cron schedule expression for this job
    fn schedule() -> &'static str;
    
    /// Validate that the job can run (optional, default implementation returns Ok)
    async fn validate(db_pool: Arc<MySqlPool>) -> CronResult<()> {
        // Default implementation - just check database connectivity
        sqlx::query("SELECT 1")
            .execute(&*db_pool)
            .await?;
        Ok(())
    }
}

/// Trait for backup jobs that need AWS S3 integration
#[async_trait]
pub trait BackupJob: CronJob {
    /// Get the S3 bucket name for this backup job
    fn s3_bucket() -> &'static str;
    
    /// Get the S3 key prefix for this backup job  
    fn s3_key_prefix() -> &'static str;
    
    /// Get the local backup directory
    fn backup_directory() -> &'static str;
    
    /// Perform the database backup to a local file
    async fn backup_database(db_pool: Arc<MySqlPool>, file_path: &str) -> CronResult<()>;
    
    /// Upload the backup file to S3
    async fn upload_to_s3(file_path: &str, s3_key: &str) -> CronResult<()>;
}