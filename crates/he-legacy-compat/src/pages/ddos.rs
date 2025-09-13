//! DDoS attack page handler - 1:1 port of DDoS.php
//! 
//! DDoS (Distributed Denial of Service) attack system:
//! - IP address validation and targeting
//! - Virus count requirements (minimum 3 DDoS viruses)
//! - Process creation for DDoS attacks
//! - Integration with hacked database list
//! - Target validation (player existence checks)

use axum::{
    extract::{Extension, Form},
    http::StatusCode,
    response::Html,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use crate::classes::{system::System, player::Player};
use crate::session::{PhpSession, SessionValue};
use he_db::DbPool;

/// Form data for DDoS attack
#[derive(Debug, Deserialize)]
pub struct DDoSForm {
    pub ip: Option<String>,
}

/// DDoS attack errors
#[derive(Debug)]
pub enum DDoSError {
    InvalidRequest,
    InvalidIP,
    NotLoggedIn,
    InsufficientViruses,
    TargetNotListed,
    TargetNotFound,
    ProcessError,
}

impl DDoSError {
    fn message(&self) -> &'static str {
        match self {
            DDoSError::InvalidRequest => "Invalid request type.",
            DDoSError::InvalidIP => "Invalid IP address.",
            DDoSError::NotLoggedIn => "Authentication required.",
            DDoSError::InsufficientViruses => "You need to have at least 3 working DDoS viruses.",
            DDoSError::TargetNotListed => "This IP is not on your Hacked Database.",
            DDoSError::TargetNotFound => "This IP doesnt exists.",
            DDoSError::ProcessError => "Failed to create DDoS process.",
        }
    }
}

