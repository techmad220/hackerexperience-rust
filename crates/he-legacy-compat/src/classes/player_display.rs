//! Player display methods - comprehensive port of display functionality from Player.class.php
//! 
//! This module handles all the HTML rendering and display logic from the original PHP Player class
//! Including:
//! - Index page rendering (showIndex)
//! - Game over page (showGameOver) 
//! - Control panel displays (controlpanel_show)
//! - Forum displays (forum_show)
//! - Settings displays (settings_show)

use crate::classes::player::{Player, PlayerError, HardwareInfo, PlayerRanking, MissionInfo, ClanInfo};
use he_db::DbPool;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use handlebars::Handlebars;
use chrono::{DateTime, Utc};

/// Display-related data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexData {
    pub ip: String,
    pub password: String,
    pub uptime: String,
    pub change_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlPanelData {
    pub hardware: HardwareInfo,
    pub user_info: UserDisplayInfo,
    pub top_players: Vec<PlayerRanking>,
    pub mission: Option<MissionInfo>,
    pub clan: Option<ClanInfo>,
    pub connected_to: String,
    pub running_tasks: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDisplayInfo {
    pub reputation: i64,
    pub rank: i64,
    pub membership: String,
    pub membership_class: String,
    pub is_master: bool,
    pub in_war: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameOverData {
    pub my_info: Option<HistoricalUserInfo>,
    pub round_info: RoundInfo,
    pub top_users: Vec<HistoricalUserInfo>,
    pub top_clans: Vec<HistoricalClanInfo>,
    pub next_round_start: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalUserInfo {
    pub rank: i64,
    pub user: String,
    pub reputation: i64,
    pub clan_name: String,
    pub best_software: String,
    pub best_software_version: String,
    pub hack_count: i64,
    pub ddos_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalClanInfo {
    pub rank: i64,
    pub name: String,
    pub power: i64,
    pub wins: i64,
    pub losses: i64,
    pub members: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundInfo {
    pub total_dooms: i64,
    pub doomed_by: String,
    pub total_researched: i64,
    pub total_hacked: i64,
    pub total_ddos: i64,
    pub round_number: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForumPost {
    pub id: i64,
    pub title: String,
    pub author: String,
    pub author_id: i64,
    pub date: DateTime<Utc>,
    pub content: String,
    pub comment_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub date: DateTime<Utc>,
    pub author: String,
}

impl Player {
    /// Show main index page
    /// Port of showIndex() method
    pub async fn show_index(&self) -> Result<String, PlayerError> {
        let uid = self.session_id.ok_or_else(|| PlayerError::ValidationError("No active session.".to_string()))?;
        
        // Get player's game data
        let result = sqlx::query!(
            "SELECT gameIP, gamePass FROM users WHERE id = ? LIMIT 1",
            uid
        )
        .fetch_one(&self.db_pool)
        .await?;

        let index_data = IndexData {
            ip: self.long_to_ip(result.gameIP.unwrap_or(0)),
            password: result.gamePass,
            uptime: self.ip_uptime(Some(uid)).await?,
            change_text: "change".to_string(), // Would be localized in real implementation
        };

        // Render the index template with control panel data
        let control_panel_data = self.get_control_panel_data().await?;
        self.render_index_template(index_data, control_panel_data).await
    }

    /// Show game over page
    /// Port of showGameOver() method
    pub async fn show_game_over(&self) -> Result<String, PlayerError> {
        let uid = self.session_id.ok_or_else(|| PlayerError::ValidationError("No active session.".to_string()))?;
        let current_round = self.get_current_round().await?;
        let previous_round = current_round - 1;

        // Get player's historical information
        let my_info = self.get_historical_user_info(uid, previous_round).await?;
        
        // Get round information
        let round_info = self.get_round_info(previous_round).await?;
        
        // Get top users and clans
        let top_users = self.get_top_historical_users(previous_round, 10).await?;
        let top_clans = self.get_top_historical_clans(previous_round, 10).await?;
        
        // Get next round start time
        let next_round_start = self.get_next_round_start_time().await?;

        let game_over_data = GameOverData {
            my_info,
            round_info,
            top_users,
            top_clans,
            next_round_start,
        };

        self.render_game_over_template(game_over_data).await
    }

    /// Get control panel data
    /// Port of controlpanel_show() functionality
    pub async fn get_control_panel_data(&self) -> Result<ControlPanelData, PlayerError> {
        let uid = self.session_id.ok_or_else(|| PlayerError::ValidationError("No active session.".to_string()))?;

        // Get hardware information
        let hardware = self.get_hardware_info(uid).await?;
        
        // Get user information
        let user_info = self.get_user_display_info(uid).await?;
        
        // Get top players
        let top_players = self.get_top_players(7).await?;
        
        // Get current mission
        let mission = self.get_current_mission(uid).await?;
        
        // Get clan info
        let clan = self.get_clan_info(uid).await?;
        
        // Get connection info
        let connected_to = self.get_connected_to(uid).await?;
        
        // Get running tasks count
        let running_tasks = self.get_running_tasks_count(uid).await?;

        Ok(ControlPanelData {
            hardware,
            user_info,
            top_players,
            mission,
            clan,
            connected_to,
            running_tasks,
        })
    }

    /// Show control panel section
    /// Port of controlpanel_show() method
    pub async fn controlpanel_show(&self, section: &str) -> Result<String, PlayerError> {
        match section {
            "hardware" => self.show_hardware_section().await,
            "userinfo" => self.show_userinfo_section().await,
            "top10" => self.show_top10_section().await,
            "fbi" => self.show_fbi_section().await,
            "news" => self.show_news_section().await,
            "round" => self.show_round_section().await,
            _ => Err(PlayerError::ValidationError(format!("Unknown section: {}", section))),
        }
    }

    /// Show hardware section
    async fn show_hardware_section(&self) -> Result<String, PlayerError> {
        let uid = self.session_id.ok_or_else(|| PlayerError::ValidationError("No active session.".to_string()))?;
        let hardware = self.get_hardware_info(uid).await?;
        
        // Format hardware values for display
        let cpu = format!("{:.1} GHz", hardware.cpu / 1000.0);
        let hdd = if hardware.hdd < 1000.0 {
            format!("{:.0} MB", hardware.hdd)
        } else {
            format!("{:.1} GB", hardware.hdd / 1000.0)
        };
        let ram = if hardware.ram < 1000.0 {
            format!("{:.0} MB", hardware.ram)
        } else {
            format!("{:.1} GB", hardware.ram / 1000.0)
        };
        let net = if hardware.net == 1000.0 {
            "1 Gbit/s".to_string()
        } else {
            format!("{:.1} Mbit/s", hardware.net)
        };
        let xhd = if hardware.xhd == 0.0 {
            "None".to_string()
        } else {
            format!("{:.1} GB", hardware.xhd / 1000.0)
        };

        // Render hardware template
        self.render_hardware_template(cpu, hdd, ram, net, xhd).await
    }

    /// Show user info section
    async fn show_userinfo_section(&self) -> Result<String, PlayerError> {
        let uid = self.session_id.ok_or_else(|| PlayerError::ValidationError("No active session.".to_string()))?;
        
        let user_info = self.get_user_display_info(uid).await?;
        let mission = self.get_current_mission(uid).await?;
        let clan = self.get_clan_info(uid).await?;
        let connected_to = self.get_connected_to(uid).await?;
        let running_tasks = self.get_running_tasks_count(uid).await?;

        self.render_userinfo_template(user_info, mission, clan, connected_to, running_tasks).await
    }

    /// Show top 10 section
    async fn show_top10_section(&self) -> Result<String, PlayerError> {
        let top_players = self.get_top_players(7).await?;
        self.render_top10_template(top_players).await
    }

    /// Show FBI section
    async fn show_fbi_section(&self) -> Result<String, PlayerError> {
        // This would integrate with the Storyline class
        Ok("<div>FBI section placeholder</div>".to_string())
    }

    /// Show news section
    async fn show_news_section(&self) -> Result<String, PlayerError> {
        let news = self.get_latest_news(3).await?;
        self.render_news_template(news).await
    }

    /// Show round section
    async fn show_round_section(&self) -> Result<String, PlayerError> {
        let current_round = self.get_current_round().await?;
        let round_info = self.get_round_display_info(current_round).await?;
        self.render_round_template(round_info).await
    }

    /// Show forum section
    /// Port of forum_show() method
    pub async fn forum_show(&self, page: &str) -> Result<String, PlayerError> {
        match page {
            "recent_comments" => self.show_recent_comments().await,
            "recent_posts" => self.show_recent_posts().await,
            "announcements" => self.show_announcements().await,
            _ => Err(PlayerError::ValidationError(format!("Unknown forum page: {}", page))),
        }
    }

    /// Show recent comments
    async fn show_recent_comments(&self) -> Result<String, PlayerError> {
        let comments = self.get_recent_forum_comments(10).await?;
        self.render_forum_comments_template(comments).await
    }

    /// Show recent posts
    async fn show_recent_posts(&self) -> Result<String, PlayerError> {
        let posts = self.get_recent_forum_posts(10).await?;
        self.render_forum_posts_template(posts).await
    }

    /// Show announcements
    async fn show_announcements(&self) -> Result<String, PlayerError> {
        let announcements = self.get_forum_announcements(5).await?;
        self.render_announcements_template(announcements).await
    }

    /// Show settings page
    /// Port of settings_show() method
    pub async fn settings_show(&self) -> Result<String, PlayerError> {
        self.render_settings_template().await
    }

    // Helper methods for data retrieval

    async fn get_hardware_info(&self, user_id: i64) -> Result<HardwareInfo, PlayerError> {
        // This would integrate with the PC/Hardware class
        // For now, return mock data
        Ok(HardwareInfo {
            cpu: 2400.0,
            hdd: 512000.0,
            ram: 8192.0,
            net: 100.0,
            xhd: 0.0,
        })
    }

    async fn get_user_display_info(&self, user_id: i64) -> Result<UserDisplayInfo, PlayerError> {
        let is_premium = self.is_premium(Some(user_id)).await?;
        let is_admin = self.is_admin(Some(user_id)).await?;
        
        let (membership, membership_class) = if is_premium {
            ("Premium".to_string(), "label-warning".to_string())
        } else if is_admin {
            ("Staff".to_string(), "label-important".to_string())
        } else {
            ("Basic".to_string(), "label-success".to_string())
        };

        // Get reputation and rank
        let reputation = self.get_player_reputation(user_id).await?;
        let rank = self.get_player_rank(user_id).await?;

        Ok(UserDisplayInfo {
            reputation,
            rank,
            membership,
            membership_class,
            is_master: false, // Would be determined by clan system
            in_war: false,   // Would be determined by clan system
        })
    }

    async fn get_current_mission(&self, user_id: i64) -> Result<Option<MissionInfo>, PlayerError> {
        // This would integrate with the Mission system
        // For now, return None
        Ok(None)
    }

    async fn get_clan_info(&self, user_id: i64) -> Result<Option<ClanInfo>, PlayerError> {
        // This would integrate with the Clan system
        // For now, return None
        Ok(None)
    }

    async fn get_connected_to(&self, user_id: i64) -> Result<String, PlayerError> {
        // Check if user is connected to any IP
        // For now, return "No one"
        Ok("No one".to_string())
    }

    async fn get_running_tasks_count(&self, user_id: i64) -> Result<i64, PlayerError> {
        let result = sqlx::query!(
            "SELECT COUNT(*) AS count FROM processes WHERE userID = ? AND status = 'RUNNING'",
            user_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(result.count)
    }

    async fn get_player_reputation(&self, user_id: i64) -> Result<i64, PlayerError> {
        let result = sqlx::query!(
            "SELECT reputation FROM cache WHERE userID = ? LIMIT 1",
            user_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(result.map(|r| r.reputation).unwrap_or(0))
    }

    async fn get_player_rank(&self, user_id: i64) -> Result<i64, PlayerError> {
        let result = sqlx::query!(
            "SELECT rank FROM ranking_user WHERE userID = ? LIMIT 1",
            user_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(result.map(|r| r.rank).unwrap_or(0))
    }

    async fn get_current_round(&self) -> Result<i32, PlayerError> {
        // This would integrate with the Storyline system
        // For now, return 1
        Ok(1)
    }

    async fn get_historical_user_info(&self, user_id: i64, round: i32) -> Result<Option<HistoricalUserInfo>, PlayerError> {
        let result = sqlx::query!(
            "SELECT rank, user, reputation, clanName, bestSoft, bestSoftVersion, hackCount, ddosCount 
             FROM hist_users WHERE userID = ? AND round = ? LIMIT 1",
            user_id,
            round
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(row) = result {
            Ok(Some(HistoricalUserInfo {
                rank: row.rank,
                user: row.user,
                reputation: row.reputation,
                clan_name: row.clanName.unwrap_or_default(),
                best_software: row.bestSoft.unwrap_or_default(),
                best_software_version: row.bestSoftVersion.unwrap_or_default(),
                hack_count: row.hackCount,
                ddos_count: row.ddosCount,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_round_info(&self, round: i32) -> Result<RoundInfo, PlayerError> {
        // This would integrate with the Storyline system to get round statistics
        Ok(RoundInfo {
            total_dooms: 0,
            doomed_by: "Unknown".to_string(),
            total_researched: 0,
            total_hacked: 0,
            total_ddos: 0,
            round_number: round,
        })
    }

    async fn get_top_historical_users(&self, round: i32, limit: i64) -> Result<Vec<HistoricalUserInfo>, PlayerError> {
        // Would fetch from hist_users table
        Ok(Vec::new())
    }

    async fn get_top_historical_clans(&self, round: i32, limit: i64) -> Result<Vec<HistoricalClanInfo>, PlayerError> {
        // Would fetch from hist_clans table
        Ok(Vec::new())
    }

    async fn get_next_round_start_time(&self) -> Result<String, PlayerError> {
        // This would calculate based on storyline system
        Ok("Soon™".to_string())
    }

    async fn get_latest_news(&self, limit: i64) -> Result<Vec<NewsItem>, PlayerError> {
        // This would integrate with the News system
        Ok(Vec::new())
    }

    async fn get_round_display_info(&self, round: i32) -> Result<String, PlayerError> {
        // This would integrate with the Storyline system
        Ok(format!("Round {} information", round))
    }

    async fn get_recent_forum_comments(&self, limit: i64) -> Result<Vec<ForumPost>, PlayerError> {
        // This would integrate with the Forum system
        Ok(Vec::new())
    }

    async fn get_recent_forum_posts(&self, limit: i64) -> Result<Vec<ForumPost>, PlayerError> {
        // This would integrate with the Forum system
        Ok(Vec::new())
    }

    async fn get_forum_announcements(&self, limit: i64) -> Result<Vec<ForumPost>, PlayerError> {
        // This would integrate with the Forum system
        Ok(Vec::new())
    }

    // Template rendering methods (would use Handlebars or similar)

    async fn render_index_template(&self, index_data: IndexData, control_panel_data: ControlPanelData) -> Result<String, PlayerError> {
        // In a real implementation, this would use a template engine
        Ok(format!(
            "<div>Index page for IP: {}, Password: {}, Uptime: {}</div>",
            index_data.ip, index_data.password, index_data.uptime
        ))
    }

    async fn render_game_over_template(&self, data: GameOverData) -> Result<String, PlayerError> {
        Ok("<div>Game Over page</div>".to_string())
    }

    async fn render_hardware_template(&self, cpu: String, hdd: String, ram: String, net: String, xhd: String) -> Result<String, PlayerError> {
        Ok(format!(
            "<div>Hardware: CPU: {}, HDD: {}, RAM: {}, NET: {}, XHD: {}</div>",
            cpu, hdd, ram, net, xhd
        ))
    }

    async fn render_userinfo_template(&self, user_info: UserDisplayInfo, mission: Option<MissionInfo>, clan: Option<ClanInfo>, connected_to: String, running_tasks: i64) -> Result<String, PlayerError> {
        Ok(format!(
            "<div>User: Rep: {}, Rank: {}, Membership: {}, Tasks: {}</div>",
            user_info.reputation, user_info.rank, user_info.membership, running_tasks
        ))
    }

    async fn render_top10_template(&self, players: Vec<PlayerRanking>) -> Result<String, PlayerError> {
        let mut html = "<div>Top Players:<ul>".to_string();
        for (i, player) in players.iter().enumerate() {
            html.push_str(&format!(
                "<li>{}. {} - {}</li>",
                i + 1, player.login, player.reputation
            ));
        }
        html.push_str("</ul></div>");
        Ok(html)
    }

    async fn render_news_template(&self, news: Vec<NewsItem>) -> Result<String, PlayerError> {
        Ok("<div>News section</div>".to_string())
    }

    async fn render_round_template(&self, round_info: String) -> Result<String, PlayerError> {
        Ok(format!("<div>{}</div>", round_info))
    }

    async fn render_forum_comments_template(&self, comments: Vec<ForumPost>) -> Result<String, PlayerError> {
        Ok("<div>Recent Comments</div>".to_string())
    }

    async fn render_forum_posts_template(&self, posts: Vec<ForumPost>) -> Result<String, PlayerError> {
        Ok("<div>Recent Posts</div>".to_string())
    }

    async fn render_announcements_template(&self, announcements: Vec<ForumPost>) -> Result<String, PlayerError> {
        Ok("<div>Announcements</div>".to_string())
    }

    async fn render_settings_template(&self) -> Result<String, PlayerError> {
        Ok("<div>Settings page</div>".to_string())
    }

    // Utility methods

    fn long_to_ip(&self, ip: i64) -> String {
        let ip = ip as u32;
        format!("{}.{}.{}.{}", 
            (ip >> 24) & 0xFF,
            (ip >> 16) & 0xFF, 
            (ip >> 8) & 0xFF,
            ip & 0xFF
        )
    }
}