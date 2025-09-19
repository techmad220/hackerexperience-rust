// NPC.CLASS.PHP PORT - Non-Player Character management
// Original: NPC system for banks, servers, and other game entities

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use sqlx::MySqlPool;
use serde::{Deserialize, Serialize};
use he_core::*;
use he_db::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcInfo {
    pub id: i64,
    pub npc_ip: u32,
    pub npc_type: i32,
    pub npc_web: Option<String>,
    pub npc_pass: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcDetails {
    pub id: i64,
    pub npc_ip: u32,
    pub npc_type: i32,
    pub name: String,
    pub description: Option<String>,
    pub web_content: Option<String>,
    pub password: String,
    pub is_down: bool,
    pub down_until: Option<DateTime<Utc>>,
    pub created_date: DateTime<Utc>,
}

pub struct Npc {
    db_pool: MySqlPool,
}

impl Npc {
    pub fn new(db_pool: MySqlPool) -> Self {
        Self { db_pool }
    }
    
    // Original PHP: issetNPC - Check if NPC exists
    pub async fn npc_exists(&self, npc_id: i64) -> Result<bool, NpcError> {
        if npc_id <= 0 {
            return Ok(false);
        }
        
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM npc WHERE id = ? LIMIT 1"
        )
        .bind(npc_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(NpcError::DatabaseError)?;
        
        Ok(count == 1)
    }
    
    // Original PHP: getNPCInfo - Get NPC information with localization
    pub async fn get_npc_info(&self, npc_id: i64, language: Option<String>) -> Result<NpcInfo, NpcError> {
        if !self.npc_exists(npc_id).await? {
            return Err(NpcError::InvalidId(npc_id));
        }
        
        let lang = language.unwrap_or_else(|| "en".to_string());
        let info_table = if lang == "pt" || lang == "br" {
            "npc_info_pt"
        } else {
            "npc_info_en"
        };
        
        let query = format!(
            "SELECT npc.id, npc.npcIP as npc_ip, npc.npcType as npc_type, 
                    {}.web as npc_web, npc.npcPass as npc_pass, {}.name
             FROM npc
             LEFT JOIN {} ON {}.npcID = npc.id
             WHERE npc.id = ? 
             LIMIT 1",
            info_table, info_table, info_table, info_table
        );
        
        let npc_info = sqlx::query_as::<_, NpcInfo>(&query)
            .bind(npc_id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(NpcError::DatabaseError)?;
        
        npc_info.ok_or(NpcError::NotFound(npc_id))
    }
    
    // Original PHP: downNPC - Check if NPC is down
    pub async fn is_npc_down(&self, npc_id: i64) -> Result<bool, NpcError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM npc_down WHERE npcID = ? AND down_until > NOW() LIMIT 1"
        )
        .bind(npc_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(NpcError::DatabaseError)?;
        
        Ok(count == 1)
    }
    
    // Original PHP: randString - Generate random string (utility function)
    pub fn rand_string(&self, length: usize, charset: Option<&str>) -> String {
        use rand::Rng;
        
        let default_charset = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let chars: Vec<char> = charset.unwrap_or(default_charset).chars().collect();
        let mut rng = rand::thread_rng();
        
        (0..length)
            .map(|_| chars[rng.gen_range(0..chars.len())])
            .collect()
    }
    
    // Original PHP: generateNPC - Generate new NPC
    pub async fn generate_npc(&self, npc_type: i32) -> Result<i64, NpcError> {
        // Generate random IP
        let ip_parts = (0..4)
            .map(|_| rand::random::<u8>())
            .collect::<Vec<_>>();
        let game_ip = format!("{}.{}.{}.{}", ip_parts[0], ip_parts[1], ip_parts[2], ip_parts[3]);
        let game_ip_long = crate::classes::system::System::ip_to_long(&game_ip)
            .map_err(|_| NpcError::InvalidIp(game_ip.clone()))?;
        
        // Generate random password
        let password = self.rand_string(12, None);
        
        // Generate name based on type
        let name = self.generate_npc_name(npc_type);
        
        // Insert NPC
        let npc_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO npc (npcIP, npcType, npcPass, created_date) 
             VALUES (?, ?, ?, NOW()) 
             RETURNING id"
        )
        .bind(game_ip_long)
        .bind(npc_type)
        .bind(&password)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(NpcError::DatabaseError)?;
        
        let npc_id = npc_id.ok_or(NpcError::CreationFailed)?;
        
        // Insert localized info
        self.create_npc_info(npc_id, &name, npc_type).await?;
        
        Ok(npc_id)
    }
    
    // Original PHP: getNPCByKey - Get NPC by key/identifier
    pub async fn get_npc_by_key(&self, key: String) -> Result<Option<NpcInfo>, NpcError> {
        let npc_info = sqlx::query_as::<_, NpcInfo>(
            "SELECT n.id, n.npcIP as npc_ip, n.npcType as npc_type, 
                    nie.web as npc_web, n.npcPass as npc_pass, nie.name
             FROM npc n
             LEFT JOIN npc_info_en nie ON nie.npcID = n.id
             WHERE n.npcPass = ? OR nie.name = ?
             LIMIT 1"
        )
        .bind(&key)
        .bind(&key)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(NpcError::DatabaseError)?;
        
        Ok(npc_info)
    }
    
    // Get all NPCs of a specific type
    pub async fn get_npcs_by_type(&self, npc_type: i32) -> Result<Vec<NpcInfo>, NpcError> {
        let npcs = sqlx::query_as::<_, NpcInfo>(
            "SELECT n.id, n.npcIP as npc_ip, n.npcType as npc_type, 
                    nie.web as npc_web, n.npcPass as npc_pass, nie.name
             FROM npc n
             LEFT JOIN npc_info_en nie ON nie.npcID = n.id
             WHERE n.npcType = ?
             ORDER BY n.id"
        )
        .bind(npc_type)
        .fetch_all(&self.db_pool)
        .await
        .map_err(NpcError::DatabaseError)?;
        
        Ok(npcs)
    }
    
    // Set NPC as down for specified duration
    pub async fn set_npc_down(&self, npc_id: i64, hours: i32, reason: Option<String>) -> Result<(), NpcError> {
        let reason_text = reason.unwrap_or_else(|| "System maintenance".to_string());
        
        sqlx::query(
            "INSERT INTO npc_down (npcID, down_until, reason, down_since) 
             VALUES (?, DATE_ADD(NOW(), INTERVAL ? HOUR), ?, NOW())
             ON DUPLICATE KEY UPDATE 
             down_until = DATE_ADD(NOW(), INTERVAL ? HOUR), 
             reason = ?"
        )
        .bind(npc_id)
        .bind(hours)
        .bind(&reason_text)
        .bind(hours)
        .bind(&reason_text)
        .execute(&self.db_pool)
        .await
        .map_err(NpcError::DatabaseError)?;
        
        Ok(())
    }
    
    // Remove NPC from down status
    pub async fn set_npc_up(&self, npc_id: i64) -> Result<(), NpcError> {
        sqlx::query("DELETE FROM npc_down WHERE npcID = ?")
            .bind(npc_id)
            .execute(&self.db_pool)
            .await
            .map_err(NpcError::DatabaseError)?;
        
        Ok(())
    }
    
    // Get detailed NPC information
    pub async fn get_npc_details(&self, npc_id: i64, language: Option<String>) -> Result<NpcDetails, NpcError> {
        let lang = language.unwrap_or_else(|| "en".to_string());
        let info_table = if lang == "pt" || lang == "br" {
            "npc_info_pt"
        } else {
            "npc_info_en"
        };
        
        let query = format!(
            "SELECT n.id, n.npcIP as npc_ip, n.npcType as npc_type, 
                    ni.name, ni.description, ni.web as web_content,
                    n.npcPass as password, n.created_date,
                    CASE WHEN nd.npcID IS NOT NULL THEN TRUE ELSE FALSE END as is_down,
                    nd.down_until
             FROM npc n
             LEFT JOIN {} ni ON ni.npcID = n.id
             LEFT JOIN npc_down nd ON nd.npcID = n.id AND nd.down_until > NOW()
             WHERE n.id = ?
             LIMIT 1",
            info_table
        );
        
        let details = sqlx::query_as::<_, NpcDetails>(&query)
            .bind(npc_id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(NpcError::DatabaseError)?;
        
        details.ok_or(NpcError::NotFound(npc_id))
    }
    
    // Get all available NPCs (not down)
    pub async fn get_available_npcs(&self, npc_type: Option<i32>) -> Result<Vec<NpcInfo>, NpcError> {
        let query = if let Some(t) = npc_type {
            sqlx::query_as::<_, NpcInfo>(
                "SELECT n.id, n.npcIP as npc_ip, n.npcType as npc_type, 
                        nie.web as npc_web, n.npcPass as npc_pass, nie.name
                 FROM npc n
                 LEFT JOIN npc_info_en nie ON nie.npcID = n.id
                 LEFT JOIN npc_down nd ON nd.npcID = n.id AND nd.down_until > NOW()
                 WHERE n.npcType = ? AND nd.npcID IS NULL
                 ORDER BY n.id"
            )
            .bind(t)
        } else {
            sqlx::query_as::<_, NpcInfo>(
                "SELECT n.id, n.npcIP as npc_ip, n.npcType as npc_type, 
                        nie.web as npc_web, n.npcPass as npc_pass, nie.name
                 FROM npc n
                 LEFT JOIN npc_info_en nie ON nie.npcID = n.id
                 LEFT JOIN npc_down nd ON nd.npcID = n.id AND nd.down_until > NOW()
                 WHERE nd.npcID IS NULL
                 ORDER BY n.id"
            )
        };
        
        let npcs = query
            .fetch_all(&self.db_pool)
            .await
            .map_err(NpcError::DatabaseError)?;
        
        Ok(npcs)
    }
    
    // Delete NPC and all associated data
    pub async fn delete_npc(&self, npc_id: i64) -> Result<(), NpcError> {
        let mut tx = self.db_pool.begin().await.map_err(NpcError::DatabaseError)?;
        
        // Delete associated data
        sqlx::query("DELETE FROM npc_down WHERE npcID = ?")
            .bind(npc_id)
            .execute(&mut *tx)
            .await
            .map_err(NpcError::DatabaseError)?;
        
        sqlx::query("DELETE FROM npc_info_en WHERE npcID = ?")
            .bind(npc_id)
            .execute(&mut *tx)
            .await
            .map_err(NpcError::DatabaseError)?;
        
        sqlx::query("DELETE FROM npc_info_pt WHERE npcID = ?")
            .bind(npc_id)
            .execute(&mut *tx)
            .await
            .map_err(NpcError::DatabaseError)?;
        
        // Delete main NPC record
        sqlx::query("DELETE FROM npc WHERE id = ?")
            .bind(npc_id)
            .execute(&mut *tx)
            .await
            .map_err(NpcError::DatabaseError)?;
        
        tx.commit().await.map_err(NpcError::DatabaseError)?;
        
        Ok(())
    }
    
    // Helper methods
    async fn create_npc_info(&self, npc_id: i64, name: &str, npc_type: i32) -> Result<(), NpcError> {
        let (en_name, en_desc, pt_name, pt_desc) = self.get_npc_templates(npc_type, name);
        
        // Insert English info
        sqlx::query(
            "INSERT INTO npc_info_en (npcID, name, description, web) VALUES (?, ?, ?, '')"
        )
        .bind(npc_id)
        .bind(&en_name)
        .bind(&en_desc)
        .execute(&self.db_pool)
        .await
        .map_err(NpcError::DatabaseError)?;
        
        // Insert Portuguese info
        sqlx::query(
            "INSERT INTO npc_info_pt (npcID, name, description, web) VALUES (?, ?, ?, '')"
        )
        .bind(npc_id)
        .bind(&pt_name)
        .bind(&pt_desc)
        .execute(&self.db_pool)
        .await
        .map_err(NpcError::DatabaseError)?;
        
        Ok(())
    }
    
    fn generate_npc_name(&self, npc_type: i32) -> String {
        match npc_type {
            1 => format!("Bank of {}", self.rand_string(6, Some("ABCDEFGHIJKLMNOPQRSTUVWXYZ"))),
            2 => format!("Corp {}", self.rand_string(4, Some("ABCDEFGHIJKLMNOPQRSTUVWXYZ"))),
            3 => format!("University of {}", self.rand_string(8, Some("ABCDEFGHIJKLMNOPQRSTUVWXYZ"))),
            10 => "FBI Server".to_string(),
            20 => "CIA Database".to_string(),
            30 => "NSA Network".to_string(),
            40 => "Bitcoin Exchange".to_string(),
            _ => format!("Server {}", self.rand_string(6, None)),
        }
    }
    
    fn get_npc_templates(&self, npc_type: i32, name: &str) -> (String, String, String, String) {
        match npc_type {
            1 => (
                name.to_string(),
                "A secure banking institution providing financial services.".to_string(),
                name.to_string(),
                "Uma instituição bancária segura fornecendo serviços financeiros.".to_string(),
            ),
            2 => (
                name.to_string(),
                "A corporate server hosting business applications.".to_string(),
                name.to_string(),
                "Um servidor corporativo hospedando aplicações de negócios.".to_string(),
            ),
            3 => (
                name.to_string(),
                "An educational institution's main server.".to_string(),
                name.to_string(),
                "Servidor principal de uma instituição educacional.".to_string(),
            ),
            10 => (
                "FBI Mainframe".to_string(),
                "Federal Bureau of Investigation secure server.".to_string(),
                "Mainframe FBI".to_string(),
                "Servidor seguro do Federal Bureau of Investigation.".to_string(),
            ),
            40 => (
                "Bitcoin Exchange".to_string(),
                "Cryptocurrency trading platform.".to_string(),
                "Corretora Bitcoin".to_string(),
                "Plataforma de negociação de criptomoedas.".to_string(),
            ),
            _ => (
                name.to_string(),
                "A generic server.".to_string(),
                name.to_string(),
                "Um servidor genérico.".to_string(),
            ),
        }
    }
}

// Implement FromRow for database types
impl sqlx::FromRow<'_, sqlx::mysql::MySqlRow> for NpcInfo {
    fn from_row(row: &'_ sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(NpcInfo {
            id: row.try_get("id")?,
            npc_ip: row.try_get("npc_ip")?,
            npc_type: row.try_get("npc_type")?,
            npc_web: row.try_get("npc_web")?,
            npc_pass: row.try_get("npc_pass")?,
            name: row.try_get("name")?,
        })
    }
}

impl sqlx::FromRow<'_, sqlx::mysql::MySqlRow> for NpcDetails {
    fn from_row(row: &'_ sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(NpcDetails {
            id: row.try_get("id")?,
            npc_ip: row.try_get("npc_ip")?,
            npc_type: row.try_get("npc_type")?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            web_content: row.try_get("web_content")?,
            password: row.try_get("password")?,
            is_down: row.try_get("is_down")?,
            down_until: row.try_get("down_until")?,
            created_date: row.try_get("created_date")?,
        })
    }
}

#[derive(Debug)]
pub enum NpcError {
    DatabaseError(sqlx::Error),
    InvalidId(i64),
    NotFound(i64),
    CreationFailed,
    InvalidIp(String),
}

impl std::fmt::Display for NpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NpcError::DatabaseError(e) => write!(f, "Database error: {}", e),
            NpcError::InvalidId(id) => write!(f, "Invalid NPC ID: {}", id),
            NpcError::NotFound(id) => write!(f, "NPC {} not found", id),
            NpcError::CreationFailed => write!(f, "Failed to create NPC"),
            NpcError::InvalidIp(ip) => write!(f, "Invalid IP address: {}", ip),
        }
    }
}

impl std::error::Error for NpcError {}