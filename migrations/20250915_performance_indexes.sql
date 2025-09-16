-- Performance Optimization: Database Indexes
-- Created: 2025-09-15
-- Purpose: Improve query performance for HackerExperience

-- ============================================
-- USER & AUTHENTICATION INDEXES
-- ============================================

-- Fast user lookups by username (login)
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at DESC);

-- Session management
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);

-- Authentication logs
CREATE INDEX IF NOT EXISTS idx_auth_logs_user_id ON auth_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_auth_logs_ip_address ON auth_logs(ip_address);
CREATE INDEX IF NOT EXISTS idx_auth_logs_timestamp ON auth_logs(timestamp DESC);

-- ============================================
-- GAME PROCESS INDEXES
-- ============================================

-- Process queries by user and status
CREATE INDEX IF NOT EXISTS idx_processes_user_id ON processes(user_id);
CREATE INDEX IF NOT EXISTS idx_processes_status ON processes(status);
CREATE INDEX IF NOT EXISTS idx_processes_user_status ON processes(user_id, status);
CREATE INDEX IF NOT EXISTS idx_processes_completion_time ON processes(completion_time);
CREATE INDEX IF NOT EXISTS idx_processes_priority ON processes(priority DESC, created_at ASC);

-- Process history for analytics
CREATE INDEX IF NOT EXISTS idx_process_history_user_id ON process_history(user_id);
CREATE INDEX IF NOT EXISTS idx_process_history_date ON process_history(completed_at DESC);

-- ============================================
-- SERVER & NETWORK INDEXES
-- ============================================

-- Server ownership and lookups
CREATE INDEX IF NOT EXISTS idx_servers_owner_id ON servers(owner_id);
CREATE INDEX IF NOT EXISTS idx_servers_ip_address ON servers(ip_address);
CREATE INDEX IF NOT EXISTS idx_servers_type ON servers(server_type);
CREATE INDEX IF NOT EXISTS idx_servers_status ON servers(status);

-- Network connections
CREATE INDEX IF NOT EXISTS idx_connections_source_id ON network_connections(source_server_id);
CREATE INDEX IF NOT EXISTS idx_connections_target_id ON network_connections(target_server_id);
CREATE INDEX IF NOT EXISTS idx_connections_active ON network_connections(is_active);

