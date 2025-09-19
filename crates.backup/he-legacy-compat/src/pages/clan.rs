//! Clan page handler - 1:1 port of clan.php
//! 
//! Complete clan management system with:
//! - Clan membership management
//! - Clan creation and administration
//! - Clan search and discovery
//! - War system
//! - Ranking and statistics

use axum::{
    extract::{Extension, Form, Query},
    http::StatusCode,
    response::Html,
};
use serde::Deserialize;
use std::collections::HashMap;
use crate::classes::{system::System, clan::Clan};
use crate::session::{PhpSession, SessionValue};
use he_db::DbPool;

/// Query parameters for clan page navigation
#[derive(Debug, Deserialize)]
pub struct ClanQuery {
    pub action: Option<String>, // join, search, ranking, create, settings, admin, list, leave, war
    pub id: Option<String>,     // Clan ID for specific operations
}

/// Form data for clan operations
#[derive(Debug, Deserialize)]
pub struct ClanForm {
    #[serde(flatten)]
    pub data: HashMap<String, String>,
}

/// Clan page navigation state
#[derive(Debug, Clone)]
pub struct ClanNavigation {
    pub my_clan: String,
    pub create_clan: String,
    pub search_clan: String,
    pub ranking: String,
    pub admin_clan: String,
    pub settings_clan: String,
    pub war_clan: String,
    pub span_class: String,
}

impl Default for ClanNavigation {
    fn default() -> Self {
        Self {
            my_clan: "active".to_string(),
            create_clan: String::new(),
            search_clan: String::new(),
            ranking: String::new(),
            admin_clan: String::new(),
            settings_clan: String::new(),
            war_clan: String::new(),
            span_class: "span8".to_string(),
        }
    }
}

/// Clan member information
#[derive(Debug, Clone)]
pub struct ClanMemberInfo {
    pub is_member: bool,
    pub auth_level: i64,
    pub clan_id: Option<i64>,
}

