//! Processes Page - Complete UI for managing game processes (100% Rust/WASM)

use leptos::*;
use leptos::html::Progress;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::time::Duration;
use crate::api::process::{get_processes, create_process, cancel_process, toggle_process, Process};

/// Main Processes Page Component
#[component]
pub fn ProcessesPage() -> impl IntoView {
    // Reactive state for processes
    let (processes, set_processes) = create_signal::<Vec<Process>>(vec![]);
    let (selected_process, set_selected_process) = create_signal::<Option<i64>>(None);
    let (show_new_process, set_show_new_process) = create_signal(false);
    let (loading, set_loading) = create_signal(false);
    let (error_msg, set_error_msg) = create_signal::<Option<String>>(None);

    // Resource metrics
    let (cpu_available, set_cpu_available) = create_signal(2000.0); // 2GHz
    let (ram_available, set_ram_available) = create_signal(4096.0); // 4GB

    // Computed resource usage
    let cpu_used = move || {
        processes.get().iter()
            .filter(|p| p.state == "running")
            .map(|p| p.cpu_usage as f32)
            .sum::<f32>()
    };

    let ram_used = move || {
        processes.get().iter()
            .filter(|p| p.state == "running")
            .map(|p| p.ram_usage as f32)
            .sum::<f32>()
    };

    // Load processes on mount and refresh periodically
    create_effect(move |_| {
        spawn_local(async move {
            set_loading.set(true);
            match get_processes().await {
                Ok(procs) => {
                    set_processes.set(procs);
                    set_error_msg.set(None);
                }
                Err(e) => {
                    set_error_msg.set(Some(format!("Failed to load processes: {}", e)));
                }
            }
            set_loading.set(false);
        });
    });

    // Auto-refresh processes periodically
    use gloo_timers::future::IntervalStream;
    use futures_util::stream::StreamExt;

    create_effect(move |_| {
        spawn_local(async move {
            let mut interval = IntervalStream::new(2000); // 2 seconds
            while interval.next().await.is_some() {
                if let Ok(procs) = get_processes().await {
                    set_processes.set(procs);
                }
            }
        });
    });

    // Handle process cancellation
    let handle_cancel = move |pid: i64| {
        spawn_local(async move {
            match cancel_process(pid).await {
                Ok(_) => {
                    // Refresh process list
                    if let Ok(procs) = get_processes().await {
                        set_processes.set(procs);
                    }
                }
                Err(e) => {
                    set_error_msg.set(Some(format!("Failed to cancel process: {}", e)));
                }
            }
        });
    };

    // Handle process pause/resume
    let handle_toggle = move |pid: i64| {
        spawn_local(async move {
            match toggle_process(pid).await {
                Ok(_) => {
                    // Refresh process list
                    if let Ok(procs) = get_processes().await {
                        set_processes.set(procs);
                    }
                }
                Err(e) => {
                    set_error_msg.set(Some(format!("Failed to toggle process: {}", e)));
                }
            }
        });
    };

    // Handle new process creation
    let handle_new_process = move |process_type: String, target: Option<String>, priority: String| {
        let priority_val = match priority.as_str() {
            "Critical" => 4,
            "High" => 3,
            "Normal" => 2,
            _ => 1,
        };

        spawn_local(async move {
            match create_process(process_type, target, priority_val).await {
                Ok(_) => {
                    set_show_new_process.set(false);
                    // Refresh process list
                    if let Ok(procs) = get_processes().await {
                        set_processes.set(procs);
                    }
                }
                Err(e) => {
                    set_error_msg.set(Some(format!("Failed to create process: {}", e)));
                }
            }
        });
    };

    view! {
        <div class="processes-page p-4">
            <div class="page-header flex justify-between items-center mb-4">
                <h1 class="text-3xl font-bold">"Process Manager"</h1>
                <button
                    class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded font-semibold"
                    on:click=move |_| set_show_new_process(!show_new_process())
                >
                    "+ New Process"
                </button>
            </div>

            // Error Display
            {move || error_msg.get().map(|msg| view! {
                <div class="mb-4 p-3 bg-red-900 border border-red-700 rounded">
                    <span class="text-red-200">{msg}</span>
                </div>
            })}

            // Resource Usage Panel
            <div class="resource-panel mb-6 p-4 bg-gray-800 rounded">
                <h2 class="text-xl font-bold mb-3">"System Resources"</h2>
                <div class="grid grid-cols-2 gap-4">
                    <ResourceMeter
                        label="CPU"
                        used=cpu_used()
                        available=cpu_available.get()
                        unit="MHz"
                    />
                    <ResourceMeter
                        label="RAM"
                        used=ram_used()
                        available=ram_available.get()
                        unit="MB"
                    />
                </div>
            </div>

            // New Process Dialog
            <Show when=move || show_new_process()>
                <NewProcessDialog
                    on_close=move || set_show_new_process(false)
                    on_submit=handle_new_process
                />
            </Show>

            // Process List
            <div class="process-list">
                <h2 class="text-xl font-bold mb-3">"Active Processes"</h2>

                {move || if loading.get() {
                    view! {
                        <div class="text-center py-8">
                            <span class="text-gray-400">"Loading processes..."</span>
                        </div>
                    }
                } else if processes.get().is_empty() {
                    view! {
                        <div class="text-center py-8">
                            <span class="text-gray-400">"No active processes"</span>
                        </div>
                    }
                } else {
                    view! {
                        <div class="process-table">
                            <div class="grid grid-cols-9 gap-2 p-2 bg-gray-700 font-semibold text-sm">
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
                                each=move || processes.get()
                                key=|p| p.pid
                                children=move |process: Process| {
                                    view! {
                                        <ProcessRow
                                            process=process.clone()
                                            selected=move || selected_process.get() == Some(process.pid)
                                            on_select=move || set_selected_process.set(Some(process.pid))
                                            on_toggle=move || handle_toggle(process.pid)
                                            on_cancel=move || handle_cancel(process.pid)
                                        />
                                    }
                                }
                            />
                        </div>
                    }
                }}
            </div>

            // Process Details Panel
            <Show when=move || selected_process.get().is_some()>
                <div class="process-details mt-6 p-4 bg-gray-800 rounded">
                    <h2 class="text-xl font-bold mb-3">"Process Details"</h2>
                    <For
                        each=move || processes.get().into_iter().filter(|p| Some(p.pid) == selected_process.get())
                        key=|p| p.pid
                        children=move |process: Process| {
                            view! {
                                <div class="grid grid-cols-2 gap-4 text-sm">
                                    <div>
                                        <span class="text-gray-400">"Process ID: "</span>
                                        <span class="font-mono">{process.pid}</span>
                                    </div>
                                    <div>
                                        <span class="text-gray-400">"Type: "</span>
                                        <span>{process.process_type}</span>
                                    </div>
                                    <div>
                                        <span class="text-gray-400">"Source: "</span>
                                        <span class="font-mono">{process.source_server}</span>
                                    </div>
                                    <div>
                                        <span class="text-gray-400">"Target: "</span>
                                        <span class="font-mono">
                                            {process.target_server.unwrap_or("Local".to_string())}
                                        </span>
                                    </div>
                                    <div>
                                        <span class="text-gray-400">"Started: "</span>
                                        <span>{process.started_at}</span>
                                    </div>
                                    <div>
                                        <span class="text-gray-400">"Progress: "</span>
                                        <div class="inline-block w-32">
                                            <div class="bg-gray-700 rounded h-4 relative">
                                                <div
                                                    class="bg-green-600 h-full rounded"
                                                    style=move || format!("width: {}%", process.completion_percentage)
                                                />
                                                <span class="absolute inset-0 text-center text-xs leading-4">
                                                    {format!("{:.1}%", process.completion_percentage)}
                                                </span>
                                            </div>
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
    used: f32,
    available: f32,
    unit: &'static str,
) -> impl IntoView {
    let percentage = (used / available * 100.0).min(100.0);
    let color = if percentage > 90.0 {
        "bg-red-600"
    } else if percentage > 70.0 {
        "bg-yellow-600"
    } else {
        "bg-green-600"
    };

    view! {
        <div class="resource-meter">
            <div class="flex justify-between mb-1">
                <span class="text-sm font-semibold">{label}</span>
                <span class="text-sm text-gray-400">
                    {format!("{:.0} / {:.0} {}", used, available, unit)}
                </span>
            </div>
            <div class="bg-gray-700 rounded h-4 relative">
                <div
                    class=format!("{} h-full rounded", color)
                    style=format!("width: {}%", percentage)
                />
                <span class="absolute inset-0 text-center text-xs leading-4">
                    {format!("{:.1}%", percentage)}
                </span>
            </div>
        </div>
    }
}

/// Process Row Component
#[component]
fn ProcessRow(
    process: Process,
    selected: impl Fn() -> bool + 'static,
    on_select: impl Fn() + 'static,
    on_toggle: impl Fn() + 'static,
    on_cancel: impl Fn() + 'static,
) -> impl IntoView {
    let state_color = match process.state.as_str() {
        "running" => "text-green-400",
        "paused" => "text-yellow-400",
        "queued" => "text-gray-400",
        "completed" => "text-blue-400",
        _ => "text-gray-500",
    };

    let priority_color = match process.priority {
        4 => "text-red-400", // Critical
        3 => "text-orange-400", // High
        2 => "text-gray-300", // Normal
        _ => "text-gray-500", // Low
    };

    let priority_text = match process.priority {
        4 => "Critical",
        3 => "High",
        2 => "Normal",
        _ => "Low",
    };

    view! {
        <div
            class=move || format!(
                "grid grid-cols-9 gap-2 p-2 hover:bg-gray-700 cursor-pointer {}",
                if selected() { "bg-gray-700" } else { "" }
            )
            on:click=move |_| on_select()
        >
            <span class="text-sm">{process.process_type.clone()}</span>
            <span class="text-sm font-mono text-gray-400">
                {process.target_server.clone().unwrap_or("Local".to_string())}
            </span>
            <span class=format!("text-sm {}", state_color)>{process.state.clone()}</span>
            <span class=format!("text-sm {}", priority_color)>{priority_text}</span>
            <div class="flex items-center">
                <div class="flex-1 bg-gray-700 rounded h-2 mr-2">
                    <div
                        class="bg-blue-600 h-full rounded"
                        style=format!("width: {}%", process.completion_percentage)
                    />
                </div>
                <span class="text-xs">{format!("{:.0}%", process.completion_percentage)}</span>
            </div>
            <span class="text-sm">{format!("{}s", process.estimated_time_remaining)}</span>
            <span class="text-sm">{process.cpu_usage}</span>
            <span class="text-sm">{process.ram_usage}</span>
            <div class="flex gap-1">
                {if process.state == "running" {
                    view! {
                        <button
                            class="px-2 py-1 bg-yellow-600 hover:bg-yellow-700 rounded text-xs"
                            on:click=move |e| {
                                e.stop_propagation();
                                on_toggle();
                            }
                        >
                            "⏸"
                        </button>
                    }
                } else if process.state == "paused" {
                    view! {
                        <button
                            class="px-2 py-1 bg-green-600 hover:bg-green-700 rounded text-xs"
                            on:click=move |e| {
                                e.stop_propagation();
                                on_toggle();
                            }
                        >
                            "▶"
                        </button>
                    }
                } else {
                    view! { <span></span> }
                }}
                <button
                    class="px-2 py-1 bg-red-600 hover:bg-red-700 rounded text-xs"
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
    F: Fn() + 'static + Copy,
    G: Fn(String, Option<String>, String) + 'static,
{
    let (process_type, set_process_type) = create_signal("scan".to_string());
    let (target_ip, set_target_ip) = create_signal("".to_string());
    let (priority, set_priority) = create_signal("Normal".to_string());

    view! {
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
             on:click=move |_| on_close()>
            <div class="bg-gray-800 p-6 rounded-lg max-w-md w-full"
                 on:click=|e| e.stop_propagation()>
                <h2 class="text-xl font-bold mb-4">"Start New Process"</h2>

                <div class="mb-4">
                    <label class="block text-sm font-semibold mb-2">"Process Type"</label>
                    <select
                        class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded"
                        on:change=move |e| set_process_type.set(event_target_value(&e))
                    >
                        <option value="scan">"Port Scan"</option>
                        <option value="crack">"Password Crack"</option>
                        <option value="exploit">"Exploit"</option>
                        <option value="download">"Download"</option>
                        <option value="upload">"Upload"</option>
                        <option value="install">"Install Software"</option>
                        <option value="ddos">"DDoS Attack"</option>
                        <option value="mine">"Bitcoin Mining"</option>
                        <option value="research">"Research"</option>
                        <option value="log_delete">"Delete Logs"</option>
                    </select>
                </div>

                <div class="mb-4">
                    <label class="block text-sm font-semibold mb-2">"Target IP (optional)"</label>
                    <input
                        type="text"
                        class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded"
                        placeholder="192.168.1.1"
                        on:input=move |e| set_target_ip.set(event_target_value(&e))
                    />
                </div>

                <div class="mb-4">
                    <label class="block text-sm font-semibold mb-2">"Priority"</label>
                    <select
                        class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded"
                        on:change=move |e| set_priority.set(event_target_value(&e))
                    >
                        <option value="Low">"Low"</option>
                        <option value="Normal" selected>"Normal"</option>
                        <option value="High">"High"</option>
                        <option value="Critical">"Critical"</option>
                    </select>
                </div>

                <div class="flex justify-end gap-2">
                    <button
                        class="px-4 py-2 bg-gray-600 hover:bg-gray-700 rounded"
                        on:click=move |_| on_close()
                    >
                        "Cancel"
                    </button>
                    <button
                        class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded font-semibold"
                        on:click=move |_| {
                            let target = if target_ip.get().is_empty() {
                                None
                            } else {
                                Some(target_ip.get())
                            };
                            on_submit(process_type.get(), target, priority.get());
                        }
                    >
                        "Start Process"
                    </button>
                </div>
            </div>
        </div>
    }
}