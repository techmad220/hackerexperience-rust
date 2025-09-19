use anyhow::{anyhow, Result};
use sqlx::{MySql, Pool};
use he_core::{Session, UserId, HeResult, HeError};
use chrono::{DateTime, Utc};

// Session repository - replaces PHP Session.class.php database methods
pub struct SessionRepository {
    pool: Pool<MySql>,
}

impl SessionRepository {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }
    
    // Create new session
    pub async fn create_session(&self, session: &Session) -> HeResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO sessions (id, user_id, language, query_count, buffer_query, exec_time, ip_address, user_agent, is_active)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            session.session_id,
            session.user_id,
            session.language,
            session.query_count,
            session.buffer_query,
            session.exec_time,
            session.ip_address,
            session.user_agent,
            session.is_active
        )
        .execute(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        Ok(())
    }
    
    // Get session by ID
    pub async fn get_session(&self, session_id: &str) -> HeResult<Option<Session>> {
        let row = sqlx::query!(
            "SELECT * FROM sessions WHERE id = ? AND is_active = 1",
            session_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        if let Some(row) = row {
            let session = Session {
                session_id: row.id,
                user_id: row.user_id.map(|id| id as UserId),
                language: row.language,
                query_count: row.query_count,
                buffer_query: row.buffer_query,
                exec_time: row.exec_time,
                created_at: DateTime::from_timestamp(row.created_at.and_utc().timestamp(), 0).map_err(|e| anyhow::anyhow!("Error: {}", e))?,
                last_activity: DateTime::from_timestamp(row.last_activity.and_utc().timestamp(), 0).map_err(|e| anyhow::anyhow!("Error: {}", e))?,
                ip_address: row.ip_address,
                user_agent: row.user_agent,
                is_active: row.is_active != 0,
            };
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }
    
    // Update session activity
    pub async fn update_session_activity(&self, session_id: &str) -> HeResult<()> {
        sqlx::query!(
            "UPDATE sessions SET last_activity = NOW() WHERE id = ?",
            session_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        Ok(())
    }
    
    // Authenticate session - link to user
    pub async fn authenticate_session(&self, session_id: &str, user_id: UserId) -> HeResult<()> {
        sqlx::query!(
            "UPDATE sessions SET user_id = ?, last_activity = NOW() WHERE id = ?",
            user_id,
            session_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        Ok(())
    }
    
    // Logout session
    pub async fn logout_session(&self, session_id: &str) -> HeResult<()> {
        sqlx::query!(
            "UPDATE sessions SET user_id = NULL, is_active = 0 WHERE id = ?",
            session_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        Ok(())
    }
    
    // Cleanup expired sessions
    pub async fn cleanup_expired_sessions(&self, timeout_hours: i32) -> HeResult<u64> {
        let result = sqlx::query!(
            "DELETE FROM sessions WHERE last_activity < DATE_SUB(NOW(), INTERVAL ? HOUR)",
            timeout_hours
        )
        .execute(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        Ok(result.rows_affected())
    }
    
    // Get active sessions for user
    pub async fn get_user_sessions(&self, user_id: UserId) -> HeResult<Vec<Session>> {
        let rows = sqlx::query!(
            "SELECT * FROM sessions WHERE user_id = ? AND is_active = 1",
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        let mut sessions = Vec::new();
        for row in rows {
            let session = Session {
                session_id: row.id,
                user_id: row.user_id.map(|id| id as UserId),
                language: row.language,
                query_count: row.query_count,
                buffer_query: row.buffer_query,
                exec_time: row.exec_time,
                created_at: DateTime::from_timestamp(row.created_at.and_utc().timestamp(), 0).map_err(|e| anyhow::anyhow!("Error: {}", e))?,
                last_activity: DateTime::from_timestamp(row.last_activity.and_utc().timestamp(), 0).map_err(|e| anyhow::anyhow!("Error: {}", e))?,
                ip_address: row.ip_address,
                user_agent: row.user_agent,
                is_active: row.is_active != 0,
            };
            sessions.push(session);
        }
        
        Ok(sessions)
    }
    
    // Increment query count for session
    pub async fn increment_query_count(&self, session_id: &str) -> HeResult<()> {
        sqlx::query!(
            "UPDATE sessions SET query_count = query_count + 1, last_activity = NOW() WHERE id = ?",
            session_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        Ok(())
    }
}