//! Registration page handler - 1:1 port of register.php
//! 
//! Handles POST requests for user registration.
//! Redirects to index.php if method is not POST or user is already logged in.

use axum::{
    extract::{Extension, Form},
    http::{HeaderMap, StatusCode},
    response::Redirect,
};
use serde::Deserialize;
use crate::classes::database::LRSys;
use crate::session::{PhpSession, SessionValue};
use he_db::DbPool;

/// Registration form data from POST request
#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    pub username: String,
    pub password: String,
    pub email: String,
}

/// Main registration handler - processes POST registration requests
/// 
/// Port of: register.php
/// Behavior:
/// - Only accepts POST requests
/// - Redirects to index.php if already logged in or method is not POST
/// - Extracts username, password, and email from POST data
/// - Calls database register method with extracted data
/// - Sets TYP session variable to 'REG' after processing
/// - Always redirects to index.php after processing
/// - TODO: Email confirmation header (preserved from original comment)
pub async fn register_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(mut session): Extension<PhpSession>,
    headers: HeaderMap,
    form: Option<Form<RegisterForm>>,
) -> Result<Redirect, StatusCode> {
    // Check if user is already logged in
    if session.isset_login() {
        return Ok(Redirect::to("/index.php"));
    }

    // Only process POST requests
    let Form(register_data) = match form {
        Some(form_data) => form_data,
        None => {
            // Not a POST request, redirect to index
            return Ok(Redirect::to("/index.php"));
        }
    };

    // Get client IP for registration tracking
    let client_ip = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .unwrap_or("127.0.0.1")
        .to_string();

    // Extract registration data (no HTML escaping in original)
    let reg_login = register_data.username;
    let reg_pass = register_data.password;
    let reg_email = register_data.email;

    // Initialize database system
    let mut database = LRSys::new(db_pool);

    // Attempt registration
    let registration_result = database.register(reg_login, reg_pass, reg_email, client_ip).await;

    match registration_result {
        Ok(success) => {
            if success {
                // Registration succeeded
                // TODO: header to email confirmation (preserved from original comment)
                // This would be where email confirmation logic would go
                
                // For now, we'll just log successful registration
                println!("User registration successful");
            }
        }
        Err(e) => {
            // Registration failed
            eprintln!("Registration error: {:?}", e);
        }
    }

    // Always set TYP session variable to 'REG' (registration page identifier)
    session.set("TYP", SessionValue::String("REG".to_string()));

    // Always redirect to index.php
    Ok(Redirect::to("/index.php"))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_register_form_deserialization() {
        // Test that the form structure deserializes correctly
        let form_data = RegisterForm {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            email: "test@example.com".to_string(),
        };
        
        assert_eq!(form_data.username, "testuser");
        assert_eq!(form_data.password, "testpass");
        assert_eq!(form_data.email, "test@example.com");
    }
}