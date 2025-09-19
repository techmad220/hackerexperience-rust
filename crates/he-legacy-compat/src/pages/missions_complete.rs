//! Complete missions handler - Connects to game mechanics
//!
//! 100% Rust implementation connecting the legacy route to game mechanics

use axum::{
    extract::{Extension, Query, Form},
    http::StatusCode,
    response::{Html, Redirect, IntoResponse},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use he_game_mechanics::missions::{
    Mission, MissionType, MissionStatus, MissionDifficulty,
    MissionRequirements, MissionRewards, MissionManager
};

#[derive(Debug, Deserialize)]
pub struct MissionQuery {
    pub id: Option<i64>,
    pub action: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MissionAction {
    pub mission_id: i64,
    pub action: String,
}

/// Main missions handler - connects to game mechanics
pub async fn missions_handler(
    Extension(pool): Extension<PgPool>,
    Extension(session): Extension<crate::session::PhpSession>,
) -> impl IntoResponse {
    if !session.isset_login() {
        return Redirect::to("/index.php").into_response();
    }

    let user_id = session.user_id();

    // Initialize the mission manager from game mechanics
    let mission_manager = MissionManager::new(pool.clone());

    // Get player state
    let player_state = get_player_state(&pool, user_id).await;

    // Get available missions
    let available_missions = mission_manager.get_available_missions(&player_state).await;

    // Get active missions
    let active_missions = mission_manager.get_active_missions(user_id).await;

    // Get completed missions
    let completed_missions = mission_manager.get_completed_missions(user_id).await;

    // Render the page
    render_missions_page(available_missions, active_missions, completed_missions).into_response()
}

/// Handle mission actions
pub async fn missions_action_handler(
    Extension(pool): Extension<PgPool>,
    Extension(session): Extension<crate::session::PhpSession>,
    Form(action): Form<MissionAction>,
) -> impl IntoResponse {
    if !session.isset_login() {
        return Redirect::to("/index.php").into_response();
    }

    let user_id = session.user_id();
    let mission_manager = MissionManager::new(pool.clone());
    let player_state = get_player_state(&pool, user_id).await;

    match action.action.as_str() {
        "accept" => {
            match mission_manager.accept_mission(user_id, action.mission_id, &player_state).await {
                Ok(_) => {
                    session.add_message("Mission accepted!");
                }
                Err(e) => {
                    session.add_error(&format!("Failed to accept mission: {}", e));
                }
            }
        }
        "complete" => {
            match mission_manager.complete_mission(user_id, action.mission_id).await {
                Ok(rewards) => {
                    apply_rewards(&pool, user_id, &rewards).await;
                    session.add_message(&format!("Mission completed! Rewards: ${}, {} XP",
                        rewards.money, rewards.experience));
                }
                Err(e) => {
                    session.add_error(&format!("Failed to complete mission: {}", e));
                }
            }
        }
        "abort" => {
            match mission_manager.abort_mission(user_id, action.mission_id).await {
                Ok(_) => {
                    session.add_message("Mission aborted");
                }
                Err(_) => {
                    session.add_error("Failed to abort mission");
                }
            }
        }
        _ => {}
    }

    Redirect::to("/missions.php").into_response()
}

async fn get_player_state(pool: &PgPool, user_id: i64) -> he_game_mechanics::PlayerState {
    // Fetch player data from database
    let player_data = sqlx::query!(
        r#"
        SELECT
            u.username, u.money, u.reputation, u.experience,
            h.cpu_mhz, h.ram_mb, h.hdd_gb, h.net_mbps,
            COUNT(DISTINCT s.id) as software_count,
            c.clan_id
        FROM users u
        LEFT JOIN hardware h ON u.id = h.user_id
        LEFT JOIN software s ON u.id = s.user_id
        LEFT JOIN clan_members c ON u.id = c.user_id
        WHERE u.id = $1
        GROUP BY u.id, h.id, c.clan_id
        "#,
        user_id
    )
    .fetch_one(pool)
    .await
    .unwrap();

    he_game_mechanics::PlayerState {
        player_id: user_id as u64,
        username: player_data.username,
        money: player_data.money as u64,
        reputation: player_data.reputation as i64,
        experience: player_data.experience as u64,
        cpu_power: player_data.cpu_mhz.unwrap_or(1000) as u32,
        ram_amount: player_data.ram_mb.unwrap_or(512) as u32,
        disk_space: player_data.hdd_gb.unwrap_or(10) as u32,
        network_speed: player_data.net_mbps.unwrap_or(10) as u32,
        software_count: player_data.software_count.unwrap_or(0) as usize,
        clan_id: player_data.clan_id.map(|id| id as u64),
        active_processes: vec![],
        completed_missions: vec![],
    }
}

async fn apply_rewards(pool: &PgPool, user_id: i64, rewards: &MissionRewards) {
    // Apply money reward
    sqlx::query!(
        "UPDATE users SET money = money + $1, experience = experience + $2, reputation = reputation + $3
         WHERE id = $4",
        rewards.money as i64,
        rewards.experience as i64,
        rewards.reputation as i64,
        user_id
    )
    .execute(pool)
    .await
    .ok();

    // Apply any software rewards
    for software in &rewards.software {
        sqlx::query!(
            "INSERT INTO software (user_id, name, type, version, size) VALUES ($1, $2, $3, $4, $5)",
            user_id,
            software.name,
            software.software_type,
            software.version,
            software.size as i64
        )
        .execute(pool)
        .await
        .ok();
    }
}

fn render_missions_page(
    available: Vec<Mission>,
    active: Vec<Mission>,
    completed: Vec<Mission>,
) -> Html<String> {
    let mut html = String::from(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Missions - HackerExperience</title>
    <link rel="stylesheet" href="/css/he-matrix.css">
    <style>
        body { background: #0a0a0a; color: #00ff00; font-family: monospace; }
        .container { max-width: 1200px; margin: 0 auto; padding: 20px; }
        .mission-section { margin-bottom: 30px; }
        .mission-card {
            background: #1a1a1a;
            border: 1px solid #00ff00;
            padding: 15px;
            margin: 10px 0;
        }
        .mission-title { font-size: 18px; font-weight: bold; color: #00ff00; }
        .mission-description { color: #c0c0c0; margin: 10px 0; }
        .mission-rewards { color: #ffff00; }
        .mission-difficulty {
            display: inline-block;
            padding: 2px 8px;
            margin-left: 10px;
            border: 1px solid;
        }
        .difficulty-easy { color: #00ff00; border-color: #00ff00; }
        .difficulty-medium { color: #ffff00; border-color: #ffff00; }
        .difficulty-hard { color: #ff8800; border-color: #ff8800; }
        .difficulty-extreme { color: #ff0000; border-color: #ff0000; }
        .btn {
            padding: 8px 15px;
            margin: 5px;
            background: #1a1a1a;
            color: #00ff00;
            border: 1px solid #00ff00;
            cursor: pointer;
            text-decoration: none;
            display: inline-block;
        }
        .btn:hover { background: #00ff00; color: #000; }
        .tabs {
            margin-bottom: 20px;
            border-bottom: 1px solid #00ff00;
        }
        .tab {
            display: inline-block;
            padding: 10px 20px;
            color: #888;
            text-decoration: none;
            border: 1px solid transparent;
        }
        .tab.active {
            color: #00ff00;
            border-color: #00ff00;
            border-bottom-color: #0a0a0a;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Mission Control</h1>

        <div class="tabs">
            <a href="#available" class="tab active">Available ({available_count})</a>
            <a href="#active" class="tab">Active ({active_count})</a>
            <a href="#completed" class="tab">Completed ({completed_count})</a>
        </div>
    "#);

    html = html.replace("{available_count}", &available.len().to_string());
    html = html.replace("{active_count}", &active.len().to_string());
    html = html.replace("{completed_count}", &completed.len().to_string());

    // Available Missions
    html.push_str(r#"<div class="mission-section" id="available"><h2>Available Missions</h2>"#);

    for mission in available {
        let difficulty_class = match mission.difficulty {
            MissionDifficulty::Easy => "difficulty-easy",
            MissionDifficulty::Medium => "difficulty-medium",
            MissionDifficulty::Hard => "difficulty-hard",
            MissionDifficulty::Extreme => "difficulty-extreme",
        };

        html.push_str(&format!(r#"
            <div class="mission-card">
                <div class="mission-title">
                    {}
                    <span class="mission-difficulty {}">{:?}</span>
                </div>
                <div class="mission-description">{}</div>
                <div class="mission-rewards">
                    Rewards: ${} | {} XP | {} Rep
                </div>
                <form method="post" style="display: inline;">
                    <input type="hidden" name="mission_id" value="{}">
                    <input type="hidden" name="action" value="accept">
                    <button type="submit" class="btn">Accept Mission</button>
                </form>
            </div>
        "#,
            mission.name,
            difficulty_class,
            mission.difficulty,
            mission.description,
            mission.rewards.money,
            mission.rewards.experience,
            mission.rewards.reputation,
            mission.id
        ));
    }

    if available.is_empty() {
        html.push_str(r#"<p style="color: #666;">No missions available. Check back later!</p>"#);
    }

    html.push_str("</div>");

    // Active Missions
    html.push_str(r#"<div class="mission-section" id="active"><h2>Active Missions</h2>"#);

    for mission in active {
        html.push_str(&format!(r#"
            <div class="mission-card">
                <div class="mission-title">{}</div>
                <div class="mission-description">{}</div>
                <div class="mission-description">Progress: {}%</div>
                <form method="post" style="display: inline;">
                    <input type="hidden" name="mission_id" value="{}">
                    <input type="hidden" name="action" value="complete">
                    <button type="submit" class="btn">Complete</button>
                </form>
                <form method="post" style="display: inline;">
                    <input type="hidden" name="mission_id" value="{}">
                    <input type="hidden" name="action" value="abort">
                    <button type="submit" class="btn">Abort</button>
                </form>
            </div>
        "#,
            mission.name,
            mission.description,
            mission.progress,
            mission.id,
            mission.id
        ));
    }

    if active.is_empty() {
        html.push_str(r#"<p style="color: #666;">No active missions.</p>"#);
    }

    html.push_str("</div>");

    // Completed Missions
    html.push_str(r#"<div class="mission-section" id="completed"><h2>Completed Missions</h2>"#);

    for mission in completed.iter().take(10) {
        html.push_str(&format!(r#"
            <div class="mission-card">
                <div class="mission-title">{} ✓</div>
                <div class="mission-description">{}</div>
            </div>
        "#,
            mission.name,
            mission.description
        ));
    }

    if completed.is_empty() {
        html.push_str(r#"<p style="color: #666;">No completed missions yet.</p>"#);
    }

    html.push_str("</div>");

    html.push_str(r#"
    </div>
</body>
</html>
    "#);

    Html(html)
}