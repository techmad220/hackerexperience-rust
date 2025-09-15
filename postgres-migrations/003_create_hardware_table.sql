-- Create hardware table - Server hardware specifications
-- Based on original hardware table from game.sql

CREATE TABLE IF NOT EXISTS hardware (
    server_id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    name VARCHAR(15) NOT NULL,
    
    -- Hardware specifications
    cpu DECIMAL(10,2) NOT NULL DEFAULT 500.00,     -- CPU power in GHz
    hdd DECIMAL(10,2) NOT NULL DEFAULT 100.00,     -- Hard disk capacity in GB
    ram DECIMAL(10,2) NOT NULL DEFAULT 256.00,     -- RAM capacity in MB
    net DECIMAL(10,2) NOT NULL DEFAULT 1.00,       -- Network speed in Mbps
    
    -- Server type and flags
    is_npc BOOLEAN NOT NULL DEFAULT FALSE,         -- NPC-owned server flag
    is_public BOOLEAN NOT NULL DEFAULT FALSE,      -- Public access server
    requires_password BOOLEAN NOT NULL DEFAULT FALSE, -- Password protection
    
    -- Security and access
    firewall_level INTEGER NOT NULL DEFAULT 0,     -- Firewall protection level
    encryption_level INTEGER NOT NULL DEFAULT 0,   -- Encryption strength
    
    -- Status and location
    online_status BOOLEAN NOT NULL DEFAULT TRUE,   -- Server online/offline
    ip_address INET,                               -- Server IP address
    location VARCHAR(100),                         -- Physical location
    
    -- Performance metrics
    current_load DECIMAL(5,2) NOT NULL DEFAULT 0.00,     -- Current CPU load %
    max_connections INTEGER NOT NULL DEFAULT 10,         -- Max simultaneous connections
    current_connections INTEGER NOT NULL DEFAULT 0,      -- Current active connections
    
    -- Timestamps and tracking
    last_access TIMESTAMPTZ,                       -- Last access time
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    
    -- Constraints
    CONSTRAINT chk_cpu_positive CHECK (cpu > 0),
    CONSTRAINT chk_hdd_positive CHECK (hdd > 0),
    CONSTRAINT chk_ram_positive CHECK (ram > 0),
    CONSTRAINT chk_net_positive CHECK (net > 0),
    CONSTRAINT chk_firewall_level CHECK (firewall_level >= 0 AND firewall_level <= 10),
    CONSTRAINT chk_encryption_level CHECK (encryption_level >= 0 AND encryption_level <= 10),
    CONSTRAINT chk_current_load CHECK (current_load >= 0 AND current_load <= 100),
    CONSTRAINT chk_max_connections CHECK (max_connections > 0),
    CONSTRAINT chk_current_connections CHECK (current_connections >= 0 AND current_connections <= max_connections)
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_hardware_user_id ON hardware(user_id);
CREATE INDEX IF NOT EXISTS idx_hardware_is_npc ON hardware(is_npc);
CREATE INDEX IF NOT EXISTS idx_hardware_user_npc ON hardware(user_id, is_npc);
CREATE INDEX IF NOT EXISTS idx_hardware_online_status ON hardware(online_status);
CREATE INDEX IF NOT EXISTS idx_hardware_ip_address ON hardware(ip_address);
CREATE INDEX IF NOT EXISTS idx_hardware_last_access ON hardware(last_access);
CREATE INDEX IF NOT EXISTS idx_hardware_firewall_level ON hardware(firewall_level);

-- Create trigger for updated_at
CREATE TRIGGER hardware_updated_at_trigger
    BEFORE UPDATE ON hardware
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to update last_access timestamp
CREATE OR REPLACE FUNCTION update_server_access()
RETURNS TRIGGER AS $$
BEGIN
    -- Update last_access when connection count changes (indicating access)
    IF OLD.current_connections != NEW.current_connections THEN
        NEW.last_access = CURRENT_TIMESTAMP;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER hardware_access_trigger
    BEFORE UPDATE OF current_connections ON hardware
    FOR EACH ROW
    EXECUTE FUNCTION update_server_access();

-- Function to validate server load doesn't exceed capacity
CREATE OR REPLACE FUNCTION validate_server_capacity()
RETURNS TRIGGER AS $$
BEGIN
    -- Ensure current connections don't exceed maximum
    IF NEW.current_connections > NEW.max_connections THEN
        RAISE EXCEPTION 'Current connections (%) cannot exceed max connections (%)', 
            NEW.current_connections, NEW.max_connections;
    END IF;
    
    -- Ensure load percentage is reasonable
    IF NEW.current_load > 100 THEN
        RAISE EXCEPTION 'Server load cannot exceed 100%';
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER hardware_capacity_trigger
    BEFORE INSERT OR UPDATE ON hardware
    FOR EACH ROW
    EXECUTE FUNCTION validate_server_capacity();

-- Create view for server performance metrics
CREATE OR REPLACE VIEW server_performance AS
SELECT 
    h.server_id,
    h.name,
    h.user_id,
    u.login as owner_login,
    h.cpu,
    h.ram,
    h.current_load,
    h.current_connections,
    h.max_connections,
    ROUND((h.current_connections::decimal / h.max_connections * 100), 2) as connection_usage_percent,
    h.firewall_level,
    h.online_status,
    h.last_access,
    h.ip_address
FROM hardware h
JOIN users u ON h.user_id = u.id
WHERE h.online_status = TRUE;

-- Add comprehensive comments
COMMENT ON TABLE hardware IS 'Server hardware specifications and performance metrics';
COMMENT ON COLUMN hardware.server_id IS 'Primary key - unique server identifier';
COMMENT ON COLUMN hardware.user_id IS 'Owner of this server (references users.id)';
COMMENT ON COLUMN hardware.cpu IS 'CPU processing power in GHz';
COMMENT ON COLUMN hardware.hdd IS 'Hard disk storage capacity in GB';
COMMENT ON COLUMN hardware.ram IS 'Random access memory capacity in MB';
COMMENT ON COLUMN hardware.net IS 'Network connection speed in Mbps';
COMMENT ON COLUMN hardware.is_npc IS 'True if server is owned by NPC/system';
COMMENT ON COLUMN hardware.firewall_level IS 'Firewall protection level (0-10)';
COMMENT ON COLUMN hardware.current_load IS 'Current CPU utilization percentage';
COMMENT ON COLUMN hardware.ip_address IS 'Server IP address for connections';

COMMENT ON VIEW server_performance IS 'Real-time view of online server performance and usage metrics';