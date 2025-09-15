-- Create sessions table - User session management and tracking
-- Handles login sessions, security, and concurrent access control

CREATE TABLE IF NOT EXISTS sessions (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    
    -- Session identification
    session_token VARCHAR(128) NOT NULL UNIQUE,   -- Secure session token
    refresh_token VARCHAR(128) UNIQUE,            -- Refresh token for token renewal
    session_name VARCHAR(50),                     -- Optional session name/description
    
    -- Network and device information
    ip_address INET NOT NULL,                     -- Client IP address
    user_agent TEXT,                              -- Browser/client user agent
    device_fingerprint VARCHAR(128),              -- Device fingerprint hash
    geo_location POINT,                           -- Geographic location
    country_code VARCHAR(2),                      -- Country code
    city VARCHAR(50),                             -- City name
    
    -- Session timing
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_activity TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    logout_at TIMESTAMPTZ,                        -- When session was explicitly ended
    
    -- Session status and flags
    is_active BOOLEAN NOT NULL DEFAULT TRUE,      -- Session is currently active
    is_expired BOOLEAN NOT NULL DEFAULT FALSE,    -- Session has expired
    is_revoked BOOLEAN NOT NULL DEFAULT FALSE,    -- Session was revoked/invalidated
    is_mobile BOOLEAN NOT NULL DEFAULT FALSE,     -- Mobile device session
    is_api BOOLEAN NOT NULL DEFAULT FALSE,        -- API/programmatic access
    
    -- Security information
    login_method VARCHAR(20) DEFAULT 'password',  -- password, oauth, api_key, etc.
    auth_level INTEGER NOT NULL DEFAULT 1,       -- Authentication strength level
    requires_2fa BOOLEAN NOT NULL DEFAULT FALSE, -- Requires two-factor authentication
    two_fa_verified BOOLEAN NOT NULL DEFAULT FALSE, -- 2FA was verified
    
    -- Session activity tracking
    page_views INTEGER NOT NULL DEFAULT 0,        -- Number of page views
    api_calls INTEGER NOT NULL DEFAULT 0,         -- Number of API calls
    actions_performed INTEGER NOT NULL DEFAULT 0, -- Game actions performed
    data_uploaded BIGINT NOT NULL DEFAULT 0,     -- Bytes uploaded
    data_downloaded BIGINT NOT NULL DEFAULT 0,    -- Bytes downloaded
    
    -- Security events and flags
    suspicious_activity BOOLEAN NOT NULL DEFAULT FALSE,
    login_attempts INTEGER NOT NULL DEFAULT 1,    -- Number of login attempts before success
    concurrent_sessions INTEGER NOT NULL DEFAULT 1, -- Number of concurrent sessions
    
    -- Browser and technical details
    browser_name VARCHAR(50),
    browser_version VARCHAR(20),
    os_name VARCHAR(50),
    os_version VARCHAR(20),
    screen_resolution VARCHAR(20),
    timezone VARCHAR(50),
    language VARCHAR(10),
    
    -- Additional metadata
    referrer_url TEXT,                            -- Where user came from
    landing_page VARCHAR(255),                    -- First page visited
    exit_page VARCHAR(255),                       -- Last page before logout
    session_data JSONB DEFAULT '{}',             -- Additional session data
    
    -- Timestamps
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    
    -- Check constraints
    CONSTRAINT chk_session_auth_level CHECK (auth_level >= 1 AND auth_level <= 5),
    CONSTRAINT chk_session_page_views CHECK (page_views >= 0),
    CONSTRAINT chk_session_api_calls CHECK (api_calls >= 0),
    CONSTRAINT chk_session_actions CHECK (actions_performed >= 0),
    CONSTRAINT chk_session_data_transfer CHECK (data_uploaded >= 0 AND data_downloaded >= 0),
    CONSTRAINT chk_session_login_attempts CHECK (login_attempts >= 1),
    CONSTRAINT chk_session_concurrent CHECK (concurrent_sessions >= 1),
    CONSTRAINT chk_session_timing CHECK (expires_at > created_at),
    CONSTRAINT chk_session_logout_timing CHECK (logout_at IS NULL OR logout_at >= created_at),
    CONSTRAINT chk_session_2fa CHECK (NOT requires_2fa OR two_fa_verified)
);

