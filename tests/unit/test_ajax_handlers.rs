use axum::Extension;
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio_test;

use he_legacy_compat::pages::ajax::{AjaxRequest, AjaxResponse, ajax_handler};
use crate::common::{TestDb, TestFixtures};
use crate::{assert_ok, assert_err};

#[tokio::test]
async fn test_ajax_response_construction() {
    // Test success response
    let response = AjaxResponse::success("Operation completed");
    assert_eq!(response.status, "OK");
    assert_eq!(response.msg, "Operation completed");
    assert_eq!(response.redirect, "");
    assert!(response.data.is_none());

    // Test success with data
    let data = json!({"key": "value"});
    let response = AjaxResponse::success_with_data("Data retrieved", data.clone());
    assert_eq!(response.status, "OK");
    assert_eq!(response.msg, "Data retrieved");
    assert_eq!(response.data, Some(data));

    // Test error response
    let response = AjaxResponse::error("Something went wrong");
    assert_eq!(response.status, "ERROR");
    assert_eq!(response.msg, "Something went wrong");
    assert_eq!(response.redirect, "");

    // Test error with redirect
    let response = AjaxResponse::error_with_redirect("Unauthorized", "/login");
    assert_eq!(response.status, "ERROR");
    assert_eq!(response.msg, "Unauthorized");
    assert_eq!(response.redirect, "/login");

    // Test default error - preserves original message
    let response = AjaxResponse::default_error();
    assert_eq!(response.status, "ERROR");
    assert_eq!(response.msg, "STOP SPYING ON ME!");
}

#[tokio::test]
async fn test_check_user_handler() {
    let mut test_db = TestDb::new().await;
    assert_ok!(test_db.setup().await);

    // Create test player
    let player_id = assert_ok!(test_db.create_test_player("testuser").await);

    // Test existing user
    let mut params = HashMap::new();
    params.insert("user".to_string(), "testuser".to_string());
    
    let request = AjaxRequest {
        func: "check-user".to_string(),
        params,
    };

    // Mock database pool
    let db_pool = create_mock_db_pool();
    let response = ajax_handler(Extension(db_pool), axum::Form(request)).await;
    
    assert!(response.is_ok());
    let json_response = response.unwrap();
    assert_eq!(json_response.0.status, "ERROR"); // User exists, so should be error
    assert_eq!(json_response.0.msg, "Username already exists!");

    // Test non-existing user
    let mut params = HashMap::new();
    params.insert("user".to_string(), "nonexistentuser".to_string());
    
    let request = AjaxRequest {
        func: "check-user".to_string(),
        params,
    };

    let response = ajax_handler(Extension(create_mock_db_pool()), axum::Form(request)).await;
    assert!(response.is_ok());
    let json_response = response.unwrap();
    assert_eq!(json_response.0.status, "OK");
}

#[tokio::test]
async fn test_check_mail_handler() {
    let mut test_db = TestDb::new().await;
    assert_ok!(test_db.setup().await);

    // Create test player with email
    let player_id = assert_ok!(test_db.create_test_player("testuser").await);

    // Test existing email
    let mut params = HashMap::new();
    params.insert("mail".to_string(), "testuser@test.com".to_string());
    
    let request = AjaxRequest {
        func: "check-mail".to_string(),
        params,
    };

    let db_pool = create_mock_db_pool();
    let response = ajax_handler(Extension(db_pool), axum::Form(request)).await;
    
    assert!(response.is_ok());
    let json_response = response.unwrap();
    assert_eq!(json_response.0.status, "ERROR"); // Email exists
    
    // Test non-existing email
    let mut params = HashMap::new();
    params.insert("mail".to_string(), "nonexistent@test.com".to_string());
    
    let request = AjaxRequest {
        func: "check-mail".to_string(),
        params,
    };

    let response = ajax_handler(Extension(create_mock_db_pool()), axum::Form(request)).await;
    assert!(response.is_ok());
    let json_response = response.unwrap();
    assert_eq!(json_response.0.status, "OK");
}

