//! Login page handler - 1:1 port of login.php
//! 
//! Handles POST requests for user authentication with keepalive option support.
//! Redirects to index.php if method is not POST or user is already logged in.

use axum::{
    extract::{Extension, Form},
    http::{HeaderMap, StatusCode},
    response::{Html, Redirect},
};
use serde::Deserialize;
use crate::classes::database::LRSys;
use crate::session::{PhpSession, SessionValue};
use he_db::DbPool;

/// Login form data from POST request
#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
    pub keepalive: Option<String>, // Checkbox value
}

/// Login query parameters (for redirects)
#[derive(Debug, Deserialize)]
pub struct LoginQuery {
    // No query parameters needed for basic login
}

/// Main login handler - processes POST authentication requests
/// 
/// Port of: login.php
/// Behavior:
/// - Only accepts POST requests
/// - Redirects to index.php if already logged in or method is not POST
/// - Sanitizes input using htmlentities equivalent
/// - Sets keepalive flag if checkbox is checked
/// - Calls database login method
/// - Sets TYP session variable to 'LOG' on failure
/// - Always redirects back to login.php after processing
pub async fn login_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(mut session): Extension<PhpSession>,
    headers: HeaderMap,
    form: Option<Form<LoginForm>>,
) -> Result<Redirect, StatusCode> {
    // Check if user is already logged in
    if session.isset_login() {
        return Ok(Redirect::to("/index.php"));
    }

    // Only process POST requests
    let Form(login_data) = match form {
        Some(form_data) => form_data,
        None => {
            // Not a POST request, redirect to index
            return Ok(Redirect::to("/index.php"));
        }
    };

    // Get client IP for logging
    let client_ip = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .unwrap_or("127.0.0.1")
        .to_string();

    // Sanitize input (equivalent to htmlentities)
    let username = html_escape::encode_text(&login_data.username).to_string();
    let password = html_escape::encode_text(&login_data.password).to_string();

    // Initialize database system
    let mut db = LRSys::new(db_pool);

    // Set keepalive flag if checkbox was checked
    if login_data.keepalive.is_some() {
        db.set_keepalive(true);
    }

    // Attempt login
    let login_result = db.login(username, password, None).await;

    match login_result {
        Ok(success) => {
            if !success {
                // Login failed - set TYP session variable
                session.set("TYP", SessionValue::String("LOG".to_string()));
            }
            // If login succeeded, session is already set by db.login()
        }
        Err(e) => {
            // Database error during login
            eprintln!("Login database error: {:?}", e);
            session.set("TYP", SessionValue::String("LOG".to_string()));
        }
    }

    // Always redirect back to login.php (which will redirect to index.php)
    Ok(Redirect::to("/login.php"))
}

/// Helper function to create HTML escaped strings
/// Equivalent to PHP's htmlentities()
pub fn html_entities(input: &str) -> String {
    html_escape::encode_text(input).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_entities() {
        assert_eq!(html_entities("<script>"), "&lt;script&gt;");
        assert_eq!(html_entities("user@domain.com"), "user@domain.com");
        assert_eq!(html_entities("test&user"), "test&amp;user");
    }
}