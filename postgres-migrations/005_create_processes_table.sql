-- Create processes table - The most complex part of the HackerExperience system
-- Based on original processes table - handles all game actions and activities

CREATE TABLE IF NOT EXISTS processes (
    pid BIGSERIAL PRIMARY KEY,                     -- Process ID
    
    -- Process ownership and targeting
    p_creator_id BIGINT NOT NULL,                  -- User who created this process
    p_victim_id BIGINT NOT NULL,                   -- Target user/server of the process
    
    -- Process type and configuration
    p_action SMALLINT NOT NULL,                    -- Action type (maps to ProcessAction enum)
    p_soft_id BIGINT NOT NULL,                     -- Software used for this process
    p_info VARCHAR(30) NOT NULL DEFAULT '',       -- Additional process information
    p_info_str TEXT NOT NULL DEFAULT '',          -- Extended process information (JSON)
    
    -- Timing information
    p_time_start TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01 00:00:01+00', -- Process start time
    p_time_pause TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01 00:00:01+00', -- When paused (if applicable)
    p_time_end TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01 00:00:01+00',   -- Expected/actual end time
    p_time_ideal INTEGER NOT NULL DEFAULT 0,      -- Ideal completion time (seconds)
    p_time_worked INTEGER NOT NULL DEFAULT 0,     -- Actual time worked (seconds)
    
    -- Resource usage
    cpu_usage DOUBLE PRECISION NOT NULL DEFAULT 0.0,  -- CPU utilization (0.0-1.0)
    net_usage DOUBLE PRECISION NOT NULL DEFAULT 0.0,  -- Network utilization (0.0-1.0)
    ram_usage INTEGER NOT NULL DEFAULT 0,             -- RAM usage in MB
    
    -- Process state and flags
    p_local BOOLEAN NOT NULL DEFAULT FALSE,        -- Local process (confusingly named - actually target IP)
    p_npc BOOLEAN NOT NULL DEFAULT FALSE,          -- Process involves NPC
    is_paused BOOLEAN NOT NULL DEFAULT FALSE,      -- Process is currently paused
    is_completed BOOLEAN NOT NULL DEFAULT FALSE,   -- Process completed successfully
    is_failed BOOLEAN NOT NULL DEFAULT FALSE,      -- Process failed
    
    -- Process priority and dependencies
    priority INTEGER NOT NULL DEFAULT 5,           -- Process priority (1-10)
    parent_pid BIGINT,                             -- Parent process (for chained processes)
    
    -- Progress and completion
    progress_percent DECIMAL(5,2) NOT NULL DEFAULT 0.00, -- Progress percentage
    completion_code INTEGER NOT NULL DEFAULT 0,    -- Completion result code
    error_message TEXT,                            -- Error message if failed
    
    -- Security and stealth
    stealth_level INTEGER NOT NULL DEFAULT 1,      -- How hidden the process is (1-10)
    detection_risk DECIMAL(5,2) NOT NULL DEFAULT 50.0, -- Risk of detection (0-100%)
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (p_creator_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (p_victim_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_pid) REFERENCES processes(pid) ON DELETE SET NULL,
    
    -- Check constraints
    CONSTRAINT chk_process_cpu_usage CHECK (cpu_usage >= 0.0 AND cpu_usage <= 1.0),
    CONSTRAINT chk_process_net_usage CHECK (net_usage >= 0.0 AND net_usage <= 1.0),
    CONSTRAINT chk_process_ram_usage CHECK (ram_usage >= 0),
    CONSTRAINT chk_process_priority CHECK (priority >= 1 AND priority <= 10),
    CONSTRAINT chk_process_stealth CHECK (stealth_level >= 1 AND stealth_level <= 10),
    CONSTRAINT chk_process_detection CHECK (detection_risk >= 0.0 AND detection_risk <= 100.0),
    CONSTRAINT chk_process_progress CHECK (progress_percent >= 0.0 AND progress_percent <= 100.0),
    CONSTRAINT chk_process_time_worked CHECK (p_time_worked >= 0),
    CONSTRAINT chk_process_time_ideal CHECK (p_time_ideal >= 0),
    CONSTRAINT chk_process_state CHECK (
        -- A process cannot be both completed and failed
        NOT (is_completed = TRUE AND is_failed = TRUE)
    )
);

-- Create comprehensive indexes for performance
CREATE INDEX IF NOT EXISTS idx_processes_creator ON processes(p_creator_id);
CREATE INDEX IF NOT EXISTS idx_processes_victim ON processes(p_victim_id);
CREATE INDEX IF NOT EXISTS idx_processes_action ON processes(p_action);
CREATE INDEX IF NOT EXISTS idx_processes_npc ON processes(p_npc);
CREATE INDEX IF NOT EXISTS idx_processes_paused ON processes(is_paused);
CREATE INDEX IF NOT EXISTS idx_processes_completed ON processes(is_completed);
CREATE INDEX IF NOT EXISTS idx_processes_failed ON processes(is_failed);
CREATE INDEX IF NOT EXISTS idx_processes_time_end ON processes(p_time_end);
CREATE INDEX IF NOT EXISTS idx_processes_priority ON processes(priority);
CREATE INDEX IF NOT EXISTS idx_processes_parent ON processes(parent_pid);
CREATE INDEX IF NOT EXISTS idx_processes_creator_action ON processes(p_creator_id, p_action);
CREATE INDEX IF NOT EXISTS idx_processes_victim_action ON processes(p_victim_id, p_action);

-- Partial indexes for active processes
CREATE INDEX IF NOT EXISTS idx_processes_active ON processes(p_creator_id, p_time_end) 
    WHERE is_paused = FALSE AND is_completed = FALSE AND is_failed = FALSE;
CREATE INDEX IF NOT EXISTS idx_processes_running ON processes(p_time_end) 
    WHERE is_paused = FALSE AND is_completed = FALSE AND is_failed = FALSE;

-- Create trigger for updated_at
CREATE TRIGGER processes_updated_at_trigger
    BEFORE UPDATE ON processes
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to calculate process progress
CREATE OR REPLACE FUNCTION calculate_process_progress(
    start_time TIMESTAMPTZ,
    end_time TIMESTAMPTZ,
    time_worked INTEGER,
    is_paused BOOLEAN
) RETURNS DECIMAL(5,2) AS $$
DECLARE
    current_time TIMESTAMPTZ := CURRENT_TIMESTAMP;
    total_duration INTEGER;
    effective_time INTEGER;
    progress DECIMAL(5,2);
BEGIN
    -- Calculate total expected duration
    total_duration := EXTRACT(EPOCH FROM (end_time - start_time))::INTEGER;
    
    -- Handle edge cases
    IF total_duration <= 0 THEN
        RETURN 0.00;
    END IF;
    
    -- If paused, use time_worked; otherwise calculate from current time
    IF is_paused THEN
        effective_time := time_worked;
    ELSE
        effective_time := GREATEST(time_worked, EXTRACT(EPOCH FROM (current_time - start_time))::INTEGER);
    END IF;
    
    -- Calculate progress percentage
    progress := (effective_time::DECIMAL / total_duration * 100.0);
    
    -- Ensure progress is within bounds
    progress := GREATEST(0.00, LEAST(100.00, progress));
    
    RETURN progress;
END;
$$ LANGUAGE plpgsql;

-- Trigger to automatically update progress
CREATE OR REPLACE FUNCTION update_process_progress()
RETURNS TRIGGER AS $$
BEGIN
    -- Only update progress for active processes
    IF NEW.is_completed = FALSE AND NEW.is_failed = FALSE THEN
        NEW.progress_percent = calculate_process_progress(
            NEW.p_time_start,
            NEW.p_time_end,
            NEW.p_time_worked,
            NEW.is_paused
        );
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER processes_progress_trigger
    BEFORE UPDATE ON processes
    FOR EACH ROW
    EXECUTE FUNCTION update_process_progress();

-- Function to handle process completion
CREATE OR REPLACE FUNCTION complete_process(
    process_pid BIGINT,
    success BOOLEAN DEFAULT TRUE,
    completion_code INTEGER DEFAULT 0,
    error_msg TEXT DEFAULT NULL
) RETURNS VOID AS $$
BEGIN
    UPDATE processes SET
        is_completed = success,
        is_failed = NOT success,
        progress_percent = CASE WHEN success THEN 100.00 ELSE progress_percent END,
        completion_code = complete_process.completion_code,
        error_message = error_msg,
        updated_at = CURRENT_TIMESTAMP
    WHERE pid = process_pid;
    
    -- Log completion
    INSERT INTO process_logs (pid, event_type, message, created_at)
    VALUES (
        process_pid,
        CASE WHEN success THEN 'COMPLETED' ELSE 'FAILED' END,
        COALESCE(error_msg, 'Process ' || CASE WHEN success THEN 'completed successfully' ELSE 'failed' END),
        CURRENT_TIMESTAMP
    );
END;
$$ LANGUAGE plpgsql;

-- Create process logs table for detailed tracking
CREATE TABLE IF NOT EXISTS process_logs (
    id BIGSERIAL PRIMARY KEY,
    pid BIGINT NOT NULL,
    event_type VARCHAR(20) NOT NULL,    -- STARTED, PAUSED, RESUMED, COMPLETED, FAILED, ERROR
    message TEXT,
    details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (pid) REFERENCES processes(pid) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_process_logs_pid ON process_logs(pid);
CREATE INDEX IF NOT EXISTS idx_process_logs_event_type ON process_logs(event_type);
CREATE INDEX IF NOT EXISTS idx_process_logs_created_at ON process_logs(created_at);

-- Create view for active processes with calculated fields
CREATE OR REPLACE VIEW active_processes AS
SELECT 
    p.pid,
    p.p_creator_id,
    u1.login as creator_login,
    p.p_victim_id,
    u2.login as victim_login,
    p.p_action,
    p.p_soft_id,
    p.p_time_start,
    p.p_time_end,
    p.progress_percent,
    p.cpu_usage,
    p.net_usage,
    p.ram_usage,
    p.priority,
    p.stealth_level,
    p.detection_risk,
    p.is_paused,
    -- Calculated fields
    EXTRACT(EPOCH FROM (p.p_time_end - CURRENT_TIMESTAMP))::INTEGER as seconds_remaining,
    EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - p.p_time_start))::INTEGER as seconds_elapsed,
    CASE 
        WHEN p.p_time_end <= CURRENT_TIMESTAMP THEN 'READY'
        WHEN p.is_paused THEN 'PAUSED'
        ELSE 'RUNNING'
    END as status
