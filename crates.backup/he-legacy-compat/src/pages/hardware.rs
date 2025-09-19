//! Hardware page handler - 1:1 port of hardware.php
//! 
//! Main hardware management page with tabs for:
//! - My hardware (default)
//! - Upgrade server
//! - Buy new server  
//! - Internet
//! - External HD

use axum::{
    extract::{Extension, Form, Query},
    http::StatusCode,
    response::Html,
};
use serde::Deserialize;
use std::collections::HashMap;
use crate::classes::system::System;
use crate::classes::pc::HardwareVPC;
use crate::session::{PhpSession, SessionValue};
use he_db::DbPool;

/// Query parameters for hardware page navigation
#[derive(Debug, Deserialize)]
pub struct HardwareQuery {
    pub opt: Option<String>, // upgrade, buy, internet, xhd
    pub id: Option<String>,  // PC/XHD ID for specific operations
}

/// Form data for hardware operations
#[derive(Debug, Deserialize)]
pub struct HardwareForm {
    // Dynamic form fields - will be processed by hardware.handlePost()
    #[serde(flatten)]
    pub data: HashMap<String, String>,
}

/// Hardware page navigation options
#[derive(Debug, Clone)]
pub struct HardwareNavigation {
    pub my_hardware: String,
    pub upgrade_pc: String,
    pub buy_pc: String,
    pub internet: String,
    pub xhd: String,
}

impl Default for HardwareNavigation {
    fn default() -> Self {
        Self {
            my_hardware: "active".to_string(),
            upgrade_pc: String::new(),
            buy_pc: String::new(),
            internet: String::new(),
            xhd: String::new(),
        }
    }
}

