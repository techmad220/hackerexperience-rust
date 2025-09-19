use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SocialError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("User not found: {0}")]
    UserNotFound(u64),
    #[error("Profile not found: {0}")]
    ProfileNotFound(u64),
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Profile generation failed: {0}")]
    ProfileGenerationFailed(String),
    #[error("Cache error: {0}")]
    Cache(String),
    #[error("Friend request error: {0}")]
    FriendRequest(String),
    #[error("Already friends")]
    AlreadyFriends,
    #[error("Friend request already sent")]
    RequestAlreadySent,
    #[error("Invalid friend request")]
    InvalidFriendRequest,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Template error: {0}")]
    Template(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FriendshipStatus {
    None,
    RequestSent,
    RequestReceived,
    Friends,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProfileVisibility {
    Public,
    Friends,
    Private,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BadgeType {
    Achievement,
    Clan,
    Special,
    Event,
    Legacy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: u64,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: String,
    pub cover_image_url: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
    pub joined_at: DateTime<Utc>,
    pub last_seen: Option<DateTime<Utc>>,
    pub visibility: ProfileVisibility,
    pub is_online: bool,
    pub is_verified: bool,
    pub profile_views: u64,
    pub friend_count: u32,
    pub reputation: i64,
    pub level: u32,
    pub experience: u64,
    pub clan_id: Option<u64>,
    pub clan_name: Option<String>,
    pub achievements: Vec<ProfileAchievement>,
    pub badges: Vec<ProfileBadge>,
    pub stats: ProfileStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileAchievement {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub icon_url: String,
    pub achieved_at: DateTime<Utc>,
    pub rarity: String,
    pub points: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileBadge {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub icon_url: String,
    pub badge_type: BadgeType,
    pub earned_at: DateTime<Utc>,
    pub is_displayed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileStats {
    pub total_hacks: u64,
    pub successful_hacks: u64,
    pub total_money_earned: u64,
    pub missions_completed: u64,
    pub pvp_wins: u32,
    pub pvp_losses: u32,
    pub time_played: Duration,
    pub countries_visited: u32,
    pub servers_owned: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRequest {
    pub id: u64,
    pub from_user_id: u64,
    pub to_user_id: u64,
    pub from_username: String,
    pub to_username: String,
    pub message: Option<String>,
    pub sent_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Friendship {
    pub user_id: u64,
    pub friend_id: u64,
    pub username: String,
    pub friend_username: String,
    pub established_at: DateTime<Utc>,
    pub friendship_strength: u32, // Based on interactions
    pub last_interaction: Option<DateTime<Utc>>,
    pub is_best_friend: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileCacheInfo {
    pub user_id: u64,
    pub language: String,
    pub generated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub file_path: String,
    pub is_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialActivity {
    pub id: u64,
    pub user_id: u64,
    pub activity_type: String,
    pub description: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub is_public: bool,
    pub likes: u32,
    pub comments: u32,
}

#[derive(Debug, Clone)]
pub struct SocialConfig {
    pub profile_cache_duration: Duration,
    pub max_friends: u32,
    pub friend_request_expiry: Duration,
    pub rate_limit_friend_requests: u32,
    pub rate_limit_window: Duration,
    pub profile_template_path: String,
    pub default_avatar_url: String,
    pub enable_profile_views: bool,
    pub max_profile_views_per_session: u32,
}

impl Default for SocialConfig {
    fn default() -> Self {
        Self {
            profile_cache_duration: Duration::hours(1),
            max_friends: 500,
            friend_request_expiry: Duration::days(30),
            rate_limit_friend_requests: 10,
            rate_limit_window: Duration::hours(1),
            profile_template_path: "/var/www/templates/profile/".to_string(),
            default_avatar_url: "/images/default_avatar.png".to_string(),
            enable_profile_views: true,
            max_profile_views_per_session: 1,
        }
    }
}

/// Social features system ported from PHP Social class
/// Handles user profiles, friendships, and social interactions
pub struct Social {
    config: SocialConfig,
    current_user_id: Option<u64>,
    current_language: String,
}

impl Social {
    /// Create new Social instance
    pub fn new(config: SocialConfig, current_user_id: Option<u64>, language: Option<String>) -> Self {
        Self {
            config,
            current_user_id,
            current_language: language.unwrap_or_else(|| "en".to_string()),
        }
    }

    /// Create Social instance with default config
    pub fn default() -> Self {
        Self::new(SocialConfig::default(), None, None)
    }

    /// Create Social instance with user context
    pub fn with_user(user_id: u64, language: Option<String>) -> Self {
        Self::new(SocialConfig::default(), Some(user_id), language)
    }

    /// Show user profile (with caching and view tracking)
    pub fn show_profile(&self, profile_id: u64) -> Result<UserProfile, SocialError> {
        // Track profile view if different user
        if let Some(current_user) = self.current_user_id {
            if current_user != profile_id && self.config.enable_profile_views {
                self.track_profile_view(profile_id, current_user)?;
            }
        }

        // Check if cached profile exists and is valid
        let cache_info = self.get_profile_cache_info(profile_id)?;
        let generate_new = !cache_info.is_valid || cache_info.expires_at <= Utc::now();

        if generate_new {
            self.generate_profile_cache(profile_id)?;
        }

        // Load profile data
        self.load_profile_data(profile_id)
    }

    /// Generate and cache user profile
    fn generate_profile_cache(&self, user_id: u64) -> Result<(), SocialError> {
        // In real implementation, this would generate HTML profile using templates
        // and cache it to disk for faster loading
        
        // Mark cache as generated
        self.update_profile_cache_info(user_id)?;
        
        Ok(())
    }

    /// Check if profile cache is valid
    fn is_profile_cache_valid(&self, user_id: u64) -> Result<bool, SocialError> {
        let cache_info = self.get_profile_cache_info(user_id)?;
        Ok(cache_info.is_valid && cache_info.expires_at > Utc::now())
    }

    /// Track profile view
    fn track_profile_view(&self, profile_id: u64, viewer_id: u64) -> Result<(), SocialError> {
        // Check session to prevent multiple views
        if self.has_viewed_profile_in_session(profile_id, viewer_id)? {
            return Ok(());
        }

        // Increment profile view count
        self.increment_profile_views(profile_id)?;
        
        // Mark as viewed in session
        self.mark_profile_viewed_in_session(profile_id, viewer_id)?;

        Ok(())
    }

    /// Get friendship status between two users
    pub fn get_friendship_status(&self, user1_id: u64, user2_id: u64) -> Result<FriendshipStatus, SocialError> {
        if user1_id == user2_id {
            return Ok(FriendshipStatus::None);
        }

        // Check if they are friends
        if self.are_friends(user1_id, user2_id)? {
            return Ok(FriendshipStatus::Friends);
        }

        // Check for pending friend requests
        if self.has_sent_friend_request(user1_id, user2_id)? {
            return Ok(FriendshipStatus::RequestSent);
        }

        if self.has_sent_friend_request(user2_id, user1_id)? {
            return Ok(FriendshipStatus::RequestReceived);
        }

        // Check if blocked
        if self.is_blocked(user1_id, user2_id)? {
            return Ok(FriendshipStatus::Blocked);
        }

        Ok(FriendshipStatus::None)
    }

    /// Send friend request
    pub fn send_friend_request(&self, to_user_id: u64, message: Option<String>) -> Result<FriendRequest, SocialError> {
        let from_user_id = self.current_user_id.ok_or(SocialError::PermissionDenied)?;

        if from_user_id == to_user_id {
            return Err(SocialError::FriendRequest("Cannot send friend request to yourself".to_string()));
        }

        // Check if already friends
        if self.are_friends(from_user_id, to_user_id)? {
            return Err(SocialError::AlreadyFriends);
        }

        // Check if request already sent
        if self.has_sent_friend_request(from_user_id, to_user_id)? {
            return Err(SocialError::RequestAlreadySent);
        }

        // Check rate limiting
        self.check_friend_request_rate_limit(from_user_id)?;

        // Get user information
        let from_user = self.get_user_info(from_user_id)?;
        let to_user = self.get_user_info(to_user_id)?;

        // Create friend request
        let friend_request = FriendRequest {
            id: self.generate_id(),
            from_user_id,
            to_user_id,
            from_username: from_user.username,
            to_username: to_user.username,
            message,
            sent_at: Utc::now(),
            expires_at: Some(Utc::now() + self.config.friend_request_expiry),
            is_read: false,
        };

        // Save friend request
        self.save_friend_request(&friend_request)?;

        // Send notification email (if configured)
        self.send_friend_request_notification(&friend_request)?;

        Ok(friend_request)
    }

    /// Accept friend request
    pub fn accept_friend_request(&self, request_id: u64) -> Result<Friendship, SocialError> {
        let current_user_id = self.current_user_id.ok_or(SocialError::PermissionDenied)?;

        // Get friend request
        let friend_request = self.get_friend_request(request_id)?;

        // Verify request is for current user
        if friend_request.to_user_id != current_user_id {
            return Err(SocialError::PermissionDenied);
        }

        // Check if request hasn't expired
        if let Some(expires_at) = friend_request.expires_at {
            if expires_at <= Utc::now() {
                return Err(SocialError::InvalidFriendRequest);
            }
        }

        // Create friendship
        let friendship = Friendship {
            user_id: friend_request.from_user_id,
            friend_id: friend_request.to_user_id,
            username: friend_request.from_username.clone(),
            friend_username: friend_request.to_username.clone(),
            established_at: Utc::now(),
            friendship_strength: 1,
            last_interaction: Some(Utc::now()),
            is_best_friend: false,
        };

        // Save friendship (bidirectional)
        self.save_friendship(&friendship)?;

        // Delete friend request
        self.delete_friend_request(request_id)?;

        // Update friend counts
        self.update_friend_count(friend_request.from_user_id)?;
        self.update_friend_count(friend_request.to_user_id)?;

        // Check for friendship achievement
        self.check_friendship_achievements(friend_request.from_user_id)?;
        self.check_friendship_achievements(friend_request.to_user_id)?;

        Ok(friendship)
    }

    /// Decline friend request
    pub fn decline_friend_request(&self, request_id: u64) -> Result<(), SocialError> {
        let current_user_id = self.current_user_id.ok_or(SocialError::PermissionDenied)?;

        // Get friend request
        let friend_request = self.get_friend_request(request_id)?;

        // Verify request is for current user
        if friend_request.to_user_id != current_user_id {
            return Err(SocialError::PermissionDenied);
        }

        // Delete friend request
        self.delete_friend_request(request_id)?;

        Ok(())
    }

    /// Remove friend
    pub fn remove_friend(&self, friend_id: u64) -> Result<(), SocialError> {
        let current_user_id = self.current_user_id.ok_or(SocialError::PermissionDenied)?;

        // Verify they are friends
        if !self.are_friends(current_user_id, friend_id)? {
            return Err(SocialError::FriendRequest("Not friends".to_string()));
        }

        // Remove friendship (bidirectional)
        self.delete_friendship(current_user_id, friend_id)?;

        // Update friend counts
        self.update_friend_count(current_user_id)?;
        self.update_friend_count(friend_id)?;

        Ok(())
    }

    /// Get user's friends list
    pub fn get_user_friends(&self, user_id: u64, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<Friendship>, SocialError> {
        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        // Get friendships
        let friendships = self.load_user_friendships(user_id, limit, offset)?;

        Ok(friendships)
    }

    /// Get pending friend requests
    pub fn get_pending_friend_requests(&self, user_id: u64) -> Result<Vec<FriendRequest>, SocialError> {
        self.load_pending_friend_requests(user_id)
    }

    /// Search users
    pub fn search_users(&self, query: &str, limit: Option<u32>) -> Result<Vec<UserProfile>, SocialError> {
        if query.trim().len() < 2 {
            return Err(SocialError::Database("Search query too short".to_string()));
        }

        let limit = limit.unwrap_or(20).min(50);

        // Simulate user search
        // In real implementation, this would search database
        Ok(vec![])
    }

    /// Add badge to user
    pub fn add_badge(&self, user_id: u64, badge_id: u64, badge_type: BadgeType) -> Result<(), SocialError> {
        // Create badge record
        let badge = ProfileBadge {
            id: badge_id,
            name: format!("Badge {}", badge_id),
            description: "Achievement badge".to_string(),
            icon_url: format!("/images/badges/{}.png", badge_id),
            badge_type,
            earned_at: Utc::now(),
            is_displayed: true,
        };

        // Save badge
        self.save_user_badge(user_id, &badge)?;

        // Invalidate profile cache
        self.invalidate_profile_cache(user_id)?;

        Ok(())
    }

    /// Get most viewed profiles
    pub fn get_most_viewed_profiles(&self, limit: u32) -> Result<Vec<UserProfile>, SocialError> {
        let limit = limit.min(50);
        
        // Simulate query for most viewed profiles
        // SELECT * FROM user_profiles ORDER BY profile_views DESC LIMIT ?
        Ok(vec![])
    }

    /// Get user activity feed
    pub fn get_user_activity(&self, user_id: u64, limit: u32) -> Result<Vec<SocialActivity>, SocialError> {
        let limit = limit.min(100);
        
        // Load user's recent activity
        self.load_user_activity(user_id, limit)
    }

    /// Private helper methods
    fn get_profile_cache_info(&self, user_id: u64) -> Result<ProfileCacheInfo, SocialError> {
        // Check cache information
        Ok(ProfileCacheInfo {
            user_id,
            language: self.current_language.clone(),
            generated_at: Utc::now() - Duration::hours(2),
            expires_at: Utc::now() + Duration::hours(1),
            file_path: format!("/tmp/profile_{}_{}.html", user_id, self.current_language),
            is_valid: false, // Force regeneration for demo
        })
    }

    fn load_profile_data(&self, user_id: u64) -> Result<UserProfile, SocialError> {
        // Load profile data from database
        if user_id == 0 {
            return Err(SocialError::UserNotFound(user_id));
        }

        // Mock profile data
        Ok(UserProfile {
            user_id,
            username: format!("user_{}", user_id),
            display_name: Some(format!("User {}", user_id)),
            avatar_url: self.config.default_avatar_url.clone(),
            cover_image_url: None,
            bio: Some("This is a test user profile".to_string()),
            location: None,
            website: None,
            joined_at: Utc::now() - Duration::days(365),
            last_seen: Some(Utc::now() - Duration::minutes(30)),
            visibility: ProfileVisibility::Public,
            is_online: false,
            is_verified: false,
            profile_views: 42,
            friend_count: 15,
            reputation: 1000,
            level: 25,
            experience: 125000,
            clan_id: None,
            clan_name: None,
            achievements: vec![],
            badges: vec![],
            stats: ProfileStats {
                total_hacks: 100,
                successful_hacks: 85,
                total_money_earned: 500000,
                missions_completed: 50,
                pvp_wins: 20,
                pvp_losses: 5,
                time_played: Duration::hours(120),
                countries_visited: 15,
                servers_owned: 8,
            },
        })
    }

    fn update_profile_cache_info(&self, user_id: u64) -> Result<(), SocialError> {
        // Update cache timestamp in database
        Ok(())
    }

    fn has_viewed_profile_in_session(&self, profile_id: u64, viewer_id: u64) -> Result<bool, SocialError> {
        // Check session storage or cache
        // For now, always return false to allow view counting
        Ok(false)
    }

    fn increment_profile_views(&self, profile_id: u64) -> Result<(), SocialError> {
        // UPDATE user_profiles SET profile_views = profile_views + 1 WHERE user_id = ?
        Ok(())
    }

    fn mark_profile_viewed_in_session(&self, profile_id: u64, viewer_id: u64) -> Result<(), SocialError> {
        // Mark in session or temporary cache
        Ok(())
    }

    fn are_friends(&self, user1_id: u64, user2_id: u64) -> Result<bool, SocialError> {
        // Check if users are friends
        // SELECT COUNT(*) FROM friendships WHERE (user_id = ? AND friend_id = ?) OR (user_id = ? AND friend_id = ?)
        Ok(false) // Mock implementation
    }

    fn has_sent_friend_request(&self, from_user_id: u64, to_user_id: u64) -> Result<bool, SocialError> {
        // Check for pending friend request
        // SELECT COUNT(*) FROM friend_requests WHERE from_user_id = ? AND to_user_id = ?
        Ok(false) // Mock implementation
    }

    fn is_blocked(&self, user1_id: u64, user2_id: u64) -> Result<bool, SocialError> {
        // Check if users have blocked each other
        // SELECT COUNT(*) FROM user_blocks WHERE (blocker_id = ? AND blocked_id = ?) OR (blocker_id = ? AND blocked_id = ?)
        Ok(false) // Mock implementation
    }

    fn check_friend_request_rate_limit(&self, user_id: u64) -> Result<(), SocialError> {
        // Check rate limiting for friend requests
        let recent_requests = self.count_recent_friend_requests(user_id)?;
        if recent_requests >= self.config.rate_limit_friend_requests {
            return Err(SocialError::RateLimitExceeded);
        }
        Ok(())
    }

    fn count_recent_friend_requests(&self, user_id: u64) -> Result<u32, SocialError> {
        // Count friend requests sent in the rate limit window
        // SELECT COUNT(*) FROM friend_requests WHERE from_user_id = ? AND sent_at > ?
        Ok(0) // Mock implementation
    }

    fn get_user_info(&self, user_id: u64) -> Result<UserInfo, SocialError> {
        if user_id == 0 {
            return Err(SocialError::UserNotFound(user_id));
        }

        Ok(UserInfo {
            id: user_id,
            username: format!("user_{}", user_id),
            email: format!("user_{}@example.com", user_id),
        })
    }

    fn save_friend_request(&self, request: &FriendRequest) -> Result<(), SocialError> {
        // INSERT INTO friend_requests
        Ok(())
    }

    fn send_friend_request_notification(&self, request: &FriendRequest) -> Result<(), SocialError> {
        // Send email notification about friend request
        Ok(())
    }

    fn get_friend_request(&self, request_id: u64) -> Result<FriendRequest, SocialError> {
        // Mock friend request
        Ok(FriendRequest {
            id: request_id,
            from_user_id: 1,
            to_user_id: 2,
            from_username: "user_1".to_string(),
            to_username: "user_2".to_string(),
            message: None,
            sent_at: Utc::now() - Duration::hours(1),
            expires_at: Some(Utc::now() + Duration::days(29)),
            is_read: false,
        })
    }

    fn save_friendship(&self, friendship: &Friendship) -> Result<(), SocialError> {
        // INSERT INTO friendships (bidirectional)
        Ok(())
    }

    fn delete_friend_request(&self, request_id: u64) -> Result<(), SocialError> {
        // DELETE FROM friend_requests WHERE id = ?
        Ok(())
    }

    fn delete_friendship(&self, user1_id: u64, user2_id: u64) -> Result<(), SocialError> {
        // DELETE FROM friendships WHERE (user_id = ? AND friend_id = ?) OR (user_id = ? AND friend_id = ?)
        Ok(())
    }

    fn update_friend_count(&self, user_id: u64) -> Result<(), SocialError> {
        // UPDATE user_profiles SET friend_count = (SELECT COUNT(*) FROM friendships WHERE user_id = ? OR friend_id = ?) WHERE user_id = ?
        Ok(())
    }

    fn check_friendship_achievements(&self, user_id: u64) -> Result<(), SocialError> {
        // Check and award friendship-related achievements
        let friend_count = self.get_friend_count(user_id)?;
        
        // Award badges for friendship milestones
        match friend_count {
            10 => self.add_badge(user_id, 48, BadgeType::Achievement)?,
            50 => self.add_badge(user_id, 49, BadgeType::Achievement)?,
            _ => {}
        }

        Ok(())
    }

    fn get_friend_count(&self, user_id: u64) -> Result<u32, SocialError> {
        // Get user's friend count
        Ok(0) // Mock implementation
    }

    fn load_user_friendships(&self, user_id: u64, limit: u32, offset: u32) -> Result<Vec<Friendship>, SocialError> {
        // Load friendships from database
        Ok(vec![])
    }

    fn load_pending_friend_requests(&self, user_id: u64) -> Result<Vec<FriendRequest>, SocialError> {
        // Load pending friend requests
        Ok(vec![])
    }

    fn save_user_badge(&self, user_id: u64, badge: &ProfileBadge) -> Result<(), SocialError> {
        // INSERT INTO user_badges
        Ok(())
    }

    fn invalidate_profile_cache(&self, user_id: u64) -> Result<(), SocialError> {
        // Mark profile cache as invalid to force regeneration
        Ok(())
    }

    fn load_user_activity(&self, user_id: u64, limit: u32) -> Result<Vec<SocialActivity>, SocialError> {
        // Load user's recent social activity
        Ok(vec![])
    }

    fn generate_id(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow::anyhow!("Error: {}", e))?
            .as_millis() as u64
    }
}

#[derive(Debug, Clone)]
struct UserInfo {
    id: u64,
    username: String,
    email: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_social_creation() {
        let social = Social::default();
        assert_eq!(social.current_language, "en");
        assert!(social.current_user_id.is_none());
    }

    #[test]
    fn test_social_with_user() {
        let social = Social::with_user(123, Some("pt_BR".to_string()));
        assert_eq!(social.current_user_id, Some(123));
        assert_eq!(social.current_language, "pt_BR");
    }

    #[test]
    fn test_friendship_status_enum() {
        let status = FriendshipStatus::Friends;
        assert!(matches!(status, FriendshipStatus::Friends));
    }

    #[test]
    fn test_profile_visibility() {
        let visibility = ProfileVisibility::Public;
        assert!(matches!(visibility, ProfileVisibility::Public));
    }

    #[test]
    fn test_badge_type() {
        let badge_type = BadgeType::Achievement;
        assert!(matches!(badge_type, BadgeType::Achievement));
    }

    #[test]
    fn test_show_profile() {
        let social = Social::with_user(1, None);
        let result = social.show_profile(2);
        assert!(result.is_ok());

        let profile = result.map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert_eq!(profile.user_id, 2);
        assert_eq!(profile.username, "user_2");
    }

    #[test]
    fn test_show_own_profile() {
        let social = Social::with_user(1, None);
        let result = social.show_profile(1);
        assert!(result.is_ok());

        let profile = result.map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert_eq!(profile.user_id, 1);
    }

    #[test]
    fn test_friendship_status_none() {
        let social = Social::default();
        let result = social.get_friendship_status(1, 2);
        assert!(result.is_ok());
        assert!(matches!(result.map_err(|e| anyhow::anyhow!("Error: {}", e))?, FriendshipStatus::None));
    }

    #[test]
    fn test_friendship_status_same_user() {
        let social = Social::default();
        let result = social.get_friendship_status(1, 1);
        assert!(result.is_ok());
        assert!(matches!(result.map_err(|e| anyhow::anyhow!("Error: {}", e))?, FriendshipStatus::None));
    }

    #[test]
    fn test_send_friend_request_no_user() {
        let social = Social::default();
        let result = social.send_friend_request(2, None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SocialError::PermissionDenied));
    }

    #[test]
    fn test_send_friend_request_to_self() {
        let social = Social::with_user(1, None);
        let result = social.send_friend_request(1, None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SocialError::FriendRequest(_)));
    }

    #[test]
    fn test_profile_cache_info() {
        let social = Social::default();
        let result = social.get_profile_cache_info(1);
        assert!(result.is_ok());

        let cache_info = result.map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert_eq!(cache_info.user_id, 1);
        assert!(!cache_info.is_valid); // Forced to false in mock
    }

    #[test]
    fn test_profile_stats_serialization() {
        let stats = ProfileStats {
            total_hacks: 100,
            successful_hacks: 85,
            total_money_earned: 500000,
            missions_completed: 50,
            pvp_wins: 20,
            pvp_losses: 5,
            time_played: Duration::hours(120),
            countries_visited: 15,
            servers_owned: 8,
        };

        let json = serde_json::to_string(&stats).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let deserialized: ProfileStats = serde_json::from_str(&json).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        assert_eq!(stats.total_hacks, deserialized.total_hacks);
        assert_eq!(stats.successful_hacks, deserialized.successful_hacks);
        assert_eq!(stats.total_money_earned, deserialized.total_money_earned);
    }

    #[test]
    fn test_friend_request_creation() {
        let request = FriendRequest {
            id: 1,
            from_user_id: 1,
            to_user_id: 2,
            from_username: "user1".to_string(),
            to_username: "user2".to_string(),
            message: Some("Hi, let's be friends!".to_string()),
            sent_at: Utc::now(),
            expires_at: Some(Utc::now() + Duration::days(30)),
            is_read: false,
        };

        assert_eq!(request.from_user_id, 1);
        assert_eq!(request.to_user_id, 2);
        assert!(request.message.is_some());
        assert!(!request.is_read);
    }
}