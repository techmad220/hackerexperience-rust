-- Create comprehensive mission and quest system tables
-- Handles missions, quests, objectives, rewards, and progression

-- Mission templates table - Base mission definitions
CREATE TABLE IF NOT EXISTS mission_templates (
    id BIGSERIAL PRIMARY KEY,
    
    -- Mission identification
    name VARCHAR(100) NOT NULL UNIQUE,
    internal_name VARCHAR(50) NOT NULL UNIQUE,    -- Code-friendly name
    category VARCHAR(30) NOT NULL,                -- tutorial, story, side, daily, weekly, clan, pvp
    subcategory VARCHAR(30),                      -- More specific categorization
    
    -- Mission content
    title VARCHAR(200) NOT NULL,                  -- Display title
    description TEXT NOT NULL,                    -- Mission description
    objective_summary TEXT NOT NULL,              -- Brief objective summary
    lore_text TEXT,                               -- Background story/lore
    
    -- Mission properties
    mission_type VARCHAR(20) NOT NULL DEFAULT 'standard', -- standard, chain, repeatable, timed
    difficulty INTEGER NOT NULL DEFAULT 1,        -- Difficulty rating (1-10)
    estimated_duration INTEGER NOT NULL DEFAULT 3600, -- Estimated completion time (seconds)
    
    -- Prerequisites and requirements
    required_level INTEGER NOT NULL DEFAULT 1,
    required_reputation INTEGER NOT NULL DEFAULT 0,
    required_skills JSONB DEFAULT '[]',           -- Required skills/abilities
    prerequisite_missions JSONB DEFAULT '[]',     -- Must complete these missions first
    blocked_by_missions JSONB DEFAULT '[]',       -- Cannot do if these are active/completed
    
    -- Availability and scheduling
    is_available BOOLEAN NOT NULL DEFAULT TRUE,   -- Mission is available to players
    is_featured BOOLEAN NOT NULL DEFAULT FALSE,   -- Featured/highlighted mission
    availability_start TIMESTAMPTZ,               -- When mission becomes available
    availability_end TIMESTAMPTZ,                 -- When mission expires
    cooldown_hours INTEGER DEFAULT 0,             -- Hours before mission can be repeated
    max_completions INTEGER,                      -- Max times mission can be completed (NULL = unlimited)
    
    -- Rewards
    reward_money BIGINT NOT NULL DEFAULT 0,       -- Money reward
    reward_experience INTEGER NOT NULL DEFAULT 0, -- Experience reward
    reward_reputation INTEGER NOT NULL DEFAULT 0, -- Reputation change
    reward_items JSONB DEFAULT '[]',              -- Item rewards
    bonus_multiplier DECIMAL(5,2) DEFAULT 1.0,    -- Bonus multiplier for exceptional performance
    
    -- Mission mechanics
    time_limit INTEGER,                           -- Time limit in seconds (NULL = no limit)
    failure_penalty BIGINT DEFAULT 0,            -- Penalty for failure
    auto_complete BOOLEAN NOT NULL DEFAULT FALSE, -- Completes automatically when objectives met
    allow_team_completion BOOLEAN NOT NULL DEFAULT FALSE, -- Can be completed by clan/team
    
    -- Mission data and configuration
    mission_data JSONB DEFAULT '{}',              -- Mission-specific configuration
    success_conditions JSONB DEFAULT '{}',       -- Conditions for success
    failure_conditions JSONB DEFAULT '{}',       -- Conditions for failure
    
    -- Statistics
    total_attempts INTEGER NOT NULL DEFAULT 0,    -- Total attempt count
    total_completions INTEGER NOT NULL DEFAULT 0, -- Total completion count
    total_failures INTEGER NOT NULL DEFAULT 0,    -- Total failure count
    average_completion_time INTEGER DEFAULT 0,    -- Average time to complete
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Check constraints
    CONSTRAINT chk_mission_difficulty CHECK (difficulty >= 1 AND difficulty <= 10),
    CONSTRAINT chk_mission_duration CHECK (estimated_duration > 0),
    CONSTRAINT chk_mission_level CHECK (required_level >= 1),
    CONSTRAINT chk_mission_cooldown CHECK (cooldown_hours >= 0),
    CONSTRAINT chk_mission_completions CHECK (max_completions IS NULL OR max_completions > 0),
    CONSTRAINT chk_mission_rewards CHECK (
        reward_money >= 0 AND reward_experience >= 0 AND
        failure_penalty >= 0 AND bonus_multiplier >= 0
    ),
    CONSTRAINT chk_mission_time_limit CHECK (time_limit IS NULL OR time_limit > 0),
    CONSTRAINT chk_mission_stats CHECK (
        total_attempts >= 0 AND total_completions >= 0 AND total_failures >= 0 AND
        average_completion_time >= 0
    ),
    CONSTRAINT chk_mission_availability CHECK (
        availability_start IS NULL OR availability_end IS NULL OR
        availability_end > availability_start
    )
);

