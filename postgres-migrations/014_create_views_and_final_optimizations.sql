-- Create comprehensive views, final optimizations, and database maintenance functions
-- This is the final migration that ties all systems together

-- Master dashboard view - Key metrics for admin dashboard
CREATE OR REPLACE VIEW admin_dashboard AS
SELECT 
    -- User statistics
    (SELECT COUNT(*) FROM users) as total_users,
    (SELECT COUNT(*) FROM users WHERE last_login > CURRENT_TIMESTAMP - INTERVAL '24 hours') as active_users_24h,
    (SELECT COUNT(*) FROM users WHERE last_login > CURRENT_TIMESTAMP - INTERVAL '7 days') as active_users_7d,
    (SELECT COUNT(*) FROM users WHERE premium = TRUE) as premium_users,
    
    -- Session statistics
    (SELECT COUNT(*) FROM sessions WHERE is_active = TRUE) as active_sessions,
    (SELECT COUNT(*) FROM sessions WHERE suspicious_activity = TRUE) as suspicious_sessions,
    
    -- Process statistics
    (SELECT COUNT(*) FROM processes WHERE is_completed = FALSE AND is_failed = FALSE) as active_processes,
    (SELECT COUNT(*) FROM processes WHERE is_paused = TRUE) as paused_processes,
    
    -- Server statistics
    (SELECT COUNT(*) FROM hardware WHERE online_status = TRUE) as online_servers,
    (SELECT COUNT(*) FROM hardware_external WHERE online_status = TRUE) as online_external_servers,
    
    -- Financial statistics
    (SELECT SUM(balance) FROM bank_accounts WHERE account_status = 'active') as total_bank_balance,
    (SELECT COUNT(*) FROM bank_transactions WHERE status = 'pending') as pending_transactions,
    
    -- Clan statistics
    (SELECT COUNT(*) FROM clans WHERE is_active = TRUE) as active_clans,
    (SELECT COUNT(*) FROM clan_wars WHERE war_status = 'active') as active_wars,
    
    -- Mission statistics
    (SELECT COUNT(*) FROM user_missions WHERE status = 'active') as active_missions,
    (SELECT COUNT(*) FROM mission_templates WHERE is_available = TRUE) as available_missions,
    
    -- Security statistics
    (SELECT COUNT(*) FROM security_logs WHERE severity = 'CRITICAL' AND occurred_at > CURRENT_TIMESTAMP - INTERVAL '24 hours') as critical_security_events_24h,
    (SELECT COUNT(*) FROM security_logs WHERE resolved = FALSE) as unresolved_security_events,
    
    -- Network statistics
    (SELECT COUNT(*) FROM network_nodes WHERE online_status = TRUE) as online_network_nodes,
    (SELECT COUNT(*) FROM network_connections WHERE connection_state = 'ESTABLISHED') as active_connections,
    
    -- System health
    CURRENT_TIMESTAMP as last_updated;

-- Player dashboard view - Key information for player interface
CREATE OR REPLACE VIEW player_dashboard AS
SELECT 
    u.id as user_id,
    u.login,
    u.email,
    u.last_login,
    u.premium,
    
    -- Player statistics
    us.level,
    us.experience,
    us.money,
    us.reputation,
    us.servers_hacked,
    us.missions_completed,
    
    -- Hardware information
    h.server_id as primary_server_id,
    h.name as primary_server_name,
    h.cpu,
    h.ram,
    h.current_load,
    h.online_status as server_online,
    
    -- Clan information
    c.id as clan_id,
    c.name as clan_name,
    c.tag as clan_tag,
    cm.role as clan_role,
    
    -- Active processes
    (SELECT COUNT(*) FROM processes p WHERE p.p_creator_id = u.id AND p.is_completed = FALSE AND p.is_failed = FALSE) as active_processes,
    
    -- Bank accounts
    (SELECT COUNT(*) FROM bank_accounts ba WHERE ba.user_id = u.id AND ba.account_status = 'active') as bank_accounts_count,
    (SELECT COALESCE(SUM(ba.balance), 0) FROM bank_accounts ba WHERE ba.user_id = u.id AND ba.account_status = 'active') as total_bank_balance,
    
    -- Recent activities
    (SELECT COUNT(*) FROM game_action_logs gal WHERE gal.user_id = u.id AND gal.started_at > CURRENT_TIMESTAMP - INTERVAL '24 hours') as actions_24h,
    
    -- Security status
    (SELECT COUNT(*) FROM security_logs sl WHERE sl.user_id = u.id AND sl.resolved = FALSE) as security_alerts,
    
    CURRENT_TIMESTAMP as last_updated
