-- Performance Optimization Indexes and Query Improvements

-- ============================================
-- CRITICAL PATH INDEXES
-- ============================================

-- User authentication (most frequent query)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_username_password
ON users(username)
INCLUDE (id, password_hash)
WHERE deleted_at IS NULL;

-- Session lookups
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sessions_token
ON sessions(token)
INCLUDE (user_id, expires_at)
WHERE expires_at > NOW();

-- ============================================
-- GAME WORLD INDEXES
-- ============================================

-- Server lookups by IP (very frequent)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_servers_ip
ON servers(ip_address)
INCLUDE (owner_id, server_type, is_online);

-- User's owned servers
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_servers_owner
ON servers(owner_id, is_online)
WHERE deleted_at IS NULL;

-- Hacked servers tracking
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_hacked_servers
ON hacked_servers(player_id, server_id)
INCLUDE (hacked_at, access_level);

-- ============================================
-- PROCESS MANAGEMENT INDEXES
-- ============================================

-- Active processes (checked constantly)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_processes_active
ON processes(player_id, status)
INCLUDE (process_type, target_server, completion_time)
WHERE status IN ('running', 'queued');

-- Process completion checks
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_processes_completion
ON processes(completion_time, status)
WHERE status = 'running' AND completion_time <= NOW() + INTERVAL '1 minute';

-- ============================================
-- HARDWARE & SOFTWARE INDEXES
-- ============================================

-- Player hardware lookups
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_hardware_player
ON hardware(player_id, hardware_type)
INCLUDE (level, capacity);

-- Software inventory
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_software_inventory
ON software_inventory(player_id, software_type)
INCLUDE (version, size_mb)
WHERE deleted_at IS NULL;

-- ============================================
-- PROGRESSION INDEXES
-- ============================================

-- Leaderboard queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_progression_leaderboard
ON player_progression(level DESC, total_experience DESC)
INCLUDE (player_id);

-- PvP rankings
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_statistics_pvp
ON player_statistics(pvp_wins DESC, pvp_losses)
INCLUDE (player_id)
WHERE pvp_matches > 0;

-- Achievement checks
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_achievements_check
ON player_achievements(player_id, achievement_id);

-- ============================================
-- MULTIPLAYER INDEXES
-- ============================================

-- Clan member lookups
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_clan_members
ON clan_members(clan_id, role)
INCLUDE (player_id, joined_at)
WHERE left_at IS NULL;

-- Active PvP matches
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_pvp_active
ON pvp_matches(status, started_at)
WHERE status IN ('challenging', 'in_progress');

-- Chat messages (recent first)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_chat_messages
ON chat_messages(room_id, created_at DESC)
INCLUDE (sender_id, content);

-- Trade listings
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_market_listings
ON market_listings(item_type, price)
WHERE status = 'active' AND expires_at > NOW();

-- ============================================
-- MATERIALIZED VIEWS FOR EXPENSIVE QUERIES
-- ============================================

-- Server statistics (refreshed hourly)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_server_stats AS
SELECT
    s.server_type,
    s.tier,
    COUNT(*) as total_servers,
    COUNT(DISTINCT hs.player_id) as unique_hackers,
    AVG(s.security_level) as avg_security,
    SUM(s.money_available) as total_money
FROM servers s
LEFT JOIN hacked_servers hs ON s.id = hs.server_id
GROUP BY s.server_type, s.tier;

CREATE UNIQUE INDEX ON mv_server_stats(server_type, tier);

-- Player rankings (refreshed every 15 minutes)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_player_rankings AS
SELECT
    p.player_id,
    u.username,
    p.level,
    p.total_experience,
    ps.pvp_wins,
    ps.servers_hacked,
    RANK() OVER (ORDER BY p.level DESC, p.total_experience DESC) as level_rank,
    RANK() OVER (ORDER BY ps.pvp_wins DESC) as pvp_rank,
    RANK() OVER (ORDER BY ps.servers_hacked DESC) as hacking_rank
FROM player_progression p
JOIN users u ON p.player_id = u.id
LEFT JOIN player_statistics ps ON p.player_id = ps.player_id
WHERE u.deleted_at IS NULL;

CREATE UNIQUE INDEX ON mv_player_rankings(player_id);
CREATE INDEX ON mv_player_rankings(level_rank) WHERE level_rank <= 100;
CREATE INDEX ON mv_player_rankings(pvp_rank) WHERE pvp_rank <= 100;