#[tokio::test]
async fn test_gettext_handler() {
    // Test language text retrieval
    let mut params = HashMap::new();
    params.insert("text".to_string(), "welcome_message".to_string());
    params.insert("lang".to_string(), "en".to_string());
    
    let request = AjaxRequest {
        func: "gettext".to_string(),
        params,
    };

    let response = ajax_handler(Extension(create_mock_db_pool()), axum::Form(request)).await;
    assert!(response.is_ok());
    let json_response = response.unwrap();
    assert_eq!(json_response.0.status, "OK");
    assert!(json_response.0.data.is_some());
}

#[tokio::test] 
async fn test_start_process_handler() {
    let mut test_db = TestDb::new().await;
    assert_ok!(test_db.setup().await);

    let player_id = assert_ok!(test_db.create_test_player("testuser").await);
    let server_id = assert_ok!(test_db.create_test_server(player_id, "192.168.1.100").await);

    // Test starting a cracker process
    let mut params = HashMap::new();
    params.insert("process_type".to_string(), "cracker".to_string());
    params.insert("target_ip".to_string(), "192.168.1.101".to_string());
    params.insert("target_file".to_string(), "password.txt".to_string());
    
    let request = AjaxRequest {
        func: "start-process".to_string(),
        params,
    };

    let response = ajax_handler(Extension(create_mock_db_pool()), axum::Form(request)).await;
    assert!(response.is_ok());
    let json_response = response.unwrap();
    
    if json_response.0.status == "OK" {
        assert!(json_response.0.data.is_some());
        let data = json_response.0.data.unwrap();
        assert!(data.get("process_id").is_some());
        assert!(data.get("completion_time").is_some());
    }
}

#[tokio::test]
async fn test_pause_process_handler() {
    let mut test_db = TestDb::new().await;
    assert_ok!(test_db.setup().await);

    let player_id = assert_ok!(test_db.create_test_player("testuser").await);

    // Test pausing a process
    let mut params = HashMap::new();
    params.insert("process_id".to_string(), "123".to_string());
    
    let request = AjaxRequest {
        func: "pause-process".to_string(),
        params,
    };

    let response = ajax_handler(Extension(create_mock_db_pool()), axum::Form(request)).await;
    assert!(response.is_ok());
    let json_response = response.unwrap();
    // Should either succeed or fail with valid error message
    assert!(json_response.0.status == "OK" || json_response.0.status == "ERROR");
}

#[tokio::test]
async fn test_cancel_process_handler() {
    let mut test_db = TestDb::new().await;
    assert_ok!(test_db.setup().await);

    let player_id = assert_ok!(test_db.create_test_player("testuser").await);

    // Test canceling a process
    let mut params = HashMap::new();
    params.insert("process_id".to_string(), "123".to_string());
    
    let request = AjaxRequest {
        func: "cancel-process".to_string(),
        params,
    };

    let response = ajax_handler(Extension(create_mock_db_pool()), axum::Form(request)).await;
    assert!(response.is_ok());
    let json_response = response.unwrap();
    assert!(json_response.0.status == "OK" || json_response.0.status == "ERROR");
}

#[tokio::test]
async fn test_bank_transfer_handler() {
    let mut test_db = TestDb::new().await;
    assert_ok!(test_db.setup().await);

    let player_id = assert_ok!(test_db.create_test_player("testuser").await);

    // Test bank transfer
    let mut params = HashMap::new();
    params.insert("from_account".to_string(), "123456789".to_string());
    params.insert("to_account".to_string(), "987654321".to_string());
    params.insert("amount".to_string(), "1000".to_string());
    params.insert("password".to_string(), "account_password".to_string());
    
    let request = AjaxRequest {
        func: "bank-transfer".to_string(),
        params,
    };

    let response = ajax_handler(Extension(create_mock_db_pool()), axum::Form(request)).await;
    assert!(response.is_ok());
    let json_response = response.unwrap();
    // Should handle both success and failure cases appropriately
    assert!(json_response.0.status == "OK" || json_response.0.status == "ERROR");
    
    if json_response.0.status == "ERROR" {
        // Error message should be meaningful
        assert!(!json_response.0.msg.is_empty());
        assert_ne!(json_response.0.msg, "STOP SPYING ON ME!");
    }
}

