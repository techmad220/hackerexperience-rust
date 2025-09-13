use serde::{Deserialize, Serialize};
use crate::{UserId, IpAddress};

// Placeholder - will be expanded based on PHP NPC.class.php  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Npc {
    pub id: UserId, // NPCs use user table but with isNPC = 1
    pub name: String,
    pub ip: IpAddress,
    pub npc_type: String,
}