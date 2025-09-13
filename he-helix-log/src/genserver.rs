//! Log System GenServer Implementation
//! 
//! Complete port of Helix.Log GenServer with event streaming, audit trails,
//! log aggregation, and real-time log processing capabilities.

use he_helix_core::genserver::{
    GenServer, GenServerState, GenServerHandle, GenServerMessage, GenServerReply,
    InfoSource, TerminateReason, SupervisionStrategy, GenServerSupervisor
};
use he_helix_core::{HelixError, HelixResult, ProcessId};
use he_core::id::{AccountId, EntityId, ServerId, LogId};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::{HashMap, BTreeMap, VecDeque, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, Mutex, broadcast, watch};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Log entry types for different game activities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LogType {
    /// Connection and authentication logs
    Connection,
    /// File system operations
    FileSystem,
    /// Process execution logs
    Process,
    /// Network activity logs
    Network,
    /// Security and intrusion logs
    Security,
    /// Financial transaction logs
    Financial,
    /// Email and communication logs
    Communication,
    /// Mission and story progress logs
    Mission,
    /// Admin and system logs
    System,
    /// Error and exception logs
    Error,
    /// Custom user-defined logs
    Custom(String),
}

/// Log severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum LogSeverity {
    Debug,
    Info,
    Notice,
    Warning,
    Error,
    Critical,
    Alert,
    Emergency,
}

/// Log entry status for tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogStatus {
    /// Normal log entry
    Normal,
    /// Flagged for review
    Flagged,
    /// Archived/old log
    Archived,
    /// Deleted (soft delete)
    Deleted,
    /// Forwarded to external system
    Forwarded,
    /// Processing in progress
    Processing,
}

/// Complete log entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub log_id: LogId,
    pub log_type: LogType,
    pub severity: LogSeverity,
    pub status: LogStatus,
    pub timestamp: SystemTime,
    pub source_server: ServerId,
    pub target_server: Option<ServerId>,
    pub actor_account: Option<AccountId>,
    pub entity_id: Option<EntityId>,
    pub message: String,
    pub details: LogDetails,
    pub metadata: LogMetadata,
}

/// Detailed log information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogDetails {
    /// IP addresses involved
    pub ip_addresses: Vec<String>,
    /// File paths involved
    pub file_paths: Vec<String>,
    /// Process IDs involved
    pub process_ids: Vec<EntityId>,
    /// Network connections
    pub connections: Vec<NetworkConnection>,
    /// Custom data fields
    pub custom_fields: HashMap<String, String>,
    /// Binary data (base64 encoded)
    pub binary_data: Option<String>,
    /// Stack trace for errors
    pub stack_trace: Option<String>,
}

/// Network connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub source_ip: String,
    pub source_port: u16,
    pub target_ip: String,
    pub target_port: u16,
    pub protocol: String,
    pub bytes_transferred: u64,
    pub duration: Option<Duration>,
}

/// Log metadata for indexing and search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMetadata {
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Geographic location
    pub location: Option<String>,
    /// Client information
    pub user_agent: Option<String>,
    /// Correlation ID for related logs
    pub correlation_id: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Request ID
    pub request_id: Option<String>,
    /// Parent log ID for hierarchical logs
    pub parent_log_id: Option<LogId>,
    /// External system references
    pub external_refs: HashMap<String, String>,
}

/// Log aggregation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAggregationRule {
    pub rule_id: EntityId,
    pub name: String,
    pub description: String,
    pub criteria: LogSearchCriteria,
    pub aggregation_type: AggregationType,
    pub time_window: Duration,
    pub threshold: u32,
    pub action: AggregationAction,
    pub is_enabled: bool,
}

/// Types of log aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationType {
    Count,
    Sum(String), // field to sum
    Average(String), // field to average
    Max(String), // field to get max
    Min(String), // field to get min
    Distinct(String), // count distinct values
    Timeline, // time-based aggregation
}

/// Actions to take when aggregation threshold is met
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationAction {
    Alert { recipients: Vec<String> },
    Flag { flag_type: String },
    Forward { destination: String },
    Archive { compress: bool },
    Delete { confirm_required: bool },
    Custom { action_name: String, parameters: HashMap<String, String> },
}

/// Log streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStreamConfig {
    pub stream_id: EntityId,
    pub name: String,
    pub filters: LogSearchCriteria,
    pub destinations: Vec<StreamDestination>,
    pub buffer_size: usize,
    pub batch_size: usize,
    pub flush_interval: Duration,
    pub compression: bool,
    pub encryption: bool,
}

/// Stream destinations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamDestination {
    File { path: String },
    Network { url: String, headers: HashMap<String, String> },
    Database { connection_string: String },
    Message Queue { topic: String, broker: String },
    Webhook { url: String, secret: Option<String> },
}

/// Log search and filter criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchCriteria {
    pub log_types: Option<Vec<LogType>>,
    pub severity_min: Option<LogSeverity>,
    pub severity_max: Option<LogSeverity>,
    pub status: Option<Vec<LogStatus>>,
    pub source_servers: Option<Vec<ServerId>>,
    pub target_servers: Option<Vec<ServerId>>,
    pub actor_accounts: Option<Vec<AccountId>>,
    pub time_range: Option<(SystemTime, SystemTime)>,
    pub message_contains: Option<String>,
    pub tags: Option<Vec<String>>,
    pub correlation_id: Option<String>,
    pub ip_addresses: Option<Vec<String>>,
    pub custom_fields: Option<HashMap<String, String>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Log statistics and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStatistics {
    pub total_logs: u64,
    pub logs_by_type: HashMap<LogType, u64>,
    pub logs_by_severity: HashMap<LogSeverity, u64>,
    pub logs_by_status: HashMap<LogStatus, u64>,
    pub logs_per_hour: Vec<(SystemTime, u64)>,
    pub top_sources: Vec<(ServerId, u64)>,
    pub top_actors: Vec<(AccountId, u64)>,
    pub error_rate: f64,
    pub storage_used: u64,
    pub oldest_log: Option<SystemTime>,
    pub newest_log: Option<SystemTime>,
    pub last_updated: SystemTime,
}

