//! Hardware Management Page - View and upgrade computer components

use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct HardwareComponent {
    pub name: String,
    pub component_type: String,
    pub level: u32,
    pub capacity: String,
    pub performance: f32,
    pub price: u64,
    pub upgrade_available: bool,
}

#[component]
pub fn HardwarePage() -> impl IntoView {
    let (components, set_components) = create_signal(vec![
        HardwareComponent {
            name: "Quantum Processor X7".to_string(),
            component_type: "CPU".to_string(),
            level: 5,
            capacity: "7.2 GHz".to_string(),
            performance: 85.0,
            price: 50000,
            upgrade_available: true,
        },
        HardwareComponent {
            name: "HyperRAM 64GB".to_string(),
            component_type: "RAM".to_string(),
            level: 4,
            capacity: "64 GB".to_string(),
            performance: 72.0,
            price: 30000,
            upgrade_available: true,
        },
        HardwareComponent {
            name: "NeuralDisk Pro".to_string(),
            component_type: "HDD".to_string(),
            level: 6,
            capacity: "10 TB".to_string(),
            performance: 90.0,
            price: 75000,
            upgrade_available: false,
        },
        HardwareComponent {
            name: "FiberOptic Network Card".to_string(),
            component_type: "NET".to_string(),
            level: 3,
            capacity: "10 Gbps".to_string(),
            performance: 60.0,
            price: 25000,
            upgrade_available: true,
        },
    ]);

    let (selected_component, set_selected_component) = create_signal::<Option<usize>>(None);
    let (upgrade_modal, set_upgrade_modal) = create_signal(false);

    let total_performance = move || {
        components.get().iter().map(|c| c.performance).sum::<f32>() / 4.0
    };

    view! {
        <div class="hardware-page">
            <div class="hardware-header">
                <h1>"Hardware Configuration"</h1>
                <div class="performance-meter">
                    <span>"Overall Performance: "</span>
                    <div class="progress">
                        <div
                            class="progress-bar"
                            style=move || format!("width: {}%", total_performance())
                        >
                            {move || format!("{:.1}%", total_performance())}
                        </div>
                    </div>
                </div>
            </div>

            <div class="hardware-grid">
                {move || components.get().iter().enumerate().map(|(idx, component)| {
                    let component = component.clone();
                    view! {
                        <div class="hardware-card">
                            <div class="component-icon">
                                {match component.component_type.as_str() {
                                    "CPU" => "⚙️",
                                    "RAM" => "💾",
                                    "HDD" => "💿",
                                    "NET" => "🌐",
                                    _ => "📦"
                                }}
                            </div>
                            <div class="component-info">
                                <h3>{&component.name}</h3>
                                <div class="component-stats">
                                    <span class="stat-label">"Level: "</span>
                                    <span class="stat-value">{component.level}</span>
                                </div>
                                <div class="component-stats">
                                    <span class="stat-label">"Capacity: "</span>
                                    <span class="stat-value">{&component.capacity}</span>
                                </div>
                                <div class="performance-bar">
                                    <div
                                        class="performance-fill"
                                        style=format!("width: {}%", component.performance)
                                    />
                                </div>
                                {if component.upgrade_available {
                                    view! {
                                        <button
                                            class="btn btn-upgrade"
                                            on:click=move |_| {
                                                set_selected_component(Some(idx));
                                                set_upgrade_modal(true);
                                            }
                                        >
                                            "Upgrade - $"{component.price}
                                        </button>
                                    }
                                } else {
                                    view! {
                                        <button class="btn btn-disabled" disabled>
                                            "Max Level"
                                        </button>
                                    }
                                }}
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>

            {move || if upgrade_modal.get() {
                view! {
                    <div class="modal-overlay" on:click=move |_| set_upgrade_modal(false)>
                        <div class="modal-content" on:click=|ev| ev.stop_propagation()>
                            <h2>"Confirm Upgrade"</h2>
                            {if let Some(idx) = selected_component.get() {
                                let component = &components.get()[idx];
                                view! {
                                    <div>
                                        <p>"Upgrade "{&component.name}" to level "{component.level + 1}"?"</p>
                                        <p class="price">"Cost: $"{component.price}</p>
                                        <div class="modal-buttons">
                                            <button
                                                class="btn btn-primary"
                                                on:click=move |_| {
                                                    // Handle upgrade logic
                                                    set_upgrade_modal(false);
                                                }
                                            >
                                                "Confirm"
                                            </button>
                                            <button
                                                class="btn btn-secondary"
                                                on:click=move |_| set_upgrade_modal(false)
                                            >
                                                "Cancel"
                                            </button>
                                        </div>
                                    </div>
                                }
                            } else {
                                view! { <p>"No component selected"</p> }
                            }}
                        </div>
                    </div>
                }
            } else {
                view! { <div></div> }
            }}
        </div>
    }
}