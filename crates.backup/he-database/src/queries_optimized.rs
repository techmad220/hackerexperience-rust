//! Optimized database queries with better performance characteristics

use crate::models::*;
use anyhow::Result;
use chrono::{Duration, Utc};
use sqlx::{PgPool, postgres::PgQueryResult};
use std::collections::HashMap;
use tracing::instrument;

/// Optimized user queries with caching hints and batch operations
pub struct OptimizedUserQueries;

impl OptimizedUserQueries {
    /// Get multiple users in a single query - much more efficient than N+1 queries
    #[instrument(skip(pool))]
    pub async fn get_users_batch(pool: &PgPool, user_ids: &[i64]) -> Result<HashMap<i64, User>> {
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users
            WHERE id = ANY($1)
            ORDER BY id
            "#,
            user_ids
        )
        .fetch_all(pool)
        .await?;

        Ok(users.into_iter().map(|u| (u.id, u)).collect())
    }

    /// Get user with related data in single query using JOINs
    #[instrument(skip(pool))]
    pub async fn get_user_with_stats(pool: &PgPool, user_id: i64) -> Result<Option<UserWithStats>> {
        let result = sqlx::query!(
            r#"
            SELECT
                u.*,
                COUNT(DISTINCT p.pid) as process_count,
                COUNT(DISTINCT s.id) as software_count,
                COALESCE(SUM(b.balance), 0) as total_balance
            FROM users u
            LEFT JOIN processes p ON u.id = p.user_id AND p.end_time > NOW()
            LEFT JOIN software s ON u.id = s.user_id
            LEFT JOIN bank_accounts b ON u.id = b.user_id
            WHERE u.id = $1
            GROUP BY u.id
            "#,
            user_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(result.map(|r| UserWithStats {
            user: User {
                id: r.id,
                login: r.login,
                pwd: r.pwd,
                email: r.email,
                online: r.online,
                last_login: r.last_login,
                created: r.created,
                last_act: r.last_act,
                last_ip: r.last_ip,
            },
            active_processes: r.process_count.unwrap_or(0),
            software_count: r.software_count.unwrap_or(0),
            total_balance: r.total_balance.unwrap_or(0),
        }))
    }

    /// Update user activity with single atomic query
    #[instrument(skip(pool))]
    pub async fn update_activity_atomic(
        pool: &PgPool,
        user_id: i64,
        ip: &str
    ) -> Result<PgQueryResult> {
        Ok(sqlx::query!(
            r#"
            UPDATE users
            SET
                last_act = NOW(),
                last_ip = $2,
                online = true
            WHERE id = $1
            "#,
            user_id,
            ip
        )
        .execute(pool)
        .await?)
    }
}

/// Optimized process queries with batch operations
pub struct OptimizedProcessQueries;

impl OptimizedProcessQueries {
    /// Get all active processes for multiple users in one query
    #[instrument(skip(pool))]
    pub async fn get_active_processes_batch(
        pool: &PgPool,
        user_ids: &[i64]
    ) -> Result<HashMap<i64, Vec<Process>>> {
        let processes = sqlx::query_as!(
            Process,
            r#"
            SELECT * FROM processes
            WHERE user_id = ANY($1)
                AND end_time > NOW()
            ORDER BY user_id, start_time DESC
            "#,
            user_ids
        )
        .fetch_all(pool)
        .await?;

        let mut result: HashMap<i64, Vec<Process>> = HashMap::new();
        for process in processes {
            result.entry(process.user_id).or_insert_with(Vec::new).push(process);
        }

        Ok(result)
    }

