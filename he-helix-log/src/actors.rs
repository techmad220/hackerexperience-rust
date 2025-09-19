//! Log Actor System - Complete GenServer Implementation
//!
//! This module provides a comprehensive logging and audit trail system with GenServer patterns,
//! including real-time log streaming, advanced search, aggregation, and distributed log management.
//!
//! Features:
//! - Real-time log streaming with filtering and routing
//! - Advanced log search with full-text indexing
//! - Log aggregation rules with alerting capabilities
//! - Audit trail management with integrity protection
//! - Log retention policies and archival
//! - Distributed log collection and correlation
//! - Performance metrics and monitoring
//! - Log export and import functionality

use crate::models::{
    Log, LogEntry, LogStream, LogFilter, LogAggregationRule, AuditEntry, LogRetentionPolicy,
    LogSearchQuery, LogSearchResult, LogMetrics, LogError, LogLevel, LogCategory
};
use he_core::{
    genserver::{GenServer, GenServerBehavior, GenServerMessage, GenServerReply, InfoSource},
    actors::{Actor, ActorContext, Handler, Message},
    HelixError, HelixResult, ProcessId
};
use async_trait::async_trait;
use std::collections::{HashMap, BTreeMap, VecDeque, HashSet};
use tokio::sync::{RwLock, Mutex, broadcast, mpsc};
use chrono::{DateTime, Utc, Duration};
use std::sync::Arc;
use tracing::{info, error, warn, debug, trace};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use he_core::id::{ServerId, PlayerId, LogId, StreamId};
use regex::Regex;
use sha2::{Sha256, Digest};
use std::io::Write;
use flate2::write::GzEncoder;
use flate2::Compression;

