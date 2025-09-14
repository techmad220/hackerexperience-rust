-- Create comprehensive network and connection system tables
-- Handles network topology, connections, routing, and network security

-- Network nodes table - All network-addressable entities
CREATE TABLE IF NOT EXISTS network_nodes (
    id BIGSERIAL PRIMARY KEY,
    
    -- Network identification
    ip_address INET NOT NULL UNIQUE,              -- IPv4/IPv6 address
    hostname VARCHAR(255),                        -- DNS hostname
    domain VARCHAR(100),                          -- Domain name
    mac_address MACADDR,                          -- MAC address
    
    -- Node type and properties
    node_type VARCHAR(20) NOT NULL,               -- server, router, switch, firewall, proxy
    node_subtype VARCHAR(30),                     -- desktop, laptop, mainframe, etc.
    is_virtual BOOLEAN NOT NULL DEFAULT FALSE,    -- Virtual machine/container
    is_public BOOLEAN NOT NULL DEFAULT FALSE,     -- Publicly accessible
    
    -- Owner and control
    owner_user_id BIGINT,                         -- User who owns this node
    owner_type VARCHAR(20) DEFAULT 'user',        -- user, npc, system, organization
    organization VARCHAR(100),                    -- Owning organization
    
    -- Geographic location
    geo_location POINT,                           -- Geographic coordinates
    country_code VARCHAR(2),                      -- Country code
    city VARCHAR(50),                             -- City name
    isp VARCHAR(100),                             -- Internet service provider
    as_number INTEGER,                            -- Autonomous system number
    
    -- Network properties
    bandwidth_mbps DECIMAL(10,2) DEFAULT 100.0,   -- Bandwidth in Mbps
    latency_ms INTEGER DEFAULT 50,                -- Base latency in milliseconds
    packet_loss DECIMAL(5,4) DEFAULT 0.0001,     -- Packet loss percentage
    uptime_percentage DECIMAL(5,2) DEFAULT 99.9,  -- Uptime percentage
    
    -- Security and access
    firewall_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    firewall_level INTEGER NOT NULL DEFAULT 3,    -- Firewall strength (0-10)
    intrusion_detection BOOLEAN DEFAULT TRUE,     -- IDS enabled
    encryption_level INTEGER DEFAULT 2,           -- Encryption strength (0-10)
    requires_authentication BOOLEAN DEFAULT TRUE,
    
    -- Status and monitoring
    online_status BOOLEAN NOT NULL DEFAULT TRUE,  -- Currently online
    last_seen TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    last_scanned TIMESTAMPTZ,                     -- Last security scan
    threat_level INTEGER DEFAULT 0,               -- Current threat level (0-10)
    
    -- Hardware and capacity
    cpu_cores INTEGER DEFAULT 2,
    ram_gb INTEGER DEFAULT 4,
    storage_gb INTEGER DEFAULT 100,
    max_connections INTEGER DEFAULT 100,
    current_connections INTEGER DEFAULT 0,
    
    -- Network services
    open_ports JSONB DEFAULT '[]',                -- Array of open ports
    running_services JSONB DEFAULT '[]',          -- Array of running services
    software_versions JSONB DEFAULT '{}',         -- Software version information
    
    -- Administrative information
    admin_contact VARCHAR(255),                   -- Administrative contact
    technical_contact VARCHAR(255),               -- Technical contact
    description TEXT,                             -- Node description
    notes TEXT,                                   -- Administrative notes
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (owner_user_id) REFERENCES users(id) ON DELETE SET NULL,
    
    -- Check constraints
    CONSTRAINT chk_network_nodes_firewall_level CHECK (firewall_level >= 0 AND firewall_level <= 10),
    CONSTRAINT chk_network_nodes_encryption_level CHECK (encryption_level >= 0 AND encryption_level <= 10),
    CONSTRAINT chk_network_nodes_threat_level CHECK (threat_level >= 0 AND threat_level <= 10),
    CONSTRAINT chk_network_nodes_bandwidth CHECK (bandwidth_mbps > 0),
    CONSTRAINT chk_network_nodes_latency CHECK (latency_ms > 0),
    CONSTRAINT chk_network_nodes_packet_loss CHECK (packet_loss >= 0 AND packet_loss <= 1),
    CONSTRAINT chk_network_nodes_uptime CHECK (uptime_percentage >= 0 AND uptime_percentage <= 100),
    CONSTRAINT chk_network_nodes_hardware CHECK (
        cpu_cores > 0 AND ram_gb > 0 AND storage_gb > 0 AND
        max_connections > 0 AND current_connections >= 0 AND
        current_connections <= max_connections
    )
);

