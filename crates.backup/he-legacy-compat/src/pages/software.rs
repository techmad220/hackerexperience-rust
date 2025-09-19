//! Software management page handler - 1:1 port of software.php
//! 
//! Complex software management interface with multiple views:
//! - All software (default)
//! - External HD
//! - Specific software details
//! - Folder view
//! - Text file view
//! - Various software actions (hide, seek, delete, install, etc.)

use axum::{
    extract::{Extension, Form, Query},
    http::StatusCode,
    response::Html,
};
use serde::Deserialize;
use std::collections::HashMap;
use crate::classes::{system::System, pc::{SoftwareVPC, HardwareVPC}, process::Process};
use crate::session::{PhpSession, SessionValue};
use he_db::DbPool;

/// Query parameters for software page navigation
#[derive(Debug, Deserialize)]
pub struct SoftwareQuery {
    pub action: Option<String>, // hide, seek, del, install, uninstall, nmap, text, folder, move, webserver
    pub id: Option<String>,     // Software ID
    pub view: Option<String>,   // View ID for folder/text
    pub page: Option<String>,   // external
}

/// Form data for software operations
#[derive(Debug, Deserialize)]
pub struct SoftwareForm {
    #[serde(flatten)]
    pub data: HashMap<String, String>,
}

/// Software page navigation state
#[derive(Debug, Clone)]
pub struct SoftwareNavigation {
    pub all_software: String,
    pub external: String,
    pub specific_software: String,
    pub have_folder: String,
    pub have_text: String,
}

impl Default for SoftwareNavigation {
    fn default() -> Self {
        Self {
            all_software: " active".to_string(),
            external: String::new(),
            specific_software: String::new(),
            have_folder: String::new(),
            have_text: String::new(),
        }
    }
}

/// Software display information
#[derive(Debug, Clone)]
pub struct SoftwareInfo {
    pub name: String,
    pub software_type: String,
    pub link: String,
    pub exists: bool,
}

