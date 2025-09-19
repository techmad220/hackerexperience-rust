use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::{UserId, ClanId, IpAddress, HeResult};

// Mapping from PHP Player.class.php
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub email: String,
    pub game_ip: IpAddress,
    pub cur_round: i32,
    pub clan_id: Option<ClanId>,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_premium: bool,
    pub is_online: bool,
    pub password_hash: String, // BCrypt hash
}

impl User {
    pub fn new(name: String, email: String, password_hash: String) -> Self {
        Self {
            id: 0, // Will be set by database
            name,
            email,
            game_ip: String::new(), // Will be generated
            cur_round: 1,
            clan_id: None,
            created_at: Utc::now(),
            last_login: None,
            is_premium: false,
            is_online: false,
            password_hash,
        }
    }
    
    pub fn is_npc(&self) -> bool {
        false // Users are never NPCs
    }
    
    pub fn get_display_name(&self) -> &str {
        &self.name
    }
    
    pub fn set_online(&mut self, online: bool) {
        self.is_online = online;
        if online {
            self.last_login = Some(Utc::now());
        }
    }
    
    pub fn join_clan(&mut self, clan_id: ClanId) {
        self.clan_id = Some(clan_id);
    }
    
    pub fn leave_clan(&mut self) {
        self.clan_id = None;
    }
}

// User statistics - separate from main User entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    pub user_id: UserId,
    pub reputation: i64,   // Called "power" in newer versions
    pub money: i64,
    pub experience: i64,
    pub total_hacks: i64,
    pub successful_hacks: i64,
    pub failed_hacks: i64,
    pub viruses_uploaded: i64,
    pub round_stats_id: Option<i64>,
}

impl UserStats {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            reputation: 0,
            money: 0,
            experience: 0,
            total_hacks: 0,
            successful_hacks: 0,
            failed_hacks: 0,
            viruses_uploaded: 0,
            round_stats_id: None,
        }
    }
    
    pub fn success_rate(&self) -> f64 {
        if self.total_hacks == 0 {
            0.0
        } else {
            self.successful_hacks as f64 / self.total_hacks as f64
        }
    }
    
    pub fn add_hack_attempt(&mut self, successful: bool) {
        self.total_hacks += 1;
        if successful {
            self.successful_hacks += 1;
        } else {
            self.failed_hacks += 1;
        }
    }
}

// User badge system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBadge {
    pub id: i64,
    pub user_id: UserId,
    pub badge_name: String,
    pub earned_at: DateTime<Utc>,
    pub description: Option<String>,
}

// User friends system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFriend {
    pub user_id: UserId,
    pub friend_id: UserId,
    pub created_at: DateTime<Utc>,
    pub status: FriendshipStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FriendshipStatus {
    Pending,
    Accepted,
    Blocked,
}

// Password reset info - matching PHP pwd_info() method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetInfo {
    pub price: Option<i64>,     // Cost in game money, None if free
    pub next_reset: i64,        // Unix timestamp when next reset is available
    pub resets_used: i32,       // Number of resets used this round
}

impl PasswordResetInfo {
    pub fn can_reset_now(&self) -> bool {
        let now = Utc::now().timestamp();
        now >= self.next_reset
    }
    
    pub fn is_free_reset(&self) -> bool {
        self.price.is_none()
    }
}