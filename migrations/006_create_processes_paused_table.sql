-- Create processes_paused table - 1:1 port from original game.sql
-- Based on: CREATE TABLE `processes_paused` from game.sql

CREATE TABLE processes_paused (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    process_id BIGINT NOT NULL,            -- Links to processes.pid
    paused_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    time_remaining INT NOT NULL,           -- Seconds left when paused
    pause_data TEXT,                       -- Additional data needed to resume
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (process_id) REFERENCES processes(pid) ON DELETE CASCADE,
    INDEX idx_process_id (process_id),
    INDEX idx_paused_at (paused_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;