-- Mission objectives table - Individual objectives within missions
CREATE TABLE IF NOT EXISTS mission_objectives (
    id BIGSERIAL PRIMARY KEY,
    mission_template_id BIGINT NOT NULL,
    
    -- Objective identification
    objective_key VARCHAR(50) NOT NULL,           -- Unique key within mission
    name VARCHAR(100) NOT NULL,
    description TEXT NOT NULL,
    
    -- Objective properties
    objective_type VARCHAR(30) NOT NULL,          -- hack_server, install_software, transfer_money, etc.
    is_required BOOLEAN NOT NULL DEFAULT TRUE,    -- Required to complete mission
    is_hidden BOOLEAN NOT NULL DEFAULT FALSE,     -- Hidden objective (bonus/secret)
    order_index INTEGER NOT NULL DEFAULT 0,       -- Display/completion order
    
    -- Objective target and parameters
    target_type VARCHAR(30),                      -- server, user, software, amount, etc.
    target_value TEXT,                            -- Specific target identifier
    quantity_required INTEGER NOT NULL DEFAULT 1, -- How many times to complete
    
    -- Objective configuration
    objective_data JSONB DEFAULT '{}',            -- Objective-specific data
    validation_rules JSONB DEFAULT '{}',          -- Rules for validating completion
    
    -- Rewards for completing this objective
    reward_money BIGINT DEFAULT 0,
    reward_experience INTEGER DEFAULT 0,
    reward_reputation INTEGER DEFAULT 0,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (mission_template_id) REFERENCES mission_templates(id) ON DELETE CASCADE,
    
    -- Unique constraint for objective key within mission
    UNIQUE (mission_template_id, objective_key),
    
    -- Check constraints
    CONSTRAINT chk_objective_quantity CHECK (quantity_required > 0),
    CONSTRAINT chk_objective_rewards CHECK (
        reward_money IS NULL OR reward_money >= 0 AND
        reward_experience IS NULL OR reward_experience >= 0
    )
);

-- User missions table - Active/completed missions for users
CREATE TABLE IF NOT EXISTS user_missions (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    mission_template_id BIGINT NOT NULL,
    
    -- Mission instance properties
    status VARCHAR(20) NOT NULL DEFAULT 'active', -- active, completed, failed, abandoned
    progress DECIMAL(5,2) NOT NULL DEFAULT 0.0,   -- Overall progress percentage
    
    -- Timing information
    accepted_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMPTZ,                       -- When player actually started
    completed_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    deadline TIMESTAMPTZ,                         -- Mission deadline (if applicable)
    
    -- Mission state and progress
    current_objectives JSONB DEFAULT '{}',        -- Current objective states
    completed_objectives JSONB DEFAULT '[]',      -- Completed objectives
    failed_objectives JSONB DEFAULT '[]',         -- Failed objectives
    
    -- Rewards and results
    money_earned BIGINT DEFAULT 0,
    experience_earned INTEGER DEFAULT 0,
    reputation_earned INTEGER DEFAULT 0,
    bonus_earned DECIMAL(5,2) DEFAULT 0.0,        -- Bonus multiplier earned
    
    -- Mission performance
    attempts INTEGER NOT NULL DEFAULT 1,          -- Number of attempts
    completion_time INTEGER,                      -- Time taken to complete (seconds)
    performance_rating INTEGER,                   -- Performance score (0-100)
    
    -- Additional data
    mission_data JSONB DEFAULT '{}',              -- Instance-specific data
    failure_reason TEXT,                          -- Reason for failure
    notes TEXT,                                   -- Player/system notes
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (mission_template_id) REFERENCES mission_templates(id) ON DELETE CASCADE,
    
    -- Check constraints
    CONSTRAINT chk_user_mission_progress CHECK (progress >= 0.0 AND progress <= 100.0),
    CONSTRAINT chk_user_mission_attempts CHECK (attempts >= 1),
    CONSTRAINT chk_user_mission_performance CHECK (performance_rating IS NULL OR (performance_rating >= 0 AND performance_rating <= 100)),
    CONSTRAINT chk_user_mission_timing CHECK (
        (started_at IS NULL OR started_at >= accepted_at) AND
        (completed_at IS NULL OR completed_at >= accepted_at) AND
        (failed_at IS NULL OR failed_at >= accepted_at) AND
        (deadline IS NULL OR deadline >= accepted_at)
    ),
    CONSTRAINT chk_user_mission_rewards CHECK (
        money_earned >= 0 AND experience_earned >= 0 AND bonus_earned >= 0.0
    )
);

