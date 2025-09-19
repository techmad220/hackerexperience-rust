//! Forum class - 1:1 port of Forum.class.php
//! 
//! Forum integration system for phpBB forum:
//! - Forum post display
//! - User session management
//! - Announcement handling
//! - Recent posts integration

use sqlx::Row;
use serde::{Serialize, Deserialize};
use he_db::DbPool;
use chrono::{DateTime, Utc};

/// Forum post structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForumPost {
    pub user_game_id: Option<i64>,
    pub user_id: i64,
    pub username: String,
    pub topic_title: String,
    pub topic_poster: i64,
    pub forum_id: i64,
    pub topic_id: i64,
    pub post_time: DateTime<Utc>,
    pub topic_replies: i64,
    pub topic_first_post_id: i64,
    pub poster_id: i64,
    pub post_id: i64,
    pub post_text: String,
    pub bbcode_bitfield: Option<String>,
    pub bbcode_uid: Option<String>,
}

/// Forum system errors
#[derive(Debug)]
pub enum ForumError {
    DatabaseError(sqlx::Error),
    InitializationError(String),
    PostProcessingError(String),
}

impl std::fmt::Display for ForumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ForumError::DatabaseError(e) => write!(f, "Database error: {}", e),
            ForumError::InitializationError(e) => write!(f, "Initialization error: {}", e),
            ForumError::PostProcessingError(e) => write!(f, "Post processing error: {}", e),
        }
    }
}

impl std::error::Error for ForumError {}

impl From<sqlx::Error> for ForumError {
    fn from(error: sqlx::Error) -> Self {
        ForumError::DatabaseError(error)
    }
}

/// Forum class - handles phpBB forum integration
pub struct Forum {
    db_pool: DbPool,
    phpbb_initialized: bool,
}

