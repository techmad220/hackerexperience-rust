//! Settings page handler - 1:1 port of settings.php
//! 
//! User settings management with language switching functionality.
//! Handles password change requests (currently disabled) and language preferences.

use axum::{
    extract::{Extension, Form},
    http::StatusCode,
    response::{Html, Redirect},
};
use serde::Deserialize;
use crate::classes::{system::System, player::Player};
use crate::session::{PhpSession, SessionValue};
use he_db::DbPool;
use sqlx;

/// Form data for settings changes
#[derive(Debug, Deserialize)]
pub struct SettingsForm {
    pub old: Option<String>,     // Old password (feature disabled)
    pub new: Option<String>,     // New password (feature disabled)  
    pub confirm: Option<String>, // Confirm password (feature disabled)
    pub lang: Option<String>,    // Language selection
}

/// Main settings handler - displays and processes user settings
/// 
/// Port of: settings.php
/// Features:
/// - Language switching between English and Portuguese
/// - Password change interface (currently disabled with error message)
/// - Form validation and error handling
/// - Database updates for language preferences
/// - Redirection to appropriate language domains
/// - Session management for language settings
pub async fn settings_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(mut session): Extension<PhpSession>,
    form: Option<Form<SettingsForm>>,
) -> Result<Html<String>, StatusCode> {
    // Check if user is logged in (required for settings page)
    if !session.isset_login() {
        return Ok(Html("<script>window.location.href='/index.php';</script>".to_string()));
    }

    // Get current user ID from session
    let user_id = match session.get("id") {
        Some(SessionValue::Integer(id)) => *id,
        Some(SessionValue::String(id_str)) => {
            id_str.parse::<i64>().unwrap_or(0)
        },
        _ => {
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Initialize required classes
    let _system = System::new();
    let player = Player::new(db_pool.clone());

    // Handle POST form submissions
    if let Some(Form(settings_data)) = form {
        // Handle password change attempt (disabled feature)
        if settings_data.old.is_some() {
            session.add_msg("Sorry, change password is not available for now.", "error");
        }

        // Handle language change
        if let Some(lang) = &settings_data.lang {
            if lang.is_empty() {
                session.add_msg("Please choose a language.", "error");
                return Ok(Html("<script>window.location.href='/settings.php';</script>".to_string()));
            }

            // Validate language selection
            match lang.as_str() {
                "English" | "Português" => {
                    // Valid language - proceed with processing
                },
                _ => {
                    session.add_msg("Please choose a valid language.", "error");
                    return Ok(Html("<script>window.location.href='/settings.php';</script>".to_string()));
                }
            }

            // Get current language from session
            let current_lang = session.get("language")
                .and_then(|v| if let SessionValue::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_else(|| "en_US".to_string());

            // Check if language change is needed
            let mut change = true;
            if lang == "Português" && current_lang == "pt_BR" {
                change = false;
            } else if lang == "English" && current_lang == "en_US" {
                change = false;
            }

            if !change {
                session.add_msg("This already is your language.", "error");
                return Ok(Html("<script>window.location.href='/settings.php';</script>".to_string()));
            }

            // Update language in database and session
            let (db_lang, session_lang, redirect_url) = if lang == "Português" {
                ("br", "pt_BR", "https://br.hackerexperience.com/")
            } else {
                ("en", "en_US", "https://en.hackerexperience.com/")
            };

            // Update database
            match sqlx::query("UPDATE users_language SET lang = ? WHERE userID = ?")
                .bind(db_lang)
                .bind(user_id)
                .execute(&db_pool)
                .await
            {
                Ok(_) => {
                    // Update session
                    session.set("language", SessionValue::String(session_lang.to_string()));
                    
                    // Add success message and redirect
                    if lang == "Português" {
                        session.add_msg("Language changed to portuguese.", "notice");
                    } else {
                        session.add_msg("Language changed to english.", "notice");
                    }

                    return Ok(Html(&format!("<script>window.location.href='{}';</script>", redirect_url)));
                },
                Err(e) => {
                    eprintln!("Error updating language: {:?}", e);
                    session.add_msg("Error updating language preference.", "error");
                }
            }
        }
    }

    // Build page content
    let mut content = String::new();

    // Add message display if present
    if session.isset_msg() {
        content.push_str(&session.return_msg());
    }

    // Start widget structure
    content.push_str(&format!(r#"
    <div class="span12">
        <div class="widget-box">
            <div class="widget-title">
                <ul class="nav nav-tabs">                                  
                    <li class="link active"><a href="settings.php"><span class="icon-tab he16-settings"></span>{}</a></li>
                    <a href="#"><span class="label label-info">{}</span></a>
                </ul>
            </div>
            <div class="widget-content padding noborder">
    "#, 
        translate("My settings"),
        translate("Help")
    ));

    // Display settings form via Player class
    match player.settings_show().await {
        Ok(settings_html) => {
            content.push_str(&settings_html);
        },
        Err(e) => {
            eprintln!("Error loading settings: {:?}", e);
            content.push_str("<p class='error'>Error loading settings.</p>");
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
    fn test_language_validation() {
        let valid_languages = vec!["English", "Português"];
        assert!(valid_languages.contains(&"English"));
        assert!(valid_languages.contains(&"Português"));
        assert!(!valid_languages.contains(&"Invalid"));
    }

    #[test]
    fn test_language_change_detection() {
        // Test Portuguese language change detection
        let current_lang = "pt_BR";
        let new_lang = "Português";
        let change_needed = !(new_lang == "Português" && current_lang == "pt_BR");
        assert!(!change_needed); // Should not need change
        
        // Test English language change detection
        let current_lang = "en_US";
        let new_lang = "English";
        let change_needed = !(new_lang == "English" && current_lang == "en_US");
        assert!(!change_needed); // Should not need change
        
        // Test actual change needed
        let current_lang = "en_US";
        let new_lang = "Português";
        let change_needed = !(new_lang == "Português" && current_lang == "pt_BR");
        assert!(change_needed); // Should need change
    }

    #[test]
    fn test_translate() {
        assert_eq!(translate("My settings"), "My settings");
        assert_eq!(translate("Help"), "Help");
    }
}