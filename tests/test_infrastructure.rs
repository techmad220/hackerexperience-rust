use std::{
    collections::HashMap,
    env,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::{
    process::Command,
    sync::{RwLock, Mutex},
    time::sleep,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::common::{TestDb, TestFixtures, MockHttpClient};
use crate::{assert_ok, assert_err};

// ===== TEST INFRASTRUCTURE UTILITIES =====

/// Comprehensive test configuration management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub database: DatabaseConfig,
    pub redis: RedisConfig,  
    pub websocket: WebSocketConfig,
    pub security: SecurityConfig,
    pub performance: PerformanceConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub database: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub heartbeat_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub rate_limit_max: u32,
    pub rate_limit_window_seconds: u64,
    pub password_min_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_concurrent_users: usize,
    pub request_timeout_seconds: u64,
    pub memory_limit_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub max_file_size_mb: usize,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                username: "postgres".to_string(),
                password: "postgres".to_string(),
                database: "hetest".to_string(),
                max_connections: 20,
                timeout_seconds: 30,
            },
            redis: RedisConfig {
                host: "localhost".to_string(),
                port: 6379,
                password: None,
                database: 1, // Use database 1 for tests
            },
            websocket: WebSocketConfig {
                host: "127.0.0.1".to_string(),
                port: 0, // Auto-assign port
                max_connections: 1000,
                heartbeat_interval_seconds: 30,
            },
            security: SecurityConfig {
                jwt_secret: "test_jwt_secret_key_change_in_production".to_string(),
                rate_limit_max: 100,
                rate_limit_window_seconds: 60,
                password_min_length: 8,
            },
            performance: PerformanceConfig {
                max_concurrent_users: 1000,
                request_timeout_seconds: 30,
                memory_limit_mb: 512,
            },
            logging: LoggingConfig {
                level: "debug".to_string(),
                file_path: Some("/tmp/hetest.log".to_string()),
                max_file_size_mb: 100,
            },
        }
    }
}

/// Test environment manager
pub struct TestEnvironment {
    pub config: TestConfig,
    pub database: Option<TestDb>,
    pub temp_dir: PathBuf,
    pub cleanup_tasks: Vec<Box<dyn FnOnce() + Send>>,
    pub services: HashMap<String, Arc<dyn TestService>>,
}

pub trait TestService: Send + Sync {
    async fn start(&self) -> Result<(), Box<dyn std::error::Error>>;
    async fn stop(&self) -> Result<(), Box<dyn std::error::Error>>;
    async fn health_check(&self) -> bool;
    fn name(&self) -> &str;
}

impl TestEnvironment {
    pub fn new() -> Self {
        let temp_dir = std::env::temp_dir().join(format!("hetest_{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");

        Self {
            config: TestConfig::default(),
            database: None,
            temp_dir,
            cleanup_tasks: Vec::new(),
            services: HashMap::new(),
        }
    }

    pub fn with_config(mut self, config: TestConfig) -> Self {
        self.config = config;
        self
    }

    pub async fn setup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Setting up test environment...");

        // Setup database
        if let Ok(_) = env::var("TEST_DATABASE_URL") {
            let mut db = TestDb::new().await;
            db.setup().await?;
            self.database = Some(db);
            println!("✓ Database setup complete");
        }

        // Setup temp directories
        fs::create_dir_all(self.temp_dir.join("logs"))?;
        fs::create_dir_all(self.temp_dir.join("data"))?;
        fs::create_dir_all(self.temp_dir.join("uploads"))?;
        println!("✓ Temporary directories created");

        // Start test services
        for (name, service) in &self.services {
            println!("Starting service: {}", name);
            service.start().await?;
            
            // Wait for service to be ready
            let mut attempts = 0;
            while attempts < 30 && !service.health_check().await {
                sleep(Duration::from_millis(100)).await;
                attempts += 1;
            }
            
            if !service.health_check().await {
                return Err(format!("Service {} failed to start", name).into());
            }
            println!("✓ Service {} started", name);
        }

        println!("✓ Test environment setup complete");
        Ok(())
    }

    pub async fn teardown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Tearing down test environment...");

        // Stop services
        for (name, service) in &self.services {
            println!("Stopping service: {}", name);
            if let Err(e) = service.stop().await {
                eprintln!("Failed to stop service {}: {}", name, e);
            }
        }

