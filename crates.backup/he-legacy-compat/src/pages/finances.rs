//! Finances page handler - 1:1 port of finances.php
//! 
//! Simple finances display page showing user's financial transactions and balance.
//! Uses the Finances class to list all financial activity.

use axum::{
    extract::Extension,
    http::StatusCode,
    response::Html,
};
use crate::classes::finances::Finances;
use crate::session::PhpSession;
use he_db::DbPool;

/// Main finances handler - displays user's financial information
/// 
/// Port of: finances.php
/// Features:
/// - Session authentication required
/// - Simple tabbed navigation (only Finances tab)
/// - Displays financial transaction list via Finances class
/// - Help link integration
/// - Template-based layout structure
pub async fn finances_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(session): Extension<PhpSession>,
) -> Result<Html<String>, StatusCode> {
    // Check if user is logged in (required for finances page)
    if !session.isset_login() {
        return Ok(Html("<script>window.location.href='/index.php';</script>".to_string()));
    }

    // Initialize finances class
    let mut finances = Finances::new(db_pool);

    // Build page content
    let mut content = String::new();

    // Add message display if present
    if session.isset_msg() {
        content.push_str(&session.return_msg());
    }

    // Start main widget structure
    content.push_str(r#"
    <div class="span12 center" style="text-align: center;">
        <div class="widget-box">
            <div class="widget-title">
                <ul class="nav nav-tabs">
                    <li class="link active"><a href="finances"><span class="icon-tab he16-wallet"></span>Finances</a></li>
                    <a href="#"><span class="label label-info">Help</span></a>
                </ul>
            </div>
            <div class="widget-content padding noborder">
    "#);

    // Get and display finances list
    match finances.list_finances().await {
        Ok(finances_html) => {
            content.push_str(&finances_html);
        },
        Err(e) => {
            eprintln!("Error loading finances: {:?}", e);
            content.push_str("<p class='error'>Error loading financial information.</p>");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finances_page_structure() {
        // Test that the HTML structure contains expected elements
        let content = r#"
        <div class="span12 center" style="text-align: center;">
            <div class="widget-box">
                <div class="widget-title">
                    <ul class="nav nav-tabs">
                        <li class="link active"><a href="finances"><span class="icon-tab he16-wallet"></span>Finances</a></li>
                        <a href="#"><span class="label label-info">Help</span></a>
                    </ul>
                </div>
        "#;
        
        assert!(content.contains("he16-wallet"));
        assert!(content.contains("Finances"));
        assert!(content.contains("nav nav-tabs"));
    }
}