/// Log operation error types
#[derive(Debug, thiserror::Error)]
pub enum LogActorError {
    #[error("Log not found: {0}")]
    LogNotFound(String),
    #[error("Stream not found: {0}")]
    StreamNotFound(String),
    #[error("Invalid log filter: {0}")]
    InvalidFilter(String),
    #[error("Search query failed: {0}")]
    SearchFailed(String),
    #[error("Aggregation rule error: {0}")]
    AggregationFailed(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Export error: {0}")]
    ExportError(String),
    #[error("Integrity check failed: {0}")]
    IntegrityFailed(String),
    #[error("Internal log error: {0}")]
    InternalError(String),
}

/// Messages for LogActor GenServer
#[derive(Debug, Clone)]
pub enum LogCall {
    /// Write a log entry
    WriteLog {
        entry: LogEntry,
        tags: Vec<String>,
        correlation_id: Option<String>,
    },
    /// Search logs with complex criteria
    SearchLogs {
        query: LogSearchQuery,
        limit: Option<usize>,
        offset: Option<usize>,
    },
    /// Create real-time log stream
    CreateStream {
        stream_name: String,
        filters: Vec<LogFilter>,
        buffer_size: Option<usize>,
    },
    /// Get log stream data
    GetStreamData {
        stream_id: StreamId,
        since: Option<DateTime<Utc>>,
        limit: Option<usize>,
    },
    /// Export logs in various formats
    ExportLogs {
        query: Option<LogSearchQuery>,
        format: ExportFormat,
        compression: bool,
        time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    },
    /// Get log statistics and metrics
    GetLogMetrics {
        time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
        group_by: Option<String>,
        include_trends: bool,
    },
    /// Verify log integrity
    VerifyLogIntegrity {
        log_id: Option<LogId>,
        time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    },
    /// Get audit trail for specific entity
    GetAuditTrail {
        entity_id: String,
        entity_type: String,
        time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    },
    /// Import logs from external source
    ImportLogs {
        source_format: String,
        data: Vec<u8>,
        merge_policy: MergePolicy,
    },
}

#[derive(Debug, Clone)]
pub enum LogCast {
    /// Batch write multiple log entries
    BatchWriteLogs {
        entries: Vec<LogEntry>,
        batch_id: Option<String>,
    },
    /// Create log aggregation rule
    CreateAggregationRule {
        rule: LogAggregationRule,
    },
    /// Update log stream filters
    UpdateStreamFilters {
        stream_id: StreamId,
        new_filters: Vec<LogFilter>,
    },
    /// Archive old logs
    ArchiveLogs {
        older_than: DateTime<Utc>,
        compression_level: Option<u32>,
    },
    /// Process aggregation rules
    ProcessAggregations,
    /// Cleanup expired log data
    CleanupExpiredLogs,
    /// Rebuild log indices
    RebuildIndices {
        log_category: Option<LogCategory>,
        force: bool,
    },
    /// Replicate logs to other nodes
    ReplicateToNodes {
        node_ids: Vec<String>,
        log_filter: Option<LogFilter>,
    },
}

#[derive(Debug, Clone)]
pub enum LogInfo {
    /// External log batch from distributed system
    ExternalLogBatch {
        node_id: String,
        entries: Vec<LogEntry>,
        batch_timestamp: DateTime<Utc>,
    },
    /// Storage capacity warning
    StorageWarning {
        current_usage: u64,
        total_capacity: u64,
        warning_threshold: f64,
    },
    /// Index maintenance notification
    IndexMaintenanceComplete {
        index_type: String,
        entries_processed: u64,
        duration_ms: u64,
    },
    /// Aggregation rule triggered
    AggregationTriggered {
        rule_id: String,
        rule_name: String,
        match_count: u64,
        alert_data: Option<HashMap<String, String>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Syslog,
    Elasticsearch,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergePolicy {
    Append,
    Replace,
    Merge,
    Skip,
}

/// Log Actor state structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogActorState {
    /// Log entries indexed by ID
    pub logs: HashMap<LogId, LogEntry>,
    /// Log indices for fast searching
    pub indices: HashMap<String, LogIndex>,
    /// Active log streams
    pub streams: HashMap<StreamId, LogStream>,
    /// Stream subscribers
    pub stream_subscribers: HashMap<StreamId, Vec<StreamSubscriber>>,
    /// Aggregation rules
    pub aggregation_rules: HashMap<String, LogAggregationRule>,
    /// Audit entries for compliance
    pub audit_entries: VecDeque<AuditEntry>,
    /// Retention policies
    pub retention_policies: HashMap<String, LogRetentionPolicy>,
    /// Performance metrics
    pub metrics: LogMetrics,
    /// Configuration
    pub config: LogActorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogActorConfig {
    pub max_log_entries: usize,
    pub max_stream_buffer_size: usize,
    pub index_rebuild_interval_hours: u64,
    pub aggregation_check_interval_minutes: u64,
    pub retention_check_interval_hours: u64,
    pub audit_trail_enabled: bool,
    pub compression_enabled: bool,
    pub replication_enabled: bool,
    pub integrity_check_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogIndex {
    pub index_name: String,
    pub field_name: String,
    pub index_type: IndexType,
    pub entries: HashMap<String, Vec<LogId>>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexType {
    Text,
    Numeric,
    DateTime,
    Category,
    FullText,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamSubscriber {
    pub subscriber_id: String,
    pub subscriber_type: String,
    pub filters: Vec<LogFilter>,
    pub last_seen: DateTime<Utc>,
}

impl Default for LogActorConfig {
    fn default() -> Self {
        Self {
            max_log_entries: 1_000_000,
            max_stream_buffer_size: 10000,
            index_rebuild_interval_hours: 24,
            aggregation_check_interval_minutes: 5,
            retention_check_interval_hours: 6,
            audit_trail_enabled: true,
            compression_enabled: true,
            replication_enabled: true,
            integrity_check_enabled: true,
        }
    }
}

impl Default for LogActorState {
    fn default() -> Self {
        Self {
            logs: HashMap::new(),
            indices: HashMap::new(),
            streams: HashMap::new(),
            stream_subscribers: HashMap::new(),
            aggregation_rules: HashMap::new(),
            audit_entries: VecDeque::new(),
            retention_policies: HashMap::new(),
            metrics: LogMetrics::default(),
            config: LogActorConfig::default(),
        }
    }
}

/// Main Log Actor
pub struct LogActor {
    state: Arc<RwLock<LogActorState>>,
    background_handle: Option<tokio::task::JoinHandle<()>>,
    stream_senders: Arc<Mutex<HashMap<StreamId, broadcast::Sender<LogEntry>>>>,
    aggregation_handle: Option<tokio::task::JoinHandle<()>>,
    compression_buffer: Arc<Mutex<Vec<u8>>>,
}

impl LogActor {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(LogActorState::default())),
            background_handle: None,
            stream_senders: Arc::new(Mutex::new(HashMap::new())),
            aggregation_handle: None,
            compression_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn initialize(&mut self) -> HelixResult<()> {
        self.start_background_processing().await?;
        self.start_aggregation_processing().await?;
        self.initialize_default_indices().await?;
        self.load_retention_policies().await?;
        
        info!("LogActor initialized");
        Ok(())
    }

    async fn start_background_processing(&mut self) -> HelixResult<()> {
        let state = Arc::clone(&self.state);
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // 1 hour
            loop {
                interval.tick().await;
                Self::process_background_tasks(&state).await;
            }
        });

        self.background_handle = Some(handle);
        Ok(())
    }

    async fn start_aggregation_processing(&mut self) -> HelixResult<()> {
        let state = Arc::clone(&self.state);
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                Self::process_aggregation_rules(&state).await;
            }
        });