-- Server logs (critical for gameplay)
CREATE INDEX IF NOT EXISTS idx_logs_server_id ON server_logs(server_id);
CREATE INDEX IF NOT EXISTS idx_logs_user_id ON server_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON server_logs(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_logs_type ON server_logs(log_type);

-- Composite index for log queries
CREATE INDEX IF NOT EXISTS idx_logs_server_time ON server_logs(server_id, timestamp DESC);

-- ============================================
-- SOFTWARE & HARDWARE INDEXES
-- ============================================

-- Software ownership and installation
CREATE INDEX IF NOT EXISTS idx_software_owner_id ON software(owner_id);
CREATE INDEX IF NOT EXISTS idx_software_type ON software(software_type);
CREATE INDEX IF NOT EXISTS idx_software_version ON software(version);

-- Software installations on servers
CREATE INDEX IF NOT EXISTS idx_installations_server_id ON software_installations(server_id);
CREATE INDEX IF NOT EXISTS idx_installations_software_id ON software_installations(software_id);

-- Hardware components
CREATE INDEX IF NOT EXISTS idx_hardware_server_id ON hardware_components(server_id);
CREATE INDEX IF NOT EXISTS idx_hardware_type ON hardware_components(component_type);
CREATE INDEX IF NOT EXISTS idx_hardware_level ON hardware_components(level);

-- ============================================
-- FINANCIAL INDEXES
-- ============================================

-- Bank accounts
CREATE INDEX IF NOT EXISTS idx_bank_accounts_user_id ON bank_accounts(user_id);
CREATE INDEX IF NOT EXISTS idx_bank_accounts_number ON bank_accounts(account_number);

-- Transactions
CREATE INDEX IF NOT EXISTS idx_transactions_from_account ON transactions(from_account_id);
CREATE INDEX IF NOT EXISTS idx_transactions_to_account ON transactions(to_account_id);
CREATE INDEX IF NOT EXISTS idx_transactions_timestamp ON transactions(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_transactions_type ON transactions(transaction_type);

-- Bitcoin wallets
CREATE INDEX IF NOT EXISTS idx_bitcoin_user_id ON bitcoin_wallets(user_id);
CREATE INDEX IF NOT EXISTS idx_bitcoin_address ON bitcoin_wallets(wallet_address);

-- ============================================
-- MISSION & QUEST INDEXES
-- ============================================

-- Mission progress tracking
CREATE INDEX IF NOT EXISTS idx_mission_progress_user_id ON mission_progress(user_id);
CREATE INDEX IF NOT EXISTS idx_mission_progress_mission_id ON mission_progress(mission_id);
CREATE INDEX IF NOT EXISTS idx_mission_progress_status ON mission_progress(status);
CREATE INDEX IF NOT EXISTS idx_mission_progress_user_status ON mission_progress(user_id, status);

-- Mission rewards
CREATE INDEX IF NOT EXISTS idx_mission_rewards_user_id ON mission_rewards(user_id);
CREATE INDEX IF NOT EXISTS idx_mission_rewards_claimed ON mission_rewards(claimed_at);

-- ============================================
-- CLAN SYSTEM INDEXES
-- ============================================

-- Clan membership
CREATE INDEX IF NOT EXISTS idx_clan_members_clan_id ON clan_members(clan_id);
CREATE INDEX IF NOT EXISTS idx_clan_members_user_id ON clan_members(user_id);
CREATE INDEX IF NOT EXISTS idx_clan_members_role ON clan_members(role);

-- Clan wars
CREATE INDEX IF NOT EXISTS idx_clan_wars_attacker_id ON clan_wars(attacker_clan_id);
CREATE INDEX IF NOT EXISTS idx_clan_wars_defender_id ON clan_wars(defender_clan_id);
CREATE INDEX IF NOT EXISTS idx_clan_wars_status ON clan_wars(status);
CREATE INDEX IF NOT EXISTS idx_clan_wars_end_time ON clan_wars(end_time);

-- ============================================
-- RANKING & LEADERBOARD INDEXES
-- ============================================

-- Player rankings
CREATE INDEX IF NOT EXISTS idx_rankings_score ON player_rankings(score DESC);
CREATE INDEX IF NOT EXISTS idx_rankings_category ON player_rankings(category, score DESC);
CREATE INDEX IF NOT EXISTS idx_rankings_updated ON player_rankings(updated_at DESC);

-- Clan rankings
CREATE INDEX IF NOT EXISTS idx_clan_rankings_score ON clan_rankings(score DESC);
CREATE INDEX IF NOT EXISTS idx_clan_rankings_updated ON clan_rankings(updated_at DESC);

-- ============================================
-- CHAT & MESSAGING INDEXES
-- ============================================

-- Private messages
CREATE INDEX IF NOT EXISTS idx_messages_sender_id ON messages(sender_id);
CREATE INDEX IF NOT EXISTS idx_messages_recipient_id ON messages(recipient_id);
CREATE INDEX IF NOT EXISTS idx_messages_sent_at ON messages(sent_at DESC);
CREATE INDEX IF NOT EXISTS idx_messages_read ON messages(is_read, recipient_id);

-- Chat messages
CREATE INDEX IF NOT EXISTS idx_chat_clan_id ON clan_chat(clan_id);
CREATE INDEX IF NOT EXISTS idx_chat_timestamp ON clan_chat(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_chat_clan_time ON clan_chat(clan_id, timestamp DESC);

-- ============================================
-- PARTIAL INDEXES FOR OPTIMIZATION
-- ============================================

-- Only index active processes
CREATE INDEX IF NOT EXISTS idx_processes_active
ON processes(user_id, priority DESC)
WHERE status IN ('running', 'queued');

-- Only index unclaimed rewards
CREATE INDEX IF NOT EXISTS idx_rewards_unclaimed
ON mission_rewards(user_id)
WHERE claimed_at IS NULL;

-- Only index online users
CREATE INDEX IF NOT EXISTS idx_users_online
ON users(last_activity)
WHERE is_online = true;

-- Only index active connections
CREATE INDEX IF NOT EXISTS idx_connections_active_only
ON network_connections(source_server_id, target_server_id)
WHERE is_active = true;

-- ============================================
-- FULL TEXT SEARCH INDEXES
-- ============================================

-- Full text search on logs
CREATE INDEX IF NOT EXISTS idx_logs_message_fts ON server_logs USING gin(to_tsvector('english', message));

-- Full text search on chat
CREATE INDEX IF NOT EXISTS idx_chat_message_fts ON clan_chat USING gin(to_tsvector('english', message));

-- Full text search on software names
CREATE INDEX IF NOT EXISTS idx_software_name_fts ON software USING gin(to_tsvector('english', name));

-- ============================================
-- ANALYZE TABLES FOR QUERY PLANNER
-- ============================================

ANALYZE users;
ANALYZE processes;
ANALYZE servers;
ANALYZE server_logs;
ANALYZE network_connections;
ANALYZE software;
ANALYZE software_installations;
ANALYZE hardware_components;
ANALYZE bank_accounts;
ANALYZE transactions;
ANALYZE mission_progress;
ANALYZE clan_members;
ANALYZE clan_wars;
ANALYZE messages;

-- ============================================
-- PERFORMANCE VIEWS
-- ============================================

-- View for active player summary
CREATE OR REPLACE VIEW v_active_players AS
SELECT
    u.user_id,
    u.username,
    u.level,
    COUNT(DISTINCT p.process_id) as active_processes,
    COUNT(DISTINCT s.server_id) as owned_servers,
    b.balance as bank_balance
FROM users u
LEFT JOIN processes p ON u.user_id = p.user_id AND p.status = 'running'
LEFT JOIN servers s ON u.user_id = s.owner_id
LEFT JOIN bank_accounts b ON u.user_id = b.user_id AND b.is_primary = true
WHERE u.is_online = true
GROUP BY u.user_id, u.username, u.level, b.balance;

-- View for server status summary
CREATE OR REPLACE VIEW v_server_status AS
SELECT
    s.server_id,
    s.ip_address,
    s.server_name,
    COUNT(DISTINCT si.software_id) as installed_software,
    COUNT(DISTINCT nc.connection_id) as active_connections,
    MAX(sl.timestamp) as last_log_entry
FROM servers s
LEFT JOIN software_installations si ON s.server_id = si.server_id
LEFT JOIN network_connections nc ON s.server_id = nc.source_server_id AND nc.is_active = true
LEFT JOIN server_logs sl ON s.server_id = sl.server_id
GROUP BY s.server_id, s.ip_address, s.server_name;

-- ============================================
-- MAINTENANCE SETTINGS
-- ============================================

-- Set autovacuum settings for high-activity tables
ALTER TABLE processes SET (autovacuum_vacuum_scale_factor = 0.1);
ALTER TABLE server_logs SET (autovacuum_vacuum_scale_factor = 0.1);
ALTER TABLE network_connections SET (autovacuum_vacuum_scale_factor = 0.1);
ALTER TABLE messages SET (autovacuum_vacuum_scale_factor = 0.1);

-- Set statistics target for frequently queried columns
ALTER TABLE users ALTER COLUMN username SET STATISTICS 1000;
ALTER TABLE processes ALTER COLUMN user_id SET STATISTICS 1000;
ALTER TABLE servers ALTER COLUMN ip_address SET STATISTICS 1000;
ALTER TABLE server_logs ALTER COLUMN server_id SET STATISTICS 1000;