impl Forum {
    /// Create new Forum instance
    /// 
    /// Port of: __construct() method
    /// Note: Original PHP version initializes phpBB, but we'll handle this differently in Rust
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            db_pool,
            phpbb_initialized: false,
        }
    }

    /// Initialize forum system
    /// 
    /// Port of phpBB initialization from constructor
    /// In the original PHP version, this loads phpBB's common.php and related files
    pub async fn initialize(&mut self) -> Result<(), ForumError> {
        // TODO: In a full implementation, this would initialize phpBB components
        // For now, we'll mark as initialized
        self.phpbb_initialized = true;
        Ok(())
    }

    /// Show forum posts
    /// 
    /// Port of: showPosts() method
    /// Types: 'announcements', 'recent_posts'
    pub async fn show_posts(&self, post_type: &str) -> Result<String, ForumError> {
        match post_type {
            "announcements" => self.show_announcements().await,
            "recent_posts" => self.show_recent_posts().await,
            _ => Err(ForumError::PostProcessingError("Invalid post type".to_string())),
        }
    }

    /// Show announcements from forum
    /// 
    /// Port of announcements query from showPosts()
    async fn show_announcements(&self) -> Result<String, ForumError> {
        let posts = sqlx::query!(
            r#"SELECT u.user_game_id, u.user_id, u.username, t.topic_title, t.topic_poster, 
               t.forum_id, t.topic_id, t.topic_time, t.topic_replies, t.topic_first_post_id, 
               p.poster_id, p.topic_id as post_topic_id, p.post_id, p.post_text, p.bbcode_bitfield, p.bbcode_uid, p.post_time
               FROM phpbb_users u, phpbb_topics t, phpbb_posts p 
               WHERE u.user_id = t.topic_poster 
               AND u.user_id = p.poster_id 
               AND t.topic_id = p.topic_id 
               AND p.post_id = t.topic_first_post_id 
               AND t.forum_id = 26
               ORDER BY t.topic_time DESC 
               LIMIT 4"#
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut html = String::from(r#"<div class="forum-announcements"><h4>Latest Announcements</h4>"#);

        for post in posts {
            let processed_text = self.process_bbcode(&post.post_text, 
                post.bbcode_uid.as_deref(), 
                post.bbcode_bitfield.as_deref()).await?;

            html.push_str(&format!(
                r#"<div class="announcement-item">
                    <h5><a href="/forum/viewtopic.php?t={}">{}</a></h5>
                    <p class="announcement-meta">By {} | {} | Replies: {}</p>
                    <div class="announcement-excerpt">{}</div>
                </div>"#,
                post.topic_id,
                html_escape::encode_text(&post.topic_title),
                html_escape::encode_text(&post.username),
                post.topic_time.format("%Y-%m-%d %H:%M"),
                post.topic_replies,
                self.truncate_text(&processed_text, 200)
            ));
        }

        html.push_str("</div>");
        Ok(html)
    }

    /// Show recent forum posts
    /// 
    /// Port of recent_posts query from showPosts()
    async fn show_recent_posts(&self) -> Result<String, ForumError> {
        // TODO: Get clan ID from session - for now using placeholder
        let clan_id = 0i64;

        let posts = sqlx::query!(
            r#"SELECT u.user_game_id, u.user_id, u.username, t.topic_title, t.forum_id, 
               t.topic_id, t.topic_last_poster_name, t.topic_last_post_time, p.post_text
               FROM phpbb_users u, phpbb_topics t, phpbb_posts p
               WHERE u.user_id = t.topic_poster 
               AND u.user_id = p.poster_id 
               AND t.topic_id = p.topic_id 
               AND p.post_id = t.topic_first_post_id
               AND (t.topic_clan = 0 OR t.topic_clan = ?)
               ORDER BY t.topic_last_post_time DESC 
               LIMIT 10"#,
            clan_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut html = String::from(r#"<div class="forum-recent-posts"><h4>Recent Forum Activity</h4>"#);

        for post in posts {
            html.push_str(&format!(
                r#"<div class="recent-post-item">
                    <h6><a href="/forum/viewtopic.php?t={}">{}</a></h6>
                    <p class="post-meta">Started by {} | Last post by {} on {}</p>
                </div>"#,
                post.topic_id,
                html_escape::encode_text(&post.topic_title),
                html_escape::encode_text(&post.username),
                html_escape::encode_text(&post.topic_last_poster_name),
                post.topic_last_post_time.format("%Y-%m-%d %H:%M")
            ));
        }

        html.push_str("</div>");
        Ok(html)
    }

    /// Process BBCode formatting
    /// 
    /// Port of phpBB's BBCode processing
    /// This is a simplified version - full implementation would require phpBB's BBCode parser
    async fn process_bbcode(&self, text: &str, uid: Option<&str>, bitfield: Option<&str>) -> Result<String, ForumError> {
        // Simplified BBCode processing
        let mut processed = text.to_string();

        // Remove BBCode UID markers (phpBB internal formatting)
        if let Some(uid_str) = uid {
            processed = processed.replace(&format!(":{}", uid_str), "");
        }

        // Basic BBCode to HTML conversion
        processed = processed
            .replace("[b]", "<strong>")
            .replace("[/b]", "</strong>")
            .replace("[i]", "<em>")
            .replace("[/i]", "</em>")
            .replace("[u]", "<u>")
            .replace("[/u]", "</u>");

        // Handle URL BBCode
        let url_regex = regex::Regex::new(r"\[url=([^\]]+)\]([^\[]+)\[/url\]").unwrap();
        processed = url_regex.replace_all(&processed, r#"<a href="$1">$2</a>"#).to_string();

        // Handle simple URL BBCode
        let simple_url_regex = regex::Regex::new(r"\[url\]([^\[]+)\[/url\]").unwrap();
        processed = simple_url_regex.replace_all(&processed, r#"<a href="$1">$1</a>"#).to_string();

        // Handle quotes
        processed = processed
            .replace("[quote]", "<blockquote>")
            .replace("[/quote]", "</blockquote>");

        Ok(processed)
    }

    /// Truncate text to specified length
    fn truncate_text(&self, text: &str, max_length: usize) -> String {
        if text.len() <= max_length {
            html_escape::encode_text(text).to_string()
        } else {
            let truncated = &text[..max_length.min(text.len())];
            format!("{}...", html_escape::encode_text(truncated))
        }
    }

    /// Handle forum logout
    /// 
    /// Port of: logout() method
    pub async fn logout(&mut self) -> Result<(), ForumError> {
        // In the original PHP version, this would handle phpBB session cleanup
        // For now, we'll just mark as uninitialized
        self.phpbb_initialized = false;
        Ok(())
    }

    /// Get forum statistics
    /// 
    /// Additional method for forum integration
    pub async fn get_forum_stats(&self) -> Result<ForumStats, ForumError> {
        let total_topics = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM phpbb_topics WHERE topic_approved = 1"
        )
        .fetch_one(&self.db_pool)
        .await?;

        let total_posts = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM phpbb_posts WHERE post_approved = 1"
        )
        .fetch_one(&self.db_pool)
        .await?;

        let total_users = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM phpbb_users WHERE user_type != 2" // Exclude bots
        )
        .fetch_one(&self.db_pool)
        .await?;

        // Get latest post info
        let latest_post = sqlx::query!(
            "SELECT p.post_time, u.username, t.topic_title 
             FROM phpbb_posts p 
             JOIN phpbb_users u ON p.poster_id = u.user_id 
             JOIN phpbb_topics t ON p.topic_id = t.topic_id 
             WHERE p.post_approved = 1 
             ORDER BY p.post_time DESC 
             LIMIT 1"
        )
        .fetch_optional(&self.db_pool)
        .await?;

        let (latest_post_time, latest_poster, latest_topic) = if let Some(post) = latest_post {
            (Some(post.post_time), Some(post.username), Some(post.topic_title))
        } else {
            (None, None, None)
        };

        Ok(ForumStats {
            total_topics,
            total_posts,
            total_users,
            latest_post_time,
            latest_poster,
            latest_topic,
        })
    }

    /// Check if user has forum account
    /// 
    /// Additional method for user integration
    pub async fn user_has_forum_account(&self, game_user_id: i64) -> Result<bool, ForumError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM phpbb_users WHERE user_game_id = ?"
        )
        .bind(game_user_id)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(count > 0)
    }
}

