-- Create user_stats table - Extended user statistics and game metrics
-- Based on original usersstats table from game.sql

CREATE TABLE IF NOT EXISTS user_stats (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL UNIQUE,
    
    -- Experience and Leveling
    experience BIGINT NOT NULL DEFAULT 0,
    level INTEGER NOT NULL DEFAULT 1,
    
    -- Financial Stats  
    money BIGINT NOT NULL DEFAULT 10000,           -- Starting money
    total_earned BIGINT NOT NULL DEFAULT 0,        -- Lifetime earnings
    total_spent BIGINT NOT NULL DEFAULT 0,         -- Lifetime spending
    
    -- Gameplay Statistics
    servers_hacked INTEGER NOT NULL DEFAULT 0,      -- Total servers hacked
    missions_completed INTEGER NOT NULL DEFAULT 0,  -- Missions completed
    processes_run INTEGER NOT NULL DEFAULT 0,       -- Total processes executed
    software_installed INTEGER NOT NULL DEFAULT 0,  -- Software installations
    
    -- Time-based Stats
    total_playtime INTEGER NOT NULL DEFAULT 0,      -- Total time played (seconds)
    sessions_count INTEGER NOT NULL DEFAULT 0,      -- Total login sessions
    
    -- Combat/PvP Stats
    attacks_made INTEGER NOT NULL DEFAULT 0,        -- Attacks initiated
    attacks_received INTEGER NOT NULL DEFAULT 0,    -- Attacks received
    successful_attacks INTEGER NOT NULL DEFAULT 0,  -- Successful attacks
    failed_attacks INTEGER NOT NULL DEFAULT 0,      -- Failed attacks
    
    -- Reputation and Social
    reputation INTEGER NOT NULL DEFAULT 0,          -- Player reputation score
    clan_id BIGINT,                                 -- Current clan membership
    
    -- Security Stats
    firewall_bypassed INTEGER NOT NULL DEFAULT 0,   -- Firewalls bypassed
    logs_deleted INTEGER NOT NULL DEFAULT 0,        -- Logs cleaned
    
    -- Technical Stats
    cpu_power INTEGER NOT NULL DEFAULT 100,         -- Base CPU power
    ram_capacity INTEGER NOT NULL DEFAULT 512,      -- RAM capacity (MB)
    storage_capacity INTEGER NOT NULL DEFAULT 1024, -- Storage capacity (MB)
    storage_used INTEGER NOT NULL DEFAULT 0,        -- Currently used storage
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraint
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_user_stats_user_id ON user_stats(user_id);
CREATE INDEX IF NOT EXISTS idx_user_stats_level ON user_stats(level);
CREATE INDEX IF NOT EXISTS idx_user_stats_experience ON user_stats(experience);
CREATE INDEX IF NOT EXISTS idx_user_stats_money ON user_stats(money);
CREATE INDEX IF NOT EXISTS idx_user_stats_reputation ON user_stats(reputation);
CREATE INDEX IF NOT EXISTS idx_user_stats_clan_id ON user_stats(clan_id);

-- Create trigger for updated_at
CREATE TRIGGER user_stats_updated_at_trigger
    BEFORE UPDATE ON user_stats
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to calculate level from experience
CREATE OR REPLACE FUNCTION calculate_level_from_experience(exp BIGINT)
RETURNS INTEGER AS $$
BEGIN
    -- Simple level calculation: every 1000 XP = 1 level
    -- Can be customized based on game balance requirements
    RETURN GREATEST(1, (exp / 1000) + 1);
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Trigger to automatically update level when experience changes
CREATE OR REPLACE FUNCTION update_level_from_experience()
RETURNS TRIGGER AS $$
BEGIN
    NEW.level = calculate_level_from_experience(NEW.experience);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER user_stats_level_update_trigger
    BEFORE INSERT OR UPDATE OF experience ON user_stats
    FOR EACH ROW
    EXECUTE FUNCTION update_level_from_experience();

-- Add table and column comments
COMMENT ON TABLE user_stats IS 'Extended user statistics and gameplay metrics';
COMMENT ON COLUMN user_stats.experience IS 'Total experience points earned';
COMMENT ON COLUMN user_stats.level IS 'Current player level (auto-calculated from experience)';
COMMENT ON COLUMN user_stats.money IS 'Current account balance';
COMMENT ON COLUMN user_stats.total_earned IS 'Lifetime total money earned';
COMMENT ON COLUMN user_stats.servers_hacked IS 'Count of successfully hacked servers';
COMMENT ON COLUMN user_stats.reputation IS 'Player reputation score (-1000 to +1000)';
COMMENT ON COLUMN user_stats.cpu_power IS 'Base CPU processing power';
COMMENT ON COLUMN user_stats.storage_used IS 'Currently used storage space in MB';