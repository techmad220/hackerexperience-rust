//! Trading System - Player-to-player item and resource exchange

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

/// Trade between two players
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub initiator: TradingParty,
    pub recipient: TradingParty,
    pub status: TradeStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub trade_type: TradeType,
    pub escrow: Option<TradeEscrow>,
    pub history: Vec<TradeEvent>,
}

/// Trading party (player) in a trade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingParty {
    pub player_id: Uuid,
    pub username: String,
    pub offer: TradeOffer,
    pub confirmed: bool,
    pub locked: bool, // Can't modify after locking
}

/// What a player is offering in trade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeOffer {
    pub money: i64,
    pub items: Vec<TradeItem>,
    pub software: Vec<TradeSoftware>,
    pub server_access: Vec<String>, // Server IPs
    pub information: Vec<TradeInformation>,
}

/// Tradeable item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeItem {
    pub item_id: String,
    pub name: String,
    pub item_type: ItemType,
    pub quantity: u32,
    pub quality: ItemQuality,
    pub market_value: i64,
}

/// Types of tradeable items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemType {
    Hardware,
    Component,
    Collectible,
    Resource,
    Key,
    Blueprint,
    Consumable,
}

/// Item quality/rarity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemQuality {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Unique,
}

/// Tradeable software
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeSoftware {
    pub software_id: String,
    pub name: String,
    pub version: String,
    pub software_type: String,
    pub size_mb: u32,
    pub market_value: i64,
    pub licensed: bool,
}

/// Tradeable information/intel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeInformation {
    pub info_id: String,
    pub info_type: InformationType,
    pub description: String,
    pub value: i64,
    pub verified: bool,
}

/// Types of information that can be traded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InformationType {
    ServerPassword,
    PlayerLocation,
    ExploitCode,
    BankAccount,
    ClanIntel,
    MarketTip,
    QuestHint,
}

/// Trade status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeStatus {
    Pending,      // Waiting for recipient
    Negotiating,  // Both parties modifying
    Locked,       // Both locked, waiting confirmation
    Confirmed,    // Both confirmed, processing
    Completed,    // Trade successful
    Cancelled,    // Trade cancelled
    Expired,      // Trade timed out
    Disputed,     // Under review
}

/// Trade type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeType {
    Direct,       // Direct player trade
    Escrow,       // Through escrow service
    Auction,      // Auction style
    Contract,     // Contract-based
    Gift,         // One-way transfer
}

/// Escrow service for secure trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeEscrow {
    pub escrow_fee: i64,
    pub items_held: bool,
    pub auto_complete: bool,
    pub completion_conditions: Vec<EscrowCondition>,
}

/// Conditions for escrow completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscrowCondition {
    TimeDelay { hours: u32 },
    LevelRequirement { min_level: u32 },
    ReputationRequirement { min_rep: i32 },
    ItemDelivery { item_id: String },
    ServiceCompletion { service: String },
}

/// Event in trade history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: TradeEventType,
    pub actor_id: Uuid,
    pub details: String,
}

/// Types of trade events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeEventType {
    Created,
    Modified,
    Locked,
    Unlocked,
    Confirmed,
    Cancelled,
    Completed,
    Disputed,
}

/// Marketplace for automated trading
#[derive(Debug, Clone)]
pub struct Marketplace {
    pub listings: HashMap<Uuid, MarketListing>,
    pub buy_orders: Vec<BuyOrder>,
    pub price_history: HashMap<String, Vec<PricePoint>>,
    pub featured_items: Vec<Uuid>,
}

/// Market listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketListing {
    pub id: Uuid,
    pub seller_id: Uuid,
    pub seller_name: String,
    pub item: MarketItem,
    pub price: i64,
    pub quantity: u32,
    pub buyout_price: Option<i64>,
    pub listed_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub views: u32,
    pub watchers: HashSet<Uuid>,
}

/// Item listed on market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketItem {
    Item(TradeItem),
    Software(TradeSoftware),
    Information(TradeInformation),
    Bundle(Vec<MarketItem>),
}

/// Buy order for items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyOrder {
    pub id: Uuid,
    pub buyer_id: Uuid,
    pub item_name: String,
    pub max_price: i64,
    pub quantity: u32,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Price history point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub timestamp: DateTime<Utc>,
    pub price: i64,
    pub volume: u32,
}