FROM users u
JOIN user_stats us ON u.id = us.user_id
LEFT JOIN hardware h ON h.user_id = u.id AND h.online_status = TRUE
LEFT JOIN clan_members cm ON cm.user_id = u.id AND cm.status = 'active'
LEFT JOIN clans c ON cm.clan_id = c.id;

-- Server status view - Real-time server monitoring
CREATE OR REPLACE VIEW server_status AS
SELECT 
    h.server_id,
    h.user_id,
    u.login as owner_login,
    h.name,
    h.ip_address,
    h.online_status,
    h.cpu,
    h.ram,
    h.current_load,
    h.current_connections,
    h.max_connections,
    h.firewall_level,
    h.last_access,
    
    -- Process information
    (SELECT COUNT(*) FROM processes p WHERE p.p_victim_id = h.user_id AND p.is_completed = FALSE) as active_attacks,
    
    -- Network information
    nn.bandwidth_mbps,
    nn.latency_ms,
    nn.threat_level,
    
    -- Security events
    (SELECT COUNT(*) FROM security_logs sl WHERE sl.target_ip = h.ip_address AND sl.occurred_at > CURRENT_TIMESTAMP - INTERVAL '1 hour') as security_events_1h,
    
    -- Performance metrics
    CASE 
        WHEN h.current_load > 90 THEN 'CRITICAL'
        WHEN h.current_load > 70 THEN 'WARNING'
        ELSE 'OK'
    END as load_status,
    
    CASE 
        WHEN h.current_connections::DECIMAL / h.max_connections > 0.9 THEN 'CRITICAL'
        WHEN h.current_connections::DECIMAL / h.max_connections > 0.7 THEN 'WARNING'
        ELSE 'OK'
    END as connection_status
    
FROM hardware h
JOIN users u ON h.user_id = u.id
LEFT JOIN network_nodes nn ON nn.ip_address = h.ip_address
WHERE h.online_status = TRUE;

-- Mission progress view - Player mission tracking
CREATE OR REPLACE VIEW mission_progress AS
SELECT 
    um.id as user_mission_id,
    um.user_id,
    u.login,
    mt.name as mission_name,
    mt.title as mission_title,
    mt.category,
    mt.difficulty,
    um.status,
    um.progress,
    um.accepted_at,
    um.deadline,
    
    -- Objective progress
    (SELECT COUNT(*) FROM user_mission_objectives umo 
     JOIN mission_objectives mo ON umo.mission_objective_id = mo.id 
     WHERE umo.user_mission_id = um.id AND mo.is_required = TRUE) as total_required_objectives,
    
    (SELECT COUNT(*) FROM user_mission_objectives umo 
     JOIN mission_objectives mo ON umo.mission_objective_id = mo.id 
     WHERE umo.user_mission_id = um.id AND mo.is_required = TRUE AND umo.status = 'completed') as completed_required_objectives,
    
    -- Time remaining
    CASE 
        WHEN um.deadline IS NOT NULL THEN 
            EXTRACT(EPOCH FROM (um.deadline - CURRENT_TIMESTAMP))::INTEGER
        ELSE NULL
    END as seconds_remaining,
    
    -- Rewards
    mt.reward_money,
    mt.reward_experience,
    mt.reward_reputation,
    
    CURRENT_TIMESTAMP as last_updated
FROM user_missions um
JOIN users u ON um.user_id = u.id
JOIN mission_templates mt ON um.mission_template_id = mt.id
WHERE um.status = 'active';

