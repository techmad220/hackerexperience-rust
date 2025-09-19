use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use chrono::{DateTime, Utc, NaiveDateTime};

use crate::error::Result;

/// Represents a hacked server in the player's list
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HackedServer {
    pub id: i32,
    pub user_id: i32,
    pub ip: i64,
    pub username: String,
    pub password: String,
    pub hacked_time: NaiveDateTime,
    pub virus_id: Option<i32>,
}

/// Bank account in the player's list
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HackedBankAccount {
    pub id: i32,
    pub user_id: i32,
    pub bank_acc: String,
    pub bank_pass: String,
    pub bank_id: i32,
    pub hacked_date: NaiveDateTime,
    pub last_money: i32,
    pub last_money_date: NaiveDateTime,
}

/// Server specifications for analysis
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ServerSpecs {
    pub list_id: i32,
    pub spec_hdd: i32,
    pub spec_net: i32,
    pub min_cpu: Option<i32>,
    pub max_cpu: Option<i32>,
    pub min_ram: Option<i32>,
    pub max_ram: Option<i32>,
}

/// Notification for list changes
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ListNotification {
    pub user_id: i32,
    pub ip: i64,
    pub notification_type: i32,
    pub virus_name: Option<String>,
}

/// Collect result information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectResult {
    pub total_money: i32,
    pub total_btc: f64,
    pub servers_processed: i32,
    pub servers_skipped: i32,
    pub collect_text: String,
}

/// Virus collection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirusStats {
    pub virus_id: i32,
    pub installed_ip: i64,
    pub virus_type: i32,
    pub virus_version: i32,
    pub last_collect: NaiveDateTime,
    pub duration_seconds: i32,
}

/// Collection history
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CollectHistory {
    pub user_id: i32,
    pub collect_text: String,
}

/// Notification types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationType {
    ServerDown = 1,
    PasswordChanged = 2,
    VirusDown = 3,
}

/// Virus types for collection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VirusType {
    Spam = 1,
    Warez = 2,
    DDoS = 3,
    Miner = 4,
}

#[async_trait]
pub trait ListRepository {
    /// Add server to hacked list
    async fn add_to_list(&self, user_id: i32, ip: i64, username: &str, password: &str, virus_id: Option<i32>) -> Result<()>;
    
    /// Add bank account to hacked list
    async fn add_bank_account(&self, user_id: i32, bank_id: i32, account: &str, password: &str) -> Result<()>;
    
    /// Check if server is in list
    async fn is_listed(&self, user_id: i32, ip: i64) -> Result<bool>;
    
    /// Check if bank account is in list
    async fn is_bank_listed(&self, user_id: i32, account: &str) -> Result<bool>;
    
    /// Check if server was exploited (no password cracked)
    async fn is_exploited(&self, user_id: i32, ip: i64) -> Result<bool>;
    
    /// Check if server password is unknown
    async fn is_unknown(&self, user_id: i32, ip: i64) -> Result<bool>;
    
    /// Check if server was accessed via download
    async fn is_download(&self, user_id: i32, ip: i64) -> Result<bool>;
    
    /// Reveal unknown password after cracking
    async fn reveal_unknown_password(&self, user_id: i32, ip: i64, username: &str, password: &str) -> Result<()>;
    
    /// Get login credentials for server
    async fn get_login_info(&self, user_id: i32, ip: i64) -> Result<Option<(String, String)>>;
    
    /// Get bank account credentials
    async fn get_bank_login_info(&self, user_id: i32, account: &str) -> Result<Option<String>>;
    
    /// Get list ID by IP
    async fn get_list_id_by_ip(&self, user_id: i32, ip: i64) -> Result<Option<i32>>;
    
    /// Get IP by list ID
    async fn get_list_ip_by_id(&self, list_id: i32) -> Result<Option<i64>>;
    
    /// Check if list entry belongs to user
    async fn is_list_entry_valid(&self, list_id: i32, user_id: i32, entry_type: i32) -> Result<bool>;
    
    /// Assign virus to server
    async fn assign_virus(&self, virus_id: i32, list_ip: i64, list_id: i32) -> Result<()>;
    
    /// Delete from list
    async fn delete_list_entry(&self, list_id: i32, user_id: i32, entry_type: i32) -> Result<()>;
    
    /// Update server specifications
    async fn update_server_specs(&self, list_id: i32, hdd: i32, net: i32) -> Result<()>;
    
