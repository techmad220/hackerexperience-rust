use tokio_postgres::{Client, NoTls};
use serde_json::Value;
use std::collections::HashMap;

/// Test database helper for setting up test data
pub struct TestDb {
    pub client: Client,
}

impl TestDb {
    pub async fn new() -> Self {
        let (client, connection) = tokio_postgres::connect(
            "host=localhost user=postgres password=postgres dbname=hetest",
            NoTls,
        ).await.expect("Failed to connect to test database");

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        Self { client }
    }

    pub async fn setup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Clear all test data
        self.client.execute("TRUNCATE TABLE players CASCADE", &[]).await?;
        self.client.execute("TRUNCATE TABLE servers CASCADE", &[]).await?;
        self.client.execute("TRUNCATE TABLE processes CASCADE", &[]).await?;
        Ok(())
    }

    pub async fn create_test_player(&mut self, username: &str) -> Result<i64, Box<dyn std::error::Error>> {
        let row = self.client.query_one(
            "INSERT INTO players (username, email, password_hash) VALUES ($1, $2, $3) RETURNING player_id",
            &[&username, &format!("{}@test.com", username), &"test_hash"]
        ).await?;
        Ok(row.get(0))
    }

    pub async fn create_test_server(&mut self, player_id: i64, ip: &str) -> Result<i64, Box<dyn std::error::Error>> {
        let row = self.client.query_one(
            "INSERT INTO servers (player_id, server_ip, server_name) VALUES ($1, $2, $3) RETURNING server_id",
            &[&player_id, &ip, &"Test Server"]
        ).await?;
        Ok(row.get(0))
    }
}

/// Mock HTTP client for testing API endpoints
pub struct MockHttpClient {
    pub responses: HashMap<String, (u16, Value)>,
}

impl MockHttpClient {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
        }
    }

    pub fn set_response(&mut self, url: &str, status: u16, body: Value) {
        self.responses.insert(url.to_string(), (status, body));
    }

    pub fn get_response(&self, url: &str) -> Option<&(u16, Value)> {
        self.responses.get(url)
    }
}

/// Test fixtures for common test data
pub struct TestFixtures;

impl TestFixtures {
    pub fn sample_player_data() -> Value {
        serde_json::json!({
            "username": "testplayer",
            "email": "test@example.com",
            "password": "password123",
            "registration_date": "2024-01-01 00:00:00",
            "last_login": "2024-01-01 12:00:00"
        })
    }

    pub fn sample_server_data() -> Value {
        serde_json::json!({
            "server_ip": "192.168.1.100",
            "server_name": "Test Server",
            "server_type": "Desktop",
            "password": "server_pass",
            "created_at": "2024-01-01 00:00:00"
        })
    }

    pub fn sample_process_data() -> Value {
        serde_json::json!({
            "process_type": "cracker",
            "target_ip": "192.168.1.101",
            "target_file": "file.txt",
            "started_at": "2024-01-01 12:00:00",
            "completion_time": 300
        })
    }

    pub fn sample_software_data() -> Value {
        serde_json::json!({
            "software_type": "cracker",
            "version": 1,
            "size": 100,
            "created_at": "2024-01-01 00:00:00"
        })
    }
}

/// Assert helpers for common test patterns
pub fn assert_json_contains(actual: &Value, expected: &Value) {
    match (actual, expected) {
        (Value::Object(actual_map), Value::Object(expected_map)) => {
            for (key, expected_value) in expected_map {
                assert!(
                    actual_map.contains_key(key),
                    "Missing key '{}' in actual JSON",
                    key
                );
                assert_json_contains(&actual_map[key], expected_value);
            }
        }
        (Value::Array(actual_arr), Value::Array(expected_arr)) => {
            assert_eq!(
                actual_arr.len(),
                expected_arr.len(),
                "Array lengths don't match"
            );
            for (actual_item, expected_item) in actual_arr.iter().zip(expected_arr.iter()) {
                assert_json_contains(actual_item, expected_item);
            }
        }
        _ => {
            assert_eq!(actual, expected, "Values don't match");
        }
    }
}

#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {
        match $result {
            Ok(val) => val,
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
    };
}

#[macro_export]
macro_rules! assert_err {
    ($result:expr) => {
        match $result {
            Ok(val) => panic!("Expected Err, got Ok: {:?}", val),
            Err(e) => e,
        }
    };
}