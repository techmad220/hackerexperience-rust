//! Monitoring, metrics, and observability for HackerExperience

use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec,
    Encoder, TextEncoder, register_counter, register_counter_vec,
    register_gauge, register_gauge_vec, register_histogram, register_histogram_vec,
};
use sysinfo::{System, SystemExt, CpuExt, DiskExt, NetworkExt};
use std::time::Duration;
use tracing::{info, warn, error};

lazy_static::lazy_static! {
    // ===========================================
    // HTTP Metrics
    // ===========================================

    static ref HTTP_REQUESTS: CounterVec = register_counter_vec!(
        "http_requests_total",
        "Total HTTP requests",
        &["method", "endpoint", "status"]
    ).unwrap();

    static ref HTTP_DURATION: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "HTTP request duration",
        &["method", "endpoint"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    ).unwrap();

    static ref ACTIVE_CONNECTIONS: Gauge = register_gauge!(
        "active_connections",
        "Number of active connections"
    ).unwrap();

    // ===========================================
    // Game Metrics
    // ===========================================

    static ref ONLINE_PLAYERS: Gauge = register_gauge!(
        "online_players_total",
        "Total online players"
    ).unwrap();

    static ref ACTIVE_PROCESSES: GaugeVec = register_gauge_vec!(
        "active_processes",
        "Active game processes by type",
        &["process_type"]
    ).unwrap();

    static ref HACKS_ATTEMPTED: Counter = register_counter!(
        "hacks_attempted_total",
        "Total hack attempts"
    ).unwrap();

    static ref HACKS_SUCCESSFUL: Counter = register_counter!(
        "hacks_successful_total",
        "Successful hacks"
    ).unwrap();

    static ref PVP_MATCHES: Counter = register_counter!(
        "pvp_matches_total",
        "Total PvP matches"
    ).unwrap();

    static ref CHAT_MESSAGES: CounterVec = register_counter_vec!(
        "chat_messages_total",
        "Chat messages sent",
        &["room"]
    ).unwrap();

    static ref TRANSACTIONS: CounterVec = register_counter_vec!(
        "transactions_total",
        "Game transactions",
        &["type"]
    ).unwrap();

    // ===========================================
    // Database Metrics
    // ===========================================

    static ref DB_CONNECTIONS: Gauge = register_gauge!(
        "database_connections_active",
        "Active database connections"
    ).unwrap();

    static ref DB_QUERY_DURATION: HistogramVec = register_histogram_vec!(
        "database_query_duration_seconds",
        "Database query duration",
        &["query_type"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]
    ).unwrap();

    // ===========================================
    // System Metrics
    // ===========================================

    static ref CPU_USAGE: Gauge = register_gauge!(
        "system_cpu_usage_percent",
        "CPU usage percentage"
    ).unwrap();

    static ref MEMORY_USAGE: Gauge = register_gauge!(
        "system_memory_usage_bytes",
        "Memory usage in bytes"
    ).unwrap();

    static ref DISK_USAGE: GaugeVec = register_gauge_vec!(
        "system_disk_usage_bytes",
        "Disk usage by mount point",
        &["mount_point"]
    ).unwrap();
}

/// Main monitoring service
pub struct MonitoringService {
    sentry_guard: Option<sentry::ClientInitGuard>,
    system: System,
}

impl MonitoringService {
    /// Initialize monitoring
    pub fn init(sentry_dsn: Option<String>) -> Self {
        // Initialize tracing
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .json()
            .init();

        // Initialize Sentry if DSN provided
        let sentry_guard = sentry_dsn.map(|dsn| {
            sentry::init((
                dsn,
                sentry::ClientOptions {
                    release: sentry::release_name!(),
                    traces_sample_rate: 0.1,
                    enable_profiling: true,
                    profiles_sample_rate: 0.1,
                    ..Default::default()
                }
            ))
        });

        info!("Monitoring service initialized");

        Self {
            sentry_guard,
            system: System::new_all(),
        }
    }