-- Mission objective progress table - Track individual objective progress
CREATE TABLE IF NOT EXISTS user_mission_objectives (
    id BIGSERIAL PRIMARY KEY,
    user_mission_id BIGINT NOT NULL,
    mission_objective_id BIGINT NOT NULL,
    
    -- Progress tracking
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, in_progress, completed, failed
    quantity_completed INTEGER NOT NULL DEFAULT 0,
    progress_data JSONB DEFAULT '{}',               -- Objective-specific progress data
    
    -- Completion information
    completed_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    failure_reason TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (user_mission_id) REFERENCES user_missions(id) ON DELETE CASCADE,
    FOREIGN KEY (mission_objective_id) REFERENCES mission_objectives(id) ON DELETE CASCADE,
    
    -- Unique constraint
    UNIQUE (user_mission_id, mission_objective_id),
    
    -- Check constraints
    CONSTRAINT chk_objective_progress_quantity CHECK (quantity_completed >= 0)
);

-- Mission logs table - Detailed logging of mission events
CREATE TABLE IF NOT EXISTS mission_logs (
    id BIGSERIAL PRIMARY KEY,
    user_mission_id BIGINT NOT NULL,
    
    -- Log entry details
    event_type VARCHAR(30) NOT NULL,              -- accepted, started, objective_completed, completed, failed, etc.
    message TEXT NOT NULL,
    details JSONB DEFAULT '{}',
    
    -- Context information
    objective_id BIGINT,                          -- Related objective if applicable
    related_entity_type VARCHAR(30),              -- Type of related entity (server, user, etc.)
    related_entity_id BIGINT,                     -- ID of related entity
    
    -- Timestamps
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (user_mission_id) REFERENCES user_missions(id) ON DELETE CASCADE,
    FOREIGN KEY (objective_id) REFERENCES mission_objectives(id) ON DELETE SET NULL
);

-- Create comprehensive indexes
CREATE INDEX IF NOT EXISTS idx_mission_templates_category ON mission_templates(category);
CREATE INDEX IF NOT EXISTS idx_mission_templates_difficulty ON mission_templates(difficulty);
CREATE INDEX IF NOT EXISTS idx_mission_templates_available ON mission_templates(is_available);
CREATE INDEX IF NOT EXISTS idx_mission_templates_featured ON mission_templates(is_featured);
CREATE INDEX IF NOT EXISTS idx_mission_templates_level ON mission_templates(required_level);

CREATE INDEX IF NOT EXISTS idx_mission_objectives_template ON mission_objectives(mission_template_id);
CREATE INDEX IF NOT EXISTS idx_mission_objectives_type ON mission_objectives(objective_type);
CREATE INDEX IF NOT EXISTS idx_mission_objectives_required ON mission_objectives(is_required);
CREATE INDEX IF NOT EXISTS idx_mission_objectives_order ON mission_objectives(order_index);

CREATE INDEX IF NOT EXISTS idx_user_missions_user_id ON user_missions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_missions_template ON user_missions(mission_template_id);
CREATE INDEX IF NOT EXISTS idx_user_missions_status ON user_missions(status);
CREATE INDEX IF NOT EXISTS idx_user_missions_accepted_at ON user_missions(accepted_at);
CREATE INDEX IF NOT EXISTS idx_user_missions_deadline ON user_missions(deadline);
CREATE INDEX IF NOT EXISTS idx_user_missions_progress ON user_missions(progress);