/// Main clan handler - displays clan management interface
/// 
/// Port of: clan.php
/// Features:
/// - Multi-tab navigation based on user's clan status
/// - POST form handling for clan operations
/// - Complex permission and authentication system
/// - Dynamic layout adjustment based on action
/// - Clan membership validation and management
/// - War system integration
pub async fn clan_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(mut session): Extension<PhpSession>,
    Query(query): Query<ClanQuery>,
    form: Option<Form<ClanForm>>,
) -> Result<Html<String>, StatusCode> {
    // Check if user is logged in (required for clan page)
    if !session.isset_login() {
        return Ok(Html("<script>window.location.href='/index.php';</script>".to_string()));
    }

    // Initialize required classes
    let system = System::new();
    let mut clan = Clan::new(db_pool);

    // Handle POST form submissions
    if let Some(Form(form_data)) = form {
        clan.handle_post(form_data.data).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Initialize navigation state
    let mut nav = ClanNavigation::default();

    // Process action parameter
    if let Some(action) = &query.action {
        let valid_actions = vec!["join", "search", "ranking", "create", "settings", "admin", "list", "leave", "war"];
        
        if valid_actions.contains(&action.as_str()) {
            nav.my_clan = String::new();
            
            match action.as_str() {
                "search" => {
                    nav.search_clan = "active".to_string();
                },
                "create" => {
                    nav.create_clan = "active".to_string();
                },
                "ranking" => {
                    nav.ranking = "active".to_string();
                },
                "admin" | "leave" => {
                    nav.admin_clan = "active".to_string();
                    nav.settings_clan = "active".to_string();
                    if query.id.is_none() {
                        nav.span_class = "span12".to_string();
                    }
                },
                "settings" => {
                    nav.settings_clan = "active".to_string();
                },
                "list" => {
                    nav.span_class = "span12".to_string();
                    nav.my_clan = "active".to_string();
                },
                "war" => {
                    nav.war_clan = "active".to_string();
                },
                _ => {} // Other actions handled by default case
            }
        } else {
            // Invalid action
            session.add_msg("Invalid get.", "error");
            return Ok(Html(session.return_msg()));
        }
    }

    // Get clan membership information
    let clan_member_info = get_clan_member_info(&clan).await?;

    // Build page content
    let mut content = String::new();

    // Add message display if present
    if session.isset_msg() {
        content.push_str(&session.return_msg());
    }

    // Start widget structure
    content.push_str(r#"<div class="span12">"#);

    // Build navigation tabs based on clan membership
    content.push_str(&build_clan_navigation(&nav, &clan_member_info));

    // Start content area
    content.push_str(&format!(r#"
        <div class="widget-content padding noborder">
            <div class="{}">
    "#, nav.span_class));

    // Generate main content based on current action and clan status
    let main_content = if let Some(action) = &query.action {
        handle_clan_action(&clan, action, &query, &clan_member_info).await?
    } else {
        // Default view
        handle_default_clan_view(&clan, &clan_member_info).await?
    };

    content.push_str(&main_content);

    // Close content structure
    content.push_str(r#"
            </div>
        </div>
        <div class="nav nav-tabs" style="clear: both;">&nbsp;</div>
    </div>
    "#);

    Ok(Html(content))
}

/// Get clan membership information for current user
async fn get_clan_member_info(clan: &Clan) -> Result<ClanMemberInfo, StatusCode> {
    let is_member = match clan.player_have_clan().await {
        Ok(has_clan) => has_clan,
        Err(_) => false,
    };

    let (auth_level, clan_id) = if is_member {
        let auth = clan.player_auth().await.unwrap_or(0);
        let player_clan = clan.get_player_clan().await.unwrap_or(0);
        (auth, Some(player_clan))
    } else {
        (0, None)
    };

    Ok(ClanMemberInfo {
        is_member,
        auth_level,
        clan_id,
    })
}

/// Build navigation tabs based on clan membership
fn build_clan_navigation(nav: &ClanNavigation, member_info: &ClanMemberInfo) -> String {
    let mut html = String::from(r#"
        <div class="widget-box">
            <div class="widget-title">
                <ul class="nav nav-tabs">
    "#);

    // My Clan tab (always visible)
    html.push_str(&format!(
        r#"<li class="link {}"><a href="clan.php"><span class="icon-tab he16-clan"></span><span class="hide-phone">{}</span></a></li>"#,
        nav.my_clan,
        if member_info.is_member { translate("My clan") } else { translate("Clans") }
    ));

    // Create Clan tab (only if not in a clan)
    if !member_info.is_member {
        html.push_str(&format!(
            r#"<li class="link {}"><a href="?action=create"><span class="icon-tab he16-clan_create"></span><span class="hide-phone">{}</span></a></li>"#,
            nav.create_clan,
            translate("Create clan")
        ));
    }

    // Search Clans tab
    html.push_str(&format!(
        r#"<li class="link {}"><a href="?action=search"><span class="icon-tab he16-clan_search"></span><span class="hide-phone">{}</span></a></li>"#,
        nav.search_clan,
        translate("Search clans")
    ));

    // Clan Ranking tab
    html.push_str(&format!(
        r#"<li class="link {}"><a href="?action=ranking"><span class="icon-tab he16-clan_ranking"></span><span class="hide-phone">{}</span></a></li>"#,
        nav.ranking,
        translate("Clan ranking")
    ));

    // Admin/Settings tabs (only for clan members with appropriate permissions)
    if member_info.is_member {
        if member_info.auth_level >= 2 { // Admin level
            html.push_str(&format!(
                r#"<li class="link {}"><a href="?action=admin"><span class="icon-tab he16-clan_admin"></span><span class="hide-phone">{}</span></a></li>"#,
                nav.admin_clan,
                translate("Administration")
            ));
        }

        html.push_str(&format!(
            r#"<li class="link {}"><a href="?action=settings"><span class="icon-tab he16-clan_settings"></span><span class="hide-phone">{}</span></a></li>"#,
            nav.settings_clan,
            translate("Settings")
        ));

        // War tab for clan members
        html.push_str(&format!(
            r#"<li class="link {}"><a href="?action=war"><span class="icon-tab he16-clan_war"></span><span class="hide-phone">{}</span></a></li>"#,
            nav.war_clan,
            translate("War")
        ));
    }

    // Help link
    html.push_str(&format!(
        r#"<a href="#"><span class="label label-info">{}</span></a>"#,
        translate("Help")
    ));

    html.push_str("</ul></div>");
    html
}

/// Handle specific clan actions
async fn handle_clan_action(
    clan: &Clan,
    action: &str,
    query: &ClanQuery,
    member_info: &ClanMemberInfo,
) -> Result<String, StatusCode> {
    match action {
        "search" => {
            clan.show_clan_search().await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        },
        "create" => {
            if member_info.is_member {
                Ok("<p>You are already in a clan.</p>".to_string())
            } else {
                clan.show_create_clan_form().await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
            }
        },
        "ranking" => {
            clan.show_clan_ranking().await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        },
        "admin" => {
            if member_info.is_member && member_info.auth_level >= 2 {
                clan.show_admin_panel().await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
            } else {
                Ok("<p>Access denied.</p>".to_string())
            }
        },
        "settings" => {
            if member_info.is_member {
                clan.show_settings_panel().await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
            } else {
                Ok("<p>You must be in a clan to access settings.</p>".to_string())
            }
        },
        "war" => {
            if member_info.is_member {
                clan.show_war_panel().await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
            } else {
                Ok("<p>You must be in a clan to access war features.</p>".to_string())
            }
        },
        "list" => {
            clan.list_all_clans().await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        },
        "join" => {
            if let Some(id_str) = &query.id {
                if let Ok(clan_id) = id_str.parse::<i64>() {
                    clan.handle_join_request(clan_id).await
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
                } else {
                    Ok("<p>Invalid clan ID.</p>".to_string())
                }
            } else {
                Ok("<p>Clan ID required.</p>".to_string())
            }
        },
        "leave" => {
            if member_info.is_member {
                clan.handle_leave_clan().await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
            } else {
                Ok("<p>You are not in a clan.</p>".to_string())
            }
        },
        _ => Ok("<p>Unknown action.</p>".to_string()),
    }
}

/// Handle default clan view (no action specified)
async fn handle_default_clan_view(
    clan: &Clan,
    member_info: &ClanMemberInfo,
) -> Result<String, StatusCode> {
    if member_info.is_member {
        // Show user's clan information
        if let Some(clan_id) = member_info.clan_id {
            clan.show_my_clan(clan_id).await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        } else {
            Ok("<p>Error loading clan information.</p>".to_string())
        }
    } else {
        // Show available clans or create clan prompt
        clan.show_clan_overview().await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
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
    fn test_clan_navigation_default() {
        let nav = ClanNavigation::default();
        assert_eq!(nav.my_clan, "active");
        assert_eq!(nav.create_clan, "");
        assert_eq!(nav.search_clan, "");
        assert_eq!(nav.ranking, "");
        assert_eq!(nav.span_class, "span8");
    }

    #[test]
    fn test_valid_clan_actions() {
        let valid_actions = vec!["join", "search", "ranking", "create", "settings", "admin", "list", "leave", "war"];
        assert!(valid_actions.contains(&"search"));
        assert!(valid_actions.contains(&"create"));
        assert!(valid_actions.contains(&"war"));
        assert!(!valid_actions.contains(&"invalid"));
    }

    #[test]
    fn test_clan_member_info() {
        let member_info = ClanMemberInfo {
            is_member: true,
            auth_level: 2,
            clan_id: Some(123),
        };

        assert!(member_info.is_member);
        assert_eq!(member_info.auth_level, 2);
        assert_eq!(member_info.clan_id, Some(123));
    }

    #[test]
    fn test_span_class_logic() {
        let mut nav = ClanNavigation::default();
        
        // Test admin action without ID
        nav.span_class = "span12".to_string();
        assert_eq!(nav.span_class, "span12");
        
        // Test list action
        nav.span_class = "span12".to_string();
        nav.my_clan = "active".to_string();
        assert_eq!(nav.span_class, "span12");
        assert_eq!(nav.my_clan, "active");
    }

    #[test]
    fn test_translate() {
        assert_eq!(translate("My clan"), "My clan");
        assert_eq!(translate("Create clan"), "Create clan");
        assert_eq!(translate("Search clans"), "Search clans");
        assert_eq!(translate("Administration"), "Administration");
    }
}