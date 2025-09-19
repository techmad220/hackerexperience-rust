use anyhow::{anyhow, Result};
//! Process entity - Game process management and execution
//! 
//! This module provides the Process struct and methods for managing game processes
//! such as hacking, downloading, uploading, research, and other timed activities.
//! This is noted as "the most complex part of Legacy and HE2" in the original code.

use sqlx::{Pool, Postgres, Row};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::entities::session::Session;
use crate::entities::player::Player;
use crate::entities::pc::HardwareVPC;
use crate::entities::system::System;
use crate::error::HeResult;

/// Represents a game process in the Hacker Experience game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    /// Process ID
    pub pid: Option<i32>,
    /// Creator user ID
    pub creator_id: i32,
    /// Victim user ID (if applicable)
    pub victim_id: Option<i32>,
    /// Process action type
    pub action: ProcessAction,
    /// Software ID used in process
    pub software_id: Option<i32>,
    /// Local/remote indicator
    pub local: String,
    /// Time left in seconds
    pub time_left: i32,
    /// Process information (serialized data)
    pub info: Option<String>,
    /// Process information string (human readable)
    pub info_str: Option<String>,
    /// Best software for this process
    pub best_soft: Option<i32>,
    /// Whether target is NPC
    pub is_npc: bool,
    /// CPU usage percentage
    pub cpu_usage: i32,
    /// Network usage percentage
    pub net_usage: i32,
    /// Process status
    pub status: ProcessStatus,
    /// Process creation time
    pub created_at: DateTime<Utc>,
    /// Process start time
    pub started_at: Option<DateTime<Utc>>,
    /// Process end time
    pub end_time: Option<DateTime<Utc>>,
    /// Whether process is paused
    pub is_paused: bool,
    /// Database connection pool
    #[serde(skip)]
    pub db_pool: Option<Pool<Postgres>>,
}

/// Process action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessAction {
    Download = 1,
    Upload = 2,
    Research = 3,
    Hack = 4,
    DDoS = 5,
    BankHack = 6,
    Collect = 7,
    PasswordReset = 8,
    IpReset = 9,
    Mission = 10,
    Study = 11,
    Custom(i32),
}

/// Process status types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Process information for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: i32,
    pub action: String,
    pub target: String,
    pub time_left: i32,
    pub progress: f32,
    pub cpu_usage: i32,
    pub net_usage: i32,
    pub is_paused: bool,
    pub can_pause: bool,
    pub can_resume: bool,
    pub can_cancel: bool,
}

/// Download speed calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSpeed {
    pub transfer_rate: f32,
    pub estimated_time: i32,
}

impl Process {
    /// Creates a new Process instance
    pub fn new(db_pool: Pool<Postgres>) -> Self {
        Self {
            pid: None,
            creator_id: 0,
            victim_id: None,
            action: ProcessAction::Download,
            software_id: None,
            local: "local".to_string(),
            time_left: 0,
            info: None,
            info_str: None,
            best_soft: None,
            is_npc: false,
            cpu_usage: 0,
            net_usage: 0,
            status: ProcessStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            end_time: None,
            is_paused: false,
            db_pool: Some(db_pool),
        }
    }

    /// Calculates download speed between two computers
    /// 
    /// # Arguments
    /// * `victim_id` - Victim computer ID
    /// * `victim_npc` - Whether victim is NPC
    /// * `action` - Process action (1=download, 2=upload)
    /// * `net_usage` - Network usage percentage
    /// * `hacker_id` - Hacker user ID
    /// 
    /// # Returns
    /// Transfer rate in KB/s
    pub async fn get_download_speed(
        &self,
        victim_id: i32,
        victim_npc: bool,
        action: i32,
        net_usage: i32,
        hacker_id: i32,
    ) -> HeResult<f32> {
        let db = self.db_pool.as_ref().ok_or_else(|| crate::error::HeError::Database(anyhow::anyhow!("DB pool not set")))?;
        let hardware = HardwareVPC::new(db.clone());
        let pc_type = if victim_npc { "NPC" } else { "VPC" };

        // Get hardware info for both parties
        let mut hacker_hardware = HardwareVPC::new(db.clone());
        let hacker_net_info = hacker_hardware.get_hardware_info(Some(hacker_id), "VPC", None).await?;

        let mut victim_hardware = HardwareVPC::new(db.clone());
        let victim_net_info = victim_hardware.get_hardware_info(Some(victim_id), pc_type, None).await?;

        let transfer_rate = if action == 1 { // download
            let download_rate_hacker = hacker_net_info.net as f32 / 8.0;
            let upload_rate_hacked = victim_net_info.net as f32 / 16.0;

            if upload_rate_hacked < download_rate_hacker {
                upload_rate_hacked
            } else {
                download_rate_hacker
            }
        } else { // upload
            let upload_rate_hacker = hacker_net_info.net as f32 / 16.0;
            let download_rate_hacked = victim_net_info.net as f32 / 8.0;

            if download_rate_hacked < upload_rate_hacker {
                download_rate_hacked
            } else {
                upload_rate_hacker
            }
        };

        Ok((transfer_rate * 1000.0) * (net_usage as f32 / 100.0))
    }

