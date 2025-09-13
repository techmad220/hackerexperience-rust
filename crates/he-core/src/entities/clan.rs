use serde::{Deserialize, Serialize};
use crate::{ClanId, UserId};

// Placeholder - will be expanded based on PHP Clan.class.php
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clan {
    pub id: ClanId,
    pub name: String,
    pub leader_id: UserId,
    pub member_count: i32,
}