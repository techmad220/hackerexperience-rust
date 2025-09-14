-- Create hardware_external table - External/NPC server hardware
-- For servers that are not owned by players but exist in the game world

CREATE TABLE IF NOT EXISTS hardware_external (
    server_id BIGSERIAL PRIMARY KEY,
    
    -- Server identification
    name VARCHAR(50) NOT NULL,
    description TEXT,
    server_type VARCHAR(20) NOT NULL DEFAULT 'public', -- public, corporate, government, etc.
    
    -- Network information
    ip_address INET NOT NULL UNIQUE,
    hostname VARCHAR(100),
    domain VARCHAR(100),
    
    -- Hardware specifications
    cpu DECIMAL(10,2) NOT NULL DEFAULT 1000.00,    -- CPU power in GHz
    hdd DECIMAL(10,2) NOT NULL DEFAULT 500.00,     -- Hard disk capacity in GB
    ram DECIMAL(10,2) NOT NULL DEFAULT 1024.00,    -- RAM capacity in MB
    net DECIMAL(10,2) NOT NULL DEFAULT 10.00,      -- Network speed in Mbps
    
    -- Security configuration
    firewall_level INTEGER NOT NULL DEFAULT 3,     -- Firewall protection level
    encryption_level INTEGER NOT NULL DEFAULT 2,   -- Encryption strength
    intrusion_detection BOOLEAN NOT NULL DEFAULT TRUE, -- IDS enabled
    requires_password BOOLEAN NOT NULL DEFAULT TRUE,   -- Password protection
    password_strength INTEGER NOT NULL DEFAULT 5,      -- Password complexity (1-10)
    
    -- Access control
    max_connections INTEGER NOT NULL DEFAULT 20,   -- Max simultaneous connections
    current_connections INTEGER NOT NULL DEFAULT 0, -- Current active connections
    public_access BOOLEAN NOT NULL DEFAULT FALSE,  -- Allows public access
    
    -- Financial information
    hack_reward BIGINT NOT NULL DEFAULT 5000,      -- Money reward for successful hack
    hack_penalty BIGINT NOT NULL DEFAULT 1000,     -- Penalty for failed hack
    
    -- Server status and behavior
    online_status BOOLEAN NOT NULL DEFAULT TRUE,   -- Server online/offline
    difficulty_level INTEGER NOT NULL DEFAULT 5,   -- Overall difficulty (1-10)
    uptime_percent DECIMAL(5,2) NOT NULL DEFAULT 99.5, -- Server reliability
    
    -- Geographic and organizational info
    organization VARCHAR(100),                      -- Owning organization
    location_country VARCHAR(2),                   -- Country code
    location_city VARCHAR(50),                     -- City name
    location_coords POINT,                         -- Geographic coordinates
    
    -- Log and monitoring
    logs_retention_days INTEGER NOT NULL DEFAULT 30, -- How long logs are kept
    monitoring_level INTEGER NOT NULL DEFAULT 3,      -- Monitoring intensity (1-5)
    
    -- Special flags and properties
    is_honeypot BOOLEAN NOT NULL DEFAULT FALSE,    -- Honeypot trap server
    is_secured BOOLEAN NOT NULL DEFAULT TRUE,      -- Has security measures
    has_backdoor BOOLEAN NOT NULL DEFAULT FALSE,   -- Has hidden backdoor
    backup_frequency INTEGER NOT NULL DEFAULT 24,  -- Backup frequency in hours
    
    -- Timing and activity
    last_backup TIMESTAMPTZ,                       -- Last backup time
    last_maintenance TIMESTAMPTZ,                  -- Last maintenance
    peak_hours_start TIME NOT NULL DEFAULT '09:00', -- Peak activity start
    peak_hours_end TIME NOT NULL DEFAULT '17:00',   -- Peak activity end
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Constraints
    CONSTRAINT chk_ext_cpu_positive CHECK (cpu > 0),
    CONSTRAINT chk_ext_hdd_positive CHECK (hdd > 0),
    CONSTRAINT chk_ext_ram_positive CHECK (ram > 0),
    CONSTRAINT chk_ext_net_positive CHECK (net > 0),
    CONSTRAINT chk_ext_firewall_level CHECK (firewall_level >= 0 AND firewall_level <= 10),
    CONSTRAINT chk_ext_encryption_level CHECK (encryption_level >= 0 AND encryption_level <= 10),
    CONSTRAINT chk_ext_difficulty_level CHECK (difficulty_level >= 1 AND difficulty_level <= 10),
    CONSTRAINT chk_ext_password_strength CHECK (password_strength >= 1 AND password_strength <= 10),
    CONSTRAINT chk_ext_monitoring_level CHECK (monitoring_level >= 1 AND monitoring_level <= 5),
    CONSTRAINT chk_ext_uptime CHECK (uptime_percent >= 0 AND uptime_percent <= 100),
    CONSTRAINT chk_ext_connections CHECK (current_connections >= 0 AND current_connections <= max_connections),
    CONSTRAINT chk_ext_logs_retention CHECK (logs_retention_days >= 0),
    CONSTRAINT chk_ext_backup_frequency CHECK (backup_frequency >= 1)
);

