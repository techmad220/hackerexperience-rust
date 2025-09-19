//! Update server statistics job
//! 
//! This job updates round statistics with current game state data.
//! Equivalent to the legacy updateServerStats.php cron job.

use crate::error::{CronError, CronResult};
use sqlx::{MySqlPool, Row};
use std::sync::Arc;
use tracing::{info, error};

/// Update server stats job implementation
pub struct UpdateServerStatsJob;

#[derive(Debug, Default)]
struct ServerStats {
    total_users: i64,
    active_users: i64,
    online_users: i64,
    users_clicks: i64,
    warez_sent: i64,
    spam_sent: i64,
    bitcoin_sent: i64,
    mail_sent: i64,
    ddos_count: i64,
    hack_count: i64,
    clans: i64,
    clans_war: i64,
    clans_members: i64,
    clans_clicks: i64,
    time_playing: i64,
    total_listed: i64,
    total_virus: i64,
    total_money: i64,
    money_hardware: i64,
    money_earned: i64,
    money_transfered: i64,
    money_research: i64,
    mission_count: i64,
    total_connections: i64,
    research_count: i64,
    total_tasks: i64,
    total_software: i64,
    total_running: i64,
    total_servers: i64,
}

impl UpdateServerStatsJob {
    /// Execute the update server stats job
    pub async fn execute(db_pool: Arc<MySqlPool>) -> CronResult<()> {
        info!("Starting server statistics update");
        
        let stats = Self::collect_statistics(&db_pool).await?;
        Self::update_round_stats(&db_pool, &stats).await?;
        
        info!("Server statistics updated successfully");
        Ok(())
    }
    
    /// Collect all server statistics
    async fn collect_statistics(db_pool: &MySqlPool) -> CronResult<ServerStats> {
        let mut stats = ServerStats::default();
        
        // Total users from cache_profile
        stats.total_users = Self::get_single_count(
            db_pool,
            "SELECT COUNT(*) as count FROM cache_profile"
        ).await?;
        
        // Active users (logged in within 14 days)
        stats.active_users = Self::get_single_count(
            db_pool,
            "SELECT COUNT(*) as count FROM users WHERE TIMESTAMPDIFF(DAY, lastLogin, NOW()) <= 14"
        ).await?;
        
        // Currently online users
        stats.online_users = Self::get_single_count(
            db_pool,
            "SELECT COUNT(*) as count FROM users_online"
        ).await?;
        
        // User activity statistics
        let user_stats = Self::get_user_stats_aggregation(db_pool).await?;
        stats.users_clicks = user_stats.0;
        stats.warez_sent = user_stats.1;
        stats.spam_sent = user_stats.2;
        stats.bitcoin_sent = user_stats.3;
        stats.ddos_count = user_stats.4;
        stats.hack_count = user_stats.5;
        stats.time_playing = user_stats.6;
        stats.money_transfered = user_stats.7;
        stats.money_hardware = user_stats.8;
        stats.money_earned = user_stats.9;
        stats.money_research = user_stats.10;
        
        // Clan statistics
        stats.clans = Self::get_single_count(
            db_pool,
            "SELECT COUNT(*) as count FROM clan"
        ).await?;
        
        stats.clans_members = Self::get_single_count(
            db_pool,
            "SELECT COUNT(*) as count FROM clan_users"
        ).await?;
        
        stats.clans_clicks = Self::get_single_count(
            db_pool,
            "SELECT COALESCE(SUM(pageClicks), 0) as count FROM clan_stats"
        ).await?;
        
        stats.clans_war = Self::get_single_count(
            db_pool,
            "SELECT COUNT(*) as count FROM clan_war_history"
        ).await?;
        
        // Game content statistics
        stats.total_listed = Self::get_single_count(
            db_pool,
            "SELECT COUNT(*) as count FROM lists"
        ).await?;
        
        stats.total_virus = Self::get_single_count(
            db_pool,
            "SELECT COUNT(*) as count FROM virus"
        ).await?;
        
        stats.total_money = Self::get_single_count(
            db_pool,
            "SELECT COALESCE(SUM(cash), 0) as count FROM bankAccounts"
        ).await?;
        
        stats.mission_count = Self::get_single_count(
            db_pool,
            "SELECT COUNT(*) as count FROM missions_history WHERE completed = 1"
        ).await?;
        
        stats.mail_sent = Self::get_single_count(
            db_pool,
            "SELECT COUNT(*) as count FROM mails WHERE mails.from > 0"
        ).await?;
        
        stats.total_connections = Self::get_single_count(
            db_pool,
            "SELECT COUNT(*) as count FROM internet_connections"
        ).await?;
        
        stats.research_count = Self::get_single_count(
            db_pool,
            "SELECT COUNT(*) as count FROM software_research"
        ).await?;
        
        stats.total_tasks = Self::get_single_count(
            db_pool,
            "SELECT COUNT(*) as count FROM processes"
        ).await?;
        
        stats.total_software = Self::get_single_count(
            db_pool,
            "SELECT COUNT(*) as count FROM software"
        ).await?;
        
        stats.total_running = Self::get_single_count(
            db_pool,
            "SELECT COUNT(*) as count FROM software_running"
        ).await?;
        
        stats.total_servers = Self::get_single_count(
            db_pool,
            "SELECT COUNT(*) as count FROM hardware"
        ).await?;
        
        Ok(stats)
    }
    