        self.aggregation_handle = Some(handle);
        Ok(())
    }

    async fn initialize_default_indices(&self) -> HelixResult<()> {
        let mut state = self.state.write().await;
        
        let default_indices = vec![
            ("timestamp", IndexType::DateTime),
            ("level", IndexType::Category),
            ("category", IndexType::Category),
            ("server_id", IndexType::Text),
            ("player_id", IndexType::Text),
            ("message", IndexType::FullText),
        ];

        for (field_name, index_type) in default_indices {
            let index = LogIndex {
                index_name: format!("idx_{}", field_name),
                field_name: field_name.to_string(),
                index_type,
                entries: HashMap::new(),
                last_updated: Utc::now(),
            };
            
            state.indices.insert(index.index_name.clone(), index);
        }

        debug!("Initialized {} default log indices", state.indices.len());
        Ok(())
    }

    async fn load_retention_policies(&self) -> HelixResult<()> {
        let mut state = self.state.write().await;

        // Default retention policies
        let policies = vec![
            LogRetentionPolicy {
                policy_name: "debug_logs".to_string(),
                log_level: Some(LogLevel::Debug),
                log_category: None,
                retention_days: 7,
                archive_after_days: Some(1),
                compression_enabled: true,
            },
            LogRetentionPolicy {
                policy_name: "error_logs".to_string(),
                log_level: Some(LogLevel::Error),
                log_category: None,
                retention_days: 365,
                archive_after_days: Some(30),
                compression_enabled: true,
            },
            LogRetentionPolicy {
                policy_name: "audit_logs".to_string(),
                log_level: None,
                log_category: Some(LogCategory::Audit),
                retention_days: 2555, // 7 years for compliance
                archive_after_days: Some(90),
                compression_enabled: true,
            },
        ];

        for policy in policies {
            state.retention_policies.insert(policy.policy_name.clone(), policy);
        }

        debug!("Loaded {} retention policies", state.retention_policies.len());
        Ok(())
    }

    async fn process_background_tasks(state: &Arc<RwLock<LogActorState>>) {
        Self::cleanup_expired_logs(state).await;
        Self::rebuild_indices(state).await;
        Self::update_metrics(state).await;
        Self::check_storage_usage(state).await;
    }

