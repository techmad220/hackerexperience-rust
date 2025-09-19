//! HackerExperience Cron System
//! 
//! This crate provides a modern async replacement for the legacy PHP cron jobs.
//! It uses tokio-cron-scheduler for scheduling and maintains the exact same 
//! business logic as the original PHP scripts.

pub mod scheduler;
pub mod jobs;
pub mod error;
pub mod traits;
pub mod utils;

pub use scheduler::CronScheduler;
pub use error::{CronError, CronResult};

use tokio_cron_scheduler::{JobScheduler, Job};
use tracing::{info, error};

/// Initialize and start the cron scheduler with all jobs
pub async fn start_cron_scheduler() -> CronResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting HackerExperience Cron Scheduler");

    let scheduler = CronScheduler::new().await?;
    scheduler.start().await?;

    info!("Cron scheduler started successfully");

    // Keep the scheduler running
    tokio::signal::ctrl_c().await.map_err(|e| {
        error!("Failed to wait for shutdown signal: {}", e);
        CronError::Runtime(format!("Failed to wait for shutdown signal: {}", e))
    })?;

    info!("Shutting down cron scheduler");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let result = CronScheduler::new().await;
        assert!(result.is_ok());
    }
}