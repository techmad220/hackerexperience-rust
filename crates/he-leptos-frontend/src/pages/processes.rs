//! Processes Page - Complete UI for managing game processes (100% Rust/WASM)

use leptos::*;
use leptos::html::Progress;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::time::Duration;

/// Process information from the game engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub id: Uuid,
    pub process_type: String,
    pub state: String,
    pub priority: String,
    pub progress: f32,
    pub time_remaining: String,
    pub cpu_usage: f32,
    pub ram_usage: f32,
    pub target: Option<String>,
}

/// Main Processes Page Component
#[component]
pub fn ProcessesPage() -> impl IntoView {
    // Reactive state for processes
    let (processes, set_processes) = create_signal(vec![]);
    let (selected_process, set_selected_process) = create_signal::<Option<Uuid>>(None);
    let (show_new_process, set_show_new_process) = create_signal(false);

    // Resource metrics
    let (cpu_available, set_cpu_available) = create_signal(1000.0);
    let (cpu_used, set_cpu_used) = create_signal(0.0);
    let (ram_available, set_ram_available) = create_signal(1024.0);
    let (ram_used, set_ram_used) = create_signal(0.0);

    // Simulate fetching processes from the game engine
    create_effect(move |_| {
        // In real implementation, this would call the game engine API
        let demo_processes = vec![
            ProcessInfo {
                id: Uuid::new_v4(),
                process_type: "Crack".to_string(),
                state: "Running".to_string(),
                priority: "Normal".to_string(),
                progress: 45.5,
                time_remaining: "2m 30s".to_string(),
                cpu_usage: 350.0,
                ram_usage: 128.0,
                target: Some("192.168.1.100".to_string()),
            },
            ProcessInfo {
                id: Uuid::new_v4(),
                process_type: "Download".to_string(),
                state: "Running".to_string(),
                priority: "Low".to_string(),
                progress: 78.0,
                time_remaining: "45s".to_string(),
                cpu_usage: 100.0,
                ram_usage: 64.0,
                target: Some("files.example.com".to_string()),
            },
            ProcessInfo {
                id: Uuid::new_v4(),
                process_type: "AntiVirus Scan".to_string(),
                state: "Queued".to_string(),
                priority: "High".to_string(),
                progress: 0.0,
                time_remaining: "Waiting...".to_string(),
                cpu_usage: 0.0,
                ram_usage: 0.0,
                target: None,
            },
        ];

        set_processes(demo_processes.clone());

        // Calculate resource usage
        let total_cpu: f32 = demo_processes.iter()
            .filter(|p| p.state == "Running")
            .map(|p| p.cpu_usage)
            .sum();
        let total_ram: f32 = demo_processes.iter()
            .filter(|p| p.state == "Running")
            .map(|p| p.ram_usage)
            .sum();

        set_cpu_used(total_cpu);
        set_ram_used(total_ram);
    });

    view! {
        <div class="processes-page">
            <div class="page-header">
                <h1>"Process Manager"</h1>
                <button
                    class="btn-primary"
                    on:click=move |_| set_show_new_process(!show_new_process())
                >
                    "+ New Process"
                </button>
            </div>

            // Resource Usage Panel
            <div class="resource-panel">
                <h2>"System Resources"</h2>
                <div class="resource-grid">
                    <ResourceMeter
                        label="CPU"
                        used=cpu_used
                        available=cpu_available
                        unit="MHz"
                    />
                    <ResourceMeter
                        label="RAM"
                        used=ram_used
                        available=ram_available
                        unit="MB"
                    />
                </div>
            </div>

            // New Process Dialog
            <Show when=move || show_new_process()>
                <NewProcessDialog
                    on_close=move || set_show_new_process(false)
                    on_submit=move |process_type, target, priority| {
                        // Here we would call the game engine to start the process
                        log::info!("Starting process: {} -> {:?}", process_type, target);
                        set_show_new_process(false);
                    }
                />
            </Show>

            // Process List
            <div class="process-list">
                <h2>"Active Processes"</h2>
                <div class="process-table">
                    <div class="table-header">
                        <span>"Type"</span>
                        <span>"Target"</span>
                        <span>"State"</span>
                        <span>"Priority"</span>
                        <span>"Progress"</span>
                        <span>"Time"</span>
                        <span>"CPU"</span>
                        <span>"RAM"</span>
                        <span>"Actions"</span>
                    </div>
                    <For
                        each=move || processes()
                        key=|p| p.id
                        children=move |process: ProcessInfo| {
                            view! {
                                <ProcessRow
                                    process=process.clone()
                                    selected=move || selected_process() == Some(process.id)
                                    on_select=move || set_selected_process(Some(process.id))
                                    on_pause=move || {
                                        log::info!("Pausing process: {}", process.id);
                                    }
                                    on_cancel=move || {
                                        log::info!("Cancelling process: {}", process.id);
                                    }
                                />
                            }
                        }
                    />
                </div>
            </div>

            // Process Details Panel
            <Show when=move || selected_process().is_some()>
                <div class="process-details">
                    <h2>"Process Details"</h2>
                    <For
                        each=move || processes().into_iter().filter(|p| Some(p.id) == selected_process())
                        key=|p| p.id
                        children=move |process: ProcessInfo| {
                            view! {
                                <div class="details-content">
                                    <div class="detail-row">
                                        <span class="label">"Process ID:"</span>
                                        <span class="value">{process.id.to_string()}</span>
                                    </div>
                                    <div class="detail-row">
                                        <span class="label">"Type:"</span>
                                        <span class="value">{process.process_type}</span>
                                    </div>
                                    <div class="detail-row">
                                        <span class="label">"Target:"</span>
                                        <span class="value">{process.target.unwrap_or("Local".to_string())}</span>
                                    </div>
                                    <div class="detail-row">
                                        <span class="label">"Progress:"</span>
                                        <div class="progress-bar-container">
                                            <div
                                                class="progress-bar-fill"
                                                style=move || format!("width: {}%", process.progress)
                                            />
                                            <span class="progress-text">{format!("{:.1}%", process.progress)}</span>
                                        </div>
                                    </div>
                                </div>
                            }
                        }
                    />
                </div>
            </Show>
        </div>
    }
}