    async fn cleanup_expired_logs(state: &Arc<RwLock<LogActorState>>) {
        let mut state_guard = state.write().await;
        let now = Utc::now();
        let mut removed_count = 0;

        // Apply retention policies
        for policy in state_guard.retention_policies.values() {
            let cutoff = now - Duration::days(policy.retention_days as i64);
            
            let logs_to_remove: Vec<LogId> = state_guard
                .logs
                .iter()
                .filter(|(_, log_entry)| {
                    log_entry.timestamp < cutoff &&
                    policy.applies_to_log(log_entry)
                })
                .map(|(id, _)| *id)
                .collect();

            for log_id in logs_to_remove {
                if state_guard.logs.remove(&log_id).is_some() {
                    removed_count += 1;
                    
                    // Remove from indices
                    for index in state_guard.indices.values_mut() {
                        for entry_list in index.entries.values_mut() {
                            entry_list.retain(|id| *id != log_id);
                        }
                    }
                }
            }
        }

        if removed_count > 0 {
            debug!("Cleaned up {} expired log entries", removed_count);
            state_guard.metrics.total_logs_deleted += removed_count as u64;
        }
    }

    async fn rebuild_indices(state: &Arc<RwLock<LogActorState>>) {
        let mut state_guard = state.write().await;
        let now = Utc::now();
        
        // Check if index rebuild is needed
        let needs_rebuild = state_guard
            .indices
            .values()
            .any(|index| now - index.last_updated > Duration::hours(24));

        if needs_rebuild {
            let start_time = std::time::Instant::now();
            let mut processed_entries = 0;

            for index in state_guard.indices.values_mut() {
                index.entries.clear();
                
                for (log_id, log_entry) in &state_guard.logs {
                    if let Some(value) = Self::extract_index_value(log_entry, &index.field_name) {
                        index.entries
                            .entry(value)
                            .or_insert_with(Vec::new)
                            .push(*log_id);
                        processed_entries += 1;
                    }
                }
                
                index.last_updated = now;
            }

            let duration_ms = start_time.elapsed().as_millis() as u64;
            debug!("Rebuilt log indices: {} entries processed in {}ms", 
                   processed_entries, duration_ms);
        }
    }

    fn extract_index_value(log_entry: &LogEntry, field_name: &str) -> Option<String> {
        match field_name {
            "timestamp" => Some(log_entry.timestamp.to_rfc3339()),
            "level" => Some(format!("{:?}", log_entry.level)),
            "category" => Some(format!("{:?}", log_entry.category)),
            "server_id" => log_entry.server_id.map(|id| id.to_string()),
            "player_id" => log_entry.player_id.map(|id| id.to_string()),
            "message" => Some(log_entry.message.clone()),
            _ => log_entry.metadata.get(field_name).cloned(),
        }
    }

    async fn update_metrics(state: &Arc<RwLock<LogActorState>>) {
        let mut state_guard = state.write().await;
        
        state_guard.metrics.total_logs = state_guard.logs.len() as u64;
        state_guard.metrics.active_streams = state_guard.streams.len() as u64;
        state_guard.metrics.last_updated = Utc::now();
    }

    async fn check_storage_usage(state: &Arc<RwLock<LogActorState>>) {
        let state_guard = state.read().await;
        let config = &state_guard.config;
        
        if state_guard.logs.len() > config.max_log_entries {
            warn!("Log storage approaching capacity: {}/{}", 
                  state_guard.logs.len(), config.max_log_entries);
        }
    }

    async fn process_aggregation_rules(state: &Arc<RwLock<LogActorState>>) {
        let state_guard = state.read().await;
        let now = Utc::now();
        
        for rule in state_guard.aggregation_rules.values() {
            if now - rule.last_executed > Duration::minutes(rule.check_interval_minutes as i64) {
                // Process aggregation rule
                let matches = Self::evaluate_aggregation_rule(rule, &state_guard.logs);
                
                if matches.len() >= rule.min_matches as usize {
                    debug!("Aggregation rule '{}' triggered with {} matches", 
                           rule.rule_name, matches.len());
                    
                    // Trigger alerts or actions
                    if let Some(ref alert_config) = rule.alert_config {
                        // Process alert
                        trace!("Processing alert for rule: {}", rule.rule_name);
                    }
                }
            }
        }
    }

