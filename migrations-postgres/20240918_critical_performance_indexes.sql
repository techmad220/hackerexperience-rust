-- Critical Performance Indexes for HackerExperience
-- Date: 2024-09-18
-- Priority: CRITICAL for production performance

-- Process-related queries (high frequency operations)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_processes_user_status_type
ON processes(p_creator_id, is_completed, is_failed, p_type)
WHERE is_completed = FALSE AND is_failed = FALSE;

-- Bank transaction queries (financial operations)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_bank_transactions_account_date
ON bank_transactions(from_account_id, created_at DESC)
INCLUDE (amount, to_account_id, status);

-- Session management (authentication performance)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sessions_user_active
ON sessions(user_id, is_active, last_activity)
WHERE is_active = TRUE;

-- Security logging queries (audit and monitoring)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_security_logs_user_time
ON security_logs(user_id, occurred_at DESC)
WHERE resolved = FALSE;

-- User stats queries (dashboard performance)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_stats_level_exp
ON user_stats(level DESC, experience DESC)
INCLUDE (user_id, money, reputation);

-- Clan member lookups
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_clan_members_active
ON clan_members(user_id, clan_id, status)
WHERE status = 'active';

-- Mission progress tracking
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_mission_progress_user_status
ON mission_progress(user_id, mission_id, status)
WHERE status IN ('active', 'in_progress');

-- Chat messages (recent messages)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_chat_messages_channel_time
ON chat_messages(channel_id, created_at DESC)
WHERE deleted_at IS NULL;

-- Software installations
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_software_installs_pc_type
ON software_installs(pc_id, software_type, is_active)
WHERE is_active = TRUE;

-- Network connections
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_network_connections_active
ON network_connections(source_pc_id, target_pc_id, status)
WHERE status = 'connected';

-- Analyze tables after index creation
ANALYZE processes;
ANALYZE bank_transactions;
ANALYZE sessions;
ANALYZE security_logs;
ANALYZE user_stats;
ANALYZE clan_members;
ANALYZE mission_progress;
ANALYZE chat_messages;
ANALYZE software_installs;
ANALYZE network_connections;