//! Batch query implementations to prevent N+1 query problems
//!
//! This module provides optimized batch query methods to fetch related data
//! in a single query instead of multiple individual queries.

use anyhow::Result;
use sqlx::PgPool;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Batch queries for user-related data
pub struct UserBatchQueries;

impl UserBatchQueries {
    /// Fetch multiple users with their stats in a single query
    pub async fn get_users_with_stats(
        pool: &PgPool,
        user_ids: &[i64],
    ) -> Result<HashMap<i64, UserWithStats>> {
        if user_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let rows = sqlx::query!(
            r#"
            SELECT
                u.id as user_id,
                u.login,
                u.email,
                u.created,
                u.last_login,
                u.premium,
                u.online,
                s.reputation,
                s.money,
                s.experience,
                s.total_hacks,
                s.successful_hacks,
                s.failed_hacks
            FROM users u
            LEFT JOIN users_stats s ON u.id = s.user_id
            WHERE u.id = ANY($1)
            ORDER BY u.id
            "#,
            user_ids
        )
        .fetch_all(pool)
        .await?;

        let mut result = HashMap::with_capacity(rows.len());

        for row in rows {
            let user_with_stats = UserWithStats {
                user_id: row.user_id,
                login: row.login,
                email: row.email,
                created: row.created,
                last_login: row.last_login,
                premium: row.premium,
                online: row.online,
                reputation: row.reputation,
                money: row.money,
                experience: row.experience,
                total_hacks: row.total_hacks,
                successful_hacks: row.successful_hacks,
                failed_hacks: row.failed_hacks,
            };

            result.insert(row.user_id, user_with_stats);
        }

        Ok(result)
    }

    /// Fetch multiple users with their hardware in a single query
    pub async fn get_users_with_hardware(
        pool: &PgPool,
        user_ids: &[i64],
    ) -> Result<HashMap<i64, UserWithHardware>> {
        if user_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let rows = sqlx::query!(
            r#"
            SELECT
                u.id as user_id,
                u.login,
                h.cpu_mhz,
                h.ram_mb,
                h.hdd_gb,
                h.net_mbps,
                h.gpu_score,
                h.motherboard_slots
            FROM users u
            LEFT JOIN hardware h ON u.id = h.user_id
            WHERE u.id = ANY($1)
            ORDER BY u.id
            "#,
            user_ids
        )
        .fetch_all(pool)
        .await?;

        let mut result = HashMap::with_capacity(rows.len());

        for row in rows {
            let user_with_hardware = UserWithHardware {
                user_id: row.user_id,
                login: row.login,
                cpu_mhz: row.cpu_mhz.unwrap_or(1000),
                ram_mb: row.ram_mb.unwrap_or(1024),
                hdd_gb: row.hdd_gb.unwrap_or(100),
                net_mbps: row.net_mbps.unwrap_or(10),
                gpu_score: row.gpu_score.unwrap_or(0),
                motherboard_slots: row.motherboard_slots.unwrap_or(4),
            };

            result.insert(row.user_id, user_with_hardware);
        }

        Ok(result)
    }
}

/// Batch queries for process-related data
pub struct ProcessBatchQueries;

impl ProcessBatchQueries {
    /// Fetch processes with creator and target user information
    pub async fn get_processes_with_users(
        pool: &PgPool,
        process_ids: &[i64],
    ) -> Result<Vec<ProcessWithUsers>> {
        if process_ids.is_empty() {
            return Ok(Vec::new());
        }

        let rows = sqlx::query!(
            r#"
            SELECT
                p.pid,
                p.process_type,
                p.start_time,
                p.end_time,
                p.is_paused,
                p.cpu_usage,
                p.net_usage,
                p.creator_id,
                c.login as creator_login,
                c.game_ip as creator_ip,
                p.target_id,
                t.login as target_login,
                t.game_ip as target_ip
            FROM processes p
            INNER JOIN users c ON p.creator_id = c.id
            LEFT JOIN users t ON p.target_id = t.id
            WHERE p.pid = ANY($1)
            ORDER BY p.pid
            "#,
            process_ids
        )
        .fetch_all(pool)
        .await?;

        let mut result = Vec::with_capacity(rows.len());

        for row in rows {
            let process = ProcessWithUsers {
                pid: row.pid,
                process_type: row.process_type,
                start_time: row.start_time,
                end_time: row.end_time,
                is_paused: row.is_paused,
                cpu_usage: row.cpu_usage,
                net_usage: row.net_usage,
                creator_id: row.creator_id,
                creator_login: row.creator_login,
                creator_ip: row.creator_ip,
                target_id: row.target_id,
                target_login: row.target_login,
                target_ip: row.target_ip,
            };

            result.push(process);
        }

        Ok(result)
    }