-- Financial overview view - Banking and transaction summary
CREATE OR REPLACE VIEW financial_overview AS
SELECT 
    u.id as user_id,
    u.login,
    
    -- User stats money
    us.money as wallet_balance,
    
    -- Bank accounts summary
    COALESCE(SUM(ba.balance), 0) as total_bank_balance,
    COUNT(ba.id) as total_accounts,
    COUNT(CASE WHEN ba.account_status = 'active' THEN 1 END) as active_accounts,
    
    -- Recent transactions (24h)
    (SELECT COUNT(*) FROM bank_transactions bt 
     WHERE (bt.from_account_id IN (SELECT id FROM bank_accounts WHERE user_id = u.id) OR
            bt.to_account_id IN (SELECT id FROM bank_accounts WHERE user_id = u.id))
     AND bt.created_at > CURRENT_TIMESTAMP - INTERVAL '24 hours') as transactions_24h,
    
    -- Money earned this week
    (SELECT COALESCE(SUM(gal.money_change), 0) FROM game_action_logs gal 
     WHERE gal.user_id = u.id AND gal.money_change > 0 
     AND gal.started_at > CURRENT_TIMESTAMP - INTERVAL '7 days') as money_earned_7d,
    
    -- Money spent this week
    (SELECT COALESCE(SUM(ABS(gal.money_change)), 0) FROM game_action_logs gal 
     WHERE gal.user_id = u.id AND gal.money_change < 0 
     AND gal.started_at > CURRENT_TIMESTAMP - INTERVAL '7 days') as money_spent_7d,
    
    -- Largest account
    (SELECT MAX(ba.balance) FROM bank_accounts ba WHERE ba.user_id = u.id AND ba.account_status = 'active') as largest_account_balance,
    
    CURRENT_TIMESTAMP as last_updated
FROM users u
JOIN user_stats us ON u.id = us.user_id
LEFT JOIN bank_accounts ba ON ba.user_id = u.id
GROUP BY u.id, u.login, us.money;

-- Security alerts view - Active security concerns
CREATE OR REPLACE VIEW security_alerts AS
SELECT 
    sl.id,
    sl.event_type,
    sl.severity,
    sl.description,
    sl.user_id,
    u.login,
    sl.target_user_id,
    tu.login as target_login,
    sl.ip_address,
    sl.target_ip,
    sl.threat_level,
    sl.occurred_at,
    sl.resolved,
    sl.false_positive,
    
    -- Risk assessment
    CASE 
        WHEN sl.severity = 'CRITICAL' AND sl.threat_level >= 8 THEN 'IMMEDIATE'
        WHEN sl.severity = 'HIGH' AND sl.threat_level >= 6 THEN 'HIGH'
        WHEN sl.severity = 'MEDIUM' AND sl.threat_level >= 4 THEN 'MEDIUM'
        ELSE 'LOW'
    END as risk_level,
    
    -- Time since occurrence
    EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - sl.occurred_at))::INTEGER as seconds_since_occurrence
    
FROM security_logs sl
LEFT JOIN users u ON sl.user_id = u.id
LEFT JOIN users tu ON sl.target_user_id = tu.id
WHERE sl.resolved = FALSE AND sl.false_positive = FALSE
ORDER BY sl.threat_level DESC, sl.occurred_at DESC;

-- Create function for comprehensive database maintenance
CREATE OR REPLACE FUNCTION perform_database_maintenance()
RETURNS TABLE(
    task VARCHAR(50),
    records_affected BIGINT,
    execution_time_ms INTEGER
) AS $$
DECLARE
    start_time TIMESTAMPTZ;
    end_time TIMESTAMPTZ;
    affected_records BIGINT;
