//! Clan System - Groups of players working together

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};

/// Clan structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clan {
    pub id: Uuid,
    pub name: String,
    pub tag: String, // 3-5 character tag
    pub description: String,
    pub leader_id: Uuid,
    pub officers: HashSet<Uuid>,
    pub members: HashSet<Uuid>,
    pub applicants: HashSet<Uuid>,
    pub created_at: DateTime<Utc>,
    pub level: u32,
    pub experience: u64,
    pub reputation: i32,
    pub bank_balance: i64,
    pub settings: ClanSettings,
    pub stats: ClanStatistics,
    pub upgrades: ClanUpgrades,
}

/// Clan settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanSettings {
    pub recruitment_open: bool,
    pub min_level_requirement: u32,
    pub min_reputation_requirement: i32,
    pub auto_accept_applications: bool,
    pub member_limit: u32,
    pub tax_rate: f32, // Percentage of member earnings
    pub war_participation: bool,
    pub message_of_the_day: String,
}

/// Clan statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanStatistics {
    pub total_hacks: u64,
    pub wars_won: u32,
    pub wars_lost: u32,
    pub territories_owned: u32,
    pub members_online: u32,
    pub weekly_activity: u64,
    pub total_earnings: i64,
}

/// Clan upgrades and perks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanUpgrades {
    pub server_boost: u32,      // +X% server capacity
    pub hack_speed: u32,        // +X% hacking speed
    pub defense_bonus: u32,     // +X% defense rating
    pub experience_boost: u32,  // +X% XP gain
    pub bank_interest: u32,     // +X% daily interest
    pub member_slots: u32,      // Additional member slots
    pub war_strength: u32,      // +X% war power
}

/// Clan member with role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanMember {
    pub player_id: Uuid,
    pub username: String,
    pub role: ClanRole,
    pub joined_at: DateTime<Utc>,
    pub contribution: i64,      // Total contributed to clan
    pub weekly_activity: u32,   // Activity points this week
    pub war_participation: u32, // Wars participated in
}

/// Clan roles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClanRole {
    Leader,
    Officer,
    Elite,
    Member,
    Recruit,
}

/// Clan war
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanWar {
    pub id: Uuid,
    pub attacker_clan: Uuid,
    pub defender_clan: Uuid,
    pub status: WarStatus,
    pub started_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub attacker_points: u64,
    pub defender_points: u64,
    pub battles: Vec<WarBattle>,
    pub prize_pool: i64,
}

/// War status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WarStatus {
    Preparation, // 24 hours to prepare
    Active,      // 48 hours of war
    Ended,       // War finished
}

/// Individual battle in a war
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarBattle {
    pub attacker_id: Uuid,
    pub defender_id: Uuid,
    pub winner: Option<Uuid>,
    pub points_earned: u32,
    pub timestamp: DateTime<Utc>,
}

/// Clan territory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanTerritory {
    pub id: String,
    pub name: String,
    pub owner_clan: Option<Uuid>,
    pub resources: TerritoryResources,
    pub defense_level: u32,
    pub capture_progress: HashMap<Uuid, f32>, // Clan ID -> capture %
}

/// Resources provided by territory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryResources {
    pub daily_money: i64,
    pub daily_experience: u32,
    pub member_bonus: f32, // Bonus to member stats
    pub special_servers: Vec<String>, // Access to special servers
}

impl Clan {
    /// Create a new clan
    pub fn new(name: String, tag: String, leader_id: Uuid) -> Self {
        let mut members = HashSet::new();
        members.insert(leader_id);

        Self {
            id: Uuid::new_v4(),
            name,
            tag,
            description: String::new(),
            leader_id,
            officers: HashSet::new(),
            members,
            applicants: HashSet::new(),
            created_at: Utc::now(),
            level: 1,
            experience: 0,
            reputation: 0,
            bank_balance: 0,
            settings: ClanSettings::default(),
            stats: ClanStatistics::default(),
            upgrades: ClanUpgrades::default(),
        }
    }

    /// Add a new member
    pub fn add_member(&mut self, player_id: Uuid) -> Result<(), ClanError> {
        if self.members.len() >= self.settings.member_limit as usize {
            return Err(ClanError::ClanFull);
        }

        self.members.insert(player_id);
        self.applicants.remove(&player_id);
        Ok(())
    }

    /// Remove a member
    pub fn remove_member(&mut self, player_id: Uuid) -> Result<(), ClanError> {
        if player_id == self.leader_id {
            return Err(ClanError::CannotRemoveLeader);
        }

        self.members.remove(&player_id);
        self.officers.remove(&player_id);
        Ok(())
    }

    /// Promote member
    pub fn promote_member(&mut self, player_id: Uuid) -> Result<(), ClanError> {
        if !self.members.contains(&player_id) {
            return Err(ClanError::NotAMember);
        }

        self.officers.insert(player_id);
        Ok(())
    }

    /// Transfer leadership
    pub fn transfer_leadership(&mut self, new_leader: Uuid) -> Result<(), ClanError> {
        if !self.members.contains(&new_leader) {
            return Err(ClanError::NotAMember);
        }

        self.officers.insert(self.leader_id); // Old leader becomes officer
        self.leader_id = new_leader;
        Ok(())
    }

