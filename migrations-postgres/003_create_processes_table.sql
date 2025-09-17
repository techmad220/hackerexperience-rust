-- Create processes table (PostgreSQL version)
-- The heart of the game engine

CREATE TABLE IF NOT EXISTS processes (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    server_id BIGINT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    type VARCHAR(50) NOT NULL, -- 'scan', 'crack', 'download', 'upload', 'install', 'ddos', 'mine'
    state VARCHAR(20) NOT NULL DEFAULT 'QUEUED', -- 'QUEUED', 'RUNNING', 'PAUSED', 'CANCELLED', 'COMPLETED', 'FAILED'
    priority VARCHAR(10) NOT NULL DEFAULT 'NORMAL', -- 'LOW', 'NORMAL', 'HIGH'
    target_id BIGINT REFERENCES servers(id),
    software_id BIGINT,
    progress DECIMAL(5,2) NOT NULL DEFAULT 0.00, -- 0.00 to 100.00
    cpu_used INTEGER NOT NULL DEFAULT 0,
    ram_used INTEGER NOT NULL DEFAULT 0,
    net_used INTEGER NOT NULL DEFAULT 0,
    time_started TIMESTAMPTZ,
    time_paused TIMESTAMPTZ,
    time_completed TIMESTAMPTZ,
    estimated_completion TIMESTAMPTZ,
    data JSONB, -- Additional process-specific data
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_processes_user_id ON processes(user_id);
CREATE INDEX idx_processes_server_id ON processes(server_id);
CREATE INDEX idx_processes_state ON processes(state);
CREATE INDEX idx_processes_type ON processes(type);
CREATE INDEX idx_processes_target_id ON processes(target_id);
CREATE INDEX idx_processes_estimated_completion ON processes(estimated_completion);

CREATE TRIGGER update_processes_updated_at BEFORE UPDATE
    ON processes FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();