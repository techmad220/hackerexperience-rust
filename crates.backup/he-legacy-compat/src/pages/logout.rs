//! Logout page handler - 1:1 port of logout.php
//! 
//! Handles user logout with Facebook integration and time tracking updates.
//! Updates ranking time played, forum logout, and session termination.

use axum::{
    extract::Extension,
    response::Redirect,
    http::StatusCode,
};
use crate::classes::{ranking::Ranking, forum::Forum};
use crate::session::PhpSession;
use he_db::DbPool;

/// Main logout handler - terminates user session and updates statistics
/// 
/// Port of: logout.php
/// Behavior:
/// - Updates time played in ranking system
/// - Performs forum logout 
/// - Terminates PHP session
/// - Handles Facebook session destruction if Facebook login was used
/// - Always redirects to index.php after logout
pub async fn logout_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(mut session): Extension<PhpSession>,
) -> Result<Redirect, StatusCode> {
    // Initialize required classes
    let mut ranking = Ranking::new(db_pool.clone());
    let mut forum = Forum::new(db_pool.clone());

    // Update time played statistics
    if let Err(e) = ranking.update_time_played().await {
        eprintln!("Error updating time played: {:?}", e);
        // Continue with logout even if time tracking fails
    }

    // Perform forum logout
    if let Err(e) = forum.logout().await {
        eprintln!("Error during forum logout: {:?}", e);
        // Continue with logout even if forum logout fails
    }

    // Handle Facebook logout if user logged in with Facebook
    if session.isset_fb_login() {
        // Initialize Facebook class
        match facebook_logout(&mut session).await {
            Ok(_) => {
                println!("Facebook session destroyed successfully");
            },
            Err(e) => {
                eprintln!("Error destroying Facebook session: {:?}", e);
                // Continue with logout even if Facebook logout fails
            }
        }
    }

    // Perform main session logout
    session.logout();

    // Redirect to index.php
    Ok(Redirect::to("/index.php"))
}

/// Handle Facebook session destruction
/// 
/// Port of Facebook logout logic from logout.php
/// Note: Facebook SDK implementation would be needed for full functionality
async fn facebook_logout(session: &mut PhpSession) -> Result<(), FacebookLogoutError> {
    // TODO: Implement Facebook SDK integration
    // Original PHP code:
    // $facebook = new Facebook(array(
    //     'appId' => 'REDACTED',
    //     'secret' => 'REDACTED'
    // ));
    // $facebook->destroySession();
    
    // For now, just clear Facebook-related session data
    session.unset("fb_login");
    session.unset("fb_user_id");
    session.unset("fb_access_token");
    
    Ok(())
}

#[derive(Debug)]
pub enum FacebookLogoutError {
    SessionError,
    ApiError(String),
}

impl std::fmt::Display for FacebookLogoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FacebookLogoutError::SessionError => write!(f, "Facebook session error"),
            FacebookLogoutError::ApiError(msg) => write!(f, "Facebook API error: {}", msg),
        }
    }
}

impl std::error::Error for FacebookLogoutError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_facebook_logout() {
        let mut session = PhpSession::new("test_session");
        session.set("fb_login", crate::session::SessionValue::Boolean(true));
        
        let result = facebook_logout(&mut session).await;
        assert!(result.is_ok());
        assert!(!session.isset("fb_login"));
    }
}