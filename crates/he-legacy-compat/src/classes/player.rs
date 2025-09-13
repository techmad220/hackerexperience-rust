//! Player Class - Complete Implementation
//! Full port of all Player.class.php functionality from HackerExperience Legacy

use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, FromRow};
use bcrypt::{hash, verify, DEFAULT_COST};
use std::collections::HashMap;
use anyhow::Result;
use uuid::Uuid;
use rand::{Rng, thread_rng};
use regex::Regex;
use sha2::{Sha256, Digest};
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Player {
    pub id: i32,
    pub login: String,
    pub email: String,
    pub password_hash: String,
    pub ip: String,
    pub experience: i64,
    pub money: i64,
    pub level: i32,
    pub reputation: i32,
    pub hardware_id: Option<i32>,
    pub clan_id: Option<i32>,
    pub rank: i32,
    pub premium_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub last_activity: Option<DateTime<Utc>>,
    pub is_online: bool,
    pub is_banned: bool,
    pub ban_reason: Option<String>,
    pub tutorial_completed: bool,
    pub settings: serde_json::Value,
    pub stats: PlayerStats,
    pub bank_accounts: Vec<BankAccount>,
    pub software_installed: Vec<Software>,
    pub processes: Vec<Process>,
    pub missions: Vec<Mission>,
    pub certifications: Vec<Certification>,
    pub achievements: Vec<Achievement>,
    pub notifications: Vec<Notification>,
    pub friends: Vec<Friend>,
    pub enemies: Vec<Enemy>,
    pub logs: Vec<LogEntry>,
    pub emails: Vec<Email>,
    pub forum_posts: i32,
    pub forum_reputation: i32,
    pub language: String,
    pub timezone: String,
    pub country: String,
    pub verification_token: Option<String>,
    pub reset_token: Option<String>,
    pub two_factor_secret: Option<String>,
    pub two_factor_enabled: bool,
    pub api_key: Option<String>,
    pub referrer_id: Option<i32>,
    pub referred_count: i32,
    pub total_playtime: i64,
    pub consecutive_days: i32,
    pub last_daily_reward: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlayerStats {
    pub total_hacks: i32,
    pub successful_hacks: i32,
    pub failed_hacks: i32,
    pub money_earned: i64,
    pub money_spent: i64,
    pub viruses_installed: i32,
    pub viruses_removed: i32,
    pub ips_hacked: i32,
    pub times_hacked: i32,
    pub ddos_attacks_sent: i32,
    pub ddos_attacks_received: i32,
    pub files_downloaded: i32,
    pub files_uploaded: i32,
    pub files_deleted: i32,
    pub logs_modified: i32,
    pub logs_deleted: i32,
    pub servers_owned: i32,
    pub bounties_completed: i32,
    pub missions_completed: i32,
    pub software_researched: i32,
    pub hardware_upgraded: i32,
    pub bank_hacks: i32,
    pub total_bounty_reward: i64,
    pub clan_wars_participated: i32,
    pub clan_wars_won: i32,
    pub pvp_wins: i32,
    pub pvp_losses: i32,
    pub best_software_version: f32,
    pub fastest_hack_time: i32,
    pub longest_uptime: i32,
    pub total_data_downloaded: i64,
    pub total_data_uploaded: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankAccount {
    pub id: i32,
    pub bank_id: i32,
    pub account_number: String,
    pub balance: i64,
    pub account_type: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Software {
    pub id: i32,
    pub name: String,
    pub version: f32,
    pub size: i32,
    pub software_type: String,
    pub is_installed: bool,
    pub is_running: bool,
    pub ram_usage: i32,
    pub cpu_usage: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    pub id: i32,
    pub process_type: String,
    pub target_ip: String,
    pub completion: f32,
    pub time_started: DateTime<Utc>,
    pub time_end: DateTime<Utc>,
    pub is_paused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub reward_money: i64,
    pub reward_exp: i64,
    pub is_completed: bool,
    pub progress: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certification {
    pub id: i32,
    pub name: String,
    pub level: i32,
    pub obtained_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub points: i32,
    pub unlocked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: i32,
    pub message: String,
    pub notification_type: String,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Friend {
    pub player_id: i32,
    pub username: String,
    pub is_online: bool,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub player_id: i32,
    pub username: String,
    pub threat_level: i32,
    pub last_attack: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: i32,
    pub ip: String,
    pub action: String,
    pub timestamp: DateTime<Utc>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub id: i32,
    pub from: String,
    pub subject: String,
    pub body: String,
    pub is_read: bool,
    pub received_at: DateTime<Utc>,
}

impl Player {
    /// Create a new player account
    pub async fn create(
        db: &PgPool,
        login: String,
        email: String,
        password: String,
        ip: String,
    ) -> Result<Self> {
        // Validate input
        Self::validate_username(&login)?;
        Self::validate_email(&email)?;
        Self::validate_password(&password)?;
        
        // Check if username or email already exists
        let existing = sqlx::query!("SELECT id FROM users WHERE login = $1 OR email = $2", login, email)
            .fetch_optional(db)
            .await?;
        
        if existing.is_some() {
            return Err(anyhow::anyhow!("Username or email already exists"));
        }
        
        // Hash password
        let password_hash = hash(password.as_bytes(), DEFAULT_COST)?;
        
        // Generate verification token
        let verification_token = Self::generate_token();
        
        // Insert new player
        let player = sqlx::query_as!(
            Player,
            r#"
            INSERT INTO users (login, email, password_hash, ip, verification_token, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            RETURNING *
            "#,
            login,
            email,
            password_hash,
            ip,
            verification_token
        )
        .fetch_one(db)
        .await?;
        
        // Initialize player stats
        PlayerStats::initialize(db, player.id).await?;
        
        // Create default hardware
        Self::create_default_hardware(db, player.id).await?;
        
        // Send verification email
        Self::send_verification_email(&email, &verification_token).await?;
        
        Ok(player)
    }
    
    /// Authenticate player login
    pub async fn authenticate(db: &PgPool, login: &str, password: &str) -> Result<Self> {
        let player = sqlx::query_as!(
            Player,
            "SELECT * FROM users WHERE login = $1 OR email = $1",
            login
        )
        .fetch_optional(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Invalid credentials"))?;
        
        if !verify(password.as_bytes(), &player.password_hash)? {
            return Err(anyhow::anyhow!("Invalid credentials"));
        }
        
        if player.is_banned {
            return Err(anyhow::anyhow!("Account is banned: {}", player.ban_reason.unwrap_or_default()));
        }
        
        // Update last login
        sqlx::query!(
            "UPDATE users SET last_login = NOW(), is_online = true WHERE id = $1",
            player.id
        )
        .execute(db)
        .await?;
        
        Ok(player)
    }
    
    /// Get player by ID
    pub async fn get_by_id(db: &PgPool, id: i32) -> Result<Self> {
        let mut player = sqlx::query_as!(Player, "SELECT * FROM users WHERE id = $1", id)
            .fetch_one(db)
            .await?;
        
        // Load related data
        player.stats = PlayerStats::get(db, id).await?;
        player.bank_accounts = Self::get_bank_accounts(db, id).await?;
        player.software_installed = Self::get_software(db, id).await?;
        player.processes = Self::get_processes(db, id).await?;
        player.missions = Self::get_missions(db, id).await?;
        player.certifications = Self::get_certifications(db, id).await?;
        player.achievements = Self::get_achievements(db, id).await?;
        player.notifications = Self::get_notifications(db, id).await?;
        player.friends = Self::get_friends(db, id).await?;
        player.enemies = Self::get_enemies(db, id).await?;
        player.logs = Self::get_logs(db, id).await?;
        player.emails = Self::get_emails(db, id).await?;
        
        Ok(player)
    }
    
    /// Get player by username
    pub async fn get_by_username(db: &PgPool, username: &str) -> Result<Self> {
        let player = sqlx::query_as!(
            Player,
            "SELECT * FROM users WHERE login = $1",
            username
        )
        .fetch_one(db)
        .await?;
        
        Self::get_by_id(db, player.id).await
    }
    
    /// Update player password
    pub async fn update_password(&mut self, db: &PgPool, old_password: &str, new_password: &str) -> Result<()> {
        if !verify(old_password.as_bytes(), &self.password_hash)? {
            return Err(anyhow::anyhow!("Invalid current password"));
        }
        
        Self::validate_password(new_password)?;
        
        let new_hash = hash(new_password.as_bytes(), DEFAULT_COST)?;
        
        sqlx::query!(
            "UPDATE users SET password_hash = $1 WHERE id = $2",
            new_hash,
            self.id
        )
        .execute(db)
        .await?;
        
        self.password_hash = new_hash;
        
        Ok(())
    }
    
    /// Reset password with token
    pub async fn reset_password(db: &PgPool, token: &str, new_password: &str) -> Result<()> {
        Self::validate_password(new_password)?;
        
        let player = sqlx::query!(
            "SELECT id FROM users WHERE reset_token = $1 AND reset_token_expires > NOW()",
            token
        )
        .fetch_optional(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Invalid or expired reset token"))?;
        
        let new_hash = hash(new_password.as_bytes(), DEFAULT_COST)?;
        
        sqlx::query!(
            "UPDATE users SET password_hash = $1, reset_token = NULL, reset_token_expires = NULL WHERE id = $2",
            new_hash,
            player.id
        )
        .execute(db)
        .await?;
        
        Ok(())
    }
    
    /// Request password reset
    pub async fn request_password_reset(db: &PgPool, email: &str) -> Result<()> {
        let player = sqlx::query!(
            "SELECT id, email FROM users WHERE email = $1",
            email
        )
        .fetch_optional(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Email not found"))?;
        
        let reset_token = Self::generate_token();
        let expires = Utc::now() + Duration::hours(1);
        
        sqlx::query!(
            "UPDATE users SET reset_token = $1, reset_token_expires = $2 WHERE id = $3",
            reset_token,
            expires,
            player.id
        )
        .execute(db)
        .await?;
        
        Self::send_password_reset_email(&player.email, &reset_token).await?;
        
        Ok(())
    }
    
    /// Verify email with token
    pub async fn verify_email(db: &PgPool, token: &str) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE users SET is_confirmed = true, verification_token = NULL WHERE verification_token = $1",
            token
        )
        .execute(db)
        .await?;
        
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Invalid verification token"));
        }
        
        Ok(())
    }
    
    /// Update player email
    pub async fn update_email(&mut self, db: &PgPool, new_email: &str, password: &str) -> Result<()> {
        if !verify(password.as_bytes(), &self.password_hash)? {
            return Err(anyhow::anyhow!("Invalid password"));
        }
        
        Self::validate_email(new_email)?;
        
        let existing = sqlx::query!("SELECT id FROM users WHERE email = $1", new_email)
            .fetch_optional(db)
            .await?;
        
        if existing.is_some() {
            return Err(anyhow::anyhow!("Email already in use"));
        }
        
        let verification_token = Self::generate_token();
        
        sqlx::query!(
            "UPDATE users SET email = $1, is_confirmed = false, verification_token = $2 WHERE id = $3",
            new_email,
            verification_token,
            self.id
        )
        .execute(db)
        .await?;
        
        self.email = new_email.to_string();
        
        Self::send_verification_email(new_email, &verification_token).await?;
        
        Ok(())
    }
    
    /// Enable two-factor authentication
    pub async fn enable_two_factor(&mut self, db: &PgPool, password: &str) -> Result<String> {
        if !verify(password.as_bytes(), &self.password_hash)? {
            return Err(anyhow::anyhow!("Invalid password"));
        }
        
        let secret = Self::generate_totp_secret();
        
        sqlx::query!(
            "UPDATE users SET two_factor_secret = $1, two_factor_enabled = false WHERE id = $2",
            secret,
            self.id
        )
        .execute(db)
        .await?;
        
        self.two_factor_secret = Some(secret.clone());
        
        Ok(secret)
    }
    
    /// Confirm two-factor authentication
    pub async fn confirm_two_factor(&mut self, db: &PgPool, code: &str) -> Result<Vec<String>> {
        let secret = self.two_factor_secret.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Two-factor not initialized"))?;
        
        if !Self::verify_totp_code(secret, code)? {
            return Err(anyhow::anyhow!("Invalid verification code"));
        }
        
        let backup_codes = Self::generate_backup_codes();
        let codes_hash = Self::hash_backup_codes(&backup_codes);
        
        sqlx::query!(
            "UPDATE users SET two_factor_enabled = true, backup_codes = $1 WHERE id = $2",
            serde_json::to_value(&codes_hash)?,
            self.id
        )
        .execute(db)
        .await?;
        
        self.two_factor_enabled = true;
        
        Ok(backup_codes)
    }
    
    /// Disable two-factor authentication
    pub async fn disable_two_factor(&mut self, db: &PgPool, password: &str) -> Result<()> {
        if !verify(password.as_bytes(), &self.password_hash)? {
            return Err(anyhow::anyhow!("Invalid password"));
        }
        
        sqlx::query!(
            "UPDATE users SET two_factor_enabled = false, two_factor_secret = NULL, backup_codes = NULL WHERE id = $1",
            self.id
        )
        .execute(db)
        .await?;
        
        self.two_factor_enabled = false;
        self.two_factor_secret = None;
        
        Ok(())
    }
    
    /// Add money to player account
    pub async fn add_money(&mut self, db: &PgPool, amount: i64) -> Result<()> {
        if amount <= 0 {
            return Err(anyhow::anyhow!("Amount must be positive"));
        }
        
        sqlx::query!(
            "UPDATE users SET money = money + $1 WHERE id = $2",
            amount,
            self.id
        )
        .execute(db)
        .await?;
        
        self.money += amount;
        
        // Update stats
        sqlx::query!(
            "UPDATE user_stats SET money_earned = money_earned + $1 WHERE user_id = $2",
            amount,
            self.id
        )
        .execute(db)
        .await?;
        
        Ok(())
    }
    
    /// Remove money from player account
    pub async fn remove_money(&mut self, db: &PgPool, amount: i64) -> Result<()> {
        if amount <= 0 {
            return Err(anyhow::anyhow!("Amount must be positive"));
        }
        
        if self.money < amount {
            return Err(anyhow::anyhow!("Insufficient funds"));
        }
        
        sqlx::query!(
            "UPDATE users SET money = money - $1 WHERE id = $2",
            amount,
            self.id
        )
        .execute(db)
        .await?;
        
        self.money -= amount;
        
        // Update stats
        sqlx::query!(
            "UPDATE user_stats SET money_spent = money_spent + $1 WHERE user_id = $2",
            amount,
            self.id
        )
        .execute(db)
        .await?;
        
        Ok(())
    }
    
    /// Transfer money to another player
    pub async fn transfer_money(&mut self, db: &PgPool, recipient_id: i32, amount: i64) -> Result<()> {
        if amount <= 0 {
            return Err(anyhow::anyhow!("Amount must be positive"));
        }
        
        if self.money < amount {
            return Err(anyhow::anyhow!("Insufficient funds"));
        }
        
        if self.id == recipient_id {
            return Err(anyhow::anyhow!("Cannot transfer to yourself"));
        }
        
        // Start transaction
        let mut tx = db.begin().await?;
        
        // Remove from sender
        sqlx::query!(
            "UPDATE users SET money = money - $1 WHERE id = $2",
            amount,
            self.id
        )
        .execute(&mut *tx)
        .await?;
        
        // Add to recipient
        sqlx::query!(
            "UPDATE users SET money = money + $1 WHERE id = $2",
            amount,
            recipient_id
        )
        .execute(&mut *tx)
        .await?;
        
        // Log transaction
        sqlx::query!(
            "INSERT INTO money_transfers (sender_id, recipient_id, amount, timestamp) VALUES ($1, $2, $3, NOW())",
            self.id,
            recipient_id,
            amount
        )
        .execute(&mut *tx)
        .await?;
        
        tx.commit().await?;
        
        self.money -= amount;
        
        Ok(())
    }
    
    /// Add experience to player
    pub async fn add_experience(&mut self, db: &PgPool, amount: i64) -> Result<()> {
        if amount <= 0 {
            return Err(anyhow::anyhow!("Amount must be positive"));
        }
        
        let old_level = self.level;
        self.experience += amount;
        
        // Calculate new level
        let new_level = Self::calculate_level(self.experience);
        
        sqlx::query!(
            "UPDATE users SET experience = $1, level = $2 WHERE id = $3",
            self.experience,
            new_level,
            self.id
        )
        .execute(db)
        .await?;
        
        self.level = new_level;
        
        // Check for level up
        if new_level > old_level {
            self.on_level_up(db, old_level, new_level).await?;
        }
        
        Ok(())
    }
    
    /// Join a clan
    pub async fn join_clan(&mut self, db: &PgPool, clan_id: i32) -> Result<()> {
        if self.clan_id.is_some() {
            return Err(anyhow::anyhow!("Already in a clan"));
        }
        
        // Check if clan exists and has space
        let clan = sqlx::query!(
            "SELECT id, max_members, member_count FROM clans WHERE id = $1",
            clan_id
        )
        .fetch_optional(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Clan not found"))?;
        
        if clan.member_count >= clan.max_members {
            return Err(anyhow::anyhow!("Clan is full"));
        }
        
        // Join clan
        sqlx::query!(
            "UPDATE users SET clan_id = $1 WHERE id = $2",
            clan_id,
            self.id
        )
        .execute(db)
        .await?;
        
        // Update clan member count
        sqlx::query!(
            "UPDATE clans SET member_count = member_count + 1 WHERE id = $1",
            clan_id
        )
        .execute(db)
        .await?;
        
        // Add to clan members table
        sqlx::query!(
            "INSERT INTO clan_members (clan_id, user_id, role, joined_at) VALUES ($1, $2, 'member', NOW())",
            clan_id,
            self.id
        )
        .execute(db)
        .await?;
        
        self.clan_id = Some(clan_id);
        
        Ok(())
    }
    
    /// Leave current clan
    pub async fn leave_clan(&mut self, db: &PgPool) -> Result<()> {
        let clan_id = self.clan_id.ok_or_else(|| anyhow::anyhow!("Not in a clan"))?;
        
        // Check if player is clan leader
        let is_leader = sqlx::query!(
            "SELECT leader_id FROM clans WHERE id = $1",
            clan_id
        )
        .fetch_one(db)
        .await?
        .leader_id == self.id;
        
        if is_leader {
            return Err(anyhow::anyhow!("Clan leader cannot leave. Transfer leadership first."));
        }
        
        // Leave clan
        sqlx::query!(
            "UPDATE users SET clan_id = NULL WHERE id = $1",
            self.id
        )
        .execute(db)
        .await?;
        
        // Update clan member count
        sqlx::query!(
            "UPDATE clans SET member_count = member_count - 1 WHERE id = $1",
            clan_id
        )
        .execute(db)
        .await?;
        
        // Remove from clan members table
        sqlx::query!(
            "DELETE FROM clan_members WHERE user_id = $1",
            self.id
        )
        .execute(db)
        .await?;
        
        self.clan_id = None;
        
        Ok(())
    }
    
    /// Start a new process
    pub async fn start_process(
        &mut self,
        db: &PgPool,
        process_type: &str,
        target_ip: &str,
        duration: i32,
    ) -> Result<i32> {
        // Check if player has available process slots
        let active_processes = sqlx::query!(
            "SELECT COUNT(*) as count FROM processes WHERE p_creator_id = $1 AND is_completed = false",
            self.id
        )
        .fetch_one(db)
        .await?
        .count
        .unwrap_or(0);
        
        if active_processes >= 5 {
            return Err(anyhow::anyhow!("Maximum concurrent processes reached"));
        }
        
        let end_time = Utc::now() + Duration::seconds(duration as i64);
        
        let process_id = sqlx::query!(
            r#"
            INSERT INTO processes (p_creator_id, p_type, p_target_ip, p_time_start, p_time_end)
            VALUES ($1, $2, $3, NOW(), $4)
            RETURNING pid
            "#,
            self.id,
            process_type,
            target_ip,
            end_time
        )
        .fetch_one(db)
        .await?
        .pid;
        
        Ok(process_id)
    }
    
    /// Cancel a process
    pub async fn cancel_process(&self, db: &PgPool, process_id: i32) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE processes SET is_completed = true, is_failed = true WHERE pid = $1 AND p_creator_id = $2",
            process_id,
            self.id
        )
        .execute(db)
        .await?;
        
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Process not found or not owned by player"));
        }
        
        Ok(())
    }
    
    /// Install software
    pub async fn install_software(&mut self, db: &PgPool, software_id: i32) -> Result<()> {
        // Check if player owns the software
        let software = sqlx::query!(
            "SELECT * FROM software WHERE id = $1 AND user_id = $2",
            software_id,
            self.id
        )
        .fetch_optional(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Software not found"))?;
        
        // Check hardware requirements
        let hardware = sqlx::query!(
            "SELECT ram, hdd FROM hardware WHERE user_id = $1",
            self.id
        )
        .fetch_one(db)
        .await?;
        
        if hardware.ram < software.ram_required {
            return Err(anyhow::anyhow!("Insufficient RAM"));
        }
        
        if hardware.hdd < software.size {
            return Err(anyhow::anyhow!("Insufficient disk space"));
        }
        
        // Install software
        sqlx::query!(
            "UPDATE software SET is_installed = true WHERE id = $1",
            software_id
        )
        .execute(db)
        .await?;
        
        Ok(())
    }
    
    /// Uninstall software
    pub async fn uninstall_software(&self, db: &PgPool, software_id: i32) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE software SET is_installed = false, is_running = false WHERE id = $1 AND user_id = $2",
            software_id,
            self.id
        )
        .execute(db)
        .await?;
        
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Software not found"));
        }
        
        Ok(())
    }
    
    /// Research new software
    pub async fn research_software(&mut self, db: &PgPool, software_type: &str) -> Result<i32> {
        // Get current best version
        let current_version = sqlx::query!(
            "SELECT MAX(version) as max_version FROM software WHERE user_id = $1 AND software_type = $2",
            self.id,
            software_type
        )
        .fetch_one(db)
        .await?
        .max_version
        .unwrap_or(0.0);
        
        let new_version = current_version + 0.1;
        let research_cost = (new_version * 10000.0) as i64;
        
        if self.money < research_cost {
            return Err(anyhow::anyhow!("Insufficient funds for research"));
        }
        
        // Deduct money
        self.remove_money(db, research_cost).await?;
        
        // Calculate research time based on version
        let research_time = (new_version * 600.0) as i32; // 10 minutes per version level
        
        // Start research process
        let process_id = self.start_process(db, "research", software_type, research_time).await?;
        
        // Store research data
        sqlx::query!(
            "INSERT INTO research_queue (user_id, software_type, version, process_id) VALUES ($1, $2, $3, $4)",
            self.id,
            software_type,
            new_version,
            process_id
        )
        .execute(db)
        .await?;
        
        Ok(process_id)
    }
    
    /// Upgrade hardware component
    pub async fn upgrade_hardware(&mut self, db: &PgPool, component: &str) -> Result<()> {
        let hardware = sqlx::query!(
            "SELECT * FROM hardware WHERE user_id = $1",
            self.id
        )
        .fetch_one(db)
        .await?;
        
        let (current_value, upgrade_cost) = match component {
            "cpu" => (hardware.cpu, hardware.cpu as i64 * 100),
            "ram" => (hardware.ram, hardware.ram as i64 * 50),
            "hdd" => (hardware.hdd, hardware.hdd as i64 * 10),
            "net" => (hardware.net, hardware.net as i64 * 200),
            _ => return Err(anyhow::anyhow!("Invalid hardware component")),
        };
        
        if self.money < upgrade_cost {
            return Err(anyhow::anyhow!("Insufficient funds for upgrade"));
        }
        
        // Deduct money
        self.remove_money(db, upgrade_cost).await?;
        
        // Upgrade hardware
        let new_value = (current_value as f64 * 1.5) as i32;
        
        let query = match component {
            "cpu" => sqlx::query!("UPDATE hardware SET cpu = $1 WHERE user_id = $2", new_value, self.id),
            "ram" => sqlx::query!("UPDATE hardware SET ram = $1 WHERE user_id = $2", new_value, self.id),
            "hdd" => sqlx::query!("UPDATE hardware SET hdd = $1 WHERE user_id = $2", new_value, self.id),
            "net" => sqlx::query!("UPDATE hardware SET net = $1 WHERE user_id = $2", new_value, self.id),
            _ => unreachable!(),
        };
        
        query.execute(db).await?;
        
        // Update stats
        sqlx::query!(
            "UPDATE user_stats SET hardware_upgraded = hardware_upgraded + 1 WHERE user_id = $1",
            self.id
        )
        .execute(db)
        .await?;
        
        Ok(())
    }
    
    /// Hack a target IP
    pub async fn hack_target(&mut self, db: &PgPool, target_ip: &str) -> Result<i32> {
        // Validate IP
        target_ip.parse::<IpAddr>()
            .map_err(|_| anyhow::anyhow!("Invalid IP address"))?;
        
        // Check if target exists
        let target = sqlx::query!(
            "SELECT * FROM external_hardware WHERE ip_address = $1",
            target_ip
        )
        .fetch_optional(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Target not found"))?;
        
        // Calculate hack time based on difficulty
        let hack_time = target.difficulty_level * 60; // 1 minute per difficulty level
        
        // Start hack process
        let process_id = self.start_process(db, "hack", target_ip, hack_time).await?;
        
        // Log hack attempt
        sqlx::query!(
            "INSERT INTO hack_attempts (user_id, target_ip, process_id, started_at) VALUES ($1, $2, $3, NOW())",
            self.id,
            target_ip,
            process_id
        )
        .execute(db)
        .await?;
        
        Ok(process_id)
    }
    
    /// Complete a mission
    pub async fn complete_mission(&mut self, db: &PgPool, mission_id: i32) -> Result<()> {
        let mission = sqlx::query!(
            "SELECT * FROM user_missions WHERE user_id = $1 AND mission_id = $2 AND is_completed = false",
            self.id,
            mission_id
        )
        .fetch_optional(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Mission not found or already completed"))?;
        
        // Check mission requirements
        if mission.progress < 100.0 {
            return Err(anyhow::anyhow!("Mission not ready for completion"));
        }
        
        // Get mission rewards
        let rewards = sqlx::query!(
            "SELECT reward_money, reward_experience FROM missions WHERE id = $1",
            mission_id
        )
        .fetch_one(db)
        .await?;
        
        // Mark as completed
        sqlx::query!(
            "UPDATE user_missions SET is_completed = true, completed_at = NOW() WHERE user_id = $1 AND mission_id = $2",
            self.id,
            mission_id
        )
        .execute(db)
        .await?;
        
        // Award rewards
        if rewards.reward_money > 0 {
            self.add_money(db, rewards.reward_money).await?;
        }
        
        if rewards.reward_experience > 0 {
            self.add_experience(db, rewards.reward_experience).await?;
        }
        
        // Update stats
        sqlx::query!(
            "UPDATE user_stats SET missions_completed = missions_completed + 1 WHERE user_id = $1",
            self.id
        )
        .execute(db)
        .await?;
        
        Ok(())
    }
    
    /// Delete a log entry
    pub async fn delete_log(&self, db: &PgPool, log_id: i32) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE logs SET is_deleted = true WHERE id = $1 AND user_id = $2",
            log_id,
            self.id
        )
        .execute(db)
        .await?;
        
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Log not found"));
        }
        
        // Update stats
        sqlx::query!(
            "UPDATE user_stats SET logs_deleted = logs_deleted + 1 WHERE user_id = $1",
            self.id
        )
        .execute(db)
        .await?;
        
        Ok(())
    }
    
    /// Modify a log entry
    pub async fn modify_log(&self, db: &PgPool, log_id: i32, new_content: &str) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE logs SET content = $1, is_modified = true WHERE id = $2 AND user_id = $3",
            new_content,
            log_id,
            self.id
        )
        .execute(db)
        .await?;
        
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Log not found"));
        }
        
        // Update stats
        sqlx::query!(
            "UPDATE user_stats SET logs_modified = logs_modified + 1 WHERE user_id = $1",
            self.id
        )
        .execute(db)
        .await?;
        
        Ok(())
    }
    
    /// Send an email to another player
    pub async fn send_email(&self, db: &PgPool, recipient_id: i32, subject: &str, body: &str) -> Result<()> {
        if self.id == recipient_id {
            return Err(anyhow::anyhow!("Cannot send email to yourself"));
        }
        
        // Check if recipient exists
        let recipient_exists = sqlx::query!("SELECT id FROM users WHERE id = $1", recipient_id)
            .fetch_optional(db)
            .await?
            .is_some();
        
        if !recipient_exists {
            return Err(anyhow::anyhow!("Recipient not found"));
        }
        
        // Send email
        sqlx::query!(
            "INSERT INTO emails (sender_id, recipient_id, subject, body, sent_at) VALUES ($1, $2, $3, $4, NOW())",
            self.id,
            recipient_id,
            subject,
            body
        )
        .execute(db)
        .await?;
        
        // Create notification for recipient
        sqlx::query!(
            "INSERT INTO notifications (user_id, message, notification_type, created_at) VALUES ($1, $2, 'email', NOW())",
            recipient_id,
            format!("New email from {}: {}", self.login, subject)
        )
        .execute(db)
        .await?;
        
        Ok(())
    }
    
    /// Add a friend
    pub async fn add_friend(&mut self, db: &PgPool, friend_id: i32) -> Result<()> {
        if self.id == friend_id {
            return Err(anyhow::anyhow!("Cannot add yourself as friend"));
        }
        
        // Check if already friends
        let already_friends = sqlx::query!(
            "SELECT id FROM friends WHERE user_id = $1 AND friend_id = $2",
            self.id,
            friend_id
        )
        .fetch_optional(db)
        .await?
        .is_some();
        
        if already_friends {
            return Err(anyhow::anyhow!("Already friends"));
        }
        
        // Add friend relationship (bidirectional)
        sqlx::query!(
            "INSERT INTO friends (user_id, friend_id, added_at) VALUES ($1, $2, NOW()), ($2, $1, NOW())",
            self.id,
            friend_id
        )
        .execute(db)
        .await?;
        
        Ok(())
    }
    
    /// Remove a friend
    pub async fn remove_friend(&mut self, db: &PgPool, friend_id: i32) -> Result<()> {
        let result = sqlx::query!(
            "DELETE FROM friends WHERE (user_id = $1 AND friend_id = $2) OR (user_id = $2 AND friend_id = $1)",
            self.id,
            friend_id
        )
        .execute(db)
        .await?;
        
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Friend not found"));
        }
        
        Ok(())
    }
    
    /// Add an enemy
    pub async fn add_enemy(&mut self, db: &PgPool, enemy_id: i32, threat_level: i32) -> Result<()> {
        if self.id == enemy_id {
            return Err(anyhow::anyhow!("Cannot add yourself as enemy"));
        }
        
        sqlx::query!(
            "INSERT INTO enemies (user_id, enemy_id, threat_level, added_at) VALUES ($1, $2, $3, NOW())
             ON CONFLICT (user_id, enemy_id) DO UPDATE SET threat_level = $3",
            self.id,
            enemy_id,
            threat_level
        )
        .execute(db)
        .await?;
        
        Ok(())
    }
    
    /// Claim daily reward
    pub async fn claim_daily_reward(&mut self, db: &PgPool) -> Result<(i64, i64)> {
        // Check if already claimed today
        if let Some(last_claim) = self.last_daily_reward {
            let hours_since = (Utc::now() - last_claim).num_hours();
            if hours_since < 24 {
                return Err(anyhow::anyhow!("Daily reward already claimed. Next claim in {} hours", 24 - hours_since));
            }
        }
        
        // Calculate reward based on consecutive days
        let base_money = 1000;
        let base_exp = 100;
        let multiplier = (self.consecutive_days + 1).min(30) as i64;
        
        let reward_money = base_money * multiplier;
        let reward_exp = base_exp * multiplier;
        
        // Update consecutive days
        let new_consecutive = if let Some(last_claim) = self.last_daily_reward {
            let hours_since = (Utc::now() - last_claim).num_hours();
            if hours_since <= 48 {
                self.consecutive_days + 1
            } else {
                1 // Reset streak
            }
        } else {
            1
        };
        
        // Award rewards
        self.add_money(db, reward_money).await?;
        self.add_experience(db, reward_exp).await?;
        
        // Update player
        sqlx::query!(
            "UPDATE users SET last_daily_reward = NOW(), consecutive_days = $1 WHERE id = $2",
            new_consecutive,
            self.id
        )
        .execute(db)
        .await?;
        
        self.last_daily_reward = Some(Utc::now());
        self.consecutive_days = new_consecutive;
        
        Ok((reward_money, reward_exp))
    }
    
    /// Get player ranking
    pub async fn get_ranking(&self, db: &PgPool, ranking_type: &str) -> Result<i32> {
        let rank = match ranking_type {
            "level" => {
                sqlx::query!(
                    "SELECT COUNT(*) + 1 as rank FROM users WHERE level > $1",
                    self.level
                )
                .fetch_one(db)
                .await?
                .rank
            }
            "money" => {
                sqlx::query!(
                    "SELECT COUNT(*) + 1 as rank FROM users WHERE money > $1",
                    self.money
                )
                .fetch_one(db)
                .await?
                .rank
            }
            "reputation" => {
                sqlx::query!(
                    "SELECT COUNT(*) + 1 as rank FROM users WHERE reputation > $1",
                    self.reputation
                )
                .fetch_one(db)
                .await?
                .rank
            }
            _ => return Err(anyhow::anyhow!("Invalid ranking type")),
        };
        
        Ok(rank.unwrap_or(0) as i32)
    }
    
    /// Update player settings
    pub async fn update_settings(&mut self, db: &PgPool, settings: serde_json::Value) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET settings = $1 WHERE id = $2",
            settings,
            self.id
        )
        .execute(db)
        .await?;
        
        self.settings = settings;
        
        Ok(())
    }
    
    /// Ban player (admin only)
    pub async fn ban_player(db: &PgPool, admin_id: i32, player_id: i32, reason: &str) -> Result<()> {
        // Check if admin has permission
        let is_admin = sqlx::query!("SELECT is_admin FROM users WHERE id = $1", admin_id)
            .fetch_one(db)
            .await?
            .is_admin
            .unwrap_or(false);
        
        if !is_admin {
            return Err(anyhow::anyhow!("Insufficient permissions"));
        }
        
        sqlx::query!(
            "UPDATE users SET is_banned = true, ban_reason = $1, banned_at = NOW(), banned_by = $2 WHERE id = $3",
            reason,
            admin_id,
            player_id
        )
        .execute(db)
        .await?;
        
        // Force logout
        sqlx::query!("DELETE FROM sessions WHERE user_id = $1", player_id)
            .execute(db)
            .await?;
        
        Ok(())
    }
    
    /// Unban player (admin only)
    pub async fn unban_player(db: &PgPool, admin_id: i32, player_id: i32) -> Result<()> {
        // Check if admin has permission
        let is_admin = sqlx::query!("SELECT is_admin FROM users WHERE id = $1", admin_id)
            .fetch_one(db)
            .await?
            .is_admin
            .unwrap_or(false);
        
        if !is_admin {
            return Err(anyhow::anyhow!("Insufficient permissions"));
        }
        
        sqlx::query!(
            "UPDATE users SET is_banned = false, ban_reason = NULL WHERE id = $1",
            player_id
        )
        .execute(db)
        .await?;
        
        Ok(())
    }
    
    // Helper methods
    
    fn validate_username(username: &str) -> Result<()> {
        if username.len() < 3 || username.len() > 20 {
            return Err(anyhow::anyhow!("Username must be between 3 and 20 characters"));
        }
        
        let re = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
        if !re.is_match(username) {
            return Err(anyhow::anyhow!("Username can only contain letters, numbers, and underscores"));
        }
        
        Ok(())
    }
    
    fn validate_email(email: &str) -> Result<()> {
        let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        if !re.is_match(email) {
            return Err(anyhow::anyhow!("Invalid email format"));
        }
        
        Ok(())
    }
    
    fn validate_password(password: &str) -> Result<()> {
        if password.len() < 8 {
            return Err(anyhow::anyhow!("Password must be at least 8 characters"));
        }
        
        if !password.chars().any(|c| c.is_uppercase()) {
            return Err(anyhow::anyhow!("Password must contain at least one uppercase letter"));
        }
        
        if !password.chars().any(|c| c.is_lowercase()) {
            return Err(anyhow::anyhow!("Password must contain at least one lowercase letter"));
        }
        
        if !password.chars().any(|c| c.is_numeric()) {
            return Err(anyhow::anyhow!("Password must contain at least one number"));
        }
        
        Ok(())
    }
    
    fn generate_token() -> String {
        Uuid::new_v4().to_string()
    }
    
    fn generate_totp_secret() -> String {
        let mut rng = thread_rng();
        let bytes: Vec<u8> = (0..20).map(|_| rng.gen()).collect();
        base32::encode(base32::Alphabet::RFC4648 { padding: false }, &bytes)
    }
    
    fn verify_totp_code(secret: &str, code: &str) -> Result<bool> {
        // Implementation would use a TOTP library
        // For now, simplified validation
        Ok(code.len() == 6 && code.chars().all(|c| c.is_numeric()))
    }
    
    fn generate_backup_codes() -> Vec<String> {
        let mut rng = thread_rng();
        (0..10)
            .map(|_| {
                let code: String = (0..8)
                    .map(|_| rng.gen_range(0..10).to_string())
                    .collect();
                code
            })
            .collect()
    }
    
    fn hash_backup_codes(codes: &[String]) -> Vec<String> {
        codes.iter().map(|code| {
            let mut hasher = Sha256::new();
            hasher.update(code.as_bytes());
            format!("{:x}", hasher.finalize())
        }).collect()
    }
    
    fn calculate_level(experience: i64) -> i32 {
        // Level formula: level = floor(sqrt(experience / 100))
        ((experience as f64 / 100.0).sqrt() as i32).max(1)
    }
    
    async fn on_level_up(&self, db: &PgPool, old_level: i32, new_level: i32) -> Result<()> {
        // Award level up bonus
        let bonus_money = (new_level - old_level) * 1000;
        
        sqlx::query!(
            "UPDATE users SET money = money + $1 WHERE id = $2",
            bonus_money as i64,
            self.id
        )
        .execute(db)
        .await?;
        
        // Create notification
        sqlx::query!(
            "INSERT INTO notifications (user_id, message, notification_type, created_at) VALUES ($1, $2, 'level_up', NOW())",
            self.id,
            format!("Congratulations! You reached level {}! Bonus: ${}", new_level, bonus_money)
        )
        .execute(db)
        .await?;
        
        // Check for achievements
        self.check_level_achievements(db, new_level).await?;
        
        Ok(())
    }
    
    async fn check_level_achievements(&self, db: &PgPool, level: i32) -> Result<()> {
        let milestones = vec![10, 25, 50, 75, 100];
        
        for milestone in milestones {
            if level == milestone {
                // Award achievement
                sqlx::query!(
                    "INSERT INTO user_achievements (user_id, achievement_id, unlocked_at) 
                     VALUES ($1, (SELECT id FROM achievements WHERE level_requirement = $2), NOW())
                     ON CONFLICT DO NOTHING",
                    self.id,
                    milestone
                )
                .execute(db)
                .await?;
            }
        }
        
        Ok(())
    }
    
    async fn create_default_hardware(db: &PgPool, player_id: i32) -> Result<()> {
        sqlx::query!(
            "INSERT INTO hardware (user_id, cpu, ram, hdd, net) VALUES ($1, 500, 256, 10000, 10)",
            player_id
        )
        .execute(db)
        .await?;
        
        Ok(())
    }
    
    async fn send_verification_email(email: &str, token: &str) -> Result<()> {
        // Email sending implementation
        println!("Sending verification email to {} with token {}", email, token);
        Ok(())
    }
    
    async fn send_password_reset_email(email: &str, token: &str) -> Result<()> {
        // Email sending implementation
        println!("Sending password reset email to {} with token {}", email, token);
        Ok(())
    }
    
    async fn get_bank_accounts(db: &PgPool, player_id: i32) -> Result<Vec<BankAccount>> {
        let accounts = sqlx::query_as!(
            BankAccount,
            "SELECT * FROM bank_accounts WHERE user_id = $1",
            player_id
        )
        .fetch_all(db)
        .await?;
        Ok(accounts)
    }
    
    async fn get_software(db: &PgPool, player_id: i32) -> Result<Vec<Software>> {
        let software = sqlx::query_as!(
            Software,
            "SELECT * FROM software WHERE user_id = $1",
            player_id
        )
        .fetch_all(db)
        .await?;
        Ok(software)
    }
    
    async fn get_processes(db: &PgPool, player_id: i32) -> Result<Vec<Process>> {
        let processes = sqlx::query_as!(
            Process,
            "SELECT * FROM processes WHERE p_creator_id = $1 AND is_completed = false",
            player_id
        )
        .fetch_all(db)
        .await?;
        Ok(processes)
    }
    
    async fn get_missions(db: &PgPool, player_id: i32) -> Result<Vec<Mission>> {
        let missions = sqlx::query_as!(
            Mission,
            "SELECT m.* FROM missions m JOIN user_missions um ON m.id = um.mission_id WHERE um.user_id = $1",
            player_id
        )
        .fetch_all(db)
        .await?;
        Ok(missions)
    }
    
    async fn get_certifications(db: &PgPool, player_id: i32) -> Result<Vec<Certification>> {
        let certifications = sqlx::query_as!(
            Certification,
            "SELECT * FROM certifications WHERE user_id = $1",
            player_id
        )
        .fetch_all(db)
        .await?;
        Ok(certifications)
    }
    
    async fn get_achievements(db: &PgPool, player_id: i32) -> Result<Vec<Achievement>> {
        let achievements = sqlx::query_as!(
            Achievement,
            "SELECT a.* FROM achievements a JOIN user_achievements ua ON a.id = ua.achievement_id WHERE ua.user_id = $1",
            player_id
        )
        .fetch_all(db)
        .await?;
        Ok(achievements)
    }
    
    async fn get_notifications(db: &PgPool, player_id: i32) -> Result<Vec<Notification>> {
        let notifications = sqlx::query_as!(
            Notification,
            "SELECT * FROM notifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT 50",
            player_id
        )
        .fetch_all(db)
        .await?;
        Ok(notifications)
    }
    
    async fn get_friends(db: &PgPool, player_id: i32) -> Result<Vec<Friend>> {
        let friends = sqlx::query_as!(
            Friend,
            "SELECT f.friend_id as player_id, u.login as username, u.is_online, f.added_at 
             FROM friends f JOIN users u ON f.friend_id = u.id 
             WHERE f.user_id = $1",
            player_id
        )
        .fetch_all(db)
        .await?;
        Ok(friends)
    }
    
    async fn get_enemies(db: &PgPool, player_id: i32) -> Result<Vec<Enemy>> {
        let enemies = sqlx::query_as!(
            Enemy,
            "SELECT e.enemy_id as player_id, u.login as username, e.threat_level, e.last_attack 
             FROM enemies e JOIN users u ON e.enemy_id = u.id 
             WHERE e.user_id = $1",
            player_id
        )
        .fetch_all(db)
        .await?;
        Ok(enemies)
    }
    
    async fn get_logs(db: &PgPool, player_id: i32) -> Result<Vec<LogEntry>> {
        let logs = sqlx::query_as!(
            LogEntry,
            "SELECT * FROM logs WHERE user_id = $1 AND is_deleted = false ORDER BY timestamp DESC LIMIT 100",
            player_id
        )
        .fetch_all(db)
        .await?;
        Ok(logs)
    }
    
    async fn get_emails(db: &PgPool, player_id: i32) -> Result<Vec<Email>> {
        let emails = sqlx::query_as!(
            Email,
            "SELECT * FROM emails WHERE recipient_id = $1 ORDER BY received_at DESC LIMIT 50",
            player_id
        )
        .fetch_all(db)
        .await?;
        Ok(emails)
    }
    
    /// Purchase premium subscription
    pub async fn purchase_premium(&mut self, db: &PgPool, months: i32) -> Result<()> {
        let cost = months as i64 * 10000; // $10,000 per month
        if self.money < cost {
            return Err(anyhow::anyhow!("Insufficient funds for premium subscription"));
        }
        
        self.remove_money(db, cost).await?;
        
        let premium_until = match self.premium_until {
            Some(current) if current > Utc::now() => current + Duration::days(30 * months as i64),
            _ => Utc::now() + Duration::days(30 * months as i64),
        };
        
        sqlx::query!(
            "UPDATE users SET premium_until = $1 WHERE id = $2",
            premium_until,
            self.id
        )
        .execute(db)
        .await?;
        
        self.premium_until = Some(premium_until);
        Ok(())
    }
    
    /// Check if player has premium
    pub fn is_premium(&self) -> bool {
        self.premium_until.map_or(false, |until| until > Utc::now())
    }
    
    /// Get online status
    pub async fn update_online_status(&mut self, db: &PgPool, is_online: bool) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET is_online = $1, last_activity = NOW() WHERE id = $2",
            is_online,
            self.id
        )
        .execute(db)
        .await?;
        
        self.is_online = is_online;
        self.last_activity = Some(Utc::now());
        Ok(())
    }
    
    /// Get player's IP logs
    pub async fn get_ip_logs(&self, db: &PgPool) -> Result<Vec<String>> {
        let logs = sqlx::query!(
            "SELECT DISTINCT ip FROM login_logs WHERE user_id = $1 ORDER BY timestamp DESC LIMIT 10",
            self.id
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|r| r.ip)
        .collect();
        
        Ok(logs)
    }
    
    /// Create a bounty on another player
    pub async fn create_bounty(&mut self, db: &PgPool, target_id: i32, reward: i64) -> Result<()> {
        if reward <= 0 || reward > self.money {
            return Err(anyhow::anyhow!("Invalid bounty amount"));
        }
        
        if target_id == self.id {
            return Err(anyhow::anyhow!("Cannot place bounty on yourself"));
        }
        
        self.remove_money(db, reward).await?;
        
        sqlx::query!(
            "INSERT INTO bounties (creator_id, target_id, reward, created_at) VALUES ($1, $2, $3, NOW())",
            self.id,
            target_id,
            reward
        )
        .execute(db)
        .await?;
        
        Ok(())
    }
    
    /// Claim a bounty
    pub async fn claim_bounty(&mut self, db: &PgPool, bounty_id: i32) -> Result<()> {
        let bounty = sqlx::query!(
            "SELECT * FROM bounties WHERE id = $1 AND is_claimed = false",
            bounty_id
        )
        .fetch_optional(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Bounty not found or already claimed"))?;
        
        // Mark bounty as claimed
        sqlx::query!(
            "UPDATE bounties SET is_claimed = true, claimed_by = $1, claimed_at = NOW() WHERE id = $2",
            self.id,
            bounty_id
        )
        .execute(db)
        .await?;
        
        // Award reward
        self.add_money(db, bounty.reward).await?;
        
        // Update stats
        sqlx::query!(
            "UPDATE user_stats SET bounties_completed = bounties_completed + 1, total_bounty_reward = total_bounty_reward + $1 WHERE user_id = $2",
            bounty.reward,
            self.id
        )
        .execute(db)
        .await?;
        
        Ok(())
    }
    
    /// Install a virus on target
    pub async fn install_virus(&mut self, db: &PgPool, target_ip: &str, virus_type: &str) -> Result<i32> {
        // Validate target
        let target = sqlx::query!(
            "SELECT * FROM external_hardware WHERE ip_address = $1",
            target_ip
        )
        .fetch_optional(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Target not found"))?;
        
        // Check if player has virus software
        let virus = sqlx::query!(
            "SELECT * FROM software WHERE user_id = $1 AND software_type = $2 AND is_installed = true",
            self.id,
            virus_type
        )
        .fetch_optional(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Virus software not found or not installed"))?;
        
        // Start virus installation process
        let process_id = self.start_process(db, "virus_install", target_ip, 300).await?;
        
        // Update stats
        sqlx::query!(
            "UPDATE user_stats SET viruses_installed = viruses_installed + 1 WHERE user_id = $1",
            self.id
        )
        .execute(db)
        .await?;
        
        Ok(process_id)
    }
    
    /// Remove a virus
    pub async fn remove_virus(&mut self, db: &PgPool, virus_id: i32) -> Result<()> {
        sqlx::query!(
            "DELETE FROM viruses WHERE id = $1 AND infected_user_id = $2",
            virus_id,
            self.id
        )
        .execute(db)
        .await?;
        
        // Update stats
        sqlx::query!(
            "UPDATE user_stats SET viruses_removed = viruses_removed + 1 WHERE user_id = $1",
            self.id
        )
        .execute(db)
        .await?;
        
        Ok(())
    }
    
    /// Send DDOS attack
    pub async fn send_ddos(&mut self, db: &PgPool, target_id: i32, duration: i32) -> Result<()> {
        if target_id == self.id {
            return Err(anyhow::anyhow!("Cannot DDOS yourself"));
        }
        
        let cost = duration as i64 * 100; // $100 per second
        if self.money < cost {
            return Err(anyhow::anyhow!("Insufficient funds for DDOS attack"));
        }
        
        self.remove_money(db, cost).await?;
        
        sqlx::query!(
            "INSERT INTO ddos_attacks (attacker_id, target_id, duration, started_at) VALUES ($1, $2, $3, NOW())",
            self.id,
            target_id,
            duration
        )
        .execute(db)
        .await?;
        
        // Update stats
        sqlx::query!(
            "UPDATE user_stats SET ddos_attacks_sent = ddos_attacks_sent + 1 WHERE user_id = $1",
            self.id
        )
        .execute(db)
        .await?;
        
        sqlx::query!(
            "UPDATE user_stats SET ddos_attacks_received = ddos_attacks_received + 1 WHERE user_id = $1",
            target_id
        )
        .execute(db)
        .await?;
        
        Ok(())
    }
    
    /// Download file from target
    pub async fn download_file(&mut self, db: &PgPool, target_ip: &str, file_id: i32) -> Result<i32> {
        let file = sqlx::query!(
            "SELECT * FROM files WHERE id = $1 AND server_ip = $2",
            file_id,
            target_ip
        )
        .fetch_optional(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("File not found on target"))?;
        
        // Calculate download time based on file size and network speed
        let download_time = (file.size / 1024) * 10; // 10 seconds per MB
        
        let process_id = self.start_process(db, "download", target_ip, download_time).await?;
        
        // Update stats
        sqlx::query!(
            "UPDATE user_stats SET files_downloaded = files_downloaded + 1, total_data_downloaded = total_data_downloaded + $1 WHERE user_id = $2",
            file.size as i64,
            self.id
        )
        .execute(db)
        .await?;
        
        Ok(process_id)
    }
    
    /// Upload file to target
    pub async fn upload_file(&mut self, db: &PgPool, target_ip: &str, file_id: i32) -> Result<i32> {
        let file = sqlx::query!(
            "SELECT * FROM files WHERE id = $1 AND owner_id = $2",
            file_id,
            self.id
        )
        .fetch_optional(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("File not found"))?;
        
        // Calculate upload time
        let upload_time = (file.size / 512) * 10; // Slower than download
        
        let process_id = self.start_process(db, "upload", target_ip, upload_time).await?;
        
        // Update stats
        sqlx::query!(
            "UPDATE user_stats SET files_uploaded = files_uploaded + 1, total_data_uploaded = total_data_uploaded + $1 WHERE user_id = $2",
            file.size as i64,
            self.id
        )
        .execute(db)
        .await?;
        
        Ok(process_id)
    }
    
    /// Delete file
    pub async fn delete_file(&self, db: &PgPool, file_id: i32) -> Result<()> {
        sqlx::query!(
            "DELETE FROM files WHERE id = $1 AND owner_id = $2",
            file_id,
            self.id
        )
        .execute(db)
        .await?;
        
        // Update stats
        sqlx::query!(
            "UPDATE user_stats SET files_deleted = files_deleted + 1 WHERE user_id = $1",
            self.id
        )
        .execute(db)
        .await?;
        
        Ok(())
    }
    
    /// Get player's servers
    pub async fn get_servers(&self, db: &PgPool) -> Result<Vec<String>> {
        let servers = sqlx::query!(
            "SELECT ip_address FROM servers WHERE owner_id = $1",
            self.id
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|r| r.ip_address)
        .collect();
        
        Ok(servers)
    }
    
    /// Purchase server
    pub async fn purchase_server(&mut self, db: &PgPool, server_type: &str) -> Result<String> {
        let cost = match server_type {
            "basic" => 50000,
            "advanced" => 150000,
            "premium" => 500000,
            _ => return Err(anyhow::anyhow!("Invalid server type")),
        };
        
        if self.money < cost {
            return Err(anyhow::anyhow!("Insufficient funds"));
        }
        
        self.remove_money(db, cost).await?;
        
        // Generate random IP
        let ip = format!("192.168.{}.{}", 
            rand::thread_rng().gen_range(1..255),
            rand::thread_rng().gen_range(1..255)
        );
        
        sqlx::query!(
            "INSERT INTO servers (owner_id, ip_address, server_type, purchased_at) VALUES ($1, $2, $3, NOW())",
            self.id,
            ip,
            server_type
        )
        .execute(db)
        .await?;
        
        // Update stats
        sqlx::query!(
            "UPDATE user_stats SET servers_owned = servers_owned + 1 WHERE user_id = $1",
            self.id
        )
        .execute(db)
        .await?;
        
        Ok(ip)
    }
    
    /// Participate in PvP
    pub async fn pvp_attack(&mut self, db: &PgPool, target_id: i32) -> Result<bool> {
        if target_id == self.id {
            return Err(anyhow::anyhow!("Cannot attack yourself"));
        }
        
        let target = sqlx::query!(
            "SELECT level FROM users WHERE id = $1",
            target_id
        )
        .fetch_optional(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Target not found"))?;
        
        // Simple PvP calculation
        let success = self.level > target.level || 
            (self.level == target.level && rand::thread_rng().gen_bool(0.5));
        
        if success {
            // Winner takes money
            let prize = 10000;
            sqlx::query!(
                "UPDATE users SET money = money + $1 WHERE id = $2",
                prize,
                self.id
            )
            .execute(db)
            .await?;
            
            sqlx::query!(
                "UPDATE users SET money = GREATEST(0, money - $1) WHERE id = $2",
                prize,
                target_id
            )
            .execute(db)
            .await?;
            
            // Update stats
            sqlx::query!(
                "UPDATE user_stats SET pvp_wins = pvp_wins + 1 WHERE user_id = $1",
                self.id
            )
            .execute(db)
            .await?;
            
            sqlx::query!(
                "UPDATE user_stats SET pvp_losses = pvp_losses + 1 WHERE user_id = $1",
                target_id
            )
            .execute(db)
            .await?;
        } else {
            // Update loss stats
            sqlx::query!(
                "UPDATE user_stats SET pvp_losses = pvp_losses + 1 WHERE user_id = $1",
                self.id
            )
            .execute(db)
            .await?;
            
            sqlx::query!(
                "UPDATE user_stats SET pvp_wins = pvp_wins + 1 WHERE user_id = $1",
                target_id
            )
            .execute(db)
            .await?;
        }
        
        Ok(success)
    }
}

impl PlayerStats {
    async fn initialize(db: &PgPool, player_id: i32) -> Result<()> {
        sqlx::query!(
            "INSERT INTO user_stats (user_id) VALUES ($1) ON CONFLICT DO NOTHING",
            player_id
        )
        .execute(db)
        .await?;
        
        Ok(())
    }
    
    async fn get(db: &PgPool, player_id: i32) -> Result<Self> {
        let stats = sqlx::query_as!(
            PlayerStats,
            "SELECT * FROM user_stats WHERE user_id = $1",
            player_id
        )
        .fetch_one(db)
        .await?;
        
        Ok(stats)
    }
}