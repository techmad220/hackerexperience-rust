//! Profile Page - User settings and statistics

use leptos::*;
use serde::{Deserialize, Serialize};

use crate::api::progression::{
    get_progression, get_statistics, get_achievements,
    PlayerProgression, PlayerStatistics, AchievementProgress,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub username: String,
    pub email: String,
    pub level: u32,
    pub experience: u64,
    pub reputation: i32,
    pub clan: Option<String>,
    pub created_at: String,
    pub last_login: String,
    pub total_hacks: u32,
    pub successful_hacks: u32,
    pub money_earned: u64,
    pub viruses_created: u32,
}

#[component]
pub fn ProfilePage() -> impl IntoView {
    // Load progression data from backend
    let progression = create_resource(
        || (),
        |_| async move { get_progression().await },
    );

    let statistics = create_resource(
        || (),
        |_| async move { get_statistics().await },
    );

    let achievements = create_resource(
        || (),
        |_| async move { get_achievements().await },
    );

    let (edit_mode, set_edit_mode) = create_signal(false);
    let (settings_tab, set_settings_tab) = create_signal("profile");

    view! {
        <div class="profile-page">
            <div class="profile-header">
                <h1>"Player Profile"</h1>
                <div class="profile-actions">
                    <button
                        class="btn btn-secondary"
                        on:click=move |_| set_edit_mode(!edit_mode.get())
                    >
                        {move || if edit_mode.get() { "Save" } else { "Edit Profile" }}
                    </button>
                </div>
            </div>

            <div class="profile-tabs">
                <button
                    class=move || if settings_tab.get() == "profile" { "tab active" } else { "tab" }
                    on:click=move |_| set_settings_tab("profile")
                >"Profile"</button>
                <button
                    class=move || if settings_tab.get() == "statistics" { "tab active" } else { "tab" }
                    on:click=move |_| set_settings_tab("statistics")
                >"Statistics"</button>
                <button
                    class=move || if settings_tab.get() == "security" { "tab active" } else { "tab" }
                    on:click=move |_| set_settings_tab("security")
                >"Security"</button>
                <button
                    class=move || if settings_tab.get() == "preferences" { "tab active" } else { "tab" }
                    on:click=move |_| set_settings_tab("preferences")
                >"Preferences"</button>
            </div>

            <div class="profile-content">
                {move || match settings_tab.get() {
                    "profile" => view! {
                        <div class="profile-info">
                            <div class="info-group">
                                <label>"Username"</label>
                                {move || if edit_mode.get() {
                                    view! {
                                        <input
                                            type="text"
                                            value=profile.get().username
                                            class="form-control"
                                        />
                                    }
                                } else {
                                    view! { <p>{profile.get().username}</p> }
                                }}
                            </div>

                            <div class="info-group">
                                <label>"Email"</label>
                                {move || if edit_mode.get() {
                                    view! {
                                        <input
                                            type="email"
                                            value=profile.get().email
                                            class="form-control"
                                        />
                                    }
                                } else {
                                    view! { <p>{profile.get().email}</p> }
                                }}
                            </div>

                            <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                                {move || {
                                    progression.get().map(|result| {
                                        match result {
                                            Ok(prog) => view! {
                                                <>
                                                    <div class="info-group">
                                                        <label>"Level"</label>
                                                        <p>{prog.level_info.level}</p>
                                                    </div>

                                                    <div class="info-group">
                                                        <label>"Experience"</label>
                                                        <div class="progress">
                                                            <div
                                                                class="progress-bar"
                                                                style=move || format!("width: {}%",
                                                                    (prog.level_info.current_experience as f32 /
                                                                     prog.level_info.experience_to_next as f32 * 100.0) as u32)
                                                            >
                                                                {prog.level_info.current_experience}" / "{prog.level_info.experience_to_next}" XP"
                                                            </div>
                                                        </div>
                                                    </div>

                                                    <div class="info-group">
                                                        <label>"Total Reputation"</label>
                                                        <p class="reputation">{prog.reputation.total_reputation}</p>
                                                    </div>

                                                    <div class="info-group">
                                                        <label>"Skill Points Available"</label>
                                                        <p>{prog.skill_tree.skill_points_available}</p>
                                                    </div>

                                                    <div class="info-group">
                                                        <label>"Achievements"</label>
                                                        <p>{prog.achievements.unlocked_achievements.len()}" unlocked"</p>
                                                    </div>
                                                </>
                                            },
                                            Err(_) => view! {
                                                <div class="error">"Failed to load progression data"</div>
                                            },
                                        }
                                    })
                                }}
                            </Suspense>

                            <div class="info-group">
                                <label>"Clan"</label>
                                <p>{profile.get().clan.unwrap_or("No clan".to_string())}</p>
                            </div>
                        </div>
                    },
                    "statistics" => view! {
                        <Suspense fallback=move || view! { <div>"Loading statistics..."</div> }>
                            {move || {
                                statistics.get().map(|result| {
                                    match result {
                                        Ok(stats) => view! {
                                            <div class="statistics-grid">
                                                <div class="stat-card">
                                                    <h3>"Hacking Stats"</h3>
                                                    <div class="stat-item">
                                                        <span>"Servers Hacked:"</span>
                                                        <strong>{stats.servers_hacked}</strong>
                                                    </div>
                                                    <div class="stat-item">
                                                        <span>"Missions Completed:"</span>
                                                        <strong>{stats.missions_completed}</strong>
                                                    </div>
                                                </div>

                                                <div class="stat-card">
                                                    <h3>"Financial Stats"</h3>
                                                    <div class="stat-item">
                                                        <span>"Money Earned:"</span>
                                                        <strong>"$"{stats.money_earned}</strong>
                                                    </div>
                                                </div>

                                                <div class="stat-card">
                                                    <h3>"PvP Stats"</h3>
                                                    <div class="stat-item">
                                                        <span>"Wins:"</span>
                                                        <strong>{stats.pvp_wins}</strong>
                                                    </div>
                                                    <div class="stat-item">
                                                        <span>"Losses:"</span>
                                                        <strong>{stats.pvp_losses}</strong>
                                                    </div>
                                                    <div class="stat-item">
                                                        <span>"Win Rate:"</span>
                                                        <strong>
                                                            {format!("{}%",
                                                                if stats.pvp_wins + stats.pvp_losses > 0 {
                                                                    stats.pvp_wins * 100 / (stats.pvp_wins + stats.pvp_losses)
                                                                } else { 0 }
                                                            )}
                                                        </strong>
                                                    </div>
                                                </div>

                                                <div class="stat-card">
                                                    <h3>"Playtime"</h3>
                                                    <div class="stat-item">
                                                        <span>"Time Played:"</span>
                                                        <strong>{format_time(stats.time_played_seconds)}</strong>
                                                    </div>
                                                </div>
                                            </div>
                                        },
                                        Err(_) => view! {
                                            <div class="error">"Failed to load statistics"</div>
                                        },
                                    }
                                })
                            }}
                        </Suspense>
                    },
                    "security" => view! {
                        <div class="security-settings">
                            <h3>"Security Settings"</h3>
                            <div class="form-group">
                                <label>"Change Password"</label>
                                <input type="password" placeholder="Current Password" class="form-control" />
                                <input type="password" placeholder="New Password" class="form-control" />
                                <input type="password" placeholder="Confirm New Password" class="form-control" />
                                <button class="btn btn-primary">"Update Password"</button>
                            </div>

                            <div class="form-group">
                                <label>"Two-Factor Authentication"</label>
                                <button class="btn btn-success">"Enable 2FA"</button>
                            </div>

                            <div class="form-group">
                                <label>"Active Sessions"</label>
                                <div class="session-list">
                                    <div class="session-item">
                                        <span>"Current Session - "</span>
                                        <span>"Browser on Windows"</span>
                                        <button class="btn btn-sm btn-danger">"Revoke"</button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    },
                    _ => view! {
                        <div class="preferences">
                            <h3>"Preferences"</h3>
                            <div class="preference-item">
                                <label>
                                    <input type="checkbox" checked=true />
                                    " Email notifications"
                                </label>
                            </div>
                            <div class="preference-item">
                                <label>
                                    <input type="checkbox" checked=false />
                                    " Show online status"
                                </label>
                            </div>
                            <div class="preference-item">
                                <label>
                                    <input type="checkbox" checked=true />
                                    " Allow clan invites"
                                </label>
                            </div>
                            <button class="btn btn-primary">"Save Preferences"</button>
                        </div>
                    }
                }}
            </div>
        </div>
    }
}

fn format_time(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}