//! Authentication handlers

use actix_web::{web, HttpResponse, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use he_database::queries::UserQueries;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub login: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub token: Option<String>,
    pub message: String,
}

pub async fn register(
    state: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
) -> HttpResponse {
    // Check if user exists
    let existing = UserQueries::get_user_by_email(&state.db.pool, &req.email).await;

    if let Ok(Some(_)) = existing {
        return HttpResponse::BadRequest().json(AuthResponse {
            success: false,
            token: None,
            message: "User already exists".to_string(),
        });
    }

    // Create user
    match UserQueries::create_user(&state.db.pool, &req.login, &req.email, &req.password).await {
        Ok(user) => {
            HttpResponse::Ok().json(AuthResponse {
                success: true,
                token: None,
                message: format!("User {} created successfully", user.login),
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(AuthResponse {
                success: false,
                token: None,
                message: format!("Registration failed: {}", e),
            })
        }
    }
}

pub async fn login(
    state: web::Data<AppState>,
    req: web::Json<LoginRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    // Get user
    let user = match UserQueries::get_user_by_email(&state.db.pool, &req.email).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(AuthResponse {
                success: false,
                token: None,
                message: "Invalid credentials".to_string(),
            });
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(AuthResponse {
                success: false,
                token: None,
                message: "Login failed".to_string(),
            });
        }
    };

    // Verify password
    let valid = UserQueries::verify_password(&user, &req.password).await.unwrap_or(false);

    if !valid {
        return HttpResponse::Unauthorized().json(AuthResponse {
            success: false,
            token: None,
            message: "Invalid credentials".to_string(),
        });
    }

    // Get client IP
    let client_ip = http_req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("127.0.0.1")
        .to_string();

    // Update last login
    let _ = UserQueries::update_last_login(&state.db.pool, user.id, &client_ip).await;

    // Generate token using auth service
    let auth_result = state.auth.authenticate(&req.email, &req.password, Some(client_ip)).await;

    match auth_result {
        Ok(he_auth::AuthenticationResult::Success { token, .. }) => {
            HttpResponse::Ok().json(AuthResponse {
                success: true,
                token: Some(token),
                message: "Login successful".to_string(),
            })
        }
        _ => {
            HttpResponse::Unauthorized().json(AuthResponse {
                success: false,
                token: None,
                message: "Authentication failed".to_string(),
            })
        }
    }
}

pub async fn logout(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> HttpResponse {
    // Extract token from header
    let token = match req.headers().get("Authorization") {
        Some(header) => {
            let value = header.to_str().unwrap_or("");
            value.strip_prefix("Bearer ").unwrap_or("").to_string()
        }
        None => {
            return HttpResponse::Unauthorized().json(AuthResponse {
                success: false,
                token: None,
                message: "No authorization token".to_string(),
            });
        }
    };

    // Validate token and get user
    if let Ok(Some(validated_user)) = state.auth.validate_token(&token).await {
        // Logout user (invalidate session)
        if let Some(session_id) = validated_user.session_id {
            let _ = state.auth.logout(&session_id).await;
        }

        HttpResponse::Ok().json(AuthResponse {
            success: true,
            token: None,
            message: "Logged out successfully".to_string(),
        })
    } else {
        HttpResponse::Unauthorized().json(AuthResponse {
            success: false,
            token: None,
            message: "Invalid token".to_string(),
        })
    }
}

pub async fn refresh_token(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> HttpResponse {
    let token = match req.headers().get("Authorization") {
        Some(header) => {
            let value = header.to_str().unwrap_or("");
            value.strip_prefix("Bearer ").unwrap_or("").to_string()
        }
        None => {
            return HttpResponse::Unauthorized().json(AuthResponse {
                success: false,
                token: None,
                message: "No authorization token".to_string(),
            });
        }
    };

    match state.auth.refresh_token(&token).await {
        Ok(Some(new_token)) => {
            HttpResponse::Ok().json(AuthResponse {
                success: true,
                token: Some(new_token),
                message: "Token refreshed".to_string(),
            })
        }
        _ => {
            HttpResponse::Unauthorized().json(AuthResponse {
                success: false,
                token: None,
                message: "Failed to refresh token".to_string(),
            })
        }
    }
}