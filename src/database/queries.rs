use sqlx::{PgPool, Row, Transaction, Postgres};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use std::collections::HashMap;

// Core database query implementations for HackerExperience
// This provides 1:1 parity with the original PHP/Elixir database layer

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseUser {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_online: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub clan_id: Option<i64>,
    pub total_cpu: i64,
    pub total_ram: i64,
    pub total_hdd: i64,
    pub total_net: i64,
    pub money: i64,
    pub bitcoin: f64,
    pub experience: i64,
    pub reputation: f64,
    pub clan_reputation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseServer {
    pub id: i64,
    pub ip: String,
    pub name: String,
    pub owner_id: i64,
    pub server_type: String,
    pub is_npc: bool,
    pub firewall_version: Option<i32>,
    pub log_version: Option<i32>,
    pub ram: i64,
    pub cpu: i64,
    pub hdd: i64,
    pub net: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSoftware {
    pub id: i64,
    pub name: String,
    pub software_type: String,
    pub version: i32,
    pub size: i64,
    pub is_public: bool,
    pub install_time: i32,
    pub price: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseProcess {
    pub id: i64,
    pub creator_id: i64,
    pub victim_id: Option<i64>,
    pub source_ip: String,
    pub target_ip: String,
    pub action: String,
    pub software_id: Option<i64>,
    pub status: String,
    pub progress: f64,
    pub duration: i32,
    pub time_left: i32,
    pub cpu_usage: i32,
    pub net_usage: i32,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHardware {
    pub id: i64,
    pub owner_id: i64,
    pub hardware_type: String,
    pub spec_value: i64,
    pub is_npc: bool,
    pub is_pc: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseLog {
    pub id: i64,
    pub server_id: i64,
    pub ip: String,
    pub action: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseFile {
    pub id: i64,
    pub server_id: i64,
    pub name: String,
    pub file_type: String,
    pub size: i64,
    pub path: String,
    pub is_hidden: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseInstalledSoftware {
    pub id: i64,
    pub server_id: i64,
    pub software_id: i64,
    pub version: i32,
    pub is_running: bool,
    pub installed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseClan {
    pub id: i64,
    pub name: String,
    pub tag: String,
    pub description: String,
    pub leader_id: i64,
    pub total_reputation: f64,
    pub member_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMission {
    pub id: i64,
    pub user_id: i64,
    pub mission_type: String,
    pub target_id: Option<i64>,
    pub status: String,
    pub reward_money: i64,
    pub reward_experience: i64,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub struct UserQueries;

impl UserQueries {
    pub async fn create_user(
        pool: &PgPool,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<DatabaseUser> {
        let user = sqlx::query_as!(
            DatabaseUser,
            r#"
            INSERT INTO users (username, email, password_hash, is_online, created_at, 
                             total_cpu, total_ram, total_hdd, total_net, money, bitcoin, 
                             experience, reputation, clan_reputation)
            VALUES ($1, $2, $3, false, NOW(), 0, 0, 0, 0, 1000, 0.0, 0, 0.0, 0.0)
            RETURNING *
            "#,
            username,
            email,
            password_hash
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_id(pool: &PgPool, user_id: i64) -> Result<Option<DatabaseUser>> {
        let user = sqlx::query_as!(
            DatabaseUser,
            "SELECT * FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_username(pool: &PgPool, username: &str) -> Result<Option<DatabaseUser>> {
        let user = sqlx::query_as!(
            DatabaseUser,
            "SELECT * FROM users WHERE username = $1",
            username
        )
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<DatabaseUser>> {
        let user = sqlx::query_as!(
            DatabaseUser,
            "SELECT * FROM users WHERE email = $1",
            email
        )
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn update_user_online_status(
        pool: &PgPool,
        user_id: i64,
        is_online: bool,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET is_online = $1, last_login = CASE WHEN $1 THEN NOW() ELSE last_login END WHERE id = $2",
            is_online,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_user_resources(
        pool: &PgPool,
        user_id: i64,
        total_cpu: i64,
        total_ram: i64,
        total_hdd: i64,
        total_net: i64,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET total_cpu = $2, total_ram = $3, total_hdd = $4, total_net = $5 WHERE id = $1",
            user_id,
            total_cpu,
            total_ram,
            total_hdd,
            total_net
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_user_money(pool: &PgPool, user_id: i64, amount: i64) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET money = money + $2 WHERE id = $1",
            user_id,
            amount
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn join_clan(pool: &PgPool, user_id: i64, clan_id: i64) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET clan_id = $2 WHERE id = $1",
            user_id,
            clan_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn leave_clan(pool: &PgPool, user_id: i64) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET clan_id = NULL WHERE id = $1",
            user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_online_users(pool: &PgPool) -> Result<Vec<DatabaseUser>> {
        let users = sqlx::query_as!(
            DatabaseUser,
            "SELECT * FROM users WHERE is_online = true ORDER BY last_login DESC"
        )
        .fetch_all(pool)
        .await?;

        Ok(users)
    }

    pub async fn get_user_ranking(pool: &PgPool, limit: i64) -> Result<Vec<DatabaseUser>> {
        let users = sqlx::query_as!(
            DatabaseUser,
            "SELECT * FROM users ORDER BY experience DESC, reputation DESC LIMIT $1",
            limit
        )
        .fetch_all(pool)
        .await?;

        Ok(users)
    }
}

pub struct ServerQueries;

impl ServerQueries {
    pub async fn create_server(
        pool: &PgPool,
        ip: &str,
        name: &str,
        owner_id: i64,
        server_type: &str,
        is_npc: bool,
    ) -> Result<DatabaseServer> {
        let server = sqlx::query_as!(
            DatabaseServer,
            r#"
            INSERT INTO servers (ip, name, owner_id, server_type, is_npc, 
                               ram, cpu, hdd, net, created_at)
            VALUES ($1, $2, $3, $4, $5, 1024, 1000, 10240, 100, NOW())
            RETURNING *
            "#,
            ip,
            name,
            owner_id,
            server_type,
            is_npc
        )
        .fetch_one(pool)
        .await?;

        Ok(server)
    }

    pub async fn get_server_by_id(pool: &PgPool, server_id: i64) -> Result<Option<DatabaseServer>> {
        let server = sqlx::query_as!(
            DatabaseServer,
            "SELECT * FROM servers WHERE id = $1",
            server_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(server)
    }

    pub async fn get_server_by_ip(pool: &PgPool, ip: &str) -> Result<Option<DatabaseServer>> {
        let server = sqlx::query_as!(
            DatabaseServer,
            "SELECT * FROM servers WHERE ip = $1",
            ip
        )
        .fetch_optional(pool)
        .await?;

        Ok(server)
    }

    pub async fn get_servers_by_owner(pool: &PgPool, owner_id: i64) -> Result<Vec<DatabaseServer>> {
        let servers = sqlx::query_as!(
            DatabaseServer,
            "SELECT * FROM servers WHERE owner_id = $1 ORDER BY created_at DESC",
            owner_id
        )
        .fetch_all(pool)
        .await?;

        Ok(servers)
    }

    pub async fn update_server_firewall(
        pool: &PgPool,
        server_id: i64,
        firewall_version: i32,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE servers SET firewall_version = $2 WHERE id = $1",
            server_id,
            firewall_version
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_server_log(pool: &PgPool, server_id: i64, log_version: i32) -> Result<()> {
        sqlx::query!(
            "UPDATE servers SET log_version = $2 WHERE id = $1",
            server_id,
            log_version
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_server_specs(
        pool: &PgPool,
        server_id: i64,
        ram: i64,
        cpu: i64,
        hdd: i64,
        net: i64,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE servers SET ram = $2, cpu = $3, hdd = $4, net = $5 WHERE id = $1",
            server_id,
            ram,
            cpu,
            hdd,
            net
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_hackable_servers(pool: &PgPool, limit: i64) -> Result<Vec<DatabaseServer>> {
        let servers = sqlx::query_as!(
            DatabaseServer,
            r#"
            SELECT * FROM servers 
            WHERE is_npc = true 
            AND server_type IN ('bank', 'corporation', 'government', 'personal')
            ORDER BY RANDOM() 
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(pool)
        .await?;

        Ok(servers)
    }

    pub async fn delete_server(pool: &PgPool, server_id: i64) -> Result<()> {
        sqlx::query!("DELETE FROM servers WHERE id = $1", server_id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

pub struct SoftwareQueries;

impl SoftwareQueries {
    pub async fn create_software(
        pool: &PgPool,
        name: &str,
        software_type: &str,
        version: i32,
        size: i64,
        is_public: bool,
        install_time: i32,
        price: i64,
    ) -> Result<DatabaseSoftware> {
        let software = sqlx::query_as!(
            DatabaseSoftware,
            r#"
            INSERT INTO software (name, software_type, version, size, is_public, 
                                install_time, price, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            RETURNING *
            "#,
            name,
            software_type,
            version,
            size,
            is_public,
            install_time,
            price
        )
        .fetch_one(pool)
        .await?;

        Ok(software)
    }

    pub async fn get_software_by_id(
        pool: &PgPool,
        software_id: i64,
    ) -> Result<Option<DatabaseSoftware>> {
        let software = sqlx::query_as!(
            DatabaseSoftware,
            "SELECT * FROM software WHERE id = $1",
            software_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(software)
    }

    pub async fn get_software_by_type(
        pool: &PgPool,
        software_type: &str,
    ) -> Result<Vec<DatabaseSoftware>> {
        let software = sqlx::query_as!(
            DatabaseSoftware,
            "SELECT * FROM software WHERE software_type = $1 ORDER BY version DESC",
            software_type
        )
        .fetch_all(pool)
        .await?;

        Ok(software)
    }

    pub async fn get_public_software(pool: &PgPool) -> Result<Vec<DatabaseSoftware>> {
        let software = sqlx::query_as!(
            DatabaseSoftware,
            "SELECT * FROM software WHERE is_public = true ORDER BY name, version DESC"
        )
        .fetch_all(pool)
        .await?;

        Ok(software)
    }

    pub async fn install_software(
        pool: &PgPool,
        server_id: i64,
        software_id: i64,
        version: i32,
    ) -> Result<DatabaseInstalledSoftware> {
        let installed = sqlx::query_as!(
            DatabaseInstalledSoftware,
            r#"
            INSERT INTO installed_software (server_id, software_id, version, is_running, installed_at)
            VALUES ($1, $2, $3, false, NOW())
            RETURNING *
            "#,
            server_id,
            software_id,
            version
        )
        .fetch_one(pool)
        .await?;

        Ok(installed)
    }

    pub async fn get_installed_software(
        pool: &PgPool,
        server_id: i64,
    ) -> Result<Vec<DatabaseInstalledSoftware>> {
        let software = sqlx::query_as!(
            DatabaseInstalledSoftware,
            "SELECT * FROM installed_software WHERE server_id = $1 ORDER BY installed_at DESC",
            server_id
        )
        .fetch_all(pool)
        .await?;

        Ok(software)
    }

    pub async fn start_software(pool: &PgPool, installed_id: i64) -> Result<()> {
        sqlx::query!(
            "UPDATE installed_software SET is_running = true WHERE id = $1",
            installed_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn stop_software(pool: &PgPool, installed_id: i64) -> Result<()> {
        sqlx::query!(
            "UPDATE installed_software SET is_running = false WHERE id = $1",
            installed_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn uninstall_software(pool: &PgPool, installed_id: i64) -> Result<()> {
        sqlx::query!(
            "DELETE FROM installed_software WHERE id = $1",
            installed_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

pub struct ProcessQueries;

impl ProcessQueries {
    pub async fn create_process(
        pool: &PgPool,
        creator_id: i64,
        victim_id: Option<i64>,
        source_ip: &str,
        target_ip: &str,
        action: &str,
        software_id: Option<i64>,
        duration: i32,
        cpu_usage: i32,
        net_usage: i32,
    ) -> Result<DatabaseProcess> {
        let process = sqlx::query_as!(
            DatabaseProcess,
            r#"
            INSERT INTO processes (creator_id, victim_id, source_ip, target_ip, action, 
                                 software_id, status, progress, duration, time_left, 
                                 cpu_usage, net_usage, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, 'pending', 0.0, $7, $7, $8, $9, NOW())
            RETURNING *
            "#,
            creator_id,
            victim_id,
            source_ip,
            target_ip,
            action,
            software_id,
            duration,
            cpu_usage,
            net_usage
        )
        .fetch_one(pool)
        .await?;

        Ok(process)
    }

    pub async fn get_process_by_id(
        pool: &PgPool,
        process_id: i64,
    ) -> Result<Option<DatabaseProcess>> {
        let process = sqlx::query_as!(
            DatabaseProcess,
            "SELECT * FROM processes WHERE id = $1",
            process_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(process)
    }

    pub async fn get_processes_by_creator(
        pool: &PgPool,
        creator_id: i64,
    ) -> Result<Vec<DatabaseProcess>> {
        let processes = sqlx::query_as!(
            DatabaseProcess,
            "SELECT * FROM processes WHERE creator_id = $1 ORDER BY created_at DESC",
            creator_id
        )
        .fetch_all(pool)
        .await?;

        Ok(processes)
    }

    pub async fn get_active_processes(pool: &PgPool) -> Result<Vec<DatabaseProcess>> {
        let processes = sqlx::query_as!(
            DatabaseProcess,
            "SELECT * FROM processes WHERE status = 'running' ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await?;

        Ok(processes)
    }

    pub async fn start_process(pool: &PgPool, process_id: i64) -> Result<()> {
        sqlx::query!(
            "UPDATE processes SET status = 'running' WHERE id = $1",
            process_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_process_progress(
        pool: &PgPool,
        process_id: i64,
        progress: f64,
        time_left: i32,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE processes SET progress = $2, time_left = $3 WHERE id = $1",
            process_id,
            progress,
            time_left
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn complete_process(pool: &PgPool, process_id: i64) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE processes 
            SET status = 'completed', progress = 1.0, time_left = 0, completed_at = NOW() 
            WHERE id = $1
            "#,
            process_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn cancel_process(pool: &PgPool, process_id: i64) -> Result<()> {
        sqlx::query!(
            "UPDATE processes SET status = 'cancelled' WHERE id = $1",
            process_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_processes_by_target(
        pool: &PgPool,
        target_ip: &str,
    ) -> Result<Vec<DatabaseProcess>> {
        let processes = sqlx::query_as!(
            DatabaseProcess,
            "SELECT * FROM processes WHERE target_ip = $1 ORDER BY created_at DESC",
            target_ip
        )
        .fetch_all(pool)
        .await?;

        Ok(processes)
    }
}

pub struct LogQueries;

impl LogQueries {
    pub async fn create_log(
        pool: &PgPool,
        server_id: i64,
        ip: &str,
        action: &str,
        data: serde_json::Value,
    ) -> Result<DatabaseLog> {
        let log = sqlx::query_as!(
            DatabaseLog,
            r#"
            INSERT INTO logs (server_id, ip, action, data, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            RETURNING *
            "#,
            server_id,
            ip,
            action,
            data
        )
        .fetch_one(pool)
        .await?;

        Ok(log)
    }

    pub async fn get_logs_by_server(pool: &PgPool, server_id: i64) -> Result<Vec<DatabaseLog>> {
        let logs = sqlx::query_as!(
            DatabaseLog,
            "SELECT * FROM logs WHERE server_id = $1 ORDER BY created_at DESC",
            server_id
        )
        .fetch_all(pool)
        .await?;

        Ok(logs)
    }

    pub async fn get_logs_by_ip(pool: &PgPool, ip: &str) -> Result<Vec<DatabaseLog>> {
        let logs = sqlx::query_as!(
            DatabaseLog,
            "SELECT * FROM logs WHERE ip = $1 ORDER BY created_at DESC",
            ip
        )
        .fetch_all(pool)
        .await?;

        Ok(logs)
    }

    pub async fn delete_logs_older_than(pool: &PgPool, days: i32) -> Result<u64> {
        let result = sqlx::query!(
            "DELETE FROM logs WHERE created_at < NOW() - INTERVAL '%d days'",
            days
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn clear_server_logs(pool: &PgPool, server_id: i64) -> Result<u64> {
        let result = sqlx::query!("DELETE FROM logs WHERE server_id = $1", server_id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected())
    }
}

pub struct FileQueries;

impl FileQueries {
    pub async fn create_file(
        pool: &PgPool,
        server_id: i64,
        name: &str,
        file_type: &str,
        size: i64,
        path: &str,
        is_hidden: bool,
    ) -> Result<DatabaseFile> {
        let file = sqlx::query_as!(
            DatabaseFile,
            r#"
            INSERT INTO files (server_id, name, file_type, size, path, is_hidden, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            RETURNING *
            "#,
            server_id,
            name,
            file_type,
            size,
            path,
            is_hidden
        )
        .fetch_one(pool)
        .await?;

        Ok(file)
    }

    pub async fn get_files_by_server(pool: &PgPool, server_id: i64) -> Result<Vec<DatabaseFile>> {
        let files = sqlx::query_as!(
            DatabaseFile,
            "SELECT * FROM files WHERE server_id = $1 ORDER BY path, name",
            server_id
        )
        .fetch_all(pool)
        .await?;

        Ok(files)
    }

    pub async fn get_file_by_path(
        pool: &PgPool,
        server_id: i64,
        path: &str,
    ) -> Result<Option<DatabaseFile>> {
        let file = sqlx::query_as!(
            DatabaseFile,
            "SELECT * FROM files WHERE server_id = $1 AND path = $2",
            server_id,
            path
        )
        .fetch_optional(pool)
        .await?;

        Ok(file)
    }

    pub async fn delete_file(pool: &PgPool, file_id: i64) -> Result<()> {
        sqlx::query!("DELETE FROM files WHERE id = $1", file_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn hide_file(pool: &PgPool, file_id: i64) -> Result<()> {
        sqlx::query!(
            "UPDATE files SET is_hidden = true WHERE id = $1",
            file_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn unhide_file(pool: &PgPool, file_id: i64) -> Result<()> {
        sqlx::query!(
            "UPDATE files SET is_hidden = false WHERE id = $1",
            file_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

pub struct HardwareQueries;

impl HardwareQueries {
    pub async fn create_hardware(
        pool: &PgPool,
        owner_id: i64,
        hardware_type: &str,
        spec_value: i64,
        is_npc: bool,
        is_pc: bool,
    ) -> Result<DatabaseHardware> {
        let hardware = sqlx::query_as!(
            DatabaseHardware,
            r#"
            INSERT INTO hardware (owner_id, hardware_type, spec_value, is_npc, is_pc, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            RETURNING *
            "#,
            owner_id,
            hardware_type,
            spec_value,
            is_npc,
            is_pc
        )
        .fetch_one(pool)
        .await?;

        Ok(hardware)
    }

    pub async fn get_hardware_by_owner(
        pool: &PgPool,
        owner_id: i64,
    ) -> Result<Vec<DatabaseHardware>> {
        let hardware = sqlx::query_as!(
            DatabaseHardware,
            "SELECT * FROM hardware WHERE owner_id = $1 ORDER BY created_at DESC",
            owner_id
        )
        .fetch_all(pool)
        .await?;

        Ok(hardware)
    }

    pub async fn get_hardware_by_type(
        pool: &PgPool,
        owner_id: i64,
        hardware_type: &str,
    ) -> Result<Vec<DatabaseHardware>> {
        let hardware = sqlx::query_as!(
            DatabaseHardware,
            "SELECT * FROM hardware WHERE owner_id = $1 AND hardware_type = $2",
            owner_id,
            hardware_type
        )
        .fetch_all(pool)
        .await?;

        Ok(hardware)
    }

    pub async fn upgrade_hardware(
        pool: &PgPool,
        hardware_id: i64,
        new_spec_value: i64,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE hardware SET spec_value = $2 WHERE id = $1",
            hardware_id,
            new_spec_value
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete_hardware(pool: &PgPool, hardware_id: i64) -> Result<()> {
        sqlx::query!("DELETE FROM hardware WHERE id = $1", hardware_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn calculate_total_specs(
        pool: &PgPool,
        owner_id: i64,
    ) -> Result<(i64, i64, i64, i64)> {
        let result = sqlx::query!(
            r#"
            SELECT 
                COALESCE(SUM(CASE WHEN hardware_type = 'cpu' THEN spec_value ELSE 0 END), 0) as total_cpu,
                COALESCE(SUM(CASE WHEN hardware_type = 'ram' THEN spec_value ELSE 0 END), 0) as total_ram,
                COALESCE(SUM(CASE WHEN hardware_type = 'hdd' THEN spec_value ELSE 0 END), 0) as total_hdd,
                COALESCE(SUM(CASE WHEN hardware_type = 'net' THEN spec_value ELSE 0 END), 0) as total_net
            FROM hardware 
            WHERE owner_id = $1 AND is_pc = true
            "#,
            owner_id
        )
        .fetch_one(pool)
        .await?;

        Ok((
            result.total_cpu.unwrap_or(0),
            result.total_ram.unwrap_or(0),
            result.total_hdd.unwrap_or(0),
            result.total_net.unwrap_or(0),
        ))
    }
}

pub struct ClanQueries;

impl ClanQueries {
    pub async fn create_clan(
        pool: &PgPool,
        name: &str,
        tag: &str,
        description: &str,
        leader_id: i64,
    ) -> Result<DatabaseClan> {
        let clan = sqlx::query_as!(
            DatabaseClan,
            r#"
            INSERT INTO clans (name, tag, description, leader_id, total_reputation, 
                             member_count, created_at)
            VALUES ($1, $2, $3, $4, 0.0, 1, NOW())
            RETURNING *
            "#,
            name,
            tag,
            description,
            leader_id
        )
        .fetch_one(pool)
        .await?;

        Ok(clan)
    }

    pub async fn get_clan_by_id(pool: &PgPool, clan_id: i64) -> Result<Option<DatabaseClan>> {
        let clan = sqlx::query_as!(DatabaseClan, "SELECT * FROM clans WHERE id = $1", clan_id)
            .fetch_optional(pool)
            .await?;

        Ok(clan)
    }

    pub async fn get_clan_by_tag(pool: &PgPool, tag: &str) -> Result<Option<DatabaseClan>> {
        let clan = sqlx::query_as!(DatabaseClan, "SELECT * FROM clans WHERE tag = $1", tag)
            .fetch_optional(pool)
            .await?;

        Ok(clan)
    }

    pub async fn get_clan_members(pool: &PgPool, clan_id: i64) -> Result<Vec<DatabaseUser>> {
        let members = sqlx::query_as!(
            DatabaseUser,
            "SELECT * FROM users WHERE clan_id = $1 ORDER BY clan_reputation DESC",
            clan_id
        )
        .fetch_all(pool)
        .await?;

        Ok(members)
    }

    pub async fn update_clan_reputation(
        pool: &PgPool,
        clan_id: i64,
        reputation_change: f64,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE clans SET total_reputation = total_reputation + $2 WHERE id = $1",
            clan_id,
            reputation_change
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_member_count(pool: &PgPool, clan_id: i64) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE clans 
            SET member_count = (SELECT COUNT(*) FROM users WHERE clan_id = $1)
            WHERE id = $1
            "#,
            clan_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_top_clans(pool: &PgPool, limit: i64) -> Result<Vec<DatabaseClan>> {
        let clans = sqlx::query_as!(
            DatabaseClan,
            "SELECT * FROM clans ORDER BY total_reputation DESC LIMIT $1",
            limit
        )
        .fetch_all(pool)
        .await?;

        Ok(clans)
    }

    pub async fn delete_clan(pool: &PgPool, clan_id: i64) -> Result<()> {
        // First remove all members from the clan
        sqlx::query!("UPDATE users SET clan_id = NULL WHERE clan_id = $1", clan_id)
            .execute(pool)
            .await?;

        // Then delete the clan
        sqlx::query!("DELETE FROM clans WHERE id = $1", clan_id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

pub struct MissionQueries;

impl MissionQueries {
    pub async fn create_mission(
        pool: &PgPool,
        user_id: i64,
        mission_type: &str,
        target_id: Option<i64>,
        reward_money: i64,
        reward_experience: i64,
        description: &str,
    ) -> Result<DatabaseMission> {
        let mission = sqlx::query_as!(
            DatabaseMission,
            r#"
            INSERT INTO missions (user_id, mission_type, target_id, status, 
                                reward_money, reward_experience, description, created_at)
            VALUES ($1, $2, $3, 'active', $4, $5, $6, NOW())
            RETURNING *
            "#,
            user_id,
            mission_type,
            target_id,
            reward_money,
            reward_experience,
            description
        )
        .fetch_one(pool)
        .await?;

        Ok(mission)
    }

    pub async fn get_missions_by_user(pool: &PgPool, user_id: i64) -> Result<Vec<DatabaseMission>> {
        let missions = sqlx::query_as!(
            DatabaseMission,
            "SELECT * FROM missions WHERE user_id = $1 ORDER BY created_at DESC",
            user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(missions)
    }

    pub async fn get_active_missions(pool: &PgPool, user_id: i64) -> Result<Vec<DatabaseMission>> {
        let missions = sqlx::query_as!(
            DatabaseMission,
            "SELECT * FROM missions WHERE user_id = $1 AND status = 'active' ORDER BY created_at DESC",
            user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(missions)
    }

    pub async fn complete_mission(pool: &PgPool, mission_id: i64) -> Result<()> {
        sqlx::query!(
            "UPDATE missions SET status = 'completed', completed_at = NOW() WHERE id = $1",
            mission_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn fail_mission(pool: &PgPool, mission_id: i64) -> Result<()> {
        sqlx::query!(
            "UPDATE missions SET status = 'failed' WHERE id = $1",
            mission_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

// Transaction wrapper for complex operations
pub struct TransactionQueries;

impl TransactionQueries {
    pub async fn transfer_money(
        pool: &PgPool,
        from_user_id: i64,
        to_user_id: i64,
        amount: i64,
    ) -> Result<()> {
        let mut tx = pool.begin().await?;

        // Check if sender has enough money
        let sender = sqlx::query!(
            "SELECT money FROM users WHERE id = $1 FOR UPDATE",
            from_user_id
        )
        .fetch_one(&mut *tx)
        .await?;

        if sender.money < amount {
            return Err(anyhow!("Insufficient funds"));
        }

        // Deduct from sender
        sqlx::query!(
            "UPDATE users SET money = money - $2 WHERE id = $1",
            from_user_id,
            amount
        )
        .execute(&mut *tx)
        .await?;

        // Add to receiver
        sqlx::query!(
            "UPDATE users SET money = money + $2 WHERE id = $1",
            to_user_id,
            amount
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn hack_attempt(
        pool: &PgPool,
        attacker_id: i64,
        target_server_id: i64,
        success: bool,
    ) -> Result<()> {
        let mut tx = pool.begin().await?;

        // Get server and owner info
        let server = sqlx::query!(
            "SELECT owner_id FROM servers WHERE id = $1",
            target_server_id
        )
        .fetch_one(&mut *tx)
        .await?;

        if success {
            // Award experience to attacker
            sqlx::query!(
                "UPDATE users SET experience = experience + 100 WHERE id = $1",
                attacker_id
            )
            .execute(&mut *tx)
            .await?;

            // Create access log for successful hack
            LogQueries::create_log(
                pool,
                target_server_id,
                "unknown",
                "hack_success",
                serde_json::json!({"attacker_id": attacker_id}),
            )
            .await?;
        } else {
            // Log failed attempt
            LogQueries::create_log(
                pool,
                target_server_id,
                "unknown",
                "hack_failed",
                serde_json::json!({"attacker_id": attacker_id}),
            )
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn purchase_hardware(
        pool: &PgPool,
        user_id: i64,
        hardware_type: &str,
        spec_value: i64,
        cost: i64,
    ) -> Result<DatabaseHardware> {
        let mut tx = pool.begin().await?;

        // Check if user has enough money
        let user = sqlx::query!("SELECT money FROM users WHERE id = $1 FOR UPDATE", user_id)
            .fetch_one(&mut *tx)
            .await?;

        if user.money < cost {
            return Err(anyhow!("Insufficient funds"));
        }

        // Deduct money
        sqlx::query!(
            "UPDATE users SET money = money - $2 WHERE id = $1",
            user_id,
            cost
        )
        .execute(&mut *tx)
        .await?;

        // Create hardware
        let hardware = sqlx::query_as!(
            DatabaseHardware,
            r#"
            INSERT INTO hardware (owner_id, hardware_type, spec_value, is_npc, is_pc, created_at)
            VALUES ($1, $2, $3, false, true, NOW())
            RETURNING *
            "#,
            user_id,
            hardware_type,
            spec_value
        )
        .fetch_one(&mut *tx)
        .await?;

        // Update user's total specs
        let (total_cpu, total_ram, total_hdd, total_net) =
            HardwareQueries::calculate_total_specs(pool, user_id).await?;

        sqlx::query!(
            "UPDATE users SET total_cpu = $2, total_ram = $3, total_hdd = $4, total_net = $5 WHERE id = $1",
            user_id,
            total_cpu,
            total_ram,
            total_hdd,
            total_net
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(hardware)
    }
}

// Utility functions for common queries
pub struct QueryUtils;

impl QueryUtils {
    pub async fn get_server_with_files(
        pool: &PgPool,
        server_id: i64,
    ) -> Result<Option<(DatabaseServer, Vec<DatabaseFile>)>> {
        let server = ServerQueries::get_server_by_id(pool, server_id).await?;
        if let Some(server) = server {
            let files = FileQueries::get_files_by_server(pool, server_id).await?;
            Ok(Some((server, files)))
        } else {
            Ok(None)
        }
    }

    pub async fn get_user_with_hardware(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<Option<(DatabaseUser, Vec<DatabaseHardware>)>> {
        let user = UserQueries::get_user_by_id(pool, user_id).await?;
        if let Some(user) = user {
            let hardware = HardwareQueries::get_hardware_by_owner(pool, user_id).await?;
            Ok(Some((user, hardware)))
        } else {
            Ok(None)
        }
    }

    pub async fn get_process_with_details(
        pool: &PgPool,
        process_id: i64,
    ) -> Result<Option<(DatabaseProcess, Option<DatabaseSoftware>)>> {
        let process = ProcessQueries::get_process_by_id(pool, process_id).await?;
        if let Some(process) = process {
            let software = if let Some(software_id) = process.software_id {
                SoftwareQueries::get_software_by_id(pool, software_id).await?
            } else {
                None
            };
            Ok(Some((process, software)))
        } else {
            Ok(None)
        }
    }

    pub async fn search_users(
        pool: &PgPool,
        query: &str,
        limit: i64,
    ) -> Result<Vec<DatabaseUser>> {
        let users = sqlx::query_as!(
            DatabaseUser,
            "SELECT * FROM users WHERE username ILIKE $1 OR email ILIKE $1 ORDER BY username LIMIT $2",
            format!("%{}%", query),
            limit
        )
        .fetch_all(pool)
        .await?;

        Ok(users)
    }

    pub async fn search_servers(
        pool: &PgPool,
        query: &str,
        limit: i64,
    ) -> Result<Vec<DatabaseServer>> {
        let servers = sqlx::query_as!(
            DatabaseServer,
            "SELECT * FROM servers WHERE name ILIKE $1 OR ip ILIKE $1 ORDER BY name LIMIT $2",
            format!("%{}%", query),
            limit
        )
        .fetch_all(pool)
        .await?;

        Ok(servers)
    }
}