BEGIN
    -- Clean up expired sessions
    start_time := CURRENT_TIMESTAMP;
    SELECT cleanup_expired_sessions() INTO affected_records;
    end_time := CURRENT_TIMESTAMP;
    
    task := 'cleanup_expired_sessions';
    records_affected := affected_records;
    execution_time_ms := EXTRACT(EPOCH FROM (end_time - start_time) * 1000)::INTEGER;
    RETURN NEXT;
    
    -- Clean up old logs
    start_time := CURRENT_TIMESTAMP;
    SELECT logs_deleted FROM cleanup_old_logs() INTO affected_records;
    end_time := CURRENT_TIMESTAMP;
    
    task := 'cleanup_old_logs';
    records_affected := affected_records;
    execution_time_ms := EXTRACT(EPOCH FROM (end_time - start_time) * 1000)::INTEGER;
    RETURN NEXT;
    
    -- Update clan statistics
    start_time := CURRENT_TIMESTAMP;
    affected_records := 0;
    
    FOR affected_records IN 
        SELECT c.id FROM clans c WHERE c.is_active = TRUE
    LOOP
        PERFORM update_clan_statistics(affected_records);
        affected_records := affected_records + 1;
    END LOOP;
    
    end_time := CURRENT_TIMESTAMP;
    
    task := 'update_clan_statistics';
    records_affected := affected_records;
    execution_time_ms := EXTRACT(EPOCH FROM (end_time - start_time) * 1000)::INTEGER;
    RETURN NEXT;
    
    -- Auto-resume processes
    start_time := CURRENT_TIMESTAMP;
    SELECT auto_resume_processes() INTO affected_records;
    end_time := CURRENT_TIMESTAMP;
    
    task := 'auto_resume_processes';
    records_affected := affected_records;
    execution_time_ms := EXTRACT(EPOCH FROM (end_time - start_time) * 1000)::INTEGER;
    RETURN NEXT;
    
    -- Calculate interest for savings accounts
    start_time := CURRENT_TIMESTAMP;
    affected_records := 0;
    
    FOR affected_records IN 
        SELECT ba.id FROM bank_accounts ba 
        WHERE ba.account_type = 'savings' 
        AND ba.account_status = 'active'
        AND (ba.last_interest_payment IS NULL OR ba.last_interest_payment < CURRENT_DATE - INTERVAL '30 days')
    LOOP
        PERFORM calculate_account_interest(affected_records);
        affected_records := affected_records + 1;
    END LOOP;
    
    end_time := CURRENT_TIMESTAMP;
    
    task := 'calculate_interest';
    records_affected := affected_records;
    execution_time_ms := EXTRACT(EPOCH FROM (end_time - start_time) * 1000)::INTEGER;
    RETURN NEXT;
    
    -- Refresh materialized views
    start_time := CURRENT_TIMESTAMP;
    
    REFRESH MATERIALIZED VIEW CONCURRENTLY hackable_servers;
    REFRESH MATERIALIZED VIEW CONCURRENTLY software_catalog;
    REFRESH MATERIALIZED VIEW log_statistics;
    
    end_time := CURRENT_TIMESTAMP;
    
    task := 'refresh_materialized_views';
    records_affected := 3;
    execution_time_ms := EXTRACT(EPOCH FROM (end_time - start_time) * 1000)::INTEGER;
    RETURN NEXT;
    
    -- Update statistics
    start_time := CURRENT_TIMESTAMP;
    ANALYZE;
    end_time := CURRENT_TIMESTAMP;
    
    task := 'analyze_tables';
    records_affected := 0;
    execution_time_ms := EXTRACT(EPOCH FROM (end_time - start_time) * 1000)::INTEGER;
    RETURN NEXT;
END;
$$ LANGUAGE plpgsql;

