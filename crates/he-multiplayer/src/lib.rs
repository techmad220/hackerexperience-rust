//! Multiplayer System - All multiplayer interactions

pub mod clan;
pub mod pvp;
pub mod chat;
pub mod trading;
pub mod alliances;
pub mod events;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Main multiplayer coordinator
#[derive(Debug, Clone)]
pub struct MultiplayerSystem {
    pub clans: HashMap<Uuid, clan::Clan>,
    pub pvp_matches: HashMap<Uuid, pvp::PvPMatch>,
    pub chat_rooms: HashMap<String, chat::ChatRoom>,
    pub trades: HashMap<Uuid, trading::Trade>,
    pub alliances: HashMap<Uuid, alliances::Alliance>,
    pub events: Vec<events::GlobalEvent>,
}

impl MultiplayerSystem {
    pub fn new() -> Self {
        Self {
            clans: HashMap::new(),
            pvp_matches: HashMap::new(),
            chat_rooms: HashMap::new(),
            trades: HashMap::new(),
            alliances: HashMap::new(),
            events: Vec::new(),
        }
    }

    /// Initialize default chat rooms
    pub fn initialize_default_rooms(&mut self) {
        // Global chat
        self.chat_rooms.insert(
            "global".to_string(),
            chat::ChatRoom::new("global", "Global Chat", chat::RoomType::Public),
        );

        // Trade chat
        self.chat_rooms.insert(
            "trade".to_string(),
            chat::ChatRoom::new("trade", "Trade Hub", chat::RoomType::Public),
        );

        // PvP chat
        self.chat_rooms.insert(
            "pvp".to_string(),
            chat::ChatRoom::new("pvp", "PvP Arena", chat::RoomType::Public),
        );

        // Help chat
        self.chat_rooms.insert(
            "help".to_string(),
            chat::ChatRoom::new("help", "Help & Support", chat::RoomType::Public),
        );
    }
}

/// Player online status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStatus {
    pub player_id: Uuid,
    pub username: String,
    pub is_online: bool,
    pub last_seen: DateTime<Utc>,
    pub current_activity: PlayerActivity,
    pub clan_id: Option<Uuid>,
    pub pvp_rating: i32,
}

/// What a player is currently doing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerActivity {
    Idle,
    Hacking { target_ip: String },
    InPvP { opponent_id: Uuid },
    InMission { mission_id: String },
    Trading { partner_id: Uuid },
    InClanWar { war_id: Uuid },
    Chatting { room: String },
}

/// Notification types for multiplayer events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MultiplayerNotification {
    ClanInvite {
        clan_id: Uuid,
        clan_name: String,
        inviter: String,
    },
    PvPChallenge {
        challenger_id: Uuid,
        challenger_name: String,
        stake: Option<i64>,
    },
    TradeOffer {
        trader_id: Uuid,
        trader_name: String,
        trade_id: Uuid,
    },
    ClanWarStarted {
        enemy_clan: String,
        war_id: Uuid,
    },
    GlobalEventStarted {
        event_name: String,
        duration: u64,
    },
    FriendRequest {
        from_id: Uuid,
        from_name: String,
    },
    Message {
        from: String,
        content: String,
        room: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiplayer_system() {
        let mut system = MultiplayerSystem::new();
        system.initialize_default_rooms();

        assert!(system.chat_rooms.contains_key("global"));
        assert!(system.chat_rooms.contains_key("trade"));
        assert_eq!(system.chat_rooms.len(), 4);
    }
}