    /// Update analyzed hardware specifications
    async fn update_analyzed_specs(&self, list_id: i32, min_cpu: i32, max_cpu: i32, min_ram: i32, max_ram: i32) -> Result<()>;
    
    /// Update bank account money
    async fn update_bank_money(&self, user_id: i32, account: &str, money: i32) -> Result<()>;
    
    /// Add notification
    async fn add_notification(&self, user_id: i32, ip: i64, notification_type: NotificationType, virus_name: Option<&str>) -> Result<()>;
    
    /// Get and clear notifications
    async fn get_notifications(&self, user_id: i32) -> Result<Vec<ListNotification>>;
    
    /// Collect money from viruses
    async fn collect_money(&self, user_id: i32, bank_account: &str) -> Result<CollectResult>;
    
    /// Get last collect information
    async fn get_last_collect(&self, user_id: i32) -> Result<Option<String>>;
    
    /// Update last collect for virus
    async fn set_last_collect(&self, virus_id: i32) -> Result<()>;
    
    /// Count DDoS viruses
    async fn count_ddos_viruses(&self, user_id: i32) -> Result<i32>;
    
    /// List DDoS viruses
    async fn list_ddos_viruses(&self, user_id: i32) -> Result<Vec<VirusStats>>;
    
    /// Get user's hacked servers
    async fn get_hacked_servers(&self, user_id: i32, limit: i32, offset: i32) -> Result<Vec<HackedServer>>;
    
    /// Get user's hacked bank accounts
    async fn get_hacked_bank_accounts(&self, user_id: i32, limit: i32, offset: i32) -> Result<Vec<HackedBankAccount>>;
    
    /// Count user's hacked servers
    async fn count_hacked_servers(&self, user_id: i32) -> Result<i32>;
    
    /// Count user's hacked bank accounts
    async fn count_hacked_bank_accounts(&self, user_id: i32) -> Result<i32>;
}

pub struct ListService {
    db: PgPool,
}

impl ListService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
    
    /// Calculate money generated by virus based on type and specs
    fn calculate_virus_money(virus_type: VirusType, version: i32, hardware_usage: i32, work_time: i32) -> f64 {
        let base_rate = match virus_type {
            VirusType::Spam => 0.00000694, // $0.025 per mhz per hour
            VirusType::Warez => 0.000555556, // $2 per mbit per hour
            VirusType::Miner => 0.00010417 / 3600.0, // BTC per mhz per hour
            VirusType::DDoS => 0.0, // No money generation
        };
        
        let virus_percent = ((version as f64 / 1000.0) * 2.0) + 1.0;
        let rate = base_rate / 2.0; // Slow mode
        
        work_time as f64 * rate * hardware_usage as f64 * virus_percent
    }
    
    /// Format collect duration for display
    fn format_work_duration(seconds: i32) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        
        if hours > 0 {
            if minutes > 0 {
                format!("{} hours and {} minutes", hours, minutes)
            } else {
                format!("{} hours", hours)
            }
        } else if minutes > 0 {
            format!("{} minutes", minutes)
        } else {
            "less than a minute".to_string()
        }
    }
}

#[async_trait]
impl ListRepository for ListService {
    /// Add server to hacked list
    async fn add_to_list(&self, user_id: i32, ip: i64, username: &str, password: &str, virus_id: Option<i32>) -> Result<()> {
        if !self.is_listed(user_id, ip).await? {
            sqlx::query!(
                "INSERT INTO lists (user_id, ip, username, password, hacked_time, virus_id)
                 VALUES ($1, $2, $3, $4, NOW(), $5)",
                user_id,
                ip,
                username,
                password,
                virus_id
            )
            .execute(&self.db)
            .await?;
            
            // Update hack count
            sqlx::query!(
                "UPDATE users_stats SET hack_count = hack_count + 1 WHERE uid = $1",
                user_id
            )
            .execute(&self.db)
            .await?;
        }
        
        Ok(())
    }
    
