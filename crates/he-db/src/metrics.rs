//! Database metrics and monitoring

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Database metrics collector
#[derive(Debug, Clone)]
pub struct DatabaseMetrics {
    inner: Arc<MetricsInner>,
}

#[derive(Debug)]
struct MetricsInner {
    // Connection metrics
    connections_created: AtomicU64,
    connections_closed: AtomicU64,
    connections_failed: AtomicU64,
    
    // Query metrics
    queries_executed: AtomicU64,
    queries_failed: AtomicU64,
    query_duration_total: AtomicU64, // in microseconds
    
    // Transaction metrics
    transactions_started: AtomicU64,
    transactions_committed: AtomicU64,
    transactions_rolled_back: AtomicU64,
    
    // Pool metrics
    pool_stats: RwLock<PoolMetrics>,
    
    // Performance metrics
    slow_queries: AtomicU64,
    deadlocks: AtomicU64,
    
    // Historical data
    historical_data: RwLock<Vec<MetricsSnapshot>>,
    max_history_size: usize,
}

impl Default for DatabaseMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabaseMetrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            inner: Arc::new(MetricsInner {
                connections_created: AtomicU64::new(0),
                connections_closed: AtomicU64::new(0),
                connections_failed: AtomicU64::new(0),
                queries_executed: AtomicU64::new(0),
                queries_failed: AtomicU64::new(0),
                query_duration_total: AtomicU64::new(0),
                transactions_started: AtomicU64::new(0),
                transactions_committed: AtomicU64::new(0),
                transactions_rolled_back: AtomicU64::new(0),
                pool_stats: RwLock::new(PoolMetrics::default()),
                slow_queries: AtomicU64::new(0),
                deadlocks: AtomicU64::new(0),
                historical_data: RwLock::new(Vec::new()),
                max_history_size: 1000,
            }),
        }
    }

    /// Record a new connection
    pub fn record_connection_created(&self) {
        self.inner.connections_created.fetch_add(1, Ordering::Relaxed);
        debug!("Database connection created");
    }

    /// Record a closed connection
    pub fn record_connection_closed(&self) {
        self.inner.connections_closed.fetch_add(1, Ordering::Relaxed);
        debug!("Database connection closed");
    }

    /// Record a failed connection
    pub fn record_connection_failed(&self) {
        self.inner.connections_failed.fetch_add(1, Ordering::Relaxed);
        debug!("Database connection failed");
    }

    /// Record a query execution
    pub fn record_query(&self, duration: Duration, success: bool) {
        if success {
            self.inner.queries_executed.fetch_add(1, Ordering::Relaxed);
            self.inner.query_duration_total.fetch_add(
                duration.as_micros() as u64,
                Ordering::Relaxed,
            );
            
            // Check for slow queries (over 1 second)
            if duration.as_secs() >= 1 {
                self.inner.slow_queries.fetch_add(1, Ordering::Relaxed);
                debug!("Slow query detected: {:?}", duration);
            }
        } else {
            self.inner.queries_failed.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record a transaction start
    pub fn record_transaction_started(&self) {
        self.inner.transactions_started.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a transaction commit
    pub fn record_transaction_committed(&self) {
        self.inner.transactions_committed.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a transaction rollback
    pub fn record_transaction_rolled_back(&self) {
        self.inner.transactions_rolled_back.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a deadlock
    pub fn record_deadlock(&self) {
        self.inner.deadlocks.fetch_add(1, Ordering::Relaxed);
        debug!("Database deadlock recorded");
    }

    /// Update pool statistics
    pub async fn update_pool_stats(&self, stats: PoolMetrics) {
        let mut pool_stats = self.inner.pool_stats.write().await;
        *pool_stats = stats;
    }

    /// Get current metrics snapshot
    pub async fn get_current_metrics(&self) -> MetricsSnapshot {
        let pool_stats = self.inner.pool_stats.read().await.clone();
        
        MetricsSnapshot {
            timestamp: chrono::Utc::now(),
            connections_created: self.inner.connections_created.load(Ordering::Relaxed),
            connections_closed: self.inner.connections_closed.load(Ordering::Relaxed),
            connections_failed: self.inner.connections_failed.load(Ordering::Relaxed),
            queries_executed: self.inner.queries_executed.load(Ordering::Relaxed),
            queries_failed: self.inner.queries_failed.load(Ordering::Relaxed),
            query_duration_total: self.inner.query_duration_total.load(Ordering::Relaxed),
            transactions_started: self.inner.transactions_started.load(Ordering::Relaxed),
            transactions_committed: self.inner.transactions_committed.load(Ordering::Relaxed),
            transactions_rolled_back: self.inner.transactions_rolled_back.load(Ordering::Relaxed),
            slow_queries: self.inner.slow_queries.load(Ordering::Relaxed),
            deadlocks: self.inner.deadlocks.load(Ordering::Relaxed),
            pool_stats,
        }
    }

    /// Get derived metrics (calculated from current metrics)
    pub async fn get_derived_metrics(&self) -> DerivedMetrics {
        let snapshot = self.get_current_metrics().await;
        
        let average_query_duration = if snapshot.queries_executed > 0 {
            Duration::from_micros(snapshot.query_duration_total / snapshot.queries_executed)
        } else {
            Duration::from_micros(0)
        };

        let query_success_rate = if snapshot.queries_executed + snapshot.queries_failed > 0 {
            snapshot.queries_executed as f64 / (snapshot.queries_executed + snapshot.queries_failed) as f64
        } else {
            1.0
        };

        let connection_success_rate = if snapshot.connections_created + snapshot.connections_failed > 0 {
            snapshot.connections_created as f64 / (snapshot.connections_created + snapshot.connections_failed) as f64
        } else {
            1.0
        };

        let active_connections = snapshot.connections_created.saturating_sub(snapshot.connections_closed);

        DerivedMetrics {
            average_query_duration,
            query_success_rate,
            connection_success_rate,
            active_connections,
            pool_utilization: if snapshot.pool_stats.max_connections > 0 {
                snapshot.pool_stats.active_connections as f64 / snapshot.pool_stats.max_connections as f64
            } else {
                0.0
            },
        }
    }

    /// Store current metrics in history
    pub async fn take_snapshot(&self) {
        let snapshot = self.get_current_metrics().await;
        let mut history = self.inner.historical_data.write().await;
        
        history.push(snapshot);
        
        // Maintain history size limit
        if history.len() > self.inner.max_history_size {
            history.remove(0);
        }
        
        debug!("Metrics snapshot taken, history size: {}", history.len());
    }

    /// Get historical metrics
    pub async fn get_historical_metrics(&self, limit: Option<usize>) -> Vec<MetricsSnapshot> {
        let history = self.inner.historical_data.read().await;
        
        if let Some(limit) = limit {
            let start_index = if history.len() > limit {
                history.len() - limit
            } else {
                0
            };
            history[start_index..].to_vec()
        } else {
            history.clone()
        }
    }

    /// Reset all metrics
    pub async fn reset(&self) {
        self.inner.connections_created.store(0, Ordering::Relaxed);
        self.inner.connections_closed.store(0, Ordering::Relaxed);
        self.inner.connections_failed.store(0, Ordering::Relaxed);
        self.inner.queries_executed.store(0, Ordering::Relaxed);
        self.inner.queries_failed.store(0, Ordering::Relaxed);
        self.inner.query_duration_total.store(0, Ordering::Relaxed);
        self.inner.transactions_started.store(0, Ordering::Relaxed);
        self.inner.transactions_committed.store(0, Ordering::Relaxed);
        self.inner.transactions_rolled_back.store(0, Ordering::Relaxed);
        self.inner.slow_queries.store(0, Ordering::Relaxed);
        self.inner.deadlocks.store(0, Ordering::Relaxed);
        
        let mut pool_stats = self.inner.pool_stats.write().await;
        *pool_stats = PoolMetrics::default();
        
        let mut history = self.inner.historical_data.write().await;
        history.clear();
        
        info!("Database metrics reset");
    }
}

/// Pool-specific metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PoolMetrics {
    pub max_connections: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub pending_acquisitions: u32,
}

/// Point-in-time metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub connections_created: u64,
    pub connections_closed: u64,
    pub connections_failed: u64,
    pub queries_executed: u64,
    pub queries_failed: u64,
    pub query_duration_total: u64, // microseconds
    pub transactions_started: u64,
    pub transactions_committed: u64,
    pub transactions_rolled_back: u64,
    pub slow_queries: u64,
    pub deadlocks: u64,
    pub pool_stats: PoolMetrics,
}

/// Derived metrics calculated from raw metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedMetrics {
    pub average_query_duration: Duration,
    pub query_success_rate: f64,
    pub connection_success_rate: f64,
    pub active_connections: u64,
    pub pool_utilization: f64,
}

/// Multi-database metrics aggregator
#[derive(Debug)]
pub struct MultiDatabaseMetrics {
    databases: RwLock<HashMap<String, DatabaseMetrics>>,
}

impl MultiDatabaseMetrics {
    /// Create a new multi-database metrics aggregator
    pub fn new() -> Self {
        Self {
            databases: RwLock::new(HashMap::new()),
        }
    }

    /// Add a database to monitor
    pub async fn add_database(&self, name: String, metrics: DatabaseMetrics) {
        let mut databases = self.databases.write().await;
        databases.insert(name.clone(), metrics);
        debug!("Added database {} to metrics monitoring", name);
    }

    /// Remove a database from monitoring
    pub async fn remove_database(&self, name: &str) {
        let mut databases = self.databases.write().await;
        if databases.remove(name).is_some() {
            debug!("Removed database {} from metrics monitoring", name);
        }
    }

    /// Get metrics for a specific database
    pub async fn get_database_metrics(&self, name: &str) -> Option<MetricsSnapshot> {
        let databases = self.databases.read().await;
        if let Some(metrics) = databases.get(name) {
            Some(metrics.get_current_metrics().await)
        } else {
            None
        }
    }

    /// Get aggregated metrics for all databases
    pub async fn get_aggregated_metrics(&self) -> AggregatedMetrics {
        let databases = self.databases.read().await;
        let mut aggregate = AggregatedMetrics::default();
        
        for (name, metrics) in databases.iter() {
            let snapshot = metrics.get_current_metrics().await;
            
            aggregate.total_connections_created += snapshot.connections_created;
            aggregate.total_queries_executed += snapshot.queries_executed;
            aggregate.total_transactions += snapshot.transactions_started;
            aggregate.total_slow_queries += snapshot.slow_queries;
            aggregate.total_deadlocks += snapshot.deadlocks;
            
            aggregate.database_count += 1;
            aggregate.database_snapshots.insert(name.clone(), snapshot);
        }
        
        aggregate.timestamp = chrono::Utc::now();
        aggregate
    }

    /// Take snapshots for all databases
    pub async fn take_all_snapshots(&self) {
        let databases = self.databases.read().await;
        
        for (name, metrics) in databases.iter() {
            metrics.take_snapshot().await;
            debug!("Snapshot taken for database: {}", name);
        }
    }
}

/// Aggregated metrics across multiple databases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub database_count: u32,
    pub total_connections_created: u64,
    pub total_queries_executed: u64,
    pub total_transactions: u64,
    pub total_slow_queries: u64,
    pub total_deadlocks: u64,
    pub database_snapshots: HashMap<String, MetricsSnapshot>,
}

