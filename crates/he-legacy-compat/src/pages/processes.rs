// PROCESSES.PHP PORT - Process management and monitoring interface
// Original: Core game mechanic for managing running processes (hacking, downloads, etc.)

use axum::{
    extract::{Query, Extension},
    http::StatusCode,
    response::Html,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use he_core::*;
use he_db::*;
use crate::session::PhpSession;
use crate::classes::{System, Process};

#[derive(Debug, Deserialize)]
pub struct ProcessesQuery {
    #[serde(default)]
    pub pid: Option<String>,
    #[serde(default)]
    pub del: Option<String>,
    #[serde(default)]
    pub page: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessInfo {
    pub id: i64,
    pub action: i32,
    pub target_ip: String,
    pub time_left: i32,
    pub total_time: i32,
    pub status: String,
    pub progress_percent: f32,
}

// Original PHP: Main processes.php handler
pub async fn processes_handler(
    Extension(db): Extension<DbPool>,
    Query(query): Query<ProcessesQuery>,
    mut session: PhpSession,
) -> Result<Html<String>, StatusCode> {
    
    // Check if user is authenticated
    if !session.isset_login() {
        return Ok(Html(render_login_required()));
    }
    
    let system = System::new();
    let process = Process::new(db.clone());
    
    // Handle process ID parameter
    let mut got_get = false;
    let mut process_id = None;
    
    if let Some(pid_str) = &query.pid {
        if let Ok(pid) = pid_str.parse::<i64>() {
            if process.process_exists(pid).await.unwrap_or(false) {
                got_get = true;
                process_id = Some(pid);
            } else {
                session.add_msg(&System::handle_error("PROC_NOT_FOUND"), "error");
            }
        } else {
            session.add_msg(&System::handle_error("INVALID_GET"), "error");
        }
    }
    
    // Handle process deletion
    if let (Some(del), Some(pid)) = (&query.del, process_id) {
        if del == "1" && got_get {
            if process.delete_process(pid, false).await.is_ok() {
                session.add_msg("Process deleted.", "notice");
                got_get = false;
                process_id = None;
            }
        }
    }
    
    // Determine which page to load
    let page_to_load = if let Some(page) = &query.page {
        match page.as_str() {
            "all" => "all",
            "cpu" => "cpu",
            "network" => "net",
            "running" => "running",
            _ => {
                session.add_msg(&System::handle_error("INVALID_GET"), "error");
                "all"
            }
        }
    } else if let Some(pid) = process_id {
        if got_get {
            // Get process info and determine type
            if let Ok(proc_info) = process.get_process_info(pid).await {
                // Actions 1 and 2 are network processes (download/upload)
                if proc_info.action == 1 || proc_info.action == 2 {
                    "net"
                } else {
                    "cpu"
                }
            } else {
                "cpu"
            }
        } else {
            "all"
        }
    } else {
        // Check if user has running processes to determine default view
        if let Ok(running_info) = process.get_running_process_info().await {
            if !running_info.is_empty() {
                "running"
            } else {
                "all"
            }
        } else {
            "all"
        }
    };
    
    // Render the appropriate page
    let content = match page_to_load {
        "all" => render_all_processes(&process, &session).await,
        "cpu" => render_cpu_processes(&process, &session).await,
        "net" => render_network_processes(&process, &session).await,
        "running" => render_running_processes(&process, &session).await,
        _ => render_all_processes(&process, &session).await,
    };
    
    // Wrap content in template
    let html = render_processes_page(&session, &content, page_to_load);
    
    Ok(Html(html))
}

// Render all processes view
async fn render_all_processes(process: &Process, session: &PhpSession) -> String {
    let user_id = session.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    
    // Get all processes for user
    let processes = process.get_user_processes(user_id, None).await.unwrap_or_default();
    
    let mut html = String::new();
    html.push_str(r#"<div class="processes-container">"#);
    html.push_str(r#"<h3>All Processes</h3>"#);
    
    if processes.is_empty() {
        html.push_str(r#"<p>No processes running.</p>"#);
    } else {
        html.push_str(r#"<table class="table table-striped">"#);
        html.push_str(r#"<thead><tr><th>ID</th><th>Action</th><th>Target</th><th>Progress</th><th>Time Left</th><th>Actions</th></tr></thead><tbody>"#);
        
        for proc in processes {
            html.push_str(&format!(
                r#"<tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td><div class="progress"><div class="progress-bar" style="width: {}%"></div></div></td>
                    <td>{}</td>
                    <td>
                        <a href="processes?pid={}" class="btn btn-sm btn-primary">View</a>
                        <a href="processes?pid={}&del=1" class="btn btn-sm btn-danger" onclick="return confirm('Delete this process?')">Delete</a>
                    </td>
                </tr>"#,
                proc.id,
                get_action_name(proc.action),
                proc.target_ip,
                proc.progress_percent,
                format_time_left(proc.time_left),
                proc.id,
                proc.id
            ));
        }
        
        html.push_str(r#"</tbody></table>"#);
    }
    
    html.push_str(r#"</div>"#);
    html
}

// Render CPU processes view
async fn render_cpu_processes(process: &Process, session: &PhpSession) -> String {
    let user_id = session.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    
    // Get CPU-intensive processes (not network operations)
    let processes = process.get_user_processes(user_id, Some("cpu")).await.unwrap_or_default();
    
    let mut html = String::new();
    html.push_str(r#"<div class="processes-container">"#);
    html.push_str(r#"<h3>CPU Processes</h3>"#);
    html.push_str(r#"<p>Processes that use CPU resources (hacking, cracking, analysis, etc.)</p>"#);
    
    if processes.is_empty() {
        html.push_str(r#"<p>No CPU processes running.</p>"#);
    } else {
        html.push_str(r#"<div class="row">"#);
        
        for proc in processes {
            html.push_str(&format!(
                r#"<div class="col-md-6">
                    <div class="widget-box">
                        <div class="widget-title">
                            <span class="icon"><i class="icon-cog"></i></span>
                            <h5>{} (PID: {})</h5>
                        </div>
                        <div class="widget-content">
                            <p><strong>Target:</strong> {}</p>
                            <p><strong>Status:</strong> {}</p>
                            <div class="progress">
                                <div class="progress-bar progress-bar-info" style="width: {}%"></div>
                            </div>
                            <p><strong>Time remaining:</strong> {}</p>
                            <div class="form-actions">
                                <a href="processes?pid={}" class="btn btn-primary">View Details</a>
                                <a href="processes?pid={}&del=1" class="btn btn-danger">Terminate</a>
                            </div>
                        </div>
                    </div>
                </div>"#,
                get_action_name(proc.action),
                proc.id,
                proc.target_ip,
                proc.status,
                proc.progress_percent,
                format_time_left(proc.time_left),
                proc.id,
                proc.id
            ));
        }
        
        html.push_str(r#"</div>"#);
    }
    
    html.push_str(r#"</div>"#);
    html
}

// Render network processes view
async fn render_network_processes(process: &Process, session: &PhpSession) -> String {
    let user_id = session.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    
    // Get network processes (downloads, uploads)
    let processes = process.get_user_processes(user_id, Some("network")).await.unwrap_or_default();
    
    let mut html = String::new();
    html.push_str(r#"<div class="processes-container">"#);
    html.push_str(r#"<h3>Network Processes</h3>"#);
    html.push_str(r#"<p>File transfers and network operations</p>"#);
    
    if processes.is_empty() {
        html.push_str(r#"<p>No network processes running.</p>"#);
    } else {
        html.push_str(r#"<div class="network-processes">"#);
        
        for proc in processes {
            let transfer_rate = calculate_transfer_rate(proc.action, proc.total_time, proc.time_left);
            
            html.push_str(&format!(
                r#"<div class="network-process">
                    <div class="process-header">
                        <h4>{} (PID: {})</h4>
                        <span class="process-status {}">{}</span>
                    </div>
                    <div class="process-details">
                        <p><strong>Target:</strong> {}</p>
                        <p><strong>Transfer Rate:</strong> {} KB/s</p>
                        <div class="progress progress-striped active">
                            <div class="progress-bar" style="width: {}%"></div>
                        </div>
                        <div class="process-stats">
                            <span>Progress: {:.1}%</span>
                            <span>Time left: {}</span>
                        </div>
                        <div class="process-actions">
                            <button onclick="pauseProcess({})" class="btn btn-warning">Pause</button>
                            <button onclick="cancelProcess({})" class="btn btn-danger">Cancel</button>
                        </div>
                    </div>
                </div>"#,
                get_action_name(proc.action),
                proc.id,
                if proc.status == "running" { "status-active" } else { "status-paused" },
                proc.status,
                proc.target_ip,
                transfer_rate,
                proc.progress_percent,
                proc.progress_percent,
                format_time_left(proc.time_left),
                proc.id,
                proc.id
            ));
        }
        
        html.push_str(r#"</div>"#);
    }
    
    html.push_str(r#"</div>"#);
    html
}

// Render running processes overview
async fn render_running_processes(process: &Process, session: &PhpSession) -> String {
    let user_id = session.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    
    // Get only actively running processes
    let processes = process.get_running_processes(user_id).await.unwrap_or_default();
    
    let mut html = String::new();
    html.push_str(r#"<div class="processes-container">"#);
    html.push_str(r#"<h3>Currently Running</h3>"#);
    
    if processes.is_empty() {
        html.push_str(r#"
            <div class="alert alert-info">
                <h4>No Active Processes</h4>
                <p>You don't have any processes currently running. Start a new operation from your desktop or software menu.</p>
            </div>
        "#);
    } else {
        html.push_str(&format!(r#"<p>You have {} processes currently running.</p>"#, processes.len()));
        html.push_str(r#"<div class="running-processes-grid">"#);
        
        for proc in processes {
            let icon = get_process_icon(proc.action);
            let color = get_process_color(proc.action);
            
            html.push_str(&format!(
                r#"<div class="process-card" style="border-left-color: {}">
                    <div class="process-icon">{}</div>
                    <div class="process-info">
                        <h5>{}</h5>
                        <p class="process-target">{}</p>
                        <div class="progress-container">
                            <div class="progress">
                                <div class="progress-bar" style="width: {}%; background-color: {}"></div>
                            </div>
                            <span class="progress-text">{:.1}%</span>
                        </div>
                        <p class="process-eta">ETA: {}</p>
                    </div>
                    <div class="process-controls">
                        <a href="processes?pid={}" class="btn btn-sm btn-default" title="View Details">
                            <i class="icon-eye-open"></i>
                        </a>
                        <a href="processes?pid={}&del=1" class="btn btn-sm btn-danger" title="Terminate">
                            <i class="icon-remove"></i>
                        </a>
                    </div>
                </div>"#,
                color,
                icon,
                get_action_name(proc.action),
                proc.target_ip,
                proc.progress_percent,
                color,
                proc.progress_percent,
                format_time_left(proc.time_left),
                proc.id,
                proc.id
            ));
        }
        
        html.push_str(r#"</div>"#);
    }
    
    html.push_str(r#"</div>"#);
    html
}

// Main page template
fn render_processes_page(session: &PhpSession, content: &str, active_tab: &str) -> String {
    let messages = if session.isset_msg() {
        // In real implementation, this would be handled by session middleware
        "".to_string()
    } else {
        "".to_string()
    };
    
    format!(
        r#"<!DOCTYPE html>
        <html>
        <head>
            <title>Processes - Hacker Experience</title>
            <meta charset="UTF-8">
            <link rel="stylesheet" href="css/style.css">
            <link rel="stylesheet" href="css/processes.css">
        </head>
        <body>
            <div id="wrap">
                <div id="header">
                    <h1>Process Manager</h1>
                    <div class="nav-tabs">
                        <a href="processes?page=running" class="{}"">Running</a>
                        <a href="processes?page=all" class="{}">All Processes</a>
                        <a href="processes?page=cpu" class="{}">CPU</a>
                        <a href="processes?page=network" class="{}">Network</a>
                    </div>
                </div>
                
                <div id="main">
                    {}
                    {}
                </div>
            </div>
            
            <script src="js/jquery.js"></script>
            <script src="js/processes.js"></script>
            <script>
                function pauseProcess(pid) {{
                    $.post('ajax.php', {{func: 'pauseProcess', pid: pid}}, function(data) {{
                        if (data.status === 'OK') {{
                            location.reload();
                        }} else {{
                            alert('Error: ' + data.msg);
                        }}
                    }});
                }}
                
                function cancelProcess(pid) {{
                    if (confirm('Are you sure you want to cancel this process?')) {{
                        window.location.href = 'processes?pid=' + pid + '&del=1';
                    }}
                }}
                
                // Auto-refresh every 30 seconds
                setInterval(function() {{
                    location.reload();
                }}, 30000);
            </script>
        </body>
        </html>"#,
        if active_tab == "running" { "nav-active" } else { "" },
        if active_tab == "all" { "nav-active" } else { "" },
        if active_tab == "cpu" { "nav-active" } else { "" },
        if active_tab == "net" { "nav-active" } else { "" },
        messages,
        content
    )
}

fn render_login_required() -> String {
    r#"<!DOCTYPE html>
    <html>
    <head>
        <title>Login Required - Hacker Experience</title>
        <meta charset="UTF-8">
        <link rel="stylesheet" href="css/style.css">
    </head>
    <body>
        <div class="alert alert-error">
            <h4>Authentication Required</h4>
            <p>You must be logged in to access the process manager.</p>
            <a href="index" class="btn btn-primary">Login</a>
        </div>
    </body>
    </html>"#.to_string()
}

// Helper functions
fn get_action_name(action: i32) -> &'static str {
    match action {
        1 => "Download",
        2 => "Upload", 
        3 => "Delete File",
        4 => "Hide File",
        5 => "Seek File",
        7 => "Antivirus Scan",
        8 => "Edit Logs",
        10 => "Format Drive",
        11 => "Hack Password",
        12 => "Hack Bank",
        13 => "Install Software",
        14 => "Uninstall Software",
        15 => "Port Scan",
        16 => "Hack (Advanced)",
        17 => "Research",
        22 => "Network Map",
        23 => "Analyze System",
        24 => "Install Doom Virus",
        25 => "Reset IP",
        26 => "Reset Password",
        27 => "DDoS Attack",
        28 => "Install Webserver",
        _ => "Unknown Process",
    }
}

fn format_time_left(seconds: i32) -> String {
    if seconds <= 0 {
        return "Completed".to_string();
    }
    
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

fn calculate_transfer_rate(action: i32, total_time: i32, time_left: i32) -> i32 {
    // Simplified transfer rate calculation
    match action {
        1 | 2 => { // Download/Upload
            let elapsed = total_time - time_left;
            if elapsed > 0 {
                (1024 * 10) / elapsed.max(1) // Fake calculation for demonstration
            } else {
                0
            }
        },
        _ => 0,
    }
}

fn get_process_icon(action: i32) -> &'static str {
    match action {
        1 => "⬇️", // Download
        2 => "⬆️", // Upload
        11 | 16 => "🔓", // Hacking
        15 => "🔍", // Port Scan
        27 => "💥", // DDoS
        _ => "⚙️", // Generic
    }
}

fn get_process_color(action: i32) -> &'static str {
    match action {
        1 | 2 => "#3498db", // Blue for transfers
        11 | 16 => "#e74c3c", // Red for hacking
        15 => "#f39c12", // Orange for scanning
        27 => "#8e44ad", // Purple for DDoS
        _ => "#95a5a6", // Gray for others
    }
}