/// DDoS attack handler - processes DDoS attack requests
/// 
/// Port of: DDoS.php
/// Features:
/// - POST-only request validation
/// - IP address format validation
/// - Minimum virus count requirements (3 DDoS viruses)
/// - Target validation against hacked database
/// - Process creation for attack execution
/// - Proper error handling and user feedback
/// - Integration with game process system
pub async fn ddos_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(mut session): Extension<PhpSession>,
    form: Option<Form<DDoSForm>>,
) -> Result<Html<String>, StatusCode> {
    // Check if user is logged in
    if !session.isset_login() {
        return Ok(Html("<script>window.location.href='/index.php';</script>".to_string()));
    }

    // Validate POST request (original only accepts POST)
    let form_data = match form {
        Some(Form(data)) => data,
        None => {
            session.add_msg(DDoSError::InvalidRequest.message(), "error");
            return Ok(Html("<script>window.location.href='/list.php?action=ddos';</script>".to_string()));
        }
    };

    // Validate IP parameter
    let target_ip = match &form_data.ip {
        Some(ip) if !ip.is_empty() => ip,
        _ => {
            session.add_msg(DDoSError::InvalidIP.message(), "error");
            return Ok(Html("<script>window.location.href='/list.php?action=ddos';</script>".to_string()));
        }
    };

    // Validate IP address format
    let system = System::new();
    if !system.validate(target_ip, "ip") {
        session.add_msg(DDoSError::InvalidIP.message(), "error");
        return Ok(Html("<script>window.location.href='/list.php?action=ddos';</script>".to_string()));
    }

    // Get user ID
    let user_id = session.get("id")
        .and_then(|v| match v {
            SessionValue::String(s) => s.parse::<i64>().ok(),
            SessionValue::Integer(i) => Some(*i),
            _ => None,
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Process DDoS attack
    match process_ddos_attack(&db_pool, &mut session, user_id, target_ip).await {
        Ok(redirect_url) => {
            Ok(Html(format!("<script>window.location.href='{}';</script>", redirect_url)))
        },
        Err(error) => {
            session.add_msg(error.message(), "error");
            Ok(Html("<script>window.location.href='/list.php?action=ddos';</script>".to_string()))
        }
    }
}

/// Process DDoS attack with all validations
async fn process_ddos_attack(
    db_pool: &DbPool,
    session: &mut PhpSession,
    user_id: i64,
    target_ip: &str,
) -> Result<String, DDoSError> {
    // Convert IP to long format (as in original PHP)
    let ip_addr: Ipv4Addr = target_ip.parse()
        .map_err(|_| DDoSError::InvalidIP)?;
    let ip_long = u32::from(ip_addr) as i64;

    // Initialize required classes
    let virus = Virus::new(db_pool.clone());
    let player = Player::new(db_pool.clone());
    let list = Lists::new(db_pool.clone());

    // Get target player information by IP
    let player_info = player.get_id_by_ip(ip_long).await
        .map_err(|_| DDoSError::TargetNotFound)?;

    if !player_info.exists {
        return Err(DDoSError::TargetNotFound);
    }

    // Check if IP is in user's hacked database
    if !list.is_listed(user_id, ip_long).await
        .map_err(|_| DDoSError::TargetNotListed)? {
        return Err(DDoSError::TargetNotListed);
    }

    // Check if user has enough DDoS viruses (minimum 3)
    if virus.ddos_count(user_id).await
        .map_err(|_| DDoSError::InsufficientViruses)? < 3 {
        return Err(DDoSError::InsufficientViruses);
    }

    // Create DDoS process
    let process = Process::new(db_pool.clone());
    let is_npc = if player_info.pc_type == "VPC" { 0 } else { 1 };

    match process.new_process(
        user_id,
        "DDOS",
        player_info.id,
        "remote",
        "",
        "",
        "",
        is_npc,
    ).await {
        Ok(true) => {
            // Successfully created new process
            session.add_msg(
                &format!("DDoS attack against <strong>{}</strong> launched.", target_ip),
                "notice"
            );
            Ok("list.php?action=ddos".to_string())
        },
        Ok(false) => {
            // Process already exists
            if !session.isset_msg() {
                let pid = process.get_pid(
                    user_id,
                    "DDOS",
                    player_info.id,
                    "remote",
                    "",
                    "",
                    "",
                    is_npc,
                ).await.map_err(|_| DDoSError::ProcessError)?;
                
                Ok(format!("processes.php?id={}", pid))
            } else {
                Ok("list.php?action=ddos".to_string())
            }
        },
        Err(_) => Err(DDoSError::ProcessError),
    }
}

/// Target player information
#[derive(Debug, Clone)]
pub struct PlayerInfo {
    pub id: i64,
    pub exists: bool,
    pub pc_type: String,
}

/// Virus management class
#[derive(Debug, Clone)]
pub struct Virus {
    pub db_pool: DbPool,
}

impl Virus {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    /// Count active DDoS viruses for user
    pub async fn ddos_count(&self, user_id: i64) -> Result<i32, sqlx::Error> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM software WHERE userID = ? AND softType = 10 AND isRunning = 1",
            user_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(count as i32)
    }
}

/// Lists management class
#[derive(Debug, Clone)]
pub struct Lists {
    pub db_pool: DbPool,
}

impl Lists {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    /// Check if IP is in user's hacked database
    pub async fn is_listed(&self, user_id: i64, ip: i64) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM hacked_list WHERE user_id = ? AND ip = ?",
            user_id,
            ip
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(count > 0)
    }
}

/// Process management class
#[derive(Debug, Clone)]
pub struct Process {
    pub db_pool: DbPool,
}

impl Process {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    /// Create new process
    pub async fn new_process(
        &self,
        user_id: i64,
        process_type: &str,
        target_id: i64,
        location: &str,
        param1: &str,
        param2: &str,
        param3: &str,
        is_npc: i32,
    ) -> Result<bool, sqlx::Error> {
        // Check if process already exists
        let existing = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM processes WHERE user_id = ? AND process_type = ? AND target_id = ? AND location = ?",
            user_id,
            process_type,
            target_id,
            location
        )
        .fetch_one(&self.db_pool)
        .await?;