    /// Complete finished processes in batch
    #[instrument(skip(pool))]
    pub async fn complete_finished_processes(pool: &PgPool) -> Result<Vec<i64>> {
        let completed = sqlx::query!(
            r#"
            UPDATE processes
            SET completed = true
            WHERE end_time <= NOW()
                AND completed = false
            RETURNING pid
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(completed.iter().map(|r| r.pid).collect())
    }

    /// Cancel multiple processes atomically
    #[instrument(skip(pool))]
    pub async fn cancel_processes_batch(
        pool: &PgPool,
        process_ids: &[i64]
    ) -> Result<u64> {
        let result = sqlx::query!(
            r#"
            UPDATE processes
            SET
                end_time = NOW(),
                completed = false,
                cancelled = true
            WHERE pid = ANY($1)
                AND end_time > NOW()
            "#,
            process_ids
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}

/// Optimized server queries with connection pooling
pub struct OptimizedServerQueries;

impl OptimizedServerQueries {
    /// Get servers with their current load and active connections
    #[instrument(skip(pool))]
    pub async fn get_servers_with_load(pool: &PgPool) -> Result<Vec<ServerWithLoad>> {
        let servers = sqlx::query!(
            r#"
            SELECT
                s.*,
                COUNT(DISTINCT p.user_id) as active_users,
                COUNT(p.pid) as active_processes,
                COALESCE(AVG(EXTRACT(EPOCH FROM (p.end_time - NOW()))), 0) as avg_process_time
            FROM servers s
            LEFT JOIN processes p ON s.id = p.target_pc_id
                AND p.end_time > NOW()
            GROUP BY s.id
            ORDER BY s.id
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(servers.into_iter().map(|s| ServerWithLoad {
            server_id: s.id,
            active_users: s.active_users.unwrap_or(0),
            active_processes: s.active_processes.unwrap_or(0),
            avg_process_time: s.avg_process_time.unwrap_or(0.0) as i32,
        }).collect())
    }

    /// Batch update server security levels
    #[instrument(skip(pool))]
    pub async fn update_security_batch(
        pool: &PgPool,
        updates: &[(i64, i32)]
    ) -> Result<u64> {
        let mut tx = pool.begin().await?;
        let mut affected = 0u64;

        for (server_id, security_level) in updates {
            let result = sqlx::query!(
                "UPDATE servers SET security_level = $1 WHERE id = $2",
                security_level,
                server_id
            )
            .execute(&mut *tx)
            .await?;

            affected += result.rows_affected();
        }

        tx.commit().await?;
        Ok(affected)
    }
}

/// Optimized leaderboard queries using materialized views
pub struct OptimizedLeaderboardQueries;

impl OptimizedLeaderboardQueries {
    /// Get top players using indexed queries
    #[instrument(skip(pool))]
    pub async fn get_top_players(
        pool: &PgPool,
        limit: i32,
        offset: i32
    ) -> Result<Vec<LeaderboardEntry>> {
        let entries = sqlx::query!(
            r#"
            SELECT
                u.id,
                u.login as username,
                p.level,
                p.experience,
                p.total_hacks,
                p.reputation,
                DENSE_RANK() OVER (ORDER BY p.experience DESC) as rank
            FROM users u
            JOIN player_progression p ON u.id = p.user_id
            WHERE u.online = true OR u.last_act > NOW() - INTERVAL '7 days'
            ORDER BY p.experience DESC
            LIMIT $1 OFFSET $2
            "#,
            limit as i64,
            offset as i64
        )
        .fetch_all(pool)
        .await?;

        Ok(entries.into_iter().map(|e| LeaderboardEntry {
            user_id: e.id,
            username: e.username,
            level: e.level.unwrap_or(1),
            experience: e.experience.unwrap_or(0),
            total_hacks: e.total_hacks.unwrap_or(0),
            reputation: e.reputation.unwrap_or(0),
            rank: e.rank.unwrap_or(0) as i32,
        }).collect())
    }

    /// Get clan leaderboard with aggregated stats
    #[instrument(skip(pool))]
    pub async fn get_top_clans(pool: &PgPool, limit: i32) -> Result<Vec<ClanLeaderboardEntry>> {
        let entries = sqlx::query!(
            r#"
            SELECT
                c.id,
                c.name,
                c.tag,
                c.reputation,
                c.member_count,
                COUNT(DISTINCT cm.user_id) as active_members,
                COALESCE(SUM(p.experience), 0) as total_experience
            FROM clans c
            JOIN clan_members cm ON c.id = cm.clan_id
            LEFT JOIN player_progression p ON cm.user_id = p.user_id
            WHERE c.is_active = true
            GROUP BY c.id
            ORDER BY c.reputation DESC, total_experience DESC
            LIMIT $1
            "#,
            limit as i64
        )
        .fetch_all(pool)
        .await?;

        Ok(entries.into_iter().map(|e| ClanLeaderboardEntry {
            clan_id: e.id,
            name: e.name,
            tag: e.tag,
            reputation: e.reputation,
            member_count: e.member_count,
            active_members: e.active_members.unwrap_or(0),
            total_experience: e.total_experience.unwrap_or(0),
        }).collect())
    }
}

// Supporting structs for optimized queries
#[derive(Debug, Clone)]
pub struct UserWithStats {
    pub user: User,
    pub active_processes: i64,
    pub software_count: i64,
    pub total_balance: i64,
}

#[derive(Debug, Clone)]
pub struct ServerWithLoad {
    pub server_id: i64,
    pub active_users: i64,
    pub active_processes: i64,
    pub avg_process_time: i32,
}

#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    pub user_id: i64,
    pub username: String,
    pub level: i32,
    pub experience: i64,
    pub total_hacks: i64,
    pub reputation: i32,
    pub rank: i32,
}

#[derive(Debug, Clone)]
pub struct ClanLeaderboardEntry {
    pub clan_id: i64,
    pub name: String,
    pub tag: String,
    pub reputation: i32,
    pub member_count: i32,
    pub active_members: i64,
    pub total_experience: i64,
}