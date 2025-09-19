//! Leaderboard Page - Global rankings and competition

use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerRank {
    pub rank: u32,
    pub username: String,
    pub level: u32,
    pub reputation: i32,
    pub clan: Option<String>,
    pub hacks: u32,
    pub money: u64,
    pub status: String,
}

#[component]
pub fn LeaderboardPage() -> impl IntoView {
    let (leaderboard, set_leaderboard) = create_signal(vec![
        PlayerRank {
            rank: 1,
            username: "ZeroCool".to_string(),
            level: 99,
            reputation: 9999,
            clan: Some("Elite Hackers".to_string()),
            hacks: 5234,
            money: 999999999,
            status: "online".to_string(),
        },
        PlayerRank {
            rank: 2,
            username: "AcidBurn".to_string(),
            level: 95,
            reputation: 8532,
            clan: Some("Elite Hackers".to_string()),
            hacks: 4892,
            money: 850000000,
            status: "online".to_string(),
        },
        PlayerRank {
            rank: 3,
            username: "CrashOverride".to_string(),
            level: 92,
            reputation: 7821,
            clan: Some("Digital Knights".to_string()),
            hacks: 4521,
            money: 720000000,
            status: "offline".to_string(),
        },
        PlayerRank {
            rank: 42,
            username: "Neo".to_string(),
            level: 42,
            reputation: 1337,
            clan: Some("The Architects".to_string()),
            hacks: 523,
            money: 12500000,
            status: "online".to_string(),
        },
    ]);

    let (ranking_type, set_ranking_type) = create_signal("reputation");
    let (time_period, set_time_period) = create_signal("all-time");

    view! {
        <div class="leaderboard-page">
            <div class="leaderboard-header">
                <h1>"Global Leaderboard"</h1>
                <div class="your-rank">
                    <span>"Your Rank: #42"</span>
                </div>
            </div>

            <div class="leaderboard-controls">
                <div class="ranking-types">
                    <button
                        class=move || if ranking_type.get() == "reputation" { "btn active" } else { "btn" }
                        on:click=move |_| set_ranking_type("reputation")
                    >"Reputation"</button>
                    <button
                        class=move || if ranking_type.get() == "level" { "btn active" } else { "btn" }
                        on:click=move |_| set_ranking_type("level")
                    >"Level"</button>
                    <button
                        class=move || if ranking_type.get() == "money" { "btn active" } else { "btn" }
                        on:click=move |_| set_ranking_type("money")
                    >"Wealth"</button>
                    <button
                        class=move || if ranking_type.get() == "hacks" { "btn active" } else { "btn" }
                        on:click=move |_| set_ranking_type("hacks")
                    >"Total Hacks"</button>
                </div>

                <div class="time-filters">
                    <select
                        class="time-select"
                        on:change=move |ev| set_time_period(event_target_value(&ev))
                    >
                        <option value="all-time">"All Time"</option>
                        <option value="monthly">"This Month"</option>
                        <option value="weekly">"This Week"</option>
                        <option value="daily">"Today"</option>
                    </select>
                </div>
            </div>

            <div class="leaderboard-table">
                <table>
                    <thead>
                        <tr>
                            <th>"Rank"</th>
                            <th>"Player"</th>
                            <th>"Level"</th>
                            <th>"Reputation"</th>
                            <th>"Clan"</th>
                            <th>"Total Hacks"</th>
                            <th>"Wealth"</th>
                            <th>"Status"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || leaderboard.get().into_iter().map(|player| {
                            let is_current_user = player.username == "Neo";
                            view! {
                                <tr class=if is_current_user { "current-user" } else { "" }>
                                    <td class="rank">
                                        {match player.rank {
                                            1 => "🥇",
                                            2 => "🥈",
                                            3 => "🥉",
                                            _ => ""
                                        }}
                                        "#"{player.rank}
                                    </td>
                                    <td class="username">{&player.username}</td>
                                    <td class="level">{player.level}</td>
                                    <td class="reputation">{player.reputation}</td>
                                    <td class="clan">
                                        {player.clan.unwrap_or("-".to_string())}
                                    </td>
                                    <td class="hacks">{player.hacks}</td>
                                    <td class="money">"$"{player.money}</td>
                                    <td class="status">
                                        <span class=format!("status-{}", player.status)>
                                            {if player.status == "online" { "●" } else { "○" }}
                                        </span>
                                    </td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}