    /// Start metrics collection
    pub async fn start_collection(&mut self) {
        loop {
            self.collect_system_metrics();
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }

    /// Collect system metrics
    fn collect_system_metrics(&mut self) {
        self.system.refresh_all();

        // CPU usage
        let cpu_usage = self.system.global_cpu_info().cpu_usage();
        CPU_USAGE.set(cpu_usage as f64);

        // Memory usage
        let memory_used = self.system.used_memory();
        MEMORY_USAGE.set(memory_used as f64);

        // Disk usage
        for disk in self.system.disks() {
            let mount_point = disk.mount_point().to_string_lossy();
            let usage = disk.total_space() - disk.available_space();
            DISK_USAGE.with_label_values(&[&mount_point]).set(usage as f64);
        }
    }

    /// Export metrics for Prometheus
    pub fn export_metrics() -> String {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

/// Request tracking middleware
pub struct RequestTracker;

impl RequestTracker {
    /// Track HTTP request
    pub fn track_request(method: &str, endpoint: &str, status: u16, duration: Duration) {
        HTTP_REQUESTS
            .with_label_values(&[method, endpoint, &status.to_string()])
            .inc();

        HTTP_DURATION
            .with_label_values(&[method, endpoint])
            .observe(duration.as_secs_f64());
    }

    /// Track WebSocket connection
    pub fn track_connection(connected: bool) {
        if connected {
            ACTIVE_CONNECTIONS.inc();
        } else {
            ACTIVE_CONNECTIONS.dec();
        }
    }
}

/// Game metrics tracker
pub struct GameMetrics;

impl GameMetrics {
    /// Track player login
    pub fn player_login() {
        ONLINE_PLAYERS.inc();
    }

    /// Track player logout
    pub fn player_logout() {
        ONLINE_PLAYERS.dec();
    }

    /// Track hack attempt
    pub fn hack_attempt(successful: bool) {
        HACKS_ATTEMPTED.inc();
        if successful {
            HACKS_SUCCESSFUL.inc();
        }
    }

    /// Track PvP match
    pub fn pvp_match() {
        PVP_MATCHES.inc();
    }

    /// Track chat message
    pub fn chat_message(room: &str) {
        CHAT_MESSAGES.with_label_values(&[room]).inc();
    }

    /// Track transaction
    pub fn transaction(transaction_type: &str) {
        TRANSACTIONS.with_label_values(&[transaction_type]).inc();
    }

    /// Track active process
    pub fn process_started(process_type: &str) {
        ACTIVE_PROCESSES.with_label_values(&[process_type]).inc();
    }

    /// Track process completion
    pub fn process_completed(process_type: &str) {
        ACTIVE_PROCESSES.with_label_values(&[process_type]).dec();
    }
}

/// Database metrics tracker
pub struct DatabaseMetrics;

impl DatabaseMetrics {
    /// Track query execution
    pub fn track_query(query_type: &str, duration: Duration) {
        DB_QUERY_DURATION
            .with_label_values(&[query_type])
            .observe(duration.as_secs_f64());
    }

    /// Update connection pool metrics
    pub fn update_connections(active: usize) {
        DB_CONNECTIONS.set(active as f64);
    }
}

/// Health check service
pub struct HealthCheck {
    checks: Vec<Box<dyn Fn() -> HealthStatus + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub name: String,
    pub healthy: bool,
    pub message: String,
}

impl HealthCheck {
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
        }
    }

    /// Add health check
    pub fn add_check<F>(&mut self, check: F)
    where
        F: Fn() -> HealthStatus + Send + Sync + 'static,
    {
        self.checks.push(Box::new(check));
    }

    /// Run all health checks
    pub fn check_health(&self) -> Vec<HealthStatus> {
        self.checks.iter().map(|check| check()).collect()
    }

    /// Overall health status
    pub fn is_healthy(&self) -> bool {
        self.check_health().iter().all(|s| s.healthy)
    }
}

/// Alert manager
pub struct AlertManager;

impl AlertManager {
    /// Send critical alert
    pub fn critical(message: &str) {
        error!("CRITICAL ALERT: {}", message);
        if sentry::Hub::current().client().is_some() {
            sentry::capture_message(message, sentry::Level::Fatal);
        }
    }

    /// Send warning alert
    pub fn warning(message: &str) {
        warn!("WARNING ALERT: {}", message);
        if sentry::Hub::current().client().is_some() {
            sentry::capture_message(message, sentry::Level::Warning);
        }
    }

    /// Send info alert
    pub fn info(message: &str) {
        info!("INFO ALERT: {}", message);
    }

    /// Check thresholds and alert
    pub fn check_thresholds() {
        // CPU threshold
        if CPU_USAGE.get() > 90.0 {
            Self::critical("CPU usage above 90%");
        } else if CPU_USAGE.get() > 75.0 {
            Self::warning("CPU usage above 75%");
        }

        // Memory threshold
        let memory_gb = MEMORY_USAGE.get() / (1024.0 * 1024.0 * 1024.0);
        if memory_gb > 14.0 {  // Assuming 16GB system
            Self::critical("Memory usage critical");
        }

        // Connection threshold
        if ACTIVE_CONNECTIONS.get() > 10000.0 {
            Self::warning("High number of active connections");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check() {
        let mut health = HealthCheck::new();

        health.add_check(|| HealthStatus {
            name: "database".to_string(),
            healthy: true,
            message: "Database connected".to_string(),
        });

        let status = health.check_health();
        assert!(!status.is_empty());
        assert!(health.is_healthy());
    }

    #[test]
    fn test_metrics() {
        GameMetrics::player_login();
        assert!(ONLINE_PLAYERS.get() > 0.0);

        GameMetrics::player_logout();
    }
}