    /// Add experience to clan
    pub fn add_experience(&mut self, amount: u64) {
        self.experience += amount;

        // Check for level up
        while self.experience >= self.experience_for_next_level() {
            self.experience -= self.experience_for_next_level();
            self.level += 1;
            self.on_level_up();
        }
    }

    /// Experience required for next level
    fn experience_for_next_level(&self) -> u64 {
        (self.level as u64 * 1000) * (self.level as u64 + 1)
    }

    /// Called when clan levels up
    fn on_level_up(&mut self) {
        // Increase member limit
        self.settings.member_limit += 5;

        // Unlock new features based on level
        match self.level {
            5 => self.upgrades.experience_boost = 5,
            10 => self.upgrades.hack_speed = 5,
            15 => self.upgrades.defense_bonus = 5,
            20 => self.upgrades.war_strength = 10,
            25 => self.upgrades.bank_interest = 5,
            _ => {}
        }
    }

    /// Deposit money to clan bank
    pub fn deposit(&mut self, amount: i64) {
        self.bank_balance += amount;
        self.stats.total_earnings += amount;
    }

    /// Withdraw money from clan bank (leader/officers only)
    pub fn withdraw(&mut self, amount: i64, requester: Uuid) -> Result<i64, ClanError> {
        if requester != self.leader_id && !self.officers.contains(&requester) {
            return Err(ClanError::InsufficientPermissions);
        }

        if amount > self.bank_balance {
            return Err(ClanError::InsufficientFunds);
        }

        self.bank_balance -= amount;
        Ok(amount)
    }

    /// Purchase clan upgrade
    pub fn purchase_upgrade(&mut self, upgrade: ClanUpgradeType) -> Result<(), ClanError> {
        let cost = upgrade.cost();

        if self.bank_balance < cost {
            return Err(ClanError::InsufficientFunds);
        }

        self.bank_balance -= cost;

        match upgrade {
            ClanUpgradeType::ServerBoost => self.upgrades.server_boost += 5,
            ClanUpgradeType::HackSpeed => self.upgrades.hack_speed += 5,
            ClanUpgradeType::DefenseBonus => self.upgrades.defense_bonus += 5,
            ClanUpgradeType::ExperienceBoost => self.upgrades.experience_boost += 5,
            ClanUpgradeType::BankInterest => self.upgrades.bank_interest += 2,
            ClanUpgradeType::MemberSlots => {
                self.upgrades.member_slots += 10;
                self.settings.member_limit += 10;
            }
            ClanUpgradeType::WarStrength => self.upgrades.war_strength += 5,
        }

        Ok(())
    }
}

/// Clan upgrade types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClanUpgradeType {
    ServerBoost,
    HackSpeed,
    DefenseBonus,
    ExperienceBoost,
    BankInterest,
    MemberSlots,
    WarStrength,
}

impl ClanUpgradeType {
    fn cost(&self) -> i64 {
        match self {
            Self::ServerBoost => 100000,
            Self::HackSpeed => 150000,
            Self::DefenseBonus => 150000,
            Self::ExperienceBoost => 200000,
            Self::BankInterest => 250000,
            Self::MemberSlots => 300000,
            Self::WarStrength => 500000,
        }
    }
}

impl Default for ClanSettings {
    fn default() -> Self {
        Self {
            recruitment_open: true,
            min_level_requirement: 5,
            min_reputation_requirement: 0,
            auto_accept_applications: false,
            member_limit: 50,
            tax_rate: 0.05,
            war_participation: true,
            message_of_the_day: String::new(),
        }
    }
}

impl Default for ClanStatistics {
    fn default() -> Self {
        Self {
            total_hacks: 0,
            wars_won: 0,
            wars_lost: 0,
            territories_owned: 0,
            members_online: 0,
            weekly_activity: 0,
            total_earnings: 0,
        }
    }
}

impl Default for ClanUpgrades {
    fn default() -> Self {
        Self {
            server_boost: 0,
            hack_speed: 0,
            defense_bonus: 0,
            experience_boost: 0,
            bank_interest: 0,
            member_slots: 0,
            war_strength: 0,
        }
    }
}

/// Clan system errors
#[derive(Debug, thiserror::Error)]
pub enum ClanError {
    #[error("Clan is full")]
    ClanFull,
    #[error("Not a member of the clan")]
    NotAMember,
    #[error("Cannot remove clan leader")]
    CannotRemoveLeader,
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Clan name already taken")]
    NameTaken,
    #[error("Invalid clan tag")]
    InvalidTag,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clan_creation() {
        let leader_id = Uuid::new_v4();
        let clan = Clan::new("Test Clan".to_string(), "TEST".to_string(), leader_id);

        assert_eq!(clan.name, "Test Clan");
        assert_eq!(clan.tag, "TEST");
        assert!(clan.members.contains(&leader_id));
        assert_eq!(clan.level, 1);
    }

    #[test]
    fn test_clan_membership() {
        let leader_id = Uuid::new_v4();
        let member_id = Uuid::new_v4();
        let mut clan = Clan::new("Test Clan".to_string(), "TEST".to_string(), leader_id);

        assert!(clan.add_member(member_id).is_ok());
        assert!(clan.members.contains(&member_id));

        assert!(clan.promote_member(member_id).is_ok());
        assert!(clan.officers.contains(&member_id));
    }
}