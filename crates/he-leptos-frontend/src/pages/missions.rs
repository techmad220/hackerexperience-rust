//! Missions Page - Quest system and objectives

use leptos::*;
use serde::{Deserialize, Serialize};
use crate::api::missions::{
    get_missions, accept_mission, get_available_missions,
    Mission, MissionTemplate
};

#[component]
pub fn MissionsPage() -> impl IntoView {
    // State for missions
    let (active_missions, set_active_missions) = create_signal::<Vec<Mission>>(vec![]);
    let (available_missions, set_available_missions) = create_signal::<Vec<MissionTemplate>>(vec![]);
    let (selected_mission, set_selected_mission) = create_signal::<Option<i64>>(None);
    let (filter_status, set_filter_status) = create_signal("all");
    let (loading, set_loading) = create_signal(false);
    let (error_msg, set_error_msg) = create_signal::<Option<String>>(None);

    // Load missions on mount
    create_effect(move |_| {
        spawn_local(async move {
            set_loading.set(true);

            // Load active missions
            match get_missions().await {
                Ok(missions) => {
                    set_active_missions.set(missions);
                    set_error_msg.set(None);
                }
                Err(e) => {
                    set_error_msg.set(Some(format!("Failed to load missions: {}", e)));
                }
            }

            // Load available missions
            match get_available_missions().await {
                Ok(templates) => {
                    set_available_missions.set(templates);
                }
                Err(e) => {
                    logging::log!("Failed to load available missions: {}", e);
                }
            }

            set_loading.set(false);
        });
    });

    // Filter missions based on status
    let filtered_active_missions = move || {
        let filter = filter_status.get();
        active_missions.get().into_iter().filter(|m| {
            filter == "all" || m.status == filter
        }).collect::<Vec<_>>()
    };

    let active_missions_count = move || {
        active_missions.get().iter().filter(|m| m.status == "active").count()
    };

    // Handle accepting a mission
    let handle_accept = move |mission_id: i64| {
        spawn_local(async move {
            match accept_mission(mission_id).await {
                Ok(true) => {
                    // Reload missions
                    if let Ok(missions) = get_missions().await {
                        set_active_missions.set(missions);
                    }
                    set_error_msg.set(Some("Mission accepted!".to_string()));
                }
                Ok(false) => {
                    set_error_msg.set(Some("Failed to accept mission".to_string()));
                }
                Err(e) => {
                    set_error_msg.set(Some(format!("Error: {}", e)));
                }
            }
        });
    };

    let difficulty_color = |difficulty: &str| -> &str {
        match difficulty.to_lowercase().as_str() {
            "easy" | "tier 1" => "text-green-400",
            "medium" | "tier 2" => "text-yellow-400",
            "hard" | "tier 3" => "text-orange-400",
            "expert" | "elite" | "tier 4" => "text-red-400",
            _ => "text-gray-400"
        }
    };

    view! {
        <div class="missions-page p-4">
            <div class="missions-header flex justify-between items-center mb-4">
                <h1 class="text-3xl font-bold">"Mission Control"</h1>
                <div class="text-sm text-gray-400">
                    "Active Missions: "{active_missions_count()}" / 3"
                </div>
            </div>

            // Error Display
            {move || error_msg.get().map(|msg| view! {
                <div class="mb-4 p-3 bg-gray-800 border border-gray-600 rounded">
                    <span class="text-yellow-400">{msg}</span>
                </div>
            })}

            // Filter buttons
            <div class="mission-filters mb-4 flex gap-2">
                <button
                    class=move || format!("px-3 py-1 rounded {}",
                        if filter_status.get() == "all" {
                            "bg-blue-600 hover:bg-blue-700"
                        } else {
                            "bg-gray-700 hover:bg-gray-600"
                        }
                    )
                    on:click=move |_| set_filter_status.set("all")
                >
                    "All Missions"
                </button>
                <button
                    class=move || format!("px-3 py-1 rounded {}",
                        if filter_status.get() == "active" {
                            "bg-blue-600 hover:bg-blue-700"
                        } else {
                            "bg-gray-700 hover:bg-gray-600"
                        }
                    )
                    on:click=move |_| set_filter_status.set("active")
                >
                    "Active"
                </button>
                <button
                    class=move || format!("px-3 py-1 rounded {}",
                        if filter_status.get() == "completed" {
                            "bg-blue-600 hover:bg-blue-700"
                        } else {
                            "bg-gray-700 hover:bg-gray-600"
                        }
                    )
                    on:click=move |_| set_filter_status.set("completed")
                >
                    "Completed"
                </button>
            </div>

            {move || if loading.get() {
                view! {
                    <div class="text-center py-8">
                        <span class="text-gray-400">"Loading missions..."</span>
                    </div>
                }
            } else {
                view! {
                    <div class="missions-container grid grid-cols-1 lg:grid-cols-2 gap-4">
                        // Active Missions Column
                        <div>
                            <h2 class="text-xl font-bold mb-3">"Your Missions"</h2>
                            <div class="space-y-3">
                                {move || if filtered_active_missions().is_empty() {
                                    view! {
                                        <div class="p-4 bg-gray-800 rounded text-center text-gray-400">
                                            "No active missions"
                                        </div>
                                    }
                                } else {
                                    view! {
                                        <div class="space-y-3">
                                            {filtered_active_missions().into_iter().map(|mission| {
                                                let mission_id = mission.id;
                                                let is_selected = move || selected_mission.get() == Some(mission_id);

                                                view! {
                                                    <div
                                                        class=move || format!(
                                                            "p-4 bg-gray-800 rounded cursor-pointer border-2 {}",
                                                            if is_selected() {
                                                                "border-blue-500"
                                                            } else {
                                                                "border-transparent hover:border-gray-600"
                                                            }
                                                        )
                                                        on:click=move |_| set_selected_mission.set(Some(mission_id))
                                                    >
                                                        <div class="flex justify-between items-start mb-2">
                                                            <h3 class="font-semibold">{mission.mission_type}</h3>
                                                            <span class={format!("text-sm {}",
                                                                match mission.status.as_str() {
                                                                    "active" => "text-green-400",
                                                                    "completed" => "text-blue-400",
                                                                    _ => "text-gray-400"
                                                                }
                                                            )}>
                                                                {mission.status}
                                                            </span>
                                                        </div>

                                                        // Progress bar
                                                        <div class="mb-3">
                                                            <div class="flex justify-between text-xs mb-1">
                                                                <span>"Progress"</span>
                                                                <span>{mission.progress}" / "{mission.total_steps}</span>
                                                            </div>
                                                            <div class="bg-gray-700 rounded h-2">
                                                                <div
                                                                    class="bg-blue-600 h-full rounded"
                                                                    style=format!("width: {}%",
                                                                        if mission.total_steps > 0 {
                                                                            mission.progress * 100 / mission.total_steps
                                                                        } else { 0 }
                                                                    )
                                                                />
                                                            </div>
                                                        </div>

                                                        // Rewards
                                                        <div class="flex justify-between text-sm">
                                                            <span class="text-green-400">
                                                                "$"{mission.reward_money}
                                                            </span>
                                                            <span class="text-yellow-400">
                                                                {mission.reward_xp}" XP"
                                                            </span>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }
                                }}
                            </div>
                        </div>

                        // Available Missions Column
                        <div>
                            <h2 class="text-xl font-bold mb-3">"Available Missions"</h2>
                            <div class="space-y-3">
                                {move || if available_missions.get().is_empty() {
                                    view! {
                                        <div class="p-4 bg-gray-800 rounded text-center text-gray-400">
                                            "No missions available"
                                        </div>
                                    }
                                } else {
                                    view! {
                                        <div class="space-y-3">
                                            {available_missions.get().into_iter().map(|template| {
                                                let template_id = template.id.clone();
                                                let difficulty_class = difficulty_color(&template.difficulty);

                                                view! {
                                                    <div class="p-4 bg-gray-800 rounded">
                                                        <div class="flex justify-between items-start mb-2">
                                                            <h3 class="font-semibold">{template.name.clone()}</h3>
                                                            <span class={format!("text-sm {}", difficulty_class)}>
                                                                {template.difficulty.clone()}
                                                            </span>
                                                        </div>

                                                        <p class="text-sm text-gray-400 mb-3">
                                                            {template.description.clone()}
                                                        </p>

                                                        // Objectives
                                                        <div class="mb-3">
                                                            <h4 class="text-xs font-semibold mb-1">"Objectives:"</h4>
                                                            <ul class="text-xs text-gray-400 space-y-1">
                                                                {template.objectives.clone().into_iter().map(|obj| {
                                                                    view! {
                                                                        <li class="ml-2">"• "{obj}</li>
                                                                    }
                                                                }).collect::<Vec<_>>()}
                                                            </ul>
                                                        </div>

                                                        // Requirements
                                                        {if !template.requirements.is_empty() {
                                                            view! {
                                                                <div class="mb-3">
                                                                    <h4 class="text-xs font-semibold mb-1">"Requirements:"</h4>
                                                                    <ul class="text-xs text-gray-500 space-y-1">
                                                                        {template.requirements.clone().into_iter().map(|req| {
                                                                            view! {
                                                                                <li class="ml-2">"• "{req}</li>
                                                                            }
                                                                        }).collect::<Vec<_>>()}
                                                                    </ul>
                                                                </div>
                                                            }
                                                        } else {
                                                            view! { <div></div> }
                                                        }}

                                                        // Rewards
                                                        <div class="flex justify-between items-center mb-3">
                                                            <div class="flex gap-3 text-sm">
                                                                <span class="text-green-400">
                                                                    "$"{template.reward_money}
                                                                </span>
                                                                <span class="text-yellow-400">
                                                                    {template.reward_xp}" XP"
                                                                </span>
                                                            </div>
                                                            {if !template.reward_items.is_empty() {
                                                                view! {
                                                                    <span class="text-xs text-purple-400">
                                                                        "+ items"
                                                                    </span>
                                                                }
                                                            } else {
                                                                view! { <span></span> }
                                                            }}
                                                        </div>

                                                        <button
                                                            class="w-full px-3 py-1 bg-green-600 hover:bg-green-700 rounded text-sm font-semibold"
                                                            on:click=move |_| {
                                                                // Parse mission ID to i64 if it's a string
                                                                if let Ok(id) = template_id.parse::<i64>() {
                                                                    handle_accept(id);
                                                                }
                                                            }
                                                        >
                                                            "Accept Mission"
                                                        </button>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }
                                }}
                            </div>
                        </div>
                    </div>
                }
            }}
        </div>
    }
}