FROM processes p
JOIN users u1 ON p.p_creator_id = u1.id
JOIN users u2 ON p.p_victim_id = u2.id
WHERE p.is_completed = FALSE AND p.is_failed = FALSE;

-- Add comprehensive comments
COMMENT ON TABLE processes IS 'Core process table handling all game actions and activities';
COMMENT ON COLUMN processes.pid IS 'Unique process identifier';
COMMENT ON COLUMN processes.p_creator_id IS 'User who initiated this process';
COMMENT ON COLUMN processes.p_victim_id IS 'Target user/server of the process';
COMMENT ON COLUMN processes.p_action IS 'Action type being performed (maps to ProcessAction enum)';
COMMENT ON COLUMN processes.p_soft_id IS 'Software/tool being used for this process';
COMMENT ON COLUMN processes.cpu_usage IS 'CPU utilization as fraction (0.0 to 1.0)';
COMMENT ON COLUMN processes.stealth_level IS 'How hidden the process is (1=obvious, 10=invisible)';
COMMENT ON COLUMN processes.detection_risk IS 'Percentage risk of being detected';
COMMENT ON COLUMN processes.parent_pid IS 'Parent process for chained/dependent processes';

COMMENT ON TABLE process_logs IS 'Detailed logging for process lifecycle events';
COMMENT ON VIEW active_processes IS 'Real-time view of running processes with calculated status';
COMMENT ON FUNCTION calculate_process_progress IS 'Calculates process completion percentage';
COMMENT ON FUNCTION complete_process IS 'Marks a process as completed or failed with logging';