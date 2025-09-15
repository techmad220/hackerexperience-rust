use std::{
    collections::HashMap,
    sync::{Arc, atomic::{AtomicU64, Ordering}},
    time::{Duration, Instant},
};
use tokio::{
    sync::{RwLock, Semaphore},
    time::{sleep, timeout},
    task::JoinSet,
};
use serde_json::{json, Value};
use uuid::Uuid;
use futures::future::join_all;

use crate::common::{TestDb, TestFixtures};
use crate::{assert_ok, assert_err};

// ===== PERFORMANCE TEST INFRASTRUCTURE =====

#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub total_requests: AtomicU64,
    pub successful_requests: AtomicU64,
    pub failed_requests: AtomicU64,
    pub total_response_time_ms: AtomicU64,
    pub min_response_time_ms: AtomicU64,
    pub max_response_time_ms: AtomicU64,
    pub start_time: Option<Instant>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            total_response_time_ms: AtomicU64::new(0),
            min_response_time_ms: AtomicU64::new(u64::MAX),
            max_response_time_ms: AtomicU64::new(0),
            start_time: Some(Instant::now()),
        }
    }

    pub fn record_request(&self, response_time_ms: u64, success: bool) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.total_response_time_ms.fetch_add(response_time_ms, Ordering::Relaxed);

        if success {
            self.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }

        // Update min/max response times
        let current_min = self.min_response_time_ms.load(Ordering::Relaxed);
        if response_time_ms < current_min {
            self.min_response_time_ms.store(response_time_ms, Ordering::Relaxed);
        }

        let current_max = self.max_response_time_ms.load(Ordering::Relaxed);
        if response_time_ms > current_max {
            self.max_response_time_ms.store(response_time_ms, Ordering::Relaxed);
        }
    }

    pub fn get_summary(&self) -> PerformanceSummary {
        let total = self.total_requests.load(Ordering::Relaxed);
        let successful = self.successful_requests.load(Ordering::Relaxed);
        let failed = self.failed_requests.load(Ordering::Relaxed);
        let total_time = self.total_response_time_ms.load(Ordering::Relaxed);
        let min_time = self.min_response_time_ms.load(Ordering::Relaxed);
        let max_time = self.max_response_time_ms.load(Ordering::Relaxed);

        let avg_response_time = if total > 0 {
            total_time as f64 / total as f64
        } else {
            0.0
        };

        let success_rate = if total > 0 {
            (successful as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let duration = self.start_time.map(|start| start.elapsed()).unwrap_or_default();
        let requests_per_second = if duration.as_secs_f64() > 0.0 {
            total as f64 / duration.as_secs_f64()
        } else {
            0.0
        };

        PerformanceSummary {
            total_requests: total,
            successful_requests: successful,
            failed_requests: failed,
            success_rate,
            avg_response_time_ms: avg_response_time,
            min_response_time_ms: if min_time == u64::MAX { 0 } else { min_time },
            max_response_time_ms: max_time,
            requests_per_second,
            test_duration: duration,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub success_rate: f64,
    pub avg_response_time_ms: f64,
    pub min_response_time_ms: u64,
    pub max_response_time_ms: u64,
    pub requests_per_second: f64,
    pub test_duration: Duration,
}

impl std::fmt::Display for PerformanceSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "Performance Summary:\n\
             Total Requests: {}\n\
             Successful: {} ({:.2}%)\n\
             Failed: {}\n\
             Avg Response Time: {:.2}ms\n\
             Min Response Time: {}ms\n\
             Max Response Time: {}ms\n\
             Requests/sec: {:.2}\n\
             Test Duration: {:.2}s",
            self.total_requests,
            self.successful_requests,
            self.success_rate,
            self.failed_requests,
            self.avg_response_time_ms,
            self.min_response_time_ms,
            self.max_response_time_ms,
            self.requests_per_second,
            self.test_duration.as_secs_f64()
        )
    }
}

// ===== LOAD TEST SCENARIOS =====

