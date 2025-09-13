//! PC (Hardware) entity - Computer hardware management and operations
//! 
//! This module provides the PC struct and methods for managing computer hardware,
//! including virtual PCs (VPC) and NPC computers. It handles hardware specifications,
//! software management, DDoS attacks, virus operations, and more.

use sqlx::{Pool, Postgres, Row};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::entities::session::Session;
use crate::entities::player::Player;
use crate::error::HeResult;

/// Represents a computer (PC/hardware) in the Hacker Experience game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareVPC {
    /// CPU power in MHz
    pub cpu: Option<i32>,
    /// Hard Drive capacity in MB
    pub hdd: Option<i32>,
    /// RAM capacity in MB
    pub ram: Option<i32>,
    /// External Hard Drive capacity in MB
    pub xhd: Option<i32>,
    /// Network speed in Mbps
    pub net: Option<i32>,
    /// Total number of PCs owned
    pub total_pcs: Option<i32>,
    /// Player ID
    pub id: Option<i32>,
    /// HDD specifications
    pub hdd_specs: Option<String>,
    /// Database connection pool
    #[serde(skip)]
    pub db_pool: Option<Pool<Postgres>>,
}

/// Hardware information returned from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub cpu: i32,
    pub hdd: i32,
    pub ram: i32,
    pub net: i32,
    pub xhd: i32,
}

/// PC specification details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PCSpec {
    pub id: i32,
    pub name: String,
    pub cpu: i32,
    pub hdd: i32,
    pub ram: i32,
    pub net: i32,
    pub user_id: i32,
    pub is_npc: bool,
}

/// Software information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareInfo {
    pub id: i32,
    pub name: String,
    pub software_type: i32,
    pub version: i32,
    pub size: i32,
    pub installed: bool,
    pub hidden: bool,
    pub folder_id: Option<i32>,
}

/// Virus information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirusInfo {
    pub id: i32,
    pub virus_type: i32,
    pub version: i32,
    pub victim_id: i32,
    pub victim_npc: bool,
    pub installed_at: DateTime<Utc>,
}

/// DDoS attack information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DDosInfo {
    pub power: i32,
    pub servers: i32,
    pub attacker_id: i32,
    pub victim_id: i32,
    pub victim_pc_type: String,
    pub damage: HashMap<String, i32>,
}

/// Log entry information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogInfo {
    pub id: i32,
    pub user_id: i32,
    pub text: String,
    pub timestamp: DateTime<Utc>,
    pub is_npc: bool,
}

impl HardwareVPC {
    /// Creates a new HardwareVPC instance
    pub fn new(db_pool: Pool<Postgres>) -> Self {
        Self {
            cpu: None,
            hdd: None,
            ram: None,
            xhd: None,
            net: None,
            total_pcs: None,
            id: None,
            hdd_specs: None,
            db_pool: Some(db_pool),
        }
    }

    /// Creates a new HardwareVPC instance with an ID
    pub fn new_with_id(id: i32, db_pool: Pool<Postgres>) -> Self {
        Self {
            cpu: None,
            hdd: None,
            ram: None,
            xhd: None,
            net: None,
            total_pcs: None,
            id: Some(id),
            hdd_specs: None,
            db_pool: Some(db_pool),
        }
    }