CREATE INDEX IF NOT EXISTS idx_user_mission_objectives_mission ON user_mission_objectives(user_mission_id);
CREATE INDEX IF NOT EXISTS idx_user_mission_objectives_objective ON user_mission_objectives(mission_objective_id);
CREATE INDEX IF NOT EXISTS idx_user_mission_objectives_status ON user_mission_objectives(status);

CREATE INDEX IF NOT EXISTS idx_mission_logs_mission ON mission_logs(user_mission_id);
CREATE INDEX IF NOT EXISTS idx_mission_logs_event_type ON mission_logs(event_type);
CREATE INDEX IF NOT EXISTS idx_mission_logs_occurred_at ON mission_logs(occurred_at);

-- GIN indexes for JSONB columns
CREATE INDEX IF NOT EXISTS idx_mission_templates_skills ON mission_templates USING GIN(required_skills);
CREATE INDEX IF NOT EXISTS idx_mission_templates_data ON mission_templates USING GIN(mission_data);
CREATE INDEX IF NOT EXISTS idx_mission_objectives_data ON mission_objectives USING GIN(objective_data);
CREATE INDEX IF NOT EXISTS idx_user_missions_objectives ON user_missions USING GIN(current_objectives);
CREATE INDEX IF NOT EXISTS idx_mission_logs_details ON mission_logs USING GIN(details);

-- Create triggers for updated_at
CREATE TRIGGER mission_templates_updated_at_trigger
    BEFORE UPDATE ON mission_templates
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER mission_objectives_updated_at_trigger
    BEFORE UPDATE ON mission_objectives
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER user_missions_updated_at_trigger
    BEFORE UPDATE ON user_missions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER user_mission_objectives_updated_at_trigger
    BEFORE UPDATE ON user_mission_objectives
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to accept a mission
CREATE OR REPLACE FUNCTION accept_mission(
    user_id_param BIGINT,
    mission_template_id_param BIGINT
) RETURNS BIGINT AS $$
DECLARE
    user_record RECORD;
    mission_record RECORD;
    user_mission_id BIGINT;
    objective_record RECORD;
    deadline_timestamp TIMESTAMPTZ;
BEGIN
    -- Get user details
    SELECT u.*, us.level, us.reputation INTO user_record
    FROM users u
    JOIN user_stats us ON u.id = us.user_id
    WHERE u.id = user_id_param;
    
    IF NOT FOUND THEN
        RAISE EXCEPTION 'User % not found', user_id_param;
    END IF;
    
    -- Get mission details
    SELECT * INTO mission_record FROM mission_templates WHERE id = mission_template_id_param;
    
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Mission template % not found', mission_template_id_param;
    END IF;
    
    -- Check if mission is available
    IF NOT mission_record.is_available THEN
        RAISE EXCEPTION 'Mission is not available';
    END IF;
    
    -- Check availability window
    IF mission_record.availability_start IS NOT NULL AND CURRENT_TIMESTAMP < mission_record.availability_start THEN
        RAISE EXCEPTION 'Mission is not yet available';
    END IF;
    
    IF mission_record.availability_end IS NOT NULL AND CURRENT_TIMESTAMP > mission_record.availability_end THEN
        RAISE EXCEPTION 'Mission is no longer available';
    END IF;
    
    -- Check requirements
    IF user_record.level < mission_record.required_level THEN
        RAISE EXCEPTION 'User level % is below required level %', user_record.level, mission_record.required_level;
    END IF;
    
    IF user_record.reputation < mission_record.required_reputation THEN
        RAISE EXCEPTION 'User reputation % is below required reputation %', user_record.reputation, mission_record.required_reputation;
    END IF;
    
    -- Check if user already has this mission active
    IF EXISTS (
        SELECT 1 FROM user_missions 
        WHERE user_id = user_id_param 
        AND mission_template_id = mission_template_id_param 
        AND status = 'active'
    ) THEN
        RAISE EXCEPTION 'User already has this mission active';
    END IF;
    
    -- Calculate deadline if mission has time limit
    IF mission_record.time_limit IS NOT NULL THEN
        deadline_timestamp := CURRENT_TIMESTAMP + INTERVAL '1 second' * mission_record.time_limit;
    END IF;
    
    -- Create user mission record
    INSERT INTO user_missions (
        user_id, mission_template_id, deadline
    ) VALUES (
        user_id_param, mission_template_id_param, deadline_timestamp
    ) RETURNING id INTO user_mission_id;
    
    -- Create objective progress records
    FOR objective_record IN 
        SELECT * FROM mission_objectives 
        WHERE mission_template_id = mission_template_id_param 
        ORDER BY order_index
    LOOP
        INSERT INTO user_mission_objectives (
            user_mission_id, mission_objective_id
        ) VALUES (
            user_mission_id, objective_record.id
        );
    END LOOP;
    
    -- Update mission statistics
    UPDATE mission_templates SET
        total_attempts = total_attempts + 1,
        updated_at = CURRENT_TIMESTAMP
    WHERE id = mission_template_id_param;
    
    -- Log mission acceptance
    INSERT INTO mission_logs (
        user_mission_id, event_type, message
    ) VALUES (
        user_mission_id, 'accepted', 'Mission accepted by user'
    );
    
    RETURN user_mission_id;
