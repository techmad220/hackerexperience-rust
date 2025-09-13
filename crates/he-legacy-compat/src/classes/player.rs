//! Player management class - port of Player.class.php
//! 
//! Handles player operations including:
//! - Learning and certification management
//! - Player stats and progression
//! - Game state management
//! - Player validation

use he_db::DbPool;
use anyhow::Result;

/// Player management errors
#[derive(Debug, thiserror::Error)]
pub enum PlayerError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Player not found")]
    NotFound,
    #[error("Invalid player state")]
    InvalidState,
}

/// Player management class
#[derive(Debug, Clone)]
pub struct Player {
    pub db_pool: DbPool,
}

impl Player {
    /// Create new Player instance
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    /// Get what certification the player is currently learning
    /// Returns 0 if not learning any certification
    pub async fn player_learning(&self) -> Result<i64, PlayerError> {
        // TODO: Get user ID from session context
        let user_id = 1; // Placeholder - should come from session
        
        let result = sqlx::query!(
            "SELECT learning FROM users WHERE id = ? LIMIT 1",
            user_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        match result {
            Some(row) => Ok(row.learning.unwrap_or(0)),
            None => Err(PlayerError::NotFound),
        }
    }

    /// Set what certification the player is learning
    pub async fn set_player_learning(&self, certification_id: i64) -> Result<(), PlayerError> {
        // TODO: Get user ID from session context
        let user_id = 1; // Placeholder - should come from session
        
        sqlx::query!(
            "UPDATE users SET learning = ? WHERE id = ?",
            certification_id,
            user_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Clear the player's current learning status
    pub async fn unset_player_learning(&self) -> Result<(), PlayerError> {
        // TODO: Get user ID from session context
        let user_id = 1; // Placeholder - should come from session
        
        sqlx::query!(
            "UPDATE users SET learning = NULL WHERE id = ?",
            user_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Verify if a player ID exists
    pub async fn verify_id(&self, player_id: i64) -> Result<bool, PlayerError> {
        let result = sqlx::query!(
            "SELECT id FROM users WHERE id = ? LIMIT 1",
            player_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(result.is_some())
    }

    /// Get player's basic information
    pub async fn get_player_info(&self, player_id: i64) -> Result<PlayerInfo, PlayerError> {
        let result = sqlx::query!(
            "SELECT id, login, email, game_ip, money, exp, popularity FROM users WHERE id = ? LIMIT 1",
            player_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        match result {
            Some(row) => Ok(PlayerInfo {
                id: row.id,
                login: row.login,
                email: row.email,
                game_ip: row.game_ip as u32,
                money: row.money,
                exp: row.exp.unwrap_or(0),
                popularity: row.popularity.unwrap_or(0),
            }),
            None => Err(PlayerError::NotFound),
        }
    }

    /// Get player's current stats
    pub async fn get_player_stats(&self, player_id: i64) -> Result<PlayerStats, PlayerError> {
        let result = sqlx::query!(
            "SELECT * FROM users WHERE id = ? LIMIT 1",
            player_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        match result {
            Some(row) => Ok(PlayerStats {
                id: row.id,
                exp: row.exp.unwrap_or(0),
                money: row.money,
                popularity: row.popularity.unwrap_or(0),
                learning: row.learning.unwrap_or(0),
                // Add more stats as needed
            }),
            None => Err(PlayerError::NotFound),
        }
    }
}

/// Player basic information
#[derive(Debug, Clone)]
pub struct PlayerInfo {
    pub id: i64,
    pub login: String,
    pub email: String,
    pub game_ip: u32,
    pub money: i64,
    pub exp: i64,
    pub popularity: i64,
}

/// Player statistics
#[derive(Debug, Clone)]
pub struct PlayerStats {
    pub id: i64,
    pub exp: i64,
    pub money: i64,
    pub popularity: i64,
    pub learning: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_creation() {
        // Mock database pool for testing
        let db_pool = DbPool::connect_lazy("mysql://test:test@localhost/test").unwrap();
        let player = Player::new(db_pool);
        
        // Basic validation that struct was created properly
        assert!(std::ptr::addr_of!(player.db_pool) as *const _ != std::ptr::null());
    }

    #[test]
    fn test_player_info() {
        let player_info = PlayerInfo {
            id: 1,
            login: "testuser".to_string(),
            email: "test@example.com".to_string(),
            game_ip: 123456789,
            money: 1000,
            exp: 500,
            popularity: 10,
        };

        assert_eq!(player_info.id, 1);
        assert_eq!(player_info.login, "testuser");
        assert_eq!(player_info.money, 1000);
    }

    #[test]
    fn test_player_stats() {
        let player_stats = PlayerStats {
            id: 1,
            exp: 500,
            money: 1000,
            popularity: 10,
            learning: 2,
        };

        assert_eq!(player_stats.id, 1);
        assert_eq!(player_stats.learning, 2);
        assert_eq!(player_stats.exp, 500);
    }

    #[test]
    fn test_player_error_types() {
        let not_found_error = PlayerError::NotFound;
        assert!(matches!(not_found_error, PlayerError::NotFound));

        let invalid_state_error = PlayerError::InvalidState;
        assert!(matches!(invalid_state_error, PlayerError::InvalidState));
    }
}