//! End clan war job
//! 
//! This job processes finished clan wars, determines winners, distributes rewards,
//! and creates news entries. Equivalent to the legacy endWar.php cron job.

use crate::error::{CronError, CronResult};
use crate::utils::round_up;
use sqlx::{MySqlPool, Row};
use std::sync::Arc;
use tracing::{info, warn, error};

/// End war job implementation
pub struct EndWarJob;

#[derive(Debug, sqlx::FromRow)]
struct FinishedWar {
    clan_id1: i32,
    clan_id2: i32,
    score1: i64,
    score2: i64,
    name1: String,
    name2: String,
    end_date: chrono::NaiveDateTime,
    start_date: chrono::NaiveDateTime,
    bounty: i64,
}

#[derive(Debug)]
struct DdosContributor {
    user_id: i32,
    power: i64,
}

impl EndWarJob {
    /// Execute the end war job
    pub async fn execute(db_pool: Arc<MySqlPool>) -> CronResult<()> {
        info!("Starting clan war ending process");
        
        let finished_wars = Self::get_finished_wars(&db_pool).await?;
        
        if finished_wars.is_empty() {
            info!("No wars to process");
            return Ok(());
        }
        
        info!("Processing {} finished wars", finished_wars.len());
        
        for war in finished_wars {
            if let Err(e) = Self::process_war(&db_pool, &war).await {
                error!("Failed to process war between clans {} and {}: {}", 
                       war.clan_id1, war.clan_id2, e);
                return Err(e);
            }
        }
        
        info!("Clan war ending process completed");
        Ok(())
    }
    
    /// Get all wars that have finished
    async fn get_finished_wars(db_pool: &MySqlPool) -> CronResult<Vec<FinishedWar>> {
        let wars = sqlx::query_as::<_, FinishedWar>(
            "SELECT 
                clan_war.clanID1 as clan_id1, clan_war.clanID2 as clan_id2, 
                clan_war.score1, clan_war.score2, 
                id1.name AS name1, id2.name AS name2, 
                clan_war.endDate as end_date, clan_war.startDate as start_date, 
                clan_war.bounty
             FROM clan_war 
             INNER JOIN clan id1 ON clan_war.clanID1 = id1.clanID
             INNER JOIN clan id2 ON clan_war.clanID2 = id2.clanID
             WHERE TIMESTAMPDIFF(SECOND, NOW(), endDate) < 0"
        )
        .fetch_all(db_pool)
        .await?;
        
        Ok(wars)
    }
    
    /// Process a single finished war
    async fn process_war(db_pool: &MySqlPool, war: &FinishedWar) -> CronResult<()> {
        info!("Processing war between {} and {}", war.name1, war.name2);
        
        // Determine winner and loser
        let (winner_id, loser_id, winner_score, loser_score, winner_name, loser_name) = 
            if war.score1 > war.score2 {
                (war.clan_id1, war.clan_id2, war.score1, war.score2, &war.name1, &war.name2)
            } else {
                (war.clan_id2, war.clan_id1, war.score2, war.score1, &war.name2, &war.name1)
            };
        
        // Get DDOS contributors for the winning clan
        let ddos_contributors = Self::get_ddos_contributors(db_pool, winner_id, loser_id).await?;
        
        if ddos_contributors.is_empty() {
            warn!("No DDOS contributors found for war between {} and {}", winner_name, loser_name);
        } else {
            // Distribute bounty among contributors
            Self::distribute_bounty(db_pool, &ddos_contributors, war.bounty).await?;
        }
        
        // Get most influential player
        let most_influential = Self::get_most_influential_player(db_pool, &ddos_contributors).await?;
        
        // Create news entry
        Self::create_news_entry(
            db_pool, 
            winner_name, 
            loser_name, 
            winner_id,
            loser_id,
            winner_score, 
            loser_score, 
            &war.end_date,
            &most_influential,
            war.bounty,
            ddos_contributors.len()
        ).await?;
        
        // Update clan statistics
        Self::update_clan_stats(db_pool, winner_id, loser_id, winner_score + loser_score).await?;
        
        // Archive the war
        Self::archive_war(db_pool, war, winner_id, loser_id, winner_score, loser_score).await?;
        
        // Clean up
        Self::cleanup_war_data(db_pool, winner_id, loser_id).await?;
        
        info!("War between {} and {} processed successfully", winner_name, loser_name);
        Ok(())
    }
    
