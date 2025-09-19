use anyhow::{anyhow, Result};
//! Player entity - Core player functionality and management
//! 
//! This module provides the Player struct and methods for managing player accounts,
//! authentication, statistics, and various player-related operations.

use sqlx::{Pool, Postgres, Row};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::entities::session::Session;
use crate::entities::system::System;
use crate::error::HeResult;

/// Represents a player in the Hacker Experience game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// Player's unique identifier
    pub id: Option<i32>,
    /// Player's login name/username
    pub name: Option<String>,
    /// Player's game IP address (as integer)
    pub game_ip: Option<i64>,
    /// Current game round
    pub cur_round: Option<i32>,
    /// Database connection pool
    #[serde(skip)]
    pub db_pool: Option<Pool<Postgres>>,
}

/// Player information returned from database queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub total: i64,
    pub login: String,
    pub game_ip: i64,
    pub home_ip: Option<i64>,
    pub game_pass: String,
    pub email: String,
}

/// Bank account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankInfo {
    pub bank_acc: String,
    pub bank_pass: String,
    pub cash: i64,
}

/// IP/PC identification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpIdResult {
    pub id: Option<i32>,
    pub exists: bool,
    pub pc_type: String, // "VPC" or "NPC"
}

/// Password reset information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordInfo {
    pub price: Option<i32>,
    pub next_reset: String,
}

/// IP reset information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpInfo {
    pub price: i32,
    pub next_reset: String,
}

impl Player {
    /// Creates a new Player instance
    pub fn new(db_pool: Pool<Postgres>) -> Self {
        Self {
            id: None,
            name: None,
            game_ip: None,
            cur_round: None,
            db_pool: Some(db_pool),
        }
    }

    /// Creates a new Player instance with an ID
    pub fn new_with_id(id: i32, db_pool: Pool<Postgres>) -> Self {
        Self {
            id: Some(id),
            name: None,
            game_ip: None,
            cur_round: None,
            db_pool: Some(db_pool),
        }
    }

    /// Handles POST requests for player actions
    /// 
    /// # Arguments
    /// * `action` - The action to perform
    /// * `post_data` - POST data containing action parameters
    /// 
    /// # Returns
    /// Result with redirect URL or error
    pub async fn handle_post(&self, action: &str, post_data: HashMap<String, String>) -> HeResult<String> {
        let system = System::new(self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?.clone());
        let post_redirect = "index".to_string();

        match action {
            "changepwd" => {
                let pwd_info = self.pwd_info().await?;
                
                if pwd_info.price.is_none() {
                    if pwd_info.next_reset != "0" {
                        return Err(crate::error::HeError::ValidationError(
                            "You can not reset your password now.".to_string()
                        ));
                    }
                    // TODO: Implement password change process
                    Ok("processes".to_string())
                } else {
                    // Handle paid password reset
                    let acc = post_data.get("acc")
                        .ok_or_else(|| crate::error::HeError::ValidationError("Missing account information.".to_string()))?;
                    
                    // Validate account number
                    if !acc.chars().all(char::is_numeric) {
                        return Err(crate::error::HeError::ValidationError("Invalid bank account.".to_string()));
                    }

                    // TODO: Implement finances check and process creation
                    Ok("processes".to_string())
                }
            },
            _ => Err(crate::error::HeError::ValidationError("Invalid POST data.".to_string())),
        }
    }

    /// Verifies if a user ID exists and is valid
    /// 
    /// # Arguments
    /// * `uid` - User ID to verify
    /// * `session_id` - Current session user ID (optional)
    /// 
    /// # Returns
    /// True if the user ID is valid, false otherwise
    pub async fn verify_id(&self, uid: i32, session_id: Option<i32>) -> HeResult<bool> {
        // Check if it's the current session user
        if let Some(sess_id) = session_id {
            if uid == sess_id {
                return Ok(true);
            }
        }

        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        let row = sqlx::query("SELECT COUNT(*) as total FROM users WHERE id = $1 LIMIT 1")
            .bind(uid)
            .fetch_one(db)
            .await?;
        
        let total: i64 = row.get("total");
        Ok(total == 1)
    }