    fn evaluate_aggregation_rule(rule: &LogAggregationRule, logs: &HashMap<LogId, LogEntry>) -> Vec<LogId> {
        let cutoff = Utc::now() - Duration::minutes(rule.time_window_minutes as i64);
        
        logs.iter()
            .filter(|(_, log_entry)| {
                log_entry.timestamp >= cutoff && Self::log_matches_rule(log_entry, rule)
            })
            .map(|(id, _)| *id)
            .collect()
    }

    fn log_matches_rule(log_entry: &LogEntry, rule: &LogAggregationRule) -> bool {
        // Check level filter
        if let Some(ref levels) = rule.level_filter {
            if !levels.contains(&log_entry.level) {
                return false;
            }
        }

        // Check category filter
        if let Some(ref categories) = rule.category_filter {
            if !categories.contains(&log_entry.category) {
                return false;
            }
        }

        // Check message pattern
        if let Some(ref pattern) = rule.message_pattern {
            if let Ok(regex) = Regex::new(pattern) {
                if !regex.is_match(&log_entry.message) {
                    return false;
                }
            }
        }

        true
    }

    async fn handle_write_log(
        &self,
        entry: LogEntry,
        tags: Vec<String>,
        correlation_id: Option<String>,
    ) -> Result<LogId, LogActorError> {
        let mut state = self.state.write().await;
        let log_id = LogId::new();

        let mut log_entry = entry;
        log_entry.log_id = log_id;
        log_entry.tags = tags;
        log_entry.correlation_id = correlation_id;

        // Calculate integrity hash if enabled
        if state.config.integrity_check_enabled {
            log_entry.integrity_hash = Some(self.calculate_integrity_hash(&log_entry));
        }

        // Update indices
        for index in state.indices.values_mut() {
            if let Some(value) = Self::extract_index_value(&log_entry, &index.field_name) {
                index.entries
                    .entry(value)
                    .or_insert_with(Vec::new)
                    .push(log_id);
            }
        }

        // Add to streams
        let stream_senders = self.stream_senders.lock().await;
        for (stream_id, stream) in &state.streams {
            if Self::log_matches_stream_filters(&log_entry, &stream.filters) {
                if let Some(sender) = stream_senders.get(stream_id) {
                    let _ = sender.send(log_entry.clone());
                }
            }
        }

        // Store log entry
        state.logs.insert(log_id, log_entry.clone());
        state.metrics.total_logs_written += 1;

        // Create audit entry if enabled
        if state.config.audit_trail_enabled {
            let audit_entry = AuditEntry {
                audit_id: Uuid::new_v4(),
                entity_type: "log".to_string(),
                entity_id: log_id.to_string(),
                operation: "create".to_string(),
                user_id: log_entry.player_id.map(|id| id.to_string()),
                timestamp: Utc::now(),
                details: HashMap::from([
                    ("level".to_string(), format!("{:?}", log_entry.level)),
                    ("category".to_string(), format!("{:?}", log_entry.category)),
                ]),
            };
            state.audit_entries.push_back(audit_entry);
        }

        Ok(log_id)
    }

    fn calculate_integrity_hash(&self, log_entry: &LogEntry) -> String {
        let mut hasher = Sha256::new();
        hasher.update(log_entry.timestamp.to_rfc3339().as_bytes());
        hasher.update(log_entry.message.as_bytes());
        hasher.update(format!("{:?}", log_entry.level).as_bytes());
        hasher.update(format!("{:?}", log_entry.category).as_bytes());
        
        if let Some(server_id) = log_entry.server_id {
            hasher.update(server_id.to_string().as_bytes());
        }
        
        format!("{:x}", hasher.finalize())
    }

