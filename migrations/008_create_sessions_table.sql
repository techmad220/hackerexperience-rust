-- Create sessions table for user session management
-- Based on PHP Session.class.php functionality

CREATE TABLE sessions (
    id VARCHAR(128) NOT NULL PRIMARY KEY,  -- Session ID
    user_id BIGINT NULL,                   -- NULL for unauthenticated sessions
    language VARCHAR(10) NOT NULL DEFAULT 'en_US',
    query_count INT NOT NULL DEFAULT 0,
    buffer_query INT NOT NULL DEFAULT 0, 
    exec_time DOUBLE NOT NULL DEFAULT 0.0,
    ip_address VARCHAR(45) NOT NULL,       -- Supports IPv6
    user_agent TEXT,
    is_active TINYINT(1) NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_activity TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_user_id (user_id),
    INDEX idx_last_activity (last_activity),
    INDEX idx_is_active (is_active),
    INDEX idx_ip_address (ip_address)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;