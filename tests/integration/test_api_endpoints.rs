use axum::{
    body::Body,
    extract::{Form, Query},
    http::{Method, Request, StatusCode, header},
    response::Response,
    routing::{get, post},
    Extension, Json, Router,
};
use tower::ServiceExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

use crate::common::{TestDb, TestFixtures, MockHttpClient, assert_json_contains};
use crate::{assert_ok, assert_err};

// ===== API ENDPOINT TESTING =====

// Test API request/response structures
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub errors: Option<Vec<String>>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            errors: None,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message.to_string()),
            errors: None,
        }
    }

    pub fn validation_errors(errors: Vec<String>) -> Self {
        Self {
            success: false,
            data: None,
            message: Some("Validation failed".to_string()),
            errors: Some(errors),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub created_at: String,
    pub last_login: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerResponse {
    pub id: u64,
    pub player_id: u64,
    pub ip: String,
    pub name: String,
    pub server_type: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessResponse {
    pub id: u64,
    pub process_type: String,
    pub status: String,
    pub progress: f32,
    pub target: Option<String>,
    pub started_at: String,
    pub completion_time: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct StartProcessRequest {
    pub process_type: String,
    pub target: Option<String>,
    pub software_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ServerConnectionRequest {
    pub target_ip: String,
    pub connection_type: String,
}

// Mock API handlers for testing
async fn get_user_handler(user_id: axum::extract::Path<u64>) -> Result<Json<ApiResponse<UserResponse>>, StatusCode> {
    if user_id.0 == 999 {
        return Ok(Json(ApiResponse::error("User not found")));
    }

    let user = UserResponse {
        id: user_id.0,
        username: format!("user{}", user_id.0),
        email: format!("user{}@example.com", user_id.0),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        last_login: Some("2024-01-01T12:00:00Z".to_string()),
    };

    Ok(Json(ApiResponse::success(user)))
}

async fn create_user_handler(Json(request): Json<CreateUserRequest>) -> Result<Json<ApiResponse<UserResponse>>, StatusCode> {
    // Validation
    let mut errors = Vec::new();
    
    if request.username.is_empty() {
        errors.push("Username is required".to_string());
    }
    
    if request.username.len() < 3 {
        errors.push("Username must be at least 3 characters".to_string());
    }
    
    if !request.email.contains('@') {
        errors.push("Valid email is required".to_string());
    }
    
    if request.password.len() < 6 {
        errors.push("Password must be at least 6 characters".to_string());
    }

    if !errors.is_empty() {
        return Ok(Json(ApiResponse::validation_errors(errors)));
    }

    // Simulate user creation
    let user = UserResponse {
        id: 123,
        username: request.username,
        email: request.email,
        created_at: "2024-01-01T00:00:00Z".to_string(),
        last_login: None,
    };

    Ok(Json(ApiResponse::success(user)))
}

async fn login_handler(Json(request): Json<LoginRequest>) -> Result<Json<ApiResponse<String>>, StatusCode> {
    if request.username == "admin" && request.password == "admin123" {
        let session_token = Uuid::new_v4().to_string();
        Ok(Json(ApiResponse::success(session_token)))
    } else {
        Ok(Json(ApiResponse::error("Invalid credentials")))
    }
}

async fn get_user_servers_handler(
    user_id: axum::extract::Path<u64>,
    _auth: AuthGuard,
) -> Result<Json<ApiResponse<Vec<ServerResponse>>>, StatusCode> {
    let servers = vec![
        ServerResponse {
            id: 1,
            player_id: user_id.0,
            ip: "192.168.1.100".to_string(),
            name: "My Server".to_string(),
            server_type: "Desktop".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        },
        ServerResponse {
            id: 2,
            player_id: user_id.0,
            ip: "10.0.0.5".to_string(),
            name: "Remote Server".to_string(),
            server_type: "Server".to_string(),
            created_at: "2024-01-02T00:00:00Z".to_string(),
        }
    ];

    Ok(Json(ApiResponse::success(servers)))
}

async fn start_process_handler(
    Json(request): Json<StartProcessRequest>,
    _auth: AuthGuard,
) -> Result<Json<ApiResponse<ProcessResponse>>, StatusCode> {
    let valid_process_types = ["cracker", "uploader", "downloader", "virus", "hasher"];
    
    if !valid_process_types.contains(&request.process_type.as_str()) {
        return Ok(Json(ApiResponse::error("Invalid process type")));
    }

    let process = ProcessResponse {
        id: 456,
        process_type: request.process_type,
        status: "running".to_string(),
        progress: 0.0,
        target: request.target,
        started_at: "2024-01-01T12:00:00Z".to_string(),
        completion_time: Some(300), // 5 minutes
    };

    Ok(Json(ApiResponse::success(process)))
}

async fn get_user_processes_handler(
    user_id: axum::extract::Path<u64>,
    _auth: AuthGuard,
) -> Result<Json<ApiResponse<Vec<ProcessResponse>>>, StatusCode> {
    let processes = vec![
        ProcessResponse {
            id: 1,
            process_type: "cracker".to_string(),
            status: "running".to_string(),
            progress: 65.5,
            target: Some("192.168.1.101".to_string()),
            started_at: "2024-01-01T12:00:00Z".to_string(),
            completion_time: Some(300),
        },
        ProcessResponse {
            id: 2,
            process_type: "uploader".to_string(),
            status: "completed".to_string(),
            progress: 100.0,
            target: Some("my_virus.exe".to_string()),
            started_at: "2024-01-01T11:30:00Z".to_string(),
            completion_time: Some(120),
        }
    ];

    Ok(Json(ApiResponse::success(processes)))
}

async fn connect_to_server_handler(
    Json(request): Json<ServerConnectionRequest>,
    _auth: AuthGuard,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // Validate IP format
    if !request.target_ip.contains('.') {
        return Ok(Json(ApiResponse::error("Invalid IP address format")));
    }

    let valid_connection_types = ["ssh", "ftp", "telnet"];
    if !valid_connection_types.contains(&request.connection_type.as_str()) {
        return Ok(Json(ApiResponse::error("Invalid connection type")));
    }

    // Simulate connection result
    let connection_id = Uuid::new_v4().to_string();
    Ok(Json(ApiResponse::success(connection_id)))
}

async fn health_check_handler() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0"
    }))
}

// Authentication guard for protected endpoints
#[derive(Debug)]
pub struct AuthGuard {
    pub user_id: u64,
}

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AuthGuard
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        if let Some(auth_header) = parts.headers.get("Authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    // In a real implementation, validate the JWT token
                    return Ok(AuthGuard { user_id: 1 });
                }
            }
        }
        Err(StatusCode::UNAUTHORIZED)
    }
}

