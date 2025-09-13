-- Create hardware_external table (XHD) - 1:1 port from original game.sql
-- Based on: CREATE TABLE `hardware_external` from game.sql

CREATE TABLE hardware_external (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    user_id BIGINT NOT NULL,
    size INT NOT NULL DEFAULT 0,           -- Storage size in MB
    used_space INT NOT NULL DEFAULT 0,    -- Currently used space  
    is_connected TINYINT(1) NOT NULL DEFAULT 0,
    name VARCHAR(50),                      -- XHD device name
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_user_id (user_id),
    INDEX idx_connected (is_connected)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;