    /// Gets player information by user ID
    /// 
    /// # Arguments
    /// * `uid` - User ID to get information for
    /// 
    /// # Returns
    /// PlayerInfo struct with player data
    pub async fn get_player_info(&self, uid: i32) -> HeResult<PlayerInfo> {
        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        let row = sqlx::query(
            "SELECT COUNT(*) AS total, login, gameIP, homeIP, gamePass, email 
             FROM users WHERE id = $1 LIMIT 1"
        )
        .bind(uid)
        .fetch_one(db)
        .await?;

        let total: i64 = row.get("total");
        if total == 0 {
            return Err(crate::error::HeError::NotFound("Player not found".to_string()));
        }

        Ok(PlayerInfo {
            total,
            login: row.get("login"),
            game_ip: row.get("gameIP"),
            home_ip: row.get("homeIP"),
            game_pass: row.get("gamePass"),
            email: row.get("email"),
        })
    }

    /// Unsets player learning status
    /// 
    /// # Arguments
    /// * `uid` - User ID to unset learning for
    pub async fn unset_player_learning(&self, uid: i32) -> HeResult<()> {
        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        sqlx::query("DELETE FROM users_learning WHERE userID = $1")
            .bind(uid)
            .execute(db)
            .await?;
        
        Ok(())
    }

    /// Sets player learning status
    /// 
    /// # Arguments
    /// * `uid` - User ID to set learning for
    /// * `cid` - Course/learning ID
    pub async fn set_player_learning(&self, uid: i32, cid: i32) -> HeResult<()> {
        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        sqlx::query("INSERT INTO users_learning (userID, learning) VALUES ($1, $2)")
            .bind(uid)
            .bind(cid)
            .execute(db)
            .await?;
        
        Ok(())
    }

    /// Gets player's current learning status
    /// 
    /// # Arguments
    /// * `uid` - User ID to check learning status for
    /// 
    /// # Returns
    /// Learning course ID or 0 if not learning
    pub async fn player_learning(&self, uid: i32) -> HeResult<i32> {
        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        let row = sqlx::query(
            "SELECT COUNT(*) AS total, learning FROM users_learning WHERE userID = $1 LIMIT 1"
        )
        .bind(uid)
        .fetch_one(db)
        .await?;

        let total: i64 = row.get("total");
        if total == 1 {
            Ok(row.get("learning"))
        } else {
            Ok(0)
        }
    }

    /// Gets player ID by IP address and PC type
    /// 
    /// # Arguments
    /// * `ip` - IP address to search for
    /// * `pc_type` - PC type ("VPC", "NPC", or empty for auto-detect)
    /// 
    /// # Returns
    /// IpIdResult with player information
    pub async fn get_id_by_ip(&self, ip: &str, pc_type: &str) -> HeResult<IpIdResult> {
        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let mut exists = true;
        let mut result_type = pc_type.to_string();
        let mut player_id = None;

        if pc_type != "VPC" && pc_type != "NPC" {
            // Auto-detect: try NPC first
            let npc_row = sqlx::query("SELECT id FROM npc WHERE npcIP = $1 LIMIT 1")
                .bind(ip)
                .fetch_optional(db)
                .await?;

            if let Some(row) = npc_row {
                player_id = Some(row.get::<i32, _>("id"));
                result_type = "NPC".to_string();
            } else {
                // Try VPC
                let vpc_row = sqlx::query("SELECT id FROM users WHERE gameIP = $1 LIMIT 1")
                    .bind(ip)
                    .fetch_optional(db)
                    .await?;

                if let Some(row) = vpc_row {
                    player_id = Some(row.get::<i32, _>("id"));
                    result_type = "VPC".to_string();
                } else {
                    exists = false;
                }
            }
        } else {
            // Specific type requested
            let sql = if pc_type == "VPC" {
                "SELECT id FROM users WHERE gameIP = $1 LIMIT 1"
            } else {
                "SELECT id FROM npc WHERE npcIP = $1 LIMIT 1"
            };

            let row = sqlx::query(sql)
                .bind(ip)
                .fetch_optional(db)
                .await?;

            if let Some(row) = row {
                player_id = Some(row.get::<i32, _>("id"));
                result_type = pc_type.to_string();
            } else {
                exists = false;
            }
        }

        Ok(IpIdResult {
            id: player_id,
            exists,
            pc_type: result_type,
        })
    }