-- Create comprehensive indexes
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(session_token);
CREATE INDEX IF NOT EXISTS idx_sessions_refresh_token ON sessions(refresh_token);
CREATE INDEX IF NOT EXISTS idx_sessions_ip_address ON sessions(ip_address);
CREATE INDEX IF NOT EXISTS idx_sessions_active ON sessions(is_active);
CREATE INDEX IF NOT EXISTS idx_sessions_expired ON sessions(is_expired);
CREATE INDEX IF NOT EXISTS idx_sessions_revoked ON sessions(is_revoked);
CREATE INDEX IF NOT EXISTS idx_sessions_last_activity ON sessions(last_activity);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_sessions_device_fingerprint ON sessions(device_fingerprint);
CREATE INDEX IF NOT EXISTS idx_sessions_suspicious ON sessions(suspicious_activity);
CREATE INDEX IF NOT EXISTS idx_sessions_user_active ON sessions(user_id, is_active);

-- Partial indexes for active sessions
CREATE INDEX IF NOT EXISTS idx_sessions_active_not_expired 
    ON sessions(user_id, last_activity) 
    WHERE is_active = TRUE AND is_expired = FALSE AND is_revoked = FALSE;

-- GIN index for session data
CREATE INDEX IF NOT EXISTS idx_sessions_data ON sessions USING GIN(session_data);

-- GIST index for geo_location
CREATE INDEX IF NOT EXISTS idx_sessions_geo ON sessions USING GIST(geo_location);

-- Create trigger for updated_at
CREATE TRIGGER sessions_updated_at_trigger
    BEFORE UPDATE ON sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to update last activity and check expiration
CREATE OR REPLACE FUNCTION update_session_activity(session_token_param VARCHAR(128))
RETURNS BOOLEAN AS $$
DECLARE
    session_record RECORD;
    session_updated BOOLEAN := FALSE;
BEGIN
    -- Get session details
    SELECT * INTO session_record 
    FROM sessions 
    WHERE session_token = session_token_param;
    
    IF NOT FOUND THEN
        RETURN FALSE;
    END IF;
    
    -- Check if session is still valid
    IF session_record.is_revoked OR session_record.is_expired OR 
       session_record.expires_at <= CURRENT_TIMESTAMP THEN
        -- Mark as expired if not already
        IF NOT session_record.is_expired THEN
            UPDATE sessions SET
                is_expired = TRUE,
                is_active = FALSE,
                updated_at = CURRENT_TIMESTAMP
            WHERE session_token = session_token_param;
        END IF;
        RETURN FALSE;
    END IF;
    
    -- Update last activity
    UPDATE sessions SET
        last_activity = CURRENT_TIMESTAMP,
        page_views = page_views + 1,
        updated_at = CURRENT_TIMESTAMP
    WHERE session_token = session_token_param;
    
    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- Function to create a new session
CREATE OR REPLACE FUNCTION create_session(
    user_id_param BIGINT,
    session_token_param VARCHAR(128),
    ip_address_param INET,
    user_agent_param TEXT DEFAULT NULL,
    device_fingerprint_param VARCHAR(128) DEFAULT NULL,
    expires_hours INTEGER DEFAULT 24,
    login_method_param VARCHAR(20) DEFAULT 'password'
) RETURNS BIGINT AS $$
DECLARE
    session_id BIGINT;
    max_sessions INTEGER := 10; -- Maximum concurrent sessions per user
    current_sessions INTEGER;
BEGIN
    -- Check current session count for user
    SELECT COUNT(*) INTO current_sessions
    FROM sessions
    WHERE user_id = user_id_param 
    AND is_active = TRUE 
    AND is_expired = FALSE 
    AND is_revoked = FALSE;
    
    -- If too many sessions, revoke oldest ones
    IF current_sessions >= max_sessions THEN
        UPDATE sessions SET
            is_revoked = TRUE,
            is_active = FALSE,
            logout_at = CURRENT_TIMESTAMP,
            updated_at = CURRENT_TIMESTAMP
        WHERE id IN (
            SELECT id FROM sessions
            WHERE user_id = user_id_param
            AND is_active = TRUE
            AND is_expired = FALSE
            AND is_revoked = FALSE
            ORDER BY last_activity ASC
            LIMIT (current_sessions - max_sessions + 1)
        );
    END IF;
    
    -- Create new session
    INSERT INTO sessions (
        user_id, session_token, ip_address, user_agent, 
        device_fingerprint, expires_at, login_method
    ) VALUES (
        user_id_param, session_token_param, ip_address_param, user_agent_param,
        device_fingerprint_param, 
        CURRENT_TIMESTAMP + INTERVAL '1 hour' * expires_hours,
        login_method_param
    ) RETURNING id INTO session_id;
    
    -- Log session creation
    INSERT INTO session_logs (session_id, event_type, message)
    VALUES (session_id, 'CREATED', 'Session created from ' || ip_address_param::TEXT);
    
    RETURN session_id;
