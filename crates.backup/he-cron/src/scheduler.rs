//! Tokio-based cron scheduler for HackerExperience
//! 
//! This module provides the main scheduler that orchestrates all cron jobs
//! using tokio-cron-scheduler for precise timing and async execution.

use crate::error::{CronError, CronResult};
use crate::jobs::*;
use tokio_cron_scheduler::{JobScheduler, Job};
use tracing::{info, error, warn};
use std::sync::Arc;
use sqlx::MySqlPool;

/// Main cron scheduler that manages all periodic tasks
pub struct CronScheduler {
    scheduler: JobScheduler,
    db_pool: Arc<MySqlPool>,
}

impl CronScheduler {
    /// Create a new cron scheduler instance
    pub async fn new() -> CronResult<Self> {
        let scheduler = JobScheduler::new()
            .await
            .map_err(|e| CronError::Runtime(format!("Failed to create scheduler: {}", e)))?;

        // Initialize database connection pool
        let db_pool = Arc::new(
            he_db::establish_connection()
                .await
                .map_err(|e| CronError::Database(format!("Failed to connect to database: {}", e)))?
        );

        Ok(Self {
            scheduler,
            db_pool,
        })
    }

    /// Start the scheduler and register all cron jobs
    pub async fn start(&self) -> CronResult<()> {
        info!("Registering cron jobs...");

        // Register all jobs with their schedules
        self.register_backup_jobs().await?;
        self.register_game_maintenance_jobs().await?;
        self.register_war_management_jobs().await?;
        self.register_statistics_jobs().await?;
        self.register_cleanup_jobs().await?;

        // Start the scheduler
        self.scheduler
            .start()
            .await
            .map_err(|e| CronError::Runtime(format!("Failed to start scheduler: {}", e)))?;

        info!("Cron scheduler started successfully");
        Ok(())
    }

    /// Register backup-related cron jobs
    async fn register_backup_jobs(&self) -> CronResult<()> {
        let db_pool = Arc::clone(&self.db_pool);

        // Backup forum database every 4 hours
        let backup_forum_job = Job::new_async("0 0 */4 * * *", move |_uuid, _l| {
            let pool = Arc::clone(&db_pool);
            Box::pin(async move {
                info!("Starting forum backup job");
                if let Err(e) = backup_forum::execute(pool).await {
                    error!("Forum backup job failed: {}", e);
                } else {
                    info!("Forum backup job completed successfully");
                }
            })
        })
        .map_err(|e| CronError::Runtime(format!("Failed to create backup forum job: {}", e)))?;

        let db_pool = Arc::clone(&self.db_pool);

        // Backup game database every 2 hours
        let backup_game_job = Job::new_async("0 0 */2 * * *", move |_uuid, _l| {
            let pool = Arc::clone(&db_pool);
            Box::pin(async move {
                info!("Starting game backup job");
                if let Err(e) = backup_game::execute(pool).await {
                    error!("Game backup job failed: {}", e);
                } else {
                    info!("Game backup job completed successfully");
                }
            })
        })
        .map_err(|e| CronError::Runtime(format!("Failed to create backup game job: {}", e)))?;

        self.scheduler.add(backup_forum_job).await
            .map_err(|e| CronError::Runtime(format!("Failed to add backup forum job: {}", e)))?;
        
        self.scheduler.add(backup_game_job).await
            .map_err(|e| CronError::Runtime(format!("Failed to add backup game job: {}", e)))?;

        info!("Registered backup jobs");
        Ok(())
    }

    /// Register game maintenance jobs
    async fn register_game_maintenance_jobs(&self) -> CronResult<()> {
        let db_pool = Arc::clone(&self.db_pool);

        // Restore NPC software every 30 minutes
        let restore_software_job = Job::new_async("0 */30 * * * *", move |_uuid, _l| {
            let pool = Arc::clone(&db_pool);
            Box::pin(async move {
                info!("Starting restore software job");
                if let Err(e) = restore_software::execute(pool).await {
                    error!("Restore software job failed: {}", e);
                } else {
                    info!("Restore software job completed successfully");
                }
            })
        })
        .map_err(|e| CronError::Runtime(format!("Failed to create restore software job: {}", e)))?;

        let db_pool = Arc::clone(&self.db_pool);

        // Generate missions every hour
        let generate_missions_job = Job::new_async("0 0 * * * *", move |_uuid, _l| {
            let pool = Arc::clone(&db_pool);
            Box::pin(async move {
                info!("Starting generate missions job");
                if let Err(e) = generate_missions::execute(pool).await {
                    error!("Generate missions job failed: {}", e);
                } else {
                    info!("Generate missions job completed successfully");
                }
            })
        })
        .map_err(|e| CronError::Runtime(format!("Failed to create generate missions job: {}", e)))?;

        let db_pool = Arc::clone(&self.db_pool);

        // Update premium status every 15 minutes
        let update_premium_job = Job::new_async("0 */15 * * * *", move |_uuid, _l| {
            let pool = Arc::clone(&db_pool);
            Box::pin(async move {
                info!("Starting update premium job");
                if let Err(e) = update_premium::execute(pool).await {
                    error!("Update premium job failed: {}", e);
                } else {
                    info!("Update premium job completed successfully");
                }
            })
        })
        .map_err(|e| CronError::Runtime(format!("Failed to create update premium job: {}", e)))?;

        self.scheduler.add(restore_software_job).await
            .map_err(|e| CronError::Runtime(format!("Failed to add restore software job: {}", e)))?;
        
        self.scheduler.add(generate_missions_job).await
            .map_err(|e| CronError::Runtime(format!("Failed to add generate missions job: {}", e)))?;
        
        self.scheduler.add(update_premium_job).await
            .map_err(|e| CronError::Runtime(format!("Failed to add update premium job: {}", e)))?;

        info!("Registered game maintenance jobs");
        Ok(())
    }