impl Default for AggregatedMetrics {
    fn default() -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            database_count: 0,
            total_connections_created: 0,
            total_queries_executed: 0,
            total_transactions: 0,
            total_slow_queries: 0,
            total_deadlocks: 0,
            database_snapshots: HashMap::new(),
        }
    }
}

/// Metrics collection service
pub struct MetricsCollectionService {
    multi_metrics: Arc<MultiDatabaseMetrics>,
    collection_interval: Duration,
}

impl MetricsCollectionService {
    /// Create a new metrics collection service
    pub fn new(
        multi_metrics: Arc<MultiDatabaseMetrics>,
        collection_interval: Duration,
    ) -> Self {
        Self {
            multi_metrics,
            collection_interval,
        }
    }

    /// Start the metrics collection service
    pub async fn start(&self) -> anyhow::Result<()> {
        let mut interval = tokio::time::interval(self.collection_interval);
        
        info!("Starting database metrics collection service");
        
        loop {
            interval.tick().await;
            
            debug!("Collecting database metrics");
            self.multi_metrics.take_all_snapshots().await;
            
            // Log aggregated metrics periodically
            let aggregated = self.multi_metrics.get_aggregated_metrics().await;
            info!(
                "Database metrics summary - Databases: {}, Total queries: {}, Total connections: {}",
                aggregated.database_count,
                aggregated.total_queries_executed,
                aggregated.total_connections_created
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_database_metrics() {
        let metrics = DatabaseMetrics::new();
        
        // Record some operations
        metrics.record_connection_created();
        metrics.record_query(Duration::from_millis(100), true);
        metrics.record_transaction_started();
        metrics.record_transaction_committed();
        
        let snapshot = metrics.get_current_metrics().await;
        assert_eq!(snapshot.connections_created, 1);
        assert_eq!(snapshot.queries_executed, 1);
        assert_eq!(snapshot.transactions_started, 1);
        assert_eq!(snapshot.transactions_committed, 1);
    }

    #[tokio::test]
    async fn test_derived_metrics() {
        let metrics = DatabaseMetrics::new();
        
        // Record successful operations
        metrics.record_query(Duration::from_millis(200), true);
        metrics.record_query(Duration::from_millis(300), true);
        
        let derived = metrics.get_derived_metrics().await;
        assert_eq!(derived.query_success_rate, 1.0);
        assert_eq!(derived.average_query_duration, Duration::from_millis(250));
    }

    #[tokio::test]
    async fn test_multi_database_metrics() {
        let multi_metrics = MultiDatabaseMetrics::new();
        
        let db1_metrics = DatabaseMetrics::new();
        let db2_metrics = DatabaseMetrics::new();
        
        // Add databases
        multi_metrics.add_database("db1".to_string(), db1_metrics.clone()).await;
        multi_metrics.add_database("db2".to_string(), db2_metrics.clone()).await;
        
        // Record some operations
        db1_metrics.record_query(Duration::from_millis(100), true);
        db2_metrics.record_query(Duration::from_millis(200), true);
        
        let aggregated = multi_metrics.get_aggregated_metrics().await;
        assert_eq!(aggregated.database_count, 2);
        assert_eq!(aggregated.total_queries_executed, 2);
    }

    #[tokio::test]
    async fn test_metrics_history() {
        let metrics = DatabaseMetrics::new();
        
        // Record some operations and take snapshots
        metrics.record_query(Duration::from_millis(100), true);
        metrics.take_snapshot().await;
        
        sleep(Duration::from_millis(10)).await;
        
        metrics.record_query(Duration::from_millis(200), true);
        metrics.take_snapshot().await;
        
        let history = metrics.get_historical_metrics(None).await;
        assert_eq!(history.len(), 2);
        assert!(history[0].timestamp < history[1].timestamp);
    }
}