-- Network connections table - Active connections between nodes
CREATE TABLE IF NOT EXISTS network_connections (
    id BIGSERIAL PRIMARY KEY,
    
    -- Connection endpoints
    source_node_id BIGINT NOT NULL,               -- Source network node
    destination_node_id BIGINT NOT NULL,          -- Destination network node
    source_ip INET NOT NULL,                      -- Source IP address
    destination_ip INET NOT NULL,                 -- Destination IP address
    source_port INTEGER,                          -- Source port
    destination_port INTEGER NOT NULL,            -- Destination port
    
    -- Connection properties
    connection_type VARCHAR(20) NOT NULL,         -- tcp, udp, ssh, http, https, ftp, etc.
    protocol VARCHAR(10) NOT NULL DEFAULT 'TCP',  -- TCP, UDP, ICMP
    connection_state VARCHAR(20) DEFAULT 'ESTABLISHED', -- ESTABLISHED, SYN_SENT, CLOSE_WAIT, etc.
    
    -- User and session context
    user_id BIGINT,                               -- User who initiated connection
    session_id BIGINT,                            -- Related session
    process_id BIGINT,                            -- Related process
    
    -- Connection metrics
    bytes_sent BIGINT DEFAULT 0,                  -- Bytes sent
    bytes_received BIGINT DEFAULT 0,              -- Bytes received
    packets_sent INTEGER DEFAULT 0,               -- Packets sent
    packets_received INTEGER DEFAULT 0,           -- Packets received
    
    -- Quality and performance
    latency_ms INTEGER,                           -- Current latency
    bandwidth_used_mbps DECIMAL(8,2) DEFAULT 0,   -- Bandwidth utilization
    packet_loss_rate DECIMAL(5,4) DEFAULT 0,     -- Packet loss rate
    
    -- Security and monitoring
    is_encrypted BOOLEAN DEFAULT FALSE,           -- Connection is encrypted
    encryption_method VARCHAR(20),               -- Encryption algorithm used
    is_monitored BOOLEAN DEFAULT FALSE,           -- Connection is being monitored
    is_suspicious BOOLEAN DEFAULT FALSE,          -- Flagged as suspicious
    
    -- Connection lifetime
    established_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_activity TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    closed_at TIMESTAMPTZ,                       -- When connection was closed
    timeout_at TIMESTAMPTZ,                      -- When connection times out
    
    -- Connection metadata
    user_agent TEXT,                              -- User agent if HTTP
    referer TEXT,                                 -- HTTP referer
    connection_data JSONB DEFAULT '{}',           -- Additional connection data
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (source_node_id) REFERENCES network_nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (destination_node_id) REFERENCES network_nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE SET NULL,
    FOREIGN KEY (process_id) REFERENCES processes(pid) ON DELETE SET NULL,
    
    -- Check constraints
    CONSTRAINT chk_network_connections_ports CHECK (
        (source_port IS NULL OR (source_port >= 1 AND source_port <= 65535)) AND
        destination_port >= 1 AND destination_port <= 65535
    ),
    CONSTRAINT chk_network_connections_bytes CHECK (bytes_sent >= 0 AND bytes_received >= 0),
    CONSTRAINT chk_network_connections_packets CHECK (packets_sent >= 0 AND packets_received >= 0),
    CONSTRAINT chk_network_connections_bandwidth CHECK (bandwidth_used_mbps >= 0),
    CONSTRAINT chk_network_connections_timing CHECK (
        (closed_at IS NULL OR closed_at >= established_at) AND
        (timeout_at IS NULL OR timeout_at >= established_at)
    )
);

