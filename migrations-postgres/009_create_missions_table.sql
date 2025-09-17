-- Create missions table (PostgreSQL version)

CREATE TABLE IF NOT EXISTS missions (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    type VARCHAR(50) NOT NULL, -- 'tutorial', 'story', 'daily', 'special'
    difficulty INTEGER NOT NULL DEFAULT 1, -- 1-10
    reward_money BIGINT NOT NULL DEFAULT 0,
    reward_exp INTEGER NOT NULL DEFAULT 0,
    prerequisite_mission_id BIGINT REFERENCES missions(id),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_missions_type ON missions(type);
CREATE INDEX idx_missions_difficulty ON missions(difficulty);

-- User mission progress
CREATE TABLE IF NOT EXISTS user_missions (
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    mission_id BIGINT NOT NULL REFERENCES missions(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL DEFAULT 'available', -- 'available', 'active', 'completed', 'failed'
    progress JSONB,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    PRIMARY KEY (user_id, mission_id)
);

CREATE INDEX idx_user_missions_status ON user_missions(status);