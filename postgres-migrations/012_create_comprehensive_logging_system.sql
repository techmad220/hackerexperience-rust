-- Create comprehensive logging system tables
-- Handles all types of logs: access, security, actions, errors, and audit trails

-- System logs table - Core system and application logging
CREATE TABLE IF NOT EXISTS system_logs (
    id BIGSERIAL PRIMARY KEY,
    
    -- Log identification and categorization
    log_level VARCHAR(10) NOT NULL,               -- DEBUG, INFO, WARN, ERROR, FATAL
    category VARCHAR(30) NOT NULL,                -- system, security, database, network, etc.
    subcategory VARCHAR(30),                      -- More specific categorization
    source_module VARCHAR(50) NOT NULL,           -- Which module/component generated log
    
    -- Log content
    message TEXT NOT NULL,                        -- Primary log message
    details JSONB DEFAULT '{}',                   -- Additional structured data
    error_code VARCHAR(20),                       -- Error code if applicable
    stack_trace TEXT,                             -- Stack trace for errors
    
    -- Context information
    user_id BIGINT,                               -- User context if applicable
    session_id BIGINT,                            -- Session context if applicable
    ip_address INET,                              -- Source IP address
    user_agent TEXT,                              -- User agent string
    request_id VARCHAR(50),                       -- Request tracking ID
    
    -- Timing and performance
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    processing_time_ms INTEGER,                   -- Processing time in milliseconds
    
    -- Log metadata
    server_instance VARCHAR(50),                  -- Which server instance
    build_version VARCHAR(20),                    -- Application version
    environment VARCHAR(20) DEFAULT 'production', -- Environment (dev, staging, prod)
    
    -- Retention and archival
    retention_days INTEGER DEFAULT 90,            -- How long to keep this log
    archived BOOLEAN NOT NULL DEFAULT FALSE,      -- Has been archived
    archived_at TIMESTAMPTZ,                      -- When archived
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE SET NULL,
    
    -- Check constraints
    CONSTRAINT chk_system_logs_level CHECK (log_level IN ('DEBUG', 'INFO', 'WARN', 'ERROR', 'FATAL')),
    CONSTRAINT chk_system_logs_processing_time CHECK (processing_time_ms IS NULL OR processing_time_ms >= 0),
    CONSTRAINT chk_system_logs_retention CHECK (retention_days > 0)
);

-- Access logs table - HTTP requests and API access
CREATE TABLE IF NOT EXISTS access_logs (
    id BIGSERIAL PRIMARY KEY,
    
    -- Request identification
    request_id VARCHAR(50) UNIQUE,                -- Unique request identifier
    session_id BIGINT,                            -- Associated session
    user_id BIGINT,                               -- User making request
    
    -- HTTP request details
    method VARCHAR(10) NOT NULL,                  -- GET, POST, PUT, DELETE, etc.
    url TEXT NOT NULL,                            -- Request URL
    route VARCHAR(255),                           -- Matched route pattern
    query_params JSONB DEFAULT '{}',              -- Query parameters
    
    -- Request headers and body
    headers JSONB DEFAULT '{}',                   -- Request headers
    content_type VARCHAR(100),                    -- Content type
    content_length BIGINT,                        -- Request body size
    
    -- Client information
    ip_address INET NOT NULL,                     -- Client IP
    user_agent TEXT,                              -- User agent
    referer TEXT,                                 -- HTTP referer
    origin VARCHAR(255),                          -- Origin header
    
    -- Response information
    status_code INTEGER NOT NULL,                 -- HTTP status code
    response_size BIGINT,                         -- Response body size
    response_time_ms INTEGER NOT NULL,            -- Response time in milliseconds
    
    -- Security and validation
    rate_limit_hit BOOLEAN NOT NULL DEFAULT FALSE, -- Hit rate limiting
    blocked_by_firewall BOOLEAN NOT NULL DEFAULT FALSE, -- Blocked by firewall
    suspicious_activity BOOLEAN NOT NULL DEFAULT FALSE, -- Flagged as suspicious
    geo_location POINT,                           -- Geographic location
    country_code VARCHAR(2),                      -- Country code
    
    -- API and authentication
    api_key_id BIGINT,                            -- API key used
    oauth_client_id VARCHAR(50),                  -- OAuth client
    auth_method VARCHAR(20),                      -- Authentication method used
    scopes JSONB DEFAULT '[]',                    -- OAuth/API scopes
    
    -- Timing information
    request_start TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    request_end TIMESTAMPTZ,                      -- When request completed
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE SET NULL,
    
    -- Check constraints
    CONSTRAINT chk_access_logs_status_code CHECK (status_code >= 100 AND status_code < 600),
    CONSTRAINT chk_access_logs_response_time CHECK (response_time_ms >= 0),
    CONSTRAINT chk_access_logs_content_length CHECK (content_length IS NULL OR content_length >= 0),
    CONSTRAINT chk_access_logs_response_size CHECK (response_size IS NULL OR response_size >= 0),
    CONSTRAINT chk_access_logs_timing CHECK (request_end IS NULL OR request_end >= request_start)
);

