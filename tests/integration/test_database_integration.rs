use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use uuid::Uuid;
use std::collections::HashMap;
use std::time::Duration;

use crate::common::{TestDb, TestFixtures};
use crate::{assert_ok, assert_err};

// ===== DATABASE INTEGRATION TESTS =====

// Database models for testing
#[derive(Debug, Clone)]
pub struct Player {
    pub player_id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub registration_date: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub credits: i64,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub server_id: i64,
    pub player_id: i64,
    pub server_ip: String,
    pub server_name: String,
    pub server_type: String,
    pub password: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Process {
    pub process_id: i64,
    pub player_id: i64,
    pub server_id: i64,
    pub process_type: String,
    pub target_ip: Option<String>,
    pub target_file: Option<String>,
    pub status: String,
    pub progress: f32,
    pub started_at: DateTime<Utc>,
    pub completion_time: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct Software {
    pub software_id: i64,
    pub server_id: i64,
    pub software_type: String,
    pub version: i32,
    pub size: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct BankAccount {
    pub account_id: i64,
    pub player_id: i64,
    pub bank_id: i32,
    pub account_number: String,
    pub balance: i64,
    pub password: String,
    pub created_at: DateTime<Utc>,
}

// Database repository implementation for testing
pub struct DatabaseRepository {
    pool: PgPool,
}

impl DatabaseRepository {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let database_url = "postgresql://postgres:postgres@localhost/hetest";
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        
        Ok(Self { pool })
    }

    pub async fn setup_schema(&self) -> Result<(), sqlx::Error> {
        // Create tables for testing
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS players (
                player_id BIGSERIAL PRIMARY KEY,
                username VARCHAR(50) UNIQUE NOT NULL,
                email VARCHAR(100) UNIQUE NOT NULL,
                password_hash VARCHAR(255) NOT NULL,
                registration_date TIMESTAMPTZ DEFAULT NOW(),
                last_login TIMESTAMPTZ,
                is_active BOOLEAN DEFAULT TRUE,
                credits BIGINT DEFAULT 0
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS servers (
                server_id BIGSERIAL PRIMARY KEY,
                player_id BIGINT NOT NULL REFERENCES players(player_id) ON DELETE CASCADE,
                server_ip VARCHAR(15) UNIQUE NOT NULL,
                server_name VARCHAR(100) NOT NULL,
                server_type VARCHAR(50) NOT NULL,
                password VARCHAR(100),
                created_at TIMESTAMPTZ DEFAULT NOW()
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS processes (
                process_id BIGSERIAL PRIMARY KEY,
                player_id BIGINT NOT NULL REFERENCES players(player_id) ON DELETE CASCADE,
                server_id BIGINT NOT NULL REFERENCES servers(server_id) ON DELETE CASCADE,
                process_type VARCHAR(50) NOT NULL,
                target_ip VARCHAR(15),
                target_file VARCHAR(255),
                status VARCHAR(20) DEFAULT 'running',
                progress REAL DEFAULT 0.0,
                started_at TIMESTAMPTZ DEFAULT NOW(),
                completion_time INTEGER
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS software (
                software_id BIGSERIAL PRIMARY KEY,
                server_id BIGINT NOT NULL REFERENCES servers(server_id) ON DELETE CASCADE,
                software_type VARCHAR(50) NOT NULL,
                version INTEGER DEFAULT 1,
                size INTEGER NOT NULL,
                created_at TIMESTAMPTZ DEFAULT NOW()
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS bank_accounts (
                account_id BIGSERIAL PRIMARY KEY,
                player_id BIGINT NOT NULL REFERENCES players(player_id) ON DELETE CASCADE,
                bank_id INTEGER NOT NULL,
                account_number VARCHAR(20) UNIQUE NOT NULL,
                balance BIGINT DEFAULT 0,
                password VARCHAR(100) NOT NULL,
                created_at TIMESTAMPTZ DEFAULT NOW()
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS game_logs (
                log_id BIGSERIAL PRIMARY KEY,
                server_id BIGINT NOT NULL REFERENCES servers(server_id) ON DELETE CASCADE,
                log_type VARCHAR(50) NOT NULL,
                source_ip VARCHAR(15),
                message TEXT NOT NULL,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                is_hidden BOOLEAN DEFAULT FALSE
            )
        "#).execute(&self.pool).await?;

        Ok(())
    }

    pub async fn cleanup(&self) -> Result<(), sqlx::Error> {
        sqlx::query("DROP TABLE IF EXISTS game_logs CASCADE").execute(&self.pool).await?;
        sqlx::query("DROP TABLE IF EXISTS bank_accounts CASCADE").execute(&self.pool).await?;
        sqlx::query("DROP TABLE IF EXISTS software CASCADE").execute(&self.pool).await?;
        sqlx::query("DROP TABLE IF EXISTS processes CASCADE").execute(&self.pool).await?;
        sqlx::query("DROP TABLE IF EXISTS servers CASCADE").execute(&self.pool).await?;
        sqlx::query("DROP TABLE IF EXISTS players CASCADE").execute(&self.pool).await?;
        Ok(())
    }

    // Player operations
    pub async fn create_player(&self, username: &str, email: &str, password_hash: &str) -> Result<i64, sqlx::Error> {
        let row = sqlx::query!(
            "INSERT INTO players (username, email, password_hash) VALUES ($1, $2, $3) RETURNING player_id",
            username, email, password_hash
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.player_id)
    }

    pub async fn get_player_by_username(&self, username: &str) -> Result<Option<Player>, sqlx::Error> {
        let row = sqlx::query_as!(
            Player,
            "SELECT * FROM players WHERE username = $1",
            username
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn update_player_last_login(&self, player_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE players SET last_login = NOW() WHERE player_id = $1",
            player_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_player_count(&self) -> Result<i64, sqlx::Error> {
        let row = sqlx::query!("SELECT COUNT(*) as count FROM players")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.count.unwrap_or(0))
    }

    // Server operations
    pub async fn create_server(&self, player_id: i64, server_ip: &str, server_name: &str, server_type: &str) -> Result<i64, sqlx::Error> {
        let row = sqlx::query!(
            "INSERT INTO servers (player_id, server_ip, server_name, server_type) VALUES ($1, $2, $3, $4) RETURNING server_id",
            player_id, server_ip, server_name, server_type
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.server_id)
    }

    pub async fn get_servers_by_player(&self, player_id: i64) -> Result<Vec<Server>, sqlx::Error> {
        let rows = sqlx::query_as!(
            Server,
            "SELECT * FROM servers WHERE player_id = $1 ORDER BY created_at DESC",
            player_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn get_server_by_ip(&self, server_ip: &str) -> Result<Option<Server>, sqlx::Error> {
        let row = sqlx::query_as!(
            Server,
            "SELECT * FROM servers WHERE server_ip = $1",
            server_ip
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    // Process operations
    pub async fn start_process(&self, player_id: i64, server_id: i64, process_type: &str, target_ip: Option<&str>, completion_time: Option<i32>) -> Result<i64, sqlx::Error> {
        let row = sqlx::query!(
            "INSERT INTO processes (player_id, server_id, process_type, target_ip, completion_time) VALUES ($1, $2, $3, $4, $5) RETURNING process_id",
            player_id, server_id, process_type, target_ip, completion_time
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.process_id)
    }

    pub async fn update_process_progress(&self, process_id: i64, progress: f32) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE processes SET progress = $1, status = CASE WHEN $1 >= 100.0 THEN 'completed' ELSE status END WHERE process_id = $2",
            progress, process_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_active_processes(&self, player_id: i64) -> Result<Vec<Process>, sqlx::Error> {
        let rows = sqlx::query_as!(
            Process,
            "SELECT * FROM processes WHERE player_id = $1 AND status IN ('running', 'paused') ORDER BY started_at DESC",
            player_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn cancel_process(&self, process_id: i64, player_id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE processes SET status = 'cancelled' WHERE process_id = $1 AND player_id = $2 AND status IN ('running', 'paused')",
            process_id, player_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // Bank account operations
    pub async fn create_bank_account(&self, player_id: i64, bank_id: i32, account_number: &str, password: &str) -> Result<i64, sqlx::Error> {
        let row = sqlx::query!(
            "INSERT INTO bank_accounts (player_id, bank_id, account_number, password) VALUES ($1, $2, $3, $4) RETURNING account_id",
            player_id, bank_id, account_number, password
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.account_id)
    }

    pub async fn transfer_money(&self, from_account: &str, to_account: &str, amount: i64) -> Result<bool, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // Check source account balance
        let from_balance = sqlx::query!(
            "SELECT balance FROM bank_accounts WHERE account_number = $1 FOR UPDATE",
            from_account
        )
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(from_row) = from_balance {
            if from_row.balance < amount {
                tx.rollback().await?;
                return Ok(false); // Insufficient funds
            }
        } else {
            tx.rollback().await?;
            return Ok(false); // Source account not found
        }

        // Check if destination account exists
        let to_exists = sqlx::query!(
            "SELECT account_id FROM bank_accounts WHERE account_number = $1",
            to_account
        )
        .fetch_optional(&mut *tx)
        .await?;

        if to_exists.is_none() {
            tx.rollback().await?;
            return Ok(false); // Destination account not found
        }

        // Perform transfer
        sqlx::query!(
            "UPDATE bank_accounts SET balance = balance - $1 WHERE account_number = $2",
            amount, from_account
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE bank_accounts SET balance = balance + $1 WHERE account_number = $2",
            amount, to_account
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(true)
    }

    pub async fn get_account_balance(&self, account_number: &str) -> Result<Option<i64>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT balance FROM bank_accounts WHERE account_number = $1",
            account_number
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.balance))
    }

    // Complex queries for testing
    pub async fn get_player_statistics(&self, player_id: i64) -> Result<HashMap<String, Value>, sqlx::Error> {
        let stats = sqlx::query!(r#"
            SELECT 
                COUNT(s.server_id) as server_count,
                COUNT(p.process_id) as total_processes,
                COUNT(CASE WHEN p.status = 'completed' THEN 1 END) as completed_processes,
                COUNT(ba.account_id) as bank_accounts,
                COALESCE(SUM(ba.balance), 0) as total_balance
            FROM players pl
            LEFT JOIN servers s ON pl.player_id = s.player_id  
            LEFT JOIN processes p ON pl.player_id = p.player_id
            LEFT JOIN bank_accounts ba ON pl.player_id = ba.player_id
            WHERE pl.player_id = $1
            GROUP BY pl.player_id
        "#, player_id)
        .fetch_one(&self.pool)
        .await?;

        let mut result = HashMap::new();
        result.insert("server_count".to_string(), json!(stats.server_count.unwrap_or(0)));
        result.insert("total_processes".to_string(), json!(stats.total_processes.unwrap_or(0)));
        result.insert("completed_processes".to_string(), json!(stats.completed_processes.unwrap_or(0)));
        result.insert("bank_accounts".to_string(), json!(stats.bank_accounts.unwrap_or(0)));
        result.insert("total_balance".to_string(), json!(stats.total_balance.unwrap_or(0)));

        Ok(result)
    }
}

// ===== DATABASE INTEGRATION TESTS =====

#[tokio::test]
async fn test_database_connection() {
    let repo = assert_ok!(DatabaseRepository::new().await);
    assert_ok!(repo.setup_schema().await);
    assert_ok!(repo.cleanup().await);
}

#[tokio::test]
async fn test_player_crud_operations() {
    let repo = assert_ok!(DatabaseRepository::new().await);
    assert_ok!(repo.setup_schema().await);

    // Test create player
    let player_id = assert_ok!(repo.create_player("testuser", "test@example.com", "hash_password123").await);
    assert!(player_id > 0);

    // Test get player by username
    let player = assert_ok!(repo.get_player_by_username("testuser").await);
    assert!(player.is_some());
    let player = player.unwrap();
    assert_eq!(player.username, "testuser");
    assert_eq!(player.email, "test@example.com");
    assert_eq!(player.password_hash, "hash_password123");
    assert!(player.is_active);
    assert_eq!(player.credits, 0);

    // Test update last login
    assert_ok!(repo.update_player_last_login(player_id).await);
    let updated_player = assert_ok!(repo.get_player_by_username("testuser").await).unwrap();
    assert!(updated_player.last_login.is_some());

    // Test player count
    let count = assert_ok!(repo.get_player_count().await);
    assert_eq!(count, 1);

    // Test duplicate username (should fail)
    let result = repo.create_player("testuser", "test2@example.com", "hash_password456").await;
    assert!(result.is_err());

    assert_ok!(repo.cleanup().await);
}

#[tokio::test]
async fn test_server_operations() {
    let repo = assert_ok!(DatabaseRepository::new().await);
    assert_ok!(repo.setup_schema().await);

    // Create test player first
    let player_id = assert_ok!(repo.create_player("testuser", "test@example.com", "hash_password").await);

    // Test create server
    let server_id = assert_ok!(repo.create_server(player_id, "192.168.1.100", "Test Server", "Desktop").await);
    assert!(server_id > 0);

    // Test get server by IP
    let server = assert_ok!(repo.get_server_by_ip("192.168.1.100").await);
    assert!(server.is_some());
    let server = server.unwrap();
    assert_eq!(server.player_id, player_id);
    assert_eq!(server.server_ip, "192.168.1.100");
    assert_eq!(server.server_name, "Test Server");
    assert_eq!(server.server_type, "Desktop");

    // Test get servers by player
    let servers = assert_ok!(repo.get_servers_by_player(player_id).await);
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].server_id, server_id);

    // Test create multiple servers
    assert_ok!(repo.create_server(player_id, "10.0.0.1", "Remote Server", "Server").await);
    let servers = assert_ok!(repo.get_servers_by_player(player_id).await);
    assert_eq!(servers.len(), 2);

    // Test duplicate IP (should fail)
    let result = repo.create_server(player_id, "192.168.1.100", "Duplicate", "Desktop").await;
    assert!(result.is_err());

    assert_ok!(repo.cleanup().await);
}

#[tokio::test]
async fn test_process_operations() {
    let repo = assert_ok!(DatabaseRepository::new().await);
    assert_ok!(repo.setup_schema().await);

    // Setup test data
    let player_id = assert_ok!(repo.create_player("testuser", "test@example.com", "hash_password").await);
    let server_id = assert_ok!(repo.create_server(player_id, "192.168.1.100", "Test Server", "Desktop").await);

    // Test start process
    let process_id = assert_ok!(repo.start_process(player_id, server_id, "cracker", Some("192.168.1.101"), Some(300)).await);
    assert!(process_id > 0);

    // Test get active processes
    let processes = assert_ok!(repo.get_active_processes(player_id).await);
    assert_eq!(processes.len(), 1);
    assert_eq!(processes[0].process_type, "cracker");
    assert_eq!(processes[0].status, "running");
    assert_eq!(processes[0].progress, 0.0);

    // Test update process progress
    assert_ok!(repo.update_process_progress(process_id, 50.0).await);
    let processes = assert_ok!(repo.get_active_processes(player_id).await);
    assert_eq!(processes[0].progress, 50.0);
    assert_eq!(processes[0].status, "running");

    // Test complete process
    assert_ok!(repo.update_process_progress(process_id, 100.0).await);
    let processes = assert_ok!(repo.get_active_processes(player_id).await);
    assert_eq!(processes.len(), 0); // Should not be in active processes anymore

    // Test cancel process
    let process_id2 = assert_ok!(repo.start_process(player_id, server_id, "uploader", Some("test.exe"), Some(120)).await);
    let cancelled = assert_ok!(repo.cancel_process(process_id2, player_id).await);
    assert!(cancelled);

    let processes = assert_ok!(repo.get_active_processes(player_id).await);
    assert_eq!(processes.len(), 0);

    assert_ok!(repo.cleanup().await);
}

#[tokio::test]
async fn test_bank_account_operations() {
    let repo = assert_ok!(DatabaseRepository::new().await);
    assert_ok!(repo.setup_schema().await);

    // Setup test data
    let player1_id = assert_ok!(repo.create_player("player1", "player1@example.com", "hash_pass1").await);
    let player2_id = assert_ok!(repo.create_player("player2", "player2@example.com", "hash_pass2").await);

    // Create bank accounts
    let account1_id = assert_ok!(repo.create_bank_account(player1_id, 1, "123456789", "account_pass1").await);
    let account2_id = assert_ok!(repo.create_bank_account(player2_id, 1, "987654321", "account_pass2").await);

    // Test initial balances
    let balance1 = assert_ok!(repo.get_account_balance("123456789").await);
    assert_eq!(balance1, Some(0));

    // Add some money to first account directly (for testing)
    sqlx::query!("UPDATE bank_accounts SET balance = 1000 WHERE account_id = $1", account1_id)
        .execute(&repo.pool)
        .await
        .unwrap();

    let balance1 = assert_ok!(repo.get_account_balance("123456789").await);
    assert_eq!(balance1, Some(1000));

    // Test successful transfer
    let transfer_result = assert_ok!(repo.transfer_money("123456789", "987654321", 300).await);
    assert!(transfer_result);

    let balance1 = assert_ok!(repo.get_account_balance("123456789").await);
    let balance2 = assert_ok!(repo.get_account_balance("987654321").await);
    assert_eq!(balance1, Some(700));
    assert_eq!(balance2, Some(300));

    // Test insufficient funds
    let transfer_result = assert_ok!(repo.transfer_money("123456789", "987654321", 1000).await);
    assert!(!transfer_result);

    // Balances should remain unchanged
    let balance1 = assert_ok!(repo.get_account_balance("123456789").await);
    let balance2 = assert_ok!(repo.get_account_balance("987654321").await);
    assert_eq!(balance1, Some(700));
    assert_eq!(balance2, Some(300));

    // Test transfer to non-existent account
    let transfer_result = assert_ok!(repo.transfer_money("123456789", "000000000", 100).await);
    assert!(!transfer_result);

    assert_ok!(repo.cleanup().await);
}

#[tokio::test]
async fn test_complex_player_statistics() {
    let repo = assert_ok!(DatabaseRepository::new().await);
    assert_ok!(repo.setup_schema().await);

    // Create test player
    let player_id = assert_ok!(repo.create_player("statsuser", "stats@example.com", "hash_password").await);

    // Create servers
    let server1_id = assert_ok!(repo.create_server(player_id, "192.168.1.100", "Server 1", "Desktop").await);
    let server2_id = assert_ok!(repo.create_server(player_id, "192.168.1.101", "Server 2", "Server").await);

    // Create processes
    let process1_id = assert_ok!(repo.start_process(player_id, server1_id, "cracker", Some("target1"), Some(300)).await);
    let process2_id = assert_ok!(repo.start_process(player_id, server2_id, "uploader", Some("file1"), Some(120)).await);

    // Complete one process
    assert_ok!(repo.update_process_progress(process1_id, 100.0).await);

    // Create bank accounts
    let account1_id = assert_ok!(repo.create_bank_account(player_id, 1, "111111111", "pass1").await);
    let account2_id = assert_ok!(repo.create_bank_account(player_id, 2, "222222222", "pass2").await);

    // Add balances
    sqlx::query!("UPDATE bank_accounts SET balance = 1500 WHERE account_id = $1", account1_id)
        .execute(&repo.pool).await.unwrap();
    sqlx::query!("UPDATE bank_accounts SET balance = 2500 WHERE account_id = $1", account2_id)
        .execute(&repo.pool).await.unwrap();

    // Get statistics
    let stats = assert_ok!(repo.get_player_statistics(player_id).await);

    assert_eq!(stats["server_count"], json!(2));
    assert_eq!(stats["total_processes"], json!(2));
    assert_eq!(stats["completed_processes"], json!(1));
    assert_eq!(stats["bank_accounts"], json!(2));
    assert_eq!(stats["total_balance"], json!(4000));

    assert_ok!(repo.cleanup().await);
}

#[tokio::test]
async fn test_database_transactions() {
    let repo = assert_ok!(DatabaseRepository::new().await);
    assert_ok!(repo.setup_schema().await);

    // Test transaction rollback scenario
    let player1_id = assert_ok!(repo.create_player("player1", "player1@example.com", "hash_pass1").await);
    let player2_id = assert_ok!(repo.create_player("player2", "player2@example.com", "hash_pass2").await);

    let account1_id = assert_ok!(repo.create_bank_account(player1_id, 1, "123456789", "pass1").await);

    // Add money to first account
    sqlx::query!("UPDATE bank_accounts SET balance = 1000 WHERE account_id = $1", account1_id)
        .execute(&repo.pool).await.unwrap();

    // Try to transfer to non-existent account (should fail and rollback)
    let transfer_result = assert_ok!(repo.transfer_money("123456789", "999999999", 500).await);
    assert!(!transfer_result);

    // Original balance should be unchanged
    let balance1 = assert_ok!(repo.get_account_balance("123456789").await);
    assert_eq!(balance1, Some(1000));

    assert_ok!(repo.cleanup().await);
}

#[tokio::test]
async fn test_database_constraints() {
    let repo = assert_ok!(DatabaseRepository::new().await);
    assert_ok!(repo.setup_schema().await);

    // Test foreign key constraints
    let player_id = assert_ok!(repo.create_player("testuser", "test@example.com", "hash_password").await);
    let server_id = assert_ok!(repo.create_server(player_id, "192.168.1.100", "Test Server", "Desktop").await);

    // Try to create process with invalid server_id (should fail)
    let result = repo.start_process(player_id, 99999, "cracker", Some("target"), Some(300)).await;
    assert!(result.is_err());

    // Try to create server with invalid player_id (should fail)
    let result = repo.create_server(99999, "10.0.0.1", "Invalid", "Desktop").await;
    assert!(result.is_err());

    assert_ok!(repo.cleanup().await);
}

#[tokio::test]
async fn test_concurrent_database_operations() {
    let repo = assert_ok!(DatabaseRepository::new().await);
    assert_ok!(repo.setup_schema().await);

    let player_id = assert_ok!(repo.create_player("concurrentuser", "concurrent@example.com", "hash_password").await);

    // Create multiple servers concurrently
    let server_futures: Vec<_> = (0..5).map(|i| {
        let repo = &repo;
        async move {
            repo.create_server(player_id, &format!("192.168.1.{}", 100 + i), &format!("Server {}", i), "Desktop").await
        }
    }).collect();

    let server_results = futures::future::join_all(server_futures).await;

    // All servers should be created successfully
    for result in server_results {
        assert!(result.is_ok());
    }

    // Verify all servers exist
    let servers = assert_ok!(repo.get_servers_by_player(player_id).await);
    assert_eq!(servers.len(), 5);

    assert_ok!(repo.cleanup().await);
}

#[tokio::test]
async fn test_database_performance() {
    let repo = assert_ok!(DatabaseRepository::new().await);
    assert_ok!(repo.setup_schema().await);

    let start_time = std::time::Instant::now();
    
    // Create many players rapidly
    let player_futures: Vec<_> = (0..100).map(|i| {
        let repo = &repo;
        async move {
            repo.create_player(&format!("user{}", i), &format!("user{}@example.com", i), "hash_password").await
        }
    }).collect();

    let player_results = futures::future::join_all(player_futures).await;
    
    let duration = start_time.elapsed();
    
    // All operations should succeed
    for result in player_results {
        assert!(result.is_ok());
    }

    // Performance check: should complete within reasonable time
    assert!(duration < Duration::from_secs(10), "Database operations too slow: {:?}", duration);

    // Verify count
    let count = assert_ok!(repo.get_player_count().await);
    assert_eq!(count, 100);

    println!("Created 100 players in {:?} ({:.2} players/sec)", 
        duration, 100.0 / duration.as_secs_f64());

    assert_ok!(repo.cleanup().await);
}

#[tokio::test] 
async fn test_database_edge_cases() {
    let repo = assert_ok!(DatabaseRepository::new().await);
    assert_ok!(repo.setup_schema().await);

    // Test empty string handling
    let result = repo.create_player("", "empty@example.com", "hash_password").await;
    assert!(result.is_err()); // Should fail due to constraints

    // Test very long strings
    let long_username = "a".repeat(100);
    let result = repo.create_player(&long_username, "long@example.com", "hash_password").await;
    assert!(result.is_err()); // Should fail due to length constraint

    // Test special characters
    let player_id = assert_ok!(repo.create_player("user_with-special.chars", "special@example.com", "hash_password").await);
    let player = assert_ok!(repo.get_player_by_username("user_with-special.chars").await);
    assert!(player.is_some());

    assert_ok!(repo.cleanup().await);
}

#[tokio::test]
async fn test_database_data_integrity() {
    let repo = assert_ok!(DatabaseRepository::new().await);
    assert_ok!(repo.setup_schema().await);

    // Test cascade deletion
    let player_id = assert_ok!(repo.create_player("testuser", "test@example.com", "hash_password").await);
    let server_id = assert_ok!(repo.create_server(player_id, "192.168.1.100", "Test Server", "Desktop").await);
    let process_id = assert_ok!(repo.start_process(player_id, server_id, "cracker", Some("target"), Some(300)).await);
    let account_id = assert_ok!(repo.create_bank_account(player_id, 1, "123456789", "password").await);

    // Verify data exists
    let servers = assert_ok!(repo.get_servers_by_player(player_id).await);
    assert_eq!(servers.len(), 1);
    let processes = assert_ok!(repo.get_active_processes(player_id).await);
    assert_eq!(processes.len(), 1);
    let balance = assert_ok!(repo.get_account_balance("123456789").await);
    assert!(balance.is_some());

    // Delete player (should cascade)
    sqlx::query!("DELETE FROM players WHERE player_id = $1", player_id)
        .execute(&repo.pool).await.unwrap();

    // Verify related data is also deleted
    let servers = assert_ok!(repo.get_servers_by_player(player_id).await);
    assert_eq!(servers.len(), 0);
    let processes = assert_ok!(repo.get_active_processes(player_id).await);
    assert_eq!(processes.len(), 0);
    let balance = assert_ok!(repo.get_account_balance("123456789").await);
    assert!(balance.is_none());

    assert_ok!(repo.cleanup().await);
}