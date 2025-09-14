-- Create processes_paused table - Tracks paused process states
-- Separate table for detailed pause information and history

CREATE TABLE IF NOT EXISTS processes_paused (
    id BIGSERIAL PRIMARY KEY,
    pid BIGINT NOT NULL,                           -- Process ID being paused
    
    -- Pause timing information
    paused_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    resumed_at TIMESTAMPTZ,                        -- When resumed (NULL if still paused)
    pause_duration INTEGER,                        -- Duration paused in seconds (calculated)
    
    -- Pause reason and context
    pause_reason VARCHAR(50) NOT NULL,             -- Reason for pause: manual, resource, security, error
    pause_message TEXT,                            -- Additional details about the pause
    paused_by_user_id BIGINT,                     -- User who initiated pause (NULL for system)
    
    -- State preservation
    progress_at_pause DECIMAL(5,2) NOT NULL,      -- Progress when paused
    cpu_usage_at_pause DOUBLE PRECISION,          -- CPU usage when paused
    net_usage_at_pause DOUBLE PRECISION,          -- Network usage when paused
    ram_usage_at_pause INTEGER,                   -- RAM usage when paused
    
    -- Resource information at pause time
    available_cpu DOUBLE PRECISION,               -- Available CPU at pause
    available_ram INTEGER,                        -- Available RAM at pause
    system_load DECIMAL(5,2),                     -- System load at pause
    
    -- Resume information
    resumed_by_user_id BIGINT,                    -- User who resumed (NULL for auto-resume)
    resume_reason VARCHAR(50),                     -- Reason for resume
    auto_resume BOOLEAN NOT NULL DEFAULT FALSE,    -- Whether this was auto-resumed
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (pid) REFERENCES processes(pid) ON DELETE CASCADE,
    FOREIGN KEY (paused_by_user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (resumed_by_user_id) REFERENCES users(id) ON DELETE SET NULL,
    
    -- Check constraints
    CONSTRAINT chk_pause_progress CHECK (progress_at_pause >= 0.0 AND progress_at_pause <= 100.0),
    CONSTRAINT chk_pause_cpu CHECK (cpu_usage_at_pause IS NULL OR (cpu_usage_at_pause >= 0.0 AND cpu_usage_at_pause <= 1.0)),
    CONSTRAINT chk_pause_net CHECK (net_usage_at_pause IS NULL OR (net_usage_at_pause >= 0.0 AND net_usage_at_pause <= 1.0)),
    CONSTRAINT chk_pause_ram CHECK (ram_usage_at_pause IS NULL OR ram_usage_at_pause >= 0),
    CONSTRAINT chk_pause_system_load CHECK (system_load IS NULL OR (system_load >= 0.0 AND system_load <= 100.0)),
    CONSTRAINT chk_pause_duration CHECK (pause_duration IS NULL OR pause_duration >= 0),
    CONSTRAINT chk_resume_timing CHECK (resumed_at IS NULL OR resumed_at >= paused_at)
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_processes_paused_pid ON processes_paused(pid);
CREATE INDEX IF NOT EXISTS idx_processes_paused_paused_at ON processes_paused(paused_at);
CREATE INDEX IF NOT EXISTS idx_processes_paused_resumed_at ON processes_paused(resumed_at);
CREATE INDEX IF NOT EXISTS idx_processes_paused_reason ON processes_paused(pause_reason);
CREATE INDEX IF NOT EXISTS idx_processes_paused_user ON processes_paused(paused_by_user_id);
CREATE INDEX IF NOT EXISTS idx_processes_paused_auto ON processes_paused(auto_resume);

-- Partial index for currently paused processes
CREATE INDEX IF NOT EXISTS idx_processes_currently_paused 
    ON processes_paused(pid, paused_at) 
    WHERE resumed_at IS NULL;

-- Create trigger for updated_at
CREATE TRIGGER processes_paused_updated_at_trigger
    BEFORE UPDATE ON processes_paused
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to calculate pause duration on resume
CREATE OR REPLACE FUNCTION calculate_pause_duration()
RETURNS TRIGGER AS $$
BEGIN
    -- Calculate duration when resuming
    IF OLD.resumed_at IS NULL AND NEW.resumed_at IS NOT NULL THEN
        NEW.pause_duration = EXTRACT(EPOCH FROM (NEW.resumed_at - NEW.paused_at))::INTEGER;
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER processes_paused_duration_trigger
    BEFORE UPDATE OF resumed_at ON processes_paused
    FOR EACH ROW
    EXECUTE FUNCTION calculate_pause_duration();

-- Function to pause a process
CREATE OR REPLACE FUNCTION pause_process(
    process_pid BIGINT,
    reason VARCHAR(50) DEFAULT 'manual',
    message TEXT DEFAULT NULL,
    user_id BIGINT DEFAULT NULL
) RETURNS BIGINT AS $$
DECLARE
    process_record RECORD;
    pause_record_id BIGINT;
BEGIN
    -- Get current process state
    SELECT * INTO process_record FROM processes WHERE pid = process_pid;
    
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Process % not found', process_pid;
    END IF;
    
    -- Check if already paused
    IF process_record.is_paused THEN
        RAISE EXCEPTION 'Process % is already paused', process_pid;
    END IF;
    
    -- Check if process is still active
    IF process_record.is_completed OR process_record.is_failed THEN
        RAISE EXCEPTION 'Cannot pause completed or failed process %', process_pid;
    END IF;
    
    -- Update process as paused
    UPDATE processes SET
        is_paused = TRUE,
        p_time_pause = CURRENT_TIMESTAMP,
        updated_at = CURRENT_TIMESTAMP
    WHERE pid = process_pid;
    
    -- Create pause record
    INSERT INTO processes_paused (
        pid, pause_reason, pause_message, paused_by_user_id,
        progress_at_pause, cpu_usage_at_pause, net_usage_at_pause, ram_usage_at_pause
    ) VALUES (
        process_pid, reason, message, user_id,
        process_record.progress_percent, process_record.cpu_usage, 
        process_record.net_usage, process_record.ram_usage
    ) RETURNING id INTO pause_record_id;
    
    -- Log the pause
    INSERT INTO process_logs (pid, event_type, message)
    VALUES (process_pid, 'PAUSED', COALESCE(message, 'Process paused: ' || reason));
    
    RETURN pause_record_id;
END;
$$ LANGUAGE plpgsql;

-- Function to resume a process
CREATE OR REPLACE FUNCTION resume_process(
    process_pid BIGINT,
    reason VARCHAR(50) DEFAULT 'manual',
    user_id BIGINT DEFAULT NULL,
    auto_resume BOOLEAN DEFAULT FALSE
) RETURNS VOID AS $$
DECLARE
    process_record RECORD;
    pause_record_id BIGINT;
BEGIN
    -- Get current process state
    SELECT * INTO process_record FROM processes WHERE pid = process_pid;
    
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Process % not found', process_pid;
    END IF;
    
    -- Check if actually paused
    IF NOT process_record.is_paused THEN
        RAISE EXCEPTION 'Process % is not paused', process_pid;
    END IF;
    
    -- Get the current pause record
    SELECT id INTO pause_record_id 
    FROM processes_paused 
    WHERE pid = process_pid AND resumed_at IS NULL
    ORDER BY paused_at DESC 
    LIMIT 1;
    
    IF NOT FOUND THEN
        RAISE EXCEPTION 'No active pause record found for process %', process_pid;
    END IF;
    
    -- Update pause record as resumed
    UPDATE processes_paused SET
        resumed_at = CURRENT_TIMESTAMP,
        resumed_by_user_id = user_id,
        resume_reason = reason,
        auto_resume = resume_process.auto_resume,
        updated_at = CURRENT_TIMESTAMP
    WHERE id = pause_record_id;
    
    -- Update process as resumed
    UPDATE processes SET
        is_paused = FALSE,
        -- Extend end time by the pause duration
        p_time_end = p_time_end + (CURRENT_TIMESTAMP - p_time_pause),
        updated_at = CURRENT_TIMESTAMP
    WHERE pid = process_pid;
    
    -- Log the resume
    INSERT INTO process_logs (pid, event_type, message)
    VALUES (process_pid, 'RESUMED', 'Process resumed: ' || reason);
END;
$$ LANGUAGE plpgsql;

-- Function to auto-resume processes based on resource availability
CREATE OR REPLACE FUNCTION auto_resume_processes()
RETURNS INTEGER AS $$
DECLARE
    resumed_count INTEGER := 0;
    pause_record RECORD;
    current_cpu DOUBLE PRECISION;
    current_ram INTEGER;
    current_load DECIMAL(5,2);
BEGIN
    -- Get current system resources (simplified - in real system would check actual resources)
    current_cpu := 0.3;  -- 30% CPU usage
    current_ram := 2048;  -- 2GB available RAM
    current_load := 25.0; -- 25% system load
    
    -- Find processes paused due to resource constraints that can now be resumed
    FOR pause_record IN
        SELECT pp.*, p.pid
        FROM processes_paused pp
        JOIN processes p ON pp.pid = p.pid
        WHERE pp.resumed_at IS NULL
        AND pp.pause_reason IN ('resource', 'cpu_limit', 'ram_limit', 'system_overload')
        AND p.is_paused = TRUE
        AND p.is_completed = FALSE
        AND p.is_failed = FALSE
        -- Check if resources are now available
        AND (pp.cpu_usage_at_pause IS NULL OR pp.cpu_usage_at_pause <= (1.0 - current_cpu))
        AND (pp.ram_usage_at_pause IS NULL OR pp.ram_usage_at_pause <= current_ram)
        AND current_load < 80.0  -- System load threshold
    LOOP
        -- Resume the process
        PERFORM resume_process(
            pause_record.pid,
            'auto_resource_available',
            NULL,
            TRUE
        );
        
        resumed_count := resumed_count + 1;
        
        -- Update resource usage estimates
        current_cpu := current_cpu + COALESCE(pause_record.cpu_usage_at_pause, 0.1);
        current_ram := current_ram - COALESCE(pause_record.ram_usage_at_pause, 100);
        
        -- Don't resume too many at once
        EXIT WHEN resumed_count >= 10;
    END LOOP;
    
    RETURN resumed_count;
END;
$$ LANGUAGE plpgsql;

-- Create view for pause statistics
CREATE OR REPLACE VIEW process_pause_stats AS
SELECT 
    p.p_creator_id,
    u.login,
    COUNT(pp.id) as total_pauses,
    COUNT(CASE WHEN pp.resumed_at IS NULL THEN 1 END) as currently_paused,
    AVG(pp.pause_duration) as avg_pause_duration,
    MAX(pp.pause_duration) as max_pause_duration,
    MIN(pp.pause_duration) as min_pause_duration,
    COUNT(CASE WHEN pp.auto_resume THEN 1 END) as auto_resumes,
    STRING_AGG(DISTINCT pp.pause_reason, ', ') as pause_reasons
FROM processes_paused pp
JOIN processes p ON pp.pid = p.pid
JOIN users u ON p.p_creator_id = u.id
GROUP BY p.p_creator_id, u.login;

-- Add comprehensive comments
COMMENT ON TABLE processes_paused IS 'Detailed tracking of process pause/resume cycles';
COMMENT ON COLUMN processes_paused.pause_reason IS 'Reason for pause: manual, resource, security, error, system_overload';
COMMENT ON COLUMN processes_paused.auto_resume IS 'Whether this pause was automatically resumed by system';
COMMENT ON COLUMN processes_paused.progress_at_pause IS 'Process completion percentage when paused';
COMMENT ON COLUMN processes_paused.pause_duration IS 'Total time paused in seconds (calculated on resume)';

COMMENT ON FUNCTION pause_process IS 'Pauses a running process and creates detailed pause record';
COMMENT ON FUNCTION resume_process IS 'Resumes a paused process and updates timing calculations';
COMMENT ON FUNCTION auto_resume_processes IS 'Automatically resumes processes when resources become available';
COMMENT ON VIEW process_pause_stats IS 'Statistical summary of pause/resume patterns by user';