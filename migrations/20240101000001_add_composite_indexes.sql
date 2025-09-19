-- Add composite indexes for performance optimization
-- These indexes target the most common query patterns identified in the codebase

-- Users table composite indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_username_active
ON users(username, active)
WHERE active = true;

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_email_active
ON users(email, active)
WHERE active = true;

-- Processes table composite indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_processes_user_status
ON processes(p_creator_id, is_paused, p_time_end)
WHERE is_paused = 0;

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_processes_active
ON processes(p_creator_id)
WHERE is_paused = 0 AND p_time_end > NOW();

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_processes_victim_status
ON processes(p_victim_id, is_paused, p_action);

-- Servers table composite indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_servers_owner_type
ON servers(owner_id, server_type, status)
WHERE status = 'active';

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_servers_ip_active
ON servers(ip_address, status)
WHERE status = 'active';

-- Hardware table composite indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_hardware_server_component
ON hardware(server_id, component_type, level);

-- Software table composite indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_software_server_type
ON software(server_id, software_type, version);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_software_server_active
ON software(server_id, is_active)
WHERE is_active = true;

-- Logs table composite indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_logs_server_time
ON logs(server_id, created_at DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_logs_user_action
ON logs(user_id, action_type, created_at DESC);

-- Banking accounts composite indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_bank_accounts_user_active
ON bank_accounts(user_id, is_active)
WHERE is_active = true;

-- Missions table composite indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_missions_user_status
ON missions(user_id, status, deadline)
WHERE status IN ('active', 'pending');

-- Clans table composite indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_clan_members_clan_role
ON clan_members(clan_id, role, joined_at);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_clan_members_user_active
ON clan_members(user_id, is_active)
WHERE is_active = true;

-- Sessions table composite indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sessions_user_active
ON sessions(user_id, expires_at)
WHERE expires_at > NOW();

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sessions_token_active
ON sessions(session_token, expires_at)
WHERE expires_at > NOW();

-- Leaderboard materialized view indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_leaderboard_rank
ON leaderboard_cache(ranking_type, rank);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_leaderboard_user
ON leaderboard_cache(user_id, ranking_type);

-- Messages table composite indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_messages_recipient_unread
ON messages(recipient_id, is_read, created_at DESC)
WHERE is_read = false;

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_messages_conversation
ON messages(sender_id, recipient_id, created_at DESC);

-- Transactions table composite indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_transactions_user_time
ON transactions(user_id, transaction_type, created_at DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_transactions_account_time
ON transactions(account_id, created_at DESC);

-- Skills table composite indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_skills_user_category
ON skills(user_id, skill_category, level);

-- Achievements table composite indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_achievements_user_unlocked
ON user_achievements(user_id, unlocked_at)
WHERE unlocked_at IS NOT NULL;

-- Files table composite indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_server_directory
ON files(server_id, directory_path, file_name);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_server_type
ON files(server_id, file_type, size);

-- Network connections composite indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_network_connections_active
ON network_connections(source_server_id, target_server_id, is_active)
WHERE is_active = true;

-- Process queue composite indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_process_queue_user_priority
ON process_queue(user_id, priority DESC, created_at)
WHERE status = 'pending';

-- Add partial indexes for common WHERE clauses
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_online
ON users(last_activity)
WHERE last_activity > NOW() - INTERVAL '15 minutes';

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_processes_running
ON processes(p_time_end)
WHERE is_paused = 0 AND p_time_end > NOW();

-- Add GIN indexes for JSONB columns if any exist
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_metadata_gin
ON users USING GIN (metadata jsonb_path_ops)
WHERE metadata IS NOT NULL;

-- Add text search indexes for search functionality
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_search
ON users USING GIN (to_tsvector('english', username || ' ' || COALESCE(display_name, '')));

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_servers_search
ON servers USING GIN (to_tsvector('english', server_name || ' ' || COALESCE(description, '')));

-- Analyze tables after index creation for query planner
ANALYZE users;
ANALYZE processes;
ANALYZE servers;
ANALYZE hardware;
ANALYZE software;
ANALYZE logs;
ANALYZE bank_accounts;
ANALYZE missions;
ANALYZE clan_members;
ANALYZE sessions;
ANALYZE messages;
ANALYZE transactions;
ANALYZE skills;
ANALYZE user_achievements;
ANALYZE files;
ANALYZE network_connections;
ANALYZE process_queue;