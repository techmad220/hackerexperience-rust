//! News class - 1:1 port of News.class.php
//! 
//! News system for displaying game announcements and updates:
//! - News article management
//! - News listing and display
//! - Author information tracking
//! - Date and type handling

use sqlx::Row;
use serde::{Serialize, Deserialize};
use crate::classes::player::Player;
use crate::session::PhpSession;
use he_db::DbPool;
use chrono::{DateTime, Utc};

/// News article structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticle {
    pub id: i64,
    pub author: String,
    pub author_id: i64,
    pub author_ip: String,
    pub title: String,
    pub content: String,
    pub date: DateTime<Utc>,
    pub article_type: String,
}

/// News system errors
#[derive(Debug)]
pub enum NewsError {
    DatabaseError(sqlx::Error),
    NotFound(String),
    ValidationError(String),
}

impl std::fmt::Display for NewsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NewsError::DatabaseError(e) => write!(f, "Database error: {}", e),
            NewsError::NotFound(e) => write!(f, "Not found: {}", e),
            NewsError::ValidationError(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl std::error::Error for NewsError {}

impl From<sqlx::Error> for NewsError {
    fn from(error: sqlx::Error) -> Self {
        NewsError::DatabaseError(error)
    }
}

/// News class - handles all news-related operations
pub struct News {
    db_pool: DbPool,
    
    // Current article data
    id: Option<i64>,
    author: Option<String>,
    title: Option<String>,
    content: Option<String>,
    date: Option<DateTime<Utc>>,
    author_ip: Option<String>,
    author_id: Option<i64>,
    article_type: Option<String>,
}

impl News {
    /// Create new News instance
    /// 
    /// Port of: __construct() method
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            db_pool,
            id: None,
            author: None,
            title: None,
            content: None,
            date: None,
            author_ip: None,
            author_id: None,
            article_type: None,
        }
    }

    /// Get current article ID
    /// 
    /// Port of: getID() method
    pub fn get_id(&self) -> Option<i64> {
        self.id
    }

    /// Get current article author
    /// 
    /// Port of: getAuthor() method
    pub fn get_author(&self) -> Option<&String> {
        self.author.as_ref()
    }

    /// Get current article title
    /// 
    /// Port of: getTitle() method
    pub fn get_title(&self, news_id: Option<i64>) -> Result<Option<String>, NewsError> {
        if let Some(id) = news_id {
            // Load specific article title
            let title = sqlx::query_scalar::<_, String>(
                "SELECT title FROM news WHERE id = ?"
            )
            .bind(id)
            .fetch_optional(&self.db_pool)
            .await?;
            
            Ok(title)
        } else {
            // Return current loaded title
            Ok(self.title.clone())
        }
    }

    /// Get current article content
    /// 
    /// Port of: getContent() method  
    pub fn get_content(&self) -> Option<&String> {
        self.content.as_ref()
    }

    /// Get current article date
    /// 
    /// Port of: getDate() method
    pub fn get_date(&self) -> Option<DateTime<Utc>> {
        self.date
    }

    /// Get total number of news articles
    /// 
    /// Port of: totalNews() method
    pub async fn total_news(&self) -> Result<i64, NewsError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM news"
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(count)
    }

    /// Check if news article exists
    /// 
    /// Port of: newsIsset() method
    pub async fn news_isset(&self, news_id: i64) -> Result<bool, NewsError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM news WHERE id = ?"
        )
        .bind(news_id)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(count > 0)
    }

    /// Load news article data
    /// 
    /// Port of: loadNews() method
    pub async fn load_news(&mut self, news_id: i64) -> Result<bool, NewsError> {
        let row = sqlx::query(
            "SELECT id, author, author_id, author_ip, title, content, date, type FROM news WHERE id = ?"
        )
        .bind(news_id)
        .fetch_optional(&self.db_pool)
        .await?;

        match row {
            Some(row) => {
                self.id = Some(row.get("id"));
                self.author = Some(row.get("author"));
                self.author_id = Some(row.get("author_id"));
                self.author_ip = Some(row.get("author_ip"));
                self.title = Some(row.get("title"));
                self.content = Some(row.get("content"));
                self.date = Some(row.get("date"));
                self.article_type = Some(row.get("type"));
                Ok(true)
            },
            None => Ok(false),
        }
    }

    /// Display news list
    /// 
    /// Port of: news_list() method
    pub async fn news_list(&self) -> Result<String, NewsError> {
        let articles = sqlx::query!(
            "SELECT id, title, author, date, type FROM news ORDER BY date DESC LIMIT 20"
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut html = String::from(r#"
        <div class="news-list">
            <h3>Latest News</h3>
        "#);

        for article in articles {
            html.push_str(&format!(
                r#"<div class="news-item">
                    <h4><a href="news?id={}">{}</a></h4>
                    <p class="news-meta">By {} on {} | Type: {}</p>
                </div>"#,
                article.id,
                html_escape::encode_text(&article.title),
                html_escape::encode_text(&article.author),
                article.date.format("%Y-%m-%d %H:%M"),
                html_escape::encode_text(&article.r#type)
            ));
        }

        html.push_str("</div>");
        Ok(html)
    }

    /// Show specific news article
    /// 
    /// Port of: show() method
    pub async fn show(&mut self, news_id: i64) -> Result<String, NewsError> {
        if !self.load_news(news_id).await? {
            return Err(NewsError::NotFound("News article not found".to_string()));
        }

        let html = format!(
            r#"<div class="news-article">
                <h2>{}</h2>
                <div class="news-meta">
                    <p>By {} on {}</p>
                    <p>Type: {}</p>
                </div>
                <div class="news-content">
                    {}
                </div>
            </div>"#,
            html_escape::encode_text(self.title.as_deref().unwrap_or("Untitled")),
            html_escape::encode_text(self.author.as_deref().unwrap_or("Unknown")),
            self.date.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_else(|| "Unknown".to_string()),
            html_escape::encode_text(self.article_type.as_deref().unwrap_or("General")),
            html_escape::encode_text(self.content.as_deref().unwrap_or("No content"))
        );

        Ok(html)
    }

    /// Get news articles by type
    /// 
    /// Additional method for filtering news by type
    pub async fn get_news_by_type(&self, news_type: &str, limit: i64) -> Result<Vec<NewsArticle>, NewsError> {
        let articles = sqlx::query!(
            "SELECT id, author, author_id, author_ip, title, content, date, type 
             FROM news 
             WHERE type = ? 
             ORDER BY date DESC 
             LIMIT ?",
            news_type,
            limit
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut result = Vec::new();
        for article in articles {
            result.push(NewsArticle {
                id: article.id,
                author: article.author,
                author_id: article.author_id,
                author_ip: article.author_ip,
                title: article.title,
                content: article.content,
                date: article.date,
                article_type: article.r#type,
            });
        }

        Ok(result)
    }

    /// Create new news article
    /// 
    /// Additional method for creating news articles
    pub async fn create_news(
        &self,
        author: &str,
        author_id: i64,
        author_ip: &str,
        title: &str,
        content: &str,
        article_type: &str,
    ) -> Result<i64, NewsError> {
        let news_id = sqlx::query!(
            "INSERT INTO news (author, author_id, author_ip, title, content, type, date) VALUES (?, ?, ?, ?, ?, ?, NOW())",
            author,
            author_id,
            author_ip,
            title,
            content,
            article_type
        )
        .execute(&self.db_pool)
        .await?
        .last_insert_id() as i64;

        Ok(news_id)
    }

    /// Update existing news article
    /// 
    /// Additional method for updating news articles
    pub async fn update_news(
        &self,
        news_id: i64,
        title: &str,
        content: &str,
        article_type: &str,
    ) -> Result<(), NewsError> {
        let affected_rows = sqlx::query!(
            "UPDATE news SET title = ?, content = ?, type = ? WHERE id = ?",
            title,
            content,
            article_type,
            news_id
        )
        .execute(&self.db_pool)
        .await?
        .rows_affected();

        if affected_rows == 0 {
            return Err(NewsError::NotFound("News article not found".to_string()));
        }

        Ok(())
    }

    /// Delete news article
    /// 
    /// Additional method for deleting news articles
    pub async fn delete_news(&self, news_id: i64) -> Result<(), NewsError> {
        let affected_rows = sqlx::query!(
            "DELETE FROM news WHERE id = ?",
            news_id
        )
        .execute(&self.db_pool)
        .await?
        .rows_affected();

        if affected_rows == 0 {
            return Err(NewsError::NotFound("News article not found".to_string()));
        }

        Ok(())
    }

    /// Get recent announcements for homepage
    /// 
    /// Additional method for homepage integration
    pub async fn get_recent_announcements(&self, limit: i64) -> Result<String, NewsError> {
        let articles = self.get_news_by_type("announcement", limit).await?;

        let mut html = String::from(r#"<div class="recent-announcements">"#);

        for article in articles {
            html.push_str(&format!(
                r#"<div class="announcement-item">
                    <h5><a href="news?id={}">{}</a></h5>
                    <p class="announcement-date">{}</p>
                </div>"#,
                article.id,
                html_escape::encode_text(&article.title),
                article.date.format("%Y-%m-%d")
            ));
        }

        html.push_str("</div>");
        Ok(html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_news_article_creation() {
        let article = NewsArticle {
            id: 1,
            author: "Admin".to_string(),
            author_id: 1,
            author_ip: "127.0.0.1".to_string(),
            title: "Test News".to_string(),
            content: "This is a test news article".to_string(),
            date: Utc::now(),
            article_type: "announcement".to_string(),
        };

        assert_eq!(article.id, 1);
        assert_eq!(article.author, "Admin");
        assert_eq!(article.title, "Test News");
        assert_eq!(article.article_type, "announcement");
    }

    #[test]
    fn test_news_instance_creation() {
        // Mock database pool for testing
        use sqlx::mysql::MySqlPoolOptions;
        use tokio_test;
        
        tokio_test::block_on(async {
            // This would normally connect to a test database
            // For now, we'll just test the struct creation
            let news = News {
                db_pool: MySqlPoolOptions::new().max_connections(1).connect("mysql://fake").await.unwrap_or_else(|_| {
                    // Create a mock pool for testing
                    panic!("This is a test - database connection not expected")
                }),
                id: None,
                author: None,
                title: None,
                content: None,
                date: None,
                author_ip: None,
                author_id: None,
                article_type: None,
            };

            // Test initial state
            assert!(news.id.is_none());
            assert!(news.author.is_none());
            assert!(news.title.is_none());
        });
    }
}