//! Profile page handler - 1:1 port of profile.php
//! 
//! Complex profile system with multiple views:
//! - Profile view (default)
//! - Email view 
//! - Search view
//! - Friends view
//! - Edit view (for own profile)
//! Handles user profile navigation, validation, and permissions.

use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::Html,
};
use serde::Deserialize;
use crate::classes::{system::System, player::Player, social::Social};
use crate::session::{PhpSession, SessionValue};
use he_db::DbPool;

/// Query parameters for profile page navigation
#[derive(Debug, Deserialize)]
pub struct ProfileQuery {
    pub id: Option<String>,   // User ID to view
    pub view: Option<String>, // email, search, friends, edit
}

/// Profile page navigation state
#[derive(Debug, Clone)]
pub struct ProfileNavigation {
    pub menu_profile: String,
    pub menu_email: String,
    pub menu_search: String,
    pub menu_friends: String,
    pub menu_edit: String,
    pub profile_id: i64,
    pub is_own_profile: bool,
    pub valid_id: bool,
    pub active_view: String,
    pub query_string: String,
}

impl ProfileNavigation {
    fn new(user_id: i64) -> Self {
        Self {
            menu_profile: "active".to_string(),
            menu_email: String::new(),
            menu_search: String::new(),
            menu_friends: String::new(),
            menu_edit: String::new(),
            profile_id: user_id,
            is_own_profile: true,
            valid_id: true,
            active_view: "profile".to_string(),
            query_string: String::new(),
        }
    }
}