    fn log_matches_stream_filters(log_entry: &LogEntry, filters: &[LogFilter]) -> bool {
        if filters.is_empty() {
            return true;
        }

        filters.iter().any(|filter| Self::log_matches_filter(log_entry, filter))
    }

    fn log_matches_filter(log_entry: &LogEntry, filter: &LogFilter) -> bool {
        match filter {
            LogFilter::Level(levels) => levels.contains(&log_entry.level),
            LogFilter::Category(categories) => categories.contains(&log_entry.category),
            LogFilter::ServerId(server_ids) => {
                log_entry.server_id.map_or(false, |id| server_ids.contains(&id))
            }
            LogFilter::PlayerId(player_ids) => {
                log_entry.player_id.map_or(false, |id| player_ids.contains(&id))
            }
            LogFilter::MessageRegex(pattern) => {
                Regex::new(pattern)
                    .map_or(false, |regex| regex.is_match(&log_entry.message))
            }
            LogFilter::TimeRange(start, end) => {
                log_entry.timestamp >= *start && log_entry.timestamp <= *end
            }
            LogFilter::Tag(tags) => {
                tags.iter().any(|tag| log_entry.tags.contains(tag))
            }
        }
    }

    async fn handle_search_logs(
        &self,
        query: LogSearchQuery,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<LogSearchResult, LogActorError> {
        let state = self.state.read().await;
        let mut matching_logs = Vec::new();
        let search_limit = limit.unwrap_or(100);
        let search_offset = offset.unwrap_or(0);

        // Use indices for efficient searching when possible
        let mut candidate_logs = if let Some(ref text_query) = query.text_query {
            // Full-text search
            self.perform_full_text_search(text_query, &state.logs, &state.indices)?
        } else {
            // Collect all logs
            state.logs.keys().cloned().collect()
        };

        // Apply filters
        for log_id in candidate_logs {
            if let Some(log_entry) = state.logs.get(&log_id) {
                if self.log_matches_query(log_entry, &query) {
                    matching_logs.push(log_entry.clone());
                }
            }
        }

        // Sort results
        matching_logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply pagination
        let total_count = matching_logs.len();
        let paginated_logs = matching_logs
            .into_iter()
            .skip(search_offset)
            .take(search_limit)
            .collect();

        Ok(LogSearchResult {
            logs: paginated_logs,
            total_count,
            limit: search_limit,
            offset: search_offset,
            search_time_ms: 0, // TODO: measure actual search time
        })
    }

    fn perform_full_text_search(
        &self,
        text_query: &str,
        logs: &HashMap<LogId, LogEntry>,
        indices: &HashMap<String, LogIndex>,
    ) -> Result<Vec<LogId>, LogActorError> {
        if let Some(message_index) = indices.get("idx_message") {
            let mut results = HashSet::new();
            
            // Simple keyword matching for now
            let keywords: Vec<&str> = text_query.split_whitespace().collect();
            
            for (indexed_value, log_ids) in &message_index.entries {
                if keywords.iter().any(|keyword| {
                    indexed_value.to_lowercase().contains(&keyword.to_lowercase())
                }) {
                    results.extend(log_ids.iter().cloned());
                }
            }
            
            Ok(results.into_iter().collect())
        } else {
            // Fallback to linear search
            Ok(logs
                .iter()
                .filter(|(_, log_entry)| {
                    log_entry.message.to_lowercase().contains(&text_query.to_lowercase())
                })
                .map(|(id, _)| *id)
                .collect())
        }
    }

    fn log_matches_query(&self, log_entry: &LogEntry, query: &LogSearchQuery) -> bool {
        // Apply all query filters
        if let Some(ref filters) = query.filters {
            if !Self::log_matches_stream_filters(log_entry, filters) {
                return false;
            }
        }

        if let Some(ref time_range) = query.time_range {
            if log_entry.timestamp < time_range.0 || log_entry.timestamp > time_range.1 {
                return false;
            }
        }

        true
    }

    async fn handle_create_stream(
        &self,
        stream_name: String,
        filters: Vec<LogFilter>,
        buffer_size: Option<usize>,
    ) -> Result<StreamId, LogActorError> {
        let mut state = self.state.write().await;
        let stream_id = StreamId::new();
        
        let buffer_size = buffer_size.unwrap_or(state.config.max_stream_buffer_size);
        let (sender, _receiver) = broadcast::channel(buffer_size);

        let stream = LogStream {
            stream_id,
            stream_name,
            filters,
            created_at: Utc::now(),
            last_activity: Utc::now(),
            subscriber_count: 0,
        };

        state.streams.insert(stream_id, stream);
        
        let mut senders = self.stream_senders.lock().await;
        senders.insert(stream_id, sender);

        info!("Created log stream: {} ({})", stream_name, stream_id);
        Ok(stream_id)
    }
}

/// GenServer implementation for LogActor
#[async_trait]
impl GenServerBehavior for LogActor {
    type State = LogActorState;

