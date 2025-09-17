-- Create clans table (PostgreSQL version)

CREATE TABLE IF NOT EXISTS clans (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE,
    tag VARCHAR(10) NOT NULL UNIQUE,
    description TEXT,
    leader_id BIGINT NOT NULL REFERENCES users(id),
    member_count INTEGER NOT NULL DEFAULT 1,
    reputation INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_clans_leader_id ON clans(leader_id);
CREATE INDEX idx_clans_reputation ON clans(reputation DESC);

CREATE TRIGGER update_clans_updated_at BEFORE UPDATE
    ON clans FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Clan members junction table
CREATE TABLE IF NOT EXISTS clan_members (
    clan_id BIGINT NOT NULL REFERENCES clans(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL DEFAULT 'member', -- 'leader', 'officer', 'member'
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (clan_id, user_id)
);

CREATE INDEX idx_clan_members_user_id ON clan_members(user_id);