    /// Lists processes for a user
    /// 
    /// # Arguments
    /// * `uid` - User ID
    /// * `process_type` - Type filter ("all", "cpu", "net")
    /// 
    /// # Returns
    /// Vector of ProcessInfo
    pub async fn list_processes(&self, uid: i32, process_type: &str) -> HeResult<Vec<ProcessInfo>> {
        let db = self.db_pool.as_ref().ok_or_else(|| crate::error::HeError::Database(anyhow::anyhow!("DB pool not set")))?;

        let (condition, columns) = match process_type {
            "all" => ("", ", cpuUsage, netUsage"),
            "cpu" => (" AND (pAction <> 1 AND pAction <> 2)", ", cpuUsage"),
            "net" => (" AND (pAction = 1 OR pAction = 2)", ", netUsage"),
            _ => ("", ", cpuUsage, netUsage"),
        };

        let sql = format!(
            "SELECT pid, pvictimid, paction, psoftid, pinfo, plocal, pnpc, isPaused, \
             EXTRACT(EPOCH FROM (pTimeEnd - NOW()))::int AS pTimeLeft {} \
             FROM processes WHERE pcreatorid = $1 {} ORDER BY ptimeend DESC",
            columns, condition
        );

        let rows = sqlx::query(&sql)
            .bind(uid)
            .fetch_all(db)
            .await?;

        let mut processes = Vec::new();
        for row in rows {
            let action_id: i32 = row.get("paction");
            let time_left: i32 = row.get("pTimeLeft");
            let is_paused: bool = row.get("isPaused");

            processes.push(ProcessInfo {
                pid: row.get("pid"),
                action: self.get_proc_action_name(action_id),
                target: self.format_process_target(&row).await?,
                time_left: if time_left > 0 { time_left } else { 0 },
                progress: self.calculate_progress(time_left, action_id).await?,
                cpu_usage: row.try_get("cpuUsage").unwrap_or(0),
                net_usage: row.try_get("netUsage").unwrap_or(0),
                is_paused,
                can_pause: time_left > 0 && !is_paused,
                can_resume: is_paused,
                can_cancel: time_left > 0,
            });
        }

        Ok(processes)
    }

    /// Gets the human-readable name for a process action
    /// 
    /// # Arguments
    /// * `action` - Process action ID
    /// 
    /// # Returns
    /// Action name string
    fn get_proc_action_name(&self, action: i32) -> String {
        match action {
            1 => "Download".into(),
            2 => "Upload".into(),
            3 => "Delete".into(),
            4 => "Hide".into(),
            5 => "Seek".into(),
            6 => "Collect".into(),
            7 => "Antivirus".into(),
            8 => "Edit Log".into(),
            9 => "Delete Log".into(),
            10 => "Format".into(),
            11 => "Hack".into(),
            12 => "Bank Hack".into(),
            13 => "Install".into(),
            14 => "Uninstall".into(),
            15 => "Port Scan".into(),
            16 => "Hack XP".into(),
            17 => "Research".into(),
            18 => "Upload XHD".into(),
            19 => "Download XHD".into(),
            20 => "Delete XHD".into(),
            22 => "Nmap".into(),
            23 => "Analyze".into(),
            24 => "Install Doom".into(),
            25 => "Reset IP".into(),
            26 => "Reset Password".into(),
            27 => "DDoS".into(),
            28 => "Install Webserver".into(),
            _ => format!("Unknown ({})", action),
        }
    }