// Create test application
fn create_test_app() -> Router {
    Router::new()
        .route("/health", get(health_check_handler))
        .route("/users", post(create_user_handler))
        .route("/users/:id", get(get_user_handler))
        .route("/users/:id/servers", get(get_user_servers_handler))
        .route("/users/:id/processes", get(get_user_processes_handler))
        .route("/auth/login", post(login_handler))
        .route("/processes", post(start_process_handler))
        .route("/servers/connect", post(connect_to_server_handler))
}

// Helper function to make authenticated requests
fn authenticated_request(method: Method, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("Authorization", "Bearer test_token")
        .header("Content-Type", "application/json")
        .body(Body::empty())
        .unwrap()
}

fn json_request(method: Method, uri: &str, body: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap()
}

fn authenticated_json_request(method: Method, uri: &str, body: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("Authorization", "Bearer test_token")
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap()
}

// ===== INTEGRATION TESTS =====

#[tokio::test]
async fn test_health_check_endpoint() {
    let app = create_test_app();

    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["status"], "healthy");
    assert!(json.get("timestamp").is_some());
    assert_eq!(json["version"], "1.0.0");
}

#[tokio::test]
async fn test_user_creation_endpoint() {
    let app = create_test_app();

    // Test valid user creation
    let request_body = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123"
    });

    let request = json_request(Method::POST, "/users", &request_body.to_string());
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let api_response: ApiResponse<UserResponse> = serde_json::from_slice(&body).unwrap();
    
    assert!(api_response.success);
    let user = api_response.data.unwrap();
    assert_eq!(user.username, "testuser");
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.id, 123);

    // Test validation errors
    let request_body = json!({
        "username": "x", // Too short
        "email": "invalid-email", // Invalid format
        "password": "123" // Too short
    });

    let request = json_request(Method::POST, "/users", &request_body.to_string());
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let api_response: ApiResponse<UserResponse> = serde_json::from_slice(&body).unwrap();
    
    assert!(!api_response.success);
    assert!(api_response.errors.is_some());
    let errors = api_response.errors.unwrap();
    assert!(errors.len() >= 3); // Should have multiple validation errors
}

