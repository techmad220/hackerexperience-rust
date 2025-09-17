-- Add process state enum and updated_at for idempotent operations

-- Create process state enum
CREATE TYPE process_state AS ENUM (
    'QUEUED',
    'RUNNING',
    'CANCELLING',
    'CANCELLED',
    'COMPLETED',
    'FAILED'
);

-- Add state column to processes table
ALTER TABLE processes
    ADD COLUMN IF NOT EXISTS state process_state NOT NULL DEFAULT 'QUEUED',
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- Create index for efficient state queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_process_state
    ON processes(state);

-- Create index for finding processes to cancel
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_process_user_state
    ON processes(user_id, state)
    WHERE state IN ('QUEUED', 'RUNNING', 'CANCELLING');

-- Add trigger to update updated_at
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER processes_updated_at
    BEFORE UPDATE ON processes
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();

-- Add resource columns if they don't exist
ALTER TABLE processes
    ADD COLUMN IF NOT EXISTS cpu_used BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS ram_used BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS server_id BIGINT REFERENCES servers(id);

-- Add available resource columns to servers if needed
ALTER TABLE servers
    ADD COLUMN IF NOT EXISTS cpu_available BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS ram_available BIGINT NOT NULL DEFAULT 0;