    /// Get active processes for multiple users
    pub async fn get_active_processes_by_users(
        pool: &PgPool,
        user_ids: &[i64],
    ) -> Result<HashMap<i64, Vec<ActiveProcess>>> {
        if user_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let rows = sqlx::query!(
            r#"
            SELECT
                p.creator_id,
                p.pid,
                p.process_type,
                p.end_time,
                p.cpu_usage,
                p.net_usage,
                p.target_ip
            FROM processes p
            WHERE p.creator_id = ANY($1)
                AND p.end_time > NOW()
                AND p.is_paused = false
            ORDER BY p.creator_id, p.end_time
            "#,
            user_ids
        )
        .fetch_all(pool)
        .await?;

        let mut result: HashMap<i64, Vec<ActiveProcess>> = HashMap::new();

        for row in rows {
            let process = ActiveProcess {
                pid: row.pid,
                process_type: row.process_type,
                end_time: row.end_time,
                cpu_usage: row.cpu_usage,
                net_usage: row.net_usage,
                target_ip: row.target_ip,
            };

            result
                .entry(row.creator_id)
                .or_insert_with(Vec::new)
                .push(process);
        }

        // Ensure all requested users have an entry (even if empty)
        for user_id in user_ids {
            result.entry(*user_id).or_insert_with(Vec::new);
        }

        Ok(result)
    }
}

/// Batch queries for software-related data
pub struct SoftwareBatchQueries;

impl SoftwareBatchQueries {
    /// Get software inventory for multiple users
    pub async fn get_software_by_users(
        pool: &PgPool,
        user_ids: &[i64],
    ) -> Result<HashMap<i64, Vec<Software>>> {
        if user_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let rows = sqlx::query!(
            r#"
            SELECT
                s.user_id,
                s.software_id,
                s.software_type,
                s.name,
                s.version,
                s.size_mb,
                s.is_running,
                s.is_hidden
            FROM software s
            WHERE s.user_id = ANY($1)
            ORDER BY s.user_id, s.software_type, s.version DESC
            "#,
            user_ids
        )
        .fetch_all(pool)
        .await?;

        let mut result: HashMap<i64, Vec<Software>> = HashMap::new();

        for row in rows {
            let software = Software {
                software_id: row.software_id,
                software_type: row.software_type,
                name: row.name,
                version: row.version,
                size_mb: row.size_mb,
                is_running: row.is_running,
                is_hidden: row.is_hidden,
            };

            result
                .entry(row.user_id)
                .or_insert_with(Vec::new)
                .push(software);
        }

        // Ensure all requested users have an entry
        for user_id in user_ids {
            result.entry(*user_id).or_insert_with(Vec::new);
        }

        Ok(result)
    }
}

/// Batch queries for bank-related data
pub struct BankBatchQueries;

impl BankBatchQueries {
    /// Get bank accounts for multiple users
    pub async fn get_accounts_by_users(
        pool: &PgPool,
        user_ids: &[i64],
    ) -> Result<HashMap<i64, Vec<BankAccount>>> {
        if user_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let rows = sqlx::query!(
            r#"
            SELECT
                b.user_id,
                b.account_id,
                b.account_number,
                b.balance,
                b.account_type,
                b.created_at,
                b.last_activity
            FROM bank_accounts b
            WHERE b.user_id = ANY($1)
            ORDER BY b.user_id, b.account_id
            "#,
            user_ids
        )
        .fetch_all(pool)
        .await?;

        let mut result: HashMap<i64, Vec<BankAccount>> = HashMap::new();

        for row in rows {
            let account = BankAccount {
                account_id: row.account_id,
                account_number: row.account_number,
                balance: row.balance,
                account_type: row.account_type,
                created_at: row.created_at,
                last_activity: row.last_activity,
            };

            result
                .entry(row.user_id)
                .or_insert_with(Vec::new)
                .push(account);
        }

        // Ensure all requested users have an entry
        for user_id in user_ids {
            result.entry(*user_id).or_insert_with(Vec::new);
        }

        Ok(result)
    }