-- Network routes table - Routing table entries
CREATE TABLE IF NOT EXISTS network_routes (
    id BIGSERIAL PRIMARY KEY,
    
    -- Route definition
    destination_network CIDR NOT NULL,           -- Destination network/subnet
    gateway_ip INET,                             -- Gateway IP (NULL for direct)
    interface_name VARCHAR(20),                  -- Network interface
    metric INTEGER DEFAULT 1,                    -- Route metric/priority
    
    -- Route properties
    route_type VARCHAR(20) DEFAULT 'static',     -- static, dynamic, default, blackhole
    is_active BOOLEAN NOT NULL DEFAULT TRUE,     -- Route is active
    is_default BOOLEAN DEFAULT FALSE,            -- Default route
    
    -- Administrative distance and preferences
    admin_distance INTEGER DEFAULT 120,          -- Administrative distance
    preference INTEGER DEFAULT 100,              -- Route preference
    
    -- Route source and management
    source_protocol VARCHAR(20) DEFAULT 'manual', -- manual, ospf, bgp, rip, etc.
    learned_from_ip INET,                        -- IP we learned route from
    node_id BIGINT,                              -- Node this route belongs to
    
    -- Route lifetime and updates
    installed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ,                      -- When route expires
    
    -- Route metadata
    description TEXT,                             -- Route description
    route_data JSONB DEFAULT '{}',               -- Additional route data
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (node_id) REFERENCES network_nodes(id) ON DELETE CASCADE,
    
    -- Check constraints
    CONSTRAINT chk_network_routes_metric CHECK (metric >= 0),
    CONSTRAINT chk_network_routes_admin_distance CHECK (admin_distance >= 0 AND admin_distance <= 255),
    CONSTRAINT chk_network_routes_preference CHECK (preference >= 0)
);

