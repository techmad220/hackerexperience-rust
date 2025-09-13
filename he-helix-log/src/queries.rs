use anyhow::Result;
use sqlx::PgPool;

use he_core::id::{EntityId, ServerId};

use crate::models::{Log, LogId, LogTouch, Revision};

pub struct LogQueries {
    pool: PgPool,
}

impl LogQueries {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_id(&self, log_id: LogId) -> Result<Option<Log>> {
        let log = sqlx::query_as!(
            Log,
            "SELECT log_id, server_id, entity_id, message, crypto_version, creation_time 
             FROM logs WHERE log_id = $1",
            log_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(log)
    }

    pub async fn get_logs_on_server(&self, server_id: ServerId) -> Result<Vec<Log>> {
        let logs = sqlx::query_as!(
            Log,
            "SELECT log_id, server_id, entity_id, message, crypto_version, creation_time 
             FROM logs WHERE server_id = $1 ORDER BY creation_time DESC",
            server_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    pub async fn get_logs_edited_by_entity(&self, entity_id: EntityId) -> Result<Vec<Log>> {
        let logs = sqlx::query_as!(
            Log,
            "SELECT l.log_id, l.server_id, l.entity_id, l.message, l.crypto_version, l.creation_time 
             FROM logs l
             INNER JOIN log_touches lt ON lt.log_id = l.log_id
             WHERE lt.entity_id = $1
             ORDER BY l.creation_time DESC",
            entity_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    pub async fn find_by_message_pattern(&self, pattern: &str) -> Result<Vec<Log>> {
        let logs = sqlx::query_as!(
            Log,
            "SELECT log_id, server_id, entity_id, message, crypto_version, creation_time 
             FROM logs WHERE message LIKE $1 ORDER BY creation_time DESC",
            format!("%{}%", pattern)
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    pub async fn get_revisions(&self, log_id: LogId) -> Result<Vec<Revision>> {
        let revisions = sqlx::query_as!(
            Revision,
            "SELECT revision_id, log_id, entity_id, message, forge_version, creation_time 
             FROM revisions WHERE log_id = $1 ORDER BY creation_time ASC",
            log_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(revisions)
    }

    pub async fn get_latest_revision(&self, log_id: LogId) -> Result<Option<Revision>> {
        let revision = sqlx::query_as!(
            Revision,
            "SELECT revision_id, log_id, entity_id, message, forge_version, creation_time 
             FROM revisions WHERE log_id = $1 ORDER BY creation_time DESC LIMIT 1",
            log_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(revision)
    }

    pub async fn get_log_touches(&self, log_id: LogId) -> Result<Vec<LogTouch>> {
        let touches = sqlx::query_as!(
            LogTouch,
            "SELECT log_touch_id, log_id, entity_id, creation_time 
             FROM log_touches WHERE log_id = $1 ORDER BY creation_time ASC",
            log_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(touches)
    }
}