/// Forum statistics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForumStats {
    pub total_topics: i64,
    pub total_posts: i64,
    pub total_users: i64,
    pub latest_post_time: Option<DateTime<Utc>>,
    pub latest_poster: Option<String>,
    pub latest_topic: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forum_post_creation() {
        let post = ForumPost {
            user_game_id: Some(123),
            user_id: 456,
            username: "testuser".to_string(),
            topic_title: "Test Topic".to_string(),
            topic_poster: 456,
            forum_id: 1,
            topic_id: 789,
            post_time: Utc::now(),
            topic_replies: 5,
            topic_first_post_id: 1000,
            poster_id: 456,
            post_id: 1000,
            post_text: "This is a test post".to_string(),
            bbcode_bitfield: None,
            bbcode_uid: None,
        };

        assert_eq!(post.username, "testuser");
        assert_eq!(post.topic_title, "Test Topic");
        assert_eq!(post.topic_replies, 5);
    }

    #[test]
    fn test_truncate_text() {
        let forum = Forum::new(
            // Mock database pool - this would be a real connection in practice
            sqlx::mysql::MySqlPoolOptions::new().max_connections(1).connect("mysql://fake").await.unwrap_or_else(|_| {
                panic!("This is a test - database connection not expected")
            })
        );

        let text = "This is a very long text that should be truncated";
        let truncated = forum.truncate_text(text, 20);
        assert!(truncated.len() <= 23); // 20 chars + "..."
        assert!(truncated.ends_with("..."));
    }

    #[tokio::test]
    async fn test_bbcode_processing() {
        let forum = Forum::new(
            // Mock database pool for testing
            sqlx::mysql::MySqlPoolOptions::new().max_connections(1).connect("mysql://fake").await.unwrap_or_else(|_| {
                panic!("This is a test - database connection not expected")
            })
        );

        let bbcode_text = "[b]Bold text[/b] and [i]italic text[/i]";
        let processed = forum.process_bbcode(bbcode_text, None, None).await.unwrap();
        
        assert!(processed.contains("<strong>Bold text</strong>"));
        assert!(processed.contains("<em>italic text</em>"));
    }
}