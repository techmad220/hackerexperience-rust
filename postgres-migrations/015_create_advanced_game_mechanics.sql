-- HackerExperience Advanced Game Mechanics
-- Creates tables and functions for complex game systems including reputation, achievements, and dynamic content

-- Reputation system for players and clans
CREATE TABLE IF NOT EXISTS reputations (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE,
    clan_id INTEGER,
    reputation_type VARCHAR(50) NOT NULL CHECK (reputation_type IN ('hacking', 'defensive', 'financial', 'social', 'military')),
    reputation_value INTEGER NOT NULL DEFAULT 0,
    reputation_level VARCHAR(20) GENERATED ALWAYS AS (
        CASE 
            WHEN reputation_value >= 10000 THEN 'legendary'
            WHEN reputation_value >= 5000 THEN 'master'
            WHEN reputation_value >= 2500 THEN 'expert'
            WHEN reputation_value >= 1000 THEN 'advanced'
            WHEN reputation_value >= 500 THEN 'intermediate'
            WHEN reputation_value >= 100 THEN 'novice'
            ELSE 'beginner'
        END
    ) STORED,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, reputation_type),
    CONSTRAINT chk_reputation_bounds CHECK (reputation_value >= 0 AND reputation_value <= 100000)
);

-- Achievement system
CREATE TABLE IF NOT EXISTS achievements (
    id SERIAL PRIMARY KEY,
    achievement_key VARCHAR(100) NOT NULL UNIQUE,
    title VARCHAR(200) NOT NULL,
    description TEXT NOT NULL,
    achievement_type VARCHAR(50) NOT NULL CHECK (achievement_type IN ('milestone', 'rare', 'hidden', 'seasonal', 'competitive')),
    difficulty_level INTEGER NOT NULL CHECK (difficulty_level BETWEEN 1 AND 10),
    reward_money INTEGER DEFAULT 0,
    reward_experience INTEGER DEFAULT 0,
    reward_items JSONB DEFAULT '[]',
    unlock_conditions JSONB NOT NULL,
    is_repeatable BOOLEAN DEFAULT FALSE,
    is_hidden BOOLEAN DEFAULT FALSE,
    icon_url VARCHAR(500),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- User achievements tracking
CREATE TABLE IF NOT EXISTS user_achievements (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    achievement_id INTEGER NOT NULL REFERENCES achievements(id) ON DELETE CASCADE,
    unlocked_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    progress_data JSONB DEFAULT '{}',
    completion_count INTEGER DEFAULT 1,
    UNIQUE(user_id, achievement_id)
);

-- Dynamic events system for world events
CREATE TABLE IF NOT EXISTS world_events (
    id SERIAL PRIMARY KEY,
    event_key VARCHAR(100) NOT NULL UNIQUE,
    event_name VARCHAR(200) NOT NULL,
    event_description TEXT NOT NULL,
    event_type VARCHAR(50) NOT NULL CHECK (event_type IN ('global', 'regional', 'clan_war', 'market_crash', 'security_breach')),
    severity_level INTEGER NOT NULL CHECK (severity_level BETWEEN 1 AND 5),
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ,
    affected_systems TEXT[] DEFAULT '{}',
    event_effects JSONB NOT NULL DEFAULT '{}',
    participation_requirements JSONB DEFAULT '{}',
    rewards JSONB DEFAULT '{}',
    is_active BOOLEAN GENERATED ALWAYS AS (
        CURRENT_TIMESTAMP BETWEEN start_time AND COALESCE(end_time, start_time + INTERVAL '7 days')
    ) STORED,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Player participation in world events
CREATE TABLE IF NOT EXISTS event_participants (
    id SERIAL PRIMARY KEY,
    event_id INTEGER NOT NULL REFERENCES world_events(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    participation_type VARCHAR(50) NOT NULL,
    contribution_score INTEGER DEFAULT 0,
    rewards_claimed BOOLEAN DEFAULT FALSE,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_activity TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(event_id, user_id)
);

-- Dynamic difficulty scaling
CREATE TABLE IF NOT EXISTS difficulty_modifiers (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    modifier_type VARCHAR(50) NOT NULL CHECK (modifier_type IN ('hacking', 'defense', 'financial', 'research')),
    base_difficulty DECIMAL(4,2) NOT NULL DEFAULT 1.00,
    skill_modifier DECIMAL(4,2) NOT NULL DEFAULT 1.00,
    reputation_modifier DECIMAL(4,2) NOT NULL DEFAULT 1.00,
    equipment_modifier DECIMAL(4,2) NOT NULL DEFAULT 1.00,
    final_difficulty DECIMAL(4,2) GENERATED ALWAYS AS (
        base_difficulty * skill_modifier * reputation_modifier * equipment_modifier
    ) STORED,
    last_calculated TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, modifier_type)
);

-- Market dynamics and pricing
CREATE TABLE IF NOT EXISTS market_data (
    id SERIAL PRIMARY KEY,
    item_type VARCHAR(100) NOT NULL,
    item_subtype VARCHAR(100),
    base_price INTEGER NOT NULL,
    current_price INTEGER NOT NULL,
    supply_level INTEGER NOT NULL DEFAULT 100,
    demand_level INTEGER NOT NULL DEFAULT 100,
    price_trend DECIMAL(5,2) DEFAULT 0.00,
    volatility_factor DECIMAL(3,2) DEFAULT 1.00,
    last_trade_volume INTEGER DEFAULT 0,
    price_history JSONB DEFAULT '[]',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(item_type, item_subtype)
);

-- Research trees and technology progression
CREATE TABLE IF NOT EXISTS research_trees (
    id SERIAL PRIMARY KEY,
    tree_name VARCHAR(100) NOT NULL UNIQUE,
    tree_category VARCHAR(50) NOT NULL CHECK (tree_category IN ('hacking', 'defense', 'hardware', 'software', 'networking')),
    description TEXT,
    unlock_requirements JSONB DEFAULT '{}',
    is_premium BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS research_nodes (
    id SERIAL PRIMARY KEY,
    tree_id INTEGER NOT NULL REFERENCES research_trees(id) ON DELETE CASCADE,
    node_key VARCHAR(100) NOT NULL,
    node_name VARCHAR(200) NOT NULL,
    description TEXT,
    research_cost INTEGER NOT NULL,
    research_time INTEGER NOT NULL, -- in seconds
    prerequisites INTEGER[] DEFAULT '{}', -- references to other node IDs
    unlocks_features TEXT[] DEFAULT '{}',
    tier_level INTEGER NOT NULL CHECK (tier_level BETWEEN 1 AND 10),
    position_x INTEGER DEFAULT 0,
    position_y INTEGER DEFAULT 0,
    UNIQUE(tree_id, node_key)
);

-- User research progress
CREATE TABLE IF NOT EXISTS user_research (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    node_id INTEGER NOT NULL REFERENCES research_nodes(id) ON DELETE CASCADE,
    research_status VARCHAR(20) NOT NULL DEFAULT 'locked' CHECK (research_status IN ('locked', 'available', 'researching', 'completed')),
    progress_percentage DECIMAL(5,2) DEFAULT 0.00,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    investment_amount INTEGER DEFAULT 0,
    UNIQUE(user_id, node_id),
    CONSTRAINT chk_research_progress CHECK (progress_percentage >= 0.00 AND progress_percentage <= 100.00)
);

-- Advanced indexing for performance
CREATE INDEX IF NOT EXISTS idx_reputations_user_type ON reputations(user_id, reputation_type);
CREATE INDEX IF NOT EXISTS idx_reputations_value ON reputations(reputation_value DESC);
CREATE INDEX IF NOT EXISTS idx_user_achievements_user ON user_achievements(user_id);
CREATE INDEX IF NOT EXISTS idx_user_achievements_unlocked ON user_achievements(unlocked_at DESC);
CREATE INDEX IF NOT EXISTS idx_world_events_active ON world_events(is_active) WHERE is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_world_events_time ON world_events(start_time, end_time);
CREATE INDEX IF NOT EXISTS idx_event_participants_event ON event_participants(event_id);
CREATE INDEX IF NOT EXISTS idx_event_participants_user ON event_participants(user_id);
CREATE INDEX IF NOT EXISTS idx_difficulty_modifiers_user ON difficulty_modifiers(user_id);
CREATE INDEX IF NOT EXISTS idx_market_data_type ON market_data(item_type, item_subtype);
CREATE INDEX IF NOT EXISTS idx_market_data_updated ON market_data(updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_research_nodes_tree ON research_nodes(tree_id);
CREATE INDEX IF NOT EXISTS idx_user_research_user ON user_research(user_id);
CREATE INDEX IF NOT EXISTS idx_user_research_status ON user_research(research_status);

-- Functions for reputation management
CREATE OR REPLACE FUNCTION update_user_reputation(
    p_user_id INTEGER,
    p_reputation_type VARCHAR(50),
    p_change INTEGER,
    p_reason TEXT DEFAULT NULL
) RETURNS VOID AS $$
BEGIN
    INSERT INTO reputations (user_id, reputation_type, reputation_value)
    VALUES (p_user_id, p_reputation_type, GREATEST(0, p_change))
    ON CONFLICT (user_id, reputation_type)
    DO UPDATE SET 
        reputation_value = GREATEST(0, LEAST(100000, reputations.reputation_value + p_change)),
        last_updated = CURRENT_TIMESTAMP;
        
    -- Log the reputation change
    INSERT INTO log_reputation (user_id, reputation_type, change_amount, reason, created_at)
    VALUES (p_user_id, p_reputation_type, p_change, p_reason, CURRENT_TIMESTAMP);
END;
$$ LANGUAGE plpgsql;

-- Function to check achievement unlock conditions
CREATE OR REPLACE FUNCTION check_achievement_unlock(
    p_user_id INTEGER,
    p_achievement_key VARCHAR(100)
) RETURNS BOOLEAN AS $$
DECLARE
    achievement_record RECORD;
    condition_met BOOLEAN := FALSE;
    user_stats RECORD;
BEGIN
    SELECT * INTO achievement_record FROM achievements WHERE achievement_key = p_achievement_key;
    
    IF NOT FOUND THEN
        RETURN FALSE;
    END IF;
    
    -- Check if already unlocked
    IF EXISTS (SELECT 1 FROM user_achievements WHERE user_id = p_user_id AND achievement_id = achievement_record.id) THEN
        RETURN TRUE;
    END IF;
    
    -- Get user stats for condition checking
    SELECT * INTO user_stats FROM user_stats WHERE user_id = p_user_id;
    
    -- Dynamic condition evaluation based on achievement requirements
    -- This would be expanded with actual condition logic
    condition_met := TRUE; -- Placeholder
    
    IF condition_met THEN
        INSERT INTO user_achievements (user_id, achievement_id)
        VALUES (p_user_id, achievement_record.id);
        
        -- Award rewards
        UPDATE user_stats SET 
            money = money + achievement_record.reward_money,
            experience = experience + achievement_record.reward_experience
        WHERE user_id = p_user_id;
    END IF;
    
    RETURN condition_met;
END;
$$ LANGUAGE plpgsql;

-- Function for dynamic difficulty calculation
CREATE OR REPLACE FUNCTION calculate_user_difficulty(
    p_user_id INTEGER,
    p_modifier_type VARCHAR(50)
) RETURNS DECIMAL AS $$
DECLARE
    skill_level INTEGER;
    reputation_value INTEGER;
    equipment_bonus DECIMAL := 1.00;
    final_difficulty DECIMAL;
BEGIN
    -- Get user skill level (from experience)
    SELECT level INTO skill_level FROM user_stats WHERE user_id = p_user_id;
    
    -- Get reputation modifier
    SELECT COALESCE(reputation_value, 0) INTO reputation_value 
    FROM reputations 
    WHERE user_id = p_user_id AND reputation_type = p_modifier_type;
    
    -- Calculate modifiers
    final_difficulty := 1.00 * 
        (1.00 + (skill_level * 0.05)) * 
        (1.00 + (reputation_value * 0.0001)) * 
        equipment_bonus;
    
    -- Update difficulty modifier record
    INSERT INTO difficulty_modifiers (user_id, modifier_type, skill_modifier, reputation_modifier, equipment_modifier)
    VALUES (p_user_id, p_modifier_type, 1.00 + (skill_level * 0.05), 1.00 + (reputation_value * 0.0001), equipment_bonus)
    ON CONFLICT (user_id, modifier_type)
    DO UPDATE SET 
        skill_modifier = 1.00 + (skill_level * 0.05),
        reputation_modifier = 1.00 + (reputation_value * 0.0001),
        equipment_modifier = equipment_bonus,
        last_calculated = CURRENT_TIMESTAMP;
    
    RETURN final_difficulty;
END;
$$ LANGUAGE plpgsql;

-- Function for market price updates
CREATE OR REPLACE FUNCTION update_market_prices() RETURNS VOID AS $$
DECLARE
    market_item RECORD;
    price_change DECIMAL;
    new_price INTEGER;
BEGIN
    FOR market_item IN SELECT * FROM market_data LOOP
        -- Calculate price change based on supply/demand
        price_change := (market_item.demand_level - market_item.supply_level) * 0.01 * market_item.volatility_factor;
        
        -- Apply random market fluctuation
        price_change := price_change + (random() - 0.5) * 0.1;
        
        -- Calculate new price
        new_price := GREATEST(1, ROUND(market_item.current_price * (1 + price_change)));
        
        -- Update market data
        UPDATE market_data SET
            current_price = new_price,
            price_trend = price_change,
            price_history = CASE 
                WHEN jsonb_array_length(price_history) >= 100 THEN 
                    price_history[1:] || jsonb_build_object('price', new_price, 'timestamp', CURRENT_TIMESTAMP)
                ELSE 
                    price_history || jsonb_build_object('price', new_price, 'timestamp', CURRENT_TIMESTAMP)
            END,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = market_item.id;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Triggers for automatic updates
CREATE OR REPLACE FUNCTION trigger_reputation_update() RETURNS TRIGGER AS $$
BEGIN
    -- Update user's overall reputation when any specific reputation changes
    PERFORM calculate_user_difficulty(NEW.user_id, NEW.reputation_type);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER reputation_change_trigger
    AFTER INSERT OR UPDATE ON reputations
    FOR EACH ROW
    EXECUTE FUNCTION trigger_reputation_update();

-- Sample data for achievements
INSERT INTO achievements (achievement_key, title, description, achievement_type, difficulty_level, reward_money, reward_experience, unlock_conditions) VALUES
('first_hack', 'First Steps', 'Successfully hack your first server', 'milestone', 1, 1000, 500, '{"hacks_completed": 1}'),
('speed_demon', 'Speed Demon', 'Complete a hack in under 30 seconds', 'rare', 3, 5000, 2000, '{"fastest_hack_time": 30}'),
('bank_robber', 'Digital Bank Robber', 'Successfully steal over $100,000 from banks', 'milestone', 5, 25000, 10000, '{"money_stolen": 100000}'),
('defensive_master', 'Fortress Builder', 'Defend against 100 hacking attempts', 'milestone', 4, 15000, 7500, '{"defenses_successful": 100}'),
('researcher', 'Mad Scientist', 'Complete 50 research projects', 'milestone', 6, 50000, 25000, '{"research_completed": 50}');

-- Sample research trees
INSERT INTO research_trees (tree_name, tree_category, description) VALUES
('Hacking Fundamentals', 'hacking', 'Basic hacking techniques and tools'),
('Advanced Defense', 'defense', 'Sophisticated security and defensive measures'),
('Network Architecture', 'networking', 'Complex networking and infrastructure systems'),
('Hardware Engineering', 'hardware', 'Advanced hardware design and optimization'),
('Software Development', 'software', 'Cutting-edge software creation and deployment');

-- Sample market data
INSERT INTO market_data (item_type, item_subtype, base_price, current_price, supply_level, demand_level) VALUES
('software', 'password_cracker', 10000, 12000, 75, 120),
('software', 'firewall', 15000, 14500, 110, 85),
('software', 'log_cleaner', 5000, 5200, 90, 95),
('hardware', 'cpu', 25000, 26500, 60, 140),
('hardware', 'ram', 12000, 11800, 120, 80),
('hardware', 'hdd', 8000, 8400, 100, 105);

COMMENT ON TABLE reputations IS 'Player reputation system tracking different skill areas';
COMMENT ON TABLE achievements IS 'Achievement definitions and metadata';
COMMENT ON TABLE user_achievements IS 'Tracks which achievements users have unlocked';
COMMENT ON TABLE world_events IS 'Dynamic world events that affect gameplay';
COMMENT ON TABLE market_data IS 'Dynamic market pricing for items and services';
COMMENT ON TABLE research_trees IS 'Technology research progression trees';