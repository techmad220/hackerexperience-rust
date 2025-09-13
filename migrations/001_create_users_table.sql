-- Create users table - 1:1 port from original game.sql
-- Based on: CREATE TABLE `users` from line 2608 in game.sql

CREATE TABLE users (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    login VARCHAR(15) NOT NULL UNIQUE,
    password VARCHAR(60) NOT NULL,  -- BCrypt hash (original was 60 chars)
    email VARCHAR(50) NOT NULL UNIQUE,
    game_pass VARCHAR(8) NOT NULL,  -- Original: gamePass
    game_ip BIGINT UNSIGNED NOT NULL,  -- Original: gameIP  
    real_ip BIGINT UNSIGNED NOT NULL,  -- Original: realIP
    home_ip BIGINT UNSIGNED NOT NULL,  -- Original: homeIP
    learning TINYINT(1) NOT NULL DEFAULT 0,
    premium TINYINT(1) NOT NULL DEFAULT 0,
    last_login TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    INDEX idx_game_ip (game_ip),
    INDEX idx_last_login (last_login),
    INDEX idx_premium (premium)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;