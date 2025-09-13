use sqlx::{MySql, Pool};
use he_core::{User, UserStats, UserId, HeResult, HeError};
use chrono::{DateTime, Utc};

// User repository - replaces PHP Player.class.php database methods
pub struct UserRepository {
    pool: Pool<MySql>,
}

impl UserRepository {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }
    
    // Create new user - equivalent to PHP signup process
    pub async fn create_user(&self, mut user: User) -> HeResult<User> {
        let result = sqlx::query!(
            r#"
            INSERT INTO users (login, password, email, game_pass, game_ip, real_ip, home_ip, learning, premium)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            user.name,
            user.password_hash,
            user.email,
            "",  // game_pass will be generated
            0i64,  // game_ip will be assigned
            0i64,  // real_ip
            0i64,  // home_ip
            false, // learning
            user.is_premium
        )
        .execute(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        user.id = result.last_insert_id() as UserId;
        Ok(user)
    }
    
    // Find user by ID - core lookup method
    pub async fn find_by_id(&self, user_id: UserId) -> HeResult<Option<User>> {
        let row = sqlx::query!(
            "SELECT * FROM users WHERE id = ?",
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        if let Some(row) = row {
            let user = User {
                id: row.id as UserId,
                name: row.login,
                email: row.email,
                game_ip: row.game_ip.to_string(), // Convert from BIGINT
                cur_round: 1, // TODO: Get from round system
                clan_id: None, // TODO: Join with clan_members
                created_at: Utc::now(), // TODO: Add to migration
                last_login: Some(DateTime::from_timestamp(row.last_login.and_utc().timestamp(), 0).unwrap()),
                is_premium: row.premium != 0,
                is_online: false, // TODO: Check sessions table
                password_hash: row.password,
            };
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
    
    // Find user by login name - for authentication
    pub async fn find_by_login(&self, login: &str) -> HeResult<Option<User>> {
        let row = sqlx::query!(
            "SELECT * FROM users WHERE login = ?",
            login
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        if let Some(row) = row {
            let user = User {
                id: row.id as UserId,
                name: row.login,
                email: row.email,
                game_ip: row.game_ip.to_string(),
                cur_round: 1,
                clan_id: None,
                created_at: Utc::now(),
                last_login: Some(DateTime::from_timestamp(row.last_login.and_utc().timestamp(), 0).unwrap()),
                is_premium: row.premium != 0,
                is_online: false,
                password_hash: row.password,
            };
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
    
    // Update user's last login time
    pub async fn update_last_login(&self, user_id: UserId) -> HeResult<()> {
        sqlx::query!(
            "UPDATE users SET last_login = NOW() WHERE id = ?",
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        Ok(())
    }
    
    // Get user stats - separate table like original
    pub async fn get_user_stats(&self, user_id: UserId) -> HeResult<Option<UserStats>> {
        let row = sqlx::query!(
            "SELECT * FROM users_stats WHERE user_id = ?",
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        if let Some(row) = row {
            let stats = UserStats {
                user_id: row.user_id as UserId,
                reputation: row.reputation,
                money: row.money,
                experience: row.experience,
                total_hacks: row.total_hacks,
                successful_hacks: row.successful_hacks,
                failed_hacks: row.failed_hacks,
                viruses_uploaded: row.viruses_uploaded,
                round_stats_id: row.round_stats_id.map(|id| id as i64),
            };
            Ok(Some(stats))
        } else {
            Ok(None)
        }
    }
    
    // Create initial user stats
    pub async fn create_user_stats(&self, user_id: UserId) -> HeResult<UserStats> {
        let stats = UserStats::new(user_id);
        
        sqlx::query!(
            r#"
            INSERT INTO users_stats (user_id, reputation, money, experience, total_hacks, successful_hacks, failed_hacks, viruses_uploaded)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            stats.user_id,
            stats.reputation,
            stats.money,
            stats.experience,
            stats.total_hacks,
            stats.successful_hacks,
            stats.failed_hacks,
            stats.viruses_uploaded
        )
        .execute(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        Ok(stats)
    }
    
    // Update user stats
    pub async fn update_user_stats(&self, stats: &UserStats) -> HeResult<()> {
        sqlx::query!(
            r#"
            UPDATE users_stats 
            SET reputation = ?, money = ?, experience = ?, total_hacks = ?, 
                successful_hacks = ?, failed_hacks = ?, viruses_uploaded = ?
            WHERE user_id = ?
            "#,
            stats.reputation,
            stats.money,
            stats.experience,
            stats.total_hacks,
            stats.successful_hacks,
            stats.failed_hacks,
            stats.viruses_uploaded,
            stats.user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        Ok(())
    }
}