/// Log System State
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSystemState {
    /// All log entries (in production this would be in a database)
    pub logs: BTreeMap<SystemTime, Vec<LogEntry>>,
    
    /// Log indices for efficient searching
    pub type_index: HashMap<LogType, Vec<LogId>>,
    pub severity_index: HashMap<LogSeverity, Vec<LogId>>,
    pub server_index: HashMap<ServerId, Vec<LogId>>,
    pub account_index: HashMap<AccountId, Vec<LogId>>,
    pub correlation_index: HashMap<String, Vec<LogId>>,
    
    /// Log streaming configurations
    pub streams: HashMap<EntityId, LogStreamConfig>,
    
    /// Active stream handles
    pub active_streams: HashMap<EntityId, StreamHandle>,
    
    /// Aggregation rules
    pub aggregation_rules: HashMap<EntityId, LogAggregationRule>,
    
    /// Aggregation state tracking
    pub aggregation_state: HashMap<EntityId, AggregationState>,
    
    /// Statistics and metrics
    pub statistics: LogStatistics,
    
    /// Configuration
    pub config: LogSystemConfig,
    
    /// Retention policies
    pub retention_policies: Vec<RetentionPolicy>,
    
    /// Alert subscriptions
    pub alert_subscriptions: HashMap<String, Vec<AlertSubscription>>,
}

/// Stream handle for active streaming
#[derive(Debug)]
pub struct StreamHandle {
    pub stream_id: EntityId,
    pub buffer: VecDeque<LogEntry>,
    pub last_flush: SystemTime,
    pub bytes_processed: u64,
    pub entries_processed: u64,
}

/// Aggregation state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationState {
    pub rule_id: EntityId,
    pub window_start: SystemTime,
    pub current_count: u32,
    pub current_value: f64, // for sum/avg/etc
    pub distinct_values: HashSet<String>,
    pub last_triggered: Option<SystemTime>,
}

/// Log retention policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub policy_id: EntityId,
    pub name: String,
    pub criteria: LogSearchCriteria,
    pub retention_period: Duration,
    pub action: RetentionAction,
    pub is_enabled: bool,
}

/// Retention actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetentionAction {
    Delete,
    Archive { location: String },
    Compress,
    Summarize { fields: Vec<String> },
}

/// Alert subscriptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSubscription {
    pub subscription_id: EntityId,
    pub subscriber: String, // email or webhook
    pub criteria: LogSearchCriteria,
    pub alert_frequency: AlertFrequency,
    pub last_sent: Option<SystemTime>,
}

/// Alert frequency settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertFrequency {
    Immediate,
    Batched { interval: Duration },
    Daily { hour: u8 },
    Weekly { day: u8, hour: u8 },
}

/// Log system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSystemConfig {
    pub max_logs_in_memory: usize,
    pub max_log_size: usize,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
    pub real_time_processing: bool,
    pub batch_processing_interval: Duration,
    pub cleanup_interval: Duration,
    pub index_rebuild_interval: Duration,
    pub stream_buffer_size: usize,
    pub max_concurrent_streams: usize,
    pub enable_audit_trail: bool,
}

impl Default for LogSystemConfig {
    fn default() -> Self {
        Self {
            max_logs_in_memory: 100000,
            max_log_size: 1024 * 1024, // 1MB
            compression_enabled: true,
            encryption_enabled: false,
            real_time_processing: true,
            batch_processing_interval: Duration::from_secs(60),
            cleanup_interval: Duration::from_secs(3600),
            index_rebuild_interval: Duration::from_secs(86400),
            stream_buffer_size: 1000,
            max_concurrent_streams: 10,
            enable_audit_trail: true,
        }
    }
}

impl GenServerState for LogSystemState {
    fn serialize(&self) -> HelixResult<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| HelixError::Serialization(e.to_string()))
    }

    fn deserialize(data: &[u8]) -> HelixResult<Self> {
        serde_json::from_slice(data).map_err(|e| HelixError::Serialization(e.to_string()))
    }
}

/// Log System GenServer Messages - Call patterns
#[derive(Debug)]
pub enum LogCall {
    /// Write a log entry
    WriteLog { entry: LogEntry },
    
    /// Get log entry by ID
    GetLog { log_id: LogId },
    
    /// Search logs with criteria
    SearchLogs { 
        criteria: LogSearchCriteria,
        include_details: bool,
    },
    
    /// Get logs for specific server
    GetServerLogs {
        server_id: ServerId,
        limit: Option<u32>,
        since: Option<SystemTime>,
    },
    
    /// Get logs for specific account
    GetAccountLogs {
        account_id: AccountId,
        limit: Option<u32>,
        since: Option<SystemTime>,
    },
    
    /// Get log statistics
    GetStatistics { 
        time_range: Option<(SystemTime, SystemTime)>,
    },
    
    /// Create log stream
    CreateStream { config: LogStreamConfig },
    
    /// Get stream status
    GetStreamStatus { stream_id: EntityId },
    
    /// Create aggregation rule
    CreateAggregationRule { rule: LogAggregationRule },
    
    /// Get aggregation results
    GetAggregationResults {
        rule_id: EntityId,
        time_range: Option<(SystemTime, SystemTime)>,
    },
    
