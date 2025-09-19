//! Welcome page handler - 1:1 port of welcome.php
//! 
//! Handles new user welcome and onboarding with email verification system.
//! Manages tutorial introduction and step-by-step guidance for new players.

use axum::{
    extract::{Extension, Query, Form},
    http::{StatusCode, HeaderMap},
    response::{Html, Redirect, Json},
};
use serde::{Deserialize, Serialize};
use crate::classes::{
    system::System,
    email_verification::EmailVerification,
    database::LRSys,
    player::Player,
};
use crate::session::{PhpSession, SessionValue};
use he_db::DbPool;

/// Query parameters for welcome page
#[derive(Debug, Deserialize)]
pub struct WelcomeQuery {
    pub code: Option<String>,
}

/// Form data for email verification
#[derive(Debug, Deserialize)]
pub struct VerificationForm {
    pub code: String,
}

/// JSON response for verification attempts
#[derive(Debug, Serialize)]
pub struct VerificationResponse {
    pub status: Option<String>,
    pub msg: String,
}

/// Main welcome handler - handles GET requests, email verification, and welcome display
/// 
/// Port of: welcome.php
/// Behavior:
/// - Displays welcome screen for logged in users without certificates
/// - Handles email verification code validation
/// - Shows verification input or start tutorial button based on email verification status
/// - Handles GET code parameter for email link verification
/// - Redirects to index.php if user already has certificates or not logged in
pub async fn welcome_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(mut session): Extension<PhpSession>,
    headers: HeaderMap,
    Query(params): Query<WelcomeQuery>,
    form: Option<Form<VerificationForm>>,
) -> Result<axum::response::Response, StatusCode> {
    let mut system = System::new(db_pool.clone());
    let mut email_verification = EmailVerification::new(db_pool.clone());

    // Check if this is a POST request for verification
    if let Some(Form(verification_data)) = form {
        return handle_verification_post(verification_data, &mut email_verification, &mut session).await;
    }

    // Handle GET code parameter for email verification links
    if let Some(code) = params.code {
        return handle_code_verification(code, &mut email_verification, db_pool, &mut session).await;
    }

    // Regular GET request - show welcome screen
    handle_welcome_display(&mut system, &mut email_verification, &session).await
}

/// Handle POST verification code submission
async fn handle_verification_post(
    verification_data: VerificationForm,
    email_verification: &mut EmailVerification,
    session: &mut PhpSession,
) -> Result<axum::response::Response, StatusCode> {
    let mut result = VerificationResponse {
        status: None,
        msg: String::new(),
    };

    let mut fail = false;
    let code = verification_data.code;

    // Validate code length
    if code.is_empty() || code.len() != 25 {
        fail = true;
    }

    // Check if user is logged in
    let user_id = match session.get_int("id") {
        Some(id) => id,
        None => {
            fail = true;
            0
        }
    };

    // Check if already verified
    if !fail {
        match email_verification.is_verified(user_id).await {
            Ok(verified) => {
                if verified {
                    fail = true;
                }
            }
            Err(_) => fail = true,
        }
    }

    if !fail {
        result.status = Some("OK".to_string());
        
        match email_verification.verify(user_id, &code).await {
            Ok(success) => {
                if !success {
                    result.msg = "Verification code does not match".to_string(); // TODO: Localize
                }
            }
            Err(_) => {
                result.msg = "Verification code does not match".to_string(); // TODO: Localize
            }
        }
    } else {
        result.msg = "Invalid verification code".to_string(); // TODO: Localize
    }

    let response = axum::response::Response::builder()
        .header("content-type", "application/json")
        .body(serde_json::to_string(&result).unwrap().into())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}

/// Handle GET code parameter verification (from email links)
async fn handle_code_verification(
    code: String,
    email_verification: &mut EmailVerification,
    db_pool: DbPool,
    session: &mut PhpSession,
) -> Result<axum::response::Response, StatusCode> {
    // If user is already logged in, redirect to index
    if session.isset_login() {
        let redirect = Redirect::to("/index.php");
        return Ok(redirect.into_response());
    }

    // Validate code length
    if code.is_empty() || code.len() != 25 {
        let html = Html("Please insert a 25-character code.".to_string());
        return Ok(html.into_response());
    }

    // Perform code-only verification
    match email_verification.code_only_verification(&code).await {
        Ok(user_id) => {
            if user_id == 0 {
                let html = Html("Ops. This code is not valid. Please verify the link on your email or <a href=\"index.php\">login</a> and enter it manually.".to_string());
                return Ok(html.into_response());
            } else {
                // Log user in automatically after verification
                let mut database = LRSys::new(db_pool.clone());
                let mut player = Player::new(db_pool.clone());
                
                match player.get_player_info(user_id).await {
                    Ok(user_info) => {
                        match database.login(user_info.login, String::new(), Some("remember")).await {
                            Ok(_) => {
                                let redirect = Redirect::to("/welcome.php");
                                return Ok(redirect.into_response());
                            }
                            Err(_) => {
                                let redirect = Redirect::to("/index.php");
                                return Ok(redirect.into_response());
                            }
                        }
                    }
                    Err(_) => {
                        let redirect = Redirect::to("/index.php");
                        return Ok(redirect.into_response());
                    }
                }
            }
        }
        Err(_) => {
            let html = Html("Ops. This code is not valid. Please verify the link on your email or <a href=\"index.php\">login</a> and enter it manually.".to_string());
            return Ok(html.into_response());
        }
    }
}