-- Network scan results table - Results of network scans
CREATE TABLE IF NOT EXISTS network_scan_results (
    id BIGSERIAL PRIMARY KEY,
    
    -- Scan identification
    scan_id VARCHAR(50) NOT NULL,                 -- Unique scan identifier
    scanner_user_id BIGINT NOT NULL,              -- User who performed scan
    target_node_id BIGINT,                        -- Target node if specific
    target_ip INET NOT NULL,                      -- Target IP address
    
    -- Scan parameters
    scan_type VARCHAR(20) NOT NULL,               -- port, vulnerability, ping, traceroute
    scan_method VARCHAR(30),                      -- tcp_syn, udp, stealth, etc.
    port_range VARCHAR(50),                       -- Port range scanned
    
    -- Scan results
    scan_status VARCHAR(20) NOT NULL,             -- completed, failed, timeout, interrupted
    ports_found JSONB DEFAULT '[]',               -- Discovered open ports
    services_found JSONB DEFAULT '[]',            -- Discovered services
    vulnerabilities JSONB DEFAULT '[]',           -- Found vulnerabilities
    
    -- Scan metadata
    scan_duration_ms INTEGER,                     -- Scan duration
    packets_sent INTEGER DEFAULT 0,               -- Packets sent during scan
    responses_received INTEGER DEFAULT 0,         -- Responses received
    
    -- Detection and stealth
    detected BOOLEAN DEFAULT FALSE,               -- Scan was detected
    detection_method VARCHAR(50),                 -- How scan was detected
    stealth_level INTEGER DEFAULT 1,             -- Stealth level used (1-10)
    
    -- Scan timing
    started_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMPTZ,                    -- When scan completed
    
    -- Additional data
    scan_data JSONB DEFAULT '{}',                 -- Additional scan data
    error_message TEXT,                           -- Error message if failed
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (scanner_user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (target_node_id) REFERENCES network_nodes(id) ON DELETE SET NULL,
    
    -- Check constraints
    CONSTRAINT chk_scan_results_stealth_level CHECK (stealth_level >= 1 AND stealth_level <= 10),
    CONSTRAINT chk_scan_results_packets CHECK (packets_sent >= 0 AND responses_received >= 0),
    CONSTRAINT chk_scan_results_duration CHECK (scan_duration_ms IS NULL OR scan_duration_ms >= 0),
    CONSTRAINT chk_scan_results_timing CHECK (completed_at IS NULL OR completed_at >= started_at)
);

-- Network traffic logs table - Network traffic monitoring
CREATE TABLE IF NOT EXISTS network_traffic_logs (
    id BIGSERIAL PRIMARY KEY,
    
    -- Traffic flow identification
    source_ip INET NOT NULL,                      -- Source IP
    destination_ip INET NOT NULL,                 -- Destination IP
    source_port INTEGER,                          -- Source port
    destination_port INTEGER,                     -- Destination port
    protocol VARCHAR(10) NOT NULL,                -- TCP, UDP, ICMP
    
    -- Traffic volume
    bytes_transferred BIGINT NOT NULL DEFAULT 0,  -- Bytes transferred
    packets_count INTEGER NOT NULL DEFAULT 0,     -- Number of packets
    
    -- Traffic characteristics
    traffic_type VARCHAR(20),                     -- web, email, file_transfer, game, etc.
    application VARCHAR(50),                      -- Application generating traffic
    is_encrypted BOOLEAN DEFAULT FALSE,           -- Traffic is encrypted
    
    -- Quality metrics
    average_latency_ms INTEGER,                   -- Average latency
    packet_loss_rate DECIMAL(5,4) DEFAULT 0,     -- Packet loss rate
    jitter_ms INTEGER DEFAULT 0,                  -- Network jitter
    
    -- Security analysis
    is_suspicious BOOLEAN DEFAULT FALSE,          -- Flagged as suspicious
    threat_score INTEGER DEFAULT 0,               -- Threat score (0-10)
    blocked BOOLEAN DEFAULT FALSE,                -- Traffic was blocked
    
    -- Timing information
    flow_start TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    flow_end TIMESTAMPTZ,                         -- When flow ended
    duration_ms INTEGER,                          -- Flow duration
    
    -- Context information
    user_id BIGINT,                               -- Associated user
    session_id BIGINT,                            -- Associated session
    
    -- Additional metadata
    traffic_data JSONB DEFAULT '{}',              -- Additional traffic data
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE SET NULL,
    
    -- Check constraints
    CONSTRAINT chk_traffic_logs_bytes CHECK (bytes_transferred >= 0),
    CONSTRAINT chk_traffic_logs_packets CHECK (packets_count >= 0),
    CONSTRAINT chk_traffic_logs_threat_score CHECK (threat_score >= 0 AND threat_score <= 10),
    CONSTRAINT chk_traffic_logs_timing CHECK (flow_end IS NULL OR flow_end >= flow_start)
);

-- Create comprehensive indexes
-- Network nodes indexes
CREATE INDEX IF NOT EXISTS idx_network_nodes_ip ON network_nodes(ip_address);
CREATE INDEX IF NOT EXISTS idx_network_nodes_hostname ON network_nodes(hostname);
CREATE INDEX IF NOT EXISTS idx_network_nodes_owner ON network_nodes(owner_user_id);
CREATE INDEX IF NOT EXISTS idx_network_nodes_type ON network_nodes(node_type);
CREATE INDEX IF NOT EXISTS idx_network_nodes_online ON network_nodes(online_status);
CREATE INDEX IF NOT EXISTS idx_network_nodes_public ON network_nodes(is_public);
CREATE INDEX IF NOT EXISTS idx_network_nodes_threat_level ON network_nodes(threat_level);
CREATE INDEX IF NOT EXISTS idx_network_nodes_country ON network_nodes(country_code);

-- Network connections indexes
CREATE INDEX IF NOT EXISTS idx_network_connections_source ON network_connections(source_node_id);
CREATE INDEX IF NOT EXISTS idx_network_connections_destination ON network_connections(destination_node_id);
CREATE INDEX IF NOT EXISTS idx_network_connections_source_ip ON network_connections(source_ip);
CREATE INDEX IF NOT EXISTS idx_network_connections_dest_ip ON network_connections(destination_ip);
CREATE INDEX IF NOT EXISTS idx_network_connections_user ON network_connections(user_id);
CREATE INDEX IF NOT EXISTS idx_network_connections_type ON network_connections(connection_type);
CREATE INDEX IF NOT EXISTS idx_network_connections_state ON network_connections(connection_state);
CREATE INDEX IF NOT EXISTS idx_network_connections_established ON network_connections(established_at);
CREATE INDEX IF NOT EXISTS idx_network_connections_activity ON network_connections(last_activity);

-- Network routes indexes
CREATE INDEX IF NOT EXISTS idx_network_routes_destination ON network_routes(destination_network);
CREATE INDEX IF NOT EXISTS idx_network_routes_gateway ON network_routes(gateway_ip);
CREATE INDEX IF NOT EXISTS idx_network_routes_node ON network_routes(node_id);
CREATE INDEX IF NOT EXISTS idx_network_routes_active ON network_routes(is_active);
CREATE INDEX IF NOT EXISTS idx_network_routes_default ON network_routes(is_default);

-- Scan results indexes
CREATE INDEX IF NOT EXISTS idx_scan_results_scan_id ON network_scan_results(scan_id);
CREATE INDEX IF NOT EXISTS idx_scan_results_scanner ON network_scan_results(scanner_user_id);
CREATE INDEX IF NOT EXISTS idx_scan_results_target_ip ON network_scan_results(target_ip);
CREATE INDEX IF NOT EXISTS idx_scan_results_type ON network_scan_results(scan_type);
CREATE INDEX IF NOT EXISTS idx_scan_results_status ON network_scan_results(scan_status);
CREATE INDEX IF NOT EXISTS idx_scan_results_started ON network_scan_results(started_at);

-- Traffic logs indexes
CREATE INDEX IF NOT EXISTS idx_traffic_logs_source_ip ON network_traffic_logs(source_ip);
CREATE INDEX IF NOT EXISTS idx_traffic_logs_dest_ip ON network_traffic_logs(destination_ip);
CREATE INDEX IF NOT EXISTS idx_traffic_logs_user ON network_traffic_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_traffic_logs_flow_start ON network_traffic_logs(flow_start);
CREATE INDEX IF NOT EXISTS idx_traffic_logs_suspicious ON network_traffic_logs(is_suspicious);
CREATE INDEX IF NOT EXISTS idx_traffic_logs_blocked ON network_traffic_logs(blocked);

-- GIN indexes for JSONB columns
CREATE INDEX IF NOT EXISTS idx_network_nodes_ports ON network_nodes USING GIN(open_ports);
CREATE INDEX IF NOT EXISTS idx_network_nodes_services ON network_nodes USING GIN(running_services);
CREATE INDEX IF NOT EXISTS idx_scan_results_ports ON network_scan_results USING GIN(ports_found);
CREATE INDEX IF NOT EXISTS idx_scan_results_vulns ON network_scan_results USING GIN(vulnerabilities);

-- GIST indexes for geographic and network data
CREATE INDEX IF NOT EXISTS idx_network_nodes_geo ON network_nodes USING GIST(geo_location);
CREATE INDEX IF NOT EXISTS idx_network_routes_dest_network ON network_routes USING GIST(destination_network);

-- Create triggers for updated_at
CREATE TRIGGER network_nodes_updated_at_trigger
    BEFORE UPDATE ON network_nodes
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER network_connections_updated_at_trigger
    BEFORE UPDATE ON network_connections
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER network_routes_updated_at_trigger
    BEFORE UPDATE ON network_routes
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to establish a network connection
CREATE OR REPLACE FUNCTION establish_connection(
    source_ip_param INET,
    destination_ip_param INET,
    destination_port_param INTEGER,
    connection_type_param VARCHAR(20),
    user_id_param BIGINT DEFAULT NULL
) RETURNS BIGINT AS $$
DECLARE
    source_node_id BIGINT;
    dest_node_id BIGINT;
    connection_id BIGINT;
BEGIN
    -- Get source and destination nodes
    SELECT id INTO source_node_id FROM network_nodes WHERE ip_address = source_ip_param;
    SELECT id INTO dest_node_id FROM network_nodes WHERE ip_address = destination_ip_param;
    
    IF source_node_id IS NULL THEN
        RAISE EXCEPTION 'Source node % not found', source_ip_param;
    END IF;
    
    IF dest_node_id IS NULL THEN
        RAISE EXCEPTION 'Destination node % not found', destination_ip_param;
    END IF;
    
    -- Create connection record
    INSERT INTO network_connections (
        source_node_id, destination_node_id, source_ip, destination_ip,
        destination_port, connection_type, user_id
    ) VALUES (
        source_node_id, dest_node_id, source_ip_param, destination_ip_param,
        destination_port_param, connection_type_param, user_id_param
    ) RETURNING id INTO connection_id;
    
    -- Update connection counts
    UPDATE network_nodes SET
        current_connections = current_connections + 1,
        updated_at = CURRENT_TIMESTAMP
    WHERE id = dest_node_id;
    
    RETURN connection_id;
END;
$$ LANGUAGE plpgsql;

-- Function to perform network scan
CREATE OR REPLACE FUNCTION perform_network_scan(
    scanner_user_id_param BIGINT,
    target_ip_param INET,
    scan_type_param VARCHAR(20),
    port_range_param VARCHAR(50) DEFAULT '1-1024'
) RETURNS VARCHAR(50) AS $$
DECLARE
    scan_id_val VARCHAR(50);
    target_node RECORD;
    ports_found JSONB;
    scan_detected BOOLEAN;
BEGIN
    -- Generate scan ID
    scan_id_val := 'SCAN-' || TO_CHAR(CURRENT_TIMESTAMP, 'YYYYMMDDHH24MISS') || '-' || EXTRACT(MICROSECONDS FROM CURRENT_TIMESTAMP)::TEXT;
    
    -- Get target node information
    SELECT * INTO target_node FROM network_nodes WHERE ip_address = target_ip_param;
    
    -- Simulate scan results (in real implementation, this would perform actual scanning)
    ports_found := CASE 
        WHEN target_node.id IS NOT NULL AND target_node.online_status THEN
            target_node.open_ports
        ELSE
            '[]'::JSONB
    END;
    
    -- Determine if scan was detected based on target's security
    scan_detected := target_node.id IS NOT NULL AND 
                    target_node.intrusion_detection AND 
                    RANDOM() > 0.3; -- 70% chance of detection with IDS
    
    -- Create scan result record
    INSERT INTO network_scan_results (
        scan_id, scanner_user_id, target_node_id, target_ip,
        scan_type, port_range, scan_status, ports_found,
        detected, completed_at
    ) VALUES (
        scan_id_val, scanner_user_id_param, target_node.id, target_ip_param,
        scan_type_param, port_range_param, 'completed', ports_found,
        scan_detected, CURRENT_TIMESTAMP
    );
    
    -- Log security event if detected
    IF scan_detected THEN
        PERFORM log_security_event(
            'port_scan',
            'MEDIUM',
            'Port scan detected from ' || (SELECT login FROM users WHERE id = scanner_user_id_param),
            target_node.owner_user_id,
            (SELECT ip_address FROM network_nodes WHERE owner_user_id = scanner_user_id_param LIMIT 1)
        );
    END IF;
    
    RETURN scan_id_val;
END;
$$ LANGUAGE plpgsql;

-- Create view for network topology
CREATE OR REPLACE VIEW network_topology AS
SELECT 
    nn.id,
    nn.ip_address,
    nn.hostname,
    nn.node_type,
    nn.owner_user_id,
    u.login as owner_name,
    nn.online_status,
    nn.firewall_level,
    nn.current_connections,
    nn.max_connections,
    nn.country_code,
    nn.city,
    ARRAY_AGG(DISTINCT nc1.destination_ip) FILTER (WHERE nc1.destination_ip IS NOT NULL) as connected_to,
    ARRAY_AGG(DISTINCT nc2.source_ip) FILTER (WHERE nc2.source_ip IS NOT NULL) as connected_from
FROM network_nodes nn
LEFT JOIN users u ON nn.owner_user_id = u.id
LEFT JOIN network_connections nc1 ON nn.id = nc1.source_node_id AND nc1.connection_state = 'ESTABLISHED'
LEFT JOIN network_connections nc2 ON nn.id = nc2.destination_node_id AND nc2.connection_state = 'ESTABLISHED'
WHERE nn.online_status = TRUE
GROUP BY nn.id, nn.ip_address, nn.hostname, nn.node_type, nn.owner_user_id, u.login, 
         nn.online_status, nn.firewall_level, nn.current_connections, nn.max_connections,
         nn.country_code, nn.city;

-- Add comprehensive comments
COMMENT ON TABLE network_nodes IS 'All network-addressable entities including servers, routers, and devices';
COMMENT ON TABLE network_connections IS 'Active network connections between nodes with metrics and monitoring';
COMMENT ON TABLE network_routes IS 'Network routing table entries for packet forwarding';
COMMENT ON TABLE network_scan_results IS 'Results of network scans including port scans and vulnerability assessments';
COMMENT ON TABLE network_traffic_logs IS 'Network traffic monitoring and analysis logs';

COMMENT ON COLUMN network_nodes.node_type IS 'Type of network node: server, router, switch, firewall, proxy';
COMMENT ON COLUMN network_connections.connection_state IS 'TCP connection state: ESTABLISHED, SYN_SENT, etc.';
COMMENT ON COLUMN network_routes.admin_distance IS 'Administrative distance for route selection';
COMMENT ON COLUMN network_scan_results.stealth_level IS 'Stealth level used during scan (1=obvious, 10=invisible)';

COMMENT ON VIEW network_topology IS 'Real-time view of network topology showing connections between nodes';
COMMENT ON FUNCTION establish_connection IS 'Establishes a network connection between two nodes';
COMMENT ON FUNCTION perform_network_scan IS 'Performs a network scan and logs results with detection simulation';