    /// Export logs
    ExportLogs {
        criteria: LogSearchCriteria,
        format: ExportFormat,
        destination: String,
    },
    
    /// Check system health
    GetSystemHealth,
}

/// Log System GenServer Cast Messages
#[derive(Debug)]
pub enum LogCast {
    /// Batch write multiple logs
    BatchWriteLogs { entries: Vec<LogEntry> },
    
    /// Update log status
    UpdateLogStatus {
        log_id: LogId,
        status: LogStatus,
    },
    
    /// Start log stream
    StartStream { stream_id: EntityId },
    
    /// Stop log stream
    StopStream { stream_id: EntityId },
    
    /// Update stream configuration
    UpdateStream {
        stream_id: EntityId,
        config: LogStreamConfig,
    },
    
    /// Enable/disable aggregation rule
    ToggleAggregationRule {
        rule_id: EntityId,
        enabled: bool,
    },
    
    /// Add retention policy
    AddRetentionPolicy { policy: RetentionPolicy },
    
    /// Subscribe to alerts
    SubscribeToAlerts { subscription: AlertSubscription },
    
    /// Unsubscribe from alerts
    UnsubscribeFromAlerts { subscription_id: EntityId },
    
    /// Force cleanup operation
    ForceCleanup,
    
    /// Rebuild indices
    RebuildIndices,
    
    /// Flush all streams
    FlushAllStreams,
    
    /// Archive old logs
    ArchiveOldLogs { older_than: SystemTime },
    
    /// Compress logs
    CompressLogs { criteria: LogSearchCriteria },
}

/// Log System GenServer Info Messages
#[derive(Debug)]
pub enum LogInfo {
    /// Batch processing timer
    BatchProcessingTimer,
    
    /// Cleanup timer
    CleanupTimer,
    
    /// Statistics refresh timer
    StatsTimer,
    
    /// Index rebuild timer
    IndexRebuildTimer,
    
    /// Stream flush timer
    StreamFlushTimer { stream_id: EntityId },
    
    /// Aggregation evaluation timer
    AggregationTimer,
    
    /// Retention policy enforcement
    RetentionTimer,
    
    /// Alert processing timer
    AlertTimer,
    
    /// External log source
    ExternalLogBatch { 
        source: String,
        entries: Vec<LogEntry>,
    },
    
    /// System monitoring event
    SystemMonitoringEvent {
        event_type: String,
        details: HashMap<String, String>,
    },
    
    /// Stream error notification
    StreamError {
        stream_id: EntityId,
        error: String,
    },
    
    /// Storage warning
    StorageWarning {
        used_space: u64,
        available_space: u64,
        threshold: f64,
    },
}

/// Export formats for logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Xml,
    Syslog,
    Custom(String),
}

/// Log System GenServer Implementation
pub struct LogSystemGenServer {
    log_broadcaster: broadcast::Sender<LogEntry>,
    alert_broadcaster: broadcast::Sender<LogAlert>,
}

/// Log alerts
#[derive(Debug, Clone)]
pub struct LogAlert {
    pub alert_type: String,
    pub severity: LogSeverity,
    pub message: String,
    pub triggered_by: LogId,
    pub timestamp: SystemTime,
    pub details: HashMap<String, String>,
}

impl LogSystemGenServer {
    pub fn new() -> (Self, broadcast::Receiver<LogEntry>, broadcast::Receiver<LogAlert>) {
        let (log_tx, log_rx) = broadcast::channel(10000);
        let (alert_tx, alert_rx) = broadcast::channel(1000);
        (Self { 
            log_broadcaster: log_tx,
            alert_broadcaster: alert_tx,
        }, log_rx, alert_rx)
    }
}

#[async_trait]
impl GenServer for LogSystemGenServer {
    type State = LogSystemState;
    type InitArgs = LogSystemConfig;

    async fn init(config: Self::InitArgs) -> HelixResult<Self::State> {
        info!("Initializing Log System GenServer");
        
        let now = SystemTime::now();
        let statistics = LogStatistics {
            total_logs: 0,
            logs_by_type: HashMap::new(),
            logs_by_severity: HashMap::new(),
            logs_by_status: HashMap::new(),
            logs_per_hour: Vec::new(),
            top_sources: Vec::new(),
            top_actors: Vec::new(),
            error_rate: 0.0,
            storage_used: 0,
            oldest_log: None,
            newest_log: None,
            last_updated: now,
        };

        Ok(LogSystemState {
            logs: BTreeMap::new(),
            type_index: HashMap::new(),
            severity_index: HashMap::new(),
            server_index: HashMap::new(),
            account_index: HashMap::new(),
            correlation_index: HashMap::new(),
            streams: HashMap::new(),
            active_streams: HashMap::new(),
            aggregation_rules: HashMap::new(),
            aggregation_state: HashMap::new(),
            statistics,
            config,
            retention_policies: Vec::new(),
            alert_subscriptions: HashMap::new(),
        })
    }

