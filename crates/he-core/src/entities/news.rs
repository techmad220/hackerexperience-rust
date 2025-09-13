use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NewsError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Invalid news ID: {0}")]
    InvalidId(u64),
    #[error("Permission denied")]
    PermissionDenied,
    #[error("News not found")]
    NotFound,
    #[error("Validation error: {0}")]
    Validation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NewsStatus {
    Draft,
    Published,
    Archived,
    Hidden,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NewsType {
    General,
    Update,
    Maintenance,
    Event,
    Security,
    Feature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticle {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub summary: Option<String>,
    pub author_id: u64,
    pub author_name: String,
    pub news_type: NewsType,
    pub status: NewsStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub view_count: u64,
    pub comment_count: u64,
    pub featured: bool,
    pub sticky: bool,
    pub allow_comments: bool,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsComment {
    pub id: u64,
    pub news_id: u64,
    pub user_id: u64,
    pub username: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub parent_id: Option<u64>,
    pub is_deleted: bool,
    pub is_moderated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsPagination {
    pub page: u32,
    pub per_page: u32,
    pub total_items: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_previous: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsFilter {
    pub news_type: Option<NewsType>,
    pub status: Option<NewsStatus>,
    pub author_id: Option<u64>,
    pub tags: Vec<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub featured_only: bool,
    pub sticky_only: bool,
    pub language: Option<String>,
}

/// News management system ported from PHP News class
/// Handles news articles, comments, and content management
pub struct News {
    current_user_id: Option<u64>,
    language: String,
}

impl News {
    /// Create new News instance
    pub fn new(current_user_id: Option<u64>, language: Option<String>) -> Self {
        Self {
            current_user_id,
            language: language.unwrap_or_else(|| "en".to_string()),
        }
    }

    /// Get news article by ID
    pub fn get_article(&self, id: u64) -> Result<NewsArticle, NewsError> {
        // Simulate database lookup
        // In actual implementation, this would query the database
        if id == 0 {
            return Err(NewsError::InvalidId(id));
        }

        // Mock article for testing
        Ok(NewsArticle {
            id,
            title: "Sample News Article".to_string(),
            content: "This is a sample news article content.".to_string(),
            summary: Some("Sample summary".to_string()),
            author_id: 1,
            author_name: "Admin".to_string(),
            news_type: NewsType::General,
            status: NewsStatus::Published,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            published_at: Some(Utc::now()),
            tags: vec!["sample".to_string(), "news".to_string()],
            view_count: 0,
            comment_count: 0,
            featured: false,
            sticky: false,
            allow_comments: true,
            language: self.language.clone(),
        })
    }

    /// Get multiple news articles with pagination
    pub fn get_articles(
        &self,
        filter: Option<NewsFilter>,
        page: u32,
        per_page: u32,
    ) -> Result<(Vec<NewsArticle>, NewsPagination), NewsError> {
        let per_page = if per_page > 100 { 100 } else { per_page };
        let per_page = if per_page < 1 { 10 } else { per_page };

        // Simulate database query with filtering
        let articles = vec![]; // Would be populated from database

        let pagination = NewsPagination {
            page,
            per_page,
            total_items: 0,
            total_pages: 0,
            has_next: false,
            has_previous: page > 1,
        };

        Ok((articles, pagination))
    }

    /// Get latest news articles
    pub fn get_latest(&self, limit: u32) -> Result<Vec<NewsArticle>, NewsError> {
        let limit = if limit > 50 { 50 } else { limit };
        
        let filter = NewsFilter {
            status: Some(NewsStatus::Published),
            ..Default::default()
        };

        let (articles, _) = self.get_articles(Some(filter), 1, limit)?;
        Ok(articles)
    }

    /// Get featured news articles
    pub fn get_featured(&self, limit: u32) -> Result<Vec<NewsArticle>, NewsError> {
        let limit = if limit > 10 { 10 } else { limit };
        
        let filter = NewsFilter {
            status: Some(NewsStatus::Published),
            featured_only: true,
            ..Default::default()
        };

        let (articles, _) = self.get_articles(Some(filter), 1, limit)?;
        Ok(articles)
    }

    /// Get sticky news articles
    pub fn get_sticky(&self) -> Result<Vec<NewsArticle>, NewsError> {
        let filter = NewsFilter {
            status: Some(NewsStatus::Published),
            sticky_only: true,
            ..Default::default()
        };

        let (articles, _) = self.get_articles(Some(filter), 1, 10)?;
        Ok(articles)
    }

    /// Create new news article
    pub fn create_article(&self, mut article: NewsArticle) -> Result<NewsArticle, NewsError> {
        // Check permissions
        if self.current_user_id.is_none() {
            return Err(NewsError::PermissionDenied);
        }

        // Validate article
        self.validate_article(&article)?;

        // Set creation timestamp and author
        article.id = self.generate_id();
        article.author_id = self.current_user_id.unwrap();
        article.created_at = Utc::now();
        article.updated_at = Utc::now();
        article.language = self.language.clone();

        // If publishing, set published_at
        if matches!(article.status, NewsStatus::Published) {
            article.published_at = Some(Utc::now());
        }

        // Simulate database save
        Ok(article)
    }

    /// Update existing news article
    pub fn update_article(&self, id: u64, updates: HashMap<String, String>) -> Result<NewsArticle, NewsError> {
        // Check permissions
        if self.current_user_id.is_none() {
            return Err(NewsError::PermissionDenied);
        }

        let mut article = self.get_article(id)?;

        // Apply updates
        for (key, value) in updates {
            match key.as_str() {
                "title" => article.title = value,
                "content" => article.content = value,
                "summary" => article.summary = Some(value),
                "status" => {
                    article.status = match value.as_str() {
                        "draft" => NewsStatus::Draft,
                        "published" => NewsStatus::Published,
                        "archived" => NewsStatus::Archived,
                        "hidden" => NewsStatus::Hidden,
                        _ => return Err(NewsError::Validation("Invalid status".to_string())),
                    };
                }
                "featured" => article.featured = value.parse().unwrap_or(false),
                "sticky" => article.sticky = value.parse().unwrap_or(false),
                "allow_comments" => article.allow_comments = value.parse().unwrap_or(true),
                _ => {} // Ignore unknown fields
            }
        }

        article.updated_at = Utc::now();

        // If changing to published status, set published_at
        if matches!(article.status, NewsStatus::Published) && article.published_at.is_none() {
            article.published_at = Some(Utc::now());
        }

        self.validate_article(&article)?;

        // Simulate database update
        Ok(article)
    }

    /// Delete news article
    pub fn delete_article(&self, id: u64) -> Result<(), NewsError> {
        // Check permissions
        if self.current_user_id.is_none() {
            return Err(NewsError::PermissionDenied);
        }

        let article = self.get_article(id)?;

        // Check if user can delete this article
        if article.author_id != self.current_user_id.unwrap() {
            // In a real system, check for admin permissions here
            return Err(NewsError::PermissionDenied);
        }

        // Simulate database deletion
        Ok(())
    }

    /// Increment view count for article
    pub fn increment_view_count(&self, id: u64) -> Result<(), NewsError> {
        // Simulate database update
        // UPDATE news SET view_count = view_count + 1 WHERE id = ?
        Ok(())
    }

    /// Search news articles
    pub fn search(
        &self,
        query: &str,
        page: u32,
        per_page: u32,
    ) -> Result<(Vec<NewsArticle>, NewsPagination), NewsError> {
        if query.trim().is_empty() {
            return Err(NewsError::Validation("Search query cannot be empty".to_string()));
        }

        // Simulate search functionality
        // In actual implementation, this would use full-text search
        let articles = vec![];

        let pagination = NewsPagination {
            page,
            per_page,
            total_items: 0,
            total_pages: 0,
            has_next: false,
            has_previous: page > 1,
        };

        Ok((articles, pagination))
    }

    /// Get articles by tag
    pub fn get_by_tag(&self, tag: &str, page: u32, per_page: u32) -> Result<(Vec<NewsArticle>, NewsPagination), NewsError> {
        let filter = NewsFilter {
            tags: vec![tag.to_string()],
            status: Some(NewsStatus::Published),
            ..Default::default()
        };

        self.get_articles(Some(filter), page, per_page)
    }

    /// Get articles by author
    pub fn get_by_author(&self, author_id: u64, page: u32, per_page: u32) -> Result<(Vec<NewsArticle>, NewsPagination), NewsError> {
        let filter = NewsFilter {
            author_id: Some(author_id),
            status: Some(NewsStatus::Published),
            ..Default::default()
        };

        self.get_articles(Some(filter), page, per_page)
    }

    /// Get popular articles (by view count)
    pub fn get_popular(&self, limit: u32, days: u32) -> Result<Vec<NewsArticle>, NewsError> {
        let limit = if limit > 20 { 20 } else { limit };
        
        // Calculate date range
        let date_from = Utc::now() - chrono::Duration::days(days as i64);
        
        let filter = NewsFilter {
            status: Some(NewsStatus::Published),
            date_from: Some(date_from),
            ..Default::default()
        };

        let (articles, _) = self.get_articles(Some(filter), 1, limit)?;
        
        // In actual implementation, this would order by view_count DESC
        Ok(articles)
    }

    /// Validate news article
    fn validate_article(&self, article: &NewsArticle) -> Result<(), NewsError> {
        if article.title.trim().is_empty() {
            return Err(NewsError::Validation("Title cannot be empty".to_string()));
        }

        if article.title.len() > 255 {
            return Err(NewsError::Validation("Title too long (max 255 characters)".to_string()));
        }

        if article.content.trim().is_empty() {
            return Err(NewsError::Validation("Content cannot be empty".to_string()));
        }

        if article.content.len() > 65535 {
            return Err(NewsError::Validation("Content too long (max 65535 characters)".to_string()));
        }

        Ok(())
    }

    /// Generate unique ID (mock implementation)
    fn generate_id(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

impl Default for NewsFilter {
    fn default() -> Self {
        Self {
            news_type: None,
            status: None,
            author_id: None,
            tags: vec![],
            date_from: None,
            date_to: None,
            featured_only: false,
            sticky_only: false,
            language: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_news_creation() {
        let news = News::new(Some(1), Some("en".to_string()));
        assert_eq!(news.language, "en");
        assert_eq!(news.current_user_id, Some(1));
    }

    #[test]
    fn test_get_article() {
        let news = News::new(None, None);
        let result = news.get_article(1);
        assert!(result.is_ok());
        
        let article = result.unwrap();
        assert_eq!(article.id, 1);
        assert_eq!(article.title, "Sample News Article");
    }

    #[test]
    fn test_invalid_article_id() {
        let news = News::new(None, None);
        let result = news.get_article(0);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NewsError::InvalidId(0)));
    }

    #[test]
    fn test_create_article_without_permission() {
        let news = News::new(None, None);
        let article = NewsArticle {
            id: 0,
            title: "Test Article".to_string(),
            content: "Test content".to_string(),
            summary: None,
            author_id: 0,
            author_name: "Test User".to_string(),
            news_type: NewsType::General,
            status: NewsStatus::Draft,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            published_at: None,
            tags: vec![],
            view_count: 0,
            comment_count: 0,
            featured: false,
            sticky: false,
            allow_comments: true,
            language: "en".to_string(),
        };

        let result = news.create_article(article);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NewsError::PermissionDenied));
    }

    #[test]
    fn test_article_validation() {
        let news = News::new(Some(1), None);
        
        // Test empty title
        let article = NewsArticle {
            id: 0,
            title: "".to_string(),
            content: "Test content".to_string(),
            summary: None,
            author_id: 1,
            author_name: "Test User".to_string(),
            news_type: NewsType::General,
            status: NewsStatus::Draft,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            published_at: None,
            tags: vec![],
            view_count: 0,
            comment_count: 0,
            featured: false,
            sticky: false,
            allow_comments: true,
            language: "en".to_string(),
        };

        let result = news.create_article(article);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NewsError::Validation(_)));
    }
}