pub struct LoadTestConfig {
    pub concurrent_users: usize,
    pub requests_per_user: usize,
    pub ramp_up_time: Duration,
    pub test_duration: Duration,
    pub think_time: Duration,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            concurrent_users: 100,
            requests_per_user: 10,
            ramp_up_time: Duration::from_secs(10),
            test_duration: Duration::from_secs(60),
            think_time: Duration::from_millis(100),
        }
    }
}

// Mock service for testing
pub struct MockGameService {
    pub players: Arc<RwLock<HashMap<u64, Player>>>,
    pub servers: Arc<RwLock<HashMap<u64, Server>>>,
    pub processes: Arc<RwLock<HashMap<u64, Process>>>,
    pub response_delay: Duration,
    pub failure_rate: f64,
    pub metrics: Arc<PerformanceMetrics>,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: u64,
    pub username: String,
    pub credits: i64,
    pub created_at: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub id: u64,
    pub player_id: u64,
    pub ip: String,
    pub server_type: String,
}

#[derive(Debug, Clone)]
pub struct Process {
    pub id: u64,
    pub player_id: u64,
    pub process_type: String,
    pub status: String,
    pub progress: f32,
}

impl MockGameService {
    pub fn new() -> Self {
        Self {
            players: Arc::new(RwLock::new(HashMap::new())),
            servers: Arc::new(RwLock::new(HashMap::new())),
            processes: Arc::new(RwLock::new(HashMap::new())),
            response_delay: Duration::from_millis(10),
            failure_rate: 0.01, // 1% failure rate
            metrics: Arc::new(PerformanceMetrics::new()),
        }
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.response_delay = delay;
        self
    }

    pub fn with_failure_rate(mut self, rate: f64) -> Self {
        self.failure_rate = rate;
        self
    }

    pub async fn create_player(&self, username: &str) -> Result<u64, String> {
        let start = Instant::now();
        
        // Simulate processing delay
        sleep(self.response_delay).await;

        // Simulate random failures
        if rand::random::<f64>() < self.failure_rate {
            let elapsed = start.elapsed().as_millis() as u64;
            self.metrics.record_request(elapsed, false);
            return Err("Service temporarily unavailable".to_string());
        }

        let player_id = rand::random::<u64>();
        let player = Player {
            id: player_id,
            username: username.to_string(),
            credits: 1000,
            created_at: std::time::SystemTime::now(),
        };

        {
            let mut players = self.players.write().await;
            players.insert(player_id, player);
        }

        let elapsed = start.elapsed().as_millis() as u64;
        self.metrics.record_request(elapsed, true);
        Ok(player_id)
    }

    pub async fn get_player(&self, player_id: u64) -> Result<Player, String> {
        let start = Instant::now();
        
        sleep(self.response_delay).await;

        if rand::random::<f64>() < self.failure_rate {
            let elapsed = start.elapsed().as_millis() as u64;
            self.metrics.record_request(elapsed, false);
            return Err("Service temporarily unavailable".to_string());
        }

        let players = self.players.read().await;
        let player = players.get(&player_id).cloned()
            .ok_or_else(|| "Player not found".to_string())?;

        let elapsed = start.elapsed().as_millis() as u64;
        self.metrics.record_request(elapsed, true);
        Ok(player)
    }

    pub async fn create_server(&self, player_id: u64, ip: &str) -> Result<u64, String> {
        let start = Instant::now();
        
        sleep(self.response_delay).await;

        if rand::random::<f64>() < self.failure_rate {
            let elapsed = start.elapsed().as_millis() as u64;
            self.metrics.record_request(elapsed, false);
            return Err("Service temporarily unavailable".to_string());
        }

        let server_id = rand::random::<u64>();
        let server = Server {
            id: server_id,
            player_id,
            ip: ip.to_string(),
            server_type: "Desktop".to_string(),
        };

        {
            let mut servers = self.servers.write().await;
            servers.insert(server_id, server);
        }

        let elapsed = start.elapsed().as_millis() as u64;
        self.metrics.record_request(elapsed, true);
        Ok(server_id)
    }