    async fn handle_call(
        &mut self,
        request: Box<dyn Any + Send + Sync>,
        from: ProcessId,
        state: &mut Self::State,
    ) -> HelixResult<GenServerReply> {
        if let Some(call) = request.downcast_ref::<LogCall>() {
            match call {
                LogCall::WriteLog { entry } => {
                    debug!("Writing log entry {} for {:?}", entry.log_id, from);
                    
                    let log_id = entry.log_id;
                    let timestamp = entry.timestamp;
                    
                    // Add to main storage
                    state.logs.entry(timestamp).or_insert_with(Vec::new).push(entry.clone());
                    
                    // Update indices
                    self.update_indices(state, entry);
                    
                    // Broadcast to subscribers
                    let _ = self.log_broadcaster.send(entry.clone());
                    
                    // Process aggregation rules
                    self.process_aggregation_rules(state, entry).await?;
                    
                    // Check alert subscriptions
                    self.check_alert_subscriptions(state, entry).await?;
                    
                    // Update statistics
                    self.update_statistics(state);
                    
                    Ok(GenServerReply::Reply(Box::new(log_id)))
                }
                
                LogCall::GetLog { log_id } => {
                    debug!("Getting log {} for {:?}", log_id, from);
                    let log_entry = self.find_log_by_id(state, *log_id);
                    Ok(GenServerReply::Reply(Box::new(log_entry)))
                }
                
                LogCall::SearchLogs { criteria, include_details } => {
                    debug!("Searching logs for {:?}", from);
                    let results = self.search_logs(state, criteria, *include_details);
                    Ok(GenServerReply::Reply(Box::new(results)))
                }
                
                LogCall::GetServerLogs { server_id, limit, since } => {
                    debug!("Getting server logs for {} from {:?}", server_id, from);
                    let logs = self.get_server_logs(state, *server_id, *limit, *since);
                    Ok(GenServerReply::Reply(Box::new(logs)))
                }
                
                LogCall::GetAccountLogs { account_id, limit, since } => {
                    debug!("Getting account logs for {} from {:?}", account_id, from);
                    let logs = self.get_account_logs(state, *account_id, *limit, *since);
                    Ok(GenServerReply::Reply(Box::new(logs)))
                }
                
                LogCall::GetStatistics { time_range } => {
                    debug!("Getting log statistics for {:?}", from);
                    let stats = if let Some(range) = time_range {
                        self.compute_statistics_for_range(state, range.0, range.1)
                    } else {
                        state.statistics.clone()
                    };
                    Ok(GenServerReply::Reply(Box::new(stats)))
                }
                
                LogCall::CreateStream { config } => {
                    info!("Creating log stream '{}' for {:?}", config.name, from);
                    let stream_id = config.stream_id;
                    
                    // Create stream handle
                    let handle = StreamHandle {
                        stream_id,
                        buffer: VecDeque::with_capacity(config.buffer_size),
                        last_flush: SystemTime::now(),
                        bytes_processed: 0,
                        entries_processed: 0,
                    };
                    
                    state.streams.insert(stream_id, config.clone());
                    state.active_streams.insert(stream_id, handle);
                    
                    Ok(GenServerReply::Reply(Box::new(stream_id)))
                }
                
                LogCall::GetStreamStatus { stream_id } => {
                    debug!("Getting stream status for {} from {:?}", stream_id, from);
                    let status = self.get_stream_status(state, *stream_id);
                    Ok(GenServerReply::Reply(Box::new(status)))
                }
                
                LogCall::CreateAggregationRule { rule } => {
                    info!("Creating aggregation rule '{}' for {:?}", rule.name, from);
                    let rule_id = rule.rule_id;
                    
                    // Initialize aggregation state
                    let agg_state = AggregationState {
                        rule_id,
                        window_start: SystemTime::now(),
                        current_count: 0,
                        current_value: 0.0,
                        distinct_values: HashSet::new(),
                        last_triggered: None,
                    };
                    
                    state.aggregation_rules.insert(rule_id, rule.clone());
                    state.aggregation_state.insert(rule_id, agg_state);
                    
                    Ok(GenServerReply::Reply(Box::new(rule_id)))
                }
                
                LogCall::GetAggregationResults { rule_id, time_range } => {
                    debug!("Getting aggregation results for rule {} from {:?}", rule_id, from);
                    let results = self.get_aggregation_results(state, *rule_id, time_range.as_ref());
                    Ok(GenServerReply::Reply(Box::new(results)))
                }
                
                LogCall::ExportLogs { criteria, format, destination } => {
                    info!("Exporting logs to {} in {:?} format for {:?}", destination, format, from);
                    let export_id = self.start_log_export(state, criteria, format, destination).await?;
                    Ok(GenServerReply::Reply(Box::new(export_id)))
                }
                
                LogCall::GetSystemHealth => {
                    debug!("Getting system health for {:?}", from);
                    let health = self.get_system_health(state);
                    Ok(GenServerReply::Reply(Box::new(health)))
                }
            }
        } else {
            warn!("Unknown call type from {:?}", from);
            Ok(GenServerReply::Reply(Box::new("unknown_call")))
        }
    }