impl Trade {
    /// Create a new trade
    pub fn new(initiator: TradingParty, recipient: TradingParty, trade_type: TradeType) -> Self {
        Self {
            id: Uuid::new_v4(),
            initiator,
            recipient,
            status: TradeStatus::Pending,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
            trade_type,
            escrow: None,
            history: vec![TradeEvent {
                timestamp: Utc::now(),
                event_type: TradeEventType::Created,
                actor_id: initiator.player_id,
                details: "Trade created".to_string(),
            }],
        }
    }

    /// Modify trade offer
    pub fn modify_offer(&mut self, player_id: Uuid, new_offer: TradeOffer) -> Result<(), TradeError> {
        if self.status != TradeStatus::Negotiating && self.status != TradeStatus::Pending {
            return Err(TradeError::InvalidStatus);
        }

        if self.initiator.player_id == player_id {
            if self.initiator.locked {
                return Err(TradeError::TradeLocked);
            }
            self.initiator.offer = new_offer;
            self.recipient.locked = false; // Unlock other party
        } else if self.recipient.player_id == player_id {
            if self.recipient.locked {
                return Err(TradeError::TradeLocked);
            }
            self.recipient.offer = new_offer;
            self.initiator.locked = false; // Unlock other party
        } else {
            return Err(TradeError::NotParticipant);
        }

        self.status = TradeStatus::Negotiating;
        self.add_event(TradeEventType::Modified, player_id, "Offer modified");

        Ok(())
    }

    /// Lock trade offer (can't modify after locking)
    pub fn lock_offer(&mut self, player_id: Uuid) -> Result<(), TradeError> {
        if self.status != TradeStatus::Negotiating && self.status != TradeStatus::Pending {
            return Err(TradeError::InvalidStatus);
        }

        if self.initiator.player_id == player_id {
            self.initiator.locked = true;
        } else if self.recipient.player_id == player_id {
            self.recipient.locked = true;
        } else {
            return Err(TradeError::NotParticipant);
        }

        // If both locked, move to confirmation
        if self.initiator.locked && self.recipient.locked {
            self.status = TradeStatus::Locked;
        }

        self.add_event(TradeEventType::Locked, player_id, "Offer locked");

        Ok(())
    }

    /// Confirm trade (execute if both confirm)
    pub fn confirm_trade(&mut self, player_id: Uuid) -> Result<(), TradeError> {
        if self.status != TradeStatus::Locked {
            return Err(TradeError::InvalidStatus);
        }

        if self.initiator.player_id == player_id {
            self.initiator.confirmed = true;
        } else if self.recipient.player_id == player_id {
            self.recipient.confirmed = true;
        } else {
            return Err(TradeError::NotParticipant);
        }

        // If both confirmed, complete trade
        if self.initiator.confirmed && self.recipient.confirmed {
            self.complete_trade()?;
        }

        self.add_event(TradeEventType::Confirmed, player_id, "Trade confirmed");

        Ok(())
    }

    /// Complete the trade
    fn complete_trade(&mut self) -> Result<(), TradeError> {
        // Validate both parties have required items
        // This would check inventories in real implementation

        self.status = TradeStatus::Completed;
        self.add_event(
            TradeEventType::Completed,
            Uuid::nil(),
            "Trade completed successfully",
        );

        Ok(())
    }

    /// Cancel trade
    pub fn cancel_trade(&mut self, player_id: Uuid) -> Result<(), TradeError> {
        if self.status == TradeStatus::Completed || self.status == TradeStatus::Cancelled {
            return Err(TradeError::InvalidStatus);
        }

        if self.initiator.player_id != player_id && self.recipient.player_id != player_id {
            return Err(TradeError::NotParticipant);
        }

        self.status = TradeStatus::Cancelled;
        self.add_event(TradeEventType::Cancelled, player_id, "Trade cancelled");

        Ok(())
    }

    /// Add event to history
    fn add_event(&mut self, event_type: TradeEventType, actor_id: Uuid, details: &str) {
        self.history.push(TradeEvent {
            timestamp: Utc::now(),
            event_type,
            actor_id,
            details: details.to_string(),
        });
    }

    /// Calculate total value of trade
    pub fn calculate_value(&self) -> (i64, i64) {
        let initiator_value = self.calculate_offer_value(&self.initiator.offer);
        let recipient_value = self.calculate_offer_value(&self.recipient.offer);
        (initiator_value, recipient_value)
    }