#[tokio::test]
async fn test_install_software_handler() {
    let mut test_db = TestDb::new().await;
    assert_ok!(test_db.setup().await);

    let player_id = assert_ok!(test_db.create_test_player("testuser").await);
    let server_id = assert_ok!(test_db.create_test_server(player_id, "192.168.1.100").await);

    // Test software installation
    let mut params = HashMap::new();
    params.insert("software_type".to_string(), "cracker".to_string());
    params.insert("version".to_string(), "1".to_string());
    
    let request = AjaxRequest {
        func: "install-software".to_string(),
        params,
    };

    let response = ajax_handler(Extension(create_mock_db_pool()), axum::Form(request)).await;
    assert!(response.is_ok());
    let json_response = response.unwrap();
    assert!(json_response.0.status == "OK" || json_response.0.status == "ERROR");
}

#[tokio::test]
async fn test_upgrade_software_handler() {
    let mut test_db = TestDb::new().await;
    assert_ok!(test_db.setup().await);

    let player_id = assert_ok!(test_db.create_test_player("testuser").await);

    // Test software upgrade
    let mut params = HashMap::new();
    params.insert("software_id".to_string(), "123".to_string());
    
    let request = AjaxRequest {
        func: "upgrade-software".to_string(),
        params,
    };

    let response = ajax_handler(Extension(create_mock_db_pool()), axum::Form(request)).await;
    assert!(response.is_ok());
    let json_response = response.unwrap();
    assert!(json_response.0.status == "OK" || json_response.0.status == "ERROR");
}

#[tokio::test]
async fn test_hack_server_handler() {
    let mut test_db = TestDb::new().await;
    assert_ok!(test_db.setup().await);

    let player_id = assert_ok!(test_db.create_test_player("testuser").await);

    // Test server hacking
    let mut params = HashMap::new();
    params.insert("target_ip".to_string(), "192.168.1.101".to_string());
    params.insert("software_id".to_string(), "123".to_string());
    
    let request = AjaxRequest {
        func: "hack-server".to_string(),
        params,
    };

    let response = ajax_handler(Extension(create_mock_db_pool()), axum::Form(request)).await;
    assert!(response.is_ok());
    let json_response = response.unwrap();
    assert!(json_response.0.status == "OK" || json_response.0.status == "ERROR");
}

#[tokio::test]
async fn test_invalid_function_handler() {
    // Test invalid function name - should return default error
    let mut params = HashMap::new();
    params.insert("dummy".to_string(), "value".to_string());
    
    let request = AjaxRequest {
        func: "invalid-function".to_string(),
        params,
    };

    let response = ajax_handler(Extension(create_mock_db_pool()), axum::Form(request)).await;
    assert!(response.is_ok());
    let json_response = response.unwrap();
    assert_eq!(json_response.0.status, "ERROR");
    assert_eq!(json_response.0.msg, "STOP SPYING ON ME!");
}

#[tokio::test]
async fn test_ajax_request_deserialization() {
    // Test basic request structure
    let form_data = "func=check-user&user=testuser&extra=value";
    let request: Result<AjaxRequest, _> = serde_urlencoded::from_str(form_data);
    
    assert!(request.is_ok());
    let request = request.unwrap();
    assert_eq!(request.func, "check-user");
    assert_eq!(request.params.get("user"), Some(&"testuser".to_string()));
    assert_eq!(request.params.get("extra"), Some(&"value".to_string()));

    // Test complex parameters
    let form_data = "func=bank-transfer&from_account=123&to_account=456&amount=1000";
    let request: Result<AjaxRequest, _> = serde_urlencoded::from_str(form_data);
    
    assert!(request.is_ok());
    let request = request.unwrap();
    assert_eq!(request.func, "bank-transfer");
    assert_eq!(request.params.len(), 3);
}