#[tokio::test]
async fn test_user_retrieval_endpoint() {
    let app = create_test_app();

    // Test existing user
    let request = Request::builder()
        .uri("/users/123")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let api_response: ApiResponse<UserResponse> = serde_json::from_slice(&body).unwrap();
    
    assert!(api_response.success);
    let user = api_response.data.unwrap();
    assert_eq!(user.id, 123);
    assert_eq!(user.username, "user123");

    // Test non-existent user
    let request = Request::builder()
        .uri("/users/999")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let api_response: ApiResponse<UserResponse> = serde_json::from_slice(&body).unwrap();
    
    assert!(!api_response.success);
    assert_eq!(api_response.message.unwrap(), "User not found");
}

#[tokio::test]
async fn test_authentication_endpoint() {
    let app = create_test_app();

    // Test successful login
    let request_body = json!({
        "username": "admin",
        "password": "admin123"
    });

    let request = json_request(Method::POST, "/auth/login", &request_body.to_string());
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let api_response: ApiResponse<String> = serde_json::from_slice(&body).unwrap();
    
    assert!(api_response.success);
    let token = api_response.data.unwrap();
    assert!(!token.is_empty());

    // Test failed login
    let request_body = json!({
        "username": "admin",
        "password": "wrongpassword"
    });

    let request = json_request(Method::POST, "/auth/login", &request_body.to_string());
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let api_response: ApiResponse<String> = serde_json::from_slice(&body).unwrap();
    
    assert!(!api_response.success);
    assert_eq!(api_response.message.unwrap(), "Invalid credentials");
}

#[tokio::test]
async fn test_protected_endpoint_authentication() {
    let app = create_test_app();

    // Test accessing protected endpoint without authentication
    let request = Request::builder()
        .uri("/users/123/servers")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Test accessing protected endpoint with authentication
    let request = authenticated_request(Method::GET, "/users/123/servers");
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let api_response: ApiResponse<Vec<ServerResponse>> = serde_json::from_slice(&body).unwrap();
    
    assert!(api_response.success);
    let servers = api_response.data.unwrap();
    assert_eq!(servers.len(), 2);
    assert_eq!(servers[0].ip, "192.168.1.100");
    assert_eq!(servers[1].ip, "10.0.0.5");
}

#[tokio::test]
async fn test_process_management_endpoints() {
    let app = create_test_app();

    // Test starting a process
    let request_body = json!({
        "process_type": "cracker",
        "target": "192.168.1.101",
        "software_id": 123
    });

    let request = authenticated_json_request(Method::POST, "/processes", &request_body.to_string());
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let api_response: ApiResponse<ProcessResponse> = serde_json::from_slice(&body).unwrap();
    
    assert!(api_response.success);
    let process = api_response.data.unwrap();
    assert_eq!(process.process_type, "cracker");
    assert_eq!(process.status, "running");
    assert_eq!(process.target, Some("192.168.1.101".to_string()));

    // Test invalid process type
    let request_body = json!({
        "process_type": "invalid_type",
        "target": "192.168.1.101"
    });

    let request = authenticated_json_request(Method::POST, "/processes", &request_body.to_string());
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let api_response: ApiResponse<ProcessResponse> = serde_json::from_slice(&body).unwrap();
    
    assert!(!api_response.success);
    assert_eq!(api_response.message.unwrap(), "Invalid process type");

    // Test getting user processes
    let request = authenticated_request(Method::GET, "/users/123/processes");
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let api_response: ApiResponse<Vec<ProcessResponse>> = serde_json::from_slice(&body).unwrap();
    
    assert!(api_response.success);
    let processes = api_response.data.unwrap();
    assert_eq!(processes.len(), 2);
    assert_eq!(processes[0].process_type, "cracker");
    assert_eq!(processes[1].process_type, "uploader");
    assert_eq!(processes[1].status, "completed");
}

