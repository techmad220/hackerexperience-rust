use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ForumError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Topic not found: {0}")]
    TopicNotFound(u64),
    #[error("Post not found: {0}")]
    PostNotFound(u64),
    #[error("Category not found: {0}")]
    CategoryNotFound(u64),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("User is banned")]
    UserBanned,
    #[error("Topic is locked")]
    TopicLocked,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    Guest,
    User,
    Moderator,
    Admin,
    Banned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TopicStatus {
    Open,
    Locked,
    Pinned,
    Archived,
    Hidden,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PostStatus {
    Published,
    Hidden,
    Deleted,
    PendingModeration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForumCategory {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub parent_id: Option<u64>,
    pub position: u32,
    pub topic_count: u64,
    pub post_count: u64,
    pub last_post_id: Option<u64>,
    pub last_post_at: Option<DateTime<Utc>>,
    pub is_visible: bool,
    pub required_role: UserRole,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForumTopic {
    pub id: u64,
    pub category_id: u64,
    pub title: String,
    pub author_id: u64,
    pub author_name: String,
    pub status: TopicStatus,
    pub post_count: u64,
    pub view_count: u64,
    pub last_post_id: Option<u64>,
    pub last_post_at: Option<DateTime<Utc>>,
    pub last_poster_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_sticky: bool,
    pub is_locked: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForumPost {
    pub id: u64,
    pub topic_id: u64,
    pub author_id: u64,
    pub author_name: String,
    pub content: String,
    pub content_parsed: String,
    pub status: PostStatus,
    pub position: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub edited_by: Option<u64>,
    pub edit_reason: Option<String>,
    pub ip_address: String,
    pub user_agent: String,
    pub likes: u64,
    pub dislikes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForumUser {
    pub id: u64,
    pub username: String,
    pub role: UserRole,
    pub post_count: u64,
    pub reputation: i64,
    pub avatar_url: Option<String>,
    pub signature: Option<String>,
    pub joined_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub is_banned: bool,
    pub ban_reason: Option<String>,
    pub ban_expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForumPagination {
    pub page: u32,
    pub per_page: u32,
    pub total_items: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_previous: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicFilter {
    pub category_id: Option<u64>,
    pub author_id: Option<u64>,
    pub status: Option<TopicStatus>,
    pub sticky_only: bool,
    pub tags: Vec<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
}

/// Forum management system ported from PHP Forum class
/// Handles forum categories, topics, posts, and user interactions
pub struct Forum {
    current_user_id: Option<u64>,
    current_user_role: UserRole,
}

impl Forum {
    /// Create new Forum instance
    pub fn new(current_user_id: Option<u64>, current_user_role: Option<UserRole>) -> Self {
        Self {
            current_user_id,
            current_user_role: current_user_role.unwrap_or(UserRole::Guest),
        }
    }

    /// Get forum category by ID
    pub fn get_category(&self, category_id: u64) -> Result<ForumCategory, ForumError> {
        // Simulate database lookup
        if category_id == 0 {
            return Err(ForumError::CategoryNotFound(category_id));
        }

        // Check permissions
        let category = ForumCategory {
            id: category_id,
            name: "General Discussion".to_string(),
            description: "General forum discussion".to_string(),
            parent_id: None,
            position: 1,
            topic_count: 10,
            post_count: 50,
            last_post_id: Some(1),
            last_post_at: Some(Utc::now()),
            is_visible: true,
            required_role: UserRole::User,
            created_at: Utc::now(),
        };

        // Check if user has permission to view category
        if !self.can_view_category(&category) {
            return Err(ForumError::PermissionDenied);
        }

        Ok(category)
    }

    /// Get all categories
    pub fn get_categories(&self) -> Result<Vec<ForumCategory>, ForumError> {
        // Simulate database query
        let categories = vec![
            ForumCategory {
                id: 1,
                name: "General Discussion".to_string(),
                description: "General forum discussion".to_string(),
                parent_id: None,
                position: 1,
                topic_count: 10,
                post_count: 50,
                last_post_id: Some(1),
                last_post_at: Some(Utc::now()),
                is_visible: true,
                required_role: UserRole::User,
                created_at: Utc::now(),
            },
        ];

        // Filter categories user can view
        let visible_categories = categories
            .into_iter()
            .filter(|cat| self.can_view_category(cat))
            .collect();

        Ok(visible_categories)
    }

    /// Get topic by ID
    pub fn get_topic(&self, topic_id: u64) -> Result<ForumTopic, ForumError> {
        if topic_id == 0 {
            return Err(ForumError::TopicNotFound(topic_id));
        }

        let topic = ForumTopic {
            id: topic_id,
            category_id: 1,
            title: "Sample Topic".to_string(),
            author_id: 1,
            author_name: "TestUser".to_string(),
            status: TopicStatus::Open,
            post_count: 5,
            view_count: 25,
            last_post_id: Some(5),
            last_post_at: Some(Utc::now()),
            last_poster_name: Some("LastPoster".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_sticky: false,
            is_locked: false,
            tags: vec!["general".to_string()],
        };

        // Check if user can view this topic
        let category = self.get_category(topic.category_id)?;
        if !self.can_view_category(&category) {
            return Err(ForumError::PermissionDenied);
        }

        Ok(topic)
    }

    /// Get topics in category with pagination
    pub fn get_topics(
        &self,
        category_id: u64,
        filter: Option<TopicFilter>,
        page: u32,
        per_page: u32,
    ) -> Result<(Vec<ForumTopic>, ForumPagination), ForumError> {
        // Check category permissions
        let _category = self.get_category(category_id)?;

        let per_page = if per_page > 50 { 50 } else { per_page };
        let per_page = if per_page < 1 { 10 } else { per_page };

        // Simulate database query with filtering
        let topics = vec![]; // Would be populated from database

        let pagination = ForumPagination {
            page,
            per_page,
            total_items: 0,
            total_pages: 0,
            has_next: false,
            has_previous: page > 1,
        };

        Ok((topics, pagination))
    }

    /// Get post by ID
    pub fn get_post(&self, post_id: u64) -> Result<ForumPost, ForumError> {
        if post_id == 0 {
            return Err(ForumError::PostNotFound(post_id));
        }

        let post = ForumPost {
            id: post_id,
            topic_id: 1,
            author_id: 1,
            author_name: "TestUser".to_string(),
            content: "Sample post content".to_string(),
            content_parsed: "<p>Sample post content</p>".to_string(),
            status: PostStatus::Published,
            position: 1,
            created_at: Utc::now(),
            updated_at: None,
            edited_by: None,
            edit_reason: None,
            ip_address: "127.0.0.1".to_string(),
            user_agent: "Test User Agent".to_string(),
            likes: 0,
            dislikes: 0,
        };

        // Check if user can view this post
        let topic = self.get_topic(post.topic_id)?;
        let category = self.get_category(topic.category_id)?;
        if !self.can_view_category(&category) {
            return Err(ForumError::PermissionDenied);
        }

        Ok(post)
    }

    /// Get posts in topic with pagination
    pub fn get_posts(
        &self,
        topic_id: u64,
        page: u32,
        per_page: u32,
    ) -> Result<(Vec<ForumPost>, ForumPagination), ForumError> {
        // Check topic permissions
        let _topic = self.get_topic(topic_id)?;

        let per_page = if per_page > 25 { 25 } else { per_page };
        let per_page = if per_page < 1 { 10 } else { per_page };

        // Simulate database query
        let posts = vec![];

        let pagination = ForumPagination {
            page,
            per_page,
            total_items: 0,
            total_pages: 0,
            has_next: false,
            has_previous: page > 1,
        };

        Ok((posts, pagination))
    }

    /// Create new topic
    pub fn create_topic(
        &self,
        category_id: u64,
        title: String,
        content: String,
        tags: Vec<String>,
    ) -> Result<ForumTopic, ForumError> {
        // Check user permissions
        if self.current_user_id.is_none() {
            return Err(ForumError::PermissionDenied);
        }

        if matches!(self.current_user_role, UserRole::Banned) {
            return Err(ForumError::UserBanned);
        }

        // Check category permissions
        let category = self.get_category(category_id)?;
        if !self.can_post_in_category(&category) {
            return Err(ForumError::PermissionDenied);
        }

        // Validate input
        if title.trim().is_empty() {
            return Err(ForumError::Validation("Title cannot be empty".to_string()));
        }

        if title.len() > 255 {
            return Err(ForumError::Validation("Title too long".to_string()));
        }

        if content.trim().is_empty() {
            return Err(ForumError::Validation("Content cannot be empty".to_string()));
        }

        // Check rate limiting
        if !self.check_rate_limit()? {
            return Err(ForumError::RateLimitExceeded);
        }

        let topic_id = self.generate_id();
        let user_id = self.current_user_id.map_err(|e| anyhow::anyhow!("Error: {}", e))?;

        // Create topic
        let topic = ForumTopic {
            id: topic_id,
            category_id,
            title: title.trim().to_string(),
            author_id: user_id,
            author_name: "TestUser".to_string(), // Would fetch from user data
            status: TopicStatus::Open,
            post_count: 1,
            view_count: 0,
            last_post_id: None,
            last_post_at: Some(Utc::now()),
            last_poster_name: Some("TestUser".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_sticky: false,
            is_locked: false,
            tags: tags.into_iter().take(10).collect(), // Limit tags
        };

        // Create first post
        let _first_post = self.create_post_internal(topic_id, content)?;

        Ok(topic)
    }

    /// Create new post in topic
    pub fn create_post(&self, topic_id: u64, content: String) -> Result<ForumPost, ForumError> {
        // Check user permissions
        if self.current_user_id.is_none() {
            return Err(ForumError::PermissionDenied);
        }

        if matches!(self.current_user_role, UserRole::Banned) {
            return Err(ForumError::UserBanned);
        }

        // Check topic exists and permissions
        let topic = self.get_topic(topic_id)?;
        
        if topic.is_locked && !self.is_moderator_or_admin() {
            return Err(ForumError::TopicLocked);
        }

        let category = self.get_category(topic.category_id)?;
        if !self.can_post_in_category(&category) {
            return Err(ForumError::PermissionDenied);
        }

        // Check rate limiting
        if !self.check_rate_limit()? {
            return Err(ForumError::RateLimitExceeded);
        }

        self.create_post_internal(topic_id, content)
    }

    /// Internal method to create post
    fn create_post_internal(&self, topic_id: u64, content: String) -> Result<ForumPost, ForumError> {
        // Validate content
        if content.trim().is_empty() {
            return Err(ForumError::Validation("Content cannot be empty".to_string()));
        }

        if content.len() > 10000 {
            return Err(ForumError::Validation("Content too long".to_string()));
        }

        let post_id = self.generate_id();
        let user_id = self.current_user_id.map_err(|e| anyhow::anyhow!("Error: {}", e))?;

        // Parse content (would use purifier in real implementation)
        let parsed_content = format!("<p>{}</p>", content.trim());

        let post = ForumPost {
            id: post_id,
            topic_id,
            author_id: user_id,
            author_name: "TestUser".to_string(), // Would fetch from user data
            content: content.trim().to_string(),
            content_parsed: parsed_content,
            status: PostStatus::Published,
            position: 1, // Would calculate from database
            created_at: Utc::now(),
            updated_at: None,
            edited_by: None,
            edit_reason: None,
            ip_address: "127.0.0.1".to_string(), // Would get from request
            user_agent: "".to_string(), // Would get from request
            likes: 0,
            dislikes: 0,
        };

        Ok(post)
    }

    /// Edit existing post
    pub fn edit_post(
        &self,
        post_id: u64,
        content: String,
        edit_reason: Option<String>,
    ) -> Result<ForumPost, ForumError> {
        let mut post = self.get_post(post_id)?;
        
        // Check permissions
        if !self.can_edit_post(&post) {
            return Err(ForumError::PermissionDenied);
        }

        // Validate content
        if content.trim().is_empty() {
            return Err(ForumError::Validation("Content cannot be empty".to_string()));
        }

        if content.len() > 10000 {
            return Err(ForumError::Validation("Content too long".to_string()));
        }

        // Update post
        post.content = content.trim().to_string();
        post.content_parsed = format!("<p>{}</p>", content.trim());
        post.updated_at = Some(Utc::now());
        post.edited_by = self.current_user_id;
        post.edit_reason = edit_reason;

        Ok(post)
    }

    /// Delete post
    pub fn delete_post(&self, post_id: u64) -> Result<(), ForumError> {
        let post = self.get_post(post_id)?;
        
        // Check permissions
        if !self.can_delete_post(&post) {
            return Err(ForumError::PermissionDenied);
        }

        // Mark as deleted instead of actual deletion
        // In real implementation: UPDATE posts SET status = 'deleted' WHERE id = ?
        Ok(())
    }

    /// Lock/unlock topic
    pub fn lock_topic(&self, topic_id: u64, lock: bool) -> Result<(), ForumError> {
        let _topic = self.get_topic(topic_id)?;
        
        if !self.is_moderator_or_admin() {
            return Err(ForumError::PermissionDenied);
        }

        // Update topic lock status
        // In real implementation: UPDATE topics SET is_locked = ? WHERE id = ?
        Ok(())
    }

    /// Pin/unpin topic
    pub fn pin_topic(&self, topic_id: u64, pin: bool) -> Result<(), ForumError> {
        let _topic = self.get_topic(topic_id)?;
        
        if !self.is_moderator_or_admin() {
            return Err(ForumError::PermissionDenied);
        }

        // Update topic pin status
        // In real implementation: UPDATE topics SET is_sticky = ? WHERE id = ?
        Ok(())
    }

    /// Search forum content
    pub fn search(
        &self,
        query: &str,
        page: u32,
        per_page: u32,
    ) -> Result<(Vec<ForumPost>, ForumPagination), ForumError> {
        if query.trim().len() < 3 {
            return Err(ForumError::Validation("Search query must be at least 3 characters".to_string()));
        }

        // Simulate search functionality
        let posts = vec![];

        let pagination = ForumPagination {
            page,
            per_page,
            total_items: 0,
            total_pages: 0,
            has_next: false,
            has_previous: page > 1,
        };

        Ok((posts, pagination))
    }

    /// Increment topic view count
    pub fn increment_topic_views(&self, topic_id: u64) -> Result<(), ForumError> {
        // In real implementation: UPDATE topics SET view_count = view_count + 1 WHERE id = ?
        Ok(())
    }

    /// Permission checking methods
    fn can_view_category(&self, category: &ForumCategory) -> bool {
        if !category.is_visible {
            return self.is_moderator_or_admin();
        }

        match category.required_role {
            UserRole::Guest => true,
            UserRole::User => !matches!(self.current_user_role, UserRole::Guest | UserRole::Banned),
            UserRole::Moderator => matches!(self.current_user_role, UserRole::Moderator | UserRole::Admin),
            UserRole::Admin => matches!(self.current_user_role, UserRole::Admin),
            UserRole::Banned => false,
        }
    }

    fn can_post_in_category(&self, category: &ForumCategory) -> bool {
        if matches!(self.current_user_role, UserRole::Guest | UserRole::Banned) {
            return false;
        }

        self.can_view_category(category)
    }

    fn can_edit_post(&self, post: &ForumPost) -> bool {
        if matches!(self.current_user_role, UserRole::Banned) {
            return false;
        }

        // Own posts or moderator/admin
        if let Some(user_id) = self.current_user_id {
            post.author_id == user_id || self.is_moderator_or_admin()
        } else {
            false
        }
    }

    fn can_delete_post(&self, post: &ForumPost) -> bool {
        if matches!(self.current_user_role, UserRole::Banned) {
            return false;
        }

        // Only moderators and admins can delete posts
        self.is_moderator_or_admin() || 
            (self.current_user_id == Some(post.author_id) && post.position == 1) // Allow deleting own first post
    }

    fn is_moderator_or_admin(&self) -> bool {
        matches!(self.current_user_role, UserRole::Moderator | UserRole::Admin)
    }

    fn check_rate_limit(&self) -> Result<bool, ForumError> {
        // Simulate rate limiting check
        // In real implementation, check last post time from database
        Ok(true)
    }

    fn generate_id(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow::anyhow!("Error: {}", e))?
            .as_millis() as u64
    }
}

impl Default for TopicFilter {
    fn default() -> Self {
        Self {
            category_id: None,
            author_id: None,
            status: None,
            sticky_only: false,
            tags: vec![],
            date_from: None,
            date_to: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forum_creation() {
        let forum = Forum::new(Some(1), Some(UserRole::User));
        assert_eq!(forum.current_user_id, Some(1));
        assert!(matches!(forum.current_user_role, UserRole::User));
    }

    #[test]
    fn test_get_category() {
        let forum = Forum::new(Some(1), Some(UserRole::User));
        let result = forum.get_category(1);
        assert!(result.is_ok());

        let category = result.map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert_eq!(category.id, 1);
        assert_eq!(category.name, "General Discussion");
    }

    #[test]
    fn test_permission_checking() {
        let forum_user = Forum::new(Some(1), Some(UserRole::User));
        let forum_guest = Forum::new(None, Some(UserRole::Guest));
        let forum_admin = Forum::new(Some(2), Some(UserRole::Admin));

        let category = ForumCategory {
            id: 1,
            name: "Test".to_string(),
            description: "Test".to_string(),
            parent_id: None,
            position: 1,
            topic_count: 0,
            post_count: 0,
            last_post_id: None,
            last_post_at: None,
            is_visible: true,
            required_role: UserRole::User,
            created_at: Utc::now(),
        };

        assert!(forum_user.can_view_category(&category));
        assert!(!forum_guest.can_view_category(&category));
        assert!(forum_admin.can_view_category(&category));

        assert!(forum_user.can_post_in_category(&category));
        assert!(!forum_guest.can_post_in_category(&category));
        assert!(forum_admin.can_post_in_category(&category));
    }

    #[test]
    fn test_create_topic_validation() {
        let forum = Forum::new(Some(1), Some(UserRole::User));
        
        // Empty title
        let result = forum.create_topic(1, "".to_string(), "Content".to_string(), vec![]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ForumError::Validation(_)));

        // Empty content
        let result = forum.create_topic(1, "Title".to_string(), "".to_string(), vec![]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ForumError::Validation(_)));
    }

    #[test]
    fn test_banned_user_permissions() {
        let forum = Forum::new(Some(1), Some(UserRole::Banned));
        
        let result = forum.create_topic(1, "Title".to_string(), "Content".to_string(), vec![]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ForumError::UserBanned));
    }
}