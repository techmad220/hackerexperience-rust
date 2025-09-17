//! Profile Page - User settings and statistics

use leptos::*;
use serde::{Deserialize, Serialize};

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
    let (profile, set_profile) = create_signal(UserProfile {
        username: "Neo".to_string(),
        email: "neo@matrix.com".to_string(),
        level: 42,
        experience: 98500,
        reputation: 1337,
        clan: Some("The Architects".to_string()),
        created_at: "2024-01-01".to_string(),
        last_login: "2025-09-17".to_string(),
        total_hacks: 523,
        successful_hacks: 489,
        money_earned: 12500000,
        viruses_created: 67,
    });

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

                            <div class="info-group">
                                <label>"Level"</label>
                                <p>{profile.get().level}</p>
                            </div>

                            <div class="info-group">
                                <label>"Experience"</label>
                                <div class="progress">
                                    <div
                                        class="progress-bar"
                                        style=format!("width: {}%", profile.get().experience % 1000 / 10)
                                    >
                                        {profile.get().experience}" XP"
                                    </div>
                                </div>
                            </div>

                            <div class="info-group">
                                <label>"Reputation"</label>
                                <p class="reputation">{profile.get().reputation}</p>
                            </div>

                            <div class="info-group">
                                <label>"Clan"</label>
                                <p>{profile.get().clan.unwrap_or("No clan".to_string())}</p>
                            </div>
                        </div>
                    },
                    "statistics" => view! {
                        <div class="statistics-grid">
                            <div class="stat-card">
                                <h3>"Hacking Stats"</h3>
                                <div class="stat-item">
                                    <span>"Total Hacks:"</span>
                                    <strong>{profile.get().total_hacks}</strong>
                                </div>
                                <div class="stat-item">
                                    <span>"Successful:"</span>
                                    <strong>{profile.get().successful_hacks}</strong>
                                </div>
                                <div class="stat-item">
                                    <span>"Success Rate:"</span>
                                    <strong>
                                        {format!("{}%",
                                            profile.get().successful_hacks * 100 / profile.get().total_hacks.max(1)
                                        )}
                                    </strong>
                                </div>
                            </div>

                            <div class="stat-card">
                                <h3>"Financial Stats"</h3>
                                <div class="stat-item">
                                    <span>"Money Earned:"</span>
                                    <strong>"$"{profile.get().money_earned}</strong>
                                </div>
                                <div class="stat-item">
                                    <span>"Viruses Created:"</span>
                                    <strong>{profile.get().viruses_created}</strong>
                                </div>
                            </div>

                            <div class="stat-card">
                                <h3>"Account Info"</h3>
                                <div class="stat-item">
                                    <span>"Member Since:"</span>
                                    <strong>{profile.get().created_at}</strong>
                                </div>
                                <div class="stat-item">
                                    <span>"Last Login:"</span>
                                    <strong>{profile.get().last_login}</strong>
                                </div>
                            </div>
                        </div>
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