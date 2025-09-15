-- HackerExperience Sample Data and Advanced Procedures
-- Creates comprehensive sample data and advanced stored procedures for testing and development

-- Advanced stored procedures for game operations
CREATE OR REPLACE FUNCTION initialize_sample_data() RETURNS TEXT AS $$
DECLARE
    result_text TEXT := '';
BEGIN
    result_text := result_text || 'Initializing HackerExperience sample data...' || E'\n';
    
    -- Insert sample users
    INSERT INTO users (login, email, password_hash, ip, is_confirmed, is_premium) VALUES
    ('admin', 'admin@hackerexperience.com', '$2b$12$LQv3c1yqBwLFoGdkVfLuR.vOwMKv8XkBm9pZKj6wXkKaHd.vOwMKv', '127.0.0.1', TRUE, TRUE),
    ('player1', 'player1@example.com', '$2b$12$LQv3c1yqBwLFoGdkVfLuR.vOwMKv8XkBm9pZKj6wXkKaHd.vOwMKv', '192.168.1.100', TRUE, FALSE),
    ('player2', 'player2@example.com', '$2b$12$LQv3c1yqBwLFoGdkVfLuR.vOwMKv8XkBm9pZKj6wXkKaHd.vOwMKv', '192.168.1.101', TRUE, FALSE),
    ('hacker_elite', 'elite@example.com', '$2b$12$LQv3c1yqBwLFoGdkVfLuR.vOwMKv8XkBm9pZKj6wXkKaHd.vOwMKv', '10.0.0.50', TRUE, TRUE),
    ('newbie_user', 'newbie@example.com', '$2b$12$LQv3c1yqBwLFoGdkVfLuR.vOwMKv8XkBm9pZKj6wXkKaHd.vOwMKv', '203.0.113.42', TRUE, FALSE)
    ON CONFLICT (login) DO NOTHING;
    
    result_text := result_text || 'Created sample users...' || E'\n';
    
    -- Insert user stats
    INSERT INTO user_stats (user_id, experience, money, level) VALUES
    (1, 50000, 1000000, 25),
    (2, 10000, 50000, 12),
    (3, 15000, 75000, 15),
    (4, 100000, 500000, 45),
    (5, 1000, 5000, 3)
    ON CONFLICT (user_id) DO NOTHING;
    
    result_text := result_text || 'Created user statistics...' || E'\n';
    
    -- Insert sample hardware
    INSERT INTO hardware (user_id, hardware_type, specs, performance_rating, security_level, connection_count) VALUES
    (1, 'server', '{"cpu": 3200, "ram": 32768, "hdd": 2048000, "net": 1000}', 95, 8, 15),
    (2, 'desktop', '{"cpu": 2400, "ram": 8192, "hdd": 512000, "net": 100}', 60, 4, 3),
    (3, 'laptop', '{"cpu": 2000, "ram": 16384, "hdd": 1024000, "net": 100}', 70, 5, 5),
    (4, 'supercomputer', '{"cpu": 4800, "ram": 65536, "hdd": 4096000, "net": 10000}', 98, 9, 25),
    (5, 'old_desktop', '{"cpu": 1600, "ram": 4096, "hdd": 256000, "net": 10}', 35, 2, 1)
    ON CONFLICT DO NOTHING;
    
    result_text := result_text || 'Created hardware configurations...' || E'\n';
    
    -- Insert external hardware (NPC targets)
    INSERT INTO external_hardware (ip_address, hardware_type, difficulty_level, security_rating, hackable, reward_money) VALUES
    ('203.0.113.10', 'bank_server', 8, 85, TRUE, 50000),
    ('198.51.100.25', 'corporate_server', 6, 70, TRUE, 25000),
    ('192.0.2.100', 'government_server', 9, 95, TRUE, 100000),
    ('203.0.113.50', 'university_server', 4, 60, TRUE, 15000),
    ('198.51.100.75', 'startup_server', 3, 45, TRUE, 8000),
    ('192.0.2.200', 'research_lab', 7, 80, TRUE, 35000)
    ON CONFLICT (ip_address) DO NOTHING;
    
    result_text := result_text || 'Created external targets...' || E'\n';
    
    -- Insert sample software
    INSERT INTO software (user_id, software_type, software_version, installation_status, usage_count, dependencies) VALUES
    (1, 'password_cracker', 'v3.2.1', 'installed', 25, '["log_cleaner"]'),
    (1, 'log_cleaner', 'v2.1.0', 'installed', 30, '[]'),
    (1, 'firewall', 'v4.5.2', 'installed', 0, '[]'),
    (2, 'password_cracker', 'v2.8.1', 'installed', 5, '[]'),
    (2, 'text_editor', 'v1.0.0', 'installed', 15, '[]'),
    (3, 'firewall', 'v3.2.0', 'installed', 0, '[]'),
    (4, 'advanced_scanner', 'v5.0.1', 'installed', 50, '["password_cracker", "log_cleaner"]'),
    (4, 'exploit_framework', 'v2.3.4', 'installed', 40, '["advanced_scanner"]')
    ON CONFLICT DO NOTHING;
    
    result_text := result_text || 'Created software installations...' || E'\n';
    
    -- Insert sample processes
    INSERT INTO processes (p_creator_id, p_software_id, p_source_ip, p_target_ip, p_time_start, p_time_end, p_data, process_type, progress) VALUES
    (2, 1, '192.168.1.100', '203.0.113.10', CURRENT_TIMESTAMP - INTERVAL '30 minutes', CURRENT_TIMESTAMP + INTERVAL '15 minutes', '{"target_file": "passwords.txt", "method": "dictionary"}', 'password_crack', 65.5),
    (3, 2, '192.168.1.101', '198.51.100.25', CURRENT_TIMESTAMP - INTERVAL '1 hour', CURRENT_TIMESTAMP + INTERVAL '20 minutes', '{"scan_type": "port_scan", "ports": [22, 80, 443]}', 'network_scan', 45.2),
    (4, 3, '10.0.0.50', '192.0.2.100', CURRENT_TIMESTAMP - INTERVAL '2 hours', CURRENT_TIMESTAMP + INTERVAL '45 minutes', '{"firewall_rules": ["block_suspicious"], "threat_level": "high"}', 'security_scan', 80.1)
    ON CONFLICT DO NOTHING;
    
    result_text := result_text || 'Created active processes...' || E'\n';
    
    -- Insert sample clans
    INSERT INTO clans (name, description, leader_id, member_count, total_reputation, clan_type) VALUES
    ('Elite Hackers', 'The most skilled hackers in the game', 4, 1, 5000, 'hacking'),
    ('Digital Defenders', 'Focused on security and defense', 1, 1, 3500, 'security'),
    ('Code Warriors', 'Programming and development specialists', 2, 1, 2800, 'development')
    ON CONFLICT (name) DO NOTHING;
    
    -- Insert clan members
    INSERT INTO clan_members (clan_id, user_id, role, joined_at, contribution_points) VALUES
    (1, 4, 'leader', CURRENT_TIMESTAMP - INTERVAL '6 months', 2500),
    (2, 1, 'leader', CURRENT_TIMESTAMP - INTERVAL '1 year', 3000),
    (3, 2, 'leader', CURRENT_TIMESTAMP - INTERVAL '8 months', 1800),
    (1, 3, 'member', CURRENT_TIMESTAMP - INTERVAL '3 months', 800)
    ON CONFLICT DO NOTHING;
    
    result_text := result_text || 'Created clans and memberships...' || E'\n';
    
    -- Insert sample missions
    INSERT INTO missions (mission_key, title, description, mission_type, difficulty_level, reward_money, reward_experience, requirements, objectives) VALUES
    ('tutorial_hack', 'First Hack Tutorial', 'Learn the basics of hacking by targeting a simple server', 'tutorial', 1, 5000, 1000, '{"min_level": 1}', '[{"type": "hack_server", "target": "203.0.113.50", "description": "Hack the university server"}]'),
    ('bank_heist', 'Digital Bank Heist', 'Infiltrate the central banking system and extract funds', 'main_quest', 8, 100000, 25000, '{"min_level": 20, "required_software": ["password_cracker", "log_cleaner"]}', '[{"type": "hack_server", "target": "203.0.113.10"}, {"type": "steal_money", "amount": 50000}]'),
    ('defense_contract', 'Corporate Defense Contract', 'Set up security systems for a major corporation', 'side_quest', 5, 35000, 8000, '{"min_level": 10, "required_software": ["firewall"]}', '[{"type": "install_firewall", "target": "198.51.100.25"}, {"type": "monitor_threats", "duration": 86400}]')
    ON CONFLICT (mission_key) DO NOTHING;
    
    result_text := result_text || 'Created sample missions...' || E'\n';
    
    -- Insert user mission progress
    INSERT INTO user_missions (user_id, mission_id, mission_status, progress_data) VALUES
    (5, 1, 'active', '{"current_objective": 0, "started_at": "' || CURRENT_TIMESTAMP || '"}'),
    (2, 1, 'completed', '{"completed_at": "' || (CURRENT_TIMESTAMP - INTERVAL '1 day') || '", "objectives_completed": 1}'),
    (4, 2, 'active', '{"current_objective": 1, "started_at": "' || (CURRENT_TIMESTAMP - INTERVAL '3 hours') || '"}')
    ON CONFLICT DO NOTHING;
    
    result_text := result_text || 'Created mission progress...' || E'\n';
    
    -- Insert sample bank accounts
    INSERT INTO bank_accounts (user_id, bank_name, account_number, balance, account_type, account_status) VALUES
    (1, 'HE Central Bank', 'ACC001001', 500000, 'premium', 'active'),
    (2, 'Digital Trust Bank', 'ACC002001', 25000, 'standard', 'active'),
    (3, 'Crypto Finance', 'ACC003001', 40000, 'standard', 'active'),
    (4, 'Elite Banking', 'ACC004001', 750000, 'premium', 'active'),
    (5, 'Starter Bank', 'ACC005001', 2500, 'basic', 'active')
    ON CONFLICT DO NOTHING;
    
    result_text := result_text || 'Created bank accounts...' || E'\n';
    
    -- Insert sample sessions
    INSERT INTO sessions (user_id, session_token, ip_address, user_agent, last_activity) VALUES
    (1, encode(gen_random_bytes(32), 'hex'), '127.0.0.1', 'HE-Client/1.0', CURRENT_TIMESTAMP),
    (2, encode(gen_random_bytes(32), 'hex'), '192.168.1.100', 'Mozilla/5.0', CURRENT_TIMESTAMP - INTERVAL '5 minutes'),
    (4, encode(gen_random_bytes(32), 'hex'), '10.0.0.50', 'HE-Elite-Client/2.0', CURRENT_TIMESTAMP - INTERVAL '2 minutes')
    ON CONFLICT DO NOTHING;
    
    result_text := result_text || 'Created active sessions...' || E'\n';
    
    -- Insert sample reputations
    INSERT INTO reputations (user_id, reputation_type, reputation_value) VALUES
    (1, 'hacking', 8500),
    (1, 'defensive', 9200),
    (2, 'hacking', 3200),
    (3, 'hacking', 4100),
    (4, 'hacking', 15000),
    (4, 'financial', 12000),
    (5, 'hacking', 150)
    ON CONFLICT DO NOTHING;
    
    result_text := result_text || 'Created reputation data...' || E'\n';
    
    -- Insert network connections
    INSERT INTO network_connections (source_ip, target_ip, connection_type, connection_status, bandwidth_usage, security_level) VALUES
    ('192.168.1.100', '203.0.113.10', 'hack_attempt', 'active', 250, 6),
    ('192.168.1.101', '198.51.100.25', 'reconnaissance', 'completed', 50, 4),
    ('10.0.0.50', '192.0.2.100', 'advanced_hack', 'active', 500, 9)
    ON CONFLICT DO NOTHING;
    
    result_text := result_text || 'Created network connections...' || E'\n';
    
    result_text := result_text || 'Sample data initialization completed successfully!' || E'\n';
    return result_text;
