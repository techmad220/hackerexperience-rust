// RANKING.CLASS.PHP PORT - Player statistics, certifications, and leaderboards
// Original: Extends Player class with ranking and certification system

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use sqlx::MySqlPool;
use serde::{Deserialize, Serialize};
use he_core::*;
use he_db::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certification {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub price: i32,
    pub requirements: Vec<i32>, // Required cert IDs
    pub research_bonus: f32,
    pub exp_bonus: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    pub user_id: i64,
    pub exp: i32,
    pub level: i32,
    pub ddos_count: i32,
    pub time_playing: f32, // in minutes
    pub money_earned: i64,
    pub money_hardware: i64,
    pub money_research: i64,
    pub money_transferred: i64,
    pub research_points: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingEntry {
    pub user_id: i64,
    pub username: String,
    pub value: i64,
    pub rank: i32,
}

pub struct Ranking {
    db_pool: MySqlPool,
    server_info: Option<HashMap<String, String>>,
}

impl Ranking {
    pub fn new(db_pool: MySqlPool) -> Self {
        Self {
            db_pool,
            server_info: None,
        }
    }
    
    // Original PHP: handlePost - Process certification purchases
    pub async fn handle_post(&self, post_data: HashMap<String, String>) -> Result<String, RankingError> {
        let redirect = "university?opt=certification";
        
        let act = post_data.get("act")
            .ok_or_else(|| RankingError::ValidationError("Invalid action".to_string()))?;
            
        match act.as_str() {
            "buy" => {
                let cert_id = post_data.get("id")
                    .and_then(|id| id.parse::<i32>().ok())
                    .ok_or_else(|| RankingError::ValidationError("Invalid ID".to_string()))?;
                    
                if cert_id < 1 || cert_id > 6 {
                    return Err(RankingError::ValidationError("Invalid ID".to_string()));
                }
                
                // TODO: Get user ID from session
                let user_id = 0i64; // placeholder
                
                // Check if player already has certification
                if self.cert_have(user_id, cert_id).await? {
                    return Err(RankingError::ValidationError("You already have this certification.".to_string()));
                }
                
                // Check if player has required certifications
                if !self.cert_have_requirements(user_id, cert_id).await? {
                    return Err(RankingError::ValidationError("You do not have the needed certifications to take this one.".to_string()));
                }
                
                // Check if player is already learning something
                let current_learning = self.get_player_learning(user_id).await?;
                if current_learning > 0 {
                    if current_learning == cert_id {
                        return Err(RankingError::ValidationError("You already bought this certification.".to_string()));
                    } else {
                        return Err(RankingError::ValidationError("You are learning a different certification.".to_string()));
                    }
                }
                
                // Get certification price
                let cert_info = self.get_certification_info(cert_id).await?;
                
                if cert_info.price > 0 {
                    let account_id = post_data.get("acc")
                        .and_then(|acc| acc.parse::<i64>().ok())
                        .ok_or_else(|| RankingError::ValidationError("Invalid bank account".to_string()))?;
                        
                    // TODO: Check if player has enough money and valid account
                    // let finances = Finances::new();
                    // if !finances.has_enough_money(user_id, cert_info.price).await? {
                    //     return Err(RankingError::ValidationError("You do not have enough money".to_string()));
                    // }
                    // finances.debit_money(cert_info.price, account_id).await?;
                }
                
                // Set player as learning this certification
                self.set_player_learning(user_id, cert_id).await?;
                
                Ok(redirect.to_string())
            },
            _ => Err(RankingError::ValidationError(format!("Unknown action: {}", act))),
        }
    }
    
    // Original PHP: updateDDoSCount - Update DDoS attack statistics
    pub async fn update_ddos_count(&self, user_id: i64, total: i32) -> Result<(), RankingError> {
        sqlx::query("UPDATE users_stats SET ddosCount = ddosCount + ? WHERE uid = ?")
            .bind(total)
            .bind(user_id)
            .execute(&self.db_pool)
            .await
            .map_err(RankingError::DatabaseError)?;
            
        Ok(())
    }
    
    // Original PHP: updateTimePlayed - Update time played statistics
    pub async fn update_time_played(&self, user_id: i64, minutes: f32) -> Result<(), RankingError> {
        sqlx::query("UPDATE users_stats SET timePlaying = timePlaying + ? WHERE uid = ?")
            .bind(minutes)
            .bind(user_id)
            .execute(&self.db_pool)
            .await
            .map_err(RankingError::DatabaseError)?;
            
        Ok(())
    }
    
    // Original PHP: updateMoneyStats - Update money-related statistics
    pub async fn update_money_stats(&self, user_id: i64, stat_type: i32, amount: i64) -> Result<(), RankingError> {
        let column = match stat_type {
            1 => "moneyHardware",   // hardware purchases
            2 => "moneyResearch",   // research spending
            3 => "moneyTransfered", // money transfers
            _ => "moneyEarned",     // money earned
        };
        
        let query = format!("UPDATE users_stats SET {} = {} + ? WHERE uid = ?", column, column);
        
        sqlx::query(&query)
            .bind(amount)
            .bind(user_id)
            .execute(&self.db_pool)
            .await
            .map_err(RankingError::DatabaseError)?;
            
        Ok(())
    }
    
    // Original PHP: exp_getLevel - Calculate level from experience points
    pub fn exp_get_level(&self, exp: i32) -> i32 {
        match exp {
            0..=99 => 1,
            100..=499 => 2,
            500..=999 => 3,
            1000..=2499 => 4,
            2500..=4999 => 5,
            5000..=9999 => 6,
            10000..=19999 => 7,
            20000..=39999 => 8,
            40000..=79999 => 9,
            _ => 10,
        }
    }
    
    // Original PHP: getResearchRank - Get player's research ranking
    pub async fn get_research_rank(&self, user_id: i64) -> Result<RankingEntry, RankingError> {
        // Get total research points and rank
        let research_data = sqlx::query_as::<_, (i64, i32, i32)>(
            "SELECT userID, COUNT(*) as total, SUM(newVersion) as research_points
             FROM software_research 
             WHERE userID = ?
             GROUP BY userID"
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(RankingError::DatabaseError)?;
        
        if let Some((_, _, points)) = research_data {
            // Calculate rank by counting users with higher points
            let rank = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) + 1 FROM (
                    SELECT userID, SUM(newVersion) as total_points
                    FROM software_research 
                    GROUP BY userID
                    HAVING total_points > ?
                ) as higher_ranks"
            )
            .bind(points)
            .fetch_one(&self.db_pool)
            .await
            .map_err(RankingError::DatabaseError)? as i32;
            
            // Get username
            let username = sqlx::query_scalar::<_, String>(
                "SELECT login FROM users WHERE id = ?"
            )
            .bind(user_id)
            .fetch_one(&self.db_pool)
            .await
            .map_err(RankingError::DatabaseError)?;
            
            Ok(RankingEntry {
                user_id,
                username,
                value: points as i64,
                rank,
            })
        } else {
            Err(RankingError::UserNotFound(user_id))
        }
    }
    
    // Original PHP: cert_have - Check if player has certification
    pub async fn cert_have(&self, user_id: i64, cert_id: i32) -> Result<bool, RankingError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users_certifications WHERE user_id = ? AND cert_id = ?"
        )
        .bind(user_id)
        .bind(cert_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(RankingError::DatabaseError)?;
        
        Ok(count > 0)
    }
    
    // Original PHP: cert_haveReq - Check if player has required certifications
    pub async fn cert_have_requirements(&self, user_id: i64, cert_id: i32) -> Result<bool, RankingError> {
        let cert_info = self.get_certification_info(cert_id).await?;
        
        for required_cert in cert_info.requirements {
            if !self.cert_have(user_id, required_cert).await? {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    // Original PHP: cert_getAll - Get all certifications for a player
    pub async fn cert_get_all(&self, user_id: i64) -> Result<Vec<i32>, RankingError> {
        let cert_ids = sqlx::query_scalar::<_, Vec<i32>>(
            "SELECT cert_id FROM users_certifications WHERE user_id = ? ORDER BY cert_id"
        )
        .bind(user_id)
        .fetch_all(&self.db_pool)
        .await
        .map_err(RankingError::DatabaseError)?;
        
        Ok(cert_ids)
    }
    
    // Get player's current learning certification
    pub async fn get_player_learning(&self, user_id: i64) -> Result<i32, RankingError> {
        let learning_id = sqlx::query_scalar::<_, Option<i32>>(
            "SELECT learning_cert_id FROM users_learning WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(RankingError::DatabaseError)?
        .flatten();
        
        Ok(learning_id.unwrap_or(0))
    }
    
    // Set player as learning a certification
    pub async fn set_player_learning(&self, user_id: i64, cert_id: i32) -> Result<(), RankingError> {
        sqlx::query(
            "INSERT INTO users_learning (user_id, learning_cert_id, start_time) 
             VALUES (?, ?, NOW()) 
             ON DUPLICATE KEY UPDATE learning_cert_id = ?, start_time = NOW()"
        )
        .bind(user_id)
        .bind(cert_id)
        .bind(cert_id)
        .execute(&self.db_pool)
        .await
        .map_err(RankingError::DatabaseError)?;
        
        Ok(())
    }
    
    // Get certification information
    async fn get_certification_info(&self, cert_id: i32) -> Result<Certification, RankingError> {
        // TODO: This should load from a configuration file or database
        // For now, return hard-coded cert info based on original certs.php
        let cert = match cert_id {
            1 => Certification {
                id: 1,
                name: "Basic Hacking".to_string(),
                description: "Learn basic hacking techniques".to_string(),
                price: 1000,
                requirements: vec![],
                research_bonus: 1.0,
                exp_bonus: 50,
            },
            2 => Certification {
                id: 2,
                name: "Advanced Hacking".to_string(),
                description: "Advanced hacking methods".to_string(),
                price: 5000,
                requirements: vec![1],
                research_bonus: 1.2,
                exp_bonus: 100,
            },
            3 => Certification {
                id: 3,
                name: "Network Security".to_string(),
                description: "Network security fundamentals".to_string(),
                price: 10000,
                requirements: vec![1, 2],
                research_bonus: 1.3,
                exp_bonus: 150,
            },
            4 => Certification {
                id: 4,
                name: "Cryptography".to_string(),
                description: "Advanced cryptographic techniques".to_string(),
                price: 25000,
                requirements: vec![1, 2, 3],
                research_bonus: 1.5,
                exp_bonus: 200,
            },
            5 => Certification {
                id: 5,
                name: "System Administration".to_string(),
                description: "Advanced system administration".to_string(),
                price: 50000,
                requirements: vec![1, 2, 3, 4],
                research_bonus: 1.7,
                exp_bonus: 300,
            },
            6 => Certification {
                id: 6,
                name: "Expert Hacker".to_string(),
                description: "Master level hacking certification".to_string(),
                price: 100000,
                requirements: vec![1, 2, 3, 4, 5],
                research_bonus: 2.0,
                exp_bonus: 500,
            },
            _ => return Err(RankingError::CertificationNotFound(cert_id)),
        };
        
        Ok(cert)
    }

    /// Display ranking based on type
    pub async fn ranking_display(&self, display_type: &str) -> Result<String, RankingError> {
        match display_type {
            "user" => self.display_user_ranking().await,
            "clan" => self.display_clan_ranking().await,
            "software" => self.display_software_ranking().await,
            "ddos" => self.display_ddos_ranking().await,
            _ => Err(RankingError::ValidationError("Invalid display type".to_string())),
        }
    }

    /// Display user ranking
    async fn display_user_ranking(&self) -> Result<String, RankingError> {
        let rankings = sqlx::query!(
            "SELECT u.id, u.login, u.exp FROM users u WHERE u.exp > 0 ORDER BY u.exp DESC LIMIT 50"
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(RankingError::DatabaseError)?;

        let mut html = String::from(r#"
            <table class="table table-striped">
                <thead>
                    <tr>
                        <th>Rank</th>
                        <th>Player</th>
                        <th>Experience</th>
                    </tr>
                </thead>
                <tbody>
        "#);

        for (index, row) in rankings.iter().enumerate() {
            html.push_str(&format!(r#"
                <tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>
            "#, index + 1, row.login, row.exp.unwrap_or(0)));
        }

        html.push_str("</tbody></table>");
        Ok(html)
    }

    /// Display clan ranking
    async fn display_clan_ranking(&self) -> Result<String, RankingError> {
        // TODO: Implement clan ranking logic
        Ok("<p>Clan ranking coming soon!</p>".to_string())
    }

    /// Display software ranking
    async fn display_software_ranking(&self) -> Result<String, RankingError> {
        // TODO: Implement software ranking logic
        Ok("<p>Software ranking coming soon!</p>".to_string())
    }

    /// Display DDoS ranking
    async fn display_ddos_ranking(&self) -> Result<String, RankingError> {
        // TODO: Implement DDoS ranking logic
        Ok("<p>DDoS ranking coming soon!</p>".to_string())
    }

    /// Validate if player can learn certification
    pub async fn cert_validate2learn(&self, cert_id: i64) -> Result<bool, RankingError> {
        let cert_id = cert_id as i32;
        
        // Check if certification exists
        if cert_id < 1 || cert_id > 6 {
            return Ok(false);
        }

        // TODO: Add more validation logic based on original PHP
        // For now, just check if the certification ID is valid
        Ok(true)
    }

    /// Get total pages for certification learning
    pub async fn cert_total_pages(&self, cert_id: i64) -> Result<i64, RankingError> {
        // TODO: Get actual page count from database or configuration
        // For now, return a fixed number based on certification level
        let pages = match cert_id {
            1 => 5,
            2 => 8,
            3 => 10,
            4 => 12,
            5 => 15,
            6 => 20,
            _ => 5,
        };
        
        Ok(pages)
    }

    /// Show specific page of certification learning
    pub async fn cert_show_page(&self, cert_id: i64, page: i64) -> Result<String, RankingError> {
        let cert_info = self.get_certification_info(cert_id as i32).await?;
        let total_pages = self.cert_total_pages(cert_id).await?;
        
        // TODO: Get user ID from session
        let user_id = 1; // Placeholder
        
        // Generate completion hash for final page
        let completion_hash = if page >= total_pages {
            format!("{:x}", md5::compute(format!("cert{}{}", cert_id, user_id)))
        } else {
            String::new()
        };

        let mut html = format!(r#"
            <div class="certification-learning">
                <h3>{}</h3>
                <p>Page {} of {}</p>
                <div class="cert-content">
                    <p>Learning content for {} - Page {}</p>
                    <p>{}</p>
                </div>
        "#, cert_info.name, page + 1, total_pages, cert_info.name, page + 1, cert_info.description);

        if page >= total_pages {
            // Final page - show completion button
            html.push_str(&format!(r#"
                <div class="cert-complete">
                    <p>Congratulations! You have completed this certification.</p>
                    <a href="?opt=certification&complete={}" class="btn btn-success">Complete Certification</a>
                </div>
            "#, completion_hash));
        } else {
            // Show next page button
            html.push_str(&format!(r#"
                <div class="cert-navigation">
                    <a href="?opt=certification&learn={}&page={}" class="btn btn-primary">Next Page</a>
                </div>
            "#, cert_id, page + 1));
        }

        html.push_str("</div>");
        Ok(html)
    }

    /// List all available certifications
    pub async fn cert_list(&self) -> Result<String, RankingError> {
        // TODO: Get user ID from session
        let user_id = 1; // Placeholder
        
        let mut html = String::from(r#"
            <div class="certifications-list">
                <h3>Available Certifications</h3>
                <table class="table table-striped">
                    <thead>
                        <tr>
                            <th>Certification</th>
                            <th>Price</th>
                            <th>Status</th>
                            <th>Action</th>
                        </tr>
                    </thead>
                    <tbody>
        "#);

        for cert_id in 1..=6 {
            let cert_info = self.get_certification_info(cert_id).await?;
            let has_cert = self.cert_have(user_id, cert_id).await?;
            let has_requirements = self.cert_have_requirements(user_id, cert_id).await?;
            
            let status = if has_cert {
                "Completed"
            } else if !has_requirements {
                "Requirements not met"
            } else {
                "Available"
            };

            let action = if has_cert {
                "✓ Completed".to_string()
            } else if !has_requirements {
                "Requirements not met".to_string()
            } else {
                format!(r#"<a href="?opt=certification&learn={}" class="btn btn-primary">Learn</a>"#, cert_id)
            };

            html.push_str(&format!(r#"
                <tr>
                    <td>{}</td>
                    <td>${}</td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>
            "#, cert_info.name, cert_info.price, status, action));
        }

        html.push_str("</tbody></table></div>");
        Ok(html)
    }

    /// Add certification to player
    pub async fn cert_add(&self, cert_id: i64) -> Result<(), RankingError> {
        // TODO: Get user ID from session
        let user_id = 1; // Placeholder
        
        sqlx::query!(
            "INSERT INTO users_certifications (user_id, cert_id, completed_at) VALUES (?, ?, NOW())",
            user_id,
            cert_id
        )
        .execute(&self.db_pool)
        .await
        .map_err(RankingError::DatabaseError)?;

        Ok(())
    }

    /// End certification learning process
    pub async fn cert_end(&self, cert_id: i64) -> Result<(), RankingError> {
        // TODO: Get user ID from session
        let user_id = 1; // Placeholder
        
        // Remove from learning table
        sqlx::query!(
            "DELETE FROM users_learning WHERE user_id = ? AND learning_cert_id = ?",
            user_id,
            cert_id
        )
        .execute(&self.db_pool)
        .await
        .map_err(RankingError::DatabaseError)?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum RankingError {
    DatabaseError(sqlx::Error),
    ValidationError(String),
    UserNotFound(i64),
    CertificationNotFound(i32),
}

impl std::fmt::Display for RankingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RankingError::DatabaseError(e) => write!(f, "Database error: {}", e),
            RankingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            RankingError::UserNotFound(id) => write!(f, "User {} not found", id),
            RankingError::CertificationNotFound(id) => write!(f, "Certification {} not found", id),
        }
    }
}

impl std::error::Error for RankingError {}