    /// Checks if a bank account exists for a user
    /// 
    /// # Arguments
    /// * `uid` - User ID
    /// * `bank_id` - Bank ID
    /// 
    /// # Returns
    /// True if bank account exists, false otherwise
    pub async fn isset_bank_account(&self, uid: i32, bank_id: i32) -> HeResult<bool> {
        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        let row = sqlx::query("SELECT id FROM bankAccounts WHERE bankID = $1 AND bankUser = $2 LIMIT 1")
            .bind(bank_id)
            .bind(uid)
            .fetch_optional(db)
            .await?;

        Ok(row.is_some())
    }

    /// Gets bank account information
    /// 
    /// # Arguments
    /// * `uid` - User ID
    /// * `bank_id` - Bank ID
    /// 
    /// # Returns
    /// BankInfo with account details
    pub async fn get_bank_info(&self, uid: i32, bank_id: i32) -> HeResult<BankInfo> {
        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        let row = sqlx::query(
            "SELECT bankAcc, bankPass, cash FROM bankAccounts WHERE bankID = $1 AND bankUser = $2 LIMIT 1"
        )
        .bind(bank_id)
        .bind(uid)
        .fetch_one(db)
        .await?;

        Ok(BankInfo {
            bank_acc: row.get("bankAcc"),
            bank_pass: row.get("bankPass"),
            cash: row.get("cash"),
        })
    }