END;
$$ LANGUAGE plpgsql;

-- Function to simulate realistic game progression
CREATE OR REPLACE FUNCTION simulate_game_activity(
    p_duration_hours INTEGER DEFAULT 24
) RETURNS TEXT AS $$
DECLARE
    result_text TEXT := '';
    user_record RECORD;
    random_target TEXT;
    process_duration INTEGER;
BEGIN
    result_text := result_text || 'Simulating ' || p_duration_hours || ' hours of game activity...' || E'\n';
    
    -- Simulate hacking attempts for active users
    FOR user_record IN SELECT id, login FROM users WHERE id BETWEEN 2 AND 5 LOOP
        -- Random hacking activity
        IF random() > 0.3 THEN
            SELECT ip_address INTO random_target FROM external_hardware ORDER BY random() LIMIT 1;
            process_duration := 600 + (random() * 3000)::INTEGER; -- 10 minutes to 1 hour
            
            INSERT INTO processes (p_creator_id, p_source_ip, p_target_ip, p_time_start, p_time_end, process_type, progress, p_data)
            VALUES (
                user_record.id,
                '192.168.1.' || (100 + user_record.id),
                random_target,
                CURRENT_TIMESTAMP - INTERVAL '1 hour' * random() * p_duration_hours,
                CURRENT_TIMESTAMP + INTERVAL '1 second' * process_duration,
                'hack_attempt',
                random() * 100,
                '{"simulation": true, "auto_generated": true}'
            );
            
            result_text := result_text || 'User ' || user_record.login || ' started hacking ' || random_target || E'\n';
        END IF;
        
        -- Random experience and money gains
        UPDATE user_stats SET 
            experience = experience + (random() * 1000)::INTEGER,
            money = money + (random() * 5000)::INTEGER
        WHERE user_id = user_record.id;
    END LOOP;
    
    -- Simulate market price fluctuations
    PERFORM update_market_prices();
    
    -- Update some processes to completed status
    UPDATE processes SET 
        is_completed = TRUE,
        completed_at = CURRENT_TIMESTAMP - INTERVAL '1 hour' * random(),
        progress = 100.0
    WHERE is_completed = FALSE AND random() > 0.7;
    
    result_text := result_text || 'Updated market prices and completed some processes...' || E'\n';
    
    -- Add some random log entries
    INSERT INTO log_hacking (user_id, target_ip, action_type, success, details, created_at)
    SELECT 
        (2 + (random() * 3)::INTEGER),
        ip_address,
        CASE WHEN random() > 0.5 THEN 'password_crack' ELSE 'port_scan' END,
        random() > 0.3,
        '{"simulation": true, "method": "automated"}',
        CURRENT_TIMESTAMP - INTERVAL '1 hour' * random() * p_duration_hours
    FROM external_hardware 
    ORDER BY random() 
    LIMIT 10;
    
    result_text := result_text || 'Generated simulation log entries...' || E'\n';
    result_text := result_text || 'Game activity simulation completed!' || E'\n';
    
    RETURN result_text;