        // Cleanup database
        if let Some(ref mut db) = self.database {
            if let Err(e) = db.setup().await {
                eprintln!("Failed to cleanup database: {}", e);
            }
        }

        // Run cleanup tasks
        for cleanup in self.cleanup_tasks.drain(..) {
            cleanup();
        }

        // Remove temp directory
        if let Err(e) = fs::remove_dir_all(&self.temp_dir) {
            eprintln!("Failed to remove temp directory: {}", e);
        }

        println!("✓ Test environment teardown complete");
        Ok(())
    }

    pub fn add_service(&mut self, name: String, service: Arc<dyn TestService>) {
        self.services.insert(name, service);
    }

    pub fn add_cleanup_task<F>(&mut self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.cleanup_tasks.push(Box::new(task));
    }

    pub fn get_temp_dir(&self) -> &Path {
        &self.temp_dir
    }

    pub fn get_log_path(&self) -> PathBuf {
        self.temp_dir.join("logs").join("test.log")
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        // Ensure cleanup happens even if teardown wasn't called explicitly
        if !self.cleanup_tasks.is_empty() || !self.temp_dir.exists() == false {
            if let Err(e) = fs::remove_dir_all(&self.temp_dir) {
                eprintln!("Failed to cleanup temp directory in drop: {}", e);
            }
        }
    }
}

/// Mock external services for testing
pub struct MockExternalService {
    name: String,
    responses: Arc<RwLock<HashMap<String, Value>>>,
    request_log: Arc<RwLock<Vec<ServiceRequest>>>,
    latency: Duration,
    failure_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceRequest {
    pub timestamp: DateTime<Utc>,
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl MockExternalService {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            responses: Arc::new(RwLock::new(HashMap::new())),
            request_log: Arc::new(RwLock::new(Vec::new())),
            latency: Duration::from_millis(100),
            failure_rate: 0.0,
        }
    }

    pub fn with_latency(mut self, latency: Duration) -> Self {
        self.latency = latency;
        self
    }

    pub fn with_failure_rate(mut self, rate: f64) -> Self {
        self.failure_rate = rate;
        self
    }

    pub async fn set_response(&self, path: &str, response: Value) {
        let mut responses = self.responses.write().await;
        responses.insert(path.to_string(), response);
    }

    pub async fn get_request_log(&self) -> Vec<ServiceRequest> {
        self.request_log.read().await.clone()
    }

    pub async fn clear_request_log(&self) {
        self.request_log.write().await.clear();
    }

    pub async fn simulate_request(
        &self,
        method: &str,
        path: &str,
        headers: HashMap<String, String>,
        body: Option<String>,
    ) -> Result<Value, String> {
        // Log the request
        {
            let mut log = self.request_log.write().await;
            log.push(ServiceRequest {
                timestamp: Utc::now(),
                method: method.to_string(),
                path: path.to_string(),
                headers,
                body,
            });
        }

        // Simulate latency
        sleep(self.latency).await;

        // Simulate random failures
        if self.failure_rate > 0.0 && rand::random::<f64>() < self.failure_rate {
            return Err("Service temporarily unavailable".to_string());
        }

        // Return configured response or default
        let responses = self.responses.read().await;
        Ok(responses.get(path).cloned().unwrap_or_else(|| {
            json!({
                "status": "success",
                "data": {},
                "message": "Mock response"
            })
        }))
    }
}

