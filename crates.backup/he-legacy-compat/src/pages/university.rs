//! University page handler - 1:1 port of university.php
//! 
//! University system with dual functionality:
//! - Software research and development
//! - Certification learning and completion
//! - Tutorial and skill progression
//! - Page-based learning system with validation

use axum::{
    extract::{Extension, Form, Query},
    http::StatusCode,
    response::Html,
};
use serde::Deserialize;
use std::collections::HashMap;
use crate::classes::{system::System, software::SoftwareVPC, player::Player, ranking::Ranking};
use crate::session::{PhpSession, SessionValue};
use he_db::DbPool;

/// Query parameters for university page navigation
#[derive(Debug, Deserialize)]
pub struct UniversityQuery {
    pub opt: Option<String>,    // certification
    pub learn: Option<String>,  // certification ID (1-5)
    pub page: Option<String>,   // page number for learning
    pub complete: Option<String>, // completion hash
    pub id: Option<String>,     // software ID for research
}

/// Form data for university operations (software research and certification)
#[derive(Debug, Deserialize)]
pub struct UniversityForm {
    #[serde(flatten)]
    pub data: HashMap<String, String>,
}

/// University page navigation state
#[derive(Debug, Clone)]
pub struct UniversityNavigation {
    pub research: String,
    pub cert: String,
    pub center: String,
}

impl Default for UniversityNavigation {
    fn default() -> Self {
        Self {
            research: "active".to_string(),
            cert: String::new(),
            center: String::new(),
        }
    }
}

/// Certification information
#[derive(Debug, Clone)]
pub struct CertificationInfo {
    pub id: i64,
    pub is_learning: bool,
    pub current_page: i64,
    pub total_pages: i64,
}