/// Main profile handler - displays user profiles with navigation
/// 
/// Port of: profile.php
/// Features:
/// - Multi-view navigation (Profile, Email, Search, Friends, Edit)
/// - User ID validation and existence checking
/// - Permission handling for different profile views
/// - Session state management for profile viewing
/// - Complex GET parameter processing
/// - Error handling for invalid users and permissions
pub async fn profile_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(mut session): Extension<PhpSession>,
    Query(query): Query<ProfileQuery>,
) -> Result<Html<String>, StatusCode> {
    // Check if user is logged in (required for profile page)
    if !session.isset_login() {
        return Ok(Html("<script>window.location.href='/index.php';</script>".to_string()));
    }

    // Get current user ID from session
    let current_user_id = match session.get("id") {
        Some(SessionValue::Integer(id)) => *id,
        Some(SessionValue::String(id_str)) => {
            id_str.parse::<i64>().unwrap_or(0)
        },
        _ => {
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Initialize required classes
    let system = System::new();
    let player = Player::new(db_pool.clone());
    let _social = Social::new(db_pool.clone());

    // Initialize navigation state
    let mut nav = ProfileNavigation::new(current_user_id);
    let mut error = false;

    // Initialize VALID_ID session if not set
    if !session.isset("VALID_ID") {
        session.set("VALID_ID", SessionValue::String(String::new()));
    }

    // Process GET parameters
    if let Some(id_str) = &query.id {
        // Viewing another user's profile
        if let Ok(target_user_id) = id_str.parse::<i64>() {
            // Verify the user ID exists
            match player.verify_id(target_user_id).await {
                Ok(true) => {
                    // Valid user - set up for viewing their profile
                    session.set("PROFILE_ID", SessionValue::Integer(target_user_id));
                    nav.profile_id = target_user_id;
                    nav.is_own_profile = target_user_id == current_user_id;
                    nav.valid_id = true;
                    nav.query_string = format!("?id={}", target_user_id);

                    // Handle view parameter for other user's profile
                    if let Some(view) = &query.view {
                        let valid_views = if nav.is_own_profile {
                            vec!["email", "search", "friends", "edit"]
                        } else {
                            vec!["email", "search", "friends"] // No edit for other users
                        };

                        if valid_views.contains(&view.as_str()) {
                            nav.menu_profile = String::new();
                            
                            match view.as_str() {
                                "email" => {
                                    nav.active_view = "email".to_string();
                                    nav.menu_email = "active".to_string();
                                },
                                "search" => {
                                    nav.active_view = "search".to_string();
                                    nav.menu_search = "active".to_string();
                                },
                                "friends" => {
                                    nav.active_view = "friends".to_string();
                                    nav.menu_friends = "active".to_string();
                                },
                                "edit" if nav.is_own_profile => {
                                    nav.active_view = "edit".to_string();
                                    nav.menu_edit = "active".to_string();
                                },
                                _ => {
                                    let error_content = system.handle_error("Invalid Get", &format!("profile?id={}", target_user_id));
                                    return Ok(Html(error_content));
                                }
                            }
                        } else {
                            let error_content = system.handle_error("Invalid Get", &format!("profile?id={}", target_user_id));
                            return Ok(Html(error_content));
                        }
                    }
                },
                _ => {
                    // Invalid user ID
                    error = true;
                    nav.valid_id = false;
                }
            }
        } else {
            // Invalid ID format
            error = true;
            let error_content = system.handle_error("INVALID_ID", "profile.php");
            return Ok(Html(error_content));
        }
    } else {
        // Viewing own profile (no ID parameter)
        session.set("PROFILE_ID", SessionValue::Integer(current_user_id));
        session.set("VALID_ID", SessionValue::String("1".to_string()));
        nav.profile_id = current_user_id;
        nav.is_own_profile = true;

        // Handle view parameter for own profile
        if let Some(view) = &query.view {
            let valid_views = vec!["edit", "search", "friends"];
            
            if valid_views.contains(&view.as_str()) {
                nav.menu_profile = String::new();
                
                match view.as_str() {
                    "edit" => {
                        nav.active_view = "edit".to_string();
                        nav.menu_edit = "active".to_string();
                    },
                    "search" => {
                        nav.active_view = "search".to_string();
                        nav.menu_search = "active".to_string();
                    },
                    "friends" => {
                        nav.active_view = "friends".to_string();
                        nav.menu_friends = "active".to_string();
                    },
                    _ => {
                        let error_content = system.handle_error("Invalid Get", "profile.php");
                        return Ok(Html(error_content));
                    }
                }
            } else {
                let error_content = system.handle_error("Invalid Get", "profile.php");
                return Ok(Html(error_content));
            }
        }
    }

    // Handle error case
    if error && !nav.valid_id {
        let error_content = system.handle_error("User not found", "profile.php");
        return Ok(Html(error_content));
    }

    // Build page content
    let mut content = String::new();

    // Add message display if present
    if session.isset_msg() {
        content.push_str(&session.return_msg());
    }

    // Get profile user information
    let profile_user = match player.get_user_info(nav.profile_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let error_content = system.handle_error("User not found", "profile.php");
            return Ok(Html(error_content));
        },
        Err(_) => {
            let error_content = system.handle_error("Database error", "profile.php");
            return Ok(Html(error_content));
        }
    };

    // Start widget structure with navigation tabs
    content.push_str(&format!(r#"
    <div class="span12">
        <div class="widget-box">
            <div class="widget-title">
                <ul class="nav nav-tabs">
                    <li class="link {}"><a href="profile{}">{}</a></li>
    "#, 
        nav.menu_profile,
        nav.query_string,
        if nav.is_own_profile { translate("My Profile") } else { &profile_user.login }
    ));

    // Add email tab if available (for other users or own profile)
    if !nav.is_own_profile || !nav.menu_email.is_empty() {
        content.push_str(&format!(r#"
                    <li class="link {}"><a href="{}{}view=email">{}</a></li>
        "#, 
            nav.menu_email,
            if nav.query_string.is_empty() { "?" } else { &format!("{}&", nav.query_string) },
            translate("Email")
        ));
    }

    // Add search tab
    content.push_str(&format!(r#"
                    <li class="link {}"><a href="{}{}view=search">{}</a></li>
    "#, 
        nav.menu_search,
        if nav.query_string.is_empty() { "?" } else { &format!("{}&", nav.query_string) },
        translate("Search")
    ));

    // Add friends tab
    content.push_str(&format!(r#"
                    <li class="link {}"><a href="{}{}view=friends">{}</a></li>
    "#, 
        nav.menu_friends,
        if nav.query_string.is_empty() { "?" } else { &format!("{}&", nav.query_string) },
        translate("Friends")
    ));

    // Add edit tab (only for own profile)
    if nav.is_own_profile {
        content.push_str(&format!(r#"
                    <li class="link {}"><a href="?view=edit">{}</a></li>
        "#, 
            nav.menu_edit,
            translate("Edit Profile")
        ));
    }

    // Close navigation and start content area
    content.push_str(&format!(r#"
                    <a href="#"><span class="label label-info">{}</span></a>
                </ul>
            </div>
            <div class="widget-content padding noborder">
    "#, translate("Help")));

    // Generate content based on active view
    match nav.active_view.as_str() {
        "profile" => {
            let profile_content = player.show_profile(nav.profile_id).await
                .unwrap_or_else(|_| "Error loading profile".to_string());
            content.push_str(&profile_content);
        },
        "email" => {
            let email_content = player.show_email_view(nav.profile_id).await
                .unwrap_or_else(|_| "Error loading email view".to_string());
            content.push_str(&email_content);
        },
        "search" => {
            let search_content = player.show_search_view().await
                .unwrap_or_else(|_| "Error loading search view".to_string());
            content.push_str(&search_content);
        },
        "friends" => {
            let friends_content = player.show_friends_view(nav.profile_id).await
                .unwrap_or_else(|_| "Error loading friends view".to_string());
            content.push_str(&friends_content);
        },
        "edit" if nav.is_own_profile => {
            let edit_content = player.show_edit_profile(nav.profile_id).await
                .unwrap_or_else(|_| "Error loading edit view".to_string());
            content.push_str(&edit_content);
        },
        _ => {
            content.push_str("<p>Invalid view</p>");
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
    fn test_profile_navigation_new() {
        let nav = ProfileNavigation::new(123);
        assert_eq!(nav.menu_profile, "active");
        assert_eq!(nav.profile_id, 123);
        assert!(nav.is_own_profile);
        assert!(nav.valid_id);
        assert_eq!(nav.active_view, "profile");
    }

    #[test]
    fn test_profile_id_parsing() {
        assert_eq!("123".parse::<i64>().unwrap(), 123);
        assert!("invalid".parse::<i64>().is_err());
    }

    #[test]
    fn test_translate() {
        assert_eq!(translate("My Profile"), "My Profile");
        assert_eq!(translate("Email"), "Email");
        assert_eq!(translate("Search"), "Search");
        assert_eq!(translate("Friends"), "Friends");
        assert_eq!(translate("Edit Profile"), "Edit Profile");
    }
}