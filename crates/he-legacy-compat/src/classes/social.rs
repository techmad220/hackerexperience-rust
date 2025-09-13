// SOCIAL.CLASS.PHP PORT - Social features, profiles, and friend system
// Original: User profiles, friend management, badges, and social interactions

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use sqlx::MySqlPool;
use serde::{Deserialize, Serialize};
use he_core::*;
use he_db::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: i64,
    pub username: String,
    pub level: i32,
    pub experience: i32,
    pub reputation: i32,
    pub money: i64,
    pub clan_id: Option<i64>,
    pub clan_name: Option<String>,
    pub join_date: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub profile_views: i32,
    pub total_hacks: i32,
    pub successful_hacks: i32,
    pub failed_hacks: i32,
    pub viruses_uploaded: i32,
    pub bio: Option<String>,
    pub badges: Vec<Badge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Badge {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub image_url: String,
    pub awarded_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRequest {
    pub id: i64,
    pub from_user_id: i64,
    pub to_user_id: i64,
    pub status: String, // 'pending', 'accepted', 'rejected'
    pub request_date: DateTime<Utc>,
    pub response_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Friendship {
    pub user1_id: i64,
    pub user2_id: i64,
    pub since_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileCache {
    pub user_id: i64,
    pub language: String,
    pub expire_date: DateTime<Utc>,
    pub generated_date: DateTime<Utc>,
}

pub struct Social {
    db_pool: MySqlPool,
    pub profile_id: Option<i64>,
    is_clan: bool,
}

impl Social {
    pub fn new(db_pool: MySqlPool) -> Self {
        Self {
            db_pool,
            profile_id: None,
            is_clan: false,
        }
    }
    
    // Original PHP: showProfile - Display user profile with caching
    pub async fn show_profile(&mut self, id: i64, language: String, viewer_id: Option<i64>) -> Result<String, SocialError> {
        // Click tracking (only if different user viewing)
        if let Some(viewer) = viewer_id {
            if viewer != id {
                self.click_profile(id).await?;
            }
        }
        
        self.profile_id = Some(id);
        
        // Check if cached profile exists and is valid
        let generate_profile = if self.is_profile_cached(id, &language).await? {
            !self.is_profile_valid(id).await?
        } else {
            true
        };
        
        if generate_profile {
            // TODO: Generate profile using Python equivalent
            // python.generateProfile(id, language);
            self.generate_profile(id, &language).await?;
        }
        
        self.profile_show_html(id, &language, viewer_id).await
    }
    
    // Original PHP: profile_show - Render profile HTML
    async fn profile_show_html(&self, id: i64, language: &str, viewer_id: Option<i64>) -> Result<String, SocialError> {
        let profile = self.get_user_profile(id).await?;
        
        // Check friendship status
        let friends_status = if let Some(viewer) = viewer_id {
            if viewer != id {
                if self.friend_exists(viewer, id).await? || self.friend_request_exists(viewer, id).await? {
                    1
                } else {
                    0
                }
            } else {
                1 // Own profile
            }
        } else {
            0 // Not logged in
        };
        
        // Render profile HTML (in real implementation, this would use a template engine)
        Ok(format!(
            r#"
            <div class="profile-container">
                <div class="profile-header">
                    <h1>{}</h1>
                    <div class="profile-stats">
                        <span>Level: {}</span>
                        <span>Experience: {}</span>
                        <span>Reputation: {}</span>
                    </div>
                </div>
                
                <div class="profile-content">
                    <div class="stats-section">
                        <h3>Hacking Statistics</h3>
                        <p>Total Hacks: {}</p>
                        <p>Successful: {}</p>
                        <p>Failed: {}</p>
                        <p>Viruses Uploaded: {}</p>
                    </div>
                    
                    <div class="clan-section">
                        {}
                    </div>
                    
                    <div class="badges-section">
                        <h3>Badges</h3>
                        <div class="badges-list">
                            {}
                        </div>
                    </div>
                </div>
                
                <script type="text/javascript">
                var fr = {};
                var uid = {};
                </script>
            </div>
            "#,
            profile.username,
            profile.level,
            profile.experience,
            profile.reputation,
            profile.total_hacks,
            profile.successful_hacks,
            profile.failed_hacks,
            profile.viruses_uploaded,
            if let Some(clan_name) = profile.clan_name {
                format!("<p>Clan: {}</p>", clan_name)
            } else {
                "No clan".to_string()
            },
            profile.badges.iter()
                .map(|badge| format!("<div class=\"badge\"><img src=\"{}\" title=\"{}\" /></div>", badge.image_url, badge.name))
                .collect::<Vec<_>>()
                .join(""),
            friends_status,
            id
        ))
    }
    
    // Original PHP: isProfileValid - Check if cached profile is still valid
    async fn is_profile_valid(&self, user_id: i64) -> Result<bool, SocialError> {
        let time_since_generated = sqlx::query_scalar::<_, Option<i32>>(
            "SELECT TIMESTAMPDIFF(SECOND, expireDate, NOW()) AS timeSinceGenerated 
             FROM cache_profile 
             WHERE userID = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(SocialError::DatabaseError)?
        .flatten();
        
        // Profile is valid if it hasn't expired (negative time means not expired yet)
        Ok(time_since_generated.map_or(false, |time| time < 0))
    }
    
    // Check if profile is cached
    async fn is_profile_cached(&self, user_id: i64, language: &str) -> Result<bool, SocialError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM cache_profile WHERE userID = ? AND language = ?"
        )
        .bind(user_id)
        .bind(language)
        .fetch_one(&self.db_pool)
        .await
        .map_err(SocialError::DatabaseError)?;
        
        Ok(count > 0)
    }
    
    // Original PHP: clickProfile - Increment profile view counter
    async fn click_profile(&self, id: i64) -> Result<(), SocialError> {
        sqlx::query("UPDATE users_stats SET profile_views = profile_views + 1 WHERE user_id = ?")
            .bind(id)
            .execute(&self.db_pool)
            .await
            .map_err(SocialError::DatabaseError)?;
        
        Ok(())
    }
    
    // Original PHP: profile_search - Search for user profiles
    pub async fn profile_search(&self, query: String) -> Result<Vec<UserProfile>, SocialError> {
        let search_term = format!("%{}%", query);
        
        let users = sqlx::query_as::<_, UserProfile>(
            "SELECT u.id as user_id, u.login as username, us.level, us.experience, us.reputation, 
                    us.money, u.clan_id, c.name as clan_name, u.registration_date as join_date,
                    u.lastLogin as last_login, us.profile_views, us.total_hacks, us.successful_hacks,
                    us.failed_hacks, us.viruses_uploaded, up.bio
             FROM users u
             LEFT JOIN users_stats us ON u.id = us.user_id
             LEFT JOIN clans c ON u.clan_id = c.id
             LEFT JOIN user_profiles up ON u.id = up.user_id
             WHERE u.login LIKE ? 
             ORDER BY us.reputation DESC
             LIMIT 20"
        )
        .bind(&search_term)
        .fetch_all(&self.db_pool)
        .await
        .map_err(SocialError::DatabaseError)?;
        
        // Get badges for each user
        let mut profiles = Vec::new();
        for mut user in users {
            user.badges = self.get_user_badges(user.user_id).await?;
            profiles.push(user);
        }
        
        Ok(profiles)
    }
    
    // Original PHP: badge_add - Add badge to user
    pub async fn badge_add(&self, badge_id: i32, user_id: i64, clan_id: Option<i64>) -> Result<(), SocialError> {
        sqlx::query(
            "INSERT INTO user_badges (user_id, badge_id, clan_id, awarded_date) VALUES (?, ?, ?, NOW())"
        )
        .bind(user_id)
        .bind(badge_id)
        .bind(clan_id)
        .execute(&self.db_pool)
        .await
        .map_err(SocialError::DatabaseError)?;
        
        Ok(())
    }
    
    // Original PHP: badge_list - Get badges for clan
    pub async fn badge_list(&self, clan_id: i64) -> Result<Vec<Badge>, SocialError> {
        let badges = sqlx::query_as::<_, Badge>(
            "SELECT b.id, b.name, b.description, b.image_url, ub.awarded_date
             FROM user_badges ub
             JOIN badges b ON ub.badge_id = b.id
             WHERE ub.clan_id = ?
             ORDER BY ub.awarded_date DESC"
        )
        .bind(clan_id)
        .fetch_all(&self.db_pool)
        .await
        .map_err(SocialError::DatabaseError)?;
        
        Ok(badges)
    }
    
    // Original PHP: profile_friends - Get user's friends list
    pub async fn profile_friends(&self, user_id: i64) -> Result<Vec<UserProfile>, SocialError> {
        let friends = sqlx::query_as::<_, UserProfile>(
            "SELECT u.id as user_id, u.login as username, us.level, us.experience, us.reputation,
                    us.money, u.clan_id, c.name as clan_name, u.registration_date as join_date,
                    u.lastLogin as last_login, us.profile_views, us.total_hacks, us.successful_hacks,
                    us.failed_hacks, us.viruses_uploaded, up.bio
             FROM friendships f
             JOIN users u ON (f.user1_id = ? AND f.user2_id = u.id) OR (f.user2_id = ? AND f.user1_id = u.id)
             LEFT JOIN users_stats us ON u.id = us.user_id
             LEFT JOIN clans c ON u.clan_id = c.id
             LEFT JOIN user_profiles up ON u.id = up.user_id
             WHERE u.id != ?
             ORDER BY f.since_date DESC"
        )
        .bind(user_id)
        .bind(user_id)
        .bind(user_id)
        .fetch_all(&self.db_pool)
        .await
        .map_err(SocialError::DatabaseError)?;
        
        Ok(friends)
    }
    
    // Original PHP: friend_isset - Check if friendship exists
    pub async fn friend_exists(&self, user1_id: i64, user2_id: i64) -> Result<bool, SocialError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM friendships 
             WHERE (user1_id = ? AND user2_id = ?) OR (user1_id = ? AND user2_id = ?)"
        )
        .bind(user1_id)
        .bind(user2_id)
        .bind(user2_id)
        .bind(user1_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(SocialError::DatabaseError)?;
        
        Ok(count > 0)
    }
    
    // Original PHP: friend_issetRequest - Check if friend request exists
    pub async fn friend_request_exists(&self, from_user_id: i64, to_user_id: i64) -> Result<bool, SocialError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM friend_requests 
             WHERE from_user_id = ? AND to_user_id = ? AND status = 'pending'"
        )
        .bind(from_user_id)
        .bind(to_user_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(SocialError::DatabaseError)?;
        
        Ok(count > 0)
    }
    
    // Original PHP: friend_request - Send friend request
    pub async fn friend_request(&self, from_user_id: i64, to_user_id: i64) -> Result<(), SocialError> {
        // Check if request already exists
        if self.friend_request_exists(from_user_id, to_user_id).await? {
            return Err(SocialError::RequestAlreadyExists);
        }
        
        // Check if they're already friends
        if self.friend_exists(from_user_id, to_user_id).await? {
            return Err(SocialError::AlreadyFriends);
        }
        
        sqlx::query(
            "INSERT INTO friend_requests (from_user_id, to_user_id, status, request_date) 
             VALUES (?, ?, 'pending', NOW())"
        )
        .bind(from_user_id)
        .bind(to_user_id)
        .execute(&self.db_pool)
        .await
        .map_err(SocialError::DatabaseError)?;
        
        Ok(())
    }
    
    // Original PHP: friend_add - Accept friend request
    pub async fn friend_add(&self, request_id: i64, accepting_user_id: i64) -> Result<(), SocialError> {
        // Get request details
        let request = sqlx::query_as::<_, FriendRequest>(
            "SELECT id, from_user_id, to_user_id, status, request_date, response_date
             FROM friend_requests 
             WHERE id = ? AND to_user_id = ? AND status = 'pending'"
        )
        .bind(request_id)
        .bind(accepting_user_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(SocialError::DatabaseError)?;
        
        let request = request.ok_or(SocialError::RequestNotFound)?;
        
        // Start transaction
        let mut tx = self.db_pool.begin().await.map_err(SocialError::DatabaseError)?;
        
        // Update request status
        sqlx::query("UPDATE friend_requests SET status = 'accepted', response_date = NOW() WHERE id = ?")
            .bind(request_id)
            .execute(&mut *tx)
            .await
            .map_err(SocialError::DatabaseError)?;
        
        // Create friendship
        sqlx::query("INSERT INTO friendships (user1_id, user2_id, since_date) VALUES (?, ?, NOW())")
            .bind(request.from_user_id)
            .bind(request.to_user_id)
            .execute(&mut *tx)
            .await
            .map_err(SocialError::DatabaseError)?;
        
        tx.commit().await.map_err(SocialError::DatabaseError)?;
        
        Ok(())
    }
    
    // Get friend count for user
    async fn friend_count(&self, user_id: i64) -> Result<i32, SocialError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM friendships 
             WHERE user1_id = ? OR user2_id = ?"
        )
        .bind(user_id)
        .bind(user_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(SocialError::DatabaseError)?;
        
        Ok(count as i32)
    }
    
    // Helper methods
    async fn get_user_profile(&self, user_id: i64) -> Result<UserProfile, SocialError> {
        let mut profile = sqlx::query_as::<_, UserProfile>(
            "SELECT u.id as user_id, u.login as username, us.level, us.experience, us.reputation,
                    us.money, u.clan_id, c.name as clan_name, u.registration_date as join_date,
                    u.lastLogin as last_login, us.profile_views, us.total_hacks, us.successful_hacks,
                    us.failed_hacks, us.viruses_uploaded, up.bio
             FROM users u
             LEFT JOIN users_stats us ON u.id = us.user_id
             LEFT JOIN clans c ON u.clan_id = c.id
             LEFT JOIN user_profiles up ON u.id = up.user_id
             WHERE u.id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(SocialError::DatabaseError)?;
        
        let mut profile = profile.ok_or(SocialError::UserNotFound(user_id))?;
        profile.badges = self.get_user_badges(user_id).await?;
        
        Ok(profile)
    }
    
    async fn get_user_badges(&self, user_id: i64) -> Result<Vec<Badge>, SocialError> {
        let badges = sqlx::query_as::<_, Badge>(
            "SELECT b.id, b.name, b.description, b.image_url, ub.awarded_date
             FROM user_badges ub
             JOIN badges b ON ub.badge_id = b.id
             WHERE ub.user_id = ?
             ORDER BY ub.awarded_date DESC"
        )
        .bind(user_id)
        .fetch_all(&self.db_pool)
        .await
        .map_err(SocialError::DatabaseError)?;
        
        Ok(badges)
    }
    
    async fn generate_profile(&self, user_id: i64, language: &str) -> Result<(), SocialError> {
        // TODO: Implement profile generation
        // This would typically create static HTML files for performance
        
        // Update cache entry
        sqlx::query(
            "INSERT INTO cache_profile (userID, language, generated_date, expire_date) 
             VALUES (?, ?, NOW(), DATE_ADD(NOW(), INTERVAL 1 HOUR))
             ON DUPLICATE KEY UPDATE generated_date = NOW(), expire_date = DATE_ADD(NOW(), INTERVAL 1 HOUR)"
        )
        .bind(user_id)
        .bind(language)
        .execute(&self.db_pool)
        .await
        .map_err(SocialError::DatabaseError)?;
        
        Ok(())
    }
}

// Implement FromRow for complex types
impl sqlx::FromRow<'_, sqlx::mysql::MySqlRow> for UserProfile {
    fn from_row(row: &'_ sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(UserProfile {
            user_id: row.try_get("user_id")?,
            username: row.try_get("username")?,
            level: row.try_get("level")?,
            experience: row.try_get("experience")?,
            reputation: row.try_get("reputation")?,
            money: row.try_get("money")?,
            clan_id: row.try_get("clan_id")?,
            clan_name: row.try_get("clan_name")?,
            join_date: row.try_get("join_date")?,
            last_login: row.try_get("last_login")?,
            profile_views: row.try_get("profile_views")?,
            total_hacks: row.try_get("total_hacks")?,
            successful_hacks: row.try_get("successful_hacks")?,
            failed_hacks: row.try_get("failed_hacks")?,
            viruses_uploaded: row.try_get("viruses_uploaded")?,
            bio: row.try_get("bio")?,
            badges: Vec::new(), // Populated separately
        })
    }
}