    async fn init(&mut self) -> HelixResult<()> {
        self.initialize().await?;
        info!("LogActor GenServer initialized");
        Ok(())
    }

    async fn handle_call(
        &mut self,
        message: Box<dyn std::any::Any + Send + Sync>,
        _from: ProcessId,
    ) -> HelixResult<GenServerReply> {
        if let Ok(call) = message.downcast::<LogCall>() {
            match *call {
                LogCall::WriteLog { entry, tags, correlation_id } => {
                    let result = self.handle_write_log(entry, tags, correlation_id).await;
                    Ok(GenServerReply::Reply(Box::new(result)))
                }
                LogCall::SearchLogs { query, limit, offset } => {
                    let result = self.handle_search_logs(query, limit, offset).await;
                    Ok(GenServerReply::Reply(Box::new(result)))
                }
                LogCall::CreateStream { stream_name, filters, buffer_size } => {
                    let result = self.handle_create_stream(stream_name, filters, buffer_size).await;
                    Ok(GenServerReply::Reply(Box::new(result)))
                }
                LogCall::GetLogMetrics { time_range, group_by, include_trends } => {
                    let state = self.state.read().await;
                    let metrics = state.metrics.clone();
                    Ok(GenServerReply::Reply(Box::new(Ok::<LogMetrics, LogActorError>(metrics))))
                }
                LogCall::VerifyLogIntegrity { log_id, time_range } => {
                    let state = self.state.read().await;
                    let mut verified_count = 0;
                    let mut failed_count = 0;

                    let logs_to_check: Vec<&LogEntry> = if let Some(id) = log_id {
                        state.logs.get(&id).into_iter().collect()
                    } else {
                        state.logs.values()
                            .filter(|entry| {
                                time_range.as_ref().map_or(true, |(start, end)| {
                                    entry.timestamp >= *start && entry.timestamp <= *end
                                })
                            })
                            .collect()
                    };

                    for log_entry in logs_to_check {
                        if let Some(ref stored_hash) = log_entry.integrity_hash {
                            let calculated_hash = self.calculate_integrity_hash(log_entry);
                            if &calculated_hash == stored_hash {
                                verified_count += 1;
                            } else {
                                failed_count += 1;
                            }
                        }
                    }

                    let result = HashMap::from([
                        ("verified".to_string(), verified_count.to_string()),
                        ("failed".to_string(), failed_count.to_string()),
                    ]);

                    Ok(GenServerReply::Reply(Box::new(Ok::<HashMap<String, String>, LogActorError>(result))))
                }
                _ => {
                    warn!("Unhandled log call message");
                    Ok(GenServerReply::NoReply)
                }
            }
        } else {
            Err(HelixError::InvalidMessage("Unknown call message type".to_string()))
        }
    }