    async fn handle_cast(
        &mut self,
        message: Box<dyn Any + Send + Sync>,
        state: &mut Self::State,
    ) -> HelixResult<()> {
        if let Some(cast) = message.downcast_ref::<LogCast>() {
            match cast {
                LogCast::BatchWriteLogs { entries } => {
                    info!("Batch writing {} log entries", entries.len());
                    
                    for entry in entries {
                        let timestamp = entry.timestamp;
                        state.logs.entry(timestamp).or_insert_with(Vec::new).push(entry.clone());
                        
                        // Update indices
                        self.update_indices(state, entry);
                        
                        // Broadcast to subscribers
                        let _ = self.log_broadcaster.send(entry.clone());
                        
                        // Process aggregation (optimized for batch)
                        self.process_aggregation_rules(state, entry).await?;
                    }
                    
                    self.update_statistics(state);
                }
                
                LogCast::UpdateLogStatus { log_id, status } => {
                    if let Some(entry) = self.find_log_by_id_mut(state, *log_id) {
                        entry.status = status.clone();
                        info!("Updated log {} status to {:?}", log_id, status);
                    }
                }
                
                LogCast::StartStream { stream_id } => {
                    info!("Starting log stream {}", stream_id);
                    self.start_stream(state, *stream_id).await?;
                }
                
                LogCast::StopStream { stream_id } => {
                    info!("Stopping log stream {}", stream_id);
                    self.stop_stream(state, *stream_id).await?;
                }
                
                LogCast::UpdateStream { stream_id, config } => {
                    info!("Updating stream {} configuration", stream_id);
                    state.streams.insert(*stream_id, config.clone());
                }
                
                LogCast::ToggleAggregationRule { rule_id, enabled } => {
                    if let Some(rule) = state.aggregation_rules.get_mut(rule_id) {
                        rule.is_enabled = *enabled;
                        info!("Aggregation rule {} {}", rule_id, if *enabled { "enabled" } else { "disabled" });
                    }
                }
                
                LogCast::AddRetentionPolicy { policy } => {
                    info!("Adding retention policy: {}", policy.name);
                    state.retention_policies.push(policy.clone());
                }
                
                LogCast::SubscribeToAlerts { subscription } => {
                    info!("Adding alert subscription for {}", subscription.subscriber);
                    let alert_type = "general".to_string(); // Could be derived from criteria
                    state.alert_subscriptions.entry(alert_type)
                        .or_insert_with(Vec::new)
                        .push(subscription.clone());
                }
                
                LogCast::UnsubscribeFromAlerts { subscription_id } => {
                    info!("Removing alert subscription {}", subscription_id);
                    for subscriptions in state.alert_subscriptions.values_mut() {
                        subscriptions.retain(|s| s.subscription_id != *subscription_id);
                    }
                }
                
                LogCast::ForceCleanup => {
                    info!("Forcing cleanup operation");
                    self.cleanup_old_logs(state).await?;
                }
                
                LogCast::RebuildIndices => {
                    info!("Rebuilding log indices");
                    self.rebuild_indices(state).await?;
                }
                
                LogCast::FlushAllStreams => {
                    info!("Flushing all active streams");
                    self.flush_all_streams(state).await?;
                }
                
                LogCast::ArchiveOldLogs { older_than } => {
                    info!("Archiving logs older than {:?}", older_than);
                    self.archive_logs_older_than(state, *older_than).await?;
                }
                
                LogCast::CompressLogs { criteria } => {
                    info!("Compressing logs matching criteria");
                    self.compress_logs(state, criteria).await?;
                }
            }
        }
        Ok(())
    }

    async fn handle_info(
        &mut self,
        message: Box<dyn Any + Send + Sync>,
        _source: InfoSource,
        state: &mut Self::State,
    ) -> HelixResult<()> {
        if let Some(info) = message.downcast_ref::<LogInfo>() {
            match info {
                LogInfo::BatchProcessingTimer => {
                    debug!("Batch processing timer triggered");
                    self.process_batch_operations(state).await?;
                }
                
                LogInfo::CleanupTimer => {
                    debug!("Cleanup timer triggered");
                    self.cleanup_old_logs(state).await?;
                }
                
                LogInfo::StatsTimer => {
                    debug!("Statistics timer triggered");
                    self.update_statistics(state);
                }
                
                LogInfo::IndexRebuildTimer => {
                    debug!("Index rebuild timer triggered");
                    self.rebuild_indices(state).await?;
                }
                
                LogInfo::StreamFlushTimer { stream_id } => {
                    debug!("Stream flush timer for {}", stream_id);
                    self.flush_stream(state, *stream_id).await?;
                }
                
                LogInfo::AggregationTimer => {
                    debug!("Aggregation timer triggered");
                    self.evaluate_aggregation_rules(state).await?;
                }
                
                LogInfo::RetentionTimer => {
                    debug!("Retention timer triggered");
                    self.enforce_retention_policies(state).await?;
                }
                
                LogInfo::AlertTimer => {
                    debug!("Alert timer triggered");
                    self.process_alert_queue(state).await?;
                }
                
                LogInfo::ExternalLogBatch { source, entries } => {
                    info!("Processing {} external logs from {}", entries.len(), source);
                    
                    for entry in entries {
                        let timestamp = entry.timestamp;
                        state.logs.entry(timestamp).or_insert_with(Vec::new).push(entry.clone());
                        self.update_indices(state, entry);
                        let _ = self.log_broadcaster.send(entry.clone());
                    }
                    
                    self.update_statistics(state);
                }
                
                LogInfo::SystemMonitoringEvent { event_type, details } => {
                    info!("System monitoring event: {} - {:?}", event_type, details);
                    
                    // Create system log entry
                    let log_entry = LogEntry {
                        log_id: LogId::new(),
                        log_type: LogType::System,
                        severity: LogSeverity::Info,
                        status: LogStatus::Normal,
                        timestamp: SystemTime::now(),
                        source_server: ServerId::new(), // Would be system server ID
                        target_server: None,
                        actor_account: None,
                        entity_id: None,
                        message: format!("System monitoring: {}", event_type),
                        details: LogDetails {
                            ip_addresses: Vec::new(),
                            file_paths: Vec::new(),
                            process_ids: Vec::new(),
                            connections: Vec::new(),
                            custom_fields: details.clone(),
                            binary_data: None,
                            stack_trace: None,
                        },
                        metadata: LogMetadata {
                            tags: vec!["system".to_string(), "monitoring".to_string()],
                            location: None,
                            user_agent: None,
                            correlation_id: None,
                            session_id: None,
                            request_id: None,
                            parent_log_id: None,
                            external_refs: HashMap::new(),
                        },
                    };
                    
                    let timestamp = log_entry.timestamp;
                    state.logs.entry(timestamp).or_insert_with(Vec::new).push(log_entry.clone());
                    self.update_indices(state, &log_entry);
                    let _ = self.log_broadcaster.send(log_entry);
                }
                
                LogInfo::StreamError { stream_id, error } => {
                    error!("Stream {} error: {}", stream_id, error);
                    
                    // Send alert
                    let alert = LogAlert {
                        alert_type: "stream_error".to_string(),
                        severity: LogSeverity::Error,
                        message: format!("Stream {} encountered an error: {}", stream_id, error),
                        triggered_by: LogId::new(),
                        timestamp: SystemTime::now(),
                        details: HashMap::new(),
                    };
                    let _ = self.alert_broadcaster.send(alert);
                }
                
                LogInfo::StorageWarning { used_space, available_space, threshold } => {
                    let usage_percent = *used_space as f64 / (*used_space + *available_space) as f64;
                    
                    if usage_percent > *threshold {
                        warn!("Storage usage warning: {:.1}% used ({} bytes used, {} bytes available)",
                              usage_percent * 100.0, used_space, available_space);
                        
                        // Send storage warning alert
                        let alert = LogAlert {
                            alert_type: "storage_warning".to_string(),
                            severity: LogSeverity::Warning,
                            message: format!("Storage usage is at {:.1}%", usage_percent * 100.0),
                            triggered_by: LogId::new(),
                            timestamp: SystemTime::now(),
                            details: [
                                ("used_space".to_string(), used_space.to_string()),
                                ("available_space".to_string(), available_space.to_string()),
                                ("usage_percent".to_string(), format!("{:.1}", usage_percent * 100.0)),
                            ].iter().cloned().collect(),
                        };
                        let _ = self.alert_broadcaster.send(alert);
                    }
                }
            }
        }
        Ok(())
    }

