-- Create processes table - 1:1 port from original game.sql
-- Based on: CREATE TABLE `processes` from line 1950 in game.sql
-- "This is the most complex part of Legacy and HE2." - Original comment

CREATE TABLE processes (
    pid BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,         -- Original: pid
    p_creator_id BIGINT NOT NULL,                           -- Original: pCreatorID
    p_victim_id BIGINT NOT NULL,                            -- Original: pVictimID
    p_action SMALLINT NOT NULL,                             -- Original: pAction (maps to ProcessAction enum)
    p_soft_id BIGINT NOT NULL,                              -- Original: pSoftID
    p_info VARCHAR(30) NOT NULL,                            -- Original: pInfo
    p_info_str TEXT NOT NULL,                               -- Original: pInfoStr
    p_time_start TIMESTAMP NOT NULL DEFAULT '1970-01-01 00:00:01',  -- Original: pTimeStart
    p_time_pause TIMESTAMP NOT NULL DEFAULT '1970-01-01 00:00:01',  -- Original: pTimePause  
    p_time_end TIMESTAMP NOT NULL DEFAULT '1970-01-01 00:00:01',    -- Original: pTimeEnd
    p_time_ideal INT NOT NULL,                              -- Original: pTimeIdeal
    p_time_worked INT NOT NULL,                             -- Original: pTimeWorked
    cpu_usage DOUBLE NOT NULL,                              -- Original: cpuUsage
    net_usage DOUBLE NOT NULL,                              -- Original: netUsage
    p_local TINYINT(1) NOT NULL,                           -- Original: pLocal (confusingly named - actually target IP)
    p_npc TINYINT(1) NOT NULL,                             -- Original: pNPC
    is_paused TINYINT(1) NOT NULL,                         -- Original: isPaused
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    FOREIGN KEY (p_creator_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (p_victim_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_creator (p_creator_id),                       -- Original: pCreatorID
    INDEX idx_victim (p_victim_id),                         -- Original: pVictimID
    INDEX idx_npc (p_npc),                                  -- Original: pNPC
    INDEX idx_time_end (p_time_end),                        -- Original: pTimeEnd
    INDEX idx_action (p_action),
    INDEX idx_paused (is_paused)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;