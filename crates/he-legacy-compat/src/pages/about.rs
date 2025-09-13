use axum::{
    extract::Query,
    http::StatusCode,
    response::{Html, Redirect},
    Extension,
};
use serde::Deserialize;
use std::collections::HashMap;
use he_core::session::PhpSession;
use sqlx::PgPool;

/// Query parameters for about page - matches original PHP $_GET handling
#[derive(Debug, Deserialize)]
pub struct AboutQuery {
    pub page: Option<String>,
    pub id: Option<String>,
}

/// About page handler - 1:1 port of about.php
/// 
/// Original: Information page with version display and changelog functionality
/// Features:
/// - Shows current version and version status
/// - Provides link to changelog
/// - Displays detailed changelog entries when requested
/// - Supports viewing individual changelog entries by ID
/// - Requires user authentication (redirects to index.php if not logged in)
pub async fn about_handler(
    Extension(db): Extension<PgPool>,
    Extension(session): Extension<PhpSession>,
    Query(params): Query<AboutQuery>,
) -> Result<Html<String>, StatusCode> {
    
    // Check if user is logged in (equivalent to $session->issetLogin())
    if !session.is_logged_in() {
        // Original PHP: header("Location:index.php");
        return Ok(Html(r#"
            <script>window.location.href = "index.php";</script>
            <meta http-equiv="refresh" content="0; url=index.php">
        "#.to_string()));
    }
    
    let user_id = session.get_user_id().unwrap_or(0);
    
    // Parse query parameters - equivalent to original PHP logic
    let got_page = params.page.as_ref().map_or(false, |p| p == "changelog");
    let got_id = params.id.as_ref().and_then(|id| id.parse::<u32>().ok()).is_some();
    let changelog_id = params.id.as_ref().and_then(|id| id.parse::<u32>().ok()).unwrap_or(0);
    
    // TODO: Get actual version information from config
    let version = "2.0.0"; // Placeholder - should come from global config
    let version_status = "-beta"; // Placeholder - should come from global config
    
    let content = if got_page && !got_id {
        // Show changelog list (equivalent to $versioning->listChanges())
        generate_changelog_list(&db).await
    } else if got_page && got_id {
        // Show specific changelog entry (equivalent to $versioning->showChange($idInfo['GET_VALUE']))
        generate_changelog_entry(&db, changelog_id).await
    } else {
        // Show default about page
        format!(
            r#"
            Current version: {}{}
            <br/><br/>
            <a href="about.php?page=changelog">View changelog</a>
            "#,
            version, version_status
        )
    };
    
    // Generate full HTML page - matches original PHP template structure
    let html = format!(
        r#"
        <html>
        <head>
            <title>About - Hacker Experience</title>
            <!-- TODO: Include templateTop.php equivalent styles/scripts -->
        </head>
        <body>
            <!-- TODO: Include game header template -->
            {}
            </table>
        </body>
        </html>
        "#,
        content
    );
    
    Ok(Html(html))
}

/// Generate changelog list HTML
/// Equivalent to Versioning->listChanges() method
async fn generate_changelog_list(db: &PgPool) -> String {
    // TODO: Implement actual database query for changelog entries
    // For now, return placeholder content that matches the expected structure
    
    r#"
    <h3>Changelog</h3>
    <div class="changelog-list">
        <p>Changelog entries will be displayed here.</p>
        <p>TODO: Implement database query for versioning table.</p>
    </div>
    "#.to_string()
}

/// Generate specific changelog entry HTML
/// Equivalent to Versioning->showChange($id) method
async fn generate_changelog_entry(db: &PgPool, id: u32) -> String {
    // TODO: Implement actual database query for specific changelog entry
    // For now, return placeholder content
    
    format!(
        r#"
        <h3>Changelog Entry #{}</h3>
        <div class="changelog-entry">
            <p>Details for changelog entry {} will be displayed here.</p>
            <p>TODO: Implement database query for specific versioning entry.</p>
            <br/>
            <a href="about.php?page=changelog">← Back to changelog</a>
        </div>
        "#,
        id, id
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_about_query_parsing() {
        // Test changelog page parameter
        let query = AboutQuery {
            page: Some("changelog".to_string()),
            id: None,
        };
        assert_eq!(query.page.as_deref(), Some("changelog"));
        assert!(query.id.is_none());
        
        // Test with ID parameter
        let query = AboutQuery {
            page: Some("changelog".to_string()),
            id: Some("123".to_string()),
        };
        assert_eq!(query.page.as_deref(), Some("changelog"));
        assert_eq!(query.id.as_deref(), Some("123"));
    }
    
    #[test]
    fn test_changelog_content_generation() {
        // Test that functions return non-empty strings
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            // Mock database - in real implementation, this would be a proper PgPool
            let db = sqlx::PgPool::connect("postgresql://localhost/test").await.unwrap_or_else(|_| {
                // For testing, we'll skip actual DB connection
                return sqlx::PgPool::connect("postgresql://localhost/nonexistent").await.unwrap();
            });
            
            // These should not panic and should return valid HTML strings
            let list = generate_changelog_list(&db).await;
            assert!(list.contains("Changelog"));
            
            let entry = generate_changelog_entry(&db, 123).await;
            assert!(entry.contains("Changelog Entry #123"));
        });
    }
}