-- Clan power rankings (refreshed every 30 minutes)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_clan_rankings AS
SELECT
    c.id as clan_id,
    c.name,
    c.level,
    COUNT(DISTINCT cm.player_id) as member_count,
    SUM(pp.level) as total_member_levels,
    COUNT(DISTINCT ct.territory_id) as territories,
    c.bank_balance,
    RANK() OVER (ORDER BY c.level DESC, COUNT(DISTINCT cm.player_id) DESC) as rank
FROM clans c
LEFT JOIN clan_members cm ON c.id = cm.clan_id AND cm.left_at IS NULL
LEFT JOIN player_progression pp ON cm.player_id = pp.player_id
LEFT JOIN clan_territories ct ON c.id = ct.clan_id
GROUP BY c.id, c.name, c.level, c.bank_balance;

CREATE UNIQUE INDEX ON mv_clan_rankings(clan_id);
CREATE INDEX ON mv_clan_rankings(rank) WHERE rank <= 100;

-- ============================================
-- QUERY OPTIMIZATION FUNCTIONS
-- ============================================

-- Fast player stats lookup
CREATE OR REPLACE FUNCTION get_player_stats(p_player_id UUID)
RETURNS TABLE (
    level INT,
    experience BIGINT,
    money BIGINT,
    servers_hacked INT,
    pvp_wins INT,
    clan_name TEXT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        pp.level,
        pp.total_experience,
        pa.balance as money,
        ps.servers_hacked,
        ps.pvp_wins,
        c.name as clan_name
    FROM player_progression pp
    LEFT JOIN player_accounts pa ON pp.player_id = pa.player_id
    LEFT JOIN player_statistics ps ON pp.player_id = ps.player_id
    LEFT JOIN clan_members cm ON pp.player_id = cm.player_id AND cm.left_at IS NULL
    LEFT JOIN clans c ON cm.clan_id = c.id
    WHERE pp.player_id = p_player_id;
END;
$$ LANGUAGE plpgsql STABLE;

-- Batch process completion check
CREATE OR REPLACE FUNCTION complete_finished_processes()
RETURNS VOID AS $$
BEGIN
    UPDATE processes
    SET status = 'completed',
        completed_at = NOW()
    WHERE status = 'running'
    AND completion_time <= NOW();

    -- Trigger any completion events
    PERFORM pg_notify('process_completed', json_build_object(
        'process_ids', array_agg(id)
    )::text)
    FROM processes
    WHERE status = 'running'
    AND completion_time <= NOW();
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- PARTITIONING FOR LARGE TABLES
-- ============================================

-- Partition chat messages by month
CREATE TABLE IF NOT EXISTS chat_messages_2024_01 PARTITION OF chat_messages
FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');

CREATE TABLE IF NOT EXISTS chat_messages_2024_02 PARTITION OF chat_messages
FOR VALUES FROM ('2024-02-01') TO ('2024-03-01');

-- Auto-create future partitions
CREATE OR REPLACE FUNCTION create_monthly_partitions()
RETURNS VOID AS $$
DECLARE
    start_date DATE;
    end_date DATE;
    partition_name TEXT;
BEGIN
    start_date := DATE_TRUNC('month', CURRENT_DATE);
    end_date := start_date + INTERVAL '1 month';

    FOR i IN 0..2 LOOP
        partition_name := 'chat_messages_' || TO_CHAR(start_date, 'YYYY_MM');

        EXECUTE format('CREATE TABLE IF NOT EXISTS %I PARTITION OF chat_messages FOR VALUES FROM (%L) TO (%L)',
            partition_name, start_date, end_date);

        start_date := end_date;
        end_date := start_date + INTERVAL '1 month';
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- REFRESH MATERIALIZED VIEWS
-- ============================================

-- Schedule periodic refreshes (use pg_cron in production)
CREATE OR REPLACE FUNCTION refresh_all_materialized_views()
RETURNS VOID AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_server_stats;
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_player_rankings;
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_clan_rankings;
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- ANALYZE TABLES FOR QUERY PLANNER
-- ============================================
ANALYZE users;
ANALYZE servers;
ANALYZE processes;
ANALYZE player_progression;
ANALYZE player_statistics;
ANALYZE clan_members;
ANALYZE chat_messages;