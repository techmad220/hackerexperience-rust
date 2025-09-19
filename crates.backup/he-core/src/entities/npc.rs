use async_trait::async_trait;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::net::Ipv4Addr;

use crate::database::Database;
use crate::entities::session::Session;
use crate::error::Result;

/// Represents an NPC (Non-Player Character) in the game
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Npc {
    pub id: i32,
    pub npc_type: i32,
    pub npc_ip: i64,
    pub npc_pass: String,
    pub down_until: Option<chrono::NaiveDateTime>,
}

/// NPC information with localized name and web data
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NpcInfo {
    pub npc_ip: i64,
    pub npc_type: i32,
    pub npc_web: Option<String>,
    pub npc_pass: String,
    pub name: String,
}

/// Information about an NPC retrieved by key
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NpcByKey {
    pub id: i32,
    pub npc_ip: i64,
}

#[async_trait]
pub trait NpcRepository {
    /// Check if an NPC exists by ID
    async fn is_npc_exists(&self, nid: i32) -> Result<bool>;
    
    /// Get NPC information with localized content
    async fn get_npc_info(&self, nid: i32, lang: Option<&str>) -> Result<Option<NpcInfo>>;
    
    /// Check if an NPC is currently down
    async fn is_npc_down(&self, nid: i32) -> Result<bool>;
    
    /// Generate a random string for NPC passwords
    fn generate_random_string(&self, length: usize) -> String;
    
    /// Generate a new NPC with specified type
    async fn generate_npc(&self, npc_type: i32) -> Result<i32>;
    
    /// Get NPC by key identifier
    async fn get_npc_by_key(&self, key: &str) -> Result<Option<NpcByKey>>;
}

pub struct NpcService {
    db: PgPool,
}

impl NpcService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
    
    /// Convert IPv4 address to long integer representation
    fn ip_to_long(ip: Ipv4Addr) -> i64 {
        let octets = ip.octets();
        ((octets[0] as i64) << 24) +
        ((octets[1] as i64) << 16) +
        ((octets[2] as i64) << 8) +
        (octets[3] as i64)
    }
    
    /// Convert long integer to IPv4 address
    fn long_to_ip(ip_long: i64) -> Ipv4Addr {
        Ipv4Addr::new(
            ((ip_long >> 24) & 0xFF) as u8,
            ((ip_long >> 16) & 0xFF) as u8,
            ((ip_long >> 8) & 0xFF) as u8,
            (ip_long & 0xFF) as u8,
        )
    }
}

#[async_trait]
impl NpcRepository for NpcService {
    /// Check if an NPC exists by ID
    async fn is_npc_exists(&self, nid: i32) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM npc WHERE id = $1",
            nid
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) > 0)
    }
    
    /// Get NPC information with localized content
    async fn get_npc_info(&self, nid: i32, lang: Option<&str>) -> Result<Option<NpcInfo>> {
        if !self.is_npc_exists(nid).await? {
            return Err(crate::error::Error::InvalidId("Invalid NPC ID".to_string()));
        }
        
        let table = match lang {
            Some("pt") | Some("br") => "npc_info_pt",
            _ => "npc_info_en",
        };
        
        let query = format!(
            "SELECT npc.npc_ip, npc.npc_type, {}.web AS npc_web, npc.npc_pass, {}.name
             FROM npc
             LEFT JOIN {} ON {}.npc_id = npc.id
             WHERE npc.id = $1
             LIMIT 1",
            table, table, table, table
        );
        
        let npc_info = sqlx::query_as::<_, NpcInfo>(&query)
            .bind(nid)
            .fetch_optional(&self.db)
            .await?;
            
        Ok(npc_info)
    }
    
    /// Check if an NPC is currently down
    async fn is_npc_down(&self, nid: i32) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM npc_down WHERE npc_id = $1",
            nid
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) > 0)
    }
    
    /// Generate a random string for NPC passwords
    fn generate_random_string(&self, length: usize) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
    
    /// Generate a new NPC with specified type
    async fn generate_npc(&self, npc_type: i32) -> Result<i32> {
        let mut rng = rand::thread_rng();
        
        // Generate random IP address
        let ip1 = rng.gen_range(0..=255);
        let ip2 = rng.gen_range(0..=255);
        let ip3 = rng.gen_range(0..=255);
        let ip4 = rng.gen_range(0..=255);
        
        let game_ip = Ipv4Addr::new(ip1, ip2, ip3, ip4);
        let gen_ip = Self::ip_to_long(game_ip);
        
        // Generate random password
        let password = self.generate_random_string(8);
        
        // Insert NPC
        let npc_id = sqlx::query_scalar!(
            "INSERT INTO npc (npc_type, npc_ip, npc_pass, down_until)
             VALUES ($1, $2, $3, NULL)
             RETURNING id",
            npc_type,
            gen_ip,
            password
        )
        .fetch_one(&self.db)
        .await?;
        
        // Insert NPC info for both languages
        sqlx::query!(
            "INSERT INTO npc_info_en (npc_id, name, web)
             VALUES ($1, 'Initech Corp', '')",
            npc_id
        )
        .execute(&self.db)
        .await?;
        
        sqlx::query!(
            "INSERT INTO npc_info_pt (npc_id, name, web)
             VALUES ($1, 'Initech Corp', '')",
            npc_id
        )
        .execute(&self.db)
        .await?;
        
        // Insert hardware for NPC
        sqlx::query!(
            "INSERT INTO hardware (user_id, cpu, net, is_npc)
             VALUES ($1, 1500, 10, true)",
            npc_id
        )
        .execute(&self.db)
        .await?;
        
        // Insert log entry for NPC
        sqlx::query!(
            "INSERT INTO log (user_id, text, is_npc)
             VALUES ($1, '', true)",
            npc_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(npc_id)
    }
    
    /// Get NPC by key identifier
    async fn get_npc_by_key(&self, key: &str) -> Result<Option<NpcByKey>> {
        let npc = sqlx::query_as!(
            NpcByKey,
            "SELECT npc.id, npc.npc_ip
             FROM npc
             INNER JOIN npc_key ON npc.id = npc_key.npc_id
             WHERE npc_key.key = $1
             LIMIT 1",
            key
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(npc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ip_conversion() {
        let ip = Ipv4Addr::new(192, 168, 1, 1);
        let ip_long = NpcService::ip_to_long(ip);
        let converted_back = NpcService::long_to_ip(ip_long);
        
        assert_eq!(ip, converted_back);
    }
    
    #[tokio::test]
    async fn test_random_string_generation() {
        let db = PgPool::connect("postgresql://test").await.unwrap_or_else(|_| {
            // For testing without a real database connection
            panic!("Database connection failed")
        });
        let npc_service = NpcService::new(db);
        let random_str = npc_service.generate_random_string(8);
        
        assert_eq!(random_str.len(), 8);
        assert!(random_str.chars().all(|c| c.is_alphanumeric()));
    }
}