    /// Gets hardware information for a user
    /// 
    /// # Arguments
    /// * `id` - User ID (if empty, uses session ID)
    /// * `pc_type` - PC type ("VPC" or "NPC")
    /// * `xhd` - External HD parameter (optional)
    /// 
    /// # Returns
    /// HardwareInfo with aggregated hardware stats
    pub async fn get_hardware_info(&mut self, id: Option<i32>, pc_type: &str, xhd: Option<&str>) -> HeResult<HardwareInfo> {
        let db = self.db_pool.as_ref().unwrap();
        let user_id = id.unwrap_or(0); // TODO: Get from session
        let is_npc = if pc_type == "NPC" { 1 } else { 0 };

        let row = sqlx::query(
            "SELECT 
                COUNT(*) AS total, 
                SUM(cpu) AS cpu, 
                SUM(hdd) AS hdd,  
                SUM(ram) AS ram, 
                net
            FROM hardware
            WHERE hardware.userID = $1 AND isNPC = $2"
        )
        .bind(user_id)
        .bind(is_npc)
        .fetch_one(db)
        .await?;

        let total_pcs: i64 = row.get("total");
        if total_pcs == 0 {
            return Err(crate::error::HeError::ValidationError(
                "Error: hardware is not registered".to_string()
            ));
        }

        let cpu: i32 = row.get("cpu");
        let hdd: i32 = row.get("hdd");
        let ram: i32 = row.get("ram");
        let net: i32 = row.get("net");

        // Get external hard drive info
        let xhd_row = sqlx::query(
            "SELECT SUM(size) AS xhd FROM hardware_external WHERE userID = $1"
        )
        .bind(user_id)
        .fetch_one(db)
        .await?;

        let xhd: Option<i32> = xhd_row.get("xhd");
        let total_xhd = xhd.unwrap_or(0);

        // Update instance variables
        self.total_pcs = Some(total_pcs as i32);
        self.cpu = Some(cpu);
        self.hdd = Some(hdd);
        self.ram = Some(ram);
        self.net = Some(net);
        self.xhd = Some(total_xhd);

        Ok(HardwareInfo {
            cpu,
            hdd,
            ram,
            net,
            xhd: total_xhd,
        })
    }

    /// Gets network specification for a user
    /// 
    /// # Arguments
    /// * `id` - User ID
    /// * `pc_type` - PC type ("VPC" or "NPC")
    /// 
    /// # Returns
    /// Network speed in Mbps
    pub async fn get_net_spec(&self, id: i32, pc_type: &str) -> HeResult<i32> {
        let db = self.db_pool.as_ref().unwrap();
        let is_npc = if pc_type == "NPC" { 1 } else { 0 };

        let row = sqlx::query("SELECT net FROM hardware WHERE userID = $1 AND isNPC = $2 LIMIT 1")
            .bind(id)
            .bind(is_npc)
            .fetch_one(db)
            .await?;

        Ok(row.get("net"))
    }

    /// Gets internet rates for a given speed
    /// 
    /// # Arguments
    /// * `speed` - Network speed
    /// 
    /// # Returns
    /// Internet rate information
    pub fn get_internet_rates(&self, speed: i32) -> HeResult<HashMap<String, i32>> {
        // TODO: Implement internet rate calculation logic
        let mut rates = HashMap::new();
        rates.insert("upload".to_string(), speed);
        rates.insert("download".to_string(), speed);
        Ok(rates)
    }

    /// Gets total number of PCs for a user
    /// 
    /// # Arguments
    /// * `id` - User ID (optional)
    /// * `pc_type` - PC type ("VPC" or "NPC", optional)
    /// 
    /// # Returns
    /// Total number of PCs
    pub async fn get_total_pcs(&self, id: Option<i32>, pc_type: Option<&str>) -> HeResult<i32> {
        let db = self.db_pool.as_ref().unwrap();
        let user_id = id.unwrap_or(0);
        let is_npc = match pc_type {
            Some("NPC") => 1,
            _ => 0,
        };

        let row = sqlx::query("SELECT COUNT(*) AS total FROM hardware WHERE userID = $1 AND isNPC = $2")
            .bind(user_id)
            .bind(is_npc)
            .fetch_one(db)
            .await?;

        Ok(row.get::<i64, _>("total") as i32)
    }