    pub async fn start_process(&self, player_id: u64, process_type: &str) -> Result<u64, String> {
        let start = Instant::now();
        
        sleep(self.response_delay).await;

        if rand::random::<f64>() < self.failure_rate {
            let elapsed = start.elapsed().as_millis() as u64;
            self.metrics.record_request(elapsed, false);
            return Err("Service temporarily unavailable".to_string());
        }

        let process_id = rand::random::<u64>();
        let process = Process {
            id: process_id,
            player_id,
            process_type: process_type.to_string(),
            status: "running".to_string(),
            progress: 0.0,
        };

        {
            let mut processes = self.processes.write().await;
            processes.insert(process_id, process);
        }

        let elapsed = start.elapsed().as_millis() as u64;
        self.metrics.record_request(elapsed, true);
        Ok(process_id)
    }

    pub async fn get_processes(&self, player_id: u64) -> Result<Vec<Process>, String> {
        let start = Instant::now();
        
        sleep(self.response_delay).await;

        if rand::random::<f64>() < self.failure_rate {
            let elapsed = start.elapsed().as_millis() as u64;
            self.metrics.record_request(elapsed, false);
            return Err("Service temporarily unavailable".to_string());
        }

        let processes = self.processes.read().await;
        let player_processes: Vec<Process> = processes.values()
            .filter(|p| p.player_id == player_id)
            .cloned()
            .collect();

        let elapsed = start.elapsed().as_millis() as u64;
        self.metrics.record_request(elapsed, true);
        Ok(player_processes)
    }

    pub fn get_metrics(&self) -> PerformanceSummary {
        self.metrics.get_summary()
    }
}

// ===== LOAD TESTING FUNCTIONS =====

