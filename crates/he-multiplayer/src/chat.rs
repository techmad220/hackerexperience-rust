//! Chat System - Real-time communication between players

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet, VecDeque};

/// Chat room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRoom {
    pub id: String,
    pub name: String,
    pub room_type: RoomType,
    pub members: HashSet<Uuid>,
    pub messages: VecDeque<ChatMessage>,
    pub moderators: HashSet<Uuid>,
    pub banned_users: HashSet<Uuid>,
    pub settings: RoomSettings,
    pub created_at: DateTime<Utc>,
}

/// Type of chat room
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoomType {
    Public,      // Global, trade, help, etc.
    Clan,        // Clan-only chat
    Private,     // Direct messages
    War,         // War coordination
    Alliance,    // Alliance chat
    System,      // System announcements
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub sender_name: String,
    pub content: String,
    pub message_type: MessageType,
    pub timestamp: DateTime<Utc>,
    pub edited: bool,
    pub edited_at: Option<DateTime<Utc>>,
    pub reactions: HashMap<String, Vec<Uuid>>, // emoji -> list of users
    pub reply_to: Option<Uuid>,
}

/// Type of message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    Emote,      // /me actions
    System,     // System messages
    Join,       // User joined
    Leave,      // User left
    Whisper,    // Private message in public room
    Command,    // Bot commands
    Trade,      // Trade offers
    ClanInvite, // Clan invitations
}

/// Room settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSettings {
    pub max_messages: usize,
    pub slow_mode: bool,
    pub slow_mode_seconds: u32,
    pub members_only: bool,
    pub level_requirement: u32,
    pub auto_moderation: bool,
    pub profanity_filter: bool,
    pub link_filter: bool,
}

/// Chat command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatCommand {
    Help,
    Who,                               // List online users
    Whisper { to: String, message: String },
    Invite { username: String },
    Kick { username: String },
    Ban { username: String, duration: Option<u64> },
    Mute { username: String, duration: u64 },
    Clear,                            // Clear chat history
    Stats { username: Option<String> },
    Trade { with: String },
    Duel { opponent: String },
}

/// Direct message conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectMessage {
    pub id: Uuid,
    pub participants: (Uuid, Uuid),
    pub messages: VecDeque<ChatMessage>,
    pub created_at: DateTime<Utc>,
    pub last_message_at: DateTime<Utc>,
    pub unread_count: HashMap<Uuid, u32>,
}

/// User chat profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatProfile {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: String,
    pub status: UserStatus,
    pub status_message: String,
    pub avatar: String,
    pub badges: Vec<ChatBadge>,
    pub blocked_users: HashSet<Uuid>,
    pub friends: HashSet<Uuid>,
    pub last_seen: DateTime<Utc>,
}

/// User online status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserStatus {
    Online,
    Away,
    Busy,
    Invisible,
    Offline,
}

/// Chat badges/titles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatBadge {
    Admin,
    Moderator,
    VIP,
    Premium,
    ClanLeader,
    Veteran,       // Long-time player
    TopHacker,     // Leaderboard position
    PvPChampion,   // PvP tournament winner
    BugHunter,     // Found and reported bugs
    Supporter,     // Financial supporter
    EventWinner,   // Won special event
}

/// Chat notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatNotification {
    pub id: Uuid,
    pub recipient_id: Uuid,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub read: bool,
    pub action_url: Option<String>,
}

/// Type of notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    Message,
    Mention,
    FriendRequest,
    ClanInvite,
    TradeOffer,
    SystemAlert,
    Achievement,
}

