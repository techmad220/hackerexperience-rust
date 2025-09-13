//! Mission class - 1:1 port of Mission.class.php
//! 
//! Mission system for game storyline and tasks:
//! - Mission management and validation
//! - Mission state tracking
//! - Mission rewards and completion
//! - Mission abort functionality

use std::collections::HashMap;
use sqlx::Row;
use serde::{Serialize, Deserialize};
use crate::classes::{system::System, ranking::Ranking, list::Lists};
use crate::session::{PhpSession, SessionValue};
use he_db::DbPool;
use chrono::{DateTime, Utc};

/// Mission information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionInfo {
    pub id: i64,
    pub mission_type: i64,
    pub title: String,
    pub description: String,
    pub reward_money: i64,
    pub reward_experience: i64,
    pub required_level: i64,
    pub status: String,
    pub created_date: DateTime<Utc>,
    pub completed_date: Option<DateTime<Utc>>,
}

/// Mission system errors
#[derive(Debug)]
pub enum MissionError {
    DatabaseError(sqlx::Error),
    ValidationError(String),
    PermissionError(String),
    NotFound(String),
    InvalidState(String),
}

impl std::fmt::Display for MissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MissionError::DatabaseError(e) => write!(f, "Database error: {}", e),
            MissionError::ValidationError(e) => write!(f, "Validation error: {}", e),
            MissionError::PermissionError(e) => write!(f, "Permission error: {}", e),
            MissionError::NotFound(e) => write!(f, "Not found: {}", e),
            MissionError::InvalidState(e) => write!(f, "Invalid state: {}", e),
        }
    }
}

impl std::error::Error for MissionError {}

impl From<sqlx::Error> for MissionError {
    fn from(error: sqlx::Error) -> Self {
        MissionError::DatabaseError(error)
    }
}

/// Mission class - handles all mission-related operations
pub struct Mission {
    db_pool: DbPool,
    list: Lists,
    ranking: Ranking,
    
    // Current mission data
    level: Option<i64>,
    sidebar: Option<String>,
}

