//! News page handler - 1:1 port of news.php
//! 
//! News system with two main views:
//! - News list (default)
//! - Specific news article view
//! Handles news navigation and article validation.

use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::Html,
};
use serde::Deserialize;
use crate::classes::{system::System, news::News};
use crate::session::PhpSession;
use he_db::DbPool;

/// Query parameters for news page navigation
#[derive(Debug, Deserialize)]
pub struct NewsQuery {
    pub id: Option<String>, // News article ID
}

/// News page navigation state
#[derive(Debug, Clone)]
pub struct NewsNavigation {
    pub news_str: String,
    pub title_str: String,
    pub got_new: bool,
    pub news_id: Option<i64>,
    pub news_title: Option<String>,
}

impl Default for NewsNavigation {
    fn default() -> Self {
        Self {
            news_str: "active".to_string(),
            title_str: String::new(),
            got_new: false,
            news_id: None,
            news_title: None,
        }
    }
}

/// Main news handler - displays news list or specific article
/// 
/// Port of: news.php
/// Features:
/// - Tabbed navigation between news list and specific articles
/// - News ID validation and existence checking
/// - Dynamic tab titles showing article names
/// - Error handling for invalid IDs and non-existent articles
/// - Session authentication not required (public news)
/// - Help link integration
pub async fn news_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(session): Extension<PhpSession>,
    Query(query): Query<NewsQuery>,
) -> Result<Html<String>, StatusCode> {
    // Initialize system classes
    let system = System::new();
    let mut news = News::new(db_pool);

    // Initialize navigation state
    let mut nav = NewsNavigation::default();

    // Process GET parameters for specific news article
    if let Some(id_str) = &query.id {
        if let Ok(news_id) = id_str.parse::<i64>() {
            // Validate that the news article exists
            match news.news_isset(news_id).await {
                Ok(true) => {
                    nav.title_str = "active".to_string();
                    nav.news_str = String::new();
                    nav.got_new = true;
                    nav.news_id = Some(news_id);
                    
                    // Get the news title for the tab
                    nav.news_title = news.get_title(news_id).await
                        .unwrap_or_else(|_| Some("Unknown Title".to_string()));
                },
                _ => {
                    // News article doesn't exist - redirect with error
                    let error_content = system.handle_error("This news does not exists.", "news.php");
                    return Ok(Html(error_content));
                }
            }
        } else {
            // Invalid ID format - redirect with error
            let error_content = system.handle_error("Invalid ID", "news.php");
            return Ok(Html(error_content));
        }
    }

    // Build page content
    let mut content = String::new();

    // Add message display if present
    if session.isset_msg() {
        content.push_str(&session.return_msg());
    }

    // Start widget structure with navigation tabs
    content.push_str(&format!(r#"
    <div class="span12">
        <div class="widget-box">
            <div class="widget-title">
                <ul class="nav nav-tabs">
                    <li class="link {}"><a href="news.php"><span class="he16-news icon-tab"></span>News</a></li>
    "#, nav.news_str));

    // Add specific news article tab if viewing an article
    if nav.got_new {
        let title = nav.news_title.as_deref().unwrap_or("News Article");
        content.push_str(&format!(r#"
                    <li class="link {}"><a href="news?id={}"><span class="he16-news_list icon-tab"></span>{}</a></li>
        "#, 
            nav.title_str, 
            nav.news_id.unwrap_or(0), 
            html_escape::encode_text(title)
        ));
    }

    // Close navigation and start content area
    content.push_str(&format!(r#"
                    <a href="#"><span class="label label-info">{}</span></a>
                </ul>
            </div>
            <div class="widget-content padding noborder">
    "#, translate("Help")));

    // Generate main content based on current view
    if !nav.got_new {
        // Default view - display news list
        match news.news_list().await {
            Ok(news_list_html) => {
                content.push_str(&news_list_html);
            },
            Err(e) => {
                eprintln!("Error loading news list: {:?}", e);
                content.push_str("<p class='error'>Error loading news list.</p>");
            }
        }
    } else {
        // Specific news article view
        if let Some(news_id) = nav.news_id {
            match news.show(news_id).await {
                Ok(article_html) => {
                    content.push_str(&article_html);
                },
                Err(e) => {
                    eprintln!("Error loading news article {}: {:?}", news_id, e);
                    content.push_str("<p class='error'>Error loading news article.</p>");
                }
            }
        }
    }

    // Close widget structure
    content.push_str(r#"
            </div>
            <div style="clear: both;" class="nav nav-tabs">&nbsp;</div>
        </div>
    </div>
    "#);

    Ok(Html(content))
}

/// Helper function for internationalization
/// TODO: Implement proper i18n system
fn translate(text: &str) -> String {
    // Placeholder - in full implementation this would use gettext or similar
    text.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_news_navigation_default() {
        let nav = NewsNavigation::default();
        assert_eq!(nav.news_str, "active");
        assert_eq!(nav.title_str, "");
        assert!(!nav.got_new);
        assert!(nav.news_id.is_none());
        assert!(nav.news_title.is_none());
    }

    #[test]
    fn test_news_id_parsing() {
        assert_eq!("123".parse::<i64>().unwrap(), 123);
        assert!("invalid".parse::<i64>().is_err());
    }

    #[test]
    fn test_news_navigation_with_article() {
        let mut nav = NewsNavigation::default();
        nav.title_str = "active".to_string();
        nav.news_str = String::new();
        nav.got_new = true;
        nav.news_id = Some(42);
        nav.news_title = Some("Test Article".to_string());
        
        assert_eq!(nav.title_str, "active");
        assert_eq!(nav.news_str, "");
        assert!(nav.got_new);
        assert_eq!(nav.news_id, Some(42));
        assert_eq!(nav.news_title, Some("Test Article".to_string()));
    }

    #[test]
    fn test_translate() {
        assert_eq!(translate("Help"), "Help");
        assert_eq!(translate("News"), "News");
    }
}