#[async_trait::async_trait]
impl TestService for MockExternalService {
    async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Mock service is always "running"
        Ok(())
    }

    async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Clear any state
        self.clear_request_log().await;
        Ok(())
    }

    async fn health_check(&self) -> bool {
        true
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Test data factory for creating consistent test data
pub struct TestDataFactory {
    player_counter: Arc<Mutex<u64>>,
    server_counter: Arc<Mutex<u64>>,
    process_counter: Arc<Mutex<u64>>,
}

impl TestDataFactory {
    pub fn new() -> Self {
        Self {
            player_counter: Arc::new(Mutex::new(1)),
            server_counter: Arc::new(Mutex::new(1)),
            process_counter: Arc::new(Mutex::new(1)),
        }
    }

    pub async fn create_player_data(&self, username_prefix: &str) -> Value {
        let mut counter = self.player_counter.lock().await;
        let id = *counter;
        *counter += 1;

        json!({
            "player_id": id,
            "username": format!("{}_{}", username_prefix, id),
            "email": format!("{}{}@test.com", username_prefix, id),
            "password_hash": "test_hash",
            "registration_date": Utc::now().to_rfc3339(),
            "last_login": null,
            "is_active": true,
            "credits": 1000,
            "level": 1
        })
    }

    pub async fn create_server_data(&self, player_id: u64) -> Value {
        let mut counter = self.server_counter.lock().await;
        let id = *counter;
        *counter += 1;

        json!({
            "server_id": id,
            "player_id": player_id,
            "server_ip": format!("192.168.1.{}", 100 + (id % 155)),
            "server_name": format!("TestServer_{}", id),
            "server_type": "Desktop",
            "password": "server_pass",
            "created_at": Utc::now().to_rfc3339()
        })
    }

    pub async fn create_process_data(&self, player_id: u64, server_id: u64, process_type: &str) -> Value {
        let mut counter = self.process_counter.lock().await;
        let id = *counter;
        *counter += 1;

        json!({
            "process_id": id,
            "player_id": player_id,
            "server_id": server_id,
            "process_type": process_type,
            "status": "running",
            "progress": 0.0,
            "target_ip": format!("10.0.0.{}", id % 255),
            "started_at": Utc::now().to_rfc3339(),
            "completion_time": 300
        })
    }

    pub fn create_bank_account_data(&self, player_id: u64, bank_id: u32) -> Value {
        json!({
            "account_id": rand::random::<u64>(),
            "player_id": player_id,
            "bank_id": bank_id,
            "account_number": format!("{:010}", rand::random::<u32>()),
            "balance": 5000,
            "password": "account_pass",
            "created_at": Utc::now().to_rfc3339()
        })
    }

    pub fn create_software_data(&self, server_id: u64, software_type: &str) -> Value {
        json!({
            "software_id": rand::random::<u64>(),
            "server_id": server_id,
            "software_type": software_type,
            "version": 1,
            "size": rand::random::<u32>() % 1000 + 100,
            "created_at": Utc::now().to_rfc3339()
        })
    }
}

/// Test assertion helpers and utilities
pub struct TestAssertions;

impl TestAssertions {
    pub fn assert_response_time(duration: Duration, max_ms: u64, operation: &str) {
        let actual_ms = duration.as_millis() as u64;
        assert!(
            actual_ms <= max_ms,
            "{} took {}ms, expected <= {}ms",
            operation,
            actual_ms,
            max_ms
        );
    }

    pub fn assert_memory_usage(usage_bytes: u64, max_mb: u64, operation: &str) {
        let usage_mb = usage_bytes / 1024 / 1024;
        assert!(
            usage_mb <= max_mb,
            "{} used {}MB memory, expected <= {}MB",
            operation,
            usage_mb,
            max_mb
        );
    }

    pub fn assert_error_rate(errors: usize, total: usize, max_percent: f64, operation: &str) {
        if total == 0 {
            return;
        }
        
        let error_rate = (errors as f64 / total as f64) * 100.0;
        assert!(
            error_rate <= max_percent,
            "{} error rate {:.2}%, expected <= {:.2}%",
            operation,
            error_rate,
            max_percent
        );
    }

    pub fn assert_throughput(operations: usize, duration: Duration, min_ops_per_sec: f64, operation: &str) {
        let ops_per_sec = operations as f64 / duration.as_secs_f64();
        assert!(
            ops_per_sec >= min_ops_per_sec,
            "{} throughput {:.2} ops/sec, expected >= {:.2} ops/sec",
            operation,
            ops_per_sec,
            min_ops_per_sec
        );
    }

    pub fn assert_json_structure(actual: &Value, expected_structure: &Value) {
        match (actual, expected_structure) {
            (Value::Object(actual_map), Value::Object(expected_map)) => {
                for (key, expected_value) in expected_map {
                    assert!(
                        actual_map.contains_key(key),
                        "Missing key '{}' in JSON response",
                        key
                    );
                    
                    if !expected_value.is_null() {
                        Self::assert_json_structure(&actual_map[key], expected_value);
                    }
                }
            }
            (Value::Array(actual_arr), Value::Array(expected_arr)) => {
                if !expected_arr.is_empty() {
                    assert!(
                        !actual_arr.is_empty(),
                        "Expected non-empty array"
                    );
                    // Check first element structure
                    Self::assert_json_structure(&actual_arr[0], &expected_arr[0]);
                }
            }
            (actual, expected) => {
                if !expected.is_null() {
                    assert_eq!(
                        std::mem::discriminant(actual),
                        std::mem::discriminant(expected),
                        "JSON type mismatch: expected {:?}, got {:?}",
                        expected,
                        actual
                    );
                }
            }
        }
    }
}

/// Test runner with advanced features
pub struct TestRunner {
    pub config: TestConfig,
    pub environment: TestEnvironment,
    pub metrics: TestMetrics,
}

#[derive(Debug, Default)]
pub struct TestMetrics {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub skipped_tests: usize,
    pub total_duration: Duration,
    pub test_results: Vec<TestResult>,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub status: TestStatus,
    pub duration: Duration,
    pub error: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
}

impl TestRunner {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = TestConfig::default();
        let mut environment = TestEnvironment::new().with_config(config.clone());
        environment.setup().await?;

        Ok(Self {
            config,
            environment,
            metrics: TestMetrics::default(),
        })
    }

    pub async fn run_test<F, Fut>(&mut self, name: &str, tags: Vec<String>, test_fn: F) -> TestResult
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error>>>,
    {
        println!("Running test: {}", name);
        let start_time = std::time::Instant::now();

        let result = match test_fn().await {
            Ok(()) => TestResult {
                name: name.to_string(),
                status: TestStatus::Passed,
                duration: start_time.elapsed(),
                error: None,
                tags,
            },
            Err(e) => TestResult {
                name: name.to_string(),
                status: TestStatus::Failed,
                duration: start_time.elapsed(),
                error: Some(e.to_string()),
                tags,
            },
        };

        // Update metrics
        self.metrics.total_tests += 1;
        match result.status {
            TestStatus::Passed => self.metrics.passed_tests += 1,
            TestStatus::Failed => self.metrics.failed_tests += 1,
            TestStatus::Skipped => self.metrics.skipped_tests += 1,
        }
        self.metrics.total_duration += result.duration;
        self.metrics.test_results.push(result.clone());

        println!(
            "Test {} {} in {:.2}s",
            name,
            match result.status {
                TestStatus::Passed => "PASSED",
                TestStatus::Failed => "FAILED",
                TestStatus::Skipped => "SKIPPED",
            },
            result.duration.as_secs_f64()
        );

        if let Some(error) = &result.error {
            println!("Error: {}", error);
        }

        result
    }

    pub fn generate_test_report(&self) -> TestReport {
        let success_rate = if self.metrics.total_tests > 0 {
            (self.metrics.passed_tests as f64 / self.metrics.total_tests as f64) * 100.0
        } else {
            0.0
        };

        TestReport {
            summary: TestSummary {
                total_tests: self.metrics.total_tests,
                passed_tests: self.metrics.passed_tests,
                failed_tests: self.metrics.failed_tests,
                skipped_tests: self.metrics.skipped_tests,
                success_rate,
                total_duration: self.metrics.total_duration,
                average_test_duration: if self.metrics.total_tests > 0 {
                    self.metrics.total_duration / self.metrics.total_tests as u32
                } else {
                    Duration::from_secs(0)
                },
            },
            test_results: self.metrics.test_results.clone(),
            failed_tests: self.metrics.test_results.iter()
                .filter(|r| r.status == TestStatus::Failed)
                .cloned()
                .collect(),
        }
    }

    pub async fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.environment.teardown().await
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TestReport {
    pub summary: TestSummary,
    pub test_results: Vec<TestResult>,
    pub failed_tests: Vec<TestResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TestSummary {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub skipped_tests: usize,
    pub success_rate: f64,
    pub total_duration: Duration,
    pub average_test_duration: Duration,
}

impl TestReport {
    pub fn save_to_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn print_summary(&self) {
        println!("\n=== TEST REPORT SUMMARY ===");
        println!("Total Tests: {}", self.summary.total_tests);
        println!("Passed: {} ({:.1}%)", 
            self.summary.passed_tests,
            (self.summary.passed_tests as f64 / self.summary.total_tests as f64) * 100.0);
        println!("Failed: {} ({:.1}%)", 
            self.summary.failed_tests,
            (self.summary.failed_tests as f64 / self.summary.total_tests as f64) * 100.0);
        println!("Skipped: {}", self.summary.skipped_tests);
        println!("Success Rate: {:.1}%", self.summary.success_rate);
        println!("Total Duration: {:.2}s", self.summary.total_duration.as_secs_f64());
        println!("Average Test Duration: {:.2}s", self.summary.average_test_duration.as_secs_f64());
        
        if !self.failed_tests.is_empty() {
            println!("\n=== FAILED TESTS ===");
            for test in &self.failed_tests {
                println!("❌ {} ({:.2}s)", test.name, test.duration.as_secs_f64());
                if let Some(error) = &test.error {
                    println!("   Error: {}", error);
                }
            }
        }
        
        println!("===========================");
    }
}

// ===== CONTINUOUS INTEGRATION HELPERS =====

pub struct CIHelper;

impl CIHelper {
    pub async fn run_database_migrations() -> Result<(), Box<dyn std::error::Error>> {
        println!("Running database migrations...");
        
        let mut cmd = Command::new("sqlx");
        cmd.args(&["migrate", "run", "--database-url", &env::var("DATABASE_URL")?]);
        
        let output = cmd.output().await?;
        
        if !output.status.success() {
            return Err(format!(
                "Migration failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ).into());
        }
        
        println!("✓ Database migrations completed");
        Ok(())
    }

    pub async fn wait_for_database(timeout: Duration) -> Result<(), Box<dyn std::error::Error>> {
        println!("Waiting for database to be ready...");
        
        let start = std::time::Instant::now();
        
        while start.elapsed() < timeout {
            let mut cmd = Command::new("pg_isready");
            cmd.args(&["-h", "localhost", "-p", "5432"]);
            
            if let Ok(output) = cmd.output().await {
                if output.status.success() {
                    println!("✓ Database is ready");
                    return Ok(());
                }
            }
            
            sleep(Duration::from_millis(500)).await;
        }
        
        Err("Timeout waiting for database".into())
    }

    pub async fn wait_for_redis(timeout: Duration) -> Result<(), Box<dyn std::error::Error>> {
        println!("Waiting for Redis to be ready...");
        
        let start = std::time::Instant::now();
        
        while start.elapsed() < timeout {
            let mut cmd = Command::new("redis-cli");
            cmd.args(&["ping"]);
            
            if let Ok(output) = cmd.output().await {
                if output.status.success() && String::from_utf8_lossy(&output.stdout).trim() == "PONG" {
                    println!("✓ Redis is ready");
                    return Ok(());
                }
            }
            
            sleep(Duration::from_millis(500)).await;
        }
        
        Err("Timeout waiting for Redis".into())
    }

    pub fn setup_test_environment() -> Result<(), Box<dyn std::error::Error>> {
        // Set environment variables for testing
        env::set_var("RUST_ENV", "test");
        env::set_var("RUST_LOG", "debug");
        env::set_var("DATABASE_URL", "postgresql://postgres:postgres@localhost/hetest");
        env::set_var("REDIS_URL", "redis://localhost:6379/1");
        
        println!("✓ Test environment variables set");
        Ok(())
    }

    pub async fn generate_coverage_report() -> Result<(), Box<dyn std::error::Error>> {
        println!("Generating code coverage report...");
        
        let mut cmd = Command::new("cargo");
        cmd.args(&["tarpaulin", "--out", "xml", "--output-dir", "target/coverage"]);
        
        let output = cmd.output().await?;
        
        if !output.status.success() {
            eprintln!("Warning: Coverage report generation failed: {}", 
                String::from_utf8_lossy(&output.stderr));
        } else {
            println!("✓ Coverage report generated in target/coverage/");
        }
        
        Ok(())
    }
}

// ===== BENCHMARK UTILITIES =====

pub struct BenchmarkRunner {
    results: Vec<BenchmarkResult>,
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub duration: Duration,
    pub operations: usize,
    pub ops_per_second: f64,
    pub memory_used: u64,
}

impl BenchmarkRunner {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub async fn run_benchmark<F, Fut>(&mut self, name: &str, operations: usize, benchmark_fn: F)
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error>>>,
    {
        println!("Running benchmark: {} ({} operations)", name, operations);
        
        let start_memory = get_memory_usage();
        let start_time = std::time::Instant::now();
        
        for _ in 0..operations {
            if let Err(e) = benchmark_fn().await {
                eprintln!("Benchmark operation failed: {}", e);
                return;
            }
        }
        
        let duration = start_time.elapsed();
        let end_memory = get_memory_usage();
        let ops_per_second = operations as f64 / duration.as_secs_f64();
        
        let result = BenchmarkResult {
            name: name.to_string(),
            duration,
            operations,
            ops_per_second,
            memory_used: end_memory.saturating_sub(start_memory),
        };
        
        println!("Benchmark {} completed: {:.2} ops/sec, {:.2}s total", 
            name, ops_per_second, duration.as_secs_f64());
        
        self.results.push(result);
    }

    pub fn get_results(&self) -> &[BenchmarkResult] {
        &self.results
    }

    pub fn print_results(&self) {
        println!("\n=== BENCHMARK RESULTS ===");
        for result in &self.results {
            println!("{}: {:.2} ops/sec ({} ops in {:.2}s, {}MB memory)",
                result.name,
                result.ops_per_second,
                result.operations,
                result.duration.as_secs_f64(),
                result.memory_used / 1024 / 1024);
        }
        println!("=========================");
    }
}

// Helper function to get memory usage (simplified)
fn get_memory_usage() -> u64 {
    // In a real implementation, you would use system APIs
    // For testing, return a simulated value
    50 * 1024 * 1024 // 50MB baseline
}

// ===== INFRASTRUCTURE TESTS =====

#[tokio::test]
async fn test_test_environment_setup_and_teardown() {
    let mut env = TestEnvironment::new();
    
    // Add a mock service
    let mock_service = Arc::new(MockExternalService::new("test_service"));
    env.add_service("test_service".to_string(), mock_service);
    
    // Setup should succeed
    assert_ok!(env.setup().await);
    
    // Check temp directory was created
    assert!(env.get_temp_dir().exists());
    assert!(env.get_temp_dir().join("logs").exists());
    assert!(env.get_temp_dir().join("data").exists());
    
    // Teardown should succeed
    assert_ok!(env.teardown().await);
    
    // Temp directory should be cleaned up
    assert!(!env.get_temp_dir().exists());
}

#[tokio::test]
async fn test_test_data_factory() {
    let factory = TestDataFactory::new();
    
    // Create test player data
    let player1 = factory.create_player_data("testuser").await;
    let player2 = factory.create_player_data("testuser").await;
    
    // Should have unique IDs
    assert_ne!(player1["player_id"], player2["player_id"]);
    assert_ne!(player1["username"], player2["username"]);
    
    // Create server data
    let server = factory.create_server_data(1).await;
    assert_eq!(server["player_id"], 1);
    assert!(server["server_ip"].as_str().unwrap().starts_with("192.168.1."));
    
    // Create process data
    let process = factory.create_process_data(1, 1, "cracker").await;
    assert_eq!(process["player_id"], 1);
    assert_eq!(process["server_id"], 1);
    assert_eq!(process["process_type"], "cracker");
}

#[tokio::test]
async fn test_mock_external_service() {
    let service = MockExternalService::new("test_api")
        .with_latency(Duration::from_millis(50))
        .with_failure_rate(0.1);
    
    // Set up a mock response
    service.set_response("/api/users", json!({"users": ["alice", "bob"]})).await;
    
    // Make requests
    let mut successful = 0;
    let mut failed = 0;
    
    for i in 0..20 {
        let headers = HashMap::from([
            ("Content-Type".to_string(), "application/json".to_string()),
        ]);
        
        match service.simulate_request("GET", "/api/users", headers, None).await {
            Ok(response) => {
                successful += 1;
                assert_eq!(response["users"][0], "alice");
            }
            Err(_) => failed += 1,
        }
    }
    
    // Should have some failures due to failure rate
    assert!(failed > 0, "Expected some failures due to 10% failure rate");
    assert!(successful > 0, "Expected some successful requests");
    
    // Check request log
    let log = service.get_request_log().await;
    assert_eq!(log.len(), 20);
}

#[tokio::test]
async fn test_test_assertions() {
    // Test response time assertion
    let fast_duration = Duration::from_millis(50);
    TestAssertions::assert_response_time(fast_duration, 100, "fast operation");
    
    // Test memory usage assertion
    let memory_usage = 50 * 1024 * 1024; // 50MB
    TestAssertions::assert_memory_usage(memory_usage, 100, "memory test");
    
    // Test error rate assertion
    TestAssertions::assert_error_rate(2, 100, 5.0, "error rate test");
    
    // Test throughput assertion  
    let duration = Duration::from_secs(1);
    TestAssertions::assert_throughput(150, duration, 100.0, "throughput test");
    
    // Test JSON structure assertion
    let actual = json!({
        "user": {
            "id": 123,
            "name": "Alice",
            "email": "alice@example.com"
        },
        "metadata": {
            "version": "1.0"
        }
    });
    
    let expected_structure = json!({
        "user": {
            "id": null,
            "name": null,
            "email": null
        },
        "metadata": {}
    });
    
    TestAssertions::assert_json_structure(&actual, &expected_structure);
}

#[tokio::test]
async fn test_test_runner() {
    let mut runner = assert_ok!(TestRunner::new().await);
    
    // Run a passing test
    runner.run_test("passing_test", vec!["unit".to_string()], || async {
        Ok(())
    }).await;
    
    // Run a failing test
    runner.run_test("failing_test", vec!["integration".to_string()], || async {
        Err("Test failure".into())
    }).await;
    
    // Generate report
    let report = runner.generate_test_report();
    assert_eq!(report.summary.total_tests, 2);
    assert_eq!(report.summary.passed_tests, 1);
    assert_eq!(report.summary.failed_tests, 1);
    assert_eq!(report.summary.success_rate, 50.0);
    
    // Print summary
    report.print_summary();
    
    // Cleanup
    assert_ok!(runner.cleanup().await);
}

#[tokio::test]
async fn test_benchmark_runner() {
    let mut runner = BenchmarkRunner::new();
    
    // Run a simple benchmark
    runner.run_benchmark("simple_operation", 1000, || async {
        // Simulate some work
        for _ in 0..100 {
            let _result = 2 + 2;
        }
        Ok(())
    }).await;
    
    // Run an async benchmark
    runner.run_benchmark("async_operation", 100, || async {
        sleep(Duration::from_micros(100)).await;
        Ok(())
    }).await;
    
    let results = runner.get_results();
    assert_eq!(results.len(), 2);
    
    // First benchmark should be very fast
    assert!(results[0].ops_per_second > 1000.0);
    
    // Second benchmark should be slower due to sleep
    assert!(results[1].ops_per_second < 1000.0);
    
    runner.print_results();
}

#[tokio::test]
async fn test_ci_helpers() {
    // Test environment setup
    assert_ok!(CIHelper::setup_test_environment());
    
    // Check environment variables were set
    assert_eq!(env::var("RUST_ENV").unwrap(), "test");
    assert!(env::var("DATABASE_URL").is_ok());
    assert!(env::var("REDIS_URL").is_ok());
}

#[tokio::test]
async fn test_config_loading_and_validation() {
    let config = TestConfig::default();
    
    // Validate default configuration
    assert_eq!(config.database.host, "localhost");
    assert_eq!(config.database.port, 5432);
    assert_eq!(config.security.password_min_length, 8);
    assert!(config.security.rate_limit_max > 0);
    
    // Test serialization/deserialization
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: TestConfig = serde_json::from_str(&json).unwrap();
    
    assert_eq!(config.database.host, deserialized.database.host);
    assert_eq!(config.redis.port, deserialized.redis.port);
}

#[tokio::test]
async fn test_comprehensive_test_infrastructure() {
    // This test validates the entire test infrastructure working together
    let mut env = TestEnvironment::new();
    let factory = TestDataFactory::new();
    let mock_service = Arc::new(MockExternalService::new("comprehensive_test"));
    
    // Setup environment
    env.add_service("mock_service".to_string(), mock_service.clone());
    assert_ok!(env.setup().await);
    
    // Create test data
    let player_data = factory.create_player_data("integration").await;
    let server_data = factory.create_server_data(player_data["player_id"].as_u64().unwrap()).await;
    
    // Setup mock service responses
    mock_service.set_response("/api/validate", json!({"valid": true})).await;
    
    // Simulate service interaction
    let response = assert_ok!(mock_service.simulate_request(
        "POST", 
        "/api/validate", 
        HashMap::new(), 
        Some(player_data.to_string())
    ).await);
    
    assert_eq!(response["valid"], true);
    
    // Verify request was logged
    let log = mock_service.get_request_log().await;
    assert_eq!(log.len(), 1);
    assert_eq!(log[0].method, "POST");
    assert_eq!(log[0].path, "/api/validate");
    
    // Test assertions
    let start_time = std::time::Instant::now();
    sleep(Duration::from_millis(10)).await;
    let duration = start_time.elapsed();
    
    TestAssertions::assert_response_time(duration, 50, "mock service call");
    
    // Cleanup
    assert_ok!(env.teardown().await);
}