    /// Get DDOS contributors for the winning clan
    async fn get_ddos_contributors(
        db_pool: &MySqlPool, 
        winner_id: i32, 
        loser_id: i32
    ) -> CronResult<Vec<DdosContributor>> {
        let rows = sqlx::query(
            "SELECT round_ddos.attID as user_id, round_ddos.power
             FROM round_ddos
             INNER JOIN clan_ddos ON clan_ddos.ddosID = round_ddos.id
             WHERE clan_ddos.attackerClan = ? AND victimClan = ?"
        )
        .bind(winner_id)
        .bind(loser_id)
        .fetch_all(db_pool)
        .await?;
        
        let mut contributors: std::collections::HashMap<i32, i64> = std::collections::HashMap::new();
        
        // Aggregate power by user (in case a user has multiple DDoS attacks)
        for row in rows {
            let user_id: i32 = row.try_get("user_id")?;
            let power: i64 = row.try_get("power")?;
            
            *contributors.entry(user_id).or_insert(0) += power;
        }
        
        Ok(contributors
            .into_iter()
            .map(|(user_id, power)| DdosContributor { user_id, power })
            .collect())
    }
    
    /// Distribute bounty among DDOS contributors based on their contribution
    async fn distribute_bounty(
        db_pool: &MySqlPool, 
        contributors: &[DdosContributor], 
        total_bounty: i64
    ) -> CronResult<()> {
        let total_power: i64 = contributors.iter().map(|c| c.power).sum();
        
        if total_power == 0 {
            warn!("Total power is 0, cannot distribute bounty");
            return Ok(());
        }
        
        for contributor in contributors {
            let influence = contributor.power as f64 / total_power as f64;
            let earned = round_up(total_bounty as f64 * influence);
            
            // Find user's bank account with lowest cash
            let bank_account = sqlx::query(
                "SELECT bankAcc FROM bankAccounts WHERE bankUser = ? ORDER BY cash ASC LIMIT 1"
            )
            .bind(contributor.user_id)
            .fetch_optional(db_pool)
            .await?;
            
            if let Some(account) = bank_account {
                let bank_acc: String = account.try_get("bankAcc")?;
                
                // Add money to bank account
                sqlx::query(
                    "UPDATE bankAccounts SET cash = cash + ? WHERE bankAcc = ?"
                )
                .bind(earned)
                .bind(&bank_acc)
                .execute(db_pool)
                .await?;
                
                // Update user stats
                sqlx::query(
                    "UPDATE users_stats SET moneyEarned = moneyEarned + ? WHERE uid = ?"
                )
                .bind(earned)
                .bind(contributor.user_id)
                .execute(db_pool)
                .await?;
                
                info!("Distributed {} to user {}", earned, contributor.user_id);
            } else {
                warn!("No bank account found for user {}", contributor.user_id);
            }
        }
        
        Ok(())
    }
    
    /// Get the most influential player (highest contribution)
    async fn get_most_influential_player(
        db_pool: &MySqlPool, 
        contributors: &[DdosContributor]
    ) -> CronResult<Option<(i32, String)>> {
        if contributors.is_empty() {
            return Ok(None);
        }
        
        let most_influential = contributors.iter().max_by_key(|c| c.power);
        
        if let Some(contributor) = most_influential {
            let player_name = sqlx::query(
                "SELECT login FROM users WHERE id = ?"
            )
            .bind(contributor.user_id)
            .fetch_optional(db_pool)
            .await?;
            
            if let Some(row) = player_name {
                let name: String = row.try_get("login")?;
                return Ok(Some((contributor.user_id, name)));
            }
        }
        
        Ok(None)
    }
    
    /// Create a news entry for the finished war
    async fn create_news_entry(
        db_pool: &MySqlPool,
        winner_name: &str,
        loser_name: &str,
        winner_id: i32,
        loser_id: i32,
        winner_score: i64,
        loser_score: i64,
        end_date: &chrono::NaiveDateTime,
        most_influential: &Option<(i32, String)>,
        bounty: i64,
        contributor_count: usize,
    ) -> CronResult<()> {
        let title = format!("{} won clan battle against {}", winner_name, loser_name);
        
        let mut content = format!(
            "The war against <a href=\"clan?id={}\">{}</a> and <a href=\"clan?id={}\">{}</a> reached its end at {}<br/>
            The total score was <font color=\"green\"><b>{}</b></font> for <a href=\"clan?id={}\">{}</a>,
            and <font color=\"red\"><b>{}</b></font> for <a href=\"clan?id={}\">{}</a>.<br/>",
            winner_id, winner_name, loser_id, loser_name,
            end_date.format("%Y-%m-%d %H:%M"),
            winner_score, winner_id, winner_name,
            loser_score, loser_id, loser_name
        );
        
        if let Some((player_id, player_name)) = most_influential {
            let total_power: i64 = winner_score;
            content.push_str(&format!(
                " The most influent player was <a href=\"profile?id={}\">{}</a>.<br/>",
                player_id, player_name
            ));
        }
        
        content.push_str(&format!(
            " The total bounty for this clan war was <font color=\"green\">${}</font>, split between {} players.",
            bounty, contributor_count
        ));
        
        sqlx::query(
            "INSERT INTO news (id, author, title, content, date) VALUES (NULL, -5, ?, ?, NOW())"
        )
        .bind(&title)
        .bind(&content)
        .execute(db_pool)
        .await?;
        
        Ok(())
    }
    