    /// Get a single count value from the database
    async fn get_single_count(db_pool: &MySqlPool, query: &str) -> CronResult<i64> {
        let row = sqlx::query(query)
            .fetch_one(db_pool)
            .await?;
        
        Ok(row.try_get::<i64, _>("count").unwrap_or(0))
    }
    
    /// Get aggregated user statistics
    async fn get_user_stats_aggregation(db_pool: &MySqlPool) -> CronResult<(i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64)> {
        let row = sqlx::query(
            "SELECT 
                COALESCE(SUM(profileViews), 0) as clicks,
                COALESCE(SUM(warezSent), 0) as warez, 
                COALESCE(SUM(spamSent), 0) as spam,
                COALESCE(SUM(bitcoinSent), 0) as bitcoin,
                COALESCE(SUM(ddosCount), 0) as ddos, 
                COALESCE(SUM(hackCount), 0) as hack,
                COALESCE(SUM(timePlaying), 0) as time_playing,
                COALESCE(SUM(moneyTransfered), 0) as money_transfered,
                COALESCE(SUM(moneyHardware), 0) as money_hardware,
                COALESCE(SUM(moneyEarned), 0) as money_earned,
                COALESCE(SUM(moneyResearch), 0) as money_research
             FROM users_stats"
        )
        .fetch_one(db_pool)
        .await?;
        
        Ok((
            row.try_get("clicks").unwrap_or(0),
            row.try_get("warez").unwrap_or(0),
            row.try_get("spam").unwrap_or(0),
            row.try_get("bitcoin").unwrap_or(0),
            row.try_get("ddos").unwrap_or(0),
            row.try_get("hack").unwrap_or(0),
            row.try_get("time_playing").unwrap_or(0),
            row.try_get("money_transfered").unwrap_or(0),
            row.try_get("money_hardware").unwrap_or(0),
            row.try_get("money_earned").unwrap_or(0),
            row.try_get("money_research").unwrap_or(0),
        ))
    }
    
    /// Update the round_stats table with collected statistics
    async fn update_round_stats(db_pool: &MySqlPool, stats: &ServerStats) -> CronResult<()> {
        sqlx::query(
            "UPDATE round_stats 
             SET totalUsers = ?, activeUsers = ?, onlineUsers = ?, usersClicks = ?,
                 warezSent = ?, spamSent = ?, bitcoinSent = ?, mailSent = ?,
                 ddosCount = ?, hackCount = ?, clans = ?, clansWar = ?,
                 clansMembers = ?, clansClicks = ?, timePlaying = ?, 
                 totalListed = ?, totalVirus = ?, totalMoney = ?,
                 moneyHardware = ?, moneyEarned = ?, moneyTransfered = ?,
                 moneyResearch = ?, missionCount = ?, totalConnections = ?,
                 researchCount = ?, totalTasks = ?, totalSoftware = ?,
                 totalRunning = ?, totalServers = ?
             ORDER BY id DESC LIMIT 1"
        )
        .bind(stats.total_users)
        .bind(stats.active_users)
        .bind(stats.online_users)
        .bind(stats.users_clicks)
        .bind(stats.warez_sent)
        .bind(stats.spam_sent)
        .bind(stats.bitcoin_sent)
        .bind(stats.mail_sent)
        .bind(stats.ddos_count)
        .bind(stats.hack_count)
        .bind(stats.clans)
        .bind(stats.clans_war)
        .bind(stats.clans_members)
        .bind(stats.clans_clicks)
        .bind(stats.time_playing)
        .bind(stats.total_listed)
        .bind(stats.total_virus)
        .bind(stats.total_money)
        .bind(stats.money_hardware)
        .bind(stats.money_earned)
        .bind(stats.money_transfered)
        .bind(stats.money_research)
        .bind(stats.mission_count)
        .bind(stats.total_connections)
        .bind(stats.research_count)
        .bind(stats.total_tasks)
        .bind(stats.total_software)
        .bind(stats.total_running)
        .bind(stats.total_servers)
        .execute(db_pool)
        .await?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_server_stats_default() {
        let stats = ServerStats::default();
        assert_eq!(stats.total_users, 0);
        assert_eq!(stats.active_users, 0);
    }
}