impl sqlx::FromRow<'_, sqlx::mysql::MySqlRow> for Badge {
    fn from_row(row: &'_ sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(Badge {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            image_url: row.try_get("image_url")?,
            awarded_date: row.try_get("awarded_date")?,
        })
    }
}

impl sqlx::FromRow<'_, sqlx::mysql::MySqlRow> for FriendRequest {
    fn from_row(row: &'_ sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(FriendRequest {
            id: row.try_get("id")?,
            from_user_id: row.try_get("from_user_id")?,
            to_user_id: row.try_get("to_user_id")?,
            status: row.try_get("status")?,
            request_date: row.try_get("request_date")?,
            response_date: row.try_get("response_date")?,
        })
    }
}

#[derive(Debug)]
pub enum SocialError {
    DatabaseError(sqlx::Error),
    UserNotFound(i64),
    RequestNotFound,
    RequestAlreadyExists,
    AlreadyFriends,
    InvalidLanguage(String),
}

impl std::fmt::Display for SocialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SocialError::DatabaseError(e) => write!(f, "Database error: {}", e),
            SocialError::UserNotFound(id) => write!(f, "User {} not found", id),
            SocialError::RequestNotFound => write!(f, "Friend request not found"),
            SocialError::RequestAlreadyExists => write!(f, "Friend request already exists"),
            SocialError::AlreadyFriends => write!(f, "Users are already friends"),
            SocialError::InvalidLanguage(lang) => write!(f, "Invalid language: {}", lang),
        }
    }
}

impl std::error::Error for SocialError {}