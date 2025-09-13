use serde_json::Value;
use std::collections::HashMap;

pub struct TestDataGenerator;

impl TestDataGenerator {
    /// Generate test player data with various scenarios
    pub fn players() -> HashMap<&'static str, Value> {
        let mut players = HashMap::new();
        
        players.insert("basic_player", serde_json::json!({
            "username": "testplayer1",
            "email": "test1@example.com",
            "password_hash": "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewDGRQN1WyPPzxhS",
            "registration_date": "2024-01-01T00:00:00Z",
            "last_login": "2024-01-01T12:00:00Z",
            "player_level": 1,
            "experience": 0,
            "money": 1000,
            "reputation": 0,
            "clan_id": null
        }));
        
        players.insert("advanced_player", serde_json::json!({
            "username": "hacker_elite",
            "email": "elite@example.com", 
            "password_hash": "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewDGRQN1WyPPzxhS",
            "registration_date": "2023-01-01T00:00:00Z",
            "last_login": "2024-01-02T08:30:00Z",
            "player_level": 25,
            "experience": 50000,
            "money": 100000,
            "reputation": 2500,
            "clan_id": 1
        }));
        
        players.insert("banned_player", serde_json::json!({
            "username": "cheater",
            "email": "cheater@example.com",
            "password_hash": "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewDGRQN1WyPPzxhS",
            "registration_date": "2024-01-01T00:00:00Z",
            "last_login": "2024-01-01T15:00:00Z",
            "player_level": 10,
            "experience": 10000,
            "money": 50000,
            "reputation": -1000,
            "clan_id": null,
            "banned_until": "2024-12-31T23:59:59Z",
            "ban_reason": "Cheating"
        }));
        
