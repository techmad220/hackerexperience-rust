-- Create software table - Player software, tools, and files
-- Based on original software table with extensive enhancements

CREATE TABLE IF NOT EXISTS software (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    
    -- Software identification
    name VARCHAR(50) NOT NULL,
    software_type VARCHAR(20) NOT NULL,           -- Type: virus, tool, cracker, etc.
    category VARCHAR(20) NOT NULL DEFAULT 'tool', -- Category: offensive, defensive, utility
    version VARCHAR(10) NOT NULL DEFAULT '1.0',
    
    -- File properties
    file_size INTEGER NOT NULL DEFAULT 0,         -- Size in MB
    checksum VARCHAR(64),                         -- File integrity hash
    file_extension VARCHAR(10) DEFAULT 'exe',     -- File extension
    
    -- Location and installation
    location INET NOT NULL,                       -- IP address where installed
    install_path VARCHAR(255) DEFAULT '/usr/bin', -- Installation directory
    folder_id BIGINT,                             -- Folder/directory ID
    installed_at TIMESTAMPTZ,                     -- Installation timestamp
    
    -- Status and behavior
    is_running BOOLEAN NOT NULL DEFAULT FALSE,    -- Currently executing
    is_hidden BOOLEAN NOT NULL DEFAULT FALSE,     -- Hidden from basic scans
    is_encrypted BOOLEAN NOT NULL DEFAULT FALSE,  -- File is encrypted
    is_compressed BOOLEAN NOT NULL DEFAULT FALSE, -- File is compressed
    
    -- Software capabilities
    effectiveness INTEGER NOT NULL DEFAULT 50,    -- Effectiveness rating (0-100)
    stealth_rating INTEGER NOT NULL DEFAULT 30,   -- Stealth capability (0-100)
    reliability INTEGER NOT NULL DEFAULT 80,      -- Reliability rating (0-100)
    resource_usage INTEGER NOT NULL DEFAULT 10,   -- CPU/RAM usage intensity
    
    -- Requirements and dependencies
    min_cpu INTEGER NOT NULL DEFAULT 100,         -- Minimum CPU required
    min_ram INTEGER NOT NULL DEFAULT 64,          -- Minimum RAM required (MB)
    required_skills JSONB DEFAULT '[]',           -- Required skills as JSON array
    dependencies JSONB DEFAULT '[]',              -- Software dependencies
    
    -- Usage and statistics
    usage_count INTEGER NOT NULL DEFAULT 0,       -- How many times used
    success_rate DECIMAL(5,2) DEFAULT 0.00,      -- Historical success rate
    last_used TIMESTAMPTZ,                        -- Last usage timestamp
    total_runtime INTEGER DEFAULT 0,              -- Total runtime in seconds
    
    -- Security and detection
    signature VARCHAR(128),                       -- AV signature (if detected)
    detection_level INTEGER NOT NULL DEFAULT 0,   -- How detectable (0-10)
    quarantined BOOLEAN NOT NULL DEFAULT FALSE,   -- Quarantined by AV
    
    -- Market and value
    market_value BIGINT NOT NULL DEFAULT 1000,    -- Current market value
    original_cost BIGINT NOT NULL DEFAULT 1000,   -- Original purchase cost
    is_cracked BOOLEAN NOT NULL DEFAULT FALSE,     -- Cracked/pirated software
    license_key VARCHAR(50),                       -- Software license key
    
    -- Metadata and configuration
    description TEXT,                              -- Software description
    configuration JSONB DEFAULT '{}',             -- Configuration settings
    custom_properties JSONB DEFAULT '{}',         -- Custom properties
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (folder_id) REFERENCES software_folders(id) ON DELETE SET NULL,
    
    -- Check constraints
    CONSTRAINT chk_software_effectiveness CHECK (effectiveness >= 0 AND effectiveness <= 100),
    CONSTRAINT chk_software_stealth CHECK (stealth_rating >= 0 AND stealth_rating <= 100),
    CONSTRAINT chk_software_reliability CHECK (reliability >= 0 AND reliability <= 100),
    CONSTRAINT chk_software_resource_usage CHECK (resource_usage >= 0 AND resource_usage <= 100),
    CONSTRAINT chk_software_detection CHECK (detection_level >= 0 AND detection_level <= 10),
    CONSTRAINT chk_software_success_rate CHECK (success_rate >= 0.00 AND success_rate <= 100.00),
    CONSTRAINT chk_software_file_size CHECK (file_size >= 0),
    CONSTRAINT chk_software_usage_count CHECK (usage_count >= 0),
    CONSTRAINT chk_software_runtime CHECK (total_runtime IS NULL OR total_runtime >= 0),
    CONSTRAINT chk_software_cpu_req CHECK (min_cpu > 0),
    CONSTRAINT chk_software_ram_req CHECK (min_ram > 0),
    CONSTRAINT chk_software_values CHECK (market_value >= 0 AND original_cost >= 0)
);