END;
$$ LANGUAGE plpgsql;

-- Function to complete a mission objective
CREATE OR REPLACE FUNCTION complete_mission_objective(
    user_mission_id_param BIGINT,
    objective_key_param VARCHAR(50),
    quantity_completed_param INTEGER DEFAULT 1
) RETURNS BOOLEAN AS $$
DECLARE
    objective_progress RECORD;
    objective_template RECORD;
    mission_record RECORD;
    total_progress DECIMAL(5,2);
BEGIN
    -- Get objective progress
    SELECT umo.*, mo.* INTO objective_progress
    FROM user_mission_objectives umo
    JOIN mission_objectives mo ON umo.mission_objective_id = mo.id
    WHERE umo.user_mission_id = user_mission_id_param
    AND mo.objective_key = objective_key_param;
    
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Objective % not found for mission %', objective_key_param, user_mission_id_param;
    END IF;
    
    -- Check if objective is already completed
    IF objective_progress.status = 'completed' THEN
        RETURN FALSE;
    END IF;
    
    -- Update objective progress
    UPDATE user_mission_objectives SET
        quantity_completed = LEAST(
            quantity_completed + quantity_completed_param,
            objective_progress.quantity_required
        ),
        status = CASE 
            WHEN quantity_completed + quantity_completed_param >= objective_progress.quantity_required 
            THEN 'completed'
            ELSE 'in_progress'
        END,
        completed_at = CASE 
            WHEN quantity_completed + quantity_completed_param >= objective_progress.quantity_required 
            THEN CURRENT_TIMESTAMP
            ELSE completed_at
        END,
        updated_at = CURRENT_TIMESTAMP
    WHERE user_mission_id = user_mission_id_param
    AND mission_objective_id = objective_progress.mission_objective_id;
    
    -- Log objective completion
    INSERT INTO mission_logs (
        user_mission_id, event_type, message, objective_id
    ) VALUES (
        user_mission_id_param, 'objective_progress', 
        'Objective progress: ' || objective_key_param,
        objective_progress.mission_objective_id
    );
    
    -- Check if objective is now completed
    IF objective_progress.quantity_completed + quantity_completed_param >= objective_progress.quantity_required THEN
        -- Award objective rewards if any
        IF objective_progress.reward_money > 0 OR objective_progress.reward_experience > 0 THEN
            UPDATE user_stats SET
                money = money + COALESCE(objective_progress.reward_money, 0),
                experience = experience + COALESCE(objective_progress.reward_experience, 0),
                reputation = reputation + COALESCE(objective_progress.reward_reputation, 0),
                updated_at = CURRENT_TIMESTAMP
            WHERE user_id = (SELECT user_id FROM user_missions WHERE id = user_mission_id_param);
        END IF;
        
        -- Check if all required objectives are completed
        PERFORM check_mission_completion(user_mission_id_param);
    END IF;
    
    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- Function to check and handle mission completion
CREATE OR REPLACE FUNCTION check_mission_completion(user_mission_id_param BIGINT)
RETURNS BOOLEAN AS $$
DECLARE
    mission_record RECORD;
    required_objectives_completed INTEGER;
    total_required_objectives INTEGER;
