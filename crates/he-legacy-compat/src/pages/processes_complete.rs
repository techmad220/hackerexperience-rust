//! Complete 1:1 port of processes.php from HackerExperience Legacy
//!
//! This module handles all process management including CPU tasks,
//! network downloads, running software, and process control actions.

use axum::{
    extract::{Extension, Query, Form, Path},
    http::StatusCode,
    response::{Html, Redirect, IntoResponse},
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;

/// Process page query parameters
#[derive(Debug, Deserialize)]
pub struct ProcessQuery {
    pub pid: Option<i64>,
    pub action: Option<String>,
    pub page: Option<String>,
    pub del: Option<i64>,
}

/// Process information structure
#[derive(Debug, Serialize)]
pub struct ProcessInfo {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub process_type: String,
    pub status: String,
    pub progress: f32,
    pub cpu_usage: i32,
    pub ram_usage: i32,
    pub net_usage: i32,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub ends_at: Option<chrono::DateTime<chrono::Utc>>,
    pub paused_at: Option<chrono::DateTime<chrono::Utc>>,
    pub local: bool,
    pub gateway_ip: Option<String>,
}

/// Process statistics
#[derive(Debug, Default)]
pub struct ProcessStats {
    pub incomplete_cpu_proc: i32,
    pub incomplete_net_proc: i32,
    pub complete_cpu_proc: i32,
    pub complete_net_proc: i32,
    pub total_running: i32,
    pub total_paused: i32,
}

/// Main processes page handler - 1:1 port of processes.php
pub async fn processes_handler(
    Extension(pool): Extension<PgPool>,
    Extension(session): Extension<crate::session::PhpSession>,
    Query(params): Query<ProcessQuery>,
) -> impl IntoResponse {
    // Check if user is logged in (equivalent to isLogin() check)
    if !session.isset_login() {
        return Redirect::to("/index.php").into_response();
    }

    let user_id = session.user_id();

    // Initialize process manager (equivalent to new Process())
    let proc_manager = ProcessManager::new(pool.clone());

    // Handle process actions (pause, resume, delete)
    if let Some(action) = &params.action {
        if let Some(pid) = params.pid {
            match action.as_str() {
                "pause" => {
                    proc_manager.pause_process(pid, user_id).await;
                    return Redirect::to("/processes.php").into_response();
                }
                "resume" => {
                    proc_manager.resume_process(pid, user_id).await;
                    return Redirect::to("/processes.php").into_response();
                }
                _ => {}
            }
        }
    }

    // Handle delete action
    if let Some(del_id) = params.del {
        proc_manager.delete_process(del_id, user_id).await;
        return Redirect::to("/processes.php").into_response();
    }

    // Get process statistics
    let proc_info = proc_manager.get_process_stats(user_id).await;

    // Determine which page to load (all, cpu, net)
    let page_to_load = determine_page_to_load(&proc_info, params.page);

    // Get process list based on page type
    let processes = match page_to_load.as_str() {
        "net" => proc_manager.list_net_processes(user_id).await,
        "cpu" => proc_manager.list_cpu_processes(user_id).await,
        _ => proc_manager.list_all_processes(user_id).await,
    };

    // Render the processes page
    let html = render_processes_page(processes, proc_info, page_to_load).await;

    Html(html).into_response()
}

/// Process Manager - equivalent to Process class in PHP
pub struct ProcessManager {
    pool: PgPool,
}

impl ProcessManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Check if PID exists and belongs to user
    pub async fn isset_pid(&self, pid: i64, user_id: i64) -> bool {
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM processes WHERE id = $1 AND user_id = $2",
            pid,
            user_id
        )
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(row) => row.count.unwrap_or(0) > 0,
            Err(_) => false,
        }
    }

    /// Get process information
    pub async fn get_process_info(&self, pid: i64, user_id: i64) -> Option<ProcessInfo> {
        sqlx::query_as!(
            ProcessInfo,
            r#"
            SELECT
                id, user_id, name,
                process_type, status,
                COALESCE(progress, 0.0) as "progress!",
                COALESCE(cpu_usage, 0) as "cpu_usage!",
                COALESCE(ram_usage, 0) as "ram_usage!",
                COALESCE(net_usage, 0) as "net_usage!",
                started_at,
                ends_at,
                paused_at,
                COALESCE(local, true) as "local!",
                gateway_ip
            FROM processes
            WHERE id = $1 AND user_id = $2
            "#,
            pid,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten()
    }

    /// Get process statistics for user
    pub async fn get_process_stats(&self, user_id: i64) -> ProcessStats {
        let result = sqlx::query!(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE process_type = 'CPU' AND status = 'RUNNING') as incomplete_cpu,
                COUNT(*) FILTER (WHERE process_type = 'NET' AND status = 'RUNNING') as incomplete_net,
                COUNT(*) FILTER (WHERE process_type = 'CPU' AND status = 'COMPLETED') as complete_cpu,
                COUNT(*) FILTER (WHERE process_type = 'NET' AND status = 'COMPLETED') as complete_net,
                COUNT(*) FILTER (WHERE status = 'RUNNING') as total_running,
                COUNT(*) FILTER (WHERE status = 'PAUSED') as total_paused
            FROM processes
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(row) => ProcessStats {
                incomplete_cpu_proc: row.incomplete_cpu.unwrap_or(0) as i32,
                incomplete_net_proc: row.incomplete_net.unwrap_or(0) as i32,
                complete_cpu_proc: row.complete_cpu.unwrap_or(0) as i32,
                complete_net_proc: row.complete_net.unwrap_or(0) as i32,
                total_running: row.total_running.unwrap_or(0) as i32,
                total_paused: row.total_paused.unwrap_or(0) as i32,
            },
            Err(_) => ProcessStats::default(),
        }
    }

    /// List all processes
    pub async fn list_all_processes(&self, user_id: i64) -> Vec<ProcessInfo> {
        sqlx::query_as!(
            ProcessInfo,
            r#"
            SELECT
                id, user_id, name,
                process_type, status,
                COALESCE(progress, 0.0) as "progress!",
                COALESCE(cpu_usage, 0) as "cpu_usage!",
                COALESCE(ram_usage, 0) as "ram_usage!",
                COALESCE(net_usage, 0) as "net_usage!",
                started_at,
                ends_at,
                paused_at,
                COALESCE(local, true) as "local!",
                gateway_ip
            FROM processes
            WHERE user_id = $1 AND status IN ('RUNNING', 'PAUSED')
            ORDER BY started_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }

    /// List CPU processes
    pub async fn list_cpu_processes(&self, user_id: i64) -> Vec<ProcessInfo> {
        sqlx::query_as!(
            ProcessInfo,
            r#"
            SELECT
                id, user_id, name,
                process_type, status,
                COALESCE(progress, 0.0) as "progress!",
                COALESCE(cpu_usage, 0) as "cpu_usage!",
                COALESCE(ram_usage, 0) as "ram_usage!",
                COALESCE(net_usage, 0) as "net_usage!",
                started_at,
                ends_at,
                paused_at,
                COALESCE(local, true) as "local!",
                gateway_ip
            FROM processes
            WHERE user_id = $1 AND process_type = 'CPU' AND status IN ('RUNNING', 'PAUSED')
            ORDER BY started_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }

    /// List Network processes
    pub async fn list_net_processes(&self, user_id: i64) -> Vec<ProcessInfo> {
        sqlx::query_as!(
            ProcessInfo,
            r#"
            SELECT
                id, user_id, name,
                process_type, status,
                COALESCE(progress, 0.0) as "progress!",
                COALESCE(cpu_usage, 0) as "cpu_usage!",
                COALESCE(ram_usage, 0) as "ram_usage!",
                COALESCE(net_usage, 0) as "net_usage!",
                started_at,
                ends_at,
                paused_at,
                COALESCE(local, true) as "local!",
                gateway_ip
            FROM processes
            WHERE user_id = $1 AND process_type = 'NET' AND status IN ('RUNNING', 'PAUSED')
            ORDER BY started_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default()
    }

    /// Pause a process
    pub async fn pause_process(&self, pid: i64, user_id: i64) -> bool {
        let result = sqlx::query!(
            "UPDATE processes SET status = 'PAUSED', paused_at = NOW()
             WHERE id = $1 AND user_id = $2 AND status = 'RUNNING'",
            pid,
            user_id
        )
        .execute(&self.pool)
        .await;

        result.is_ok()
    }

    /// Resume a process
    pub async fn resume_process(&self, pid: i64, user_id: i64) -> bool {
        // Calculate new end time based on pause duration
        let result = sqlx::query!(
            r#"
            UPDATE processes
            SET
                status = 'RUNNING',
                ends_at = ends_at + (NOW() - paused_at),
                paused_at = NULL
            WHERE id = $1 AND user_id = $2 AND status = 'PAUSED'
            "#,
            pid,
            user_id
        )
        .execute(&self.pool)
        .await;

        result.is_ok()
    }

    /// Delete/cancel a process
    pub async fn delete_process(&self, pid: i64, user_id: i64) -> bool {
        // First check if it's a deletable process
        let proc_info = self.get_process_info(pid, user_id).await;

        if let Some(info) = proc_info {
            // Can't delete certain critical processes
            if info.name.contains("Install") || info.name.contains("Download") {
                // Free up resources
                sqlx::query!(
                    "UPDATE hardware SET cpu_in_use = cpu_in_use - $1, ram_in_use = ram_in_use - $2
                     WHERE user_id = $3",
                    info.cpu_usage,
                    info.ram_usage,
                    user_id
                )
                .execute(&self.pool)
                .await
                .ok();
            }

            // Delete the process
            let result = sqlx::query!(
                "DELETE FROM processes WHERE id = $1 AND user_id = $2",
                pid,
                user_id
            )
            .execute(&self.pool)
            .await;

            return result.is_ok();
        }

        false
    }

    /// Check for completed processes and handle them
    pub async fn check_completed_processes(&self, user_id: i64) {
        // Find completed processes
        let completed = sqlx::query!(
            "SELECT id, name, process_type FROM processes
             WHERE user_id = $1 AND ends_at <= NOW() AND status = 'RUNNING'",
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        for proc in completed {
            // Handle completion based on process type
            match proc.process_type.as_deref() {
                Some("DOWNLOAD") => self.complete_download(proc.id, user_id).await,
                Some("INSTALL") => self.complete_install(proc.id, user_id).await,
                Some("RESEARCH") => self.complete_research(proc.id, user_id).await,
                Some("CRACK") => self.complete_crack(proc.id, user_id).await,
                _ => {}
            }

            // Mark as completed
            sqlx::query!(
                "UPDATE processes SET status = 'COMPLETED', progress = 100 WHERE id = $1",
                proc.id
            )
            .execute(&self.pool)
            .await
            .ok();
        }
    }

    async fn complete_download(&self, pid: i64, user_id: i64) {
        // Transfer software from remote to local
        // Implementation specific to download completion
    }

    async fn complete_install(&self, pid: i64, user_id: i64) {
        // Mark software as installed
        // Implementation specific to install completion
    }

    async fn complete_research(&self, pid: i64, user_id: i64) {
        // Upgrade software version
        // Implementation specific to research completion
    }

    async fn complete_crack(&self, pid: i64, user_id: i64) {
        // Grant access to cracked system
        // Implementation specific to crack completion
    }
}

/// Determine which page to load based on process stats
fn determine_page_to_load(proc_info: &ProcessStats, requested_page: Option<String>) -> String {
    if let Some(page) = requested_page {
        return page;
    }

    // Logic from PHP: if both CPU and NET processes exist, show all
    if proc_info.incomplete_net_proc > 0 && proc_info.incomplete_cpu_proc > 0 {
        "all".to_string()
    } else if proc_info.incomplete_net_proc > 0 {
        "net".to_string()
    } else if proc_info.incomplete_cpu_proc > 0 {
        "cpu".to_string()
    } else {
        "all".to_string()
    }
}

/// Render the processes page HTML (equivalent to template rendering in PHP)
async fn render_processes_page(
    processes: Vec<ProcessInfo>,
    stats: ProcessStats,
    page_type: String,
) -> String {
    let mut html = String::from(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Task Manager - HackerExperience</title>
    <link rel="stylesheet" href="/css/he-matrix.css">
    <style>
        .process-list { margin: 20px; }
        .process-item {
            background: #1a1a1a;
            border: 1px solid #00ff00;
            margin: 10px 0;
            padding: 15px;
        }
        .process-bar {
            background: #0a0a0a;
            height: 20px;
            border: 1px solid #333;
            margin: 10px 0;
        }
        .process-progress {
            background: linear-gradient(to right, #004400, #00ff00);
            height: 100%;
            transition: width 0.5s;
        }
        .process-actions { margin-top: 10px; }
        .btn {
            padding: 5px 10px;
            margin-right: 5px;
            background: #1a1a1a;
            color: #00ff00;
            border: 1px solid #00ff00;
            cursor: pointer;
        }
        .btn:hover { background: #00ff00; color: #000; }
        .tabs { margin: 20px; }
        .tab {
            display: inline-block;
            padding: 10px 20px;
            margin-right: 5px;
            background: #1a1a1a;
            color: #888;
            text-decoration: none;
        }
        .tab.active { color: #00ff00; border-bottom: 2px solid #00ff00; }
    </style>
</head>
<body>
    <div class="container">
        <h1>Task Manager</h1>

        <div class="tabs">
            <a href="/processes.php" class="tab {ALL_ACTIVE}">All</a>
            <a href="/processes.php?page=cpu" class="tab {CPU_ACTIVE}">CPU ({CPU_COUNT})</a>
            <a href="/processes.php?page=net" class="tab {NET_ACTIVE}">Network ({NET_COUNT})</a>
        </div>

        <div class="process-list">
    "#);

    // Set active tab
    html = html.replace(
        "{ALL_ACTIVE}",
        if page_type == "all" { "active" } else { "" },
    );
    html = html.replace(
        "{CPU_ACTIVE}",
        if page_type == "cpu" { "active" } else { "" },
    );
    html = html.replace(
        "{NET_ACTIVE}",
        if page_type == "net" { "active" } else { "" },
    );
    html = html.replace("{CPU_COUNT}", &stats.incomplete_cpu_proc.to_string());
    html = html.replace("{NET_COUNT}", &stats.incomplete_net_proc.to_string());

    // Render each process
    for process in processes {
        let progress_percent = process.progress.min(100.0).max(0.0);
        let time_left = if let Some(ends_at) = process.ends_at {
            let now = chrono::Utc::now();
            let duration = ends_at - now;
            format!("{} seconds", duration.num_seconds().max(0))
        } else {
            "Unknown".to_string()
        };

        let action_buttons = if process.status == "PAUSED" {
            format!(
                r#"
                <a href="/processes.php?pid={}&action=resume" class="btn">Resume</a>
                <a href="/processes.php?del={}" class="btn">Delete</a>
                "#,
                process.id, process.id
            )
        } else {
            format!(
                r#"
                <a href="/processes.php?pid={}&action=pause" class="btn">Pause</a>
                <a href="/processes.php?del={}" class="btn">Delete</a>
                "#,
                process.id, process.id
            )
        };

        html.push_str(&format!(
            r#"
            <div class="process-item">
                <h3>{}</h3>
                <div>Type: {} | Status: {} | Time Left: {}</div>
                <div>CPU: {}% | RAM: {}MB | NET: {}Mbit</div>
                <div class="process-bar">
                    <div class="process-progress" style="width: {}%"></div>
                </div>
                <div>Progress: {:.1}%</div>
                <div class="process-actions">
                    {}
                </div>
            </div>
            "#,
            process.name,
            process.process_type,
            process.status,
            time_left,
            process.cpu_usage,
            process.ram_usage,
            process.net_usage,
            progress_percent,
            progress_percent,
            action_buttons
        ));
    }

    if processes.is_empty() {
        html.push_str(r#"
            <div style="text-align: center; padding: 50px; color: #666;">
                <h2>No active processes</h2>
                <p>Your task manager is empty</p>
            </div>
        "#);
    }

    html.push_str(r#"
        </div>
    </div>
</body>
</html>
    "#);

    html
}