-- Create software folders table first (referenced by software table)
CREATE TABLE IF NOT EXISTS software_folders (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    location INET NOT NULL,                       -- IP where folder exists
    name VARCHAR(100) NOT NULL,                   -- Folder name
    path VARCHAR(255) NOT NULL,                   -- Full path
    parent_folder_id BIGINT,                      -- Parent folder
    is_hidden BOOLEAN NOT NULL DEFAULT FALSE,     -- Hidden folder
    is_system BOOLEAN NOT NULL DEFAULT FALSE,     -- System folder
    permissions INTEGER NOT NULL DEFAULT 755,     -- Unix-style permissions
    size_limit BIGINT,                            -- Size limit in bytes
    current_size BIGINT NOT NULL DEFAULT 0,      -- Current folder size
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_folder_id) REFERENCES software_folders(id) ON DELETE CASCADE,
    
    UNIQUE (user_id, location, path)
);

-- Create indexes for software table
CREATE INDEX IF NOT EXISTS idx_software_user_id ON software(user_id);
CREATE INDEX IF NOT EXISTS idx_software_location ON software(location);
CREATE INDEX IF NOT EXISTS idx_software_type ON software(software_type);
CREATE INDEX IF NOT EXISTS idx_software_category ON software(category);
CREATE INDEX IF NOT EXISTS idx_software_running ON software(is_running);
CREATE INDEX IF NOT EXISTS idx_software_hidden ON software(is_hidden);
CREATE INDEX IF NOT EXISTS idx_software_folder ON software(folder_id);
CREATE INDEX IF NOT EXISTS idx_software_quarantined ON software(quarantined);
CREATE INDEX IF NOT EXISTS idx_software_effectiveness ON software(effectiveness);
CREATE INDEX IF NOT EXISTS idx_software_last_used ON software(last_used);
CREATE INDEX IF NOT EXISTS idx_software_market_value ON software(market_value);
CREATE INDEX IF NOT EXISTS idx_software_user_location ON software(user_id, location);
CREATE INDEX IF NOT EXISTS idx_software_type_effectiveness ON software(software_type, effectiveness);

-- Create indexes for software folders
CREATE INDEX IF NOT EXISTS idx_software_folders_user_id ON software_folders(user_id);
CREATE INDEX IF NOT EXISTS idx_software_folders_location ON software_folders(location);
CREATE INDEX IF NOT EXISTS idx_software_folders_parent ON software_folders(parent_folder_id);
CREATE INDEX IF NOT EXISTS idx_software_folders_hidden ON software_folders(is_hidden);

-- GIN indexes for JSONB columns
CREATE INDEX IF NOT EXISTS idx_software_dependencies ON software USING GIN(dependencies);
CREATE INDEX IF NOT EXISTS idx_software_skills ON software USING GIN(required_skills);
CREATE INDEX IF NOT EXISTS idx_software_config ON software USING GIN(configuration);

-- Create triggers for updated_at
CREATE TRIGGER software_updated_at_trigger
    BEFORE UPDATE ON software
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER software_folders_updated_at_trigger
    BEFORE UPDATE ON software_folders
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to update usage statistics
CREATE OR REPLACE FUNCTION update_software_usage(
    software_id BIGINT,
    execution_time INTEGER DEFAULT NULL,
    was_successful BOOLEAN DEFAULT NULL
) RETURNS VOID AS $$
DECLARE
    current_success_rate DECIMAL(5,2);
    current_usage_count INTEGER;
    new_success_rate DECIMAL(5,2);
BEGIN
    -- Get current statistics
    SELECT usage_count, success_rate 
    INTO current_usage_count, current_success_rate
    FROM software WHERE id = software_id;
    
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Software % not found', software_id;
    END IF;
    
    -- Calculate new success rate if success status provided
    IF was_successful IS NOT NULL THEN
        IF current_usage_count = 0 THEN
            new_success_rate := CASE WHEN was_successful THEN 100.00 ELSE 0.00 END;
        ELSE
            -- Weighted average of previous success rate and new result
            new_success_rate := (
                (current_success_rate * current_usage_count) + 
                (CASE WHEN was_successful THEN 100.0 ELSE 0.0 END)
            ) / (current_usage_count + 1);
        END IF;
    ELSE
        new_success_rate := current_success_rate;
    END IF;
    
    -- Update software statistics
    UPDATE software SET
        usage_count = usage_count + 1,
        success_rate = new_success_rate,
        last_used = CURRENT_TIMESTAMP,
        total_runtime = COALESCE(total_runtime, 0) + COALESCE(execution_time, 0),
        updated_at = CURRENT_TIMESTAMP
    WHERE id = software_id;
END;
$$ LANGUAGE plpgsql;