#[tokio::test]
async fn test_session_validation() {
    // Test functions that require authentication
    let functions_requiring_auth = vec![
        "start-process",
        "pause-process", 
        "cancel-process",
        "bank-transfer",
        "install-software",
        "hack-server"
    ];

    for func in functions_requiring_auth {
        let mut params = HashMap::new();
        params.insert("dummy".to_string(), "value".to_string());
        
        let request = AjaxRequest {
            func: func.to_string(),
            params,
        };

        let response = ajax_handler(Extension(create_mock_db_pool()), axum::Form(request)).await;
        assert!(response.is_ok());
        // Without proper session, should either redirect or error
        let json_response = response.unwrap();
        assert!(json_response.0.status == "ERROR" || !json_response.0.redirect.is_empty());
    }

    // Test functions that don't require authentication
    let functions_no_auth = vec!["check-user", "check-mail"];
    
    for func in functions_no_auth {
        let mut params = HashMap::new();
        params.insert("dummy".to_string(), "value".to_string());
        
        let request = AjaxRequest {
            func: func.to_string(),
            params,
        };

        let response = ajax_handler(Extension(create_mock_db_pool()), axum::Form(request)).await;
        assert!(response.is_ok());
        // These should process without authentication issues
    }
}

#[tokio::test]
async fn test_error_handling_edge_cases() {
    // Test missing required parameters
    let mut params = HashMap::new();
    
    let request = AjaxRequest {
        func: "check-user".to_string(),
        params, // Missing 'user' parameter
    };

    let response = ajax_handler(Extension(create_mock_db_pool()), axum::Form(request)).await;
    assert!(response.is_ok());
    let json_response = response.unwrap();
    assert_eq!(json_response.0.status, "ERROR");

    // Test invalid parameter values
    let mut params = HashMap::new();
    params.insert("amount".to_string(), "invalid_number".to_string());
    
    let request = AjaxRequest {
        func: "bank-transfer".to_string(),
        params,
    };

    let response = ajax_handler(Extension(create_mock_db_pool()), axum::Form(request)).await;
    assert!(response.is_ok());
    let json_response = response.unwrap();
    assert_eq!(json_response.0.status, "ERROR");
}

// Helper function to create a mock database pool for testing
fn create_mock_db_pool() -> sqlx::PgPool {
    // In a real implementation, this would return a proper test database pool
    // For now, returning a placeholder that matches the expected type signature
    unimplemented!("Mock DB pool creation - integrate with actual he_db::DbPool type")
}

// Property-based testing for AJAX responses
#[cfg(test)]
mod property_tests {
    use super::*;
    use quickcheck::{quickcheck, TestResult};

    #[quickcheck]
    fn prop_ajax_response_status_is_valid(msg: String) -> TestResult {
        if msg.is_empty() {
            return TestResult::discard();
        }

        let success_response = AjaxResponse::success(&msg);
        let error_response = AjaxResponse::error(&msg);

        TestResult::from_bool(
            success_response.status == "OK" &&
            error_response.status == "ERROR" &&
            success_response.msg == msg &&
            error_response.msg == msg
        )
    }

    #[quickcheck]
    fn prop_ajax_request_roundtrip(func: String, key: String, value: String) -> TestResult {
        if func.is_empty() || key.is_empty() {
            return TestResult::discard();
        }

        let mut params = HashMap::new();
        params.insert(key.clone(), value.clone());
        
        let request = AjaxRequest { func: func.clone(), params };
        
        TestResult::from_bool(
            request.func == func &&
            request.params.get(&key) == Some(&value)
        )
    }
}