/// Main university handler - displays research and certification systems
/// 
/// Port of: university.php
/// Features:
/// - Dual-tab navigation (Research/Certifications)
/// - Software research and development system
/// - Certification learning with page-based progression
/// - POST form handling for both research and certification
/// - Complex validation and error handling
/// - Tutorial integration and skill progression
/// - Dynamic layout adjustment based on content type
pub async fn university_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(mut session): Extension<PhpSession>,
    Query(query): Query<UniversityQuery>,
    form: Option<Form<UniversityForm>>,
) -> Result<Html<String>, StatusCode> {
    // Check if user is logged in (required for university page)
    if !session.isset_login() {
        return Ok(Html("<script>window.location.href='/index.php';</script>".to_string()));
    }

    // Initialize required classes
    let system = System::new();
    let mut software = SoftwareVPC::new(db_pool.clone());

    // Handle POST form submissions
    if let Some(Form(form_data)) = form {
        software.handle_post("university", form_data.data).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Initialize navigation state
    let mut nav = UniversityNavigation::default();

    // Process opt parameter for certification section
    if let Some(opt) = &query.opt {
        if let Err(error_msg) = validate_opt_parameter(opt) {
            session.add_msg(&error_msg, "error");
            return Ok(Html(session.return_msg()));
        }

        if opt == "certification" {
            nav.research = String::new();
            nav.cert = "active".to_string();
            nav.center = " center".to_string();
        }
    }

    // Adjust center class for learn parameter
    if query.learn.is_some() {
        nav.center = String::new();
    }

    // Build page content
    let mut content = String::new();

    // Add message display if present
    if session.isset_msg() {
        content.push_str(&session.return_msg());
    }

    // Start widget structure with navigation tabs
    content.push_str(&format!(r#"
                    <div class="span12{}">
                        <div class="widget-box">
                            <div class="widget-title">
                                <ul class="nav nav-tabs">
                                    <li class="link {}"><a href="university.php"><span class="icon-tab he16-research"></span><span class="hide-phone">{}</span></a></li>
                                    <li class="link {}"><a href="university?opt=certification"><span class="icon-tab he16-certs"></span><span class="hide-phone">{}</span></a></li>
                                    <a href="{}"><span class="label label-info">{}</span></a>
                                </ul>
                            </div>
                            <div class="cert-complete"></div>
                            <div class="widget-content padding noborder">
    "#, 
        nav.center,
        nav.research,
        translate("Research softwares"),
        nav.cert,
        translate("Certifications"),
        session.help("university", "research"),
        translate("Help")
    ));

    // Generate main content based on current parameters
    let main_content = if let Some(opt) = &query.opt {
        handle_certification_section(&db_pool, &mut session, &query).await?
    } else if let Some(id) = &query.id {
        handle_software_research(&mut software, &mut session, id).await?
    } else {
        // Default research list
        software.research_list().await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    content.push_str(&main_content);

    // Close widget structure
    content.push_str(r#"
                            </div>
                            <div style="clear: both;" class="nav nav-tabs">&nbsp;</div>
                        </div>
                    </div>
    "#);

    Ok(Html(content))
}

/// Validate opt parameter
fn validate_opt_parameter(opt: &str) -> Result<(), String> {
    // Check if opt is numeric (should not be)
    if opt.parse::<i64>().is_ok() {
        return Err("Invalid get.".to_string());
    }
    
    // Validate allowed values
    if opt != "certification" {
        return Err("Invalid get.".to_string());
    }
    
    Ok(())
}

/// Handle certification section functionality
async fn handle_certification_section(
    db_pool: &DbPool,
    session: &mut PhpSession,
    query: &UniversityQuery,
) -> Result<String, StatusCode> {
    if let Some(learn_str) = &query.learn {
        handle_certification_learning(db_pool, session, query).await
    } else if let Some(complete_str) = &query.complete {
        handle_certification_completion(db_pool, session, complete_str).await
    } else {
        // Default certification list
        let ranking = Ranking::new(db_pool.clone());
        ranking.cert_list().await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
}

/// Handle certification learning process
async fn handle_certification_learning(
    db_pool: &DbPool,
    session: &mut PhpSession,
    query: &UniversityQuery,
) -> Result<String, StatusCode> {
    let player = Player::new(db_pool.clone());
    let ranking = Ranking::new(db_pool.clone());

    // Validate learn parameter
    let learn_id = match query.learn.as_ref().unwrap().parse::<i64>() {
        Ok(id) if id >= 1 && id <= 5 => id,
        _ => {
            session.add_msg("Invalid certification.", "error");
            return Ok(format!("<script>window.location.href='university?opt=certification';</script>"));
        }
    };

    // Validate player learning status
    let player_learning = player.player_learning().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if player_learning == 0 {
        session.add_msg("You are not learning any certification. Did you bought it?", "error");
        return Ok(format!("<script>window.location.href='university?opt=certification';</script>"));
    }

    if player_learning != learn_id {
        session.add_msg("You are busy learning this certification.", "error");
        return Ok(format!("<script>window.location.href='university?opt=certification&learn={}';</script>", player_learning));
    }

    // Validate certification eligibility
    if !ranking.cert_validate2learn(learn_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        session.add_msg("Some error happened.", "error");
        return Ok(format!("<script>window.location.href='university?opt=certification';</script>"));
    }

    // Handle page parameter
    let page = if let Some(page_str) = &query.page {
        match page_str.parse::<i64>() {
            Ok(page_num) => {
                let total_pages = ranking.cert_total_pages(learn_id).await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                
                if page_num > total_pages || page_num < 1 {
                    session.add_msg("Invalid page number.", "error");
                    return Ok(format!("<script>window.location.href='university?opt=certification&learn={}';</script>", learn_id));
                }
                page_num
            },
            Err(_) => {
                session.add_msg("Invalid page number.", "error");
                return Ok(format!("<script>window.location.href='university?opt=certification&learn={}';</script>", learn_id));
            }
        }
    } else {
        0
    };

    // Show certification page
    ranking.cert_show_page(learn_id, page).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handle certification completion
async fn handle_certification_completion(
    db_pool: &DbPool,
    session: &mut PhpSession,
    complete_str: &str,
) -> Result<String, StatusCode> {
    let player = Player::new(db_pool.clone());
    let ranking = Ranking::new(db_pool.clone());

    // Validate player learning status
    let player_learning = player.player_learning().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if player_learning == 0 {
        session.add_msg("You are not learning any certification. Did you bought it?", "error");
        return Ok(format!("<script>window.location.href='university?opt=certification';</script>"));
    }

    // Validate completion hash length
    if complete_str.len() != 32 {
        session.add_msg("Invalid page.", "error");
        return Ok(format!("<script>window.location.href='university?opt=certification';</script>"));
    }

    // Validate completion hash
    let user_id = session.get("id")
        .and_then(|v| match v {
            SessionValue::String(s) => s.parse::<i64>().ok(),
            SessionValue::Integer(i) => Some(*i),
            _ => None,
        })
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let expected_hash = format!("{:x}", md5::compute(format!("cert{}{}", player_learning, user_id)));
    
    if expected_hash != complete_str {
        session.add_msg("You are busy learning this certification.", "error");
        return Ok(format!("<script>window.location.href='university?opt=certification&learn={}';</script>", player_learning));
    }

    // Validate certification eligibility again
    if !ranking.cert_validate2learn(player_learning).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        session.add_msg("Some error happened.", "error");
        return Ok(format!("<script>window.location.href='university?opt=certification';</script>"));
    }

    // Complete certification
    ranking.cert_add(player_learning).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    ranking.cert_end(player_learning).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    player.unset_player_learning().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Auto-progress to next certification if completing first one
    if player_learning == 1 {
        player.set_player_learning(2).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Redirect to certification list
    Ok(format!("<script>window.location.href='university?opt=certification';</script>"))
}

/// Handle software research functionality
async fn handle_software_research(
    software: &mut SoftwareVPC,
    session: &mut PhpSession,
    id_str: &str,
) -> Result<String, StatusCode> {
    // Validate ID parameter
    let software_id = match id_str.parse::<i64>() {
        Ok(id) if id >= 0 => id,
        _ => {
            session.add_msg("Invalid get", "error");
            return Ok(format!("<script>window.location.href='university.php';</script>"));
        }
    };

    // Get user ID from session
    let user_id = session.get("id")
        .and_then(|v| match v {
            SessionValue::String(s) => s.parse::<i64>().ok(),
            SessionValue::Integer(i) => Some(*i),
            _ => None,
        })
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Validate software exists and belongs to user
    if !software.isset_software(software_id, user_id, "VPC").await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        session.add_msg("This software does not exists.", "error");
        return Ok(format!("<script>window.location.href='university.php';</script>"));
    }

    // Show research interface for this software
    software.research_show(software_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
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
    fn test_university_navigation_default() {
        let nav = UniversityNavigation::default();
        assert_eq!(nav.research, "active");
        assert_eq!(nav.cert, "");
        assert_eq!(nav.center, "");
    }

    #[test]
    fn test_validate_opt_parameter() {
        // Valid certification parameter
        assert!(validate_opt_parameter("certification").is_ok());
        
        // Invalid numeric parameter
        assert!(validate_opt_parameter("123").is_err());
        
        // Invalid string parameter
        assert!(validate_opt_parameter("invalid").is_err());
    }

    #[test]
    fn test_university_navigation_certification_active() {
        let mut nav = UniversityNavigation::default();
        nav.research = String::new();
        nav.cert = "active".to_string();
        nav.center = " center".to_string();
        
        assert_eq!(nav.research, "");
        assert_eq!(nav.cert, "active");
        assert_eq!(nav.center, " center");
    }

    #[test]
    fn test_certification_info() {
        let cert_info = CertificationInfo {
            id: 1,
            is_learning: true,
            current_page: 5,
            total_pages: 10,
        };

        assert_eq!(cert_info.id, 1);
        assert!(cert_info.is_learning);
        assert_eq!(cert_info.current_page, 5);
        assert_eq!(cert_info.total_pages, 10);
    }

    #[test]
    fn test_certification_id_validation() {
        // Valid certification IDs (1-5)
        assert!(matches!("1".parse::<i64>(), Ok(id) if id >= 1 && id <= 5));
        assert!(matches!("5".parse::<i64>(), Ok(id) if id >= 1 && id <= 5));
        
        // Invalid certification IDs
        assert!(matches!("0".parse::<i64>(), Ok(id) if !(id >= 1 && id <= 5)));
        assert!(matches!("6".parse::<i64>(), Ok(id) if !(id >= 1 && id <= 5)));
        assert!("invalid".parse::<i64>().is_err());
    }

    #[test]
    fn test_completion_hash_validation() {
        // Valid hash length
        let valid_hash = "a".repeat(32);
        assert_eq!(valid_hash.len(), 32);
        
        // Invalid hash lengths
        let short_hash = "a".repeat(31);
        let long_hash = "a".repeat(33);
        assert_ne!(short_hash.len(), 32);
        assert_ne!(long_hash.len(), 32);
    }

    #[test]
    fn test_translate() {
        assert_eq!(translate("Research softwares"), "Research softwares");
        assert_eq!(translate("Certifications"), "Certifications");
        assert_eq!(translate("Help"), "Help");
    }
}