BEGIN
    -- Get mission details
    SELECT um.*, mt.* INTO mission_record
    FROM user_missions um
    JOIN mission_templates mt ON um.mission_template_id = mt.id
    WHERE um.id = user_mission_id_param;
    
    -- Count completed required objectives
    SELECT COUNT(*) INTO required_objectives_completed
    FROM user_mission_objectives umo
    JOIN mission_objectives mo ON umo.mission_objective_id = mo.id
    WHERE umo.user_mission_id = user_mission_id_param
    AND mo.is_required = TRUE
    AND umo.status = 'completed';
    
    -- Count total required objectives
    SELECT COUNT(*) INTO total_required_objectives
    FROM mission_objectives mo
    WHERE mo.mission_template_id = mission_record.mission_template_id
    AND mo.is_required = TRUE;
    
    -- Check if mission is complete
    IF required_objectives_completed >= total_required_objectives THEN
        -- Calculate completion time
        UPDATE user_missions SET
            status = 'completed',
            progress = 100.0,
            completed_at = CURRENT_TIMESTAMP,
            completion_time = EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - accepted_at))::INTEGER,
            money_earned = mission_record.reward_money,
            experience_earned = mission_record.reward_experience,
            reputation_earned = mission_record.reward_reputation,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = user_mission_id_param;
        
        -- Award mission rewards
        UPDATE user_stats SET
            money = money + mission_record.reward_money,
            experience = experience + mission_record.reward_experience,
            reputation = reputation + mission_record.reward_reputation,
            missions_completed = missions_completed + 1,
            updated_at = CURRENT_TIMESTAMP
        WHERE user_id = mission_record.user_id;
        
        -- Update mission template statistics
        UPDATE mission_templates SET
            total_completions = total_completions + 1,
            average_completion_time = (
                (average_completion_time * (total_completions - 1) + 
                 EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - mission_record.accepted_at))::INTEGER) / 
                total_completions
            ),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = mission_record.mission_template_id;
        
        -- Log mission completion
        INSERT INTO mission_logs (
            user_mission_id, event_type, message
        ) VALUES (
            user_mission_id_param, 'completed', 'Mission completed successfully'
        );
        
        RETURN TRUE;
    END IF;
    
    RETURN FALSE;
END;
$$ LANGUAGE plpgsql;

-- Create view for available missions per user
CREATE OR REPLACE VIEW available_missions AS
SELECT 
    mt.*,
    CASE 
        WHEN mt.availability_start IS NOT NULL AND CURRENT_TIMESTAMP < mt.availability_start THEN FALSE
        WHEN mt.availability_end IS NOT NULL AND CURRENT_TIMESTAMP > mt.availability_end THEN FALSE
        ELSE mt.is_available
    END as currently_available
FROM mission_templates mt
WHERE mt.is_available = TRUE
ORDER BY mt.is_featured DESC, mt.difficulty ASC, mt.created_at DESC;

-- Add comprehensive comments
COMMENT ON TABLE mission_templates IS 'Base mission definitions with requirements and rewards';
COMMENT ON TABLE mission_objectives IS 'Individual objectives that make up missions';
COMMENT ON TABLE user_missions IS 'User mission instances tracking progress and completion';
COMMENT ON TABLE user_mission_objectives IS 'Individual objective progress for user missions';
COMMENT ON TABLE mission_logs IS 'Detailed logging of mission events and progress';

COMMENT ON COLUMN mission_templates.mission_type IS 'Mission type: standard, chain, repeatable, timed';
COMMENT ON COLUMN mission_templates.required_skills IS 'JSON array of required skills/abilities';
COMMENT ON COLUMN mission_objectives.objective_type IS 'Type of objective: hack_server, install_software, etc.';
COMMENT ON COLUMN user_missions.current_objectives IS 'JSON object tracking current objective states';

COMMENT ON VIEW available_missions IS 'Real-time view of missions available to players';
COMMENT ON FUNCTION accept_mission IS 'Handles mission acceptance with requirement validation';
COMMENT ON FUNCTION complete_mission_objective IS 'Updates objective progress and checks mission completion';
COMMENT ON FUNCTION check_mission_completion IS 'Checks and handles mission completion with rewards';