/// Resource Meter Component
#[component]
fn ResourceMeter(
    label: &'static str,
    used: ReadSignal<f32>,
    available: ReadSignal<f32>,
    unit: &'static str,
) -> impl IntoView {
    let percentage = move || (used() / available() * 100.0).min(100.0);
    let color = move || {
        let pct = percentage();
        if pct > 90.0 { "red" }
        else if pct > 70.0 { "yellow" }
        else { "green" }
    };

    view! {
        <div class="resource-meter">
            <div class="meter-header">
                <span class="meter-label">{label}</span>
                <span class="meter-value">
                    {move || format!("{:.0} / {:.0} {}", used(), available(), unit)}
                </span>
            </div>
            <div class="meter-bar">
                <div
                    class=move || format!("meter-fill {}", color())
                    style=move || format!("width: {}%", percentage())
                />
            </div>
            <span class="meter-percentage">{move || format!("{:.1}%", percentage())}</span>
        </div>
    }
}

/// Process Row Component
#[component]
fn ProcessRow(
    process: ProcessInfo,
    selected: impl Fn() -> bool + 'static,
    on_select: impl Fn() + 'static,
    on_pause: impl Fn() + 'static,
    on_cancel: impl Fn() + 'static,
) -> impl IntoView {
    let state_class = match process.state.as_str() {
        "Running" => "state-running",
        "Paused" => "state-paused",
        "Queued" => "state-queued",
        _ => "state-completed",
    };

    let priority_class = match process.priority.as_str() {
        "Critical" => "priority-critical",
        "High" => "priority-high",
        "Normal" => "priority-normal",
        _ => "priority-low",
    };

    view! {
        <div
            class=move || format!("process-row {} {}",
                if selected() { "selected" } else { "" },
                state_class
            )
            on:click=move |_| on_select()
        >
            <span class="process-type">{process.process_type.clone()}</span>
            <span class="process-target">{process.target.unwrap_or("Local".to_string())}</span>
            <span class=format!("process-state {}", state_class)>{process.state.clone()}</span>
            <span class=format!("process-priority {}", priority_class)>{process.priority.clone()}</span>
            <div class="process-progress">
                <div class="mini-progress-bar">
                    <div
                        class="mini-progress-fill"
                        style=format!("width: {}%", process.progress)
                    />
                </div>
                <span class="progress-text">{format!("{:.0}%", process.progress)}</span>
            </div>
            <span class="process-time">{process.time_remaining.clone()}</span>
            <span class="process-cpu">{format!("{:.0}", process.cpu_usage)}</span>
            <span class="process-ram">{format!("{:.0}", process.ram_usage)}</span>
            <div class="process-actions">
                {if process.state == "Running" {
                    view! {
                        <button
                            class="btn-small btn-pause"
                            on:click=move |e| {
                                e.stop_propagation();
                                on_pause();
                            }
                        >
                            "⏸"
                        </button>
                    }.into_view()
                } else if process.state == "Paused" {
                    view! {
                        <button
                            class="btn-small btn-resume"
                            on:click=move |e| {
                                e.stop_propagation();
                                on_pause();
                            }
                        >
                            "▶"
                        </button>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }}
                <button
                    class="btn-small btn-cancel"
                    on:click=move |e| {
                        e.stop_propagation();
                        on_cancel();
                    }
                >
                    "✖"
                </button>
            </div>
        </div>
    }
}

/// New Process Dialog Component
#[component]
fn NewProcessDialog<F, G>(
    on_close: F,
    on_submit: G,
) -> impl IntoView
where
    F: Fn() + 'static,
    G: Fn(String, Option<String>, String) + 'static,
{
    let (process_type, set_process_type) = create_signal("Crack".to_string());
    let (target_ip, set_target_ip) = create_signal("".to_string());
    let (priority, set_priority) = create_signal("Normal".to_string());

    view! {
        <div class="modal-overlay" on:click=move |_| on_close()>
            <div class="modal-content" on:click=|e| e.stop_propagation()>
                <h2>"Start New Process"</h2>

                <div class="form-group">
                    <label>"Process Type"</label>
                    <select on:change=move |e| set_process_type(event_target_value(&e))>
                        <option value="Crack">"Crack"</option>
                        <option value="Download">"Download"</option>
                        <option value="Upload">"Upload"</option>
                        <option value="Install">"Install"</option>
                        <option value="Scan">"Port Scan"</option>
                        <option value="DDoS">"DDoS Attack"</option>
                        <option value="Mine">"Bitcoin Mine"</option>
                        <option value="Research">"Research"</option>
                    </select>
                </div>

                <div class="form-group">
                    <label>"Target IP (optional)"</label>
                    <input
                        type="text"
                        placeholder="192.168.1.1"
                        on:input=move |e| set_target_ip(event_target_value(&e))
                    />
                </div>

                <div class="form-group">
                    <label>"Priority"</label>
                    <select on:change=move |e| set_priority(event_target_value(&e))>
                        <option value="Low">"Low"</option>
                        <option value="Normal" selected>"Normal"</option>
                        <option value="High">"High"</option>
                        <option value="Critical">"Critical"</option>
                    </select>
                </div>

                <div class="modal-actions">
                    <button class="btn-secondary" on:click=move |_| on_close()>
                        "Cancel"
                    </button>
                    <button
                        class="btn-primary"
                        on:click=move |_| {
                            let target = if target_ip().is_empty() {
                                None
                            } else {
                                Some(target_ip())
                            };
                            on_submit(process_type(), target, priority());
                        }
                    >
                        "Start Process"
                    </button>
                </div>
            </div>
        </div>
    }
}