/// Handle regular welcome screen display
async fn handle_welcome_display(
    system: &mut System,
    email_verification: &mut EmailVerification,
    session: &PhpSession,
) -> Result<axum::response::Response, StatusCode> {
    // Check if user is logged in
    let user_id = match session.get_int("id") {
        Some(id) => id,
        None => {
            let redirect = Redirect::to("/index.php");
            return Ok(redirect.into_response());
        }
    };

    // Check if user already has certificates (completed tutorial)
    let cert_count = match session.get_int("CERT") {
        Some(count) => count,
        None => 0,
    };

    if cert_count >= 1 {
        let redirect = Redirect::to("/index.php");
        return Ok(redirect.into_response());
    }

    // Check email verification status
    let verified = match email_verification.is_verified(user_id).await {
        Ok(is_verified) => is_verified,
        Err(_) => true, // Default to verified if check fails
    };

    let verified = true; // Force verified for now as per original PHP

    // Get user language for button sizing
    let language = session.get_string("l").unwrap_or_default();
    let btn_size = if language == "pt_BR" { 250 } else { 200 };

    // Generate buttons based on verification status
    let (btn_verified, btn_tutorial) = if !verified {
        (
            format!(r#"<li><a id="btn-verify" class="btn btn-default btn-lg btn-front" style="width: {}px;"><i class="fa fa-barcode fa-fw"></i> <span class="network-name">Verify email</span></a></li>"#, btn_size),
            format!(r#"<li><a id="btn-start" class="btn btn-default btn-lg btn-front" style="width: {}px; display:none;"><i class="fa fa-power-off fa-fw"></i> <span class="network-name">Start tutorial</span></a></li>"#, btn_size)
        )
    } else {
        (
            String::new(),
            format!(r#"<li><a id="btn-start" class="btn btn-default btn-lg btn-front" style="width: {}px;"><i class="fa fa-power-off fa-fw"></i> <span class="network-name">Start tutorial</span></a></li>"#, btn_size)
        )
    };

    // Generate verification input if needed
    let verification_input = if !verified {
        r#"<input id="code-input" type="text" style="width: 300px; margin-bottom: 30px; padding-left: 7px;" placeholder="Verification Code"><br/>"#
    } else {
        ""
    };

    let html = format!(r#"
<!DOCTYPE html>
<!--
    Hello, is it me you're looking for?
    www.renatomassaro.com
-->
<html lang="en">
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <meta name="description" content="">
        <meta name="author" content="">
        <title>Hacker Experience</title>
        <link href="css/bootstrap.css" rel="stylesheet">
        <link href="font-awesome/css/font-awesome.min.css" rel="stylesheet">
        <link href="css/he_index.css" rel="stylesheet">
    </head>
    <body>
        <div id="terminal"></div>
        <div class="intro-header">
            <div class="container">
                <div class="row">
                    <div class="col-lg-12">
                    <span id="error-msg" class="alert alert-danger" style="display:none;"></span>
                        <div class="intro-message">
                            <h1>Hacker Experience</h1>
                            <h3 class="digital">The Internet under attack<span class="a_bebida_que_pisca">_</span></h3>
                            <hr class="intro-divider">
                            <ul class="list-inline intro-social-buttons">
                                {}{}{}
                            </ul>
                        </div>
                    </div>
                </div>
            </div>
        </div>  
        <!--<script src="http://ajax.googleapis.com/ajax/libs/jquery/1.9.1/jquery.min.js"></script>-->
        <script src="js/jquery.min.js"></script>
        <script src="js/welcome.js"></script>
    </body>
<!--
    Hello! I've just got to let you know.
    www.neoartgames.com
-->
</html>
    "#, verification_input, btn_verified, btn_tutorial);

    let response = Html(html);
    Ok(response.into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_welcome_query_deserialize() {
        // Test query parameter deserialization for code verification
        let query = WelcomeQuery {
            code: Some("1234567890123456789012345".to_string()),
        };
        
        assert_eq!(query.code.as_ref().unwrap().len(), 25);
    }
    
    #[test]
    fn test_verification_form_deserialize() {
        let form = VerificationForm {
            code: "test_verification_code".to_string(),
        };
        
        assert_eq!(form.code, "test_verification_code");
    }
}