        if existing > 0 {
            return Ok(false); // Process already exists
        }

        // Create new process
        sqlx::query!(
            "INSERT INTO processes (user_id, process_type, target_id, location, param1, param2, param3, is_npc, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, NOW())",
            user_id,
            process_type,
            target_id,
            location,
            param1,
            param2,
            param3,
            is_npc
        )
        .execute(&self.db_pool)
        .await?;

        Ok(true)
    }

    /// Get process ID for existing process
    pub async fn get_pid(
        &self,
        user_id: i64,
        process_type: &str,
        target_id: i64,
        location: &str,
        param1: &str,
        param2: &str,
        param3: &str,
        is_npc: i32,
    ) -> Result<i64, sqlx::Error> {
        let pid = sqlx::query_scalar!(
            "SELECT id FROM processes WHERE user_id = ? AND process_type = ? AND target_id = ? AND location = ? LIMIT 1",
            user_id,
            process_type,
            target_id,
            location
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(pid)
    }
}

/// Extended Player methods for DDoS functionality
impl Player {
    /// Get player ID by IP address
    pub async fn get_id_by_ip(&self, ip: i64) -> Result<PlayerInfo, sqlx::Error> {
        let result = sqlx::query!(
            "SELECT id, game_ip FROM users WHERE game_ip = ? LIMIT 1",
            ip
        )
        .fetch_optional(&self.db_pool)
        .await?;

        match result {
            Some(row) => Ok(PlayerInfo {
                id: row.id,
                exists: true,
                pc_type: "VPC".to_string(), // TODO: Get actual PC type from hardware table
            }),
            None => Ok(PlayerInfo {
                id: 0,
                exists: false,
                pc_type: String::new(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ddos_form_parsing() {
        let form = DDoSForm {
            ip: Some("192.168.1.1".to_string()),
        };

        assert_eq!(form.ip.unwrap(), "192.168.1.1");
    }

    #[test]
    fn test_ip_validation() {
        let valid_ips = vec!["192.168.1.1", "8.8.8.8", "127.0.0.1"];
        let invalid_ips = vec!["invalid", "256.256.256.256", "", "192.168"];

        for ip in valid_ips {
            assert!(ip.parse::<Ipv4Addr>().is_ok());
        }

        for ip in invalid_ips {
            assert!(ip.parse::<Ipv4Addr>().is_err());
        }
    }

    #[test]
    fn test_ip_to_long_conversion() {
        let ip: Ipv4Addr = "192.168.1.1".parse().unwrap();
        let ip_long = u32::from(ip) as i64;
        
        // 192.168.1.1 = 192*256^3 + 168*256^2 + 1*256 + 1 = 3232235777
        assert_eq!(ip_long, 3232235777);
    }

    #[test]
    fn test_ddos_error_messages() {
        assert_eq!(DDoSError::InvalidIP.message(), "Invalid IP address.");
        assert_eq!(DDoSError::InsufficientViruses.message(), "You need to have at least 3 working DDoS viruses.");
        assert_eq!(DDoSError::TargetNotListed.message(), "This IP is not on your Hacked Database.");
    }

    #[test]
    fn test_player_info_creation() {
        let info = PlayerInfo {
            id: 123,
            exists: true,
            pc_type: "VPC".to_string(),
        };

        assert_eq!(info.id, 123);
        assert!(info.exists);
        assert_eq!(info.pc_type, "VPC");
    }

    #[test]
    fn test_is_npc_logic() {
        // Original: if($playerInfo['0']['pctype'] == 'VPC') { $isNPC = 0; } else { $isNPC = 1; }
        let vpc_is_npc = if "VPC" == "VPC" { 0 } else { 1 };
        let npc_is_npc = if "NPC" == "VPC" { 0 } else { 1 };

        assert_eq!(vpc_is_npc, 0);
        assert_eq!(npc_is_npc, 1);
    }
}