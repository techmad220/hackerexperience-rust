-- Create users_stats table - 1:1 port from original game.sql  
-- Based on: CREATE TABLE `users_stats` from game.sql

CREATE TABLE users_stats (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    user_id BIGINT NOT NULL,
    reputation BIGINT NOT NULL DEFAULT 0,  -- Called "power" in newer versions
    money BIGINT NOT NULL DEFAULT 0,
    experience BIGINT NOT NULL DEFAULT 0, 
    total_hacks BIGINT NOT NULL DEFAULT 0,
    successful_hacks BIGINT NOT NULL DEFAULT 0,
    failed_hacks BIGINT NOT NULL DEFAULT 0,
    viruses_uploaded BIGINT NOT NULL DEFAULT 0,
    round_stats_id BIGINT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_user_id (user_id),
    INDEX idx_reputation (reputation),
    INDEX idx_money (money)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;