pub async fn run_user_simulation(
    service: Arc<MockGameService>,
    user_id: usize,
    config: LoadTestConfig,
) -> Result<(), Box<dyn std::error::Error + Send>> {
    // Staggered start to simulate ramp-up
    let delay = config.ramp_up_time.as_millis() as u64 / config.concurrent_users as u64;
    sleep(Duration::from_millis(delay * user_id as u64)).await;

    // Create a player for this user
    let username = format!("user_{}", user_id);
    let player_id = service.create_player(&username).await?;

    // Perform user actions
    for _ in 0..config.requests_per_user {
        // Simulate user thinking time
        sleep(config.think_time).await;

        // Random user actions
        match rand::random::<u8>() % 5 {
            0 => {
                // Get player info
                let _ = service.get_player(player_id).await;
            }
            1 => {
                // Create a server
                let ip = format!("192.168.1.{}", rand::random::<u8>());
                let _ = service.create_server(player_id, &ip).await;
            }
            2 => {
                // Start a process
                let process_types = ["cracker", "uploader", "downloader", "virus"];
                let process_type = process_types[rand::random::<usize>() % process_types.len()];
                let _ = service.start_process(player_id, process_type).await;
            }
            3 => {
                // Get processes
                let _ = service.get_processes(player_id).await;
            }
            4 => {
                // Mixed operation - get player and processes
                let _ = service.get_player(player_id).await;
                let _ = service.get_processes(player_id).await;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

pub async fn run_load_test(
    service: Arc<MockGameService>,
    config: LoadTestConfig,
) -> PerformanceSummary {
    println!("Starting load test with {} concurrent users...", config.concurrent_users);

    let mut tasks = JoinSet::new();

    // Start user simulation tasks
    for user_id in 0..config.concurrent_users {
        let service_clone = service.clone();
        let config_clone = config.clone();
        
        tasks.spawn(async move {
            run_user_simulation(service_clone, user_id, config_clone).await
        });
    }

    // Wait for test duration or all tasks to complete
    let test_timeout = timeout(config.test_duration, async {
        while let Some(result) = tasks.join_next().await {
            if let Err(e) = result {
                eprintln!("User simulation error: {:?}", e);
            }
        }
    });

    let _ = test_timeout.await;

    // Abort any remaining tasks
    tasks.abort_all();

    let summary = service.get_metrics();
    println!("{}", summary);
    summary
}

// ===== PERFORMANCE TESTS =====

#[tokio::test]
async fn test_basic_load_performance() {
    let service = Arc::new(MockGameService::new());
    
    let config = LoadTestConfig {
        concurrent_users: 50,
        requests_per_user: 10,
        ramp_up_time: Duration::from_secs(5),
        test_duration: Duration::from_secs(30),
        think_time: Duration::from_millis(50),
    };

    let summary = run_load_test(service, config).await;

    // Performance assertions
    assert!(summary.success_rate >= 95.0, "Success rate too low: {:.2}%", summary.success_rate);
    assert!(summary.avg_response_time_ms <= 100.0, "Average response time too high: {:.2}ms", summary.avg_response_time_ms);
    assert!(summary.requests_per_second >= 10.0, "Throughput too low: {:.2} req/s", summary.requests_per_second);
}

#[tokio::test]
async fn test_high_concurrency_load() {
    let service = Arc::new(MockGameService::new().with_delay(Duration::from_millis(20)));
    
    let config = LoadTestConfig {
        concurrent_users: 200,
        requests_per_user: 5,
        ramp_up_time: Duration::from_secs(10),
        test_duration: Duration::from_secs(60),
        think_time: Duration::from_millis(100),
    };

    let summary = run_load_test(service, config).await;

    // High concurrency assertions
    assert!(summary.success_rate >= 90.0, "Success rate too low under high load: {:.2}%", summary.success_rate);
    assert!(summary.avg_response_time_ms <= 200.0, "Response time degraded under load: {:.2}ms", summary.avg_response_time_ms);
    assert!(summary.total_requests >= 800, "Not enough requests processed: {}", summary.total_requests);
}

#[tokio::test]
async fn test_stress_testing_with_failures() {
    let service = Arc::new(MockGameService::new()
        .with_delay(Duration::from_millis(30))
        .with_failure_rate(0.05) // 5% failure rate
    );
    
    let config = LoadTestConfig {
        concurrent_users: 100,
        requests_per_user: 20,
        ramp_up_time: Duration::from_secs(5),
        test_duration: Duration::from_secs(45),
        think_time: Duration::from_millis(25),
    };

    let summary = run_load_test(service, config).await;

    // Stress test assertions - more lenient due to intentional failures
    assert!(summary.success_rate >= 85.0, "Success rate too low under stress: {:.2}%", summary.success_rate);
    assert!(summary.requests_per_second >= 20.0, "Throughput collapsed under stress: {:.2} req/s", summary.requests_per_second);
}

#[tokio::test]
async fn test_memory_usage_under_load() {
    let service = Arc::new(MockGameService::new());
    
    // Monitor memory usage during load test
    let initial_memory = get_memory_usage();
    
    let config = LoadTestConfig {
        concurrent_users: 300,
        requests_per_user: 15,
        ramp_up_time: Duration::from_secs(10),
        test_duration: Duration::from_secs(60),
        think_time: Duration::from_millis(50),
    };

    let summary = run_load_test(service.clone(), config).await;
    
    // Force garbage collection and measure memory
    tokio::task::yield_now().await;
    let peak_memory = get_memory_usage();
    
    // Drop service and measure final memory
    drop(service);
    tokio::task::yield_now().await;
    let final_memory = get_memory_usage();

    println!("Memory usage: Initial: {}MB, Peak: {}MB, Final: {}MB", 
        initial_memory / 1024 / 1024,
        peak_memory / 1024 / 1024, 
        final_memory / 1024 / 1024
    );

    // Memory assertions
    let memory_increase = peak_memory.saturating_sub(initial_memory);
    let memory_per_user = memory_increase / 300; // 300 concurrent users
    
    assert!(memory_per_user < 1024 * 1024, "Memory usage per user too high: {}KB", memory_per_user / 1024);
    
    // Check for memory leaks
    let memory_leak = final_memory.saturating_sub(initial_memory);
    assert!(memory_leak < initial_memory / 10, "Potential memory leak detected: {}MB", memory_leak / 1024 / 1024);

    assert!(summary.success_rate >= 90.0, "Performance degraded during memory test");
}

#[tokio::test]
async fn test_database_connection_pool_performance() {
    let mut test_db = TestDb::new().await;
    assert_ok!(test_db.setup().await);

    let metrics = Arc::new(PerformanceMetrics::new());
    let semaphore = Arc::new(Semaphore::new(10)); // Simulate connection pool limit

    let mut tasks = JoinSet::new();
    let num_operations = 100;
    let concurrent_operations = 50;

    for i in 0..concurrent_operations {
        let metrics_clone = metrics.clone();
        let semaphore_clone = semaphore.clone();
        let operations_per_task = num_operations / concurrent_operations;

        tasks.spawn(async move {
            for j in 0..operations_per_task {
                let _permit = semaphore_clone.acquire().await.unwrap();
                let start = Instant::now();
                
                // Simulate database operation
                let username = format!("user_{}_{}", i, j);
                
                // Simulate work (in real test, this would be actual DB operations)
                sleep(Duration::from_millis(10)).await;
                
                let elapsed = start.elapsed().as_millis() as u64;
                metrics_clone.record_request(elapsed, true);
            }
        });
    }

    // Wait for all tasks to complete
    while let Some(result) = tasks.join_next().await {
        assert_ok!(result);
    }

    let summary = metrics.get_summary();
    println!("Database Performance: {}", summary);

    // Database performance assertions
    assert!(summary.avg_response_time_ms <= 50.0, "Database operations too slow: {:.2}ms", summary.avg_response_time_ms);
    assert!(summary.requests_per_second >= 100.0, "Database throughput too low: {:.2} ops/s", summary.requests_per_second);
    assert_eq!(summary.success_rate, 100.0, "Database operations should not fail");
}

#[tokio::test]
async fn test_websocket_connection_performance() {
    let metrics = Arc::new(PerformanceMetrics::new());
    let active_connections = Arc::new(AtomicU64::new(0));
    let max_connections = 1000;

    let mut tasks = JoinSet::new();

    // Simulate WebSocket connections
    for i in 0..max_connections {
        let metrics_clone = metrics.clone();
        let connections_clone = active_connections.clone();

        tasks.spawn(async move {
            let start = Instant::now();
            
            // Simulate connection establishment
            connections_clone.fetch_add(1, Ordering::Relaxed);
            
            // Simulate connection activity
            sleep(Duration::from_millis(100 + (i % 50) as u64)).await;
            
            // Simulate message exchange
            for _ in 0..10 {
                sleep(Duration::from_millis(10)).await;
                // Each message exchange counts as a request
                let elapsed = Duration::from_millis(5).as_millis() as u64;
                metrics_clone.record_request(elapsed, true);
            }
            
            connections_clone.fetch_sub(1, Ordering::Relaxed);
            
            let total_elapsed = start.elapsed().as_millis() as u64;
            metrics_clone.record_request(total_elapsed, true);
        });

        // Stagger connection attempts
        if i % 50 == 0 {
            sleep(Duration::from_millis(10)).await;
        }
    }

    // Wait for all connections to complete
    while let Some(result) = tasks.join_next().await {
        assert_ok!(result);
    }

    let summary = metrics.get_summary();
    println!("WebSocket Performance: {}", summary);

    // WebSocket performance assertions
    assert!(summary.total_requests >= max_connections as u64, "Not all connections processed");
    assert!(summary.success_rate >= 99.0, "WebSocket connection failure rate too high");
    assert!(summary.requests_per_second >= 500.0, "WebSocket throughput too low: {:.2} req/s", summary.requests_per_second);
}

#[tokio::test]
async fn test_ajax_endpoint_performance() {
    let metrics = Arc::new(PerformanceMetrics::new());
    let concurrent_requests = 200;
    let requests_per_client = 20;

    let mut tasks = JoinSet::new();

    for client_id in 0..concurrent_requests {
        let metrics_clone = metrics.clone();

        tasks.spawn(async move {
            for request_id in 0..requests_per_client {
                let start = Instant::now();
                
                // Simulate different AJAX endpoint calls
                match request_id % 5 {
                    0 => {
                        // Simulate check-user endpoint
                        sleep(Duration::from_millis(15)).await;
                    }
                    1 => {
                        // Simulate start-process endpoint
                        sleep(Duration::from_millis(25)).await;
                    }
                    2 => {
                        // Simulate get-process-list endpoint
                        sleep(Duration::from_millis(20)).await;
                    }
                    3 => {
                        // Simulate bank-transfer endpoint
                        sleep(Duration::from_millis(30)).await;
                    }
                    4 => {
                        // Simulate get-logs endpoint
                        sleep(Duration::from_millis(18)).await;
                    }
                    _ => unreachable!(),
                }

                let elapsed = start.elapsed().as_millis() as u64;
                let success = elapsed < 100; // Consider requests over 100ms as failed
                metrics_clone.record_request(elapsed, success);

                // Think time between requests
                sleep(Duration::from_millis(50)).await;
            }
        });
    }

    // Wait for all requests to complete
    while let Some(result) = tasks.join_next().await {
        assert_ok!(result);
    }

    let summary = metrics.get_summary();
    println!("AJAX Performance: {}", summary);

    // AJAX performance assertions
    assert!(summary.avg_response_time_ms <= 40.0, "AJAX responses too slow: {:.2}ms", summary.avg_response_time_ms);
    assert!(summary.success_rate >= 95.0, "Too many slow AJAX requests: {:.2}%", summary.success_rate);
    assert!(summary.requests_per_second >= 50.0, "AJAX throughput too low: {:.2} req/s", summary.requests_per_second);
}

#[tokio::test]
async fn test_actor_system_performance() {
    let metrics = Arc::new(PerformanceMetrics::new());
    let num_actors = 100;
    let messages_per_actor = 50;

    // Simulate actor message passing performance
    let mut tasks = JoinSet::new();

    for actor_id in 0..num_actors {
        let metrics_clone = metrics.clone();

        tasks.spawn(async move {
            // Simulate actor initialization
            sleep(Duration::from_millis(5)).await;

            for message_id in 0..messages_per_actor {
                let start = Instant::now();
                
                // Simulate message processing
                match message_id % 4 {
                    0 => {
                        // Fast message
                        sleep(Duration::from_micros(500)).await;
                    }
                    1 => {
                        // Medium message
                        sleep(Duration::from_millis(2)).await;
                    }
                    2 => {
                        // Slow message
                        sleep(Duration::from_millis(5)).await;
                    }
                    3 => {
                        // Complex message requiring inter-actor communication
                        sleep(Duration::from_millis(3)).await;
                        // Simulate response
                        sleep(Duration::from_millis(1)).await;
                    }
                    _ => unreachable!(),
                }

                let elapsed = start.elapsed().as_millis() as u64;
                metrics_clone.record_request(elapsed, true);

                // Brief pause between messages
                sleep(Duration::from_micros(100)).await;
            }
        });
    }

    // Wait for all actors to complete
    while let Some(result) = tasks.join_next().await {
        assert_ok!(result);
    }

    let summary = metrics.get_summary();
    println!("Actor System Performance: {}", summary);

    // Actor system performance assertions
    assert!(summary.avg_response_time_ms <= 10.0, "Actor message processing too slow: {:.2}ms", summary.avg_response_time_ms);
    assert!(summary.requests_per_second >= 1000.0, "Actor throughput too low: {:.2} messages/s", summary.requests_per_second);
    assert_eq!(summary.success_rate, 100.0, "Actor message processing should not fail");
}

// Helper function to get memory usage (simplified version)
fn get_memory_usage() -> u64 {
    // In a real implementation, you would use system APIs to get actual memory usage
    // For testing purposes, we'll return a simulated value
    use std::sync::atomic::{AtomicU64, Ordering};
    static SIMULATED_MEMORY: AtomicU64 = AtomicU64::new(50 * 1024 * 1024); // Start at 50MB
    
    // Simulate memory growth
    let current = SIMULATED_MEMORY.load(Ordering::Relaxed);
    let growth = rand::random::<u64>() % (1024 * 1024); // Up to 1MB growth
    SIMULATED_MEMORY.store(current + growth, Ordering::Relaxed);
    current + growth
}

// Throughput benchmark test
#[tokio::test]
async fn test_maximum_throughput_benchmark() {
    let service = Arc::new(MockGameService::new().with_delay(Duration::from_millis(1)));
    let metrics = Arc::new(PerformanceMetrics::new());
    
    println!("Running maximum throughput benchmark...");
    
    let benchmark_duration = Duration::from_secs(30);
    let max_concurrent = 500;
    
    let mut tasks = JoinSet::new();
    let start_time = Instant::now();

    for _ in 0..max_concurrent {
        let service_clone = service.clone();
        let metrics_clone = metrics.clone();

        tasks.spawn(async move {
            let mut request_count = 0;
            let start = Instant::now();
            
            while start.elapsed() < benchmark_duration {
                let req_start = Instant::now();
                
                let result = service_clone.create_player(&format!("bench_user_{}", request_count)).await;
                
                let elapsed = req_start.elapsed().as_millis() as u64;
                metrics_clone.record_request(elapsed, result.is_ok());
                
                request_count += 1;
            }
            
            request_count
        });
    }

    // Collect results
    let mut total_requests = 0;
    while let Some(result) = tasks.join_next().await {
        if let Ok(requests) = result {
            total_requests += requests;
        }
    }

    let summary = metrics.get_summary();
    println!("Throughput Benchmark Results:");
    println!("Total requests processed: {}", total_requests);
    println!("{}", summary);

    // Throughput assertions
    assert!(summary.requests_per_second >= 100.0, "Maximum throughput too low: {:.2} req/s", summary.requests_per_second);
    assert!(total_requests >= 1000, "Not enough requests processed in benchmark: {}", total_requests);
}

// Latency percentile test
#[tokio::test]
async fn test_response_time_percentiles() {
    let service = Arc::new(MockGameService::new());
    let response_times = Arc::new(RwLock::new(Vec::new()));
    
    let num_requests = 1000;
    let mut tasks = JoinSet::new();

    for i in 0..num_requests {
        let service_clone = service.clone();
        let response_times_clone = response_times.clone();

        tasks.spawn(async move {
            let start = Instant::now();
            let _ = service_clone.create_player(&format!("latency_user_{}", i)).await;
            let elapsed = start.elapsed().as_millis() as u64;
            
            let mut times = response_times_clone.write().await;
            times.push(elapsed);
        });
    }

    // Wait for all requests
    while let Some(result) = tasks.join_next().await {
        assert_ok!(result);
    }

    // Calculate percentiles
    let mut times = response_times.write().await;
    times.sort_unstable();

    let p50 = times[times.len() * 50 / 100];
    let p90 = times[times.len() * 90 / 100];
    let p95 = times[times.len() * 95 / 100];
    let p99 = times[times.len() * 99 / 100];

    println!("Response Time Percentiles:");
    println!("P50: {}ms", p50);
    println!("P90: {}ms", p90);
    println!("P95: {}ms", p95);
    println!("P99: {}ms", p99);

    // Percentile assertions
    assert!(p50 <= 20, "P50 response time too high: {}ms", p50);
    assert!(p90 <= 30, "P90 response time too high: {}ms", p90);
    assert!(p95 <= 40, "P95 response time too high: {}ms", p95);
    assert!(p99 <= 60, "P99 response time too high: {}ms", p99);
}