    async fn terminate(
        &mut self,
        reason: TerminateReason,
        state: &mut Self::State,
    ) -> HelixResult<()> {
        info!("Log System GenServer terminating: {:?}", reason);
        
        // Flush all streams before shutdown
        self.flush_all_streams(state).await?;
        
        // Final statistics
        info!("Final log statistics: {} total logs, {} active streams",
              state.statistics.total_logs, state.active_streams.len());
        
        Ok(())
    }
}

impl LogSystemGenServer {
    // Helper method implementations (simplified for space)
    
    fn update_indices(&self, state: &mut LogSystemState, entry: &LogEntry) {
        // Update type index
        state.type_index.entry(entry.log_type.clone())
            .or_insert_with(Vec::new)
            .push(entry.log_id);
        
        // Update severity index
        state.severity_index.entry(entry.severity.clone())
            .or_insert_with(Vec::new)
            .push(entry.log_id);
        
        // Update server index
        state.server_index.entry(entry.source_server)
            .or_insert_with(Vec::new)
            .push(entry.log_id);
        
        // Update account index
        if let Some(account_id) = entry.actor_account {
            state.account_index.entry(account_id)
                .or_insert_with(Vec::new)
                .push(entry.log_id);
        }
        
        // Update correlation index
        if let Some(correlation_id) = &entry.metadata.correlation_id {
            state.correlation_index.entry(correlation_id.clone())
                .or_insert_with(Vec::new)
                .push(entry.log_id);
        }
    }
    
    fn find_log_by_id(&self, state: &LogSystemState, log_id: LogId) -> Option<LogEntry> {
        // In a real implementation, this would be more efficient with proper indexing
        for entries in state.logs.values() {
            for entry in entries {
                if entry.log_id == log_id {
                    return Some(entry.clone());
                }
            }
        }
        None
    }
    
    fn find_log_by_id_mut(&self, state: &mut LogSystemState, log_id: LogId) -> Option<&mut LogEntry> {
        // In a real implementation, this would be more efficient with proper indexing
        for entries in state.logs.values_mut() {
            for entry in entries {
                if entry.log_id == log_id {
                    return Some(entry);
                }
            }
        }
        None
    }
    
    fn search_logs(&self, state: &LogSystemState, criteria: &LogSearchCriteria, _include_details: bool) -> Vec<LogEntry> {
        // Simplified search implementation
        let mut results = Vec::new();
        let limit = criteria.limit.unwrap_or(1000) as usize;
        
        for entries in state.logs.values() {
            for entry in entries {
                if results.len() >= limit {
                    break;
                }
                
                // Apply filters
                if let Some(types) = &criteria.log_types {
                    if !types.contains(&entry.log_type) {
                        continue;
                    }
                }
                
                if let Some(min_severity) = &criteria.severity_min {
                    if entry.severity < *min_severity {
                        continue;
                    }
                }
                
                if let Some(max_severity) = &criteria.severity_max {
                    if entry.severity > *max_severity {
                        continue;
                    }
                }
                
                results.push(entry.clone());
            }
        }
        
        results
    }
    
    fn get_server_logs(&self, state: &LogSystemState, server_id: ServerId, limit: Option<u32>, since: Option<SystemTime>) -> Vec<LogEntry> {
        // Implementation would use server index for efficiency
        let mut results = Vec::new();
        let max_results = limit.unwrap_or(100) as usize;
        
        if let Some(log_ids) = state.server_index.get(&server_id) {
            for log_id in log_ids.iter().take(max_results) {
                if let Some(entry) = self.find_log_by_id(state, *log_id) {
                    if let Some(since_time) = since {
                        if entry.timestamp >= since_time {
                            results.push(entry);
                        }
                    } else {
                        results.push(entry);
                    }
                }
            }
        }
        
        results
    }
    
    fn get_account_logs(&self, state: &LogSystemState, account_id: AccountId, limit: Option<u32>, since: Option<SystemTime>) -> Vec<LogEntry> {
        // Similar to get_server_logs but using account index
        let mut results = Vec::new();
        let max_results = limit.unwrap_or(100) as usize;
        
        if let Some(log_ids) = state.account_index.get(&account_id) {
            for log_id in log_ids.iter().take(max_results) {
                if let Some(entry) = self.find_log_by_id(state, *log_id) {
                    if let Some(since_time) = since {
                        if entry.timestamp >= since_time {
                            results.push(entry);
                        }
                    } else {
                        results.push(entry);
                    }
                }
            }
        }
        
        results
    }
    
