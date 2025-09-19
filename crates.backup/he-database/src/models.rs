//! Database models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub login: String,
    pub pwd: String, // Hashed password
    pub email: String,
    pub online: bool,
    pub last_login: DateTime<Utc>,
    pub created: DateTime<Utc>,
    pub last_act: DateTime<Utc>,
    pub last_ip: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Process {
    pub pid: i64,
    pub user_id: i64,
    pub game_id: Option<String>,
    pub pc_id: String,
    pub target_pc_id: Option<String>,
    pub target_file_id: Option<String>,
    pub target_folder: Option<String>,
    pub process_type: String,
    pub priority: i32,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Hardware {
    pub hardware_id: i64,
    pub user_id: i64,
    pub cpu_mhz: i32,
    pub ram_mb: i32,
    pub hdd_mb: i32,
    pub net_mbps: i32,
    pub gpu_cores: i32,
    pub total_slots: i32,
    pub used_slots: i32,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Software {
    pub id: i64,
    pub pc_id: String,
    pub software_type: String,
    pub name: String,
    pub version: String,
    pub size: i64,
    pub installed: bool,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct BankAccount {
    pub id: i64,
    pub user_id: i64,
    pub account_number: String,
    pub balance: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Mission {
    pub id: i64,
    pub user_id: i64,
    pub mission_type: String,
    pub status: String,
    pub reward_money: i64,
    pub reward_xp: i32,
    pub progress: i32,
    pub total_steps: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Clan {
    pub id: i64,
    pub name: String,
    pub tag: String,
    pub leader_id: i64,
    pub reputation: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ClanMember {
    pub clan_id: i64,
    pub user_id: i64,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Log {
    pub id: i64,
    pub pc_id: String,
    pub user_id: i64,
    pub log_type: String,
    pub message: String,
    pub ip_address: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Virus {
    pub id: i64,
    pub owner_id: i64,
    pub infected_pc_id: String,
    pub virus_type: String,
    pub version: String,
    pub status: String,
    pub installed_at: DateTime<Utc>,
}