-- Create function to get database health metrics
CREATE OR REPLACE FUNCTION get_database_health()
RETURNS TABLE(
    metric_name VARCHAR(50),
    metric_value BIGINT,
    status VARCHAR(20),
    threshold_warning BIGINT,
    threshold_critical BIGINT
) AS $$
BEGIN
    -- Total users
    metric_name := 'total_users';
    SELECT COUNT(*) INTO metric_value FROM users;
    threshold_warning := 50000;
    threshold_critical := 100000;
    status := CASE 
        WHEN metric_value >= threshold_critical THEN 'CRITICAL'
        WHEN metric_value >= threshold_warning THEN 'WARNING'
        ELSE 'OK'
    END;
    RETURN NEXT;
    
    -- Active sessions
    metric_name := 'active_sessions';
    SELECT COUNT(*) INTO metric_value FROM sessions WHERE is_active = TRUE;
    threshold_warning := 1000;
    threshold_critical := 5000;
    status := CASE 
        WHEN metric_value >= threshold_critical THEN 'CRITICAL'
        WHEN metric_value >= threshold_warning THEN 'WARNING'
        ELSE 'OK'
    END;
    RETURN NEXT;
    
    -- Active processes
    metric_name := 'active_processes';
    SELECT COUNT(*) INTO metric_value FROM processes WHERE is_completed = FALSE AND is_failed = FALSE;
    threshold_warning := 10000;
    threshold_critical := 50000;
    status := CASE 
        WHEN metric_value >= threshold_critical THEN 'CRITICAL'
        WHEN metric_value >= threshold_warning THEN 'WARNING'
        ELSE 'OK'
    END;
    RETURN NEXT;
    
    -- Unresolved security events
    metric_name := 'unresolved_security_events';
    SELECT COUNT(*) INTO metric_value FROM security_logs WHERE resolved = FALSE AND false_positive = FALSE;
    threshold_warning := 100;
    threshold_critical := 500;
    status := CASE 
        WHEN metric_value >= threshold_critical THEN 'CRITICAL'
        WHEN metric_value >= threshold_warning THEN 'WARNING'
        ELSE 'OK'
    END;
    RETURN NEXT;
    
    -- Database size (approximate)
    metric_name := 'database_size_mb';
    SELECT pg_database_size(current_database()) / (1024 * 1024) INTO metric_value;
    threshold_warning := 50000; -- 50GB
    threshold_critical := 100000; -- 100GB
    status := CASE 
        WHEN metric_value >= threshold_critical THEN 'CRITICAL'
        WHEN metric_value >= threshold_warning THEN 'WARNING'
        ELSE 'OK'
    END;
    RETURN NEXT;
END;
$$ LANGUAGE plpgsql;

-- Create indexes for the new views (where beneficial)
CREATE INDEX IF NOT EXISTS idx_security_logs_unresolved ON security_logs(resolved, false_positive) WHERE resolved = FALSE AND false_positive = FALSE;
CREATE INDEX IF NOT EXISTS idx_user_missions_active ON user_missions(user_id, status) WHERE status = 'active';
CREATE INDEX IF NOT EXISTS idx_bank_accounts_user_status ON bank_accounts(user_id, account_status) WHERE account_status = 'active';