    /// Gets PC specification by server ID
    /// 
    /// # Arguments
    /// * `server_id` - Server ID (optional)
    /// * `pc_type` - PC type
    /// * `id` - User ID
    /// * `unknown_id` - Unknown ID parameter (optional)
    /// 
    /// # Returns
    /// PCSpec with detailed PC information
    pub async fn get_pc_spec(&self, server_id: Option<i32>, pc_type: &str, id: i32, unknown_id: Option<&str>) -> HeResult<PCSpec> {
        let db = self.db_pool.as_ref().unwrap();
        let is_npc = if pc_type == "NPC" { 1 } else { 0 };

        let mut query = "SELECT id, serverName, cpu, hdd, ram, net, userID FROM hardware WHERE userID = $1 AND isNPC = $2".to_string();
        let mut params: Vec<Box<dyn sqlx::Encode<'_, Postgres> + Send>> = vec![
            Box::new(id),
            Box::new(is_npc),
        ];

        if let Some(sid) = server_id {
            query.push_str(" AND id = $3");
            params.push(Box::new(sid));
        }

        query.push_str(" LIMIT 1");

        // For now, use a simpler query approach
        let row = if let Some(sid) = server_id {
            sqlx::query("SELECT id, serverName, cpu, hdd, ram, net, userID FROM hardware WHERE userID = $1 AND isNPC = $2 AND id = $3 LIMIT 1")
                .bind(id)
                .bind(is_npc)
                .bind(sid)
                .fetch_one(db)
                .await?
        } else {
            sqlx::query("SELECT id, serverName, cpu, hdd, ram, net, userID FROM hardware WHERE userID = $1 AND isNPC = $2 LIMIT 1")
                .bind(id)
                .bind(is_npc)
                .fetch_one(db)
                .await?
        };