END;
$$ LANGUAGE plpgsql;

-- Function to revoke/logout session
CREATE OR REPLACE FUNCTION revoke_session(
    session_token_param VARCHAR(128),
    reason VARCHAR(100) DEFAULT 'logout'
) RETURNS BOOLEAN AS $$
DECLARE
    session_id BIGINT;
BEGIN
    -- Update session as revoked
    UPDATE sessions SET
        is_revoked = TRUE,
        is_active = FALSE,
        logout_at = CURRENT_TIMESTAMP,
        updated_at = CURRENT_TIMESTAMP
    WHERE session_token = session_token_param
    AND is_revoked = FALSE
    RETURNING id INTO session_id;
    
    IF session_id IS NULL THEN
        RETURN FALSE;
    END IF;
    
    -- Log session revocation
    INSERT INTO session_logs (session_id, event_type, message)
    VALUES (session_id, 'REVOKED', 'Session revoked: ' || reason);
    
    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- Function to cleanup expired sessions
CREATE OR REPLACE FUNCTION cleanup_expired_sessions()
RETURNS INTEGER AS $$
DECLARE
    cleaned_count INTEGER;
BEGIN
    -- Mark expired sessions
    WITH expired_sessions AS (
        UPDATE sessions SET
            is_expired = TRUE,
            is_active = FALSE,
            updated_at = CURRENT_TIMESTAMP
        WHERE expires_at <= CURRENT_TIMESTAMP
        AND is_expired = FALSE
        RETURNING id
    )
    SELECT COUNT(*) INTO cleaned_count FROM expired_sessions;
    
    -- Delete very old sessions (older than 90 days)
    DELETE FROM sessions 
    WHERE created_at < CURRENT_TIMESTAMP - INTERVAL '90 days';
    
    RETURN cleaned_count;
END;
$$ LANGUAGE plpgsql;

-- Create session logs table
CREATE TABLE IF NOT EXISTS session_logs (
    id BIGSERIAL PRIMARY KEY,
    session_id BIGINT NOT NULL,
    event_type VARCHAR(20) NOT NULL,              -- CREATED, ACTIVITY, EXPIRED, REVOKED, SUSPICIOUS
    message TEXT,
    ip_address INET,
    details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_session_logs_session_id ON session_logs(session_id);
CREATE INDEX IF NOT EXISTS idx_session_logs_event_type ON session_logs(event_type);
CREATE INDEX IF NOT EXISTS idx_session_logs_created_at ON session_logs(created_at);

-- Create view for active sessions with user info
CREATE OR REPLACE VIEW active_sessions AS
SELECT 
    s.id,
    s.session_token,
    s.user_id,
    u.login as username,
    s.ip_address,
    s.device_fingerprint,
    s.browser_name,
    s.os_name,
    s.created_at,
    s.last_activity,
    s.expires_at,
    s.page_views,
    s.api_calls,
    s.actions_performed,
    s.suspicious_activity,
    s.country_code,
    s.city,
    EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - s.last_activity))::INTEGER as seconds_since_activity,
    EXTRACT(EPOCH FROM (s.expires_at - CURRENT_TIMESTAMP))::INTEGER as seconds_until_expiry
FROM sessions s
JOIN users u ON s.user_id = u.id
WHERE s.is_active = TRUE 
AND s.is_expired = FALSE 
AND s.is_revoked = FALSE
AND s.expires_at > CURRENT_TIMESTAMP;

-- Add comprehensive comments
COMMENT ON TABLE sessions IS 'User session management with security tracking and device fingerprinting';
COMMENT ON COLUMN sessions.session_token IS 'Unique session identifier token';
COMMENT ON COLUMN sessions.device_fingerprint IS 'Hash of device characteristics for security';
COMMENT ON COLUMN sessions.auth_level IS 'Authentication strength level (1-5)';
COMMENT ON COLUMN sessions.suspicious_activity IS 'Flag for sessions with suspicious patterns';
COMMENT ON COLUMN sessions.session_data IS 'Additional session data as JSON';

COMMENT ON TABLE session_logs IS 'Detailed logging of session lifecycle and security events';
COMMENT ON VIEW active_sessions IS 'Real-time view of currently active user sessions';

COMMENT ON FUNCTION update_session_activity IS 'Updates session activity timestamp and validates session';
COMMENT ON FUNCTION create_session IS 'Creates new user session with concurrent session limits';
COMMENT ON FUNCTION revoke_session IS 'Revokes/logs out a session with reason tracking';
COMMENT ON FUNCTION cleanup_expired_sessions IS 'Cleans up expired sessions and removes old records';