    /// Calculate value of an offer
    fn calculate_offer_value(&self, offer: &TradeOffer) -> i64 {
        let mut total = offer.money;

        for item in &offer.items {
            total += item.market_value * item.quantity as i64;
        }

        for software in &offer.software {
            total += software.market_value;
        }

        for info in &offer.information {
            total += info.value;
        }

        // Server access has value too (estimated)
        total += offer.server_access.len() as i64 * 10000;

        total
    }
}

impl Marketplace {
    pub fn new() -> Self {
        Self {
            listings: HashMap::new(),
            buy_orders: Vec::new(),
            price_history: HashMap::new(),
            featured_items: Vec::new(),
        }
    }

    /// Create a new listing
    pub fn create_listing(&mut self, listing: MarketListing) {
        self.listings.insert(listing.id, listing);
    }

    /// Buy item from listing
    pub fn buy_item(&mut self, listing_id: Uuid, buyer_id: Uuid, quantity: u32) -> Result<i64, TradeError> {
        let listing = self.listings.get_mut(&listing_id)
            .ok_or(TradeError::ListingNotFound)?;

        if listing.quantity < quantity {
            return Err(TradeError::InsufficientQuantity);
        }

        if listing.seller_id == buyer_id {
            return Err(TradeError::CannotTradeSelf);
        }

        let total_cost = listing.price * quantity as i64;

        // Update listing
        listing.quantity -= quantity;
        if listing.quantity == 0 {
            self.listings.remove(&listing_id);
        }

        // Record price point
        self.record_price(listing.item.get_name(), listing.price, quantity);

        Ok(total_cost)
    }

    /// Record price in history
    fn record_price(&mut self, item_name: String, price: i64, volume: u32) {
        let history = self.price_history.entry(item_name).or_insert_with(Vec::new);

        history.push(PricePoint {
            timestamp: Utc::now(),
            price,
            volume,
        });

        // Keep only last 1000 points
        if history.len() > 1000 {
            history.remove(0);
        }
    }

    /// Get average price for item
    pub fn get_average_price(&self, item_name: &str, hours: u32) -> Option<i64> {
        let history = self.price_history.get(item_name)?;
        let cutoff = Utc::now() - Duration::hours(hours as i64);

        let recent: Vec<&PricePoint> = history.iter()
            .filter(|p| p.timestamp > cutoff)
            .collect();

        if recent.is_empty() {
            return None;
        }

        let total: i64 = recent.iter().map(|p| p.price).sum();
        Some(total / recent.len() as i64)
    }
}

impl MarketItem {
    fn get_name(&self) -> String {
        match self {
            MarketItem::Item(item) => item.name.clone(),
            MarketItem::Software(software) => software.name.clone(),
            MarketItem::Information(info) => info.description.clone(),
            MarketItem::Bundle(_) => "Bundle".to_string(),
        }
    }
}

use std::collections::HashSet;

/// Trade system errors
#[derive(Debug, thiserror::Error)]
pub enum TradeError {
    #[error("Invalid trade status")]
    InvalidStatus,
    #[error("Trade is locked")]
    TradeLocked,
    #[error("Not a participant in this trade")]
    NotParticipant,
    #[error("Insufficient items")]
    InsufficientItems,
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Trade expired")]
    TradeExpired,
    #[error("Listing not found")]
    ListingNotFound,
    #[error("Insufficient quantity")]
    InsufficientQuantity,
    #[error("Cannot trade with yourself")]
    CannotTradeSelf,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trade() {
        let initiator = TradingParty {
            player_id: Uuid::new_v4(),
            username: "Player1".to_string(),
            offer: TradeOffer {
                money: 1000,
                items: vec![],
                software: vec![],
                server_access: vec![],
                information: vec![],
            },
            confirmed: false,
            locked: false,
        };

        let recipient = TradingParty {
            player_id: Uuid::new_v4(),
            username: "Player2".to_string(),
            offer: TradeOffer {
                money: 500,
                items: vec![],
                software: vec![],
                server_access: vec![],
                information: vec![],
            },
            confirmed: false,
            locked: false,
        };

        let mut trade = Trade::new(initiator, recipient, TradeType::Direct);
        assert_eq!(trade.status, TradeStatus::Pending);

        // Test locking
        assert!(trade.lock_offer(trade.initiator.player_id).is_ok());
        assert!(trade.lock_offer(trade.recipient.player_id).is_ok());
        assert_eq!(trade.status, TradeStatus::Locked);
    }
}