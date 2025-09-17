//! Achievements Page - View and track achievements

use leptos::*;
use leptos_router::*;

use crate::api::progression::{
    get_achievements, get_progression,
    AchievementProgress, PlayerProgression,
};

#[component]
pub fn AchievementsPage() -> impl IntoView {
    let achievements = create_resource(
        || (),
        |_| async move { get_achievements().await },
    );

    let progression = create_resource(
        || (),
        |_| async move { get_progression().await },
    );

    let (filter, set_filter) = create_signal("all".to_string());

    view! {
        <div class="achievements-page">
            <h1>"Achievements"</h1>

            <Suspense fallback=move || view! { <div>"Loading achievements..."</div> }>
                {move || {
                    achievements.get().map(|result| {
                        match result {
                            Ok(ach) => view! {
                                <div class="achievements-container">
                                    <AchievementStats achievement_progress=ach.clone()/>

                                    <div class="achievement-filters">
                                        <button
                                            class=move || if filter.get() == "all" { "filter-btn active" } else { "filter-btn" }
                                            on:click=move |_| set_filter.set("all".to_string())
                                        >
                                            "All"
                                        </button>
                                        <button
                                            class=move || if filter.get() == "unlocked" { "filter-btn active" } else { "filter-btn" }
                                            on:click=move |_| set_filter.set("unlocked".to_string())
                                        >
                                            "Unlocked"
                                        </button>
                                        <button
                                            class=move || if filter.get() == "locked" { "filter-btn active" } else { "filter-btn" }
                                            on:click=move |_| set_filter.set("locked".to_string())
                                        >
                                            "Locked"
                                        </button>
                                    </div>

                                    <AchievementCategories
                                        achievement_progress=ach.clone()
                                        filter=filter.get()
                                    />
                                </div>
                            }.into_view(),
                            Err(e) => view! {
                                <div class="error">"Failed to load achievements: "{e.to_string()}</div>
                            }.into_view(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn AchievementStats(achievement_progress: AchievementProgress) -> impl IntoView {
    let total_achievements = 50; // Would come from backend
    let unlocked = achievement_progress.unlocked_achievements.len();
    let percentage = (unlocked as f32 / total_achievements as f32 * 100.0) as u32;

    view! {
        <div class="achievement-stats">
            <div class="stat">
                <span class="label">"Total Points:"</span>
                <span class="value">{achievement_progress.achievement_points}</span>
            </div>
            <div class="stat">
                <span class="label">"Unlocked:"</span>
                <span class="value">{unlocked}" / "{total_achievements}</span>
            </div>
            <div class="stat">
                <span class="label">"Completion:"</span>
                <span class="value">{percentage}"%"</span>
            </div>
            <div class="completion-bar">
                <div class="completion-fill" style:width=move || format!("{}%", percentage)></div>
            </div>
        </div>
    }
}

#[component]
fn AchievementCategories(
    achievement_progress: AchievementProgress,
    filter: String,
) -> impl IntoView {
    let categories = vec![
        ("hacking", "Hacking", get_hacking_achievements()),
        ("progression", "Progression", get_progression_achievements()),
        ("social", "Social", get_social_achievements()),
        ("special", "Special", get_special_achievements()),
    ];

    view! {
        <div class="achievement-categories">
            {categories
                .into_iter()
                .map(|(id, name, achievements)| {
                    view! {
                        <div class="category-section">
                            <h2>{name}</h2>
                            <div class="achievements-grid">
                                {achievements
                                    .into_iter()
                                    .filter(|ach| {
                                        match filter.as_str() {
                                            "unlocked" => achievement_progress.unlocked_achievements.contains(&ach.id),
                                            "locked" => !achievement_progress.unlocked_achievements.contains(&ach.id),
                                            _ => true,
                                        }
                                    })
                                    .map(|ach| {
                                        let is_unlocked = achievement_progress.unlocked_achievements.contains(&ach.id);
                                        view! {
                                            <AchievementCard
                                                achievement=ach
                                                is_unlocked=is_unlocked
                                            />
                                        }
                                    })
                                    .collect::<Vec<_>>()
                                }
                            </div>
                        </div>
                    }
                })
                .collect::<Vec<_>>()
            }
        </div>
    }
}

#[component]
fn AchievementCard(
    achievement: AchievementInfo,
    is_unlocked: bool,
) -> impl IntoView {
    view! {
        <div class=move || {
            if is_unlocked {
                format!("achievement-card unlocked {}", achievement.rarity.to_lowercase())
            } else {
                "achievement-card locked"
            }
        }>
            <div class="achievement-icon">{achievement.icon}</div>
            <div class="achievement-info">
                <h3>{achievement.name}</h3>
                <p class="description">{achievement.description}</p>
                <div class="achievement-meta">
                    <span class="points">{achievement.points}" points"</span>
                    <span class="rarity">{achievement.rarity}</span>
                </div>
                {if !achievement.hidden || is_unlocked {
                    view! {
                        <div class="rewards">
                            "Rewards: "{achievement.rewards}
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <div class="hidden-text">"???"</div>
                    }.into_view()
                }}
            </div>
        </div>
    }
}

#[derive(Clone)]
struct AchievementInfo {
    id: String,
    name: String,
    description: String,
    icon: String,
    points: u32,
    rarity: String,
    rewards: String,
    hidden: bool,
}

fn get_hacking_achievements() -> Vec<AchievementInfo> {
    vec![
        AchievementInfo {
            id: "first_hack".to_string(),
            name: "First Blood".to_string(),
            description: "Successfully hack your first server".to_string(),
            icon: "🔓".to_string(),
            points: 10,
            rarity: "Common".to_string(),
            rewards: "100 XP, $1000".to_string(),
            hidden: false,
        },
        AchievementInfo {
            id: "hack_100".to_string(),
            name: "Century Mark".to_string(),
            description: "Hack 100 servers".to_string(),
            icon: "💯".to_string(),
            points: 50,
            rarity: "Uncommon".to_string(),
            rewards: "5000 XP, Elite Scanner".to_string(),
            hidden: false,
        },
        AchievementInfo {
            id: "hack_elite".to_string(),
            name: "Elite Access".to_string(),
            description: "Successfully hack an elite tier server".to_string(),
            icon: "🎯".to_string(),
            points: 100,
            rarity: "Epic".to_string(),
            rewards: "10000 XP, Quantum Decryptor".to_string(),
            hidden: false,
        },
    ]
}

fn get_progression_achievements() -> Vec<AchievementInfo> {
    vec![
        AchievementInfo {
            id: "level_10".to_string(),
            name: "Double Digits".to_string(),
            description: "Reach level 10".to_string(),
            icon: "🔟".to_string(),
            points: 20,
            rarity: "Common".to_string(),
            rewards: "1000 XP, Title: Experienced".to_string(),
            hidden: false,
        },
        AchievementInfo {
            id: "level_50".to_string(),
            name: "Halfway to Legend".to_string(),
            description: "Reach level 50".to_string(),
            icon: "⭐".to_string(),
            points: 75,
            rarity: "Rare".to_string(),
            rewards: "10000 XP, Skill Reset Token".to_string(),
            hidden: false,
        },
    ]
}

fn get_social_achievements() -> Vec<AchievementInfo> {
    vec![
        AchievementInfo {
            id: "pvp_first_win".to_string(),
            name: "First Victory".to_string(),
            description: "Win your first PvP hack".to_string(),
            icon: "🏆".to_string(),
            points: 15,
            rarity: "Common".to_string(),
            rewards: "500 XP, Title: Competitor".to_string(),
            hidden: false,
        },
        AchievementInfo {
            id: "pvp_champion".to_string(),
            name: "Undefeated".to_string(),
            description: "Win 100 PvP hacks".to_string(),
            icon: "🥇".to_string(),
            points: 100,
            rarity: "Epic".to_string(),
            rewards: "15000 XP, PvP Shield".to_string(),
            hidden: false,
        },
    ]
}

fn get_special_achievements() -> Vec<AchievementInfo> {
    vec![
        AchievementInfo {
            id: "mystery_solver".to_string(),
            name: "The Truth".to_string(),
            description: "???".to_string(),
            icon: "🎭".to_string(),
            points: 200,
            rarity: "Legendary".to_string(),
            rewards: "???".to_string(),
            hidden: true,
        },
        AchievementInfo {
            id: "millionaire".to_string(),
            name: "Digital Millionaire".to_string(),
            description: "Earn 1,000,000 credits total".to_string(),
            icon: "💰".to_string(),
            points: 75,
            rarity: "Rare".to_string(),
            rewards: "10000 XP, Golden USB".to_string(),
            hidden: false,
        },
    ]
}