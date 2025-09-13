-- Create software table - 1:1 port from original game.sql
-- Based on: CREATE TABLE `software` from line 2350 in game.sql

CREATE TABLE software (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    user_id BIGINT NOT NULL,
    name VARCHAR(50) NOT NULL,
    software_type VARCHAR(20) NOT NULL,     -- Type of software (virus, tool, etc)
    version VARCHAR(10) NOT NULL DEFAULT '1.0',
    size INT NOT NULL DEFAULT 0,            -- File size in MB
    location VARCHAR(15) NOT NULL,          -- IP address where installed
    installed_at TIMESTAMP NULL,            -- When it was installed
    is_running TINYINT(1) NOT NULL DEFAULT 0,
    is_hidden TINYINT(1) NOT NULL DEFAULT 0,
    folder_id BIGINT NULL,                  -- If in a folder
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_user_id (user_id),
    INDEX idx_location (location),
    INDEX idx_type (software_type),
    INDEX idx_running (is_running),
    INDEX idx_hidden (is_hidden)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;