-- Create function to initialize sample data for testing
CREATE OR REPLACE FUNCTION initialize_sample_data()
RETURNS VOID AS $$
BEGIN
    -- Insert sample users if none exist
    IF NOT EXISTS (SELECT 1 FROM users LIMIT 1) THEN
        INSERT INTO users (login, password, email, game_pass, game_ip, real_ip, home_ip) VALUES
        ('admin', '$2b$12$XYZ123...', 'admin@hackerexperience.com', 'ADM12345', 
         INET_CLIENT_ADDR_TO_BIGINT('192.168.1.100'), INET_CLIENT_ADDR_TO_BIGINT('10.0.1.100'), INET_CLIENT_ADDR_TO_BIGINT('192.168.1.100')),
        ('player1', '$2b$12$ABC123...', 'player1@example.com', 'PLY12345',
         INET_CLIENT_ADDR_TO_BIGINT('192.168.1.101'), INET_CLIENT_ADDR_TO_BIGINT('10.0.1.101'), INET_CLIENT_ADDR_TO_BIGINT('192.168.1.101')),
        ('player2', '$2b$12$DEF123...', 'player2@example.com', 'PLY67890',
         INET_CLIENT_ADDR_TO_BIGINT('192.168.1.102'), INET_CLIENT_ADDR_TO_BIGINT('10.0.1.102'), INET_CLIENT_ADDR_TO_BIGINT('192.168.1.102'));
        
        -- Insert corresponding user stats
        INSERT INTO user_stats (user_id, experience, level, money) VALUES
        (1, 50000, 25, 1000000),
        (2, 5000, 8, 50000),
        (3, 2000, 5, 25000);
    END IF;
    
    -- Insert sample banks if none exist
    IF NOT EXISTS (SELECT 1 FROM banks LIMIT 1) THEN
        INSERT INTO banks (name, short_name, country_code, city) VALUES
        ('First National Bank', 'FNB', 'US', 'New York'),
        ('International Trust Bank', 'ITB', 'CH', 'Zurich'),
        ('Crypto Exchange Bank', 'CEB', 'JP', 'Tokyo');
    END IF;
    
    -- Insert sample mission templates if none exist
    IF NOT EXISTS (SELECT 1 FROM mission_templates LIMIT 1) THEN
        INSERT INTO mission_templates (name, internal_name, category, title, description, objective_summary, difficulty, reward_money, reward_experience) VALUES
        ('First Hack', 'tutorial_first_hack', 'tutorial', 'Your First Hack', 'Learn the basics of hacking by compromising a simple server.', 'Hack into the tutorial server', 1, 1000, 100),
        ('Data Theft', 'corp_data_theft', 'story', 'Corporate Espionage', 'Steal sensitive data from MegaCorp servers.', 'Download classified files', 5, 25000, 1000),
        ('Bank Heist', 'bank_infiltration', 'story', 'Digital Bank Robbery', 'Infiltrate the bank''s security systems.', 'Transfer funds without detection', 8, 100000, 5000);
        
        -- Insert objectives for missions
        INSERT INTO mission_objectives (mission_template_id, objective_key, name, description, objective_type, quantity_required) VALUES
        (1, 'connect_to_server', 'Connect to Server', 'Establish connection to target server', 'connect', 1),
        (1, 'bypass_firewall', 'Bypass Firewall', 'Get past the server firewall', 'hack_firewall', 1),
        (1, 'gain_access', 'Gain Access', 'Successfully access the server', 'gain_access', 1);
    END IF;
    
    RAISE NOTICE 'Sample data initialized successfully';
END;
$$ LANGUAGE plpgsql;

-- Helper function to convert IP string to bigint (for sample data)
CREATE OR REPLACE FUNCTION INET_CLIENT_ADDR_TO_BIGINT(ip_str TEXT)
RETURNS BIGINT AS $$
DECLARE
    parts INTEGER[];
    result BIGINT;
BEGIN
    parts := string_to_array(ip_str, '.')::INTEGER[];
    result := (parts[1]::BIGINT << 24) + (parts[2]::BIGINT << 16) + (parts[3]::BIGINT << 8) + parts[4]::BIGINT;
    RETURN result;
END;
$$ LANGUAGE plpgsql;

-- Final performance optimization: Update table statistics
ANALYZE;

-- Create final comprehensive comments
COMMENT ON VIEW admin_dashboard IS 'Comprehensive dashboard view for system administrators';
COMMENT ON VIEW player_dashboard IS 'Player-specific dashboard with key game information';
COMMENT ON VIEW server_status IS 'Real-time server monitoring and status information';
COMMENT ON VIEW mission_progress IS 'Active mission tracking for players';
COMMENT ON VIEW financial_overview IS 'Financial summary for players including bank accounts and transactions';
COMMENT ON VIEW security_alerts IS 'Active security concerns requiring attention';

COMMENT ON FUNCTION perform_database_maintenance IS 'Comprehensive database maintenance routine for scheduled execution';
COMMENT ON FUNCTION get_database_health IS 'Returns key database health metrics with status indicators';
COMMENT ON FUNCTION initialize_sample_data IS 'Initializes sample data for testing and development';

-- Final success message
DO $$
BEGIN
    RAISE NOTICE '===========================================';
    RAISE NOTICE 'HackerExperience Database Schema Complete!';
    RAISE NOTICE '===========================================';
    RAISE NOTICE 'Total tables created: 30+';
    RAISE NOTICE 'Total functions created: 25+';
    RAISE NOTICE 'Total views created: 15+';
    RAISE NOTICE 'Total indexes created: 100+';
    RAISE NOTICE 'Database is ready for production use!';
    RAISE NOTICE '===========================================';
END $$;