use std::collections::HashMap;
use serde_json::Value;
use tokio;
use actix_web::{test, web, App};

mod common;
use common::{TestDb, TestFixtures, MockHttpClient, assert_json_contains};

#[tokio::test]
async fn test_player_registration() {
    let mut db = TestDb::new().await;
    db.setup().await.expect("Failed to setup test database");

    let player_data = TestFixtures::sample_player_data();
    
    // Test player creation
    let player_id = db.create_test_player("testuser").await.expect("Failed to create test player");
    assert!(player_id > 0);

    // Verify player exists in database
    let row = db.client.query_one(
        "SELECT username, email FROM players WHERE player_id = $1",
        &[&player_id]
    ).await.expect("Failed to query player");
    
    let username: String = row.get(0);
    let email: String = row.get(1);
    
    assert_eq!(username, "testuser");
    assert_eq!(email, "testuser@test.com");
}

#[tokio::test]
async fn test_server_creation() {
    let mut db = TestDb::new().await;
    db.setup().await.expect("Failed to setup test database");

    let player_id = db.create_test_player("testuser").await.expect("Failed to create test player");
    let server_id = db.create_test_server(player_id, "192.168.1.100").await.expect("Failed to create test server");
    
    assert!(server_id > 0);

    // Verify server exists and is linked to player
    let row = db.client.query_one(
        "SELECT player_id, server_ip, server_name FROM servers WHERE server_id = $1",
        &[&server_id]
    ).await.expect("Failed to query server");
    
    let db_player_id: i64 = row.get(0);
    let server_ip: String = row.get(1);
    let server_name: String = row.get(2);
    
    assert_eq!(db_player_id, player_id);
    assert_eq!(server_ip, "192.168.1.100");
    assert_eq!(server_name, "Test Server");
}

#[tokio::test]
async fn test_process_lifecycle() {
    let mut db = TestDb::new().await;
    db.setup().await.expect("Failed to setup test database");

    let player_id = db.create_test_player("testuser").await.expect("Failed to create test player");
    let server_id = db.create_test_server(player_id, "192.168.1.100").await.expect("Failed to create test server");

    // Create a test process
    let process_id = db.client.query_one(
        "INSERT INTO processes (server_id, process_type, target_ip, status) VALUES ($1, $2, $3, $4) RETURNING process_id",
        &[&server_id, &"cracker", &"192.168.1.101", &"running"]
    ).await.expect("Failed to create process")
    .get::<_, i64>(0);

    assert!(process_id > 0);

    // Test process completion
    let rows_affected = db.client.execute(
        "UPDATE processes SET status = $1, completed_at = NOW() WHERE process_id = $2",
        &[&"completed", &process_id]
    ).await.expect("Failed to update process");

    assert_eq!(rows_affected, 1);

    // Verify process is completed
    let row = db.client.query_one(
        "SELECT status, completed_at FROM processes WHERE process_id = $1",
        &[&process_id]
    ).await.expect("Failed to query process");
    
    let status: String = row.get(0);
    let completed_at: Option<chrono::NaiveDateTime> = row.get(1);
    
    assert_eq!(status, "completed");
    assert!(completed_at.is_some());
}

#[tokio::test]
async fn test_software_management() {
    let mut db = TestDb::new().await;
    db.setup().await.expect("Failed to setup test database");

    let player_id = db.create_test_player("testuser").await.expect("Failed to create test player");
    let server_id = db.create_test_server(player_id, "192.168.1.100").await.expect("Failed to create test server");

    // Create test software
    let software_id = db.client.query_one(
        "INSERT INTO software (server_id, software_type, version, size) VALUES ($1, $2, $3, $4) RETURNING software_id",
        &[&server_id, &"cracker", &1i32, &100i32]
    ).await.expect("Failed to create software")
    .get::<_, i64>(0);

    assert!(software_id > 0);

    // Test software upgrade
    let rows_affected = db.client.execute(
        "UPDATE software SET version = $1, size = $2 WHERE software_id = $3",
        &[&2i32, &150i32, &software_id]
    ).await.expect("Failed to upgrade software");

    assert_eq!(rows_affected, 1);

    // Verify software upgrade
    let row = db.client.query_one(
        "SELECT version, size FROM software WHERE software_id = $1",
        &[&software_id]
    ).await.expect("Failed to query software");
    
    let version: i32 = row.get(0);
    let size: i32 = row.get(1);
    
    assert_eq!(version, 2);
    assert_eq!(size, 150);
}

#[tokio::test] 
async fn test_authentication_flow() {
    let mut db = TestDb::new().await;
    db.setup().await.expect("Failed to setup test database");

    // Create test user with known password hash
    let password_hash = "test_hash_value";
    let player_id = db.client.query_one(
        "INSERT INTO players (username, email, password_hash) VALUES ($1, $2, $3) RETURNING player_id",
        &[&"authtest", &"authtest@test.com", &password_hash]
    ).await.expect("Failed to create test player")
    .get::<_, i64>(0);

    // Test valid authentication
    let auth_row = db.client.query_opt(
        "SELECT player_id, username FROM players WHERE username = $1 AND password_hash = $2",
        &[&"authtest", &password_hash]
    ).await.expect("Failed to query authentication");

    assert!(auth_row.is_some());
    let row = auth_row.unwrap();
    let auth_player_id: i64 = row.get(0);
    let auth_username: String = row.get(1);
    
    assert_eq!(auth_player_id, player_id);
    assert_eq!(auth_username, "authtest");

    // Test invalid authentication
    let invalid_auth = db.client.query_opt(
        "SELECT player_id FROM players WHERE username = $1 AND password_hash = $2",
        &[&"authtest", &"wrong_hash"]
    ).await.expect("Failed to query invalid authentication");

    assert!(invalid_auth.is_none());
}

#[tokio::test]
async fn test_network_scanning() {
    let mut db = TestDb::new().await;
    db.setup().await.expect("Failed to setup test database");

    let player_id = db.create_test_player("scanner").await.expect("Failed to create test player");
    let server_id = db.create_test_server(player_id, "192.168.1.100").await.expect("Failed to create test server");

    // Create target servers to scan
    let target1_id = db.create_test_server(player_id, "192.168.1.101").await.expect("Failed to create target server 1");
    let target2_id = db.create_test_server(player_id, "192.168.1.102").await.expect("Failed to create target server 2");

    // Test network scan discovery
    let discovered = db.client.query(
        "SELECT server_id, server_ip FROM servers WHERE server_ip LIKE '192.168.1.%' ORDER BY server_ip",
        &[]
    ).await.expect("Failed to query network scan");

    assert_eq!(discovered.len(), 3); // Scanner + 2 targets
    
    // Verify IPs are in correct order
    let ip1: String = discovered[0].get(1);
    let ip2: String = discovered[1].get(1);
    let ip3: String = discovered[2].get(1);
    
    assert_eq!(ip1, "192.168.1.100");
    assert_eq!(ip2, "192.168.1.101");
    assert_eq!(ip3, "192.168.1.102");
}