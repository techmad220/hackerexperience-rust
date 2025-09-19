use crate::client::WebSocketClient;
use crate::events::WebSocketMessage;
use async_trait::async_trait;
use he_core::entities::User;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

/// Authentication provider trait for WebSocket connections
#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn authenticate_token(&self, token: &str) -> Result<User, AuthError>;
    async fn validate_session(&self, user_id: Uuid, session_id: &str) -> Result<bool, AuthError>;
    async fn can_access_topic(&self, user_id: Uuid, topic: &str) -> Result<bool, AuthError>;
}

/// JWT-based authentication for WebSocket connections
pub struct JwtAuthProvider {
    secret: String,
    // In a real implementation, you'd include JWT validation logic here
}

impl JwtAuthProvider {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }
}

#[async_trait]
impl AuthProvider for JwtAuthProvider {
    async fn authenticate_token(&self, token: &str) -> Result<User, AuthError> {
        // Validate JWT token using the auth module
        use he_auth::jwt::{JwtManager, JwtConfig};

        if token.is_empty() {
            return Err(AuthError::InvalidToken);
        }

        // Initialize JWT manager with the same secret
        let mut config = JwtConfig::default();
        config.secret = self.secret.clone();

        let jwt_manager = JwtManager::new(config)
            .map_err(|_| AuthError::InvalidToken)?;

        // Validate and extract claims
        let claims = jwt_manager.validate_token(token)
            .map_err(|_| AuthError::InvalidToken)?;

        // Return authenticated user
        Ok(User {
            id: claims.user_id,
            username: claims.email.split('@').next().unwrap_or("user").to_string(),
            email: claims.email,
            created_at: chrono::Utc::now(), // Would be fetched from database
            updated_at: chrono::Utc::now(), // Would be fetched from database
            // In production, fetch full user details from database using claims.user_id
        })
    }

    async fn validate_session(&self, user_id: Uuid, session_id: &str) -> Result<bool, AuthError> {
        // Validate session by checking if it's a valid UUID and belongs to user
        // In production, this would check against a session store/database

        // Parse session_id as UUID to ensure it's valid format
        Uuid::parse_str(session_id)
            .map_err(|_| AuthError::InvalidSession)?;

        // In production: Check database/Redis for active session
        // For now, we'll accept any valid UUID format for the given user
        Ok(true)
    }

    async fn can_access_topic(&self, user_id: Uuid, topic: &str) -> Result<bool, AuthError> {
        // Implement topic-based authorization
        match topic {
            // Public topics
            "lobby:global" | "system:announcements" => Ok(true),
            
            // User-specific topics
            topic if topic.starts_with("user:") => {
                let topic_user_id = topic.strip_prefix("user:")
                    .and_then(|id_str| Uuid::parse_str(id_str).ok());
                
                match topic_user_id {
                    Some(topic_user_id) => Ok(topic_user_id == user_id),
                    None => Ok(false),
                }
            }
            
            // Chat channels (implement based on your game rules)
            topic if topic.starts_with("chat:") => {
                // For now, allow all authenticated users to access chat
                Ok(true)
            }
            
            // Server-specific topics (check if user has access to the server)
            topic if topic.starts_with("server:") => {
                // TODO: Check if user has access to this server
                // This would involve checking server ownership/access in the database
                Ok(true)
            }
            
            // Process-specific topics (check if user owns the process)
            topic if topic.starts_with("process:") => {
                // TODO: Check if user owns this process
                Ok(true)
            }
            
            // Mission-specific topics
            topic if topic.starts_with("mission:") => {
                // TODO: Check if user is participating in this mission
                Ok(true)
            }
            
            // Network/faction topics
            topic if topic.starts_with("network:") => {
                // TODO: Check if user is member of this network
                Ok(true)
            }
            
            // Default: deny access to unknown topics
            _ => Ok(false),
        }
    }
}

/// Authentication message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthMessage {
    Authenticate { token: String },
    SessionValidate { session_id: String },
    RefreshToken { refresh_token: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub user_id: Option<Uuid>,
    pub error: Option<String>,
    pub permissions: Vec<String>,
}

/// Authentication middleware for WebSocket connections
pub struct WebSocketAuth {
    provider: Arc<dyn AuthProvider>,
}

impl WebSocketAuth {
    pub fn new(provider: Arc<dyn AuthProvider>) -> Self {
        Self { provider }
    }

    /// Authenticate a WebSocket client using a token
    pub async fn authenticate_client(
        &self,
        client: &mut WebSocketClient,
        token: &str,
    ) -> Result<AuthResponse, AuthError> {
        match self.provider.authenticate_token(token).await {
            Ok(user) => {
                client.authenticate(user.id).await;
                
                Ok(AuthResponse {
                    success: true,
                    user_id: Some(user.id),
                    error: None,
                    permissions: self.get_user_permissions(user.id).await,
                })
            }
            Err(e) => {
                Ok(AuthResponse {
                    success: false,
                    user_id: None,
                    error: Some(e.to_string()),
                    permissions: vec![],
                })
            }
        }
    }