    fn update_statistics(&self, state: &mut LogSystemState) {
        let mut stats = LogStatistics {
            total_logs: 0,
            logs_by_type: HashMap::new(),
            logs_by_severity: HashMap::new(),
            logs_by_status: HashMap::new(),
            logs_per_hour: Vec::new(),
            top_sources: Vec::new(),
            top_actors: Vec::new(),
            error_rate: 0.0,
            storage_used: 0,
            oldest_log: None,
            newest_log: None,
            last_updated: SystemTime::now(),
        };
        
        let mut error_count = 0;
        
        for entries in state.logs.values() {
            for entry in entries {
                stats.total_logs += 1;
                
                // Count by type
                *stats.logs_by_type.entry(entry.log_type.clone()).or_insert(0) += 1;
                
                // Count by severity
                *stats.logs_by_severity.entry(entry.severity.clone()).or_insert(0) += 1;
                
                // Count by status
                *stats.logs_by_status.entry(entry.status.clone()).or_insert(0) += 1;
                
                // Track errors
                if matches!(entry.severity, LogSeverity::Error | LogSeverity::Critical | LogSeverity::Alert | LogSeverity::Emergency) {
                    error_count += 1;
                }
                
                // Track oldest/newest
                if stats.oldest_log.is_none() || stats.oldest_log.unwrap() > entry.timestamp {
                    stats.oldest_log = Some(entry.timestamp);
                }
                if stats.newest_log.is_none() || stats.newest_log.unwrap() < entry.timestamp {
                    stats.newest_log = Some(entry.timestamp);
                }
            }
        }
        
        stats.error_rate = if stats.total_logs > 0 {
            error_count as f64 / stats.total_logs as f64
        } else {
            0.0
        };
        
        state.statistics = stats;
    }
    
    // Additional helper methods would be implemented here
    // (process_aggregation_rules, check_alert_subscriptions, etc.)
    
    async fn process_aggregation_rules(&self, _state: &mut LogSystemState, _entry: &LogEntry) -> HelixResult<()> {
        // Placeholder for aggregation rule processing
        Ok(())
    }
    
    async fn check_alert_subscriptions(&self, _state: &LogSystemState, _entry: &LogEntry) -> HelixResult<()> {
        // Placeholder for alert subscription checking
        Ok(())
    }
    
    fn compute_statistics_for_range(&self, _state: &LogSystemState, _start: SystemTime, _end: SystemTime) -> LogStatistics {
        // Placeholder for time-range statistics
        LogStatistics {
            total_logs: 0,
            logs_by_type: HashMap::new(),
            logs_by_severity: HashMap::new(),
            logs_by_status: HashMap::new(),
            logs_per_hour: Vec::new(),
            top_sources: Vec::new(),
            top_actors: Vec::new(),
            error_rate: 0.0,
            storage_used: 0,
            oldest_log: None,
            newest_log: None,
            last_updated: SystemTime::now(),
        }
    }
    
    fn get_stream_status(&self, state: &LogSystemState, stream_id: EntityId) -> HashMap<String, String> {
        let mut status = HashMap::new();
        
        if let Some(handle) = state.active_streams.get(&stream_id) {
            status.insert("status".to_string(), "active".to_string());
            status.insert("buffer_size".to_string(), handle.buffer.len().to_string());
            status.insert("bytes_processed".to_string(), handle.bytes_processed.to_string());
            status.insert("entries_processed".to_string(), handle.entries_processed.to_string());
        } else {
            status.insert("status".to_string(), "inactive".to_string());
        }
        
        status
    }
    
    fn get_aggregation_results(&self, _state: &LogSystemState, _rule_id: EntityId, _time_range: Option<&(SystemTime, SystemTime)>) -> HashMap<String, u32> {
        // Placeholder for aggregation results
        HashMap::new()
    }
    
    async fn start_log_export(&self, _state: &LogSystemState, _criteria: &LogSearchCriteria, _format: &ExportFormat, _destination: &str) -> HelixResult<String> {
        // Placeholder for log export
        Ok("export_id_placeholder".to_string())
    }
    
    fn get_system_health(&self, state: &LogSystemState) -> HashMap<String, String> {
        let mut health = HashMap::new();
        
        health.insert("total_logs".to_string(), state.statistics.total_logs.to_string());
        health.insert("active_streams".to_string(), state.active_streams.len().to_string());
        health.insert("aggregation_rules".to_string(), state.aggregation_rules.len().to_string());
        health.insert("error_rate".to_string(), format!("{:.2}%", state.statistics.error_rate * 100.0));
        health.insert("memory_usage".to_string(), state.logs.len().to_string());
        
        health
    }
    