#[tokio::test]
async fn test_server_connection_endpoint() {
    let app = create_test_app();

    // Test valid server connection
    let request_body = json!({
        "target_ip": "192.168.1.100",
        "connection_type": "ssh"
    });

    let request = authenticated_json_request(Method::POST, "/servers/connect", &request_body.to_string());
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let api_response: ApiResponse<String> = serde_json::from_slice(&body).unwrap();
    
    assert!(api_response.success);
    let connection_id = api_response.data.unwrap();
    assert!(!connection_id.is_empty());

    // Test invalid IP format
    let request_body = json!({
        "target_ip": "invalid_ip",
        "connection_type": "ssh"
    });

    let request = authenticated_json_request(Method::POST, "/servers/connect", &request_body.to_string());
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let api_response: ApiResponse<String> = serde_json::from_slice(&body).unwrap();
    
    assert!(!api_response.success);
    assert_eq!(api_response.message.unwrap(), "Invalid IP address format");

    // Test invalid connection type
    let request_body = json!({
        "target_ip": "192.168.1.100",
        "connection_type": "invalid_type"
    });

    let request = authenticated_json_request(Method::POST, "/servers/connect", &request_body.to_string());
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let api_response: ApiResponse<String> = serde_json::from_slice(&body).unwrap();
    
    assert!(!api_response.success);
    assert_eq!(api_response.message.unwrap(), "Invalid connection type");
}

#[tokio::test]
async fn test_api_error_handling() {
    let app = create_test_app();

    // Test malformed JSON request
    let request = Request::builder()
        .method(Method::POST)
        .uri("/users")
        .header("Content-Type", "application/json")
        .body(Body::from("{invalid json"))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Test missing content-type header
    let request = Request::builder()
        .method(Method::POST)
        .uri("/users")
        .body(Body::from("{}"))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    // Should handle missing content-type gracefully
    assert!(response.status().is_client_error() || response.status().is_server_error());

    // Test non-existent endpoint
    let request = Request::builder()
        .uri("/nonexistent")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_api_content_type_handling() {
    let app = create_test_app();

    // Test JSON content type
    let request_body = json!({
        "username": "testuser",
        "email": "test@example.com", 
        "password": "password123"
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/users")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Response should have JSON content type
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());
}

#[tokio::test]
async fn test_concurrent_api_requests() {
    let app = create_test_app();

    // Create multiple concurrent requests
    let requests: Vec<_> = (0..10).map(|i| {
        let app = app.clone();
        async move {
            let request = Request::builder()
                .uri(&format!("/users/{}", i + 1))
                .body(Body::empty())
                .unwrap();
            app.oneshot(request).await
        }
    }).collect();

    let responses = futures::future::join_all(requests).await;

    // All requests should complete successfully
    for response in responses {
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}

#[tokio::test]
async fn test_api_request_validation() {
    let app = create_test_app();

    // Test request with missing required fields
    let request_body = json!({
        "username": "test"
        // Missing email and password
    });

    let request = json_request(Method::POST, "/users", &request_body.to_string());
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Test request with extra fields (should be ignored)
    let request_body = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123",
        "extra_field": "should_be_ignored"
    });

    let request = json_request(Method::POST, "/users", &request_body.to_string());
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let api_response: ApiResponse<UserResponse> = serde_json::from_slice(&body).unwrap();
    assert!(api_response.success);
}

#[tokio::test]
async fn test_api_rate_limiting() {
    // This would test rate limiting if implemented
    let app = create_test_app();

    // Make many requests rapidly
    let mut responses = Vec::new();
    for _ in 0..50 {
        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        responses.push(response.status());
    }

    // In a real rate-limited API, some requests would return 429 Too Many Requests
    // For this test, we just verify all requests complete
    for status in responses {
        assert!(status.is_success() || status == StatusCode::TOO_MANY_REQUESTS);
    }
}

// End-to-end API workflow test
#[tokio::test]
async fn test_complete_user_workflow() {
    let app = create_test_app();

    // 1. Create user
    let request_body = json!({
        "username": "workflowuser",
        "email": "workflow@example.com",
        "password": "password123"
    });

    let request = json_request(Method::POST, "/users", &request_body.to_string());
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 2. Login
    let request_body = json!({
        "username": "admin", // Using admin for successful login
        "password": "admin123"
    });

    let request = json_request(Method::POST, "/auth/login", &request_body.to_string());
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 3. Get user servers
    let request = authenticated_request(Method::GET, "/users/123/servers");
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 4. Start a process
    let request_body = json!({
        "process_type": "cracker",
        "target": "192.168.1.101"
    });

    let request = authenticated_json_request(Method::POST, "/processes", &request_body.to_string());
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 5. Check process status
    let request = authenticated_request(Method::GET, "/users/123/processes");
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let api_response: ApiResponse<Vec<ProcessResponse>> = serde_json::from_slice(&body).unwrap();
    
    assert!(api_response.success);
    let processes = api_response.data.unwrap();
    assert!(!processes.is_empty());
}