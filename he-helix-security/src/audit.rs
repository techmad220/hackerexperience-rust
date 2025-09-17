//! Comprehensive audit logging for security events

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::net::IpAddr;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEvent {
    // Authentication events
    LoginAttempt {
        username: String,
        ip: IpAddr,
        success: bool,
        reason: Option<String>,
    },
    LoginSuccess {
        user_id: i64,
        username: String,
        ip: IpAddr,
        session_id: String,
    },
    LoginFailure {
        username: String,
        ip: IpAddr,
        reason: String,
        attempt_count: i32,
    },
    LogoutEvent {
        user_id: i64,
        session_id: String,
        reason: String, // "user_initiated", "timeout", "forced"
    },
    PasswordChange {
        user_id: i64,
        ip: IpAddr,
    },
    PasswordResetRequest {
        email: String,
        ip: IpAddr,
    },

    // Authorization events
    PermissionDenied {
        user_id: i64,
        resource: String,
        action: String,
        ip: IpAddr,
    },
    ElevatedPrivilegeUsed {
        user_id: i64,
        action: String,
        target: String,
    },

    // Game-specific security events
    ProcessManipulation {
        user_id: i64,
        process_id: i64,
        action: String, // "cancel", "pause", "modify"
        suspicious: bool,
    },
    ResourceOverflow {
        user_id: i64,
        resource_type: String,
        attempted_value: i64,
        max_value: i64,
    },
    SuspiciousTransfer {
        from_user_id: i64,
        to_account: String,
        amount: i64,
        flags: Vec<String>, // ["rapid_succession", "unusual_amount", "new_recipient"]
    },
    HackingAttempt {
        attacker_id: i64,
        target_id: i64,
        success: bool,
        detection_level: f32,
    },

    // System security events
    RateLimitExceeded {
        ip: IpAddr,
        endpoint: String,
        requests_count: i32,
        window_seconds: i64,
    },
    DDoSAttackDetected {
        source_ips: Vec<IpAddr>,
        target_endpoint: String,
        requests_per_second: f64,
    },
    IntrusionDetected {
        ip: IpAddr,
        pattern: String,
        threat_level: String,
        details: String,
    },
    SqlInjectionAttempt {
        user_id: Option<i64>,
        ip: IpAddr,
        payload: String,
        endpoint: String,
    },
    XssAttempt {
        user_id: Option<i64>,
        ip: IpAddr,
        payload: String,
        field: String,
    },

    // Data protection events
    DataExport {
        user_id: i64,
        data_type: String,
        record_count: i32,
    },
    DataEncryption {
        record_type: String,
        record_id: i64,
        fields: Vec<String>,
    },
    DataDecryption {
        user_id: i64,
        record_type: String,
        record_id: i64,
        purpose: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub severity: String, // "info", "warning", "critical"
    pub event_data: serde_json::Value,
    pub user_id: Option<i64>,
    pub ip_address: Option<IpAddr>,
    pub session_id: Option<String>,
    pub correlation_id: Option<String>, // For tracking related events
}

pub struct AuditLogger {
    pool: PgPool,
    async_channel: tokio::sync::mpsc::UnboundedSender<AuditLog>,
}

impl AuditLogger {
    pub async fn new(pool: PgPool) -> anyhow::Result<Self> {
        // Create audit log table if it doesn't exist
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS audit_logs (
                id BIGSERIAL PRIMARY KEY,
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                event_type VARCHAR(100) NOT NULL,
                severity VARCHAR(20) NOT NULL,
                event_data JSONB NOT NULL,
                user_id BIGINT,
                ip_address INET,
                session_id VARCHAR(255),
                correlation_id UUID,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

                INDEX idx_timestamp (timestamp),
                INDEX idx_event_type (event_type),
                INDEX idx_severity (severity),
                INDEX idx_user_id (user_id),
                INDEX idx_ip_address (ip_address),
                INDEX idx_correlation_id (correlation_id)
            )
            "#
        )
        .execute(&pool)
        .await?;

        // Create async channel for non-blocking logging
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<AuditLog>();

        let pool_clone = pool.clone();

        // Spawn background task to write logs
        tokio::spawn(async move {
            while let Some(log) = rx.recv().await {
                if let Err(e) = Self::write_log(&pool_clone, &log).await {
                    error!("Failed to write audit log: {}", e);
                }
            }
        });

        Ok(Self {
            pool,
            async_channel: tx,
        })
    }

    pub async fn log_event(&self, event: SecurityEvent) {
        let (event_type, severity, event_data) = self.classify_event(&event);

        let (user_id, ip_address, session_id) = self.extract_metadata(&event);

        let audit_log = AuditLog {
            id: 0, // Will be assigned by database
            timestamp: Utc::now(),
            event_type,
            severity: severity.to_string(),
            event_data,
            user_id,
            ip_address,
            session_id,
            correlation_id: None, // TODO: Implement correlation tracking
        };

        // Log to tracing based on severity
        match severity {
            "critical" => warn!("SECURITY: {:?}", event),
            "warning" => info!("Security event: {:?}", event),
            _ => info!("Audit: {:?}", event),
        }

        // Send to async channel for database write
        if let Err(e) = self.async_channel.send(audit_log) {
            error!("Failed to queue audit log: {}", e);
        }
    }

    fn classify_event(&self, event: &SecurityEvent) -> (String, &'static str, serde_json::Value) {
        match event {
            SecurityEvent::LoginFailure { attempt_count, .. } if *attempt_count > 5 => {
                ("login_failure".to_string(), "critical", serde_json::to_value(event).unwrap())
            }
            SecurityEvent::SqlInjectionAttempt { .. } |
            SecurityEvent::XssAttempt { .. } |
            SecurityEvent::IntrusionDetected { .. } |
            SecurityEvent::DDoSAttackDetected { .. } => {
                ("security_attack".to_string(), "critical", serde_json::to_value(event).unwrap())
            }
            SecurityEvent::SuspiciousTransfer { .. } |
            SecurityEvent::ResourceOverflow { .. } |
            SecurityEvent::PermissionDenied { .. } => {
                ("suspicious_activity".to_string(), "warning", serde_json::to_value(event).unwrap())
            }
            _ => {
                ("normal_activity".to_string(), "info", serde_json::to_value(event).unwrap())
            }
        }
    }

    fn extract_metadata(&self, event: &SecurityEvent) -> (Option<i64>, Option<IpAddr>, Option<String>) {
        match event {
            SecurityEvent::LoginSuccess { user_id, ip, session_id, .. } => {
                (Some(*user_id), Some(*ip), Some(session_id.clone()))
            }
            SecurityEvent::LoginFailure { ip, .. } |
            SecurityEvent::LoginAttempt { ip, .. } => {
                (None, Some(*ip), None)
            }
            SecurityEvent::ProcessManipulation { user_id, .. } |
            SecurityEvent::ResourceOverflow { user_id, .. } => {
                (Some(*user_id), None, None)
            }
            _ => (None, None, None)
        }
    }

    async fn write_log(pool: &PgPool, log: &AuditLog) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO audit_logs (
                timestamp, event_type, severity, event_data,
                user_id, ip_address, session_id, correlation_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            log.timestamp,
            log.event_type,
            log.severity,
            log.event_data,
            log.user_id,
            log.ip_address.map(|ip| ip.to_string()),
            log.session_id,
            log.correlation_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    // Query methods for security analysis
    pub async fn get_failed_login_attempts(&self, ip: IpAddr, window_minutes: i64) -> anyhow::Result<i32> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as count
            FROM audit_logs
            WHERE event_type = 'login_failure'
            AND ip_address = $1
            AND timestamp > NOW() - INTERVAL '$2 minutes'
            "#,
            ip.to_string(),
            window_minutes
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count.unwrap_or(0) as i32)
    }

    pub async fn get_security_events(&self, severity: &str, limit: i64) -> anyhow::Result<Vec<AuditLog>> {
        let logs = sqlx::query_as!(
            AuditLog,
            r#"
            SELECT *
            FROM audit_logs
            WHERE severity = $1
            ORDER BY timestamp DESC
            LIMIT $2
            "#,
            severity,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }
}