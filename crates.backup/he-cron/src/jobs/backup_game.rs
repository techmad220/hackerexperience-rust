//! Game database backup job
//! 
//! This job creates a backup of the main game database and uploads it to AWS S3.
//! Equivalent to the legacy backup_game.php cron job.

use crate::error::{CronError, CronResult};
use crate::utils::{execute_command, format_backup_timestamp};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client, primitives::ByteStream};
use chrono::Utc;
use sqlx::MySqlPool;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tracing::{info, error};

/// Game backup job implementation
pub struct GameBackupJob;

impl GameBackupJob {
    /// Execute the game backup job
    pub async fn execute(db_pool: Arc<MySqlPool>) -> CronResult<()> {
        let timestamp = format_backup_timestamp(Utc::now());
        let backup_name = format!("{}_game", timestamp);
        let local_path = format!("/var/web/backup/game/{}.sql", backup_name);
        
        info!("Starting game database backup: {}", backup_name);
        
        // Create backup directory if it doesn't exist
        if let Some(parent) = Path::new(&local_path).parent() {
            fs::create_dir_all(parent).await?;
        }
        
        // Perform database backup using mysqldump
        Self::backup_database(&local_path).await?;
        
        // Upload to S3
        Self::upload_to_s3(&local_path, &backup_name).await?;
        
        // Clean up local file
        fs::remove_file(&local_path).await?;
        
        info!("Game backup completed successfully: {}", backup_name);
        Ok(())
    }
    
    /// Backup the game database to a local SQL file
    async fn backup_database(output_path: &str) -> CronResult<()> {
        let mysql_host = std::env::var("GAME_DB_HOST").unwrap_or_else(|_| "localhost".to_string());
        let mysql_user = std::env::var("GAME_DB_USER").unwrap_or_else(|_| "he".to_string());
        let mysql_password = std::env::var("GAME_DB_PASSWORD")
            .map_err(|_| CronError::Config("GAME_DB_PASSWORD environment variable not set".to_string()))?;
        let mysql_database = std::env::var("GAME_DB_NAME").unwrap_or_else(|_| "game".to_string());
        
        let args = vec![
            "--opt",
            &format!("-h{}", mysql_host),
            &format!("-u{}", mysql_user),
            &format!("-p{}", mysql_password),
            &mysql_database,
        ];
        
        info!("Executing mysqldump for game database");
        
        let output = execute_command("mysqldump", &args).await?;
        
        // Write output to file
        fs::write(output_path, output).await?;
        
        info!("Game database backup written to: {}", output_path);
        Ok(())
    }
    
    /// Upload backup file to AWS S3
    async fn upload_to_s3(file_path: &str, backup_name: &str) -> CronResult<()> {
        let region_provider = RegionProviderChain::default_provider()
            .or_else("us-east-1");
        let config = aws_config::from_env().region(region_provider).load().await;
        let client = Client::new(&config);
        
        let bucket = std::env::var("S3_BACKUP_BUCKET")
            .map_err(|_| CronError::Config("S3_BACKUP_BUCKET environment variable not set".to_string()))?;
        
        let now = Utc::now();
        let s3_key = format!(
            "/{}/{}/{}/{}",
            now.format("%Y"),
            now.format("%m"), 
            now.format("%d"),
            backup_name
        );
        
        info!("Uploading backup to S3: s3://{}/{}", bucket, s3_key);
        
        let body = ByteStream::from_path(file_path)
            .await
            .map_err(|e| CronError::S3(format!("Failed to read backup file: {}", e)))?;
        
        client
            .put_object()
            .bucket(&bucket)
            .key(&s3_key)
            .body(body)
            .send()
            .await
            .map_err(|e| CronError::S3(format!("Failed to upload to S3: {}", e)))?;
        
        info!("Game backup uploaded to S3 successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_backup_name_format() {
        let timestamp = format_backup_timestamp(Utc::now());
        let backup_name = format!("{}_game", timestamp);
        assert!(backup_name.ends_with("_game"));
        assert!(backup_name.len() > 10); // Should have timestamp prefix
    }
}