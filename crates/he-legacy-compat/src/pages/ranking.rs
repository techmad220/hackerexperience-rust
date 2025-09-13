//! Ranking page handler - 1:1 port of ranking.php
//! 
//! Player ranking and leaderboard system with multiple categories:
//! - User ranking (default)
//! - Clan ranking  
//! - Software ranking
//! - DDoS ranking

use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::Html,
};
use serde::Deserialize;
use crate::classes::{system::System, ranking::Ranking};
use crate::session::PhpSession;
use he_db::DbPool;

/// Query parameters for ranking page navigation
#[derive(Debug, Deserialize)]
pub struct RankingQuery {
    pub show: Option<String>, // clan, software, ddos
}

/// Ranking page navigation state
#[derive(Debug, Clone)]
pub struct RankingNavigation {
    pub user: String,
    pub clan: String,
    pub soft: String,
    pub ddos: String,
    pub display_type: String,
}

impl Default for RankingNavigation {
    fn default() -> Self {
        Self {
            user: " active".to_string(),
            clan: String::new(),
            soft: String::new(),
            ddos: String::new(),
            display_type: "user".to_string(),
        }
    }
}

/// Main ranking handler - displays various ranking categories
/// 
/// Port of: ranking.php
/// Features:
/// - Multi-tab navigation (User, Clan, Software, DDoS)
/// - Dynamic ranking display based on category
/// - GET parameter validation and error handling
/// - Session message display
/// - Ranking class integration for data display
pub async fn ranking_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(mut session): Extension<PhpSession>,
    Query(query): Query<RankingQuery>,
) -> Result<Html<String>, StatusCode> {
    // Check if user is logged in (required for ranking page)
    if !session.isset_login() {
        return Ok(Html("<script>window.location.href='/index.php';</script>".to_string()));
    }

    // Initialize required classes
    let system = System::new();
    let ranking = Ranking::new(db_pool);

    // Initialize navigation state
    let mut nav = RankingNavigation::default();

    // Process GET parameters for show selection
    if let Some(show) = &query.show {
        let valid_shows = vec!["clan", "software", "ddos"];
        
        if valid_shows.contains(&show.as_str()) {
            nav.user = String::new();
            
            match show.as_str() {
                "clan" => {
                    nav.clan = " active".to_string();
                    nav.display_type = "clan".to_string();
                },
                "software" => {
                    nav.soft = " active".to_string();
                    nav.display_type = "software".to_string();
                },
                "ddos" => {
                    nav.ddos = " active".to_string();
                    nav.display_type = "ddos".to_string();
                },
                _ => {} // Should not reach here due to validation above
            }
        } else {
            // Invalid GET parameter
            session.add_msg("Invalid get.", "error");
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
                    <li class="link{}"><a href="ranking.php"><span class="icon-tab he16-rank_user"></span><span class="hide-phone">{}</span></a></li>
                    <li class="link{}"><a href="?show=clan"><span class="icon-tab he16-rank_clan"></span><span class="hide-phone">{}</span></a></li>
                    <li class="link{}"><a href="?show=software"><span class="icon-tab he16-rank_software"></span><span class="hide-phone">{}</span></a></li>
                    <li class="link{}"><a href="?show=ddos"><span class="icon-tab he16-rank_ddos"></span><span class="hide-phone">{}</span></a></li>
                </ul>
            </div>
            <div class="widget-content padding noborder center">
    "#, 
        nav.user,
        translate("User ranking"),
        nav.clan,
        translate("Clan ranking"),
        nav.soft,
        translate("Software ranking"),
        nav.ddos,
        translate("DDoS Ranking")
    ));

    // Display ranking content based on selected category
    match ranking.ranking_display(&nav.display_type).await {
        Ok(ranking_html) => {
            content.push_str(&ranking_html);
        },
        Err(e) => {
            eprintln!("Error displaying ranking for {}: {:?}", nav.display_type, e);
            content.push_str(&format!(
                "<p class='error'>Error loading {} ranking.</p>",
                nav.display_type
            ));
        }
    }

    // Close widget structure
    content.push_str(r#"
            </div>
            <div style="clear: both;" class="nav nav-tabs"></div>
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
    fn test_ranking_navigation_default() {
        let nav = RankingNavigation::default();
        assert_eq!(nav.user, " active");
        assert_eq!(nav.clan, "");
        assert_eq!(nav.soft, "");
        assert_eq!(nav.ddos, "");
        assert_eq!(nav.display_type, "user");
    }

    #[test]
    fn test_valid_show_parameters() {
        let valid_shows = vec!["clan", "software", "ddos"];
        assert!(valid_shows.contains(&"clan"));
        assert!(valid_shows.contains(&"software"));
        assert!(valid_shows.contains(&"ddos"));
        assert!(!valid_shows.contains(&"invalid"));
    }

    #[test]
    fn test_ranking_navigation_clan_active() {
        let mut nav = RankingNavigation::default();
        nav.user = String::new();
        nav.clan = " active".to_string();
        nav.display_type = "clan".to_string();
        
        assert_eq!(nav.user, "");
        assert_eq!(nav.clan, " active");
        assert_eq!(nav.display_type, "clan");
    }

    #[test]
    fn test_ranking_navigation_software_active() {
        let mut nav = RankingNavigation::default();
        nav.user = String::new();
        nav.soft = " active".to_string();
        nav.display_type = "software".to_string();
        
        assert_eq!(nav.user, "");
        assert_eq!(nav.soft, " active");
        assert_eq!(nav.display_type, "software");
    }

    #[test]
    fn test_ranking_navigation_ddos_active() {
        let mut nav = RankingNavigation::default();
        nav.user = String::new();
        nav.ddos = " active".to_string();
        nav.display_type = "ddos".to_string();
        
        assert_eq!(nav.user, "");
        assert_eq!(nav.ddos, " active");
        assert_eq!(nav.display_type, "ddos");
    }

    #[test]
    fn test_translate() {
        assert_eq!(translate("User ranking"), "User ranking");
        assert_eq!(translate("Clan ranking"), "Clan ranking");
        assert_eq!(translate("Software ranking"), "Software ranking");
        assert_eq!(translate("DDoS Ranking"), "DDoS Ranking");
    }
}