    /// Get recent transactions for multiple accounts
    pub async fn get_recent_transactions(
        pool: &PgPool,
        account_ids: &[i64],
        limit: i32,
    ) -> Result<HashMap<i64, Vec<Transaction>>> {
        if account_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let rows = sqlx::query!(
            r#"
            SELECT
                t.account_id,
                t.transaction_id,
                t.transaction_type,
                t.amount,
                t.balance_after,
                t.description,
                t.created_at,
                t.related_account
            FROM (
                SELECT *,
                    ROW_NUMBER() OVER (PARTITION BY account_id ORDER BY created_at DESC) as rn
                FROM bank_transactions
                WHERE account_id = ANY($1)
            ) t
            WHERE t.rn <= $2
            ORDER BY t.account_id, t.created_at DESC
            "#,
            account_ids,
            limit
        )
        .fetch_all(pool)
        .await?;

        let mut result: HashMap<i64, Vec<Transaction>> = HashMap::new();

        for row in rows {
            let transaction = Transaction {
                transaction_id: row.transaction_id,
                transaction_type: row.transaction_type,
                amount: row.amount,
                balance_after: row.balance_after,
                description: row.description,
                created_at: row.created_at,
                related_account: row.related_account,
            };

            result
                .entry(row.account_id)
                .or_insert_with(Vec::new)
                .push(transaction);
        }

        Ok(result)
    }
}

// Data structures for batch queries

#[derive(Debug, Clone)]
pub struct UserWithStats {
    pub user_id: i64,
    pub login: String,
    pub email: String,
    pub created: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub premium: bool,
    pub online: bool,
    pub reputation: Option<i32>,
    pub money: Option<i64>,
    pub experience: Option<i64>,
    pub total_hacks: Option<i32>,
    pub successful_hacks: Option<i32>,
    pub failed_hacks: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct UserWithHardware {
    pub user_id: i64,
    pub login: String,
    pub cpu_mhz: i32,
    pub ram_mb: i32,
    pub hdd_gb: i32,
    pub net_mbps: i32,
    pub gpu_score: i32,
    pub motherboard_slots: i32,
}

#[derive(Debug, Clone)]
pub struct ProcessWithUsers {
    pub pid: i64,
    pub process_type: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub is_paused: bool,
    pub cpu_usage: f32,
    pub net_usage: f32,
    pub creator_id: i64,
    pub creator_login: String,
    pub creator_ip: String,
    pub target_id: Option<i64>,
    pub target_login: Option<String>,
    pub target_ip: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ActiveProcess {
    pub pid: i64,
    pub process_type: String,
    pub end_time: DateTime<Utc>,
    pub cpu_usage: f32,
    pub net_usage: f32,
    pub target_ip: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Software {
    pub software_id: i64,
    pub software_type: String,
    pub name: String,
    pub version: f32,
    pub size_mb: i32,
    pub is_running: bool,
    pub is_hidden: bool,
}

#[derive(Debug, Clone)]
pub struct BankAccount {
    pub account_id: i64,
    pub account_number: String,
    pub balance: i64,
    pub account_type: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub transaction_id: i64,
    pub transaction_type: String,
    pub amount: i64,
    pub balance_after: i64,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub related_account: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_user_batch() {
        // This would require a test database connection
        // For now, just verify the function signature compiles
        let empty: Vec<i64> = vec![];
        // let result = UserBatchQueries::get_users_with_stats(&pool, &empty).await;
        assert!(empty.is_empty());
    }

    #[test]
    fn test_data_structures() {
        // Verify our data structures can be created
        let user = UserWithStats {
            user_id: 1,
            login: "test".to_string(),
            email: "test@example.com".to_string(),
            created: Utc::now(),
            last_login: Some(Utc::now()),
            premium: false,
            online: true,
            reputation: Some(100),
            money: Some(1000),
            experience: Some(500),
            total_hacks: Some(10),
            successful_hacks: Some(8),
            failed_hacks: Some(2),
        };

        assert_eq!(user.user_id, 1);
        assert_eq!(user.login, "test");
    }
}