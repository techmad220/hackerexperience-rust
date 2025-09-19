//! Skills Page - Skill tree and specialization

use leptos::*;
use leptos_router::*;

use crate::api::progression::{
    get_progression, invest_skill, reset_skills,
    PlayerProgression, SkillTree,
};

#[component]
pub fn SkillsPage() -> impl IntoView {
    let progression = create_resource(
        || (),
        |_| async move { get_progression().await },
    );

    let (selected_branch, set_selected_branch) = create_signal("hacking".to_string());

    view! {
        <div class="skills-page">
            <h1>"Skill Tree"</h1>

            <Suspense fallback=move || view! { <div>"Loading skills..."</div> }>
                {move || {
                    progression.get().map(|result| {
                        match result {
                            Ok(prog) => view! {
                                <div class="skills-container">
                                    <SkillPointsDisplay skill_tree=prog.skill_tree.clone()/>

                                    <div class="skill-branches">
                                        <BranchSelector
                                            selected=selected_branch
                                            set_selected=set_selected_branch
                                        />

                                        <SkillBranchView
                                            branch=selected_branch.get()
                                            skill_tree=prog.skill_tree.clone()
                                        />
                                    </div>

                                    <div class="skill-actions">
                                        <button
                                            class="btn btn-warning"
                                            on:click=move |_| {
                                                spawn_local(async move {
                                                    let _ = reset_skills().await;
                                                    progression.refetch();
                                                });
                                            }
                                        >
                                            "Reset All Skills"
                                        </button>
                                    </div>
                                </div>
                            }.into_view(),
                            Err(e) => view! {
                                <div class="error">"Failed to load skills: "{e.to_string()}</div>
                            }.into_view(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn SkillPointsDisplay(skill_tree: SkillTree) -> impl IntoView {
    view! {
        <div class="skill-points-display">
            <div class="points-available">
                <span class="label">"Available Points:"</span>
                <span class="value">{skill_tree.skill_points_available}</span>
            </div>
            <div class="points-spent">
                <span class="label">"Total Invested:"</span>
                <span class="value">{skill_tree.skill_points_spent}</span>
            </div>
        </div>
    }
}

#[component]
fn BranchSelector(
    selected: ReadSignal<String>,
    set_selected: WriteSignal<String>,
) -> impl IntoView {
    let branches = vec![
        ("hacking", "Hacking", "🔓"),
        ("defense", "Defense", "🛡️"),
        ("stealth", "Stealth", "🥷"),
        ("hardware", "Hardware", "💻"),
        ("software", "Software", "📀"),
        ("networking", "Networking", "🌐"),
    ];

    view! {
        <div class="branch-selector">
            {branches
                .into_iter()
                .map(|(id, name, icon)| {
                    let id_clone = id.to_string();
                    view! {
                        <button
                            class=move || {
                                if selected.get() == id {
                                    "branch-btn active"
                                } else {
                                    "branch-btn"
                                }
                            }
                            on:click=move |_| set_selected.set(id_clone.clone())
                        >
                            <span class="icon">{icon}</span>
                            <span class="name">{name}</span>
                        </button>
                    }
                })
                .collect::<Vec<_>>()
            }
        </div>
    }
}

#[component]
fn SkillBranchView(
    branch: String,
    skill_tree: SkillTree,
) -> impl IntoView {
    let skills = get_branch_skills(&branch);

    view! {
        <div class="skill-branch-view">
            <h2>{format_branch_name(&branch)}</h2>

            <div class="skills-grid">
                {skills
                    .into_iter()
                    .map(|skill| {
                        let is_unlocked = skill_tree.unlocked_skills.contains(&skill.id);
                        let skill_id = skill.id.clone();

                        view! {
                            <div class=move || {
                                if is_unlocked {
                                    "skill-node unlocked"
                                } else {
                                    "skill-node locked"
                                }
                            }>
                                <div class="skill-icon">{&skill.icon}</div>
                                <div class="skill-name">{&skill.name}</div>
                                <div class="skill-description">{&skill.description}</div>
                                <div class="skill-level">
                                    "Level: 0 / "{skill.max_level}
                                </div>

                                <button
                                    class="btn btn-sm btn-primary"
                                    disabled=move || skill_tree.skill_points_available == 0
                                    on:click=move |_| {
                                        let sid = skill_id.clone();
                                        spawn_local(async move {
                                            let _ = invest_skill(sid, 1).await;
                                        });
                                    }
                                >
                                    "Invest Point"
                                </button>
                            </div>
                        }
                    })
                    .collect::<Vec<_>>()
                }
            </div>
        </div>
    }
}

#[derive(Clone)]
struct SkillInfo {
    id: String,
    name: String,
    description: String,
    icon: String,
    max_level: u32,
}

fn get_branch_skills(branch: &str) -> Vec<SkillInfo> {
    match branch {
        "hacking" => vec![
            SkillInfo {
                id: "password_cracking".to_string(),
                name: "Password Cracking".to_string(),
                description: "Increases password cracking speed by 5% per level".to_string(),
                icon: "🔑".to_string(),
                max_level: 10,
            },
            SkillInfo {
                id: "exploit_development".to_string(),
                name: "Exploit Development".to_string(),
                description: "Create custom exploits for specific targets".to_string(),
                icon: "💣".to_string(),
                max_level: 5,
            },
            SkillInfo {
                id: "sql_injection".to_string(),
                name: "SQL Injection".to_string(),
                description: "Master database exploitation techniques".to_string(),
                icon: "💉".to_string(),
                max_level: 5,
            },
        ],
        "defense" => vec![
            SkillInfo {
                id: "firewall_mastery".to_string(),
                name: "Firewall Mastery".to_string(),
                description: "Improves firewall effectiveness by 10% per level".to_string(),
                icon: "🧱".to_string(),
                max_level: 10,
            },
            SkillInfo {
                id: "intrusion_detection".to_string(),
                name: "Intrusion Detection".to_string(),
                description: "Detect hacking attempts faster".to_string(),
                icon: "🚨".to_string(),
                max_level: 5,
            },
        ],
        "stealth" => vec![
            SkillInfo {
                id: "log_deletion".to_string(),
                name: "Log Deletion".to_string(),
                description: "Remove traces of your activities".to_string(),
                icon: "🗑️".to_string(),
                max_level: 5,
            },
            SkillInfo {
                id: "proxy_chains".to_string(),
                name: "Proxy Chains".to_string(),
                description: "Route through multiple proxies".to_string(),
                icon: "🔗".to_string(),
                max_level: 5,
            },
        ],
        "hardware" => vec![
            SkillInfo {
                id: "cpu_overclocking".to_string(),
                name: "CPU Overclocking".to_string(),
                description: "Increase processing power by 5% per level".to_string(),
                icon: "⚡".to_string(),
                max_level: 10,
            },
            SkillInfo {
                id: "ram_optimization".to_string(),
                name: "RAM Optimization".to_string(),
                description: "Use memory more efficiently".to_string(),
                icon: "💾".to_string(),
                max_level: 5,
            },
        ],
        "software" => vec![
            SkillInfo {
                id: "virus_development".to_string(),
                name: "Virus Development".to_string(),
                description: "Create more powerful viruses".to_string(),
                icon: "🦠".to_string(),
                max_level: 10,
            },
            SkillInfo {
                id: "ai_assistants".to_string(),
                name: "AI Assistants".to_string(),
                description: "Automate hacking tasks with AI".to_string(),
                icon: "🤖".to_string(),
                max_level: 5,
            },
        ],
        "networking" => vec![
            SkillInfo {
                id: "packet_sniffing".to_string(),
                name: "Packet Sniffing".to_string(),
                description: "Intercept network traffic".to_string(),
                icon: "📡".to_string(),
                max_level: 5,
            },
            SkillInfo {
                id: "ddos_amplification".to_string(),
                name: "DDoS Amplification".to_string(),
                description: "Increase DDoS attack power".to_string(),
                icon: "🌊".to_string(),
                max_level: 5,
            },
        ],
        _ => vec![],
    }
}

fn format_branch_name(branch: &str) -> String {
    match branch {
        "hacking" => "Hacking Skills",
        "defense" => "Defense Skills",
        "stealth" => "Stealth Skills",
        "hardware" => "Hardware Skills",
        "software" => "Software Skills",
        "networking" => "Networking Skills",
        _ => "Unknown Branch",
    }.to_string()
}