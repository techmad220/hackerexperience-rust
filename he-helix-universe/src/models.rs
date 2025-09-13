use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use he_core::id::{EntityId, ServerId, NetworkId};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "npc_type", rename_all = "lowercase")]
pub enum NPCType {
    Server,
    Bank,
    Shop,
    Mission,
    Story,
}

impl NPCType {
    pub fn possible_types() -> Vec<NPCType> {
        vec![
            NPCType::Server,
            NPCType::Bank,
            NPCType::Shop,
            NPCType::Mission,
            NPCType::Story,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NPC {
    pub npc_id: Uuid,
    pub npc_type: NPCType,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Bank {
    pub bank_id: Uuid,
    pub name: String,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ATM {
    pub atm_id: ServerId,
    pub bank_id: Uuid,
    pub region: String,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedEntry {
    pub id: Uuid,
    pub npc_type: NPCType,
    pub servers: Vec<SeedServer>,
    pub anycast: Option<String>,
    pub custom: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedServer {
    pub id: ServerId,
    pub static_ip: Option<String>,
    pub custom: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct CreateNPCParams {
    pub npc_id: Uuid,
    pub npc_type: NPCType,
}

#[derive(Debug, Clone)]
pub struct CreateBankParams {
    pub bank_id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct CreateATMParams {
    pub atm_id: ServerId,
    pub bank_id: Uuid,
    pub region: String,
}