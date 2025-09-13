use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use he_core::id::{EntityId, ServerId};

use crate::events::{LogCreatedEvent, LogDeletedEvent, LogModifiedEvent};
use crate::models::{
    CreateLogParams, Log, LogId, LogIndex, LogTouch, RenderedLogIndex, ReviseLogParams, Revision,
    UpdateLogParams,
};
use crate::queries::LogQueries;

pub struct LogActions {
    pool: PgPool,
    queries: LogQueries,
}

impl LogActions {
    pub fn new(pool: PgPool) -> Self {
        let queries = LogQueries::new(pool.clone());
        Self { pool, queries }
    }

    /// Creates a new log linked to entity on server with message as content.
    pub async fn create(
        &self,
        params: CreateLogParams,
    ) -> Result<(Log, Vec<LogCreatedEvent>)> {
        let mut tx = self.pool.begin().await?;
        let log_id = Uuid::new_v4();
        let revision_id = Uuid::new_v4();
        let now = Utc::now();

        // Create the initial revision
        sqlx::query!(
            "INSERT INTO revisions (revision_id, log_id, entity_id, message, forge_version, creation_time)
             VALUES ($1, $2, $3, $4, $5, $6)",
            revision_id,
            log_id,
            params.entity_id,
            params.message,
            params.forge_version.unwrap_or(0),
            now
        )
        .execute(&mut *tx)
        .await?;

        // Create the log
        sqlx::query!(
            "INSERT INTO logs (log_id, server_id, entity_id, message, crypto_version, creation_time)
             VALUES ($1, $2, $3, $4, $5, $6)",
            log_id,
            params.server_id,
            params.entity_id,
            params.message,
            params.crypto_version,
            now
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        let log = Log {
            log_id,
            server_id: params.server_id,
            entity_id: params.entity_id,
            message: params.message,
            crypto_version: params.crypto_version,
            creation_time: now,
        };

        let event = LogCreatedEvent::new(&log);
        Ok((log, vec![event]))
    }

    /// Adds a revision over log.
    /// 
    /// - `log` is the log to revise
    /// - `params.entity_id` is the entity doing the revision
    /// - `params.message` is the new log's content
    /// - `params.forge_version` is the version of log forger used
    pub async fn revise(
        &self,
        log: &Log,
        params: ReviseLogParams,
    ) -> Result<(Log, Vec<LogModifiedEvent>)> {
        let mut tx = self.pool.begin().await?;
        let revision_id = Uuid::new_v4();
        let log_touch_id = Uuid::new_v4();
        let now = Utc::now();

        // Create the revision
        sqlx::query!(
            "INSERT INTO revisions (revision_id, log_id, entity_id, message, forge_version, creation_time)
             VALUES ($1, $2, $3, $4, $5, $6)",
            revision_id,
            log.log_id,
            params.entity_id,
            params.message,
            params.forge_version,
            now
        )
        .execute(&mut *tx)
        .await?;

        // Update the log message
        sqlx::query!(
            "UPDATE logs SET message = $1 WHERE log_id = $2",
            params.message,
            log.log_id
        )
        .execute(&mut *tx)
        .await?;

        // Create log touch record
        sqlx::query!(
            "INSERT INTO log_touches (log_touch_id, log_id, entity_id, creation_time)
             VALUES ($1, $2, $3, $4)",
            log_touch_id,
            log.log_id,
            params.entity_id,
            now
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        let updated_log = Log {
            message: params.message,
            ..log.clone()
        };

        let event = LogModifiedEvent::new(&updated_log);
        Ok((updated_log, vec![event]))
    }

    /// Recovers log to a previous revision.
    /// 
    /// Returns:
    /// - `Ok(RecoverResult::Recovered)` if the log was recovered to a previous revision
    /// - `Ok(RecoverResult::Deleted)` if the log was deleted (when recovering original forged log)
    /// - `Err(RecoverError::OriginalRevision)` if log is already in original state and not forged
    pub async fn recover(
        &self,
        log: &Log,
    ) -> Result<RecoverResult> {
        let revisions = self.queries.get_revisions(log.log_id).await?;
        
        if revisions.len() <= 1 {
            // Check if this is a forged log (crypto_version is set)
            if log.crypto_version.is_some() {
                // Delete the forged log
                sqlx::query!(
                    "DELETE FROM logs WHERE log_id = $1",
                    log.log_id
                )
                .execute(&self.pool)
                .await?;

                let event = LogDeletedEvent::new(log);
                return Ok(RecoverResult::Deleted(vec![event]));
            } else {
                return Err(anyhow::anyhow!("Cannot recover original revision"));
            }
        }

        // Remove the latest revision and update log to previous revision's message
        let latest_revision = &revisions[revisions.len() - 1];
        let previous_revision = &revisions[revisions.len() - 2];

        let mut tx = self.pool.begin().await?;

        // Delete the latest revision
        sqlx::query!(
            "DELETE FROM revisions WHERE revision_id = $1",
            latest_revision.revision_id
        )
        .execute(&mut *tx)
        .await?;

        // Update log to previous revision's message
        sqlx::query!(
            "UPDATE logs SET message = $1 WHERE log_id = $2",
            previous_revision.message,
            log.log_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        let updated_log = Log {
            message: previous_revision.message.clone(),
            ..log.clone()
        };

        let event = LogModifiedEvent::new(&updated_log);
        Ok(RecoverResult::Recovered(vec![event]))
    }

    /// Returns the Log index, with information about the logs on the server.
    pub async fn get_server_log_index(&self, server_id: ServerId) -> Result<Vec<LogIndex>> {
        let logs = self.queries.get_logs_on_server(server_id).await?;
        
        let index = logs
            .into_iter()
            .map(|log| LogIndex {
                log_id: log.log_id,
                message: log.message,
                timestamp: log.creation_time,
            })
            .collect();

        Ok(index)
    }

    /// Top-level renderer for log index
    pub fn render_index(index: Vec<LogIndex>) -> Vec<RenderedLogIndex> {
        index
            .into_iter()
            .map(|log| RenderedLogIndex {
                log_id: log.log_id.to_string(),
                message: log.message,
                timestamp: log.timestamp.timestamp().to_string(),
            })
            .collect()
    }
}

#[derive(Debug)]
pub enum RecoverResult {
    Recovered(Vec<LogModifiedEvent>),
    Deleted(Vec<LogDeletedEvent>),
}