    /// Check if a client can access a specific topic
    pub async fn can_access_topic(
        &self,
        client: &WebSocketClient,
        topic: &str,
    ) -> Result<bool, AuthError> {
        if !client.authenticated {
            // Allow access to public topics for unauthenticated users
            return Ok(self.is_public_topic(topic));
        }

        if let Some(user_id) = client.user_id {
            self.provider.can_access_topic(user_id, topic).await
        } else {
            Ok(false)
        }
    }

    /// Handle authentication messages
    pub async fn handle_auth_message(
        &self,
        client: &mut WebSocketClient,
        message: AuthMessage,
    ) -> Result<AuthResponse, AuthError> {
        match message {
            AuthMessage::Authenticate { token } => {
                self.authenticate_client(client, &token).await
            }
            AuthMessage::SessionValidate { session_id } => {
                if let Some(user_id) = client.user_id {
                    let is_valid = self.provider.validate_session(user_id, &session_id).await?;
                    
                    if is_valid {
                        Ok(AuthResponse {
                            success: true,
                            user_id: Some(user_id),
                            error: None,
                            permissions: self.get_user_permissions(user_id).await,
                        })
                    } else {
                        Ok(AuthResponse {
                            success: false,
                            user_id: None,
                            error: Some("Invalid session".to_string()),
                            permissions: vec![],
                        })
                    }
                } else {
                    Ok(AuthResponse {
                        success: false,
                        user_id: None,
                        error: Some("Not authenticated".to_string()),
                        permissions: vec![],
                    })
                }
            }
            AuthMessage::RefreshToken { .. } => {
                // TODO: Implement token refresh logic
                Err(AuthError::NotImplemented)
            }
        }
    }

    /// Check if a topic is publicly accessible
    fn is_public_topic(&self, topic: &str) -> bool {
        matches!(topic, "lobby:global" | "system:announcements")
    }

    /// Get user permissions (placeholder implementation)
    async fn get_user_permissions(&self, _user_id: Uuid) -> Vec<String> {
        // TODO: Implement actual permission system
        vec![
            "read:public".to_string(),
            "write:public".to_string(),
            "access:user_channels".to_string(),
        ]
    }
}

/// Authentication-related errors
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid token")]
    InvalidToken,
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Invalid session")]
    InvalidSession,
    
    #[error("Permission denied")]
    PermissionDenied,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Authentication required")]
    AuthenticationRequired,
    
    #[error("Not implemented")]
    NotImplemented,
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Middleware for topic access control
pub struct TopicAccessControl {
    auth: Arc<WebSocketAuth>,
}

impl TopicAccessControl {
    pub fn new(auth: Arc<WebSocketAuth>) -> Self {
        Self { auth }
    }

    /// Check if client can join a topic
    pub async fn can_join_topic(
        &self,
        client: &WebSocketClient,
        topic: &str,
    ) -> Result<bool, AuthError> {
        self.auth.can_access_topic(client, topic).await
    }

    /// Check if client can send to a topic
    pub async fn can_send_to_topic(
        &self,
        client: &WebSocketClient,
        topic: &str,
    ) -> Result<bool, AuthError> {
        // For now, use the same logic as joining
        // In the future, you might want different permissions for reading vs writing
        self.auth.can_access_topic(client, topic).await
    }

    /// Get allowed topics for a user
    pub async fn get_allowed_topics(&self, user_id: Uuid) -> Vec<String> {
        // TODO: Implement based on user roles and permissions
        vec![
            "lobby:global".to_string(),
            "system:announcements".to_string(),
            format!("user:{}", user_id),
            "chat:general".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_jwt_auth_provider() {
        let provider = JwtAuthProvider::new("secret".to_string());
        
        // Test invalid token
        let result = provider.authenticate_token("").await;
        assert!(result.is_err());
        
        // Test valid token (mock)
        let result = provider.authenticate_token("valid_token").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_topic_access() {
        let provider = Arc::new(JwtAuthProvider::new("secret".to_string()));
        let auth = WebSocketAuth::new(provider);
        
        let user_id = Uuid::new_v4();
        
        // Test public topic access
        let can_access = auth.provider.can_access_topic(user_id, "lobby:global").await.unwrap();
        assert!(can_access);
        
        // Test user-specific topic access
        let user_topic = format!("user:{}", user_id);
        let can_access = auth.provider.can_access_topic(user_id, &user_topic).await.unwrap();
        assert!(can_access);
        
        // Test access to other user's topic
        let other_user_id = Uuid::new_v4();
        let other_topic = format!("user:{}", other_user_id);
        let can_access = auth.provider.can_access_topic(user_id, &other_topic).await.unwrap();
        assert!(!can_access);
    }

    #[tokio::test]
    async fn test_websocket_auth() {
        let provider = Arc::new(JwtAuthProvider::new("secret".to_string()));
        let auth = WebSocketAuth::new(provider);
        
        let (sender, _receiver) = mpsc::unbounded_channel();
        let mut client = WebSocketClient::new(Uuid::new_v4(), sender);
        
        // Test authentication
        let response = auth.authenticate_client(&mut client, "valid_token").await.unwrap();
        assert!(response.success);
        assert!(response.user_id.is_some());
        assert!(client.authenticated);
    }
}