-- Security logs table - Security events and incidents
CREATE TABLE IF NOT EXISTS security_logs (
    id BIGSERIAL PRIMARY KEY,
    
    -- Event identification
    event_type VARCHAR(30) NOT NULL,              -- login_attempt, hack_attempt, suspicious_activity, etc.
    severity VARCHAR(10) NOT NULL,                -- LOW, MEDIUM, HIGH, CRITICAL
    category VARCHAR(30) NOT NULL,                -- authentication, authorization, intrusion, etc.
    
    -- Event details
    description TEXT NOT NULL,                    -- Event description
    details JSONB DEFAULT '{}',                   -- Additional event data
    threat_level INTEGER NOT NULL DEFAULT 1,     -- Threat level (1-10)
    
    -- Affected entities
    user_id BIGINT,                               -- Affected user
    target_user_id BIGINT,                        -- Target user (for attacks)
    server_id BIGINT,                             -- Affected server
    ip_address INET,                              -- Source IP address
    target_ip INET,                               -- Target IP address
    
    -- Security context
    attack_vector VARCHAR(50),                    -- How attack was performed
    exploit_used VARCHAR(100),                    -- Specific exploit or tool
    success BOOLEAN,                              -- Whether attack/attempt succeeded
    blocked BOOLEAN NOT NULL DEFAULT FALSE,       -- Whether event was blocked
    
    -- Investigation and response
    investigated BOOLEAN NOT NULL DEFAULT FALSE,  -- Has been investigated
    resolved BOOLEAN NOT NULL DEFAULT FALSE,      -- Issue has been resolved
    false_positive BOOLEAN NOT NULL DEFAULT FALSE, -- Marked as false positive
    investigator_user_id BIGINT,                  -- Who investigated
    resolution_notes TEXT,                        -- Investigation/resolution notes
    
    -- Geographic and device info
    geo_location POINT,
    country_code VARCHAR(2),
    device_fingerprint VARCHAR(128),
    user_agent TEXT,
    
    -- Related events and correlation
    correlation_id VARCHAR(50),                   -- Group related events
    parent_event_id BIGINT,                       -- Parent security event
    
    -- Timing
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    detected_at TIMESTAMPTZ,                      -- When threat was detected
    resolved_at TIMESTAMPTZ,                      -- When resolved
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (target_user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (investigator_user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (parent_event_id) REFERENCES security_logs(id) ON DELETE SET NULL,
    
    -- Check constraints
    CONSTRAINT chk_security_logs_severity CHECK (severity IN ('LOW', 'MEDIUM', 'HIGH', 'CRITICAL')),
    CONSTRAINT chk_security_logs_threat_level CHECK (threat_level >= 1 AND threat_level <= 10),
    CONSTRAINT chk_security_logs_timing CHECK (
        (detected_at IS NULL OR detected_at >= occurred_at) AND
        (resolved_at IS NULL OR resolved_at >= occurred_at)
    )
);

-- Game action logs table - Player actions and game events
CREATE TABLE IF NOT EXISTS game_action_logs (
    id BIGSERIAL PRIMARY KEY,
    
    -- Action identification
    user_id BIGINT NOT NULL,                      -- User performing action
    action_type VARCHAR(30) NOT NULL,             -- hack, transfer, install, etc.
    action_category VARCHAR(20) NOT NULL,         -- gameplay, financial, social, administrative
    
    -- Action details
    description TEXT NOT NULL,                    -- Human-readable description
    action_data JSONB DEFAULT '{}',               -- Action-specific data
    
    -- Target information
    target_type VARCHAR(30),                      -- user, server, software, account, etc.
    target_id BIGINT,                             -- ID of target entity
    target_info JSONB DEFAULT '{}',               -- Additional target information
    
    -- Action context
    server_ip INET,                               -- Server where action occurred
    process_id BIGINT,                            -- Related process if applicable
    mission_id BIGINT,                            -- Related mission if applicable
    clan_id BIGINT,                               -- Clan context if applicable
    
    -- Action results
    success BOOLEAN NOT NULL,                     -- Whether action succeeded
    result_code INTEGER,                          -- Result/error code
    result_message TEXT,                          -- Result message
    
    -- Financial impact
    money_change BIGINT DEFAULT 0,                -- Money gained/lost
    fee_paid BIGINT DEFAULT 0,                    -- Fees paid for action
    
    -- Experience and progression
    experience_gained INTEGER DEFAULT 0,          -- Experience points gained
    reputation_change INTEGER DEFAULT 0,          -- Reputation change
    skill_progress JSONB DEFAULT '{}',            -- Skill progression data
    
    -- Timing and performance
    started_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMPTZ,                     -- When action completed
    duration_ms INTEGER,                          -- Action duration
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (process_id) REFERENCES processes(pid) ON DELETE SET NULL,
    FOREIGN KEY (clan_id) REFERENCES clans(id) ON DELETE SET NULL,
    
    -- Check constraints
    CONSTRAINT chk_game_logs_duration CHECK (duration_ms IS NULL OR duration_ms >= 0),
    CONSTRAINT chk_game_logs_timing CHECK (completed_at IS NULL OR completed_at >= started_at),
    CONSTRAINT chk_game_logs_fees CHECK (fee_paid >= 0)
);

-- Audit logs table - Administrative actions and system changes
CREATE TABLE IF NOT EXISTS audit_logs (
    id BIGSERIAL PRIMARY KEY,
    
    -- Action identification
    actor_user_id BIGINT,                         -- User performing action
    actor_type VARCHAR(20) NOT NULL DEFAULT 'user', -- user, system, api, cron
    action VARCHAR(50) NOT NULL,                  -- create, update, delete, etc.
    resource_type VARCHAR(30) NOT NULL,           -- users, clans, missions, etc.
    resource_id BIGINT,                           -- ID of affected resource
    
    -- Change tracking
    old_values JSONB DEFAULT '{}',                -- Previous values
    new_values JSONB DEFAULT '{}',                -- New values
    changed_fields JSONB DEFAULT '[]',            -- List of changed fields
    
    -- Context and reasoning
    reason TEXT,                                  -- Reason for change
    notes TEXT,                                   -- Additional notes
    request_id VARCHAR(50),                       -- Related request ID
    
    -- Administrative context
    admin_level INTEGER,                          -- Admin level of actor
    approval_required BOOLEAN DEFAULT FALSE,      -- Required approval
    approved_by_user_id BIGINT,                   -- Who approved
    approved_at TIMESTAMPTZ,                      -- When approved
    
    -- Impact assessment
    severity VARCHAR(10) DEFAULT 'LOW',           -- LOW, MEDIUM, HIGH, CRITICAL
    affects_users INTEGER DEFAULT 0,              -- Number of users affected
    downtime_seconds INTEGER DEFAULT 0,           -- Downtime caused
    
    -- Source information
    ip_address INET,                              -- Source IP
    user_agent TEXT,                              -- User agent
    session_id BIGINT,                            -- Session context
    
    -- Timestamps
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (actor_user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (approved_by_user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE SET NULL,
    
    -- Check constraints
    CONSTRAINT chk_audit_logs_severity CHECK (severity IN ('LOW', 'MEDIUM', 'HIGH', 'CRITICAL')),
    CONSTRAINT chk_audit_logs_affects_users CHECK (affects_users >= 0),
    CONSTRAINT chk_audit_logs_downtime CHECK (downtime_seconds >= 0)
);

-- Performance logs table - System performance metrics
CREATE TABLE IF NOT EXISTS performance_logs (
    id BIGSERIAL PRIMARY KEY,
    
    -- Metric identification
    metric_name VARCHAR(50) NOT NULL,             -- cpu_usage, memory_usage, response_time, etc.
    metric_category VARCHAR(30) NOT NULL,         -- system, database, application, network
    component VARCHAR(50) NOT NULL,               -- Which component/service
    
    -- Metric values
    value DECIMAL(15,6) NOT NULL,                 -- Metric value
    unit VARCHAR(20) NOT NULL,                    -- Unit of measurement
    threshold_warning DECIMAL(15,6),              -- Warning threshold
    threshold_critical DECIMAL(15,6),             -- Critical threshold
    
    -- Status and alerting
    status VARCHAR(10) DEFAULT 'OK',              -- OK, WARNING, CRITICAL
    alert_sent BOOLEAN NOT NULL DEFAULT FALSE,    -- Whether alert was sent
    alert_recipients JSONB DEFAULT '[]',          -- Who was alerted
    
    -- Context information
    server_instance VARCHAR(50),                  -- Server instance
    environment VARCHAR(20),                      -- Environment
    region VARCHAR(50),                           -- Geographic region
    
    -- Additional metrics
    additional_data JSONB DEFAULT '{}',           -- Additional metric data
    tags JSONB DEFAULT '[]',                      -- Metric tags
    
    -- Timing
    measured_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Check constraints
    CONSTRAINT chk_performance_logs_status CHECK (status IN ('OK', 'WARNING', 'CRITICAL')),
    CONSTRAINT chk_performance_logs_thresholds CHECK (
        threshold_warning IS NULL OR threshold_critical IS NULL OR
        threshold_critical >= threshold_warning
    )
);

-- Create comprehensive indexes for all log tables
-- System logs indexes
CREATE INDEX IF NOT EXISTS idx_system_logs_level ON system_logs(log_level);
CREATE INDEX IF NOT EXISTS idx_system_logs_category ON system_logs(category);
CREATE INDEX IF NOT EXISTS idx_system_logs_source_module ON system_logs(source_module);
CREATE INDEX IF NOT EXISTS idx_system_logs_user_id ON system_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_system_logs_occurred_at ON system_logs(occurred_at);
CREATE INDEX IF NOT EXISTS idx_system_logs_ip_address ON system_logs(ip_address);
CREATE INDEX IF NOT EXISTS idx_system_logs_archived ON system_logs(archived);

-- Access logs indexes  
CREATE INDEX IF NOT EXISTS idx_access_logs_user_id ON access_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_access_logs_session_id ON access_logs(session_id);
CREATE INDEX IF NOT EXISTS idx_access_logs_ip_address ON access_logs(ip_address);
CREATE INDEX IF NOT EXISTS idx_access_logs_method ON access_logs(method);
CREATE INDEX IF NOT EXISTS idx_access_logs_status_code ON access_logs(status_code);
CREATE INDEX IF NOT EXISTS idx_access_logs_request_start ON access_logs(request_start);
CREATE INDEX IF NOT EXISTS idx_access_logs_response_time ON access_logs(response_time_ms);
CREATE INDEX IF NOT EXISTS idx_access_logs_suspicious ON access_logs(suspicious_activity);

-- Security logs indexes
CREATE INDEX IF NOT EXISTS idx_security_logs_event_type ON security_logs(event_type);
CREATE INDEX IF NOT EXISTS idx_security_logs_severity ON security_logs(severity);
CREATE INDEX IF NOT EXISTS idx_security_logs_user_id ON security_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_security_logs_target_user_id ON security_logs(target_user_id);
CREATE INDEX IF NOT EXISTS idx_security_logs_ip_address ON security_logs(ip_address);
CREATE INDEX IF NOT EXISTS idx_security_logs_occurred_at ON security_logs(occurred_at);
CREATE INDEX IF NOT EXISTS idx_security_logs_threat_level ON security_logs(threat_level);
CREATE INDEX IF NOT EXISTS idx_security_logs_resolved ON security_logs(resolved);
CREATE INDEX IF NOT EXISTS idx_security_logs_correlation ON security_logs(correlation_id);

-- Game action logs indexes
CREATE INDEX IF NOT EXISTS idx_game_logs_user_id ON game_action_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_game_logs_action_type ON game_action_logs(action_type);
CREATE INDEX IF NOT EXISTS idx_game_logs_category ON game_action_logs(action_category);
CREATE INDEX IF NOT EXISTS idx_game_logs_target_type ON game_action_logs(target_type);
CREATE INDEX IF NOT EXISTS idx_game_logs_success ON game_action_logs(success);
CREATE INDEX IF NOT EXISTS idx_game_logs_started_at ON game_action_logs(started_at);
CREATE INDEX IF NOT EXISTS idx_game_logs_server_ip ON game_action_logs(server_ip);

-- Audit logs indexes
CREATE INDEX IF NOT EXISTS idx_audit_logs_actor ON audit_logs(actor_user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action);
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource_type ON audit_logs(resource_type);
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource_id ON audit_logs(resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_occurred_at ON audit_logs(occurred_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_severity ON audit_logs(severity);

-- Performance logs indexes
CREATE INDEX IF NOT EXISTS idx_performance_logs_metric_name ON performance_logs(metric_name);
CREATE INDEX IF NOT EXISTS idx_performance_logs_category ON performance_logs(metric_category);
CREATE INDEX IF NOT EXISTS idx_performance_logs_component ON performance_logs(component);
CREATE INDEX IF NOT EXISTS idx_performance_logs_status ON performance_logs(status);
CREATE INDEX IF NOT EXISTS idx_performance_logs_measured_at ON performance_logs(measured_at);

-- GIN indexes for JSONB columns
CREATE INDEX IF NOT EXISTS idx_system_logs_details ON system_logs USING GIN(details);
CREATE INDEX IF NOT EXISTS idx_access_logs_headers ON access_logs USING GIN(headers);
CREATE INDEX IF NOT EXISTS idx_security_logs_details ON security_logs USING GIN(details);
CREATE INDEX IF NOT EXISTS idx_game_logs_action_data ON game_action_logs USING GIN(action_data);
CREATE INDEX IF NOT EXISTS idx_audit_logs_old_values ON audit_logs USING GIN(old_values);
CREATE INDEX IF NOT EXISTS idx_audit_logs_new_values ON audit_logs USING GIN(new_values);
CREATE INDEX IF NOT EXISTS idx_performance_logs_data ON performance_logs USING GIN(additional_data);

-- GIST indexes for geographic data
CREATE INDEX IF NOT EXISTS idx_access_logs_geo ON access_logs USING GIST(geo_location);
CREATE INDEX IF NOT EXISTS idx_security_logs_geo ON security_logs USING GIST(geo_location);

-- Create triggers for updated_at where applicable
CREATE TRIGGER security_logs_updated_at_trigger
    BEFORE UPDATE ON security_logs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to log user actions
CREATE OR REPLACE FUNCTION log_user_action(
    user_id_param BIGINT,
    action_type_param VARCHAR(30),
    description_param TEXT,
    success_param BOOLEAN DEFAULT TRUE,
    target_type_param VARCHAR(30) DEFAULT NULL,
    target_id_param BIGINT DEFAULT NULL,
    money_change_param BIGINT DEFAULT 0,
    experience_gain_param INTEGER DEFAULT 0
) RETURNS BIGINT AS $$
DECLARE
    log_id BIGINT;
    action_category VARCHAR(20);
BEGIN
    -- Determine action category based on type
    action_category := CASE 
        WHEN action_type_param IN ('hack', 'scan', 'crack') THEN 'gameplay'
        WHEN action_type_param IN ('transfer', 'deposit', 'withdraw') THEN 'financial'
        WHEN action_type_param IN ('join_clan', 'leave_clan', 'message') THEN 'social'
        WHEN action_type_param IN ('ban', 'kick', 'promote') THEN 'administrative'
        ELSE 'gameplay'
    END;
    
    -- Insert log record
    INSERT INTO game_action_logs (
        user_id, action_type, action_category, description, success,
        target_type, target_id, money_change, experience_gained
    ) VALUES (
        user_id_param, action_type_param, action_category, description_param, success_param,
        target_type_param, target_id_param, money_change_param, experience_gain_param
    ) RETURNING id INTO log_id;
    
    RETURN log_id;
END;
$$ LANGUAGE plpgsql;

-- Function to log security events
CREATE OR REPLACE FUNCTION log_security_event(
    event_type_param VARCHAR(30),
    severity_param VARCHAR(10),
    description_param TEXT,
    user_id_param BIGINT DEFAULT NULL,
    ip_address_param INET DEFAULT NULL,
    success_param BOOLEAN DEFAULT NULL,
    details_param JSONB DEFAULT '{}'
) RETURNS BIGINT AS $$
DECLARE
    log_id BIGINT;
    category_val VARCHAR(30);
    threat_level_val INTEGER;
BEGIN
    -- Determine category and threat level based on event type
    category_val := CASE 
        WHEN event_type_param IN ('login_attempt', 'failed_login') THEN 'authentication'
        WHEN event_type_param IN ('hack_attempt', 'intrusion') THEN 'intrusion'
        WHEN event_type_param IN ('permission_denied', 'unauthorized_access') THEN 'authorization'
        ELSE 'general'
    END;
    
    threat_level_val := CASE severity_param
        WHEN 'LOW' THEN 2
        WHEN 'MEDIUM' THEN 5
        WHEN 'HIGH' THEN 7
        WHEN 'CRITICAL' THEN 9
        ELSE 1
    END;
    
    -- Insert security log
    INSERT INTO security_logs (
        event_type, severity, category, description, details,
        user_id, ip_address, success, threat_level
    ) VALUES (
        event_type_param, severity_param, category_val, description_param, details_param,
        user_id_param, ip_address_param, success_param, threat_level_val
    ) RETURNING id INTO log_id;
    
    RETURN log_id;
END;
$$ LANGUAGE plpgsql;

-- Function to clean up old logs based on retention policies
CREATE OR REPLACE FUNCTION cleanup_old_logs()
RETURNS TABLE(logs_deleted BIGINT, tables_affected INTEGER) AS $$
DECLARE
    deleted_count BIGINT := 0;
    total_deleted BIGINT := 0;
    affected_tables INTEGER := 0;
BEGIN
    -- Clean up system logs based on retention_days
    DELETE FROM system_logs 
    WHERE created_at < CURRENT_TIMESTAMP - INTERVAL '1 day' * retention_days;
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    IF deleted_count > 0 THEN
        total_deleted := total_deleted + deleted_count;
        affected_tables := affected_tables + 1;
    END IF;
    
    -- Clean up access logs (default 30 days)
    DELETE FROM access_logs 
    WHERE created_at < CURRENT_TIMESTAMP - INTERVAL '30 days';
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    IF deleted_count > 0 THEN
        total_deleted := total_deleted + deleted_count;
        affected_tables := affected_tables + 1;
    END IF;
    
    -- Clean up game action logs (default 90 days)
    DELETE FROM game_action_logs 
    WHERE created_at < CURRENT_TIMESTAMP - INTERVAL '90 days';
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    IF deleted_count > 0 THEN
        total_deleted := total_deleted + deleted_count;
        affected_tables := affected_tables + 1;
    END IF;
    
    -- Keep security logs longer (default 365 days)
    DELETE FROM security_logs 
    WHERE created_at < CURRENT_TIMESTAMP - INTERVAL '365 days'
    AND resolved = TRUE AND false_positive = TRUE;
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    IF deleted_count > 0 THEN
        total_deleted := total_deleted + deleted_count;
        affected_tables := affected_tables + 1;
    END IF;
    
    -- Keep audit logs longest (default 7 years for compliance)
    DELETE FROM audit_logs 
    WHERE created_at < CURRENT_TIMESTAMP - INTERVAL '7 years';
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    IF deleted_count > 0 THEN
        total_deleted := total_deleted + deleted_count;
        affected_tables := affected_tables + 1;
    END IF;
    
    -- Clean up performance logs (default 30 days)
    DELETE FROM performance_logs 
    WHERE created_at < CURRENT_TIMESTAMP - INTERVAL '30 days';
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    IF deleted_count > 0 THEN
        total_deleted := total_deleted + deleted_count;
        affected_tables := affected_tables + 1;
    END IF;
    
    logs_deleted := total_deleted;
    tables_affected := affected_tables;
    RETURN NEXT;
END;
$$ LANGUAGE plpgsql;

-- Create materialized view for log statistics
CREATE MATERIALIZED VIEW log_statistics AS
SELECT 
    'system_logs' as table_name,
    COUNT(*) as total_records,
    COUNT(CASE WHEN log_level = 'ERROR' THEN 1 END) as error_count,
    COUNT(CASE WHEN log_level = 'WARN' THEN 1 END) as warning_count,
    MIN(created_at) as oldest_record,
    MAX(created_at) as newest_record
FROM system_logs
UNION ALL
SELECT 
    'access_logs' as table_name,
    COUNT(*) as total_records,
    COUNT(CASE WHEN status_code >= 400 THEN 1 END) as error_count,
    COUNT(CASE WHEN suspicious_activity THEN 1 END) as warning_count,
    MIN(created_at) as oldest_record,
    MAX(created_at) as newest_record
FROM access_logs
UNION ALL
SELECT 
    'security_logs' as table_name,
    COUNT(*) as total_records,
    COUNT(CASE WHEN severity = 'CRITICAL' THEN 1 END) as error_count,
    COUNT(CASE WHEN severity IN ('HIGH', 'MEDIUM') THEN 1 END) as warning_count,
    MIN(created_at) as oldest_record,
    MAX(created_at) as newest_record
FROM security_logs
UNION ALL
SELECT 
    'game_action_logs' as table_name,
    COUNT(*) as total_records,
    COUNT(CASE WHEN NOT success THEN 1 END) as error_count,
    0 as warning_count,
    MIN(created_at) as oldest_record,
    MAX(created_at) as newest_record
FROM game_action_logs
UNION ALL
SELECT 
    'audit_logs' as table_name,
    COUNT(*) as total_records,
    COUNT(CASE WHEN severity = 'CRITICAL' THEN 1 END) as error_count,
    COUNT(CASE WHEN severity IN ('HIGH', 'MEDIUM') THEN 1 END) as warning_count,
    MIN(created_at) as oldest_record,
    MAX(created_at) as newest_record
FROM audit_logs;

-- Function to refresh log statistics
CREATE OR REPLACE FUNCTION refresh_log_statistics()
RETURNS VOID AS $$
BEGIN
    REFRESH MATERIALIZED VIEW log_statistics;
END;
$$ LANGUAGE plpgsql;

-- Add comprehensive comments
COMMENT ON TABLE system_logs IS 'Core system and application logging with structured data and retention policies';
COMMENT ON TABLE access_logs IS 'HTTP access logs with request/response details and security tracking';
COMMENT ON TABLE security_logs IS 'Security events, incidents, and threat detection with investigation tracking';
COMMENT ON TABLE game_action_logs IS 'Player actions and game events with context and results';
COMMENT ON TABLE audit_logs IS 'Administrative actions and system changes for compliance and auditing';
COMMENT ON TABLE performance_logs IS 'System performance metrics with alerting and monitoring';

COMMENT ON FUNCTION log_user_action IS 'Logs user gameplay actions with automatic categorization';
COMMENT ON FUNCTION log_security_event IS 'Logs security events with automatic threat level assessment';
COMMENT ON FUNCTION cleanup_old_logs IS 'Removes old log records based on retention policies';
COMMENT ON MATERIALIZED VIEW log_statistics IS 'Statistical overview of log tables and record counts';