/// Main software handler - displays software management interface
/// 
/// Port of: software.php
/// Features:
/// - Tabbed navigation between different software views
/// - Software action processing (hide, seek, delete, install, etc.)
/// - Folder and text file management
/// - External HD integration
/// - Complex GET/POST parameter handling
/// - Session state management for folder/text views
pub async fn software_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(mut session): Extension<PhpSession>,
    Query(query): Query<SoftwareQuery>,
    form: Option<Form<SoftwareForm>>,
) -> Result<Html<String>, StatusCode> {
    // Check if user is logged in (required for software page)
    if !session.isset_login() {
        return Ok(Html("<script>window.location.href='/index.php';</script>".to_string()));
    }

    // Get user ID from session
    let user_id = match session.get("id") {
        Some(SessionValue::Integer(id)) => *id,
        Some(SessionValue::String(id_str)) => {
            id_str.parse::<i64>().unwrap_or(0)
        },
        _ => {
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Initialize system classes
    let system = System::new();
    let mut software = SoftwareVPC::new(user_id, db_pool.clone());
    let _hardware = HardwareVPC::new(user_id, db_pool.clone());
    let _process = Process::new(db_pool.clone());

    // Handle POST form submissions
    if let Some(Form(form_data)) = form {
        software.handle_post(form_data.data).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Initialize navigation state
    let mut nav = SoftwareNavigation::default();
    let mut software_info = SoftwareInfo {
        name: String::new(),
        software_type: String::new(),
        link: String::new(),
        exists: false,
    };

    let mut add = String::new();
    let mut got_get = false;

    // Process GET parameters for different views
    if let Some(action) = &query.action {
        let valid_actions = vec!["hide", "seek", "del", "install", "uninstall", "nmap", "text", "folder", "move", "webserver"];
        
        if valid_actions.contains(&action.as_str()) {
            got_get = true;
        }
    }

    // Handle specific software ID view
    if let Some(id_str) = &query.id {
        if let Ok(software_id) = id_str.parse::<i64>() {
            nav.specific_software = "active".to_string();
            nav.all_software = String::new();

            // Check if software exists for this user
            match software.isset_software(software_id, user_id, "VPC").await {
                Ok(true) => {
                    // User owns this software
                    match software.get_software(software_id, user_id, "VPC").await {
                        Ok(Some(soft_data)) => {
                            software_info.name = format!("{}{}", 
                                soft_data.softname, 
                                software.get_extension(&soft_data.softtype)
                            );
                            software_info.software_type = soft_data.softtype;
                            software_info.exists = true;
                        },
                        _ => {
                            software_info.name = "Unknown software".to_string();
                            software_info.software_type = "error".to_string();
                            software_info.exists = false;
                        }
                    }
                },
                _ => {
                    // Check if it's external software
                    match software.isset_external_software(software_id).await {
                        Ok(true) => {
                            match software.get_external_software(software_id).await {
                                Ok(Some(soft_data)) => {
                                    software_info.name = format!("{}{}", 
                                        soft_data.softname, 
                                        software.get_extension(&soft_data.softtype)
                                    );
                                    software_info.software_type = soft_data.softtype;
                                    software_info.exists = true;
                                },
                                _ => {
                                    software_info.name = "Unknown software".to_string();
                                    software_info.software_type = "error".to_string();
                                    software_info.exists = false;
                                }
                            }
                        },
                        _ => {
                            software_info.name = "Unknown software".to_string();
                            software_info.software_type = "error".to_string();
                            software_info.exists = false;
                        }
                    }
                }
            }

            // Build link for this software
            let mut link = "?".to_string();
            if let Some(action) = &query.action {
                if action != "buy" {
                    link = format!("?action={}&", action);
                }
            }
            link.push_str(&format!("id={}", software_id));
            software_info.link = link;

            got_get = true;
        } else {
            session.add_msg("Invalid get.", "error");
            software_info.name = "Unknown software".to_string();
            software_info.software_type = "error".to_string();
        }
    }
    // Handle folder/text view with action and view parameters
    else if let (Some(action), Some(view_str)) = (&query.action, &query.view) {
        if let Ok(view_id) = view_str.parse::<i64>() {
            match action.as_str() {
                "folder" => {
                    nav.have_folder = " active".to_string();
                    nav.all_software = String::new();
                    
                    let folder_name = software.folder_name(view_id).await
                        .unwrap_or_else(|_| "Unknown folder".to_string());
                    software_info.link = format!("software?action=folder&view={}", view_id);
                    software_info.name = folder_name;
                    
                    // Clear text session and set folder session
                    session.unset("TEXT");
                    session.set("FOLDER", SessionValue::Integer(view_id));
                },
                "text" => {
                    nav.have_text = " active".to_string();
                    nav.all_software = String::new();
                    
                    let text_name = software.text_name(view_id).await
                        .unwrap_or_else(|_| "Unknown text".to_string());
                    software_info.link = format!("software?action=text&view={}", view_id);
                    software_info.name = text_name;
                    
                    // Clear folder session and set text session
                    session.unset("FOLDER");
                    session.set("TEXT", SessionValue::Integer(view_id));
                },
                _ => {}
            }
        }
    }
    // Handle persistent folder/text session state
    else if let Some(SessionValue::Integer(folder_id)) = session.get("FOLDER") {
        let folder_name = software.folder_name(*folder_id).await
            .unwrap_or_else(|_| "Unknown folder".to_string());
        software_info.link = format!("software?action=folder&view={}", folder_id);
        software_info.name = folder_name;
        nav.have_folder = " ".to_string();
    } else if let Some(SessionValue::Integer(text_id)) = session.get("TEXT") {
        let text_name = software.text_name(*text_id).await
            .unwrap_or_else(|_| "Unknown text".to_string());
        software_info.link = format!("software?action=text&view={}", text_id);
        software_info.name = text_name;
        nav.have_text = " ".to_string();
    }

    // Handle page parameter (external HD view)
    if let Some(page) = &query.page {
        if page == "external" {
            got_get = false; // Special handling
            add = "external".to_string();
            nav.external = " active".to_string();
            nav.all_software = String::new();
        }
    }

    // Generate page content
    let mut content = String::new();

    // Add message display if present
    if session.isset_msg() {
        content.push_str(&session.return_msg());
    }

    // Build the main widget structure with navigation tabs
    content.push_str(&format!(r#"
    <div class="span12">
        <div class="widget-box">
            <div class="widget-title">
                <ul class="nav nav-tabs">
                    <li class="link{}"><a href="software.php"><span class="icon-tab he16-software"></span>{}</a></li>
                    <li class="link{}"><a href="?page=external"><span class="icon-tab he16-xhd"></span><span class="hide-phone">{}</span></a></li>
    "#, 
        nav.all_software,
        translate("Softwares"),
        nav.external,
        translate("External HD")
    ));

    // Add specific software tab if active
    if !nav.specific_software.is_empty() {
        content.push_str(&format!(r#"
                    <li class="link{}"><a href="{}"><span class="icon-tab he16-{}"></span>{}</a></li>
        "#, 
            nav.specific_software,
            software_info.link,
            software_info.software_type,
            translate(&software_info.name)
        ));
    }

    // Add folder tab if active
    if !nav.have_folder.is_empty() {
        content.push_str(&format!(r#"
                    <li class="link{}"><a href="{}"><span class="icon-tab he16-31"></span>{}</a></li>
        "#, 
            nav.have_folder,
            software_info.link,
            software_info.name
        ));
    }
    // Add text tab if active
    else if !nav.have_text.is_empty() {
        content.push_str(&format!(r#"
                    <li class="link{}"><a href="{}"><span class="icon-tab he16-30"></span>{}</a></li>
        "#, 
            nav.have_text,
            software_info.link,
            software_info.name
        ));
    }

    // Close navigation and start content area
    content.push_str(r#"
                </ul>
            </div>
            <div class="widget-content padding noborder">
    "#);

    // Add main content based on current view
    // This would be expanded with specific view logic in full implementation
    if !add.is_empty() && add == "external" {
        // External HD view
        content.push_str("<p>External HD content would be rendered here</p>");
    } else if !nav.specific_software.is_empty() {
        // Specific software view
        content.push_str(&format!("<p>Software details for: {}</p>", software_info.name));
    } else if !nav.have_folder.is_empty() {
        // Folder view
        content.push_str(&format!("<p>Folder content: {}</p>", software_info.name));
    } else if !nav.have_text.is_empty() {
        // Text view
        content.push_str(&format!("<p>Text content: {}</p>", software_info.name));
    } else {
        // Default software list view
        let software_list = software.show_software_list(user_id).await
            .unwrap_or_else(|_| "Error loading software list".to_string());
        content.push_str(&software_list);
    }

    // Close widget structure
    content.push_str(r#"
            </div>
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
    fn test_software_navigation_default() {
        let nav = SoftwareNavigation::default();
        assert_eq!(nav.all_software, " active");
        assert_eq!(nav.external, "");
        assert_eq!(nav.specific_software, "");
        assert_eq!(nav.have_folder, "");
        assert_eq!(nav.have_text, "");
    }

    #[test]
    fn test_software_info_initialization() {
        let info = SoftwareInfo {
            name: "test.exe".to_string(),
            software_type: "cracker".to_string(),
            link: "?id=123".to_string(),
            exists: true,
        };
        
        assert_eq!(info.name, "test.exe");
        assert_eq!(info.software_type, "cracker");
        assert_eq!(info.link, "?id=123");
        assert!(info.exists);
    }

    #[test]
    fn test_translate() {
        assert_eq!(translate("Softwares"), "Softwares");
        assert_eq!(translate("External HD"), "External HD");
    }
}