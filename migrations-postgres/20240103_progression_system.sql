-- Player Progression Tables

-- Player Level and Experience
CREATE TABLE IF NOT EXISTS player_progression (
    player_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    level INT DEFAULT 1,
    current_experience BIGINT DEFAULT 0,
    total_experience BIGINT DEFAULT 0,
    skill_points_available INT DEFAULT 0,
    skill_points_spent INT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Player Skills
CREATE TABLE IF NOT EXISTS player_skills (
    id SERIAL PRIMARY KEY,
    player_id UUID REFERENCES users(id) ON DELETE CASCADE,
    skill_id VARCHAR(100) NOT NULL,
    current_level INT DEFAULT 0,
    unlocked_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(player_id, skill_id)
);

-- Player Achievements
CREATE TABLE IF NOT EXISTS player_achievements (
    id SERIAL PRIMARY KEY,
    player_id UUID REFERENCES users(id) ON DELETE CASCADE,
    achievement_id VARCHAR(100) NOT NULL,
    unlocked_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(player_id, achievement_id)
);

-- Player Unlockables
CREATE TABLE IF NOT EXISTS player_unlockables (
    id SERIAL PRIMARY KEY,
    player_id UUID REFERENCES users(id) ON DELETE CASCADE,
    content_id VARCHAR(100) NOT NULL,
    unlocked_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(player_id, content_id)
);

-- Player Faction Reputation
CREATE TABLE IF NOT EXISTS player_reputation (
    id SERIAL PRIMARY KEY,
    player_id UUID REFERENCES users(id) ON DELETE CASCADE,
    faction_id VARCHAR(100) NOT NULL,
    reputation_points INT DEFAULT 0,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(player_id, faction_id)
);

-- Player Statistics
CREATE TABLE IF NOT EXISTS player_statistics (
    player_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    servers_hacked INT DEFAULT 0,
    total_hacks INT DEFAULT 0,
    missions_completed INT DEFAULT 0,
    money_earned BIGINT DEFAULT 0,
    files_downloaded INT DEFAULT 0,
    viruses_uploaded INT DEFAULT 0,
    processes_completed INT DEFAULT 0,
    pvp_wins INT DEFAULT 0,
    pvp_losses INT DEFAULT 0,
    pvp_matches INT DEFAULT 0,
    time_played_seconds BIGINT DEFAULT 0,
    last_login TIMESTAMPTZ DEFAULT NOW(),
    consecutive_login_days INT DEFAULT 1,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Leaderboards (cached/denormalized for performance)
CREATE TABLE IF NOT EXISTS leaderboards (
    id SERIAL PRIMARY KEY,
    board_type VARCHAR(50) NOT NULL, -- 'level', 'reputation', 'pvp', 'achievements'
    player_id UUID REFERENCES users(id) ON DELETE CASCADE,
    player_name VARCHAR(255) NOT NULL,
    rank INT NOT NULL,
    value BIGINT NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(board_type, player_id)
);

-- Create indexes for performance
CREATE INDEX idx_player_progression_level ON player_progression(level DESC);
CREATE INDEX idx_player_progression_total_exp ON player_progression(total_experience DESC);
CREATE INDEX idx_player_skills_player ON player_skills(player_id);
CREATE INDEX idx_player_achievements_player ON player_achievements(player_id);
CREATE INDEX idx_player_unlockables_player ON player_unlockables(player_id);
CREATE INDEX idx_player_reputation_player ON player_reputation(player_id);
CREATE INDEX idx_player_statistics_servers ON player_statistics(servers_hacked DESC);
CREATE INDEX idx_player_statistics_pvp ON player_statistics(pvp_wins DESC);
CREATE INDEX idx_leaderboards_type_rank ON leaderboards(board_type, rank);

-- Triggers to update timestamps
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_player_progression_updated
    BEFORE UPDATE ON player_progression
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_player_reputation_updated
    BEFORE UPDATE ON player_reputation
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_player_statistics_updated
    BEFORE UPDATE ON player_statistics
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_leaderboards_updated
    BEFORE UPDATE ON leaderboards
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();

-- Function to update leaderboards (call periodically or on major events)
CREATE OR REPLACE FUNCTION update_leaderboard(board VARCHAR(50))
RETURNS VOID AS $$
BEGIN
    -- Delete old entries
    DELETE FROM leaderboards WHERE board_type = board;

    -- Insert new rankings based on board type
    IF board = 'level' THEN
        INSERT INTO leaderboards (board_type, player_id, player_name, rank, value)
        SELECT
            'level',
            p.player_id,
            u.username,
            ROW_NUMBER() OVER (ORDER BY p.level DESC, p.total_experience DESC),
            p.level
        FROM player_progression p
        JOIN users u ON p.player_id = u.id
        ORDER BY p.level DESC, p.total_experience DESC
        LIMIT 100;
    ELSIF board = 'pvp' THEN
        INSERT INTO leaderboards (board_type, player_id, player_name, rank, value)
        SELECT
            'pvp',
            s.player_id,
            u.username,
            ROW_NUMBER() OVER (ORDER BY s.pvp_wins DESC),
            s.pvp_wins
        FROM player_statistics s
        JOIN users u ON s.player_id = u.id
        WHERE s.pvp_matches > 0
        ORDER BY s.pvp_wins DESC
        LIMIT 100;
    ELSIF board = 'achievements' THEN
        INSERT INTO leaderboards (board_type, player_id, player_name, rank, value)
        SELECT
            'achievements',
            a.player_id,
            u.username,
            ROW_NUMBER() OVER (ORDER BY COUNT(*) DESC),
            COUNT(*)
        FROM player_achievements a
        JOIN users u ON a.player_id = u.id
        GROUP BY a.player_id, u.username
        ORDER BY COUNT(*) DESC
        LIMIT 100;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Initialize progression for existing users (if any)
INSERT INTO player_progression (player_id)
SELECT id FROM users
WHERE id NOT IN (SELECT player_id FROM player_progression)
ON CONFLICT DO NOTHING;

INSERT INTO player_statistics (player_id)
SELECT id FROM users
WHERE id NOT IN (SELECT player_id FROM player_statistics)
ON CONFLICT DO NOTHING;