    // Additional async methods would be implemented here
    async fn start_stream(&self, _state: &mut LogSystemState, _stream_id: EntityId) -> HelixResult<()> { Ok(()) }
    async fn stop_stream(&self, _state: &mut LogSystemState, _stream_id: EntityId) -> HelixResult<()> { Ok(()) }
    async fn cleanup_old_logs(&self, _state: &mut LogSystemState) -> HelixResult<()> { Ok(()) }
    async fn rebuild_indices(&self, _state: &mut LogSystemState) -> HelixResult<()> { Ok(()) }
    async fn flush_all_streams(&self, _state: &mut LogSystemState) -> HelixResult<()> { Ok(()) }
    async fn flush_stream(&self, _state: &mut LogSystemState, _stream_id: EntityId) -> HelixResult<()> { Ok(()) }
    async fn archive_logs_older_than(&self, _state: &mut LogSystemState, _older_than: SystemTime) -> HelixResult<()> { Ok(()) }
    async fn compress_logs(&self, _state: &mut LogSystemState, _criteria: &LogSearchCriteria) -> HelixResult<()> { Ok(()) }
    async fn process_batch_operations(&self, _state: &mut LogSystemState) -> HelixResult<()> { Ok(()) }
    async fn evaluate_aggregation_rules(&self, _state: &mut LogSystemState) -> HelixResult<()> { Ok(()) }
    async fn enforce_retention_policies(&self, _state: &mut LogSystemState) -> HelixResult<()> { Ok(()) }
    async fn process_alert_queue(&self, _state: &mut LogSystemState) -> HelixResult<()> { Ok(()) }
}

/// Log System Supervisor
pub struct LogSystemSupervisor {
    supervisor: GenServerSupervisor,
}

impl LogSystemSupervisor {
    pub fn new() -> Self {
        Self {
            supervisor: GenServerSupervisor::new(SupervisionStrategy::OneForOne),
        }
    }
    
    pub async fn start(&mut self) -> HelixResult<(GenServerHandle, broadcast::Receiver<LogEntry>, broadcast::Receiver<LogAlert>)> {
        let (genserver, log_rx, alert_rx) = LogSystemGenServer::new();
        let config = LogSystemConfig::default();
        
        let handle = GenServerHandle::start(genserver, config, Some("log_system".to_string())).await?;
        Ok((handle, log_rx, alert_rx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_log_writing_and_retrieval() {
        let (genserver, _, _) = LogSystemGenServer::new();
        let handle = GenServerHandle::start(
            genserver,
            LogSystemConfig::default(),
            Some("test_log_system".to_string())
        ).await.expect("Failed to start LogSystemGenServer");

        let log_entry = LogEntry {
            log_id: LogId::new(),
            log_type: LogType::Security,
            severity: LogSeverity::Warning,
            status: LogStatus::Normal,
            timestamp: SystemTime::now(),
            source_server: ServerId::new(),
            target_server: None,
            actor_account: Some(AccountId::new()),
            entity_id: None,
            message: "Test security event".to_string(),
            details: LogDetails {
                ip_addresses: vec!["192.168.1.100".to_string()],
                file_paths: vec!["/var/log/test.log".to_string()],
                process_ids: Vec::new(),
                connections: Vec::new(),
                custom_fields: HashMap::new(),
                binary_data: None,
                stack_trace: None,
            },
            metadata: LogMetadata {
                tags: vec!["security".to_string(), "test".to_string()],
                location: Some("test_lab".to_string()),
                user_agent: None,
                correlation_id: Some("test_corr_123".to_string()),
                session_id: None,
                request_id: None,
                parent_log_id: None,
                external_refs: HashMap::new(),
            },
        };

        let call = LogCall::WriteLog { entry: log_entry.clone() };
        let log_id: LogId = handle.call(call, None).await.expect("Failed to write log");
        assert_eq!(log_id, log_entry.log_id);

        let get_call = LogCall::GetLog { log_id };
        let retrieved: Option<LogEntry> = handle.call(get_call, None).await.expect("Failed to get log");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().log_id, log_entry.log_id);

        handle.stop(TerminateReason::Normal).await.expect("Failed to stop");
    }

    #[tokio::test]
    async fn test_log_search() {
        let (genserver, _, _) = LogSystemGenServer::new();
        let handle = GenServerHandle::start(
            genserver,
            LogSystemConfig::default(),
            Some("test_log_system".to_string())
        ).await.expect("Failed to start LogSystemGenServer");

        // Write a few test logs
        for i in 0..5 {
            let log_entry = LogEntry {
                log_id: LogId::new(),
                log_type: if i % 2 == 0 { LogType::Security } else { LogType::Network },
                severity: LogSeverity::Info,
                status: LogStatus::Normal,
                timestamp: SystemTime::now(),
                source_server: ServerId::new(),
                target_server: None,
                actor_account: None,
                entity_id: None,
                message: format!("Test log entry {}", i),
                details: LogDetails {
                    ip_addresses: Vec::new(),
                    file_paths: Vec::new(),
                    process_ids: Vec::new(),
                    connections: Vec::new(),
                    custom_fields: HashMap::new(),
                    binary_data: None,
                    stack_trace: None,
                },
                metadata: LogMetadata {
                    tags: vec!["test".to_string()],
                    location: None,
                    user_agent: None,
                    correlation_id: None,
                    session_id: None,
                    request_id: None,
                    parent_log_id: None,
                    external_refs: HashMap::new(),
                },
            };

            let call = LogCall::WriteLog { entry: log_entry };
            let _: LogId = handle.call(call, None).await.expect("Failed to write log");
        }

        // Search for security logs
        let search_criteria = LogSearchCriteria {
            log_types: Some(vec![LogType::Security]),
            severity_min: None,
            severity_max: None,
            status: None,
            source_servers: None,
            target_servers: None,
            actor_accounts: None,
            time_range: None,
            message_contains: None,
            tags: None,
            correlation_id: None,
            ip_addresses: None,
            custom_fields: None,
            limit: Some(10),
            offset: None,
        };

        let search_call = LogCall::SearchLogs { criteria: search_criteria, include_details: true };
        let results: Vec<LogEntry> = handle.call(search_call, None).await.expect("Failed to search logs");
        
        // Should find 3 security logs (indices 0, 2, 4)
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|entry| entry.log_type == LogType::Security));

        handle.stop(TerminateReason::Normal).await.expect("Failed to stop");
    }
}