-- Create indexes for performance and common queries
CREATE INDEX IF NOT EXISTS idx_hardware_external_ip ON hardware_external(ip_address);
CREATE INDEX IF NOT EXISTS idx_hardware_external_type ON hardware_external(server_type);
CREATE INDEX IF NOT EXISTS idx_hardware_external_online ON hardware_external(online_status);
CREATE INDEX IF NOT EXISTS idx_hardware_external_difficulty ON hardware_external(difficulty_level);
CREATE INDEX IF NOT EXISTS idx_hardware_external_organization ON hardware_external(organization);
CREATE INDEX IF NOT EXISTS idx_hardware_external_public_access ON hardware_external(public_access);
CREATE INDEX IF NOT EXISTS idx_hardware_external_location ON hardware_external(location_country, location_city);
CREATE INDEX IF NOT EXISTS idx_hardware_external_firewall ON hardware_external(firewall_level);
CREATE INDEX IF NOT EXISTS idx_hardware_external_coords ON hardware_external USING GIST(location_coords);

-- Create trigger for updated_at
CREATE TRIGGER hardware_external_updated_at_trigger
    BEFORE UPDATE ON hardware_external
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to calculate server vulnerability score
CREATE OR REPLACE FUNCTION calculate_vulnerability_score(
    fw_level INTEGER,
    enc_level INTEGER, 
    pwd_strength INTEGER,
    has_ids BOOLEAN,
    is_honeypot BOOLEAN
) RETURNS DECIMAL(4,2) AS $$
DECLARE
    base_score DECIMAL(4,2) := 10.0;
    vulnerability DECIMAL(4,2);
BEGIN
    -- Start with base vulnerability
    vulnerability := base_score;
    
    -- Reduce vulnerability based on security measures
    vulnerability := vulnerability - (fw_level * 0.8);        -- Firewall impact
    vulnerability := vulnerability - (enc_level * 0.6);       -- Encryption impact
    vulnerability := vulnerability - (pwd_strength * 0.4);    -- Password impact
    
    -- IDS reduces vulnerability
    IF has_ids THEN
        vulnerability := vulnerability - 1.0;
    END IF;
    
    -- Honeypots are special case - appear vulnerable but are traps
    IF is_honeypot THEN
        vulnerability := vulnerability + 2.0;  -- Appear more vulnerable
    END IF;
    
    -- Ensure score is within reasonable bounds
    vulnerability := GREATEST(0.1, LEAST(10.0, vulnerability));
    
    RETURN vulnerability;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Create materialized view for hackable servers
CREATE MATERIALIZED VIEW hackable_servers AS
SELECT 
    he.server_id,
    he.name,
    he.ip_address,
    he.server_type,
    he.organization,
    he.difficulty_level,
    he.hack_reward,
    he.firewall_level,
    he.encryption_level,
    he.requires_password,
    he.password_strength,
    calculate_vulnerability_score(
        he.firewall_level, 
        he.encryption_level, 
        he.password_strength,
        he.intrusion_detection,
        he.is_honeypot
    ) as vulnerability_score,
    he.is_honeypot,
    he.public_access,
    he.online_status,
    he.current_connections,
    he.max_connections,
    he.location_country,
    he.location_city
FROM hardware_external he
WHERE he.online_status = TRUE
ORDER BY he.difficulty_level, vulnerability_score DESC;

-- Create index on materialized view
CREATE UNIQUE INDEX idx_hackable_servers_id ON hackable_servers(server_id);
CREATE INDEX idx_hackable_servers_difficulty ON hackable_servers(difficulty_level);
CREATE INDEX idx_hackable_servers_vulnerability ON hackable_servers(vulnerability_score);
CREATE INDEX idx_hackable_servers_reward ON hackable_servers(hack_reward);

-- Function to refresh hackable servers view
CREATE OR REPLACE FUNCTION refresh_hackable_servers()
RETURNS VOID AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY hackable_servers;
END;
$$ LANGUAGE plpgsql;

-- Add comprehensive comments
COMMENT ON TABLE hardware_external IS 'External/NPC servers available for hacking in the game world';
COMMENT ON COLUMN hardware_external.server_type IS 'Type of server: public, corporate, government, military, etc.';
COMMENT ON COLUMN hardware_external.difficulty_level IS 'Overall hacking difficulty from 1 (easy) to 10 (expert)';
COMMENT ON COLUMN hardware_external.hack_reward IS 'Money reward for successfully hacking this server';
COMMENT ON COLUMN hardware_external.is_honeypot IS 'True if server is a trap/honeypot for catching hackers';
COMMENT ON COLUMN hardware_external.vulnerability_score IS 'Calculated vulnerability based on security measures';
COMMENT ON COLUMN hardware_external.monitoring_level IS 'How closely server activity is monitored (1-5)';
COMMENT ON COLUMN hardware_external.logs_retention_days IS 'Number of days server keeps access logs';

COMMENT ON MATERIALIZED VIEW hackable_servers IS 'Optimized view of servers available for hacking with calculated metrics';
COMMENT ON FUNCTION calculate_vulnerability_score IS 'Calculates server vulnerability score based on security measures';
COMMENT ON FUNCTION refresh_hackable_servers IS 'Refreshes the materialized view of hackable servers';