    /// Creates a new bank account
    /// 
    /// # Arguments
    /// * `uid` - User ID
    /// * `bank_id` - Bank ID
    /// * `bank_acc` - Bank account number
    /// * `bank_pass` - Bank account password
    /// 
    /// # Returns
    /// True if account was created successfully
    pub async fn create_bank_account(&self, uid: i32, bank_id: i32, bank_acc: &str, bank_pass: &str) -> HeResult<bool> {
        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        let result = sqlx::query(
            "INSERT INTO bankAccounts (bankAcc, bankID, bankPass, bankUser, cash, dateCreated) 
             VALUES ($1, $2, $3, $4, 0, NOW())"
        )
        .bind(bank_acc)
        .bind(bank_id)
        .bind(bank_pass)
        .bind(uid)
        .execute(db)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    /// Checks if a user exists by username
    /// 
    /// # Arguments
    /// * `username` - Username to check
    /// 
    /// # Returns
    /// True if user exists, false otherwise
    pub async fn isset_user(&self, username: &str) -> HeResult<bool> {
        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        let row = sqlx::query("SELECT COUNT(*) AS total FROM users WHERE login = $1 LIMIT 1")
            .bind(username)
            .fetch_one(db)
            .await?;

        let total: i64 = row.get("total");
        Ok(total == 1)
    }

    /// Gets user ID by username
    /// 
    /// # Arguments
    /// * `username` - Username to lookup
    /// 
    /// # Returns
    /// User ID
    pub async fn get_id_by_user(&self, username: &str) -> HeResult<i32> {
        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        let row = sqlx::query("SELECT id FROM users WHERE login = $1 LIMIT 1")
            .bind(username)
            .fetch_one(db)
            .await?;

        Ok(row.get("id"))
    }

    /// Counts logged in users
    /// 
    /// # Returns
    /// Number of currently logged in users
    pub async fn count_logged_in_users(&self) -> HeResult<i64> {
        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        let row = sqlx::query("SELECT COUNT(*) AS t_users FROM users_online")
            .fetch_one(db)
            .await?;

        Ok(row.get("t_users"))
    }

    /// Checks if a user is an admin
    /// 
    /// # Arguments
    /// * `uid` - User ID to check (optional, uses session if None)
    /// 
    /// # Returns
    /// True if user is admin, false otherwise
    pub async fn is_admin(&self, uid: Option<i32>) -> HeResult<bool> {
        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let user_id = uid.or(self.id).unwrap_or(0);
        
        let row = sqlx::query("SELECT COUNT(*) AS total FROM users_admin WHERE userID = $1 LIMIT 1")
            .bind(user_id)
            .fetch_one(db)
            .await?;

        let total: i64 = row.get("total");
        Ok(total > 0)
    }

    /// Checks if a user is premium
    /// 
    /// # Arguments
    /// * `uid` - User ID to check (optional, uses session if None)
    /// 
    /// # Returns
    /// True if user is premium, false otherwise
    pub async fn is_premium(&self, uid: Option<i32>) -> HeResult<bool> {
        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let user_id = uid.or(self.id).unwrap_or(0);
        
        let row = sqlx::query("SELECT COUNT(*) AS total FROM users_premium WHERE id = $1 LIMIT 1")
            .bind(user_id)
            .fetch_one(db)
            .await?;

        let total: i64 = row.get("total");
        Ok(total == 1)
    }

    /// Checks if a user is a noob (new player)
    /// 
    /// # Arguments
    /// * `uid` - User ID to check
    /// 
    /// # Returns
    /// True if user is a noob, false otherwise
    pub async fn is_noob(&self, uid: i32) -> HeResult<bool> {
        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        // Check if player participated in previous rounds
        let row = sqlx::query("SELECT COUNT(*) AS total FROM hist_users WHERE userID = $1 LIMIT 1")
            .bind(uid)
            .fetch_one(db)
            .await?;

        let previous_round: i64 = row.get("total");
        if previous_round > 0 {
            return Ok(false);
        }

        // Check experience points
        let row = sqlx::query("SELECT exp FROM users_stats WHERE uid = $1 LIMIT 1")
            .bind(uid)
            .fetch_one(db)
            .await?;

        let exp: i32 = row.get("exp");
        Ok(exp < 100)
    }

    /// Gets IP uptime for a user
    /// 
    /// # Arguments
    /// * `uid` - User ID
    /// 
    /// # Returns
    /// Formatted uptime string
    pub async fn ip_uptime(&self, uid: i32) -> HeResult<String> {
        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        let row = sqlx::query(
            "SELECT TIMESTAMPDIFF(SECOND, lastIpReset, NOW()) AS uptime FROM users_stats WHERE uid = $1 LIMIT 1"
        )
        .bind(uid)
        .fetch_one(db)
        .await?;

        let uptime_seconds: i32 = row.get("uptime");
        
        // Convert seconds to human-readable format
        let days = uptime_seconds / 86400;
        let hours = (uptime_seconds % 86400) / 3600;
        let minutes = (uptime_seconds % 3600) / 60;

        if days > 0 {
            if hours == 0 && minutes == 0 {
                return Ok(format!("{} day{}", days, if days == 1 { "" } else { "s" }));
            } else {
                return Ok(format!("{} day{} and {} hour{}", 
                    days, if days == 1 { "" } else { "s" },
                    hours, if hours == 1 { "" } else { "s" }
                ));
            }
        }

        if hours > 0 {
            Ok(format!("{} hour{}", hours, if hours == 1 { "" } else { "s" }))
        } else {
            Ok(format!("{} minute{}", minutes, if minutes == 1 { "" } else { "s" }))
        }
    }

    /// Gets IP reset information
    /// 
    /// # Arguments
    /// * `uid` - User ID
    /// 
    /// # Returns
    /// IpInfo with pricing and timing details
    pub async fn ip_info(&self, uid: i32) -> HeResult<IpInfo> {
        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        let row = sqlx::query(
            "SELECT ipResets, TIMESTAMPDIFF(SECOND, lastIpReset, NOW()) AS uptime FROM users_stats WHERE uid = $1 LIMIT 1"
        )
        .bind(uid)
        .fetch_one(db)
        .await?;

        let ip_resets: i32 = row.get("ipResets");
        let uptime: i32 = row.get("uptime");

        let study_price = self.ip_study_price(ip_resets, uptime).await?;
        
        // TODO: Calculate wait time based on study_price data
        let next_reset = if study_price.price > 0 {
            "TODO: Calculate wait time".to_string()
        } else {
            "0".to_string()
        };

        Ok(IpInfo {
            price: study_price.price,
            next_reset,
        })
    }

    /// Calculates IP reset pricing
    /// 
    /// # Arguments
    /// * `resets` - Number of previous resets
    /// * `uptime` - Current uptime in seconds
    /// 
    /// # Returns
    /// Pricing information
    pub async fn ip_study_price(&self, resets: i32, uptime: i32) -> HeResult<IpInfo> {
        if resets == 0 {
            return Ok(IpInfo {
                price: 0,
                next_reset: "0".to_string(),
            });
        }

        let time = match resets {
            1 => 3 * 3600,     // 3 hours
            2 => 6 * 3600,     // 6 hours
            3 => 12 * 3600,    // 12 hours
            4 => 24 * 3600,    // 24 hours
            5 => 48 * 3600,    // 48 hours
            6 => 96 * 3600,    // 96 hours
            _ => 168 * 3600,   // 168 hours (1 week)
        };

        if time <= uptime {
            return Ok(IpInfo {
                price: 0,
                next_reset: "0".to_string(),
            });
        }

        let price = 100.0 + 500.0 * resets as f64 + (resets as f64).powf(3.8) / 10.0;
        let price = if price > 100000.0 { 100000 } else { price as i32 };

        Ok(IpInfo {
            price,
            next_reset: time.to_string(),
        })
    }

    /// Gets password reset information
    /// 
    /// # Arguments
    /// * `uid` - User ID (optional, uses session if None)
    /// 
    /// # Returns
    /// PasswordInfo with pricing and timing details
    pub async fn pwd_info(&self) -> HeResult<PasswordInfo> {
        let db = self.db_pool.as_ref().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let uid = self.id.unwrap_or(0);
        
        let row = sqlx::query(
            "SELECT pwdResets, TIMESTAMPDIFF(SECOND, lastPwdReset, NOW()) AS lastReset FROM users_stats WHERE uid = $1 LIMIT 1"
        )
        .bind(uid)
        .fetch_one(db)
        .await?;

        let pwd_resets: i32 = row.get("pwdResets");
        let last_reset: i32 = row.get("lastReset");

        let study_price = self.pwd_study_price(pwd_resets, last_reset).await?;
        
        // TODO: Calculate wait time based on study_price data
        let next_reset = if study_price.price.is_some() && study_price.price.map_err(|e| anyhow::anyhow!("Error: {}", e))? > 0 {
            "TODO: Calculate wait time".to_string()
        } else {
            "0".to_string()
        };

        Ok(PasswordInfo {
            price: study_price.price,
            next_reset,
        })
    }

    /// Calculates password reset pricing
    /// 
    /// # Arguments
    /// * `pwd_resets` - Number of previous password resets
    /// * `last_reset` - Time since last reset in seconds
    /// 
    /// # Returns
    /// Pricing information
    pub async fn pwd_study_price(&self, pwd_resets: i32, last_reset: i32) -> HeResult<PasswordInfo> {
        if pwd_resets == 0 {
            return Ok(PasswordInfo {
                price: None,
                next_reset: "0".to_string(),
            });
        }

        let time = match pwd_resets {
            1 => 3 * 3600,     // 3 hours
            2 => 6 * 3600,     // 6 hours
            3 => 12 * 3600,    // 12 hours
            4 => 24 * 3600,    // 24 hours
            5 => 48 * 3600,    // 48 hours
            6 => 96 * 3600,    // 96 hours
            _ => 168 * 3600,   // 168 hours (1 week)
        };

        if time <= last_reset {
            return Ok(PasswordInfo {
                price: None,
                next_reset: "0".to_string(),
            });
        }

        let price = 50.0 + 10.0 * pwd_resets as f64 + (pwd_resets as f64).powf(3.2) / 10.0;
        
        Ok(PasswordInfo {
            price: Some(price as i32),
            next_reset: time.to_string(),
        })
    }

    /// Gets player profile picture path
    /// 
    /// # Arguments
    /// * `uid` - User ID
    /// * `username` - Username
    /// * `thumbnail` - Whether to return thumbnail version
    /// 
    /// # Returns
    /// Profile picture file path
    pub fn get_profile_pic(&self, uid: i32, username: &str, thumbnail: bool) -> String {
        if uid > 0 {
            let size = if thumbnail { "thumbnail/" } else { "" };
            let filename = format!("{}{}.jpg", md5::compute(format!("{}{}", username, uid)), "");
            let common_path = format!("images/profile/{}{}", size, filename);
            
            // TODO: Check if file exists
            // For now, return default path
            format!("images/profile/{}unsub.jpg", size)
        } else {
            match uid {
                0 => "images/profile/tux.png".to_string(),
                -1 => "images/profile/tux-ec.png".to_string(),
                -2 => "images/profile/tux-fbi.png".to_string(),
                -3 => "images/profile/tux-safenet.png".to_string(),
                -4 => "images/profile/tux-clan2.png".to_string(),
                -5 => "images/profile/tux-clan.png".to_string(),
                -6 => "images/profile/tux-social.png".to_string(),
                -7 => "images/profile/tux-badges.png".to_string(),
                _ => "images/profile/tux.png".to_string(),
            }
        }
    }

    // TODO: Implement display methods (showIndex, showGameOver, controlpanel_show, etc.)
    // These methods contain HTML rendering logic that would need to be handled
    // by the presentation layer in the Rust architecture
}