    /// Update clan statistics for winner and loser
    async fn update_clan_stats(
        db_pool: &MySqlPool, 
        winner_id: i32, 
        loser_id: i32, 
        total_score: i64
    ) -> CronResult<()> {
        // Update winner clan
        let power_increase = total_score as f64 / 8.0;
        sqlx::query(
            "UPDATE clan
             INNER JOIN clan_stats ON clan.clanID = clan_stats.cid
             SET clan_stats.won = clan_stats.won + 1, clan.power = clan.power + ?
             WHERE clan.clanID = ?"
        )
        .bind(power_increase)
        .bind(winner_id)
        .execute(db_pool)
        .await?;
        
        // Update loser clan
        sqlx::query(
            "UPDATE clan_stats SET lost = lost + 1 WHERE cid = ?"
        )
        .bind(loser_id)
        .execute(db_pool)
        .await?;
        
        Ok(())
    }
    
    /// Archive the war to history
    async fn archive_war(
        db_pool: &MySqlPool,
        war: &FinishedWar,
        winner_id: i32,
        loser_id: i32,
        winner_score: i64,
        loser_score: i64,
    ) -> CronResult<()> {
        // Add to clan war history
        let war_id = sqlx::query(
            "INSERT INTO clan_war_history 
             (id, idWinner, idLoser, scoreWinner, scoreLoser, startDate, endDate, bounty)
             VALUES (NULL, ?, ?, ?, ?, ?, NOW(), ?)"
        )
        .bind(winner_id)
        .bind(loser_id)
        .bind(winner_score)
        .bind(loser_score)
        .bind(&war.start_date)
        .bind(war.bounty)
        .execute(db_pool)
        .await?
        .last_insert_id();
        
        // Archive DDOS data
        let ddos_data = sqlx::query(
            "SELECT attackerClan, victimClan, ddosID FROM clan_ddos 
             WHERE (attackerClan = ? AND victimClan = ?) OR (attackerClan = ? AND victimClan = ?)"
        )
        .bind(winner_id)
        .bind(loser_id)
        .bind(loser_id)
        .bind(winner_id)
        .fetch_all(db_pool)
        .await?;
        
        for row in ddos_data {
            let attacker_clan: i32 = row.try_get("attackerClan")?;
            let victim_clan: i32 = row.try_get("victimClan")?;
            let ddos_id: i32 = row.try_get("ddosID")?;
            
            sqlx::query(
                "INSERT INTO clan_ddos_history (attackerClan, victimClan, ddosID, warID) 
                 VALUES (?, ?, ?, ?)"
            )
            .bind(attacker_clan)
            .bind(victim_clan)
            .bind(ddos_id)
            .bind(war_id)
            .execute(db_pool)
            .await?;
        }
        
        Ok(())
    }
    
    /// Clean up war data
    async fn cleanup_war_data(db_pool: &MySqlPool, winner_id: i32, loser_id: i32) -> CronResult<()> {
        // Remove from active wars (commented out in original PHP - keeping for reference)
        // sqlx::query(
        //     "DELETE FROM clan_war WHERE (clanID1 = ? and clanID2 = ?) OR (clanID2 = ? and clanID1 = ?)"
        // )
        // .bind(winner_id).bind(loser_id).bind(winner_id).bind(loser_id)
        // .execute(db_pool).await?;
        
        // Remove DDOS data (commented out in original PHP - keeping for reference)
        // sqlx::query(
        //     "DELETE FROM clan_ddos WHERE (attackerClan = ? AND victimClan = ?) OR (attackerClan = ? AND victimClan = ?)"
        // )
        // .bind(winner_id).bind(loser_id).bind(loser_id).bind(winner_id)
        // .execute(db_pool).await?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ddos_contributor() {
        let contributor = DdosContributor {
            user_id: 123,
            power: 1000,
        };
        
        assert_eq!(contributor.user_id, 123);
        assert_eq!(contributor.power, 1000);
    }
}