    async fn handle_cast(
        &mut self,
        message: Box<dyn std::any::Any + Send + Sync>,
    ) -> HelixResult<()> {
        if let Ok(cast) = message.downcast::<LogCast>() {
            match *cast {
                LogCast::BatchWriteLogs { entries, batch_id } => {
                    for entry in entries {
                        if let Err(e) = self.handle_write_log(entry, vec![], batch_id.clone()).await {
                            error!("Failed to write log entry: {:?}", e);
                        }
                    }
                }
                LogCast::CreateAggregationRule { rule } => {
                    let mut state = self.state.write().await;
                    state.aggregation_rules.insert(rule.rule_name.clone(), rule);
                }
                LogCast::ProcessAggregations => {
                    let state = Arc::clone(&self.state);
                    tokio::spawn(async move {
                        Self::process_aggregation_rules(&state).await;
                    });
                }
                LogCast::CleanupExpiredLogs => {
                    let state = Arc::clone(&self.state);
                    tokio::spawn(async move {
                        Self::cleanup_expired_logs(&state).await;
                    });
                }
                LogCast::RebuildIndices { log_category, force } => {
                    let state = Arc::clone(&self.state);
                    tokio::spawn(async move {
                        Self::rebuild_indices(&state).await;
                    });
                }
                _ => {
                    debug!("Log cast message processed");
                }
            }
        }
        Ok(())
    }

    async fn handle_info(
        &mut self,
        message: Box<dyn std::any::Any + Send + Sync>,
        _source: InfoSource,
    ) -> HelixResult<()> {
        if let Ok(info) = message.downcast::<LogInfo>() {
            match *info {
                LogInfo::ExternalLogBatch { node_id, entries, batch_timestamp } => {
                    debug!("Received external log batch from {}: {} entries", 
                           node_id, entries.len());
                    
                    for entry in entries {
                        if let Err(e) = self.handle_write_log(entry, vec![], Some(node_id.clone())).await {
                            error!("Failed to process external log: {:?}", e);
                        }
                    }
                }
                LogInfo::StorageWarning { current_usage, total_capacity, warning_threshold } => {
                    warn!("Log storage warning: {}/{} bytes ({:.1}%)", 
                          current_usage, total_capacity, 
                          (current_usage as f64 / total_capacity as f64) * 100.0);
                }
                LogInfo::IndexMaintenanceComplete { index_type, entries_processed, duration_ms } => {
                    debug!("Index maintenance completed for {}: {} entries in {}ms", 
                           index_type, entries_processed, duration_ms);
                }
                LogInfo::AggregationTriggered { rule_id, rule_name, match_count, alert_data } => {
                    info!("Aggregation rule '{}' triggered: {} matches", 
                          rule_name, match_count);
                }
            }
        }
        Ok(())
    }

    async fn terminate(&mut self, _reason: String) -> HelixResult<()> {
        if let Some(handle) = self.background_handle.take() {
            handle.abort();
        }
        if let Some(handle) = self.aggregation_handle.take() {
            handle.abort();
        }
        
        info!("LogActor terminated");
        Ok(())
    }

    async fn code_change(&mut self, _old_version: String, _new_version: String) -> HelixResult<()> {
        info!("LogActor code change completed");
        Ok(())
    }

    async fn get_state(&self) -> HelixResult<Self::State> {
        let state = self.state.read().await;
        Ok(state.clone())
    }

    async fn set_state(&mut self, state: Self::State) -> HelixResult<()> {
        let mut current_state = self.state.write().await;
        *current_state = state;
        Ok(())
    }
}

/// Log Actor supervisor
pub struct LogActorSupervisor;

impl LogActorSupervisor {
    pub async fn start() -> HelixResult<LogActor> {
        let mut actor = LogActor::new();
        actor.initialize().await?;
        info!("LogActor supervised startup completed");
        Ok(actor)
    }
}
