-- Create hardware table - 1:1 port from original game.sql
-- Based on: CREATE TABLE `hardware` from line 790 in game.sql

CREATE TABLE hardware (
    server_id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,  -- Original: serverID
    user_id BIGINT NOT NULL,                               -- Original: userID
    name VARCHAR(15) NOT NULL,
    cpu FLOAT NOT NULL DEFAULT 500,
    hdd FLOAT NOT NULL DEFAULT 100, 
    ram FLOAT NOT NULL DEFAULT 256,
    net FLOAT NOT NULL DEFAULT 1,
    is_npc TINYINT(1) NOT NULL DEFAULT 0,                  -- Original: isNPC
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_user_npc (user_id, is_npc),  -- Original: IndiceComNPC
    INDEX idx_is_npc (is_npc)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;