impl ChatRoom {
    /// Create a new chat room
    pub fn new(id: &str, name: &str, room_type: RoomType) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            room_type,
            members: HashSet::new(),
            messages: VecDeque::with_capacity(1000),
            moderators: HashSet::new(),
            banned_users: HashSet::new(),
            settings: RoomSettings::default(),
            created_at: Utc::now(),
        }
    }

    /// Add a message to the room
    pub fn add_message(&mut self, message: ChatMessage) -> Result<(), ChatError> {
        // Check if sender is banned
        if self.banned_users.contains(&message.sender_id) {
            return Err(ChatError::UserBanned);
        }

        // Check slow mode
        if self.settings.slow_mode {
            if let Some(last_msg) = self.messages.iter().rev()
                .find(|m| m.sender_id == message.sender_id)
            {
                let seconds_since = (message.timestamp - last_msg.timestamp).num_seconds();
                if seconds_since < self.settings.slow_mode_seconds as i64 {
                    return Err(ChatError::SlowMode);
                }
            }
        }

        // Apply filters if enabled
        let filtered_content = if self.settings.profanity_filter {
            self.filter_profanity(message.content.clone())
        } else {
            message.content.clone()
        };

        let mut final_message = message;
        final_message.content = filtered_content;

        // Add message
        self.messages.push_back(final_message);

        // Trim old messages
        while self.messages.len() > self.settings.max_messages {
            self.messages.pop_front();
        }

        Ok(())
    }

    /// Filter profanity from message
    fn filter_profanity(&self, content: String) -> String {
        // Simple filter - replace bad words with asterisks
        let bad_words = vec!["badword1", "badword2"]; // Would be loaded from config
        let mut filtered = content;

        for word in bad_words {
            filtered = filtered.replace(word, &"*".repeat(word.len()));
        }

        filtered
    }

    /// Join the room
    pub fn join(&mut self, user_id: Uuid, username: String) -> Result<(), ChatError> {
        if self.banned_users.contains(&user_id) {
            return Err(ChatError::UserBanned);
        }

        self.members.insert(user_id);

        // Add join message
        let join_msg = ChatMessage {
            id: Uuid::new_v4(),
            sender_id: user_id,
            sender_name: username.clone(),
            content: format!("{} joined the room", username),
            message_type: MessageType::Join,
            timestamp: Utc::now(),
            edited: false,
            edited_at: None,
            reactions: HashMap::new(),
            reply_to: None,
        };

        self.messages.push_back(join_msg);
        Ok(())
    }

    /// Leave the room
    pub fn leave(&mut self, user_id: Uuid, username: String) {
        self.members.remove(&user_id);

        // Add leave message
        let leave_msg = ChatMessage {
            id: Uuid::new_v4(),
            sender_id: user_id,
            sender_name: username.clone(),
            content: format!("{} left the room", username),
            message_type: MessageType::Leave,
            timestamp: Utc::now(),
            edited: false,
            edited_at: None,
            reactions: HashMap::new(),
            reply_to: None,
        };

        self.messages.push_back(leave_msg);
    }

    /// Ban a user from the room
    pub fn ban_user(&mut self, user_id: Uuid, moderator_id: Uuid) -> Result<(), ChatError> {
        if !self.moderators.contains(&moderator_id) {
            return Err(ChatError::InsufficientPermissions);
        }

        self.banned_users.insert(user_id);
        self.members.remove(&user_id);
        Ok(())
    }

    /// Unban a user
    pub fn unban_user(&mut self, user_id: Uuid, moderator_id: Uuid) -> Result<(), ChatError> {
        if !self.moderators.contains(&moderator_id) {
            return Err(ChatError::InsufficientPermissions);
        }

        self.banned_users.remove(&user_id);
        Ok(())
    }

    /// Add moderator
    pub fn add_moderator(&mut self, user_id: Uuid) {
        self.moderators.insert(user_id);
    }

    /// Clear chat history
    pub fn clear_history(&mut self, moderator_id: Uuid) -> Result<(), ChatError> {
        if !self.moderators.contains(&moderator_id) {
            return Err(ChatError::InsufficientPermissions);
        }

        self.messages.clear();
        Ok(())
    }
}

impl Default for RoomSettings {
    fn default() -> Self {
        Self {
            max_messages: 1000,
            slow_mode: false,
            slow_mode_seconds: 5,
            members_only: false,
            level_requirement: 0,
            auto_moderation: true,
            profanity_filter: true,
            link_filter: false,
        }
    }
}

impl ChatMessage {
    /// Create a new text message
    pub fn new_text(sender_id: Uuid, sender_name: String, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender_id,
            sender_name,
            content,
            message_type: MessageType::Text,
            timestamp: Utc::now(),
            edited: false,
            edited_at: None,
            reactions: HashMap::new(),
            reply_to: None,
        }
    }

    /// Edit the message
    pub fn edit(&mut self, new_content: String) {
        self.content = new_content;
        self.edited = true;
        self.edited_at = Some(Utc::now());
    }

    /// Add reaction to message
    pub fn add_reaction(&mut self, emoji: String, user_id: Uuid) {
        self.reactions
            .entry(emoji)
            .or_insert_with(Vec::new)
            .push(user_id);
    }

    /// Remove reaction from message
    pub fn remove_reaction(&mut self, emoji: &str, user_id: Uuid) {
        if let Some(users) = self.reactions.get_mut(emoji) {
            users.retain(|&id| id != user_id);
            if users.is_empty() {
                self.reactions.remove(emoji);
            }
        }
    }
}

/// Chat system errors
#[derive(Debug, thiserror::Error)]
pub enum ChatError {
    #[error("User is banned from this room")]
    UserBanned,
    #[error("Slow mode is active")]
    SlowMode,
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    #[error("Room not found")]
    RoomNotFound,
    #[error("User not found")]
    UserNotFound,
    #[error("Message too long")]
    MessageTooLong,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_room() {
        let mut room = ChatRoom::new("global", "Global Chat", RoomType::Public);
        let user_id = Uuid::new_v4();

        assert!(room.join(user_id, "TestUser".to_string()).is_ok());
        assert!(room.members.contains(&user_id));

        let message = ChatMessage::new_text(
            user_id,
            "TestUser".to_string(),
            "Hello, world!".to_string(),
        );

        assert!(room.add_message(message).is_ok());
        assert!(!room.messages.is_empty());
    }

    #[test]
    fn test_message_reactions() {
        let mut message = ChatMessage::new_text(
            Uuid::new_v4(),
            "User".to_string(),
            "Test message".to_string(),
        );

        let user_id = Uuid::new_v4();
        message.add_reaction("👍".to_string(), user_id);

        assert!(message.reactions.contains_key("👍"));
        assert!(message.reactions["👍"].contains(&user_id));

        message.remove_reaction("👍", user_id);
        assert!(!message.reactions.contains_key("👍"));
    }
}