    /// Formats the process target for display
    /// 
    /// # Arguments
    /// * `row` - Database row with process data
    /// 
    /// # Returns
    /// Formatted target string
    async fn format_process_target(&self, row: &sqlx::postgres::PgRow) -> HeResult<String> {
        let victim_id: Option<i32> = row.get("pvictimid");
        let local: String = row.get("plocal");
        let is_npc: bool = row.get("pnpc");

        if local == "local" {
            Ok("Local".to_string())
        } else if let Some(vid) = victim_id {
            if is_npc {
                Ok(format!("NPC #{}", vid))
            } else {
                // TODO: Get player name or IP
                Ok(format!("Player #{}", vid))
            }
        } else {
            Ok("Unknown".to_string())
        }
    }

    /// Calculates process progress percentage
    /// 
    /// # Arguments
    /// * `time_left` - Time left in seconds
    /// * `action_id` - Process action ID
    /// 
    /// # Returns
    /// Progress as percentage (0.0 - 100.0)
    async fn calculate_progress(&self, time_left: i32, action_id: i32) -> HeResult<f32> {
        if time_left <= 0 {
            return Ok(100.0);
        }

        // TODO: Calculate original duration based on action and get actual progress
        // For now, return a placeholder
        Ok(50.0)
    }

    /// Creates a new process
    /// 
    /// # Arguments
    /// * `user_id` - Creator user ID
    /// * `action` - Process action string
    /// * `victim_id` - Victim ID
    /// * `host` - Target host
    /// * `software_id` - Software ID
    /// * `info` - Process info
    /// * `info_str` - Process info string
    /// * `is_npc` - Whether target is NPC
    /// 
    /// # Returns
    /// True if process was created successfully
    pub async fn new_process(
        &mut self,
        user_id: i32,
        action: &str,
        victim_id: &str,
        host: &str,
        software_id: &str,
        info: &str,
        info_str: &str,
        is_npc: i32,
    ) -> HeResult<bool> {
        let db = self.db_pool.as_ref().ok_or_else(|| crate::error::HeError::Database(anyhow::anyhow!("DB pool not set")))?;

        // Convert string parameters
        let action_id = self.parse_action_string(action)?;
        let victim_id_num = if victim_id.is_empty() { None } else { victim_id.parse::<i32>().ok() };
        let software_id_num = if software_id.is_empty() { None } else { software_id.parse::<i32>().ok() };

        // Check if process already exists
        if self.isset_process(user_id, action_id, victim_id, host, software_id, info).await? {
            return Ok(false);
        }

        // Calculate process duration
        let duration = self.calculate_process_duration(action_id, user_id, victim_id_num, host, is_npc, software_id_num, info).await?;

        // Insert new process
        let end_time = Utc::now() + chrono::Duration::seconds(duration as i64);

        let result = sqlx::query(
            "INSERT INTO processes (pcreatorid, paction, pvictimid, plocal, psoftid, pinfo, pinfostr, pnpc, ptimeend, cpuUsage, netUsage) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
        )
        .bind(user_id)
        .bind(action_id)
        .bind(victim_id_num)
        .bind(host)
        .bind(software_id_num)
        .bind(info)
        .bind(info_str)
        .bind(is_npc == 1)
        .bind(end_time)
        .bind(50) // Default CPU usage
        .bind(25) // Default net usage
        .execute(db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Parses action string to action ID
    /// 
    /// # Arguments
    /// * `action` - Action string
    /// 
    /// # Returns
    /// Action ID
    fn parse_action_string(&self, action: &str) -> HeResult<i32> {
        match action {
            "DOWNLOAD" => Ok(1),
            "UPLOAD" => Ok(2),
            "RESEARCH" => Ok(3),
            "HACK" => Ok(4),
            "DDOS" => Ok(5),
            "BANK_HACK" => Ok(6),
            "COLLECT" => Ok(7),
            "RESET_PWD" => Ok(8),
            "RESET_IP" => Ok(9),
            "MISSION" => Ok(10),
            "STUDY" => Ok(11),
            _ => Err(crate::error::HeError::ValidationError(format!("Unknown action: {}", action))),
        }
    }

    /// Checks if a process already exists
    /// 
    /// # Arguments
    /// * `user_id` - User ID
    /// * `action_id` - Action ID
    /// * `victim_id` - Victim ID string
    /// * `host` - Host string
    /// * `software_id` - Software ID string
    /// * `info` - Info string
    /// 
    /// # Returns
    /// True if process exists
    async fn isset_process(
        &self,
        user_id: i32,
        action_id: i32,
        victim_id: &str,
        host: &str,
        software_id: &str,
        info: &str,
    ) -> HeResult<bool> {
        let db = self.db_pool.as_ref().ok_or_else(|| crate::error::HeError::Database(anyhow::anyhow!("DB pool not set")))?;

        let victim_id_num = if victim_id.is_empty() { None } else { victim_id.parse::<i32>().ok() };
        let software_id_num = if software_id.is_empty() { None } else { software_id.parse::<i32>().ok() };

        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM processes 
             WHERE pcreatorid = $1 AND paction = $2 AND pvictimid = $3 AND plocal = $4 AND psoftid = $5 AND pinfo = $6"
        )
        .bind(user_id)
        .bind(action_id)
        .bind(victim_id_num)
        .bind(host)
        .bind(software_id_num)
        .bind(info)
        .fetch_one(db)
        .await?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    /// Calculates process duration based on various factors
    /// 
    /// # Arguments
    /// * `action_id` - Process action ID
    /// * `user_id` - User ID
    /// * `victim_id` - Victim ID (optional)
    /// * `host` - Host string
    /// * `is_npc` - Whether target is NPC
    /// * `software_id` - Software ID (optional)
    /// * `info` - Additional info
    /// 
    /// # Returns
    /// Duration in seconds
    async fn calculate_process_duration(
        &self,
        action_id: i32,
        user_id: i32,
        victim_id: Option<i32>,
        host: &str,
        is_npc: i32,
        software_id: Option<i32>,
        info: &str,
    ) -> HeResult<i32> {
        // TODO: Implement complex duration calculation based on:
        // - User's hardware specs
        // - Software versions
        // - Target difficulty
        // - Network speeds
        // - Process type
        
        // For now, return basic durations based on action
        let base_duration = match action_id {
            1 | 2 => 300,  // Download/Upload: 5 minutes
            3 => 1800,     // Research: 30 minutes
            4 => 600,      // Hack: 10 minutes
            5 => 900,      // DDoS: 15 minutes
            6 => 1200,     // Bank Hack: 20 minutes
            7 => 180,      // Collect: 3 minutes
            8 | 9 => 3600, // Resets: 1 hour
            10 => 1500,    // Mission: 25 minutes
            11 => 2400,    // Study: 40 minutes
            _ => 600,      // Default: 10 minutes
        };

        Ok(base_duration)
    }

    /// Gets process information by PID
    /// 
    /// # Arguments
    /// * `pid` - Process ID
    /// 
    /// # Returns
    /// Process information
    pub async fn get_process_info(&self, pid: i32) -> HeResult<ProcessInfo> {
        let db = self.db_pool.as_ref().ok_or_else(|| crate::error::HeError::Database(anyhow::anyhow!("DB pool not set")))?;

        let row = sqlx::query(
            "SELECT pid, paction, pvictimid, plocal, pnpc, isPaused, \
             EXTRACT(EPOCH FROM (pTimeEnd - NOW()))::int AS pTimeLeft, cpuUsage, netUsage \
             FROM processes WHERE pid = $1 LIMIT 1"
        )
        .bind(pid)
        .fetch_one(db)
        .await?;

        let action_id: i32 = row.get("paction");
        let time_left: i32 = row.get("pTimeLeft");
        let is_paused: bool = row.get("isPaused");

        Ok(ProcessInfo {
            pid: row.get("pid"),
            action: self.get_proc_action_name(action_id),
            target: self.format_process_target(&row).await?,
            time_left: if time_left > 0 { time_left } else { 0 },
            progress: self.calculate_progress(time_left, action_id).await?,
            cpu_usage: row.get("cpuUsage"),
            net_usage: row.get("netUsage"),
            is_paused,
            can_pause: time_left > 0 && !is_paused,
            can_resume: is_paused,
            can_cancel: time_left > 0,
        })
    }

    /// Pauses a process
    /// 
    /// # Arguments
    /// * `pid` - Process ID
    /// 
    /// # Returns
    /// True if successfully paused
    pub async fn pause_process(&self, pid: i32) -> HeResult<bool> {
        let db = self.db_pool.as_ref().ok_or_else(|| crate::error::HeError::Database(anyhow::anyhow!("DB pool not set")))?;

        let result = sqlx::query("UPDATE processes SET isPaused = true WHERE pid = $1")
            .bind(pid)
            .execute(db)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Resumes a process
    /// 
    /// # Arguments
    /// * `pid` - Process ID
    /// 
    /// # Returns
    /// True if successfully resumed
    pub async fn resume_process(&self, pid: i32) -> HeResult<bool> {
        let db = self.db_pool.as_ref().ok_or_else(|| crate::error::HeError::Database(anyhow::anyhow!("DB pool not set")))?;

        let result = sqlx::query("UPDATE processes SET isPaused = false WHERE pid = $1")
            .bind(pid)
            .execute(db)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Deletes a process
    /// 
    /// # Arguments
    /// * `pid` - Process ID
    /// * `update` - Whether to update related data
    /// 
    /// # Returns
    /// True if successfully deleted
    pub async fn delete_process(&self, pid: i32, update: bool) -> HeResult<bool> {
        let db = self.db_pool.as_ref().ok_or_else(|| crate::error::HeError::Database(anyhow::anyhow!("DB pool not set")))?;

        if update {
            // TODO: Update related data (hardware usage, etc.)
        }

        let result = sqlx::query("DELETE FROM processes WHERE pid = $1")
            .bind(pid)
            .execute(db)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Completes a process
    /// 
    /// # Arguments
    /// * `pid` - Process ID
    /// 
    /// # Returns
    /// True if successfully completed
    pub async fn complete_process(&self, pid: i32) -> HeResult<bool> {
        // TODO: Implement process completion logic
        // This involves:
        // - Executing the process action
        // - Updating game state
        // - Awarding experience
        // - Handling success/failure
        // - Cleaning up process data
        
        self.delete_process(pid, true).await
    }

    /// Gets total number of processes for a user
    /// 
    /// # Arguments
    /// * `user_id` - User ID (optional, uses session if None)
    /// 
    /// # Returns
    /// Total process count
    pub async fn total_processes(&self, user_id: Option<i32>) -> HeResult<i32> {
        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let uid = user_id.unwrap_or(0); // TODO: Get from session

        let row = sqlx::query("SELECT COUNT(*) as count FROM processes WHERE pcreatorid = $1")
            .bind(uid)
            .fetch_one(db)
            .await?;

        Ok(row.get::<i64, _>("count") as i32)
    }

    /// Checks if a process ID exists
    /// 
    /// # Arguments
    /// * `pid` - Process ID
    /// 
    /// # Returns
    /// True if process exists
    pub async fn isset_pid(&self, pid: i32) -> HeResult<bool> {
        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;

        let row = sqlx::query("SELECT COUNT(*) as count FROM processes WHERE pid = $1")
            .bind(pid)
            .fetch_one(db)
            .await?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    /// Checks if a process is paused
    /// 
    /// # Arguments
    /// * `pid` - Process ID
    /// 
    /// # Returns
    /// True if process is paused
    pub async fn is_paused(&self, pid: i32) -> HeResult<bool> {
        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;

        let row = sqlx::query("SELECT isPaused FROM processes WHERE pid = $1 LIMIT 1")
            .bind(pid)
            .fetch_one(db)
            .await?;

        Ok(row.get("isPaused"))
    }

    // TODO: Implement remaining methods:
    // - updateProcessTime: Update process timing when hardware changes
    // - updateProcessUsage: Update CPU/net usage
    // - completeProcess: Handle process completion with specific logic per action type
    // - studyProcess: Calculate process requirements
    // - issetDDoSProcess: Check for DDoS processes
    // - And many more complex process management methods
    //
    // The Process class is indeed the most complex part of the system,
    // handling dozens of different process types with intricate timing,
    // resource management, and game logic calculations.
}