END;
$$ LANGUAGE plpgsql;

-- Function to get comprehensive user dashboard data
CREATE OR REPLACE FUNCTION get_user_dashboard(p_user_id INTEGER) RETURNS JSONB AS $$
DECLARE
    dashboard_data JSONB := '{}';
    user_info RECORD;
    stats_info RECORD;
    active_processes INTEGER;
    completed_missions INTEGER;
    clan_info RECORD;
    reputation_data JSONB;
BEGIN
    -- Get user basic info
    SELECT u.id, u.login, u.email, u.is_premium, u.created_at
    INTO user_info
    FROM users u WHERE u.id = p_user_id;
    
    IF NOT FOUND THEN
        RETURN '{"error": "User not found"}';
    END IF;
    
    -- Get user stats
    SELECT level, experience, money, total_logins, last_login
    INTO stats_info
    FROM user_stats WHERE user_id = p_user_id;
    
    -- Count active processes
    SELECT COUNT(*) INTO active_processes
    FROM processes WHERE p_creator_id = p_user_id AND is_completed = FALSE;
    
    -- Count completed missions
    SELECT COUNT(*) INTO completed_missions
    FROM user_missions WHERE user_id = p_user_id AND mission_status = 'completed';
    
    -- Get clan information
    SELECT c.name, c.clan_type, cm.role, cm.contribution_points
    INTO clan_info
    FROM clan_members cm
    JOIN clans c ON cm.clan_id = c.id
    WHERE cm.user_id = p_user_id AND cm.status = 'active';
    
    -- Get reputation data
    SELECT jsonb_object_agg(reputation_type, reputation_value) INTO reputation_data
    FROM reputations WHERE user_id = p_user_id;
    
    -- Build dashboard JSON
    dashboard_data := jsonb_build_object(
        'user', jsonb_build_object(
            'id', user_info.id,
            'login', user_info.login,
            'email', user_info.email,
            'is_premium', user_info.is_premium,
            'member_since', user_info.created_at
        ),
        'stats', jsonb_build_object(
            'level', COALESCE(stats_info.level, 1),
            'experience', COALESCE(stats_info.experience, 0),
            'money', COALESCE(stats_info.money, 0),
            'total_logins', COALESCE(stats_info.total_logins, 0),
            'last_login', stats_info.last_login
        ),
        'activity', jsonb_build_object(
            'active_processes', active_processes,
            'completed_missions', completed_missions
        ),
        'clan', CASE 
            WHEN clan_info.name IS NOT NULL THEN jsonb_build_object(
                'name', clan_info.name,
                'type', clan_info.clan_type,
                'role', clan_info.role,
                'contribution_points', clan_info.contribution_points
            )
            ELSE NULL
        END,
        'reputation', COALESCE(reputation_data, '{}'::JSONB),
        'generated_at', CURRENT_TIMESTAMP
    );
    
    RETURN dashboard_data;
