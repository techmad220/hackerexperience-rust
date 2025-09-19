//! Internet page handler - 1:1 port of internet.php
//! 
//! Network simulation and hacking interface:
//! - IP address navigation
//! - Internet session management
//! - Server browsing and interaction
//! - Certification requirement validation

use axum::{
    extract::{Extension, Form, Query},
    http::StatusCode,
    response::Html,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use crate::classes::{system::System, player::Player, internet::Internet, ranking::Ranking};
use crate::session::{PhpSession, SessionValue};
use he_db::DbPool;

/// Query parameters for internet navigation
#[derive(Debug, Deserialize)]
pub struct InternetQuery {
    pub ip: Option<String>, // IP address to navigate to
}

/// Form data for internet operations
#[derive(Debug, Deserialize)]
pub struct InternetForm {
    #[serde(flatten)]
    pub data: HashMap<String, String>,
}

/// Main internet handler - network simulation interface
/// 
/// Port of: internet.php
/// Features:
/// - Certification requirement validation (Hacking 101)
/// - IP address navigation and validation
/// - Internet session management
/// - POST form handling for network operations
/// - Dynamic server navigation based on session state
/// - Home IP fallback for new sessions
pub async fn internet_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(mut session): Extension<PhpSession>,
    Query(query): Query<InternetQuery>,
    form: Option<Form<InternetForm>>,
) -> Result<Html<String>, StatusCode> {
    // Check if user is logged in (required for internet page)
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

    // Initialize required classes
    let system = System::new();
    let _player = Player::new(db_pool.clone());
    let mut internet = Internet::new(db_pool.clone());
    let ranking = Ranking::new(db_pool.clone());

    // Check certification requirement - Hacking 101 (cert ID 2)
    match ranking.cert_have(2).await {
        Ok(false) => {
            // User doesn't have required certification
            session.add_msg(&format!(
                "You need the certification {} to enable this page.",
                translate("Hacking 101")
            ), "error");
            return Ok(Html("<script>window.location.href='/university?opt=certification';</script>".to_string()));
        },
        Err(_) => {
            // Error checking certification
            return Ok(Html("<script>window.location.href='/university?opt=certification';</script>".to_string()));
        },
        _ => {} // User has certification, continue
    }

    // Handle POST form submissions
    if let Some(Form(form_data)) = form {
        internet.handle_post(form_data.data).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Determine navigation target
    let target_ip = if let Some(ip_str) = &query.ip {
        // User provided IP address
        let trimmed_ip = ip_str.trim();
        
        // Validate IP address format
        if !system.validate(trimmed_ip, "ip") {
            return Ok(Html("Invalid IP".to_string()));
        }

        // Convert IP string to numeric format
        match trimmed_ip.parse::<Ipv4Addr>() {
            Ok(ip_addr) => {
                // Convert to long format (equivalent to PHP's ip2long)
                let ip_long = u32::from(ip_addr) as i64;
                ip_long
            },
            Err(_) => {
                return Ok(Html("Invalid IP format".to_string()));
            }
        }
    } else {
        // No IP provided - determine from session state
        if session.is_internet_logged() {
            // User is logged into a server
            match session.get("LOGGED_IN") {
                Some(SessionValue::Integer(ip)) => *ip,
                Some(SessionValue::String(ip_str)) => {
                    ip_str.parse::<i64>().unwrap_or_else(|| {
                        // Fallback to home IP if parsing fails
                        internet.home_get_ip().unwrap_or(0)
                    })
                },
                _ => internet.home_get_ip().unwrap_or(0),
            }
        } else if session.isset_internet_session() {
            // User has internet session but not logged in
            match session.get("CUR_IP") {
                Some(SessionValue::Integer(ip)) => *ip,
                Some(SessionValue::String(ip_str)) => {
                    ip_str.parse::<i64>().unwrap_or_else(|| {
                        // Fallback to home IP if parsing fails
                        internet.home_get_ip().unwrap_or(0)
                    })
                },
                _ => internet.home_get_ip().unwrap_or(0),
            }
        } else {
            // No internet session - go to user's home IP
            internet.home_get_ip().unwrap_or(0)
        }
    };

    // Navigate to the determined IP address
    let navigation_content = match internet.navigate(target_ip).await {
        Ok(html) => html,
        Err(e) => {
            eprintln!("Error navigating to IP {}: {:?}", target_ip, e);
            format!(
                r#"<div class="error-message">
                    <h3>Navigation Error</h3>
                    <p>Unable to connect to the specified address.</p>
                    <p>Error: {}</p>
                    <a href="/internet" class="btn btn-primary">Return to Home</a>
                </div>"#,
                e
            )
        }
    };

    Ok(Html(navigation_content))
}

/// Convert IP address long format back to string
/// Helper function equivalent to PHP's long2ip
pub fn long_to_ip(ip_long: i64) -> String {
    let ip_u32 = ip_long as u32;
    let ip_addr = Ipv4Addr::from(ip_u32);
    ip_addr.to_string()
}

/// Convert IP address string to long format
/// Helper function equivalent to PHP's ip2long
pub fn ip_to_long(ip_str: &str) -> Result<i64, std::net::AddrParseError> {
    let ip_addr: Ipv4Addr = ip_str.parse()?;
    Ok(u32::from(ip_addr) as i64)
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
    fn test_ip_conversion() {
        let ip_str = "192.168.1.1";
        let ip_long = ip_to_long(ip_str).unwrap();
        let converted_back = long_to_ip(ip_long);
        
        assert_eq!(converted_back, ip_str);
    }

    #[test]
    fn test_ip_validation() {
        let valid_ip = "192.168.1.1";
        let invalid_ip = "999.999.999.999";
        
        assert!(valid_ip.parse::<Ipv4Addr>().is_ok());
        assert!(invalid_ip.parse::<Ipv4Addr>().is_err());
    }

    #[test]
    fn test_ip_to_long_conversion() {
        // Test common IP addresses
        let localhost = ip_to_long("127.0.0.1").unwrap();
        assert_eq!(localhost, 2130706433); // 127.0.0.1 as long
        
        let private_ip = ip_to_long("192.168.1.1").unwrap();
        assert_eq!(private_ip, 3232235777); // 192.168.1.1 as long
    }

    #[test]
    fn test_long_to_ip_conversion() {
        // Test conversion back from long
        let localhost_long = 2130706433i64;
        let ip_str = long_to_ip(localhost_long);
        assert_eq!(ip_str, "127.0.0.1");
        
        let private_long = 3232235777i64;
        let ip_str = long_to_ip(private_long);
        assert_eq!(ip_str, "192.168.1.1");
    }

    #[test]
    fn test_internet_query_parsing() {
        // Test with IP
        let query_with_ip = InternetQuery {
            ip: Some("192.168.1.1".to_string()),
        };
        assert!(query_with_ip.ip.is_some());
        assert_eq!(query_with_ip.ip.unwrap(), "192.168.1.1");

        // Test without IP
        let query_without_ip = InternetQuery {
            ip: None,
        };
        assert!(query_without_ip.ip.is_none());
    }

    #[test]
    fn test_translate() {
        assert_eq!(translate("Hacking 101"), "Hacking 101");
        assert_eq!(translate("Invalid IP"), "Invalid IP");
    }
}