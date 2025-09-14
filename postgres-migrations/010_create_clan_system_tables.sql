-- Create comprehensive clan system tables
-- Handles clans, memberships, roles, activities, and clan warfare

-- Clans table - Main clan information
CREATE TABLE IF NOT EXISTS clans (
    id BIGSERIAL PRIMARY KEY,
    
    -- Clan identification
    name VARCHAR(50) NOT NULL UNIQUE,
    tag VARCHAR(10) NOT NULL UNIQUE,              -- Clan tag/abbreviation
    description TEXT,
    motto VARCHAR(255),                           -- Clan motto/slogan
    
    -- Leadership and management
    leader_id BIGINT NOT NULL,                    -- Clan leader
    founder_id BIGINT NOT NULL,                   -- Original founder
    co_leader_id BIGINT,                         -- Optional co-leader
    
    -- Clan properties
    clan_type VARCHAR(20) NOT NULL DEFAULT 'public', -- public, private, invite_only, elite
    max_members INTEGER NOT NULL DEFAULT 50,      -- Maximum members allowed
    current_members INTEGER NOT NULL DEFAULT 1,   -- Current member count
    
    -- Status and visibility
    is_active BOOLEAN NOT NULL DEFAULT TRUE,      -- Clan is active
    is_recruiting BOOLEAN NOT NULL DEFAULT TRUE,  -- Accepting new members
    is_featured BOOLEAN NOT NULL DEFAULT FALSE,   -- Featured clan
    requires_approval BOOLEAN NOT NULL DEFAULT FALSE, -- Manual approval for joins
    
    -- Requirements for joining
    min_level INTEGER NOT NULL DEFAULT 1,         -- Minimum level to join
    min_reputation INTEGER NOT NULL DEFAULT 0,    -- Minimum reputation required
    required_skills JSONB DEFAULT '[]',           -- Required skills as JSON array
    
    -- Clan statistics and performance
    total_reputation BIGINT NOT NULL DEFAULT 0,   -- Combined member reputation
    average_level DECIMAL(5,2) NOT NULL DEFAULT 1.0, -- Average member level
    clan_rating INTEGER NOT NULL DEFAULT 1000,    -- ELO-style clan rating
    
    -- Activity and engagement metrics
    total_missions_completed INTEGER NOT NULL DEFAULT 0,
    total_servers_hacked INTEGER NOT NULL DEFAULT 0,
    total_wars_won INTEGER NOT NULL DEFAULT 0,
    total_wars_lost INTEGER NOT NULL DEFAULT 0,
    clan_bank_balance BIGINT NOT NULL DEFAULT 0,  -- Shared clan treasury
    
    -- Geographic and preference info
    primary_timezone VARCHAR(50),                 -- Primary timezone
    preferred_language VARCHAR(10) DEFAULT 'en',  -- Language preference
    region VARCHAR(50),                           -- Geographic region
    
    -- Clan customization
    logo_url VARCHAR(255),                        -- Clan logo image
    banner_url VARCHAR(255),                      -- Clan banner image
    website_url VARCHAR(255),                     -- Clan website
    discord_invite VARCHAR(100),                  -- Discord server invite
    
    -- Settings and configuration
    allow_member_invites BOOLEAN NOT NULL DEFAULT TRUE,
    auto_kick_inactive_days INTEGER DEFAULT 30,   -- Auto-kick after N days inactive
    member_contribution_required BIGINT DEFAULT 0, -- Required monthly contribution
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    disbanded_at TIMESTAMPTZ,                     -- If clan was disbanded
    
    -- Foreign key constraints
    FOREIGN KEY (leader_id) REFERENCES users(id) ON DELETE RESTRICT,
    FOREIGN KEY (founder_id) REFERENCES users(id) ON DELETE RESTRICT,
    FOREIGN KEY (co_leader_id) REFERENCES users(id) ON DELETE SET NULL,
    
    -- Check constraints
    CONSTRAINT chk_clan_max_members CHECK (max_members >= 1 AND max_members <= 200),
    CONSTRAINT chk_clan_current_members CHECK (current_members >= 0 AND current_members <= max_members),
    CONSTRAINT chk_clan_min_level CHECK (min_level >= 1),
    CONSTRAINT chk_clan_rating CHECK (clan_rating >= 0 AND clan_rating <= 3000),
    CONSTRAINT chk_clan_average_level CHECK (average_level >= 1.0),
    CONSTRAINT chk_clan_stats CHECK (
        total_missions_completed >= 0 AND total_servers_hacked >= 0 AND
        total_wars_won >= 0 AND total_wars_lost >= 0
    ),
    CONSTRAINT chk_clan_balance CHECK (clan_bank_balance >= 0),
    CONSTRAINT chk_clan_contribution CHECK (member_contribution_required >= 0),
    CONSTRAINT chk_clan_auto_kick CHECK (auto_kick_inactive_days IS NULL OR auto_kick_inactive_days > 0)
);