-- Function to install software
CREATE OR REPLACE FUNCTION install_software(
    software_id BIGINT,
    target_location INET,
    target_folder_id BIGINT DEFAULT NULL
) RETURNS VOID AS $$
DECLARE
    software_record RECORD;
    available_space BIGINT;
BEGIN
    -- Get software details
    SELECT * INTO software_record FROM software WHERE id = software_id;
    
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Software % not found', software_id;
    END IF;
    
    -- Check if already installed at this location
    IF software_record.location = target_location AND software_record.installed_at IS NOT NULL THEN
        RAISE EXCEPTION 'Software already installed at location %', target_location;
    END IF;
    
    -- Check available space (simplified check)
    -- In real implementation, would check actual disk space
    available_space := 10000; -- 10GB available
    
    IF software_record.file_size > available_space THEN
        RAISE EXCEPTION 'Insufficient disk space. Required: %MB, Available: %MB', 
            software_record.file_size, available_space;
    END IF;
    
    -- Install the software
    UPDATE software SET
        location = target_location,
        folder_id = target_folder_id,
        installed_at = CURRENT_TIMESTAMP,
        updated_at = CURRENT_TIMESTAMP
    WHERE id = software_id;
    
    -- Update folder size if applicable
    IF target_folder_id IS NOT NULL THEN
        UPDATE software_folders SET
            current_size = current_size + software_record.file_size * 1024 * 1024, -- Convert MB to bytes
            updated_at = CURRENT_TIMESTAMP
        WHERE id = target_folder_id;
    END IF;
    
    -- Log the installation
    INSERT INTO software_logs (software_id, event_type, message)
    VALUES (software_id, 'INSTALLED', 'Software installed to ' || target_location::TEXT);
END;
$$ LANGUAGE plpgsql;

-- Create software logs table
CREATE TABLE IF NOT EXISTS software_logs (
    id BIGSERIAL PRIMARY KEY,
    software_id BIGINT NOT NULL,
    event_type VARCHAR(20) NOT NULL,              -- CREATED, INSTALLED, EXECUTED, UPDATED, DELETED
    message TEXT,
    details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (software_id) REFERENCES software(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_software_logs_software_id ON software_logs(software_id);
CREATE INDEX IF NOT EXISTS idx_software_logs_event_type ON software_logs(event_type);
CREATE INDEX IF NOT EXISTS idx_software_logs_created_at ON software_logs(created_at);

-- Create materialized view for software catalog
CREATE MATERIALIZED VIEW software_catalog AS
SELECT 
    s.software_type,
    s.category,
    COUNT(*) as total_count,
    AVG(s.effectiveness) as avg_effectiveness,
    AVG(s.stealth_rating) as avg_stealth,
    AVG(s.reliability) as avg_reliability,
    AVG(s.market_value) as avg_market_value,
    MIN(s.market_value) as min_price,
    MAX(s.market_value) as max_price,
    COUNT(CASE WHEN s.is_running THEN 1 END) as currently_running,
    SUM(s.usage_count) as total_usage
FROM software s
WHERE s.quarantined = FALSE
GROUP BY s.software_type, s.category
ORDER BY total_count DESC;

CREATE UNIQUE INDEX idx_software_catalog_type_category ON software_catalog(software_type, category);

-- Function to refresh software catalog
CREATE OR REPLACE FUNCTION refresh_software_catalog()
RETURNS VOID AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY software_catalog;
END;
$$ LANGUAGE plpgsql;

-- Add comprehensive comments
COMMENT ON TABLE software IS 'Player software, tools, viruses and files with comprehensive metadata';
COMMENT ON COLUMN software.software_type IS 'Type of software: virus, cracker, ddos, scanner, etc.';
COMMENT ON COLUMN software.effectiveness IS 'How effective the software is (0-100)';
COMMENT ON COLUMN software.stealth_rating IS 'How well software avoids detection (0-100)';
COMMENT ON COLUMN software.detection_level IS 'How detectable by antivirus (0=undetectable, 10=obvious)';
COMMENT ON COLUMN software.required_skills IS 'JSON array of required skills to use effectively';
COMMENT ON COLUMN software.dependencies IS 'JSON array of required software dependencies';
COMMENT ON COLUMN software.success_rate IS 'Historical success rate percentage';

COMMENT ON TABLE software_folders IS 'Folder structure for organizing software installations';
COMMENT ON TABLE software_logs IS 'Detailed logging of software lifecycle events';
COMMENT ON MATERIALIZED VIEW software_catalog IS 'Statistical overview of available software by type and category';

COMMENT ON FUNCTION update_software_usage IS 'Updates software usage statistics and success rates';
COMMENT ON FUNCTION install_software IS 'Installs software to target location with space and dependency checks';
COMMENT ON FUNCTION refresh_software_catalog IS 'Refreshes the materialized view of software statistics';