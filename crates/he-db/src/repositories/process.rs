use sqlx::{MySql, Pool};
use he_core::{Process, ProcessId, UserId, ProcessAction, ProcessStatus, HeResult, HeError};
use chrono::{DateTime, Utc};

// Process repository - replaces PHP Process.class.php database methods
// "This is the most complex part of Legacy and HE2." - Original comment
pub struct ProcessRepository {
    pool: Pool<MySql>,
}

impl ProcessRepository {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }
    
    // Create new process - equivalent to starting a new game action
    pub async fn create_process(&self, mut process: Process) -> HeResult<Process> {
        let result = sqlx::query!(
            r#"
            INSERT INTO processes (
                p_creator_id, p_victim_id, p_action, p_soft_id, p_info, p_info_str,
                p_time_start, p_time_end, p_time_ideal, p_time_worked,
                cpu_usage, net_usage, p_local, p_npc, is_paused
            )
            VALUES (?, ?, ?, ?, ?, ?, NOW(), DATE_ADD(NOW(), INTERVAL ? SECOND), ?, 0, ?, ?, 0, ?, 0)
            "#,
            process.creator_id,
            process.victim_id.unwrap_or(0),
            process.action.as_i32(),
            process.software_id.unwrap_or(0),
            process.info.as_deref().unwrap_or(""),
            process.info_str.as_deref().unwrap_or(""),
            process.time_left,
            process.time_left,
            process.cpu_usage,
            process.net_usage,
            process.is_npc
        )
        .execute(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        process.id = result.last_insert_id() as ProcessId;
        Ok(process)
    }
    
    // Get active processes for user
    pub async fn get_active_processes(&self, user_id: UserId) -> HeResult<Vec<Process>> {
        let rows = sqlx::query!(
            r#"
            SELECT * FROM processes 
            WHERE p_creator_id = ? AND p_time_end > NOW() AND is_paused = 0
            ORDER BY p_time_start ASC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        let mut processes = Vec::new();
        for row in rows {
            if let Some(action) = ProcessAction::from_i32(row.p_action as i32) {
                let process = Process {
                    id: row.pid as ProcessId,
                    creator_id: row.p_creator_id as UserId,
                    victim_id: if row.p_victim_id == 0 { None } else { Some(row.p_victim_id as UserId) },
                    action,
                    software_id: if row.p_soft_id == 0 { None } else { Some(row.p_soft_id) },
                    target_ip: "0.0.0.0".to_string(), // TODO: Map from p_local
                    time_left: self.calculate_time_left(&row.p_time_end).await?,
                    info: if row.p_info.is_empty() { None } else { Some(row.p_info) },
                    info_str: if row.p_info_str.is_empty() { None } else { Some(row.p_info_str) },
                    is_npc: row.p_npc != 0,
                    cpu_usage: row.cpu_usage as i32,
                    net_usage: row.net_usage as i32,
                    created_at: DateTime::from_timestamp(row.p_time_start.and_utc().timestamp(), 0).unwrap(),
                    started_at: Some(DateTime::from_timestamp(row.p_time_start.and_utc().timestamp(), 0).unwrap()),
                    completed_at: None,
                    status: if row.is_paused != 0 { ProcessStatus::Paused } else { ProcessStatus::Running },
                    priority: 5, // Default priority
                };
                processes.push(process);
            }
        }
        
        Ok(processes)
    }
    
    // Get process by ID
    pub async fn get_process(&self, process_id: ProcessId) -> HeResult<Option<Process>> {
        let row = sqlx::query!(
            "SELECT * FROM processes WHERE pid = ?",
            process_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        if let Some(row) = row {
            if let Some(action) = ProcessAction::from_i32(row.p_action as i32) {
                let process = Process {
                    id: row.pid as ProcessId,
                    creator_id: row.p_creator_id as UserId,
                    victim_id: if row.p_victim_id == 0 { None } else { Some(row.p_victim_id as UserId) },
                    action,
                    software_id: if row.p_soft_id == 0 { None } else { Some(row.p_soft_id) },
                    target_ip: "0.0.0.0".to_string(), // TODO: Map from p_local
                    time_left: self.calculate_time_left(&row.p_time_end).await?,
                    info: if row.p_info.is_empty() { None } else { Some(row.p_info) },
                    info_str: if row.p_info_str.is_empty() { None } else { Some(row.p_info_str) },
                    is_npc: row.p_npc != 0,
                    cpu_usage: row.cpu_usage as i32,
                    net_usage: row.net_usage as i32,
                    created_at: DateTime::from_timestamp(row.p_time_start.and_utc().timestamp(), 0).unwrap(),
                    started_at: Some(DateTime::from_timestamp(row.p_time_start.and_utc().timestamp(), 0).unwrap()),
                    completed_at: None,
                    status: if row.is_paused != 0 { ProcessStatus::Paused } else { ProcessStatus::Running },
                    priority: 5,
                };
                return Ok(Some(process));
            }
        }
        
        Ok(None)
    }
    
    // Update process status
    pub async fn update_process_status(&self, process_id: ProcessId, is_paused: bool) -> HeResult<()> {
        let paused_flag = if is_paused { 1 } else { 0 };
        
        sqlx::query!(
            "UPDATE processes SET is_paused = ?, p_time_pause = NOW() WHERE pid = ?",
            paused_flag,
            process_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        Ok(())
    }
    
    // Complete process - mark as finished
    pub async fn complete_process(&self, process_id: ProcessId) -> HeResult<()> {
        sqlx::query!(
            "UPDATE processes SET p_time_end = NOW() WHERE pid = ?",
            process_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        Ok(())
    }
    
    // Delete completed processes (cleanup)
    pub async fn cleanup_completed_processes(&self, hours_old: i32) -> HeResult<u64> {
        let result = sqlx::query!(
            "DELETE FROM processes WHERE p_time_end < DATE_SUB(NOW(), INTERVAL ? HOUR)",
            hours_old
        )
        .execute(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        Ok(result.rows_affected())
    }
    
    // Helper: Calculate time left for process
    async fn calculate_time_left(&self, end_time: &chrono::NaiveDateTime) -> HeResult<i32> {
        let now = Utc::now().naive_utc();
        let end = *end_time;
        
        if end > now {
            let duration = end - now;
            Ok(duration.num_seconds() as i32)
        } else {
            Ok(0) // Process completed
        }
    }
}