    /// Register war management jobs
    async fn register_war_management_jobs(&self) -> CronResult<()> {
        let db_pool = Arc::clone(&self.db_pool);

        // DEFCON processing every 5 minutes
        let defcon_job = Job::new_async("0 */5 * * * *", move |_uuid, _l| {
            let pool = Arc::clone(&db_pool);
            Box::pin(async move {
                info!("Starting DEFCON job");
                if let Err(e) = defcon::execute(pool).await {
                    error!("DEFCON job failed: {}", e);
                } else {
                    info!("DEFCON job completed successfully");
                }
            })
        })
        .map_err(|e| CronError::Runtime(format!("Failed to create DEFCON job: {}", e)))?;

        let db_pool = Arc::clone(&self.db_pool);

        // End war processing every minute
        let end_war_job = Job::new_async("0 * * * * *", move |_uuid, _l| {
            let pool = Arc::clone(&db_pool);
            Box::pin(async move {
                info!("Starting end war job");
                if let Err(e) = end_war::execute(pool).await {
                    error!("End war job failed: {}", e);
                } else {
                    info!("End war job completed successfully");
                }
            })
        })
        .map_err(|e| CronError::Runtime(format!("Failed to create end war job: {}", e)))?;

        self.scheduler.add(defcon_job).await
            .map_err(|e| CronError::Runtime(format!("Failed to add DEFCON job: {}", e)))?;
        
        self.scheduler.add(end_war_job).await
            .map_err(|e| CronError::Runtime(format!("Failed to add end war job: {}", e)))?;

        info!("Registered war management jobs");
        Ok(())
    }

    /// Register statistics jobs
    async fn register_statistics_jobs(&self) -> CronResult<()> {
        let db_pool = Arc::clone(&self.db_pool);

        // Update server statistics every 10 minutes
        let update_server_stats_job = Job::new_async("0 */10 * * * *", move |_uuid, _l| {
            let pool = Arc::clone(&db_pool);
            Box::pin(async move {
                info!("Starting update server stats job");
                if let Err(e) = update_server_stats::execute(pool).await {
                    error!("Update server stats job failed: {}", e);
                } else {
                    info!("Update server stats job completed successfully");
                }
            })
        })
        .map_err(|e| CronError::Runtime(format!("Failed to create update server stats job: {}", e)))?;

        self.scheduler.add(update_server_stats_job).await
            .map_err(|e| CronError::Runtime(format!("Failed to add update server stats job: {}", e)))?;

        info!("Registered statistics jobs");
        Ok(())
    }

    /// Register cleanup jobs
    async fn register_cleanup_jobs(&self) -> CronResult<()> {
        let db_pool = Arc::clone(&self.db_pool);

        // SafeNet update every 30 minutes
        let safenet_update_job = Job::new_async("0 */30 * * * *", move |_uuid, _l| {
            let pool = Arc::clone(&db_pool);
            Box::pin(async move {
                info!("Starting SafeNet update job");
                if let Err(e) = safenet_update::execute(pool).await {
                    error!("SafeNet update job failed: {}", e);
                } else {
                    info!("SafeNet update job completed successfully");
                }
            })
        })
        .map_err(|e| CronError::Runtime(format!("Failed to create SafeNet update job: {}", e)))?;

        let db_pool = Arc::clone(&self.db_pool);

        // Doom updater every minute (for checking doom virus countdown)
        let doom_updater_job = Job::new_async("0 * * * * *", move |_uuid, _l| {
            let pool = Arc::clone(&db_pool);
            Box::pin(async move {
                info!("Starting doom updater job");
                if let Err(e) = doom_updater::execute(pool).await {
                    error!("Doom updater job failed: {}", e);
                } else {
                    info!("Doom updater job completed successfully");
                }
            })
        })
        .map_err(|e| CronError::Runtime(format!("Failed to create doom updater job: {}", e)))?;

        self.scheduler.add(safenet_update_job).await
            .map_err(|e| CronError::Runtime(format!("Failed to add SafeNet update job: {}", e)))?;
        
        self.scheduler.add(doom_updater_job).await
            .map_err(|e| CronError::Runtime(format!("Failed to add doom updater job: {}", e)))?;

        info!("Registered cleanup jobs");
        Ok(())
    }

    /// Shutdown the scheduler gracefully
    pub async fn shutdown(&self) -> CronResult<()> {
        info!("Shutting down cron scheduler");
        self.scheduler
            .shutdown()
            .await
            .map_err(|e| CronError::Runtime(format!("Failed to shutdown scheduler: {}", e)))?;
        Ok(())
    }
}