impl Mission {
    /// Create new Mission instance
    /// 
    /// Port of: __construct() method
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            list: Lists::new(db_pool.clone()),
            ranking: Ranking::new(db_pool.clone()),
            db_pool,
            level: None,
            sidebar: None,
        }
    }

    /// Handle POST data for mission operations
    /// 
    /// Port of: handlePost() method
    /// Actions:
    /// - abort: Abort current mission
    /// - accept: Accept new mission
    /// - complete: Complete mission
    pub async fn handle_post(&mut self, post_data: HashMap<String, String>) -> Result<String, MissionError> {
        let system = System::new();
        
        let act = post_data.get("act")
            .ok_or_else(|| MissionError::ValidationError("Invalid POST - missing action".to_string()))?;

        match act.as_str() {
            "abort" => self.handle_abort_mission(post_data).await,
            "accept" => self.handle_accept_mission(post_data).await,
            "complete" => self.handle_complete_mission(post_data).await,
            _ => Err(MissionError::ValidationError("Invalid action".to_string())),
        }
    }

    /// Handle mission abort
    /// 
    /// Port of abort case from handlePost()
    async fn handle_abort_mission(&mut self, _post_data: HashMap<String, String>) -> Result<String, MissionError> {
        // TODO: Get mission info from session - for now using placeholder values
        let mission_id = 1i64; // This should come from $_SESSION['MISSION_ID']
        let mission_type = 10i64; // This should come from $_SESSION['MISSION_TYPE']

        // Validate mission exists
        if !self.isset_mission(mission_id).await? {
            return Err(MissionError::NotFound("This mission does not exist".to_string()));
        }

        // Check mission type constraints
        if mission_type > 49 {
            return Err(MissionError::PermissionError("This mission cannot be aborted".to_string()));
        }

        // Abort the mission
        self.abort_mission(mission_id).await?;

        // Clear mission session data
        // TODO: Clear $_SESSION['MISSION_ID'], $_SESSION['MISSION_TYPE'], etc.

        Ok("Mission aborted successfully".to_string())
    }

    /// Handle mission acceptance
    async fn handle_accept_mission(&mut self, post_data: HashMap<String, String>) -> Result<String, MissionError> {
        let mission_id_str = post_data.get("mission_id")
            .ok_or_else(|| MissionError::ValidationError("Missing mission ID".to_string()))?;

        let mission_id: i64 = mission_id_str.parse()
            .map_err(|_| MissionError::ValidationError("Invalid mission ID".to_string()))?;

        // Validate mission exists and is available
        if !self.isset_mission(mission_id).await? {
            return Err(MissionError::NotFound("Mission does not exist".to_string()));
        }

        // Check if user already has a mission
        // TODO: Check session for existing mission

        // Accept the mission
        self.accept_mission(mission_id).await?;

        // Set mission session data
        // TODO: Set $_SESSION['MISSION_ID'], $_SESSION['MISSION_TYPE'], etc.

        Ok("Mission accepted successfully".to_string())
    }

    /// Handle mission completion
    async fn handle_complete_mission(&mut self, post_data: HashMap<String, String>) -> Result<String, MissionError> {
        // TODO: Get current mission from session
        let mission_id = 1i64; // This should come from session

        // Validate mission can be completed
        if !self.can_complete_mission(mission_id).await? {
            return Err(MissionError::InvalidState("Mission cannot be completed yet".to_string()));
        }

        // Complete the mission and give rewards
        self.complete_mission(mission_id).await?;

        Ok("Mission completed successfully".to_string())
    }

    /// Check if mission exists
    /// 
    /// Port of: issetMission() method
    pub async fn isset_mission(&self, mission_id: i64) -> Result<bool, MissionError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM missions WHERE id = ?"
        )
        .bind(mission_id)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(count > 0)
    }

    /// Get mission information
    pub async fn get_mission_info(&self, mission_id: i64) -> Result<Option<MissionInfo>, MissionError> {
        let row = sqlx::query(
            "SELECT id, mission_type, title, description, reward_money, reward_experience, required_level, status, created_date, completed_date FROM missions WHERE id = ?"
        )
        .bind(mission_id)
        .fetch_optional(&self.db_pool)
        .await?;

        match row {
            Some(row) => {
                let mission_info = MissionInfo {
                    id: row.get("id"),
                    mission_type: row.get("mission_type"),
                    title: row.get("title"),
                    description: row.get("description"),
                    reward_money: row.get("reward_money"),
                    reward_experience: row.get("reward_experience"),
                    required_level: row.get("required_level"),
                    status: row.get("status"),
                    created_date: row.get("created_date"),
                    completed_date: row.get("completed_date"),
                };
                Ok(Some(mission_info))
            },
            None => Ok(None),
        }
    }

    /// Abort mission
    async fn abort_mission(&self, mission_id: i64) -> Result<(), MissionError> {
        sqlx::query!(
            "UPDATE user_missions SET status = 'aborted', aborted_date = NOW() WHERE mission_id = ? AND status = 'active'",
            mission_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Accept mission
    async fn accept_mission(&self, mission_id: i64) -> Result<(), MissionError> {
        // TODO: Get user ID from session
        let user_id = 1i64;

        sqlx::query!(
            "INSERT INTO user_missions (user_id, mission_id, status, accepted_date) VALUES (?, ?, 'active', NOW())",
            user_id,
            mission_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Check if mission can be completed
    async fn can_complete_mission(&self, mission_id: i64) -> Result<bool, MissionError> {
        // TODO: Get user ID from session
        let user_id = 1i64;

        // Check if mission is active for user
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM user_missions WHERE user_id = ? AND mission_id = ? AND status = 'active'"
        )
        .bind(user_id)
        .bind(mission_id)
        .fetch_one(&self.db_pool)
        .await?;

        if count == 0 {
            return Ok(false);
        }

        // TODO: Add specific mission completion requirements check
        // This would depend on mission type and current game state

        Ok(true)
    }

    /// Complete mission and give rewards
    async fn complete_mission(&self, mission_id: i64) -> Result<(), MissionError> {
        // TODO: Get user ID from session
        let user_id = 1i64;

        // Get mission info for rewards
        let mission = self.get_mission_info(mission_id).await?
            .ok_or_else(|| MissionError::NotFound("Mission not found".to_string()))?;

        // Mark mission as completed
        sqlx::query!(
            "UPDATE user_missions SET status = 'completed', completed_date = NOW() WHERE user_id = ? AND mission_id = ? AND status = 'active'",
            user_id,
            mission_id
        )
        .execute(&self.db_pool)
        .await?;

        // Give rewards
        if mission.reward_money > 0 {
            sqlx::query!(
                "UPDATE users_stats SET money = money + ? WHERE user_id = ?",
                mission.reward_money,
                user_id
            )
            .execute(&self.db_pool)
            .await?;
        }

        if mission.reward_experience > 0 {
            sqlx::query!(
                "UPDATE users_stats SET experience = experience + ? WHERE user_id = ?",
                mission.reward_experience,
                user_id
            )
            .execute(&self.db_pool)
            .await?;
        }

        // Update ranking
        self.ranking.update_user_ranking(user_id).await
            .map_err(|_| MissionError::DatabaseError(sqlx::Error::RowNotFound))?;

        Ok(())
    }

    /// Get available missions for user
    pub async fn get_available_missions(&self, user_level: i64) -> Result<Vec<MissionInfo>, MissionError> {
        // TODO: Get user ID from session
        let user_id = 1i64;

        let missions = sqlx::query!(
            r#"SELECT m.id, m.mission_type, m.title, m.description, m.reward_money, m.reward_experience, m.required_level, 'available' as status, NOW() as created_date, NULL as completed_date
               FROM missions m 
               WHERE m.required_level <= ? 
               AND m.id NOT IN (
                   SELECT um.mission_id 
                   FROM user_missions um 
                   WHERE um.user_id = ? 
                   AND um.status IN ('completed', 'active')
               )
               ORDER BY m.required_level ASC, m.id ASC"#,
            user_level,
            user_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut result = Vec::new();
        for mission in missions {
            result.push(MissionInfo {
                id: mission.id,
                mission_type: mission.mission_type,
                title: mission.title,
                description: mission.description,
                reward_money: mission.reward_money,
                reward_experience: mission.reward_experience,
                required_level: mission.required_level,
                status: mission.status,
                created_date: mission.created_date,
                completed_date: mission.completed_date,
            });
        }

        Ok(result)
    }

    /// Display mission list
    pub async fn display_missions(&self) -> Result<String, MissionError> {
        // TODO: Get user level from session/database
        let user_level = 1i64;

        let missions = self.get_available_missions(user_level).await?;

        let mut html = String::from(r#"<div class="mission-list"><h3>Available Missions</h3>"#);

        for mission in missions {
            html.push_str(&format!(
                r#"<div class="mission-item">
                    <h4>{}</h4>
                    <p>{}</p>
                    <div class="mission-rewards">
                        <span class="money-reward">${}</span>
                        <span class="xp-reward">{} XP</span>
                    </div>
                    <form method="POST" style="display: inline;">
                        <input type="hidden" name="act" value="accept">
                        <input type="hidden" name="mission_id" value="{}">
                        <button type="submit" class="btn btn-success">Accept Mission</button>
                    </form>
                </div>"#,
                html_escape::encode_text(&mission.title),
                html_escape::encode_text(&mission.description),
                mission.reward_money,
                mission.reward_experience,
                mission.id
            ));
        }

        html.push_str("</div>");
        Ok(html)
    }

    /// Get current active mission for user
    pub async fn get_current_mission(&self) -> Result<Option<MissionInfo>, MissionError> {
        // TODO: Get user ID from session
        let user_id = 1i64;

        let row = sqlx::query(
            r#"SELECT m.id, m.mission_type, m.title, m.description, m.reward_money, m.reward_experience, m.required_level, um.status, um.accepted_date as created_date, um.completed_date
               FROM missions m 
               JOIN user_missions um ON m.id = um.mission_id 
               WHERE um.user_id = ? AND um.status = 'active' 
               LIMIT 1"#
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await?;

        match row {
            Some(row) => {
                let mission_info = MissionInfo {
                    id: row.get("id"),
                    mission_type: row.get("mission_type"),
                    title: row.get("title"),
                    description: row.get("description"),
                    reward_money: row.get("reward_money"),
                    reward_experience: row.get("reward_experience"),
                    required_level: row.get("required_level"),
                    status: row.get("status"),
                    created_date: row.get("created_date"),
                    completed_date: row.get("completed_date"),
                };
                Ok(Some(mission_info))
            },
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mission_info_creation() {
        let mission = MissionInfo {
            id: 1,
            mission_type: 10,
            title: "Test Mission".to_string(),
            description: "A test mission for validation".to_string(),
            reward_money: 1000,
            reward_experience: 500,
            required_level: 5,
            status: "available".to_string(),
            created_date: Utc::now(),
            completed_date: None,
        };

        assert_eq!(mission.id, 1);
        assert_eq!(mission.mission_type, 10);
        assert_eq!(mission.title, "Test Mission");
        assert_eq!(mission.reward_money, 1000);
        assert_eq!(mission.reward_experience, 500);
        assert!(mission.completed_date.is_none());
    }

    #[test]
    fn test_mission_error_display() {
        let error = MissionError::ValidationError("Test error".to_string());
        assert_eq!(format!("{}", error), "Validation error: Test error");
    }
}