        players
    }
    
    /// Generate test server data
    pub fn servers() -> HashMap<&'static str, Value> {
        let mut servers = HashMap::new();
        
        servers.insert("desktop_server", serde_json::json!({
            "player_id": 1,
            "server_ip": "192.168.1.100",
            "server_name": "Home Desktop",
            "server_type": "Desktop",
            "password": "password123",
            "created_at": "2024-01-01T00:00:00Z",
            "cpu": 1000,
            "memory": 2048,
            "storage": 250000,
            "network": 100
        }));
        
        servers.insert("corporate_server", serde_json::json!({
            "player_id": 2,
            "server_ip": "10.0.0.50",
            "server_name": "Corp Server",
            "server_type": "Corporate",
            "password": "C0rp0r4t3!",
            "created_at": "2024-01-01T00:00:00Z",
            "cpu": 5000,
            "memory": 16384,
            "storage": 2000000,
            "network": 1000
        }));
        
        servers.insert("npc_server", serde_json::json!({
            "player_id": null,
            "server_ip": "203.0.113.42",
            "server_name": "Bank Server",
            "server_type": "NPC",
            "password": "Ultra$ecure2024",
            "created_at": "2024-01-01T00:00:00Z",
            "cpu": 10000,
            "memory": 32768,
            "storage": 5000000,
            "network": 10000,
            "difficulty": 500
        }));
        
        servers
    }
    
    /// Generate test software data
    pub fn software() -> HashMap<&'static str, Value> {
        let mut software = HashMap::new();
        
        software.insert("basic_cracker", serde_json::json!({
            "server_id": 1,
            "software_type": "cracker",
            "version": 1,
            "size": 100,
            "created_at": "2024-01-01T01:00:00Z"
        }));
        
        software.insert("advanced_cracker", serde_json::json!({
            "server_id": 1,
            "software_type": "cracker",
            "version": 10,
            "size": 5000,
            "created_at": "2024-01-01T01:00:00Z"
        }));
        
        software.insert("hasher", serde_json::json!({
            "server_id": 1,
            "software_type": "hasher",
            "version": 5,
            "size": 1500,
            "created_at": "2024-01-01T02:00:00Z"
        }));
        
        software.insert("firewall", serde_json::json!({
            "server_id": 2,
            "software_type": "firewall",
            "version": 8,
            "size": 3000,
            "created_at": "2024-01-01T00:30:00Z"
        }));
        
        software.insert("antivirus", serde_json::json!({
            "server_id": 2,
            "software_type": "antivirus",
            "version": 6,
            "size": 2500,
            "created_at": "2024-01-01T00:45:00Z"
        }));
        
        software
    }
    
    /// Generate test process data
    pub fn processes() -> HashMap<&'static str, Value> {
        let mut processes = HashMap::new();
        
        processes.insert("running_cracker", serde_json::json!({
            "server_id": 1,
            "process_type": "cracker",
            "target_ip": "192.168.1.101",
            "target_file": "bank_data.txt",
            "status": "running",
            "started_at": "2024-01-01T12:00:00Z",
            "completion_time": 300,
            "progress": 45
        }));
        
        processes.insert("completed_process", serde_json::json!({
            "server_id": 1,
            "process_type": "hasher",
            "target_ip": "10.0.0.50",
            "target_file": "encrypted.zip",
            "status": "completed",
            "started_at": "2024-01-01T11:00:00Z",
            "completed_at": "2024-01-01T11:05:00Z",
            "completion_time": 300,
            "progress": 100,
            "result": "success"
        }));
        
        processes.insert("failed_process", serde_json::json!({
            "server_id": 1,
            "process_type": "cracker",
            "target_ip": "203.0.113.42",
            "target_file": "secure_vault.enc",
            "status": "failed",
            "started_at": "2024-01-01T10:00:00Z",
            "completed_at": "2024-01-01T10:10:00Z",
            "completion_time": 600,
            "progress": 100,
            "result": "failed",
            "failure_reason": "Insufficient software version"
        }));
        
        processes
    }
    
    /// Generate test mission data
    pub fn missions() -> HashMap<&'static str, Value> {
        let mut missions = HashMap::new();
        
        missions.insert("basic_hack", serde_json::json!({
            "mission_id": 1,
            "title": "Corporate Infiltration",
            "description": "Hack into the corporate server and steal sensitive data",
            "difficulty": 100,
            "reward_money": 5000,
            "reward_experience": 500,
            "target_ip": "10.0.0.50",
            "target_file": "financial_records.xlsx",
            "time_limit": 3600,
            "status": "available"
        }));
        
        missions.insert("advanced_mission", serde_json::json!({
            "mission_id": 2,
            "title": "Bank Heist",
            "description": "Break into the bank's secure server and transfer funds",
            "difficulty": 300,
            "reward_money": 50000,
            "reward_experience": 2500,
            "target_ip": "203.0.113.42",
            "target_file": "account_database.db",
            "time_limit": 7200,
            "status": "available",
            "requirements": {
                "min_level": 20,
                "required_software": ["cracker", "hasher", "encryptor"]
            }
        }));
        
        missions.insert("clan_mission", serde_json::json!({
            "mission_id": 3,
            "title": "Rival Clan Attack",
            "description": "Coordinate with your clan to attack rival servers",
            "difficulty": 250,
            "reward_money": 25000,
            "reward_experience": 1500,
            "reward_reputation": 100,
            "target_clan_id": 2,
            "time_limit": 10800,
            "status": "available",
            "mission_type": "clan",
            "min_participants": 3
        }));
        
        missions
    }
    
    /// Generate test log entries
    pub fn logs() -> HashMap<&'static str, Value> {
        let mut logs = HashMap::new();
        
        logs.insert("login_log", serde_json::json!({
            "server_id": 1,
            "log_type": "connection",
            "message": "User login successful from 192.168.1.10",
            "timestamp": "2024-01-01T12:00:00Z",
            "level": "info"
        }));
        
        logs.insert("attack_log", serde_json::json!({
            "server_id": 2,
            "log_type": "security",
            "message": "Intrusion detected from 192.168.1.100 - cracker attempt blocked",
            "timestamp": "2024-01-01T12:30:00Z",
            "level": "warning",
            "attacker_ip": "192.168.1.100"
        }));
        
        logs.insert("system_log", serde_json::json!({
            "server_id": 1,
            "log_type": "system",
            "message": "Software upgrade completed: cracker v2.0 installed",
            "timestamp": "2024-01-01T11:00:00Z",
            "level": "info"
        }));
        
        logs
    }
    
    /// Generate test clan data  
    pub fn clans() -> HashMap<&'static str, Value> {
        let mut clans = HashMap::new();
        
        clans.insert("elite_hackers", serde_json::json!({
            "clan_id": 1,
            "name": "Elite Hackers",
            "description": "The most skilled hackers in the game",
            "leader_id": 2,
            "created_at": "2023-12-01T00:00:00Z",
            "member_count": 15,
            "reputation": 5000,
            "level": 10
        }));
        
        clans.insert("newbie_clan", serde_json::json!({
            "clan_id": 2,
            "name": "Script Kiddies",
            "description": "Learning the ropes",
            "leader_id": 3,
            "created_at": "2024-01-15T00:00:00Z",
            "member_count": 5,
            "reputation": 100,
            "level": 1
        }));
        
        clans
    }
}

/// Helper functions for test data manipulation
pub mod helpers {
    use serde_json::Value;
    
    pub fn merge_json(base: &Value, overlay: &Value) -> Value {
        match (base, overlay) {
            (Value::Object(base_obj), Value::Object(overlay_obj)) => {
                let mut result = base_obj.clone();
                for (key, value) in overlay_obj {
                    result.insert(key.clone(), value.clone());
                }
                Value::Object(result)
            }
            _ => overlay.clone(),
        }
    }
    
    pub fn generate_ip_range(base: &str, start: u8, end: u8) -> Vec<String> {
        let parts: Vec<&str> = base.split('.').collect();
        if parts.len() != 3 {
            return vec![];
        }
        
        let network = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
        (start..=end).map(|i| format!("{}.{}", network, i)).collect()
    }
    
    pub fn create_test_session(player_id: i64) -> Value {
        use rand::{thread_rng, Rng};
        use rand::distributions::Alphanumeric;
        
        let session_token: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
        
        serde_json::json!({
            "session_token": session_token,
            "player_id": player_id,
            "created_at": chrono::Utc::now().to_rfc3339(),
            "expires_at": (chrono::Utc::now() + chrono::Duration::hours(24)).to_rfc3339(),
            "last_activity": chrono::Utc::now().to_rfc3339()
        })
    }
}