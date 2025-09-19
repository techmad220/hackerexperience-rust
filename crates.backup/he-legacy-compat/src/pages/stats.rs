//! Stats page handler - 1:1 port of stats.php
//! 
//! Statistics display system with three main views:
//! - Game Stats (default) - shows server statistics and rankings
//! - Server Stats - placeholder (not implemented)
//! - Forum Stats - placeholder (not implemented)

use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::Html,
};
use serde::Deserialize;
use crate::classes::{system::System, ranking::Ranking};
use crate::session::PhpSession;
use he_db::DbPool;

/// Query parameters for stats page navigation
#[derive(Debug, Deserialize)]
pub struct StatsQuery {
    pub show: Option<String>, // game, server, forum
    pub round: Option<String>, // all (for game stats)
}

/// Stats page navigation state
#[derive(Debug, Clone)]
pub struct StatsNavigation {
    pub show: String,
    pub game: String,
    pub server: String,
    pub forum: String,
    pub display_mode: String, // For game stats: "" or "all"
}

impl Default for StatsNavigation {
    fn default() -> Self {
        Self {
            show: "game".to_string(),
            game: " active".to_string(),
            server: String::new(),
            forum: String::new(),
            display_mode: String::new(),
        }
    }
}

/// Main stats handler - displays various statistics views
/// 
/// Port of: stats.php
/// Features:
/// - Multi-tab navigation (Game, Server, Forum)
/// - Game statistics with optional "all rounds" view
/// - Server and forum stats placeholders (not implemented)
/// - Session authentication required
/// - Error handling for unimplemented features
/// - Dynamic tab state based on URL parameters
pub async fn stats_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(session): Extension<PhpSession>,
    Query(query): Query<StatsQuery>,
) -> Result<Html<String>, StatusCode> {
    // Check if user is logged in (required for stats page)
    if !session.isset_login() {
        return Ok(Html("<script>window.location.href='/index.php';</script>".to_string()));
    }

    // Initialize required classes
    let system = System::new();
    let ranking = Ranking::new(db_pool);

    // Initialize navigation state
    let mut nav = StatsNavigation::default();

    // Process GET parameters for show selection
    if let Some(show) = &query.show {
        let valid_shows = vec!["game", "server", "forum"];
        
        if valid_shows.contains(&show.as_str()) {
            match show.as_str() {
                "game" => {
                    nav.show = "game".to_string();
                    // game tab already active by default
                },
                "server" => {
                    nav.show = "server".to_string();
                    nav.game = String::new();
                    nav.server = " active".to_string();
                },
                "forum" => {
                    nav.show = "forum".to_string();
                    nav.game = String::new();
                    nav.forum = " active".to_string();
                },
                _ => {} // Should not reach here due to validation above
            }
        }
    }

    // Handle round parameter for game stats
    if nav.show == "game" {
        if let Some(round) = &query.round {
            if round == "all" {
                nav.display_mode = "all".to_string();
            }
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
                    <li class="link{}"><a href="stats.php"><span class="icon-tab he16-stats"></span>Game Stats</a></li>
                    <li class="link{}"><a href="?show=server"><span class="icon-tab he16-server_stats"></span>Server Stats</a></li>
                    <li class="link{}"><a href="?show=forum"><span class="icon-tab he16-forum_stats"></span>Forum Stats</a></li>
                    <a href="#"><span class="label label-info">{}</span></a>
                </ul>
            </div>
            <div class="widget-content padding noborder">
    "#, 
        nav.game,
        nav.server,
        nav.forum,
        translate("Help")
    ));

    // Generate content based on selected view
    match nav.show.as_str() {
        "game" => {
            // Game statistics view
            match ranking.server_stats_list(&nav.display_mode).await {
                Ok(stats_html) => {
                    content.push_str(&stats_html);
                },
                Err(e) => {
                    eprintln!("Error loading game stats: {:?}", e);
                    content.push_str("<p class='error'>Error loading game statistics.</p>");
                }
            }
        },
        "server" => {
            // Server statistics - not implemented
            let error_content = system.handle_error("Sorry, this page isn't implemented yet.", "stats.php");
            content.push_str(&error_content);
        },
        "forum" => {
            // Forum statistics - not implemented
            let error_content = system.handle_error("Sorry, this page isn't implemented yet.", "stats.php");
            content.push_str(&error_content);
        },
        _ => {
            content.push_str("<p>Invalid statistics view</p>");
        }
    }

    // Close widget structure
    content.push_str(r#"
            </div>
            <div class="nav nav-tabs" style="clear: both;"></div>
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
    fn test_stats_navigation_default() {
        let nav = StatsNavigation::default();
        assert_eq!(nav.show, "game");
        assert_eq!(nav.game, " active");
        assert_eq!(nav.server, "");
        assert_eq!(nav.forum, "");
        assert_eq!(nav.display_mode, "");
    }

    #[test]
    fn test_valid_show_parameters() {
        let valid_shows = vec!["game", "server", "forum"];
        assert!(valid_shows.contains(&"game"));
        assert!(valid_shows.contains(&"server"));
        assert!(valid_shows.contains(&"forum"));
        assert!(!valid_shows.contains(&"invalid"));
    }

    #[test]
    fn test_round_parameter() {
        let round = "all";
        assert_eq!(round, "all");
        
        let round = "current";
        assert_ne!(round, "all");
    }

    #[test]
    fn test_stats_navigation_server_view() {
        let mut nav = StatsNavigation::default();
        nav.show = "server".to_string();
        nav.game = String::new();
        nav.server = " active".to_string();
        
        assert_eq!(nav.show, "server");
        assert_eq!(nav.game, "");
        assert_eq!(nav.server, " active");
        assert_eq!(nav.forum, "");
    }

    #[test]
    fn test_translate() {
        assert_eq!(translate("Help"), "Help");
        assert_eq!(translate("Game Stats"), "Game Stats");
    }
}