        Ok(PCSpec {
            id: row.get("id"),
            name: row.get("serverName"),
            cpu: row.get("cpu"),
            hdd: row.get("hdd"),
            ram: row.get("ram"),
            net: row.get("net"),
            user_id: row.get("userID"),
            is_npc: is_npc == 1,
        })
    }

    /// Calculates RAM usage for a user
    /// 
    /// # Arguments
    /// * `id` - User ID
    /// * `pc_type` - PC type
    /// 
    /// # Returns
    /// RAM usage in MB
    pub async fn calculate_ram_usage(&self, id: i32, pc_type: &str) -> HeResult<i32> {
        let db = self.db_pool.as_ref().unwrap();
        let is_npc = if pc_type == "NPC" { 1 } else { 0 };

        let row = sqlx::query(
            "SELECT SUM(softRam) AS ram_usage FROM software WHERE userID = $1 AND isNPC = $2 AND softInstalled = 1"
        )
        .bind(id)
        .bind(is_npc)
        .fetch_one(db)
        .await?;

        let ram_usage: Option<i32> = row.get("ram_usage");
        Ok(ram_usage.unwrap_or(0))
    }

    /// Gets software RAM usage
    /// 
    /// # Arguments
    /// * `id` - User ID
    /// * `pc_type` - PC type
    /// * `soft_id` - Software ID
    /// 
    /// # Returns
    /// Software RAM usage in MB
    pub async fn get_soft_usage(&self, id: i32, pc_type: &str, soft_id: i32) -> HeResult<i32> {
        let db = self.db_pool.as_ref().unwrap();
        let is_npc = if pc_type == "NPC" { 1 } else { 0 };

        let row = sqlx::query(
            "SELECT softRam FROM software WHERE id = $1 AND userID = $2 AND isNPC = $3 LIMIT 1"
        )
        .bind(soft_id)
        .bind(id)
        .bind(is_npc)
        .fetch_one(db)
        .await?;

        Ok(row.get("softRam"))
    }

    /// Calculates HDD usage for a user
    /// 
    /// # Arguments
    /// * `id` - User ID
    /// * `pc_type` - PC type
    /// 
    /// # Returns
    /// HDD usage in MB
    pub async fn calculate_hdd_usage(&self, id: i32, pc_type: &str) -> HeResult<i32> {
        let db = self.db_pool.as_ref().unwrap();
        let is_npc = if pc_type == "NPC" { 1 } else { 0 };

        let row = sqlx::query(
            "SELECT SUM(softSize) AS hdd_usage FROM software WHERE userID = $1 AND isNPC = $2"
        )
        .bind(id)
        .bind(is_npc)
        .fetch_one(db)
        .await?;

        let hdd_usage: Option<i32> = row.get("hdd_usage");
        Ok(hdd_usage.unwrap_or(0))
    }

    /// Handles POST requests for hardware operations
    /// 
    /// # Arguments
    /// * `post_data` - POST data containing action parameters
    /// 
    /// # Returns
    /// Result with redirect URL or error
    pub async fn handle_post(&self, post_data: HashMap<String, String>) -> HeResult<String> {
        let action = post_data.get("act")
            .ok_or_else(|| crate::error::HeError::ValidationError("Invalid POST data.".to_string()))?;

        match action.as_str() {
            "buy" => {
                // TODO: Implement hardware purchase logic
                Ok("hardware".to_string())
            },
            "upgrade" => {
                // TODO: Implement hardware upgrade logic
                Ok("hardware".to_string())
            },
            "rename" => {
                let server_id = post_data.get("server_id")
                    .ok_or_else(|| crate::error::HeError::ValidationError("Missing server ID.".to_string()))?
                    .parse::<i32>()
                    .map_err(|_| crate::error::HeError::ValidationError("Invalid server ID.".to_string()))?;
                
                let server_name = post_data.get("server_name")
                    .ok_or_else(|| crate::error::HeError::ValidationError("Missing server name.".to_string()))?;
                
                self.rename_server(server_id, server_name).await?;
                Ok("hardware".to_string())
            },
            _ => Err(crate::error::HeError::ValidationError("Invalid POST data.".to_string())),
        }
    }

    /// Renames a server
    /// 
    /// # Arguments
    /// * `server_id` - Server ID to rename
    /// * `server_name` - New server name
    pub async fn rename_server(&self, server_id: i32, server_name: &str) -> HeResult<()> {
        let db = self.db_pool.as_ref().unwrap();

        sqlx::query("UPDATE hardware SET serverName = $1 WHERE id = $2")
            .bind(server_name)
            .bind(server_id)
            .execute(db)
            .await?;

        Ok(())
    }

    /// Renames an external hard drive
    /// 
    /// # Arguments
    /// * `server_id` - Server ID
    /// * `server_name` - New name
    pub async fn rename_xhd(&self, server_id: i32, server_name: &str) -> HeResult<()> {
        let db = self.db_pool.as_ref().unwrap();

        sqlx::query("UPDATE hardware_external SET name = $1 WHERE id = $2")
            .bind(server_name)
            .bind(server_id)
            .execute(db)
            .await?;

        Ok(())
    }

    /// Gets PC pricing information
    /// 
    /// # Arguments
    /// * `clan` - Whether this is for a clan (optional)
    /// 
    /// # Returns
    /// Pricing information
    pub fn get_pc_price(&self, clan: Option<bool>) -> HeResult<HashMap<String, i32>> {
        // TODO: Implement PC pricing logic
        let mut prices = HashMap::new();
        let base_price = if clan.unwrap_or(false) { 15000 } else { 10000 };
        
        prices.insert("cpu".to_string(), base_price);
        prices.insert("hdd".to_string(), base_price / 2);
        prices.insert("ram".to_string(), base_price / 4);
        prices.insert("net".to_string(), base_price * 2);
        
        Ok(prices)
    }

    /// Gets external hard drive pricing
    /// 
    /// # Arguments
    /// * `total` - Total capacity (optional)
    /// 
    /// # Returns
    /// XHD price
    pub fn get_xhd_price(&self, total: Option<i32>) -> HeResult<i32> {
        let capacity = total.unwrap_or(1000); // Default 1GB
        let price = (capacity as f64 * 0.5) as i32; // $0.5 per MB
        Ok(price.max(100)) // Minimum $100
    }

    /// Gets external hard drive usage
    /// 
    /// # Returns
    /// XHD usage in MB
    pub async fn get_xhd_usage(&self) -> HeResult<i32> {
        // TODO: Implement XHD usage calculation
        Ok(0)
    }

    /// Gets external hard drive information
    /// 
    /// # Returns
    /// XHD information
    pub async fn get_xhd_info(&self) -> HeResult<HashMap<String, i32>> {
        let db = self.db_pool.as_ref().unwrap();
        let user_id = self.id.unwrap_or(0);

        let row = sqlx::query("SELECT SUM(size) AS total_size FROM hardware_external WHERE userID = $1")
            .bind(user_id)
            .fetch_one(db)
            .await?;

        let mut info = HashMap::new();
        let total_size: Option<i32> = row.get("total_size");
        info.insert("total_size".to_string(), total_size.unwrap_or(0));
        info.insert("usage".to_string(), self.get_xhd_usage().await?);

        Ok(info)
    }

    /// Gets external hard drive by server ID
    /// 
    /// # Arguments
    /// * `server_id` - Server ID
    /// 
    /// # Returns
    /// XHD information
    pub async fn get_xhd(&self, server_id: i32) -> HeResult<HashMap<String, String>> {
        let db = self.db_pool.as_ref().unwrap();

        let row = sqlx::query("SELECT * FROM hardware_external WHERE id = $1 LIMIT 1")
            .bind(server_id)
            .fetch_one(db)
            .await?;

        let mut info = HashMap::new();
        info.insert("id".to_string(), row.get::<i32, _>("id").to_string());
        info.insert("name".to_string(), row.get("name"));
        info.insert("size".to_string(), row.get::<i32, _>("size").to_string());

        Ok(info)
    }

    /// Checks if software is installed
    /// 
    /// # Arguments
    /// * `soft_id` - Software ID
    /// * `uid` - User ID
    /// * `pc_type` - PC type
    /// 
    /// # Returns
    /// True if software is installed
    pub async fn is_installed(&self, soft_id: i32, uid: i32, pc_type: &str) -> HeResult<bool> {
        let db = self.db_pool.as_ref().unwrap();
        let is_npc = if pc_type == "NPC" { 1 } else { 0 };

        let row = sqlx::query(
            "SELECT COUNT(*) AS total FROM software WHERE id = $1 AND userID = $2 AND isNPC = $3 AND softInstalled = 1 LIMIT 1"
        )
        .bind(soft_id)
        .bind(uid)
        .bind(is_npc)
        .fetch_one(db)
        .await?;

        let total: i64 = row.get("total");
        Ok(total > 0)
    }

    /// Gets software RAM usage for installed software
    /// 
    /// # Arguments
    /// * `soft_id` - Software ID
    /// * `uid` - User ID
    /// * `pc_type` - PC type
    /// 
    /// # Returns
    /// RAM usage in MB
    pub async fn soft_ram_usage(&self, soft_id: i32, uid: i32, pc_type: &str) -> HeResult<i32> {
        let db = self.db_pool.as_ref().unwrap();
        let is_npc = if pc_type == "NPC" { 1 } else { 0 };

        let row = sqlx::query(
            "SELECT softRam FROM software WHERE id = $1 AND userID = $2 AND isNPC = $3 AND softInstalled = 1 LIMIT 1"
        )
        .bind(soft_id)
        .bind(uid)
        .bind(is_npc)
        .fetch_one(db)
        .await?;

        Ok(row.get("softRam"))
    }

    // TODO: Add remaining methods for:
    // - Software management (show_software, get_software, etc.)
    // - Virus operations (doom_*, virus operations)
    // - DDoS attacks and mitigation
    // - Log management
    // - Research operations
    // - External software management
    // - Text file operations
    // - Folder operations
    // 
    // These methods contain complex business logic that would need detailed
    // analysis and porting from the original PHP implementation.
}