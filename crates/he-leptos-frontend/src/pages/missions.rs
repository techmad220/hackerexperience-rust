//! Missions Page - Quest system and objectives

use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Mission {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub difficulty: String,
    pub reward_money: u64,
    pub reward_xp: u32,
    pub status: String,
    pub progress: u32,
    pub total_steps: u32,
    pub time_limit: Option<String>,
}

#[component]
pub fn MissionsPage() -> impl IntoView {
    let (missions, set_missions) = create_signal(vec![
        Mission {
            id: 1,
            title: "First Blood".to_string(),
            description: "Successfully hack into your first target system".to_string(),
            difficulty: "Easy".to_string(),
            reward_money: 5000,
            reward_xp: 100,
            status: "completed".to_string(),
            progress: 1,
            total_steps: 1,
            time_limit: None,
        },
        Mission {
            id: 2,
            title: "Data Heist".to_string(),
            description: "Steal confidential files from a corporate server".to_string(),
            difficulty: "Medium".to_string(),
            reward_money: 25000,
            reward_xp: 500,
            status: "active".to_string(),
            progress: 3,
            total_steps: 5,
            time_limit: Some("2 days".to_string()),
        },
        Mission {
            id: 3,
            title: "Banking Breach".to_string(),
            description: "Infiltrate a bank network and transfer funds".to_string(),
            difficulty: "Hard".to_string(),
            reward_money: 100000,
            reward_xp: 1500,
            status: "available".to_string(),
            progress: 0,
            total_steps: 8,
            time_limit: Some("1 week".to_string()),
        },
        Mission {
            id: 4,
            title: "Government Secrets".to_string(),
            description: "Access classified government databases".to_string(),
            difficulty: "Expert".to_string(),
            reward_money: 500000,
            reward_xp: 5000,
            status: "locked".to_string(),
            progress: 0,
            total_steps: 15,
            time_limit: Some("48 hours".to_string()),
        },
        Mission {
            id: 5,
            title: "Virus Outbreak".to_string(),
            description: "Deploy a virus to 100 different systems".to_string(),
            difficulty: "Medium".to_string(),
            reward_money: 50000,
            reward_xp: 750,
            status: "active".to_string(),
            progress: 42,
            total_steps: 100,
            time_limit: None,
        },
    ]);

    let (selected_mission, set_selected_mission) = create_signal::<Option<u32>>(None);
    let (filter_status, set_filter_status) = create_signal("all");

    let filtered_missions = move || {
        let filter = filter_status.get();
        missions.get().into_iter().filter(|m| {
            filter == "all" || m.status == filter
        }).collect::<Vec<_>>()
    };

    let active_missions_count = move || {
        missions.get().iter().filter(|m| m.status == "active").count()
    };

    let difficulty_color = |difficulty: &str| -> &str {
        match difficulty {
            "Easy" => "difficulty-easy",
            "Medium" => "difficulty-medium",
            "Hard" => "difficulty-hard",
            "Expert" => "difficulty-expert",
            _ => "difficulty-unknown"
        }
    };

    view! {
        <div class="missions-page">
            <div class="missions-header">
                <h1>"Mission Control"</h1>
                <div class="mission-stats">
                    <span>"Active Missions: "{active_missions_count()}" / 3"</span>
                </div>
            </div>

            <div class="mission-filters">
                <button
                    class=move || if filter_status.get() == "all" { "filter-btn active" } else { "filter-btn" }
                    on:click=move |_| set_filter_status("all")
                >"All Missions"</button>
                <button
                    class=move || if filter_status.get() == "active" { "filter-btn active" } else { "filter-btn" }
                    on:click=move |_| set_filter_status("active")
                >"Active"</button>
                <button
                    class=move || if filter_status.get() == "available" { "filter-btn active" } else { "filter-btn" }
                    on:click=move |_| set_filter_status("available")
                >"Available"</button>
                <button
                    class=move || if filter_status.get() == "completed" { "filter-btn active" } else { "filter-btn" }
                    on:click=move |_| set_filter_status("completed")
                >"Completed"</button>
            </div>

            <div class="missions-list">
                {move || filtered_missions().into_iter().map(|mission| {
                    let mission_id = mission.id;
                    let is_selected = move || selected_mission.get() == Some(mission_id);

                    view! {
                        <div
                            class=move || format!("mission-card mission-{} {}",
                                mission.status,
                                if is_selected() { "selected" } else { "" }
                            )
                            on:click=move |_| set_selected_mission(Some(mission_id))
                        >
                            <div class="mission-header">
                                <h3>{&mission.title}</h3>
                                <span class=difficulty_color(&mission.difficulty)>
                                    {&mission.difficulty}
                                </span>
                            </div>

                            <p class="mission-description">{&mission.description}</p>

                            {if mission.status == "active" || mission.status == "completed" {
                                view! {
                                    <div class="mission-progress">
                                        <div class="progress-label">
                                            "Progress: "{mission.progress}" / "{mission.total_steps}
                                        </div>
                                        <div class="progress-bar">
                                            <div
                                                class="progress-fill"
                                                style=format!("width: {}%",
                                                    mission.progress * 100 / mission.total_steps
                                                )
                                            />
                                        </div>
                                    </div>
                                }
                            } else {
                                view! { <div></div> }
                            }}

                            <div class="mission-rewards">
                                <span class="reward-money">"💰 $"{mission.reward_money}</span>
                                <span class="reward-xp">"⭐ "{mission.reward_xp}" XP"</span>
                                {if let Some(time_limit) = &mission.time_limit {
                                    view! {
                                        <span class="time-limit">"⏱️ "{time_limit}</span>
                                    }
                                } else {
                                    view! { <span></span> }
                                }}
                            </div>

                            <div class="mission-actions">
                                {match mission.status.as_str() {
                                    "available" => view! {
                                        <button class="btn btn-accept">"Accept Mission"</button>
                                    },
                                    "active" => view! {
                                        <button class="btn btn-abandon">"Abandon"</button>
                                    },
                                    "completed" => view! {
                                        <button class="btn btn-claim">"Claim Rewards"</button>
                                    },
                                    "locked" => view! {
                                        <button class="btn btn-locked" disabled>"Locked"</button>
                                    },
                                    _ => view! { <div></div> }
                                }}
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}