    /// Add bank account to hacked list
    async fn add_bank_account(&self, user_id: i32, bank_id: i32, account: &str, password: &str) -> Result<()> {
        sqlx::query!(
            "INSERT INTO lists_bank_accounts (user_id, bank_acc, bank_pass, bank_id, hacked_date, last_money, last_money_date)
             VALUES ($1, $2, $3, $4, NOW(), -1, NOW())",
            user_id,
            account,
            password,
            bank_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Check if server is in list
    async fn is_listed(&self, user_id: i32, ip: i64) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM lists WHERE user_id = $1 AND ip = $2",
            user_id,
            ip
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) > 0)
    }
    
    /// Check if bank account is in list
    async fn is_bank_listed(&self, user_id: i32, account: &str) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM lists_bank_accounts WHERE user_id = $1 AND bank_acc = $2",
            user_id,
            account
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) > 0)
    }
    
    /// Check if server was exploited (no password cracked)
    async fn is_exploited(&self, user_id: i32, ip: i64) -> Result<bool> {
        let password = sqlx::query_scalar!(
            "SELECT password FROM lists WHERE user_id = $1 AND ip = $2",
            user_id,
            ip
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(password.as_deref() == Some("exploited"))
    }
    
    /// Check if server password is unknown
    async fn is_unknown(&self, user_id: i32, ip: i64) -> Result<bool> {
        let password = sqlx::query_scalar!(
            "SELECT password FROM lists WHERE user_id = $1 AND ip = $2",
            user_id,
            ip
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(password.as_deref() == Some("unknown"))
    }
    
    /// Check if server was accessed via download
    async fn is_download(&self, user_id: i32, ip: i64) -> Result<bool> {
        let password = sqlx::query_scalar!(
            "SELECT password FROM lists WHERE user_id = $1 AND ip = $2",
            user_id,
            ip
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(password.as_deref() == Some("download"))
    }
    
    /// Reveal unknown password after cracking
    async fn reveal_unknown_password(&self, user_id: i32, ip: i64, username: &str, password: &str) -> Result<()> {
        sqlx::query!(
            "UPDATE lists SET username = $3, password = $4 WHERE user_id = $1 AND ip = $2",
            user_id,
            ip,
            username,
            password
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Get login credentials for server
    async fn get_login_info(&self, user_id: i32, ip: i64) -> Result<Option<(String, String)>> {
        let credentials = sqlx::query!(
            "SELECT username, password FROM lists WHERE user_id = $1 AND ip = $2",
            user_id,
            ip
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(credentials.map(|c| (c.username, c.password)))
    }
    
    /// Get bank account credentials
    async fn get_bank_login_info(&self, user_id: i32, account: &str) -> Result<Option<String>> {
        let password = sqlx::query_scalar!(
            "SELECT bank_pass FROM lists_bank_accounts WHERE user_id = $1 AND bank_acc = $2",
            user_id,
            account
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(password)
    }
    
    /// Get list ID by IP
    async fn get_list_id_by_ip(&self, user_id: i32, ip: i64) -> Result<Option<i32>> {
        let id = sqlx::query_scalar!(
            "SELECT id FROM lists WHERE user_id = $1 AND ip = $2",
            user_id,
            ip
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(id)
    }
    
    /// Get IP by list ID
    async fn get_list_ip_by_id(&self, list_id: i32) -> Result<Option<i64>> {
        let ip = sqlx::query_scalar!(
            "SELECT ip FROM lists WHERE id = $1",
            list_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(ip)
    }
    
    /// Check if list entry belongs to user
    async fn is_list_entry_valid(&self, list_id: i32, user_id: i32, entry_type: i32) -> Result<bool> {
        let table = if entry_type == 1 { "lists" } else { "lists_bank_accounts" };
        
        let query = format!("SELECT COUNT(*) FROM {} WHERE id = $1 AND user_id = $2", table);
        let count = sqlx::query_scalar(&query)
            .bind(list_id)
            .bind(user_id)
            .fetch_one(&self.db)
            .await?;
        
        Ok(count.unwrap_or(0) > 0)
    }
    
    /// Assign virus to server
    async fn assign_virus(&self, virus_id: i32, list_ip: i64, list_id: i32) -> Result<()> {
        // Update virus last collect time
        sqlx::query!(
            "UPDATE virus SET last_collect = NOW() WHERE virus_id = $1 AND installed_ip = $2",
            virus_id,
            list_ip
        )
        .execute(&self.db)
        .await?;
        
        // Update list entry
        sqlx::query!(
            "UPDATE lists SET virus_id = $1 WHERE id = $2",
            virus_id,
            list_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Delete from list
    async fn delete_list_entry(&self, list_id: i32, user_id: i32, entry_type: i32) -> Result<()> {
        if entry_type == 1 {
            // Deactivate associated viruses
            sqlx::query!(
                "UPDATE virus 
                 SET active = false
                 FROM lists
                 WHERE virus.installed_ip = lists.ip 
                   AND lists.id = $1 
                   AND virus.installed_by = $2",
                list_id,
                user_id
            )
            .execute(&self.db)
            .await?;
            
            // Delete server entry
            sqlx::query!(
                "DELETE FROM lists WHERE id = $1 AND user_id = $2",
                list_id,
                user_id
            )
            .execute(&self.db)
            .await?;
            
            // Delete specs
            sqlx::query!(
                "DELETE FROM lists_specs WHERE list_id = $1",
                list_id
            )
            .execute(&self.db)
            .await?;
        } else {
            // Delete bank account
            sqlx::query!(
                "DELETE FROM lists_bank_accounts WHERE id = $1 AND user_id = $2",
                list_id,
                user_id
            )
            .execute(&self.db)
            .await?;
        }
        
        Ok(())
    }
    
    /// Update server specifications
    async fn update_server_specs(&self, list_id: i32, hdd: i32, net: i32) -> Result<()> {
        let exists = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM lists_specs WHERE list_id = $1",
            list_id
        )
        .fetch_one(&self.db)
        .await?;
        
        if exists.unwrap_or(0) > 0 {
            sqlx::query!(
                "UPDATE lists_specs SET spec_hdd = $2, spec_net = $3 WHERE list_id = $1",
                list_id,
                hdd,
                net
            )
            .execute(&self.db)
            .await?;
        } else {
            sqlx::query!(
                "INSERT INTO lists_specs (list_id, spec_hdd, spec_net) VALUES ($1, $2, $3)",
                list_id,
                hdd,
                net
            )
            .execute(&self.db)
            .await?;
        }
        
        Ok(())
    }
    
    /// Update analyzed hardware specifications
    async fn update_analyzed_specs(&self, list_id: i32, min_cpu: i32, max_cpu: i32, min_ram: i32, max_ram: i32) -> Result<()> {
        let exists = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM lists_specs_analyzed WHERE list_id = $1",
            list_id
        )
        .fetch_one(&self.db)
        .await?;
        
        if exists.unwrap_or(0) > 0 {
            sqlx::query!(
                "UPDATE lists_specs_analyzed 
                 SET min_cpu = $2, max_cpu = $3, min_ram = $4, max_ram = $5
                 WHERE list_id = $1",
                list_id,
                min_cpu,
                max_cpu,
                min_ram,
                max_ram
            )
            .execute(&self.db)
            .await?;
        } else {
            sqlx::query!(
                "INSERT INTO lists_specs_analyzed (list_id, min_cpu, max_cpu, min_ram, max_ram)
                 VALUES ($1, $2, $3, $4, $5)",
                list_id,
                min_cpu,
                max_cpu,
                min_ram,
                max_ram
            )
            .execute(&self.db)
            .await?;
        }
        
        Ok(())
    }
    
    /// Update bank account money
    async fn update_bank_money(&self, user_id: i32, account: &str, money: i32) -> Result<()> {
        sqlx::query!(
            "UPDATE lists_bank_accounts 
             SET last_money = $3, last_money_date = NOW() 
             WHERE user_id = $1 AND bank_acc = $2",
            user_id,
            account,
            money
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Add notification
    async fn add_notification(&self, user_id: i32, ip: i64, notification_type: NotificationType, virus_name: Option<&str>) -> Result<()> {
        sqlx::query!(
            "INSERT INTO lists_notifications (user_id, ip, notification_type, virus_name)
             VALUES ($1, $2, $3, $4)",
            user_id,
            ip,
            notification_type as i32,
            virus_name
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Get and clear notifications
    async fn get_notifications(&self, user_id: i32) -> Result<Vec<ListNotification>> {
        let notifications = sqlx::query_as!(
            ListNotification,
            "SELECT user_id, ip, notification_type, virus_name 
             FROM lists_notifications WHERE user_id = $1",
            user_id
        )
        .fetch_all(&self.db)
        .await?;
        
        // Clear notifications
        sqlx::query!(
            "DELETE FROM lists_notifications WHERE user_id = $1",
            user_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(notifications)
    }
    
    /// Collect money from viruses - simplified implementation
    async fn collect_money(&self, user_id: i32, bank_account: &str) -> Result<CollectResult> {
        // This is a simplified version of the complex collection logic
        let virus_stats = sqlx::query!(
            "SELECT v.virus_id, v.installed_ip, v.virus_type, v.virus_version, v.last_collect,
                    EXTRACT(epoch FROM (NOW() - v.last_collect)) as duration_seconds
             FROM virus v
             INNER JOIN lists l ON l.virus_id = v.virus_id
             WHERE v.installed_by = $1 AND v.virus_type != 3 AND v.active = true",
            user_id
        )
        .fetch_all(&self.db)
        .await?;
        
        let mut total_money = 0;
        let mut total_btc = 0.0;
        let mut servers_processed = 0;
        let mut collect_text = String::new();
        
        for virus in virus_stats {
            let duration = virus.duration_seconds.unwrap_or(0) as i32;
            
            // Only collect if minimum time has passed (10 minutes)
            if duration >= 600 {
                // Simplified calculation
                let money = duration / 60; // $1 per minute
                total_money += money;
                servers_processed += 1;
                
                collect_text.push_str(&format!(
                    "Server {} generated ${} in {}.\n", 
                    Self::long_to_ip(virus.installed_ip),
                    money,
                    Self::format_work_duration(duration)
                ));
                
                // Update last collect
                self.set_last_collect(virus.virus_id).await?;
            }
        }
        
        // Save collect text
        sqlx::query!(
            "INSERT INTO lists_collect (user_id, collect_text) 
             VALUES ($1, $2)
             ON CONFLICT (user_id) DO UPDATE SET collect_text = $2",
            user_id,
            collect_text
        )
        .execute(&self.db)
        .await?;
        
        Ok(CollectResult {
            total_money,
            total_btc,
            servers_processed,
            servers_skipped: 0,
            collect_text,
        })
    }
    
    /// Get last collect information
    async fn get_last_collect(&self, user_id: i32) -> Result<Option<String>> {
        let text = sqlx::query_scalar!(
            "SELECT collect_text FROM lists_collect WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(text)
    }
    
    /// Update last collect for virus
    async fn set_last_collect(&self, virus_id: i32) -> Result<()> {
        sqlx::query!(
            "UPDATE virus SET last_collect = NOW() WHERE virus_id = $1",
            virus_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Count DDoS viruses
    async fn count_ddos_viruses(&self, user_id: i32) -> Result<i32> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM virus_ddos 
             INNER JOIN lists ON lists.virus_id = virus_ddos.ddos_id
             WHERE virus_ddos.user_id = $1 AND virus_ddos.active = true",
            user_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) as i32)
    }
    
    /// List DDoS viruses - simplified
    async fn list_ddos_viruses(&self, user_id: i32) -> Result<Vec<VirusStats>> {
        // Simplified implementation
        Ok(vec![])
    }
    
    /// Get user's hacked servers
    async fn get_hacked_servers(&self, user_id: i32, limit: i32, offset: i32) -> Result<Vec<HackedServer>> {
        let servers = sqlx::query_as!(
            HackedServer,
            "SELECT id, user_id, ip, username, password, hacked_time, virus_id
             FROM lists 
             WHERE user_id = $1
             ORDER BY hacked_time DESC
             LIMIT $2 OFFSET $3",
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await?;
        
        Ok(servers)
    }
    
    /// Get user's hacked bank accounts
    async fn get_hacked_bank_accounts(&self, user_id: i32, limit: i32, offset: i32) -> Result<Vec<HackedBankAccount>> {
        let accounts = sqlx::query_as!(
            HackedBankAccount,
            "SELECT id, user_id, bank_acc, bank_pass, bank_id, hacked_date, last_money, last_money_date
             FROM lists_bank_accounts 
             WHERE user_id = $1
             ORDER BY hacked_date DESC
             LIMIT $2 OFFSET $3",
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await?;
        
        Ok(accounts)
    }
    
    /// Count user's hacked servers
    async fn count_hacked_servers(&self, user_id: i32) -> Result<i32> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM lists WHERE user_id = $1",
            user_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) as i32)
    }
    
    /// Count user's hacked bank accounts
    async fn count_hacked_bank_accounts(&self, user_id: i32) -> Result<i32> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM lists_bank_accounts WHERE user_id = $1",
            user_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) as i32)
    }
}

impl ListService {
    /// Convert long to IP address string  
    fn long_to_ip(ip_long: i64) -> String {
        format!("{}.{}.{}.{}", 
            (ip_long >> 24) & 0xFF,
            (ip_long >> 16) & 0xFF,
            (ip_long >> 8) & 0xFF,
            ip_long & 0xFF
        )
    }
}