-- Clan members table - Member relationships and roles
CREATE TABLE IF NOT EXISTS clan_members (
    id BIGSERIAL PRIMARY KEY,
    clan_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    
    -- Member role and permissions
    role VARCHAR(20) NOT NULL DEFAULT 'member',   -- leader, co_leader, officer, veteran, member, recruit
    rank_title VARCHAR(50),                       -- Custom rank title
    permissions BIGINT NOT NULL DEFAULT 0,       -- Bitfield for permissions
    
    -- Membership status
    status VARCHAR(20) NOT NULL DEFAULT 'active', -- active, inactive, suspended, banned
    join_type VARCHAR(20) NOT NULL DEFAULT 'invited', -- invited, applied, recruited, auto
    
    -- Member contribution and activity
    contributions_total BIGINT NOT NULL DEFAULT 0, -- Total money contributed
    contributions_monthly BIGINT NOT NULL DEFAULT 0, -- This month's contributions
    activity_score INTEGER NOT NULL DEFAULT 0,    -- Member activity score
    missions_completed INTEGER NOT NULL DEFAULT 0, -- Missions completed for clan
    
    -- Recognition and achievements
    commendations INTEGER NOT NULL DEFAULT 0,     -- Commendations received
    warnings INTEGER NOT NULL DEFAULT 0,          -- Warnings received
    clan_achievements JSONB DEFAULT '[]',         -- Clan-specific achievements
    
    -- Membership timing
    joined_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_active TIMESTAMPTZ,                      -- Last activity in clan context
    promoted_at TIMESTAMPTZ,                      -- Last promotion date
    contribution_deadline TIMESTAMPTZ,            -- Next contribution due date
    
    -- Exit information
    left_at TIMESTAMPTZ,                          -- When member left
    kicked_at TIMESTAMPTZ,                        -- When member was kicked
    kicked_by_user_id BIGINT,                     -- Who kicked the member
    kick_reason TEXT,                             -- Reason for kick/ban
    
    -- Notes and tracking
    recruiter_user_id BIGINT,                     -- Who recruited this member
    notes TEXT,                                   -- Officer notes about member
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (clan_id) REFERENCES clans(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (kicked_by_user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (recruiter_user_id) REFERENCES users(id) ON DELETE SET NULL,
    
    -- Unique constraint - user can only be in one clan at a time
    UNIQUE (user_id),
    
    -- Check constraints
    CONSTRAINT chk_clan_member_contributions CHECK (contributions_total >= 0 AND contributions_monthly >= 0),
    CONSTRAINT chk_clan_member_activity CHECK (activity_score >= 0),
    CONSTRAINT chk_clan_member_missions CHECK (missions_completed >= 0),
    CONSTRAINT chk_clan_member_commendations CHECK (commendations >= 0 AND warnings >= 0),
    CONSTRAINT chk_clan_member_timing CHECK (
        (left_at IS NULL OR left_at >= joined_at) AND
        (kicked_at IS NULL OR kicked_at >= joined_at) AND
        (promoted_at IS NULL OR promoted_at >= joined_at)
    )
);

-- Clan activities table - Track clan events and activities
CREATE TABLE IF NOT EXISTS clan_activities (
    id BIGSERIAL PRIMARY KEY,
    clan_id BIGINT NOT NULL,
    user_id BIGINT,                               -- User who performed activity (NULL for system)
    
    -- Activity details
    activity_type VARCHAR(30) NOT NULL,           -- join, leave, promote, demote, kick, mission, war, etc.
    activity_category VARCHAR(20) NOT NULL DEFAULT 'general', -- admin, mission, social, combat, financial
    description TEXT NOT NULL,
    
    -- Activity targets and context
    target_user_id BIGINT,                        -- Target user for admin actions
    target_clan_id BIGINT,                        -- Target clan for clan wars
    related_mission_id BIGINT,                    -- Related mission if applicable
    
    -- Activity metadata
    activity_data JSONB DEFAULT '{}',             -- Additional activity data
    importance_level INTEGER NOT NULL DEFAULT 3,  -- 1=low, 3=normal, 5=high
    is_public BOOLEAN NOT NULL DEFAULT TRUE,      -- Visible to all clan members
    
    -- Timestamps
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (clan_id) REFERENCES clans(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (target_user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (target_clan_id) REFERENCES clans(id) ON DELETE SET NULL,
    
    -- Check constraints
    CONSTRAINT chk_clan_activity_importance CHECK (importance_level >= 1 AND importance_level <= 5)
);

-- Clan wars table - Track clan vs clan conflicts
CREATE TABLE IF NOT EXISTS clan_wars (
    id BIGSERIAL PRIMARY KEY,
    
    -- War participants
    attacker_clan_id BIGINT NOT NULL,
    defender_clan_id BIGINT NOT NULL,
    
    -- War details
    war_name VARCHAR(100),                        -- Optional war name
    war_type VARCHAR(20) NOT NULL DEFAULT 'standard', -- standard, tournament, raid, siege
    war_status VARCHAR(20) NOT NULL DEFAULT 'declared', -- declared, active, completed, cancelled
    
    -- War objectives and rules
    objective VARCHAR(30) NOT NULL DEFAULT 'dominance', -- dominance, territory, resources, honor
    max_participants INTEGER NOT NULL DEFAULT 10,      -- Max participants per side
    duration_hours INTEGER NOT NULL DEFAULT 72,        -- War duration
    
    -- War scoring
    attacker_score INTEGER NOT NULL DEFAULT 0,
    defender_score INTEGER NOT NULL DEFAULT 0,
    victory_condition VARCHAR(30) DEFAULT 'highest_score', -- highest_score, objective_based, elimination
    
    -- War results
    winner_clan_id BIGINT,                        -- Winning clan
    war_result VARCHAR(20),                       -- victory, defeat, draw, cancelled
    
    -- Rewards and penalties
    winning_reward BIGINT NOT NULL DEFAULT 10000,
    losing_penalty BIGINT NOT NULL DEFAULT 5000,
    reputation_change INTEGER NOT NULL DEFAULT 50,
    
    -- War timing
    declared_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    
    -- War metadata
    war_data JSONB DEFAULT '{}',                  -- Additional war data
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (attacker_clan_id) REFERENCES clans(id) ON DELETE CASCADE,
    FOREIGN KEY (defender_clan_id) REFERENCES clans(id) ON DELETE CASCADE,
    FOREIGN KEY (winner_clan_id) REFERENCES clans(id) ON DELETE SET NULL,
    
    -- Check constraints
    CONSTRAINT chk_clan_war_participants CHECK (attacker_clan_id != defender_clan_id),
    CONSTRAINT chk_clan_war_scores CHECK (attacker_score >= 0 AND defender_score >= 0),
    CONSTRAINT chk_clan_war_timing CHECK (
        (started_at IS NULL OR started_at >= declared_at) AND
        (ended_at IS NULL OR ended_at >= declared_at)
    ),
    CONSTRAINT chk_clan_war_rewards CHECK (
        winning_reward >= 0 AND losing_penalty >= 0 AND
        reputation_change >= -100 AND reputation_change <= 100
    ),
    CONSTRAINT chk_clan_war_duration CHECK (duration_hours > 0 AND duration_hours <= 168) -- Max 1 week
);

-- Create comprehensive indexes
CREATE INDEX IF NOT EXISTS idx_clans_name ON clans(name);
CREATE INDEX IF NOT EXISTS idx_clans_tag ON clans(tag);
CREATE INDEX IF NOT EXISTS idx_clans_leader ON clans(leader_id);
CREATE INDEX IF NOT EXISTS idx_clans_type ON clans(clan_type);
CREATE INDEX IF NOT EXISTS idx_clans_active ON clans(is_active);
CREATE INDEX IF NOT EXISTS idx_clans_recruiting ON clans(is_recruiting);
CREATE INDEX IF NOT EXISTS idx_clans_rating ON clans(clan_rating);
CREATE INDEX IF NOT EXISTS idx_clans_region ON clans(region);

CREATE INDEX IF NOT EXISTS idx_clan_members_clan_id ON clan_members(clan_id);
CREATE INDEX IF NOT EXISTS idx_clan_members_user_id ON clan_members(user_id);
CREATE INDEX IF NOT EXISTS idx_clan_members_role ON clan_members(role);
CREATE INDEX IF NOT EXISTS idx_clan_members_status ON clan_members(status);
CREATE INDEX IF NOT EXISTS idx_clan_members_joined_at ON clan_members(joined_at);
CREATE INDEX IF NOT EXISTS idx_clan_members_activity_score ON clan_members(activity_score);

CREATE INDEX IF NOT EXISTS idx_clan_activities_clan_id ON clan_activities(clan_id);
CREATE INDEX IF NOT EXISTS idx_clan_activities_user_id ON clan_activities(user_id);
CREATE INDEX IF NOT EXISTS idx_clan_activities_type ON clan_activities(activity_type);
CREATE INDEX IF NOT EXISTS idx_clan_activities_occurred_at ON clan_activities(occurred_at);
CREATE INDEX IF NOT EXISTS idx_clan_activities_importance ON clan_activities(importance_level);

CREATE INDEX IF NOT EXISTS idx_clan_wars_attacker ON clan_wars(attacker_clan_id);
CREATE INDEX IF NOT EXISTS idx_clan_wars_defender ON clan_wars(defender_clan_id);
CREATE INDEX IF NOT EXISTS idx_clan_wars_status ON clan_wars(war_status);
CREATE INDEX IF NOT EXISTS idx_clan_wars_declared_at ON clan_wars(declared_at);

-- GIN indexes for JSONB columns
CREATE INDEX IF NOT EXISTS idx_clans_required_skills ON clans USING GIN(required_skills);
CREATE INDEX IF NOT EXISTS idx_clan_members_achievements ON clan_members USING GIN(clan_achievements);
CREATE INDEX IF NOT EXISTS idx_clan_activities_data ON clan_activities USING GIN(activity_data);
CREATE INDEX IF NOT EXISTS idx_clan_wars_data ON clan_wars USING GIN(war_data);

-- Create triggers for updated_at
CREATE TRIGGER clans_updated_at_trigger
    BEFORE UPDATE ON clans
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER clan_members_updated_at_trigger
    BEFORE UPDATE ON clan_members
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER clan_wars_updated_at_trigger
    BEFORE UPDATE ON clan_wars
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to join a clan
CREATE OR REPLACE FUNCTION join_clan(
    clan_id_param BIGINT,
    user_id_param BIGINT,
    join_type_param VARCHAR(20) DEFAULT 'applied'
) RETURNS BIGINT AS $$
DECLARE
    clan_record RECORD;
    user_record RECORD;
    member_id BIGINT;
BEGIN
    -- Get clan details
    SELECT * INTO clan_record FROM clans WHERE id = clan_id_param;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Clan % not found', clan_id_param;
    END IF;
    
    -- Get user details
    SELECT u.*, us.level, us.reputation INTO user_record 
    FROM users u 
    JOIN user_stats us ON u.id = us.user_id 
    WHERE u.id = user_id_param;
    
    IF NOT FOUND THEN
        RAISE EXCEPTION 'User % not found', user_id_param;
    END IF;
    
    -- Check if clan is accepting members
    IF NOT clan_record.is_recruiting THEN
        RAISE EXCEPTION 'Clan is not currently recruiting';
    END IF;
    
    -- Check if clan is full
    IF clan_record.current_members >= clan_record.max_members THEN
        RAISE EXCEPTION 'Clan is full (% / %)', clan_record.current_members, clan_record.max_members;
    END IF;
    
    -- Check requirements
    IF user_record.level < clan_record.min_level THEN
        RAISE EXCEPTION 'User level % is below minimum requirement %', user_record.level, clan_record.min_level;
    END IF;
    
    IF user_record.reputation < clan_record.min_reputation THEN
        RAISE EXCEPTION 'User reputation % is below minimum requirement %', user_record.reputation, clan_record.min_reputation;
    END IF;
    
    -- Check if user is already in a clan
    IF EXISTS (SELECT 1 FROM clan_members WHERE user_id = user_id_param AND status = 'active') THEN
        RAISE EXCEPTION 'User is already in an active clan';
    END IF;
    
    -- Add member to clan
    INSERT INTO clan_members (
        clan_id, user_id, role, join_type, status
    ) VALUES (
        clan_id_param, user_id_param, 'member', join_type_param, 
        CASE WHEN clan_record.requires_approval AND join_type_param = 'applied' THEN 'pending' ELSE 'active' END
    ) RETURNING id INTO member_id;
    
    -- Update clan member count if member is active
    IF NOT clan_record.requires_approval OR join_type_param != 'applied' THEN
        UPDATE clans SET
            current_members = current_members + 1,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = clan_id_param;
    END IF;
    
    -- Log the activity
    INSERT INTO clan_activities (
        clan_id, user_id, activity_type, activity_category, description
    ) VALUES (
        clan_id_param, user_id_param, 'join', 'admin',
        user_record.login || ' joined the clan via ' || join_type_param
    );
    
    RETURN member_id;
END;
$$ LANGUAGE plpgsql;

-- Function to update clan statistics
CREATE OR REPLACE FUNCTION update_clan_statistics(clan_id_param BIGINT)
RETURNS VOID AS $$
DECLARE
    member_stats RECORD;
BEGIN
    -- Calculate clan statistics from active members
    SELECT 
        COUNT(*) as member_count,
        COALESCE(AVG(us.level), 1.0) as avg_level,
        COALESCE(SUM(us.reputation), 0) as total_rep,
        COALESCE(SUM(cm.missions_completed), 0) as total_missions
    INTO member_stats
    FROM clan_members cm
    JOIN user_stats us ON cm.user_id = us.user_id
    WHERE cm.clan_id = clan_id_param AND cm.status = 'active';
    
    -- Update clan with calculated statistics
    UPDATE clans SET
        current_members = member_stats.member_count,
        average_level = member_stats.avg_level,
        total_reputation = member_stats.total_rep,
        total_missions_completed = member_stats.total_missions,
        updated_at = CURRENT_TIMESTAMP
    WHERE id = clan_id_param;
END;
$$ LANGUAGE plpgsql;

-- Create view for clan rankings
CREATE OR REPLACE VIEW clan_rankings AS
SELECT 
    c.id,
    c.name,
    c.tag,
    c.clan_rating,
    c.current_members,
    c.average_level,
    c.total_reputation,
    c.total_missions_completed,
    c.total_wars_won,
    c.total_wars_lost,
    CASE 
        WHEN (c.total_wars_won + c.total_wars_lost) = 0 THEN 0
        ELSE ROUND((c.total_wars_won::DECIMAL / (c.total_wars_won + c.total_wars_lost) * 100), 2)
    END as war_win_percentage,
    l.login as leader_name,
    c.region,
    c.is_recruiting,
    ROW_NUMBER() OVER (ORDER BY c.clan_rating DESC, c.total_reputation DESC) as rank
FROM clans c
JOIN users l ON c.leader_id = l.id
WHERE c.is_active = TRUE
ORDER BY c.clan_rating DESC, c.total_reputation DESC;

-- Add comprehensive comments
COMMENT ON TABLE clans IS 'Clan organizations with membership management and statistics';
COMMENT ON TABLE clan_members IS 'Clan membership records with roles, contributions, and activity tracking';
COMMENT ON TABLE clan_activities IS 'Log of all clan activities and events for transparency';
COMMENT ON TABLE clan_wars IS 'Clan vs clan warfare system with scoring and rewards';

COMMENT ON COLUMN clans.clan_rating IS 'ELO-style rating system for clan skill/success';
COMMENT ON COLUMN clan_members.permissions IS 'Bitfield storing member permissions and capabilities';
COMMENT ON COLUMN clan_members.activity_score IS 'Calculated score based on member activity and contributions';
COMMENT ON COLUMN clan_wars.victory_condition IS 'How victory is determined for this war';

COMMENT ON VIEW clan_rankings IS 'Real-time clan rankings by rating and performance metrics';
COMMENT ON FUNCTION join_clan IS 'Handles clan membership requests with validation and requirements';
COMMENT ON FUNCTION update_clan_statistics IS 'Recalculates clan statistics from member data';