END;
$$ LANGUAGE plpgsql;

-- Function to get system statistics for admin dashboard
CREATE OR REPLACE FUNCTION get_system_statistics() RETURNS JSONB AS $$
DECLARE
    stats JSONB := '{}';
BEGIN
    WITH user_stats AS (
        SELECT 
            COUNT(*) as total_users,
            COUNT(*) FILTER (WHERE is_confirmed = TRUE) as confirmed_users,
            COUNT(*) FILTER (WHERE is_premium = TRUE) as premium_users,
            COUNT(*) FILTER (WHERE created_at > CURRENT_TIMESTAMP - INTERVAL '7 days') as new_users_week
        FROM users
    ),
    process_stats AS (
        SELECT 
            COUNT(*) as total_processes,
            COUNT(*) FILTER (WHERE is_completed = FALSE) as active_processes,
            COUNT(*) FILTER (WHERE is_completed = TRUE) as completed_processes,
            AVG(EXTRACT(EPOCH FROM (completed_at - initiated_at))) as avg_completion_time
        FROM processes
    ),
    financial_stats AS (
        SELECT 
            SUM(money) as total_money_in_game,
            AVG(money) as avg_user_money,
            SUM(balance) as total_bank_balance
        FROM user_stats us
        LEFT JOIN bank_accounts ba ON us.user_id = ba.user_id
    )
    SELECT jsonb_build_object(
        'users', jsonb_build_object(
            'total', us.total_users,
            'confirmed', us.confirmed_users,
            'premium', us.premium_users,
            'new_this_week', us.new_users_week
        ),
        'processes', jsonb_build_object(
            'total', ps.total_processes,
            'active', ps.active_processes,
            'completed', ps.completed_processes,
            'avg_completion_time_seconds', COALESCE(ps.avg_completion_time, 0)
        ),
        'economy', jsonb_build_object(
            'total_money_in_circulation', COALESCE(fs.total_money_in_game, 0),
            'average_user_wealth', COALESCE(fs.avg_user_money, 0),
            'total_bank_deposits', COALESCE(fs.total_bank_balance, 0)
        ),
        'generated_at', CURRENT_TIMESTAMP
    ) INTO stats
    FROM user_stats us
    CROSS JOIN process_stats ps
    CROSS JOIN financial_stats fs;
    
    RETURN stats;
