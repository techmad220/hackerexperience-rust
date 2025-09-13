-- Create clan table - 1:1 port from original game.sql
-- Based on: CREATE TABLE `clan` from game.sql

CREATE TABLE clans (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(20) NOT NULL UNIQUE,
    leader_id BIGINT NOT NULL,
    description TEXT,
    tag VARCHAR(5),                        -- Clan tag/abbreviation
    member_count INT NOT NULL DEFAULT 1,
    max_members INT NOT NULL DEFAULT 50,
    is_public TINYINT(1) NOT NULL DEFAULT 1,  -- Can anyone join?
    reputation BIGINT NOT NULL DEFAULT 0,
    founded_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    FOREIGN KEY (leader_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_leader (leader_id),
    INDEX idx_reputation (reputation),
    INDEX idx_public (is_public)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;