/// Main hardware handler - displays hardware management interface
/// 
/// Port of: hardware.php
/// Features:
/// - Session authentication required
/// - Tabbed navigation (My hardware, Upgrade, Buy, Internet, XHD)
/// - POST form handling for hardware operations
/// - GET parameter routing for different views
/// - Template rendering with PHP session compatibility
pub async fn hardware_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(mut session): Extension<PhpSession>,
    Query(query): Query<HardwareQuery>,
    form: Option<Form<HardwareForm>>,
) -> Result<Html<String>, StatusCode> {
    // Check if user is logged in (required for hardware page)
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

    // Initialize system and hardware classes
    let system = System::new();
    let mut hardware = HardwareVPC::new(user_id, db_pool.clone());

    // Get hardware info for VPC
    hardware.get_hardware_info(String::new(), "VPC").await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Initialize navigation state
    let mut nav = HardwareNavigation::default();
    let mut style = "style=\"text-align: center;\"";

    // Handle POST form submissions
    if let Some(Form(form_data)) = form {
        hardware.handle_post(form_data.data).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Clear session variables
    session.unset("XHD_ID");
    session.unset("CUR_PC");

    // Process GET parameters for navigation
    if let Some(opt) = &query.opt {
        let valid_opts = vec!["upgrade", "buy", "internet", "xhd"];
        
        if valid_opts.contains(&opt.as_str()) {
            nav.my_hardware = String::new();
            style = "";
            
            match opt.as_str() {
                "upgrade" => nav.upgrade_pc = "active".to_string(),
                "buy" => nav.buy_pc = "active".to_string(),
                "internet" => nav.internet = "active".to_string(),
                "xhd" => nav.xhd = "active".to_string(),
                _ => {}
            }
        }
    }

    // Generate page content
    let mut content = String::new();
    
    // Add message display if present
    if session.isset_msg() {
        content.push_str(&session.return_msg());
    }

    // Start widget structure
    content.push_str(&format!(r#"
    <div class="span12" {}>
        <div class="widget-box">
            <div class="widget-title">
                <ul class="nav nav-tabs">
                    <li class="link {}"><a href="hardware"><span class="icon-tab he16-servers"></span><span class="hide-phone">{}</span></a></li>
                    <li class="hide-phone link {}"><a href="?opt=upgrade"><span class="icon-tab he16-upgradeserver"></span>{}</a></li>
                    <li class="link {}"><a href="?opt=buy"><span class="icon-tab he16-buyserver"></span><span class="hide-phone">{}</span></a></li>
                    <li class="link {}"><a href="?opt=internet"><span class="he16-net icon-tab"></span><span class="hide-phone">{}</a></li>
                    <li class="link {}"><a href="?opt=xhd"><span class="icon-tab he16-xhd"></span><span class="hide-phone">{}</span></a></li>
                    <a href="{}"><span class="label label-info">{}</span></a>
                </ul>
            </div>
            <div class="widget-content padding noborder">
    "#, 
        style,
        nav.my_hardware,
        translate("My hardware"),
        nav.upgrade_pc,
        translate("Upgrade server"),
        nav.buy_pc,
        translate("Buy new server"),
        nav.internet,
        translate("Internet"),
        nav.xhd,
        translate("External HD"),
        session.help("hardware"),
        translate("Help")
    ));

    // Handle different page views based on GET parameters
    if let Some(opt) = &query.opt {
        let valid_opts = vec!["upgrade", "buy", "internet", "xhd"];
        
        if valid_opts.contains(&opt.as_str()) {
            match opt.as_str() {
                "upgrade" => {
                    if let Some(id) = &query.id {
                        // Validate numeric ID
                        if let Ok(pc_id) = id.parse::<i64>() {
                            // Get PC specifications
                            let pc_info = hardware.get_pc_spec(pc_id, "VPC", String::new()).await;
                            
                            match pc_info {
                                Ok(Some(pc_data)) => {
                                    // Set current PC in session and show store page
                                    session.set("CUR_PC", SessionValue::Integer(pc_id));
                                    let store_content = hardware.store_show_page(Some(pc_data), "CPU").await
                                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                                    content.push_str(&store_content);
                                },
                                Ok(None) => {
                                    // PC doesn't exist
                                    let error_content = system.handle_error("This server does not exists.", "hardware?opt=upgrade");
                                    content.push_str(&error_content);
                                },
                                Err(_) => {
                                    let error_content = system.handle_error("Database error.", "hardware?opt=upgrade");
                                    content.push_str(&error_content);
                                }
                            }
                        } else {
                            // Invalid ID format
                            let error_content = system.handle_error("Invalid server ID.", "hardware?opt=upgrade");
                            content.push_str(&error_content);
                        }
                    } else {
                        // List PCs for upgrade
                        let pc_list = hardware.list_pcs(user_id, "upgrade").await
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                        content.push_str(&pc_list);
                    }
                },
                "buy" => {
                    // Show buy PC page
                    let buy_content = hardware.store_show_page(None, "BUY_PC").await
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                    content.push_str(&buy_content);
                },
                "internet" => {
                    // Show internet page
                    let net_content = hardware.store_show_page(None, "NET").await
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                    content.push_str(&net_content);
                },
                "xhd" => {
                    if let Some(id) = &query.id {
                        if let Ok(xhd_id) = id.parse::<i64>() {
                            // Upgrade existing XHD
                            session.set("XHD_ID", SessionValue::Integer(xhd_id));
                            let xhd_content = hardware.store_show_page(None, "XHD-UPG").await
                                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                            content.push_str(&xhd_content);
                        } else {
                            // Buy new XHD
                            let xhd_content = hardware.store_show_page(None, "XHD-BUY").await
                                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                            content.push_str(&xhd_content);
                        }
                    } else {
                        // Buy new XHD (no ID provided)
                        let xhd_content = hardware.store_show_page(None, "XHD-BUY").await
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                        content.push_str(&xhd_content);
                    }
                },
                _ => {
                    return Ok(Html("<script>window.location.href='/hardware';</script>".to_string()));
                }
            }
        } else {
            return Ok(Html("<script>window.location.href='/hardware';</script>".to_string()));
        }
    } else {
        // Default view - show user's hardware
        
        // Clear recent buy session variable
        session.unset("RECENT_BUY");
        
        // Show PC total/summary
        let pc_total = hardware.show_pc_total(user_id).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        content.push_str(&pc_total);
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
    fn test_hardware_navigation_default() {
        let nav = HardwareNavigation::default();
        assert_eq!(nav.my_hardware, "active");
        assert_eq!(nav.upgrade_pc, "");
        assert_eq!(nav.buy_pc, "");
        assert_eq!(nav.internet, "");
        assert_eq!(nav.xhd, "");
    }

    #[test]
    fn test_translate() {
        assert_eq!(translate("My hardware"), "My hardware");
        assert_eq!(translate("Help"), "Help");
    }
}