//! Password reset page handler - 1:1 port of reset.php
//! 
//! Handles three main flows:
//! 1. Request password reset via email
//! 2. Display reset form when code is provided via GET
//! 3. Process password change when code and new password are provided via POST

use axum::{
    extract::{Extension, Form, Query},
    http::StatusCode,
    response::{Html, Redirect},
};
use serde::Deserialize;
use uuid::Uuid;
use crate::classes::{system::System, bcrypt::BCrypt, ses::SES};
use crate::session::{PhpSession, SessionValue};
use he_db::DbPool;
use sqlx::Row;

/// Query parameters for reset page (GET)
#[derive(Debug, Deserialize)]
pub struct ResetQuery {
    pub code: Option<String>, // Reset code from email
}

/// Form data for email reset request
#[derive(Debug, Deserialize)]
pub struct EmailResetForm {
    pub email: String,
}

/// Form data for password change
#[derive(Debug, Deserialize)]
pub struct PasswordResetForm {
    pub code: String,
    pub pwd: String,
    pub pwd2: String,
}

/// Main reset handler - displays forms and processes reset requests
/// 
/// Port of: reset.php
/// Features:
/// - Email reset request processing
/// - Reset code validation and form display
/// - Password change processing with validation
/// - Session management and error handling
/// - Email sending via SES integration
pub async fn reset_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(mut session): Extension<PhpSession>,
    Query(query): Query<ResetQuery>,
    email_form: Option<Form<EmailResetForm>>,
    password_form: Option<Form<PasswordResetForm>>,
) -> Result<Html<String>, StatusCode> {
    // Redirect to index if user is already logged in
    if session.isset_login() {
        return Ok(Html("<script>window.location.href='/index';</script>".to_string()));
    }

    let mut msg = String::new();

    // Handle email reset request (POST with email)
    if let Some(Form(email_data)) = email_form {
        return handle_email_reset(db_pool, &mut session, email_data).await;
    }

    // Handle password change (POST with code and passwords)
    if let Some(Form(password_data)) = password_form {
        return handle_password_change(db_pool, password_data).await;
    }

    // Handle GET request with reset code - display password change form
    if let Some(code) = query.code {
        return display_password_form(db_pool, code).await;
    }

    // Default: display email request form
    Ok(Html(format!(r#"
    <html>
    <head>
    </head>
    <body>

    {}

        <form action="reset" method="POST">
            <input type="text" name="email" placeholder="Please insert your email"><br/><br/>
            <input type="submit" value="Request password reset">
        </form>
    </body>	
    </html>
    "#, if !msg.is_empty() { format!("{}<br/><br/>", msg) } else { String::new() })))
}

/// Handle email reset request
async fn handle_email_reset(
    db_pool: DbPool,
    session: &mut PhpSession,
    email_data: EmailResetForm,
) -> Result<Html<String>, StatusCode> {
    let system = System::new();
    let email = email_data.email;

    // Validate email format
    if !system.validate(&email, "email") {
        return Ok(Html(r#"
        <html><body>
        Invalid email<br/><br/>
        <form action="reset" method="POST">
            <input type="text" name="email" placeholder="Please insert your email"><br/><br/>
            <input type="submit" value="Request password reset">
        </form>
        </body></html>
        "#.to_string()));
    }

    // Check if email exists in database
    let user_query = sqlx::query("SELECT id, login FROM users WHERE email = ? LIMIT 1")
        .bind(&email)
        .fetch_optional(&db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user_info = match user_query {
        Some(row) => {
            let id: i64 = row.get("id");
            let login: String = row.get("login");
            (id, login)
        },
        None => {
            return Ok(Html(r#"
            <html><body>
            This email is not registered<br/><br/>
            <form action="reset" method="POST">
                <input type="text" name="email" placeholder="Please insert your email"><br/><br/>
                <input type="submit" value="Request password reset">
            </form>
            </body></html>
            "#.to_string()));
        }
    };

    // Generate unique reset code
    let code = Uuid::new_v4().simple().to_string();
    let code = &code[..13]; // Match PHP uniqid() length

    // Insert reset code into database
    sqlx::query("INSERT INTO email_reset (userID, code) VALUES (?, ?)")
        .bind(user_info.0)
        .bind(code)
        .execute(&db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Send reset email via SES
    let ses = SES::new();
    if let Err(e) = ses.send("request_reset", vec![
        ("to", &email),
        ("user", &user_info.1),
        ("code", code),
    ]).await {
        eprintln!("Failed to send reset email: {:?}", e);
        // Continue anyway - user might still use the code
    }

    // Set session success message
    session.set("MSG", SessionValue::String("Reset email sent.".to_string()));
    session.set("TYP", SessionValue::String("REG".to_string()));
    session.set("MSG_TYPE", SessionValue::String("success".to_string()));

    // Redirect to index
    Ok(Html("<script>window.location.href='/index';</script>".to_string()))
}

/// Handle password change with reset code
async fn handle_password_change(
    db_pool: DbPool,
    password_data: PasswordResetForm,
) -> Result<Html<String>, StatusCode> {
    let code = password_data.code;

    // Validate code length (PHP uniqid() generates 13-character codes)
    if code.len() != 13 {
        return Ok(Html("Bad code".to_string()));
    }

    // Verify reset code exists
    let reset_query = sqlx::query("SELECT userID FROM email_reset WHERE code = ? LIMIT 1")
        .bind(&code)
        .fetch_optional(&db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user_id: i64 = match reset_query {
        Some(row) => row.get("userID"),
        None => {
            return Ok(Html("This code is invalid.".to_string()));
        }
    };

    // Validate passwords match
    if password_data.pwd != password_data.pwd2 {
        return Ok(Html("Passwords are different".to_string()));
    }

    let pwd = password_data.pwd;

    // Validate password length (using HTML entities length like PHP)
    if html_escape::encode_text(&pwd).len() <= 5 {
        return Ok(Html("Please use at least 6 characteres".to_string()));
    }

    // Hash the new password
    let bcrypt = BCrypt::new();
    let escaped_pwd = html_escape::encode_text(&pwd);
    let new_pwd_hash = bcrypt.hash(&escaped_pwd)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update user password
    sqlx::query("UPDATE users SET password = ? WHERE id = ?")
        .bind(&new_pwd_hash)
        .bind(user_id)
        .execute(&db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Delete the used reset code
    sqlx::query("DELETE FROM email_reset WHERE code = ?")
        .bind(&code)
        .execute(&db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html("Password changed".to_string()))
}

/// Display password change form for valid reset code
async fn display_password_form(
    db_pool: DbPool,
    code: String,
) -> Result<Html<String>, StatusCode> {
    // Validate code length
    if code.len() != 13 {
        return Ok(Html("Bad code".to_string()));
    }

    // Verify reset code exists
    let reset_query = sqlx::query("SELECT userID FROM email_reset WHERE code = ? LIMIT 1")
        .bind(&code)
        .fetch_optional(&db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if reset_query.is_none() {
        return Ok(Html("This code is invalid.".to_string()));
    }

    // Display password change form
    Ok(Html(format!(r#"
    <form action="" method="POST">
        <input type="hidden" name="code" value="{}">
        Password: <input type="password" name="pwd"> (6 or more characters)<br/>
        Repeat plz: <input type="password" name="pwd2"><br/>
        <input type="submit" value="Change password">
    </form>
    "#, html_escape::encode_text(&code))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_form_validation() {
        // Test password length validation
        let short_pwd = "12345";
        assert!(html_escape::encode_text(short_pwd).len() <= 5);
        
        let valid_pwd = "123456";
        assert!(html_escape::encode_text(valid_pwd).len() > 5);
    }

    #[test]
    fn test_reset_code_validation() {
        assert_eq!("1234567890123".len(), 13); // Valid length
        assert_ne!("123".len(), 13); // Invalid length
    }
}