END;
$$ LANGUAGE plpgsql;

-- Function to perform maintenance and cleanup
CREATE OR REPLACE FUNCTION perform_maintenance_cleanup() RETURNS TEXT AS $$
DECLARE
    result_text TEXT := '';
    deleted_count INTEGER;
BEGIN
    result_text := result_text || 'Starting maintenance cleanup...' || E'\n';
    
    -- Clean up old completed processes (older than 30 days)
    DELETE FROM processes 
    WHERE is_completed = TRUE 
    AND completed_at < CURRENT_TIMESTAMP - INTERVAL '30 days';
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    result_text := result_text || 'Deleted ' || deleted_count || ' old processes' || E'\n';
    
    -- Clean up expired sessions (older than 7 days)
    DELETE FROM sessions 
    WHERE last_activity < CURRENT_TIMESTAMP - INTERVAL '7 days';
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    result_text := result_text || 'Deleted ' || deleted_count || ' expired sessions' || E'\n';
    
    -- Clean up old log entries (keep last 90 days)
    DELETE FROM log_hacking 
    WHERE created_at < CURRENT_TIMESTAMP - INTERVAL '90 days';
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    result_text := result_text || 'Deleted ' || deleted_count || ' old hacking logs' || E'\n';
    
    -- Update statistics
    ANALYZE;
    result_text := result_text || 'Updated table statistics' || E'\n';
    
    -- Vacuum important tables
    VACUUM ANALYZE users, processes, sessions;
    result_text := result_text || 'Vacuumed and analyzed core tables' || E'\n';
    
    result_text := result_text || 'Maintenance cleanup completed!' || E'\n';
    RETURN result_text;
END;
$$ LANGUAGE plpgsql;

-- Function to simulate clan warfare
CREATE OR REPLACE FUNCTION simulate_clan_warfare(
    p_clan_id_1 INTEGER,
    p_clan_id_2 INTEGER,
    p_duration_hours INTEGER DEFAULT 24
) RETURNS TEXT AS $$
DECLARE
    result_text TEXT := '';
    clan1_name VARCHAR(100);
    clan2_name VARCHAR(100);
    war_id INTEGER;
BEGIN
    -- Get clan names
    SELECT name INTO clan1_name FROM clans WHERE id = p_clan_id_1;
    SELECT name INTO clan2_name FROM clans WHERE id = p_clan_id_2;
    
    result_text := result_text || 'Starting clan war between ' || clan1_name || ' and ' || clan2_name || E'\n';
    
    -- Insert clan war activity
    INSERT INTO clan_activities (clan_id, activity_type, description, points_earned, created_at)
    VALUES 
    (p_clan_id_1, 'warfare', 'Initiated war against ' || clan2_name, 100, CURRENT_TIMESTAMP),
    (p_clan_id_2, 'warfare', 'War declared by ' || clan1_name, 50, CURRENT_TIMESTAMP);
    
    -- Simulate battle activities over time
    INSERT INTO clan_activities (clan_id, activity_type, description, points_earned, created_at)
    SELECT 
        CASE WHEN random() > 0.5 THEN p_clan_id_1 ELSE p_clan_id_2 END,
        'battle',
        'Warfare engagement #' || generate_series,
        (random() * 200)::INTEGER,
        CURRENT_TIMESTAMP - INTERVAL '1 hour' * random() * p_duration_hours
    FROM generate_series(1, 10);
    
    result_text := result_text || 'Generated warfare activities for ' || p_duration_hours || ' hours' || E'\n';
    
    -- Update clan reputation
    UPDATE clans SET total_reputation = total_reputation + (random() * 500)::INTEGER
    WHERE id IN (p_clan_id_1, p_clan_id_2);
    
    result_text := result_text || 'Updated clan reputations' || E'\n';
    
    RETURN result_text;
END;
$$ LANGUAGE plpgsql;

-- Scheduled job simulation (would be handled by external cron in production)
CREATE OR REPLACE FUNCTION run_scheduled_maintenance() RETURNS TEXT AS $$
DECLARE
    result_text TEXT := '';
BEGIN
    result_text := result_text || 'Running scheduled maintenance tasks...' || E'\n';
    
    -- Update market prices
    PERFORM update_market_prices();
    result_text := result_text || 'Updated market prices' || E'\n';
    
    -- Process automatic mission rewards
    UPDATE user_stats SET 
        experience = experience + 100,
        money = money + 1000
    WHERE user_id IN (
        SELECT DISTINCT um.user_id 
        FROM user_missions um 
        JOIN missions m ON um.mission_id = m.id 
        WHERE um.mission_status = 'completed' 
        AND um.updated_at > CURRENT_TIMESTAMP - INTERVAL '1 hour'
    );
    
    result_text := result_text || 'Processed mission rewards' || E'\n';
    
    -- Clean up old data
    result_text := result_text || perform_maintenance_cleanup();
    
    -- Update user levels based on experience
    UPDATE user_stats SET 
        level = LEAST(100, 1 + (experience / 2000)::INTEGER)
    WHERE level < 1 + (experience / 2000)::INTEGER;
    
    result_text := result_text || 'Updated user levels' || E'\n';
    
    result_text := result_text || 'Scheduled maintenance completed successfully!' || E'\n';
    RETURN result_text;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION initialize_sample_data() IS 'Creates comprehensive sample data for testing and development';
COMMENT ON FUNCTION simulate_game_activity(INTEGER) IS 'Simulates realistic game activity over a specified time period';
COMMENT ON FUNCTION get_user_dashboard(INTEGER) IS 'Returns comprehensive dashboard data for a specific user';
COMMENT ON FUNCTION get_system_statistics() IS 'Returns system-wide statistics for admin dashboard';
COMMENT ON FUNCTION perform_maintenance_cleanup() IS 'Performs database cleanup and maintenance tasks';
COMMENT ON FUNCTION simulate_clan_warfare(INTEGER, INTEGER, INTEGER) IS 'Simulates clan warfare activities between two clans';
COMMENT ON FUNCTION run_scheduled_maintenance() IS 'Runs all scheduled maintenance tasks (market updates, rewards, cleanup)';