use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use rand::{Rng, thread_rng};
use crate::{UserId, IpAddress, SoftwareId, HardwareId, HeResult, HackerExperienceError};

/// Complete economic system with banking, Bitcoin, hardware pricing, and marketplace
#[derive(Debug, Clone)]
pub struct EconomySystem {
    /// User bank accounts - user_id -> BankAccount
    bank_accounts: HashMap<UserId, BankAccount>,
    /// User Bitcoin wallets - user_id -> BitcoinWallet
    bitcoin_wallets: HashMap<UserId, BitcoinWallet>,
    /// Active bank transfers - transfer_id -> BankTransfer
    active_transfers: HashMap<u64, BankTransfer>,
    /// Bitcoin transactions - transaction_id -> BitcoinTransaction
    bitcoin_transactions: HashMap<u64, BitcoinTransaction>,
    /// Hardware market listings - listing_id -> HardwareListing
    hardware_market: HashMap<u64, HardwareListing>,
    /// Software market listings - listing_id -> SoftwareListing
    software_market: HashMap<u64, SoftwareListing>,
    /// Market history for price calculations
    market_history: Vec<MarketTransaction>,
    /// Current market prices
    current_prices: MarketPrices,
    /// Bitcoin exchange rate (to game money)
    bitcoin_exchange_rate: f64,
}

impl EconomySystem {
    pub fn new() -> Self {
        let mut system = Self {
            bank_accounts: HashMap::new(),
            bitcoin_wallets: HashMap::new(),
            active_transfers: HashMap::new(),
            bitcoin_transactions: HashMap::new(),
            hardware_market: HashMap::new(),
            software_market: HashMap::new(),
            market_history: Vec::new(),
            current_prices: MarketPrices::default(),
            bitcoin_exchange_rate: 2500.0, // $2500 per Bitcoin
        };

        system.initialize_market_prices();
        system
    }

    /// Create bank account for user
    pub fn create_bank_account(&mut self, user_id: UserId, initial_balance: i64) -> HeResult<BankAccount> {
        if self.bank_accounts.contains_key(&user_id) {
            return Err(HackerExperienceError::BankAccountAlreadyExists);
        }

        let mut rng = thread_rng();
        let account = BankAccount {
            user_id,
            account_number: format!("{:010}", rng.gen::<u32>()),
            balance: initial_balance,
            created_at: Utc::now(),
            last_transaction: Utc::now(),
            transaction_history: Vec::new(),
            daily_transfer_limit: 50000, // $50,000 default
            interest_rate: 0.01, // 1% annual
            account_type: BankAccountType::Standard,
            frozen: false,
        };

        self.bank_accounts.insert(user_id, account.clone());
        Ok(account)
    }

    /// Create Bitcoin wallet for user
    pub fn create_bitcoin_wallet(&mut self, user_id: UserId) -> HeResult<BitcoinWallet> {
        if self.bitcoin_wallets.contains_key(&user_id) {
            return Err(HackerExperienceError::BitcoinWalletAlreadyExists);
        }

        let wallet = BitcoinWallet {
            user_id,
            address: self.generate_bitcoin_address(),
            balance: 0.0,
            created_at: Utc::now(),
            transaction_history: Vec::new(),
            private_key: self.generate_private_key(),
            last_mining_attempt: None,
            mining_power: 0,
            total_mined: 0.0,
        };

        self.bitcoin_wallets.insert(user_id, wallet.clone());
        Ok(wallet)
    }

    /// Transfer money between bank accounts
    pub fn transfer_money(
        &mut self,
        from_user_id: UserId,
        to_account_number: &str,
        amount: i64,
        description: Option<String>
    ) -> HeResult<BankTransfer> {
        // Validate sender account
        let from_account = self.bank_accounts.get(&from_user_id)
            .ok_or(HackerExperienceError::BankAccountNotFound)?;

        if from_account.frozen {
            return Err(HackerExperienceError::AccountFrozen);
        }

        if from_account.balance < amount {
            return Err(HackerExperienceError::InsufficientFunds);
        }

        // Check daily transfer limit
        let today_transfers = self.calculate_daily_transfers(from_user_id);
        if today_transfers + amount > from_account.daily_transfer_limit {
            return Err(HackerExperienceError::DailyLimitExceeded);
        }

        // Find recipient account
        let to_user_id = self.find_user_by_account_number(to_account_number)
            .ok_or(HackerExperienceError::RecipientAccountNotFound)?;

        let mut rng = thread_rng();
        let transfer_id = rng.gen::<u64>();
        
        // Calculate transfer fee (0.1% minimum $10)
        let fee = ((amount as f64 * 0.001).max(10.0)) as i64;
        let total_amount = amount + fee;

        if from_account.balance < total_amount {
            return Err(HackerExperienceError::InsufficientFundsWithFees);
        }

        // Create transfer
        let transfer = BankTransfer {
            id: transfer_id,
            from_user_id,
            to_user_id,
            from_account: from_account.account_number.clone(),
            to_account: to_account_number.to_string(),
            amount,
            fee,
            description: description.unwrap_or_else(|| "Bank transfer".to_string()),
            status: TransferStatus::Processing,
            initiated_at: Utc::now(),
            completed_at: None,
            processing_time: self.calculate_transfer_time(amount),
        };

        // Deduct from sender immediately
        if let Some(sender_account) = self.bank_accounts.get_mut(&from_user_id) {
            sender_account.balance -= total_amount;
            sender_account.last_transaction = Utc::now();
            sender_account.transaction_history.push(BankTransaction {
                id: rng.gen::<u64>(),
                transaction_type: TransactionType::Transfer,
                amount: -(total_amount),
                description: format!("Transfer to {} (Fee: ${})", to_account_number, fee),
                timestamp: Utc::now(),
                balance_after: sender_account.balance,
            });
        }

        self.active_transfers.insert(transfer_id, transfer.clone());
        Ok(transfer)
    }

    /// Process pending bank transfers
    pub fn process_bank_transfers(&mut self) -> HeResult<Vec<TransferResult>> {
        let mut results = Vec::new();
        let now = Utc::now();
        let mut completed_transfers = Vec::new();

        for (transfer_id, transfer) in &mut self.active_transfers {
            if transfer.status == TransferStatus::Processing {
                let completion_time = transfer.initiated_at + Duration::seconds(transfer.processing_time as i64);
                
                if now >= completion_time {
                    // Complete the transfer
                    transfer.status = TransferStatus::Completed;
                    transfer.completed_at = Some(now);

                    // Add money to recipient
                    if let Some(recipient_account) = self.bank_accounts.get_mut(&transfer.to_user_id) {
                        recipient_account.balance += transfer.amount;
                        recipient_account.last_transaction = now;
                        recipient_account.transaction_history.push(BankTransaction {
                            id: thread_rng().gen::<u64>(),
                            transaction_type: TransactionType::Received,
                            amount: transfer.amount,
                            description: format!("Transfer from {}", transfer.from_account),
                            timestamp: now,
                            balance_after: recipient_account.balance,
                        });
                    }

                    results.push(TransferResult {
                        transfer_id: *transfer_id,
                        status: TransferStatus::Completed,
                        message: "Transfer completed successfully".to_string(),
                    });

                    completed_transfers.push(*transfer_id);
                }
            }
        }

        // Remove completed transfers
        for transfer_id in completed_transfers {
            self.active_transfers.remove(&transfer_id);
        }

        Ok(results)
    }

    /// Mine Bitcoin (simplified mining simulation)
    pub fn mine_bitcoin(&mut self, user_id: UserId, mining_power: u32, duration_hours: u32) -> HeResult<BitcoinMiningResult> {
        let wallet = self.bitcoin_wallets.get_mut(&user_id)
            .ok_or(HackerExperienceError::BitcoinWalletNotFound)?;

        let now = Utc::now();
        
        // Check if user can mine (cooldown period)
        if let Some(last_mining) = wallet.last_mining_attempt {
            if now - last_mining < Duration::hours(1) {
                return Err(HackerExperienceError::MiningCooldownActive);
            }
        }

        // Calculate mining reward based on power and time
        let base_reward = self.calculate_bitcoin_mining_reward(mining_power, duration_hours);
        let mut rng = thread_rng();
        
        // Add randomness (±20%)
        let randomness = rng.gen_range(0.8..=1.2);
        let actual_reward = base_reward * randomness;

        // Update wallet
        wallet.balance += actual_reward;
        wallet.total_mined += actual_reward;
        wallet.mining_power = mining_power;
        wallet.last_mining_attempt = Some(now);

        // Add transaction record
        wallet.transaction_history.push(BitcoinTransaction {
            id: rng.gen::<u64>(),
            transaction_type: BitcoinTransactionType::Mining,
            amount: actual_reward,
            from_address: None,
            to_address: Some(wallet.address.clone()),
            description: format!("Mining reward (Power: {}, Duration: {}h)", mining_power, duration_hours),
            timestamp: now,
            confirmations: 6, // Instant confirmations for mining
            fee: 0.0,
        });

        Ok(BitcoinMiningResult {
            reward: actual_reward,
            mining_power,
            duration_hours,
            efficiency: self.calculate_mining_efficiency(mining_power),
            estimated_next_reward: self.calculate_bitcoin_mining_reward(mining_power, 1),
        })
    }

    /// Exchange Bitcoin for game money
    pub fn exchange_bitcoin_to_money(&mut self, user_id: UserId, bitcoin_amount: f64) -> HeResult<ExchangeResult> {
        let wallet = self.bitcoin_wallets.get_mut(&user_id)
            .ok_or(HackerExperienceError::BitcoinWalletNotFound)?;

        if wallet.balance < bitcoin_amount {
            return Err(HackerExperienceError::InsufficientBitcoin);
        }

        let bank_account = self.bank_accounts.get_mut(&user_id)
            .ok_or(HackerExperienceError::BankAccountNotFound)?;

        // Calculate exchange amount with fee
        let exchange_rate = self.get_current_bitcoin_rate();
        let gross_amount = (bitcoin_amount * exchange_rate) as i64;
        let fee = (gross_amount as f64 * 0.02) as i64; // 2% exchange fee
        let net_amount = gross_amount - fee;

        // Execute exchange
        wallet.balance -= bitcoin_amount;
        bank_account.balance += net_amount;

        let mut rng = thread_rng();
        let now = Utc::now();

        // Record Bitcoin transaction
        wallet.transaction_history.push(BitcoinTransaction {
            id: rng.gen::<u64>(),
            transaction_type: BitcoinTransactionType::Exchange,
            amount: -bitcoin_amount,
            from_address: Some(wallet.address.clone()),
            to_address: None,
            description: format!("Exchange to game money (Rate: ${:.2})", exchange_rate),
            timestamp: now,
            confirmations: 1,
            fee: 0.0,
        });

        // Record bank transaction
        bank_account.transaction_history.push(BankTransaction {
            id: rng.gen::<u64>(),
            transaction_type: TransactionType::Exchange,
            amount: net_amount,
            description: format!("Bitcoin exchange ({:.6} BTC @ ${:.2})", bitcoin_amount, exchange_rate),
            timestamp: now,
            balance_after: bank_account.balance,
        });

        Ok(ExchangeResult {
            bitcoin_amount,
            exchange_rate,
            gross_money: gross_amount,
            fee,
            net_money: net_amount,
        })
    }

    /// List hardware for sale
    pub fn list_hardware_for_sale(
        &mut self,
        seller_id: UserId,
        hardware_id: HardwareId,
        price: i64,
        description: String
    ) -> HeResult<HardwareListing> {
        let mut rng = thread_rng();
        let listing_id = rng.gen::<u64>();

        let listing = HardwareListing {
            id: listing_id,
            seller_id,
            hardware_id,
            hardware_type: self.get_hardware_type(hardware_id),
            specifications: self.get_hardware_specs(hardware_id),
            price,
            description,
            listed_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(7), // 7 day listings
            status: ListingStatus::Active,
            views: 0,
            watchers: 0,
        };

        self.hardware_market.insert(listing_id, listing.clone());
        Ok(listing)
    }

    /// List software for sale
    pub fn list_software_for_sale(
        &mut self,
        seller_id: UserId,
        software_id: SoftwareId,
        price: i64,
        description: String
    ) -> HeResult<SoftwareListing> {
        let mut rng = thread_rng();
        let listing_id = rng.gen::<u64>();

        let listing = SoftwareListing {
            id: listing_id,
            seller_id,
            software_id,
            software_type: self.get_software_type(software_id),
            version: self.get_software_version(software_id),
            power_rating: self.get_software_power(software_id),
            price,
            description,
            listed_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(7),
            status: ListingStatus::Active,
            views: 0,
            watchers: 0,
            license_type: LicenseType::SingleUse,
        };

        self.software_market.insert(listing_id, listing.clone());
        Ok(listing)
    }

    /// Purchase hardware from market
    pub fn purchase_hardware(&mut self, buyer_id: UserId, listing_id: u64) -> HeResult<PurchaseResult> {
        let listing = self.hardware_market.get(&listing_id)
            .ok_or(HackerExperienceError::ListingNotFound)?
            .clone();

        if listing.status != ListingStatus::Active {
            return Err(HackerExperienceError::ListingNotAvailable);
        }

        if listing.seller_id == buyer_id {
            return Err(HackerExperienceError::CannotBuyOwnListing);
        }

        // Check buyer funds
        let buyer_account = self.bank_accounts.get(&buyer_id)
            .ok_or(HackerExperienceError::BankAccountNotFound)?;

        if buyer_account.balance < listing.price {
            return Err(HackerExperienceError::InsufficientFunds);
        }

        // Calculate marketplace fee (5%)
        let marketplace_fee = (listing.price as f64 * 0.05) as i64;
        let seller_amount = listing.price - marketplace_fee;

        // Execute transaction
        self.transfer_marketplace_funds(buyer_id, listing.seller_id, listing.price, marketplace_fee)?;

        // Update listing status
        if let Some(listing_mut) = self.hardware_market.get_mut(&listing_id) {
            listing_mut.status = ListingStatus::Sold;
        }

        // Record market transaction
        self.record_market_transaction(MarketTransaction {
            id: thread_rng().gen::<u64>(),
            transaction_type: MarketTransactionType::Hardware,
            item_id: listing.hardware_id as u64,
            buyer_id,
            seller_id: listing.seller_id,
            price: listing.price,
            marketplace_fee,
            timestamp: Utc::now(),
        });

        Ok(PurchaseResult {
            item_type: ItemType::Hardware,
            item_id: listing.hardware_id as u64,
            price: listing.price,
            marketplace_fee,
            seller_amount,
            transaction_id: thread_rng().gen::<u64>(),
        })
    }

    /// Purchase software from market
    pub fn purchase_software(&mut self, buyer_id: UserId, listing_id: u64) -> HeResult<PurchaseResult> {
        let listing = self.software_market.get(&listing_id)
            .ok_or(HackerExperienceError::ListingNotFound)?
            .clone();

        if listing.status != ListingStatus::Active {
            return Err(HackerExperienceError::ListingNotAvailable);
        }

        if listing.seller_id == buyer_id {
            return Err(HackerExperienceError::CannotBuyOwnListing);
        }

        // Check buyer funds
        let buyer_account = self.bank_accounts.get(&buyer_id)
            .ok_or(HackerExperienceError::BankAccountNotFound)?;

        if buyer_account.balance < listing.price {
            return Err(HackerExperienceError::InsufficientFunds);
        }

        // Calculate marketplace fee (3% for software)
        let marketplace_fee = (listing.price as f64 * 0.03) as i64;
        let seller_amount = listing.price - marketplace_fee;

        // Execute transaction
        self.transfer_marketplace_funds(buyer_id, listing.seller_id, listing.price, marketplace_fee)?;

        // Update listing status
        if let Some(listing_mut) = self.software_market.get_mut(&listing_id) {
            listing_mut.status = ListingStatus::Sold;
        }

        // Record market transaction
        self.record_market_transaction(MarketTransaction {
            id: thread_rng().gen::<u64>(),
            transaction_type: MarketTransactionType::Software,
            item_id: listing.software_id as u64,
            buyer_id,
            seller_id: listing.seller_id,
            price: listing.price,
            marketplace_fee,
            timestamp: Utc::now(),
        });

        Ok(PurchaseResult {
            item_type: ItemType::Software,
            item_id: listing.software_id as u64,
            price: listing.price,
            marketplace_fee,
            seller_amount,
            transaction_id: thread_rng().gen::<u64>(),
        })
    }

    /// Get current market prices for hardware/software
    pub fn get_market_prices(&self) -> &MarketPrices {
        &self.current_prices
    }

    /// Update market prices based on supply and demand
    pub fn update_market_prices(&mut self) -> HeResult<()> {
        // Analyze recent transactions to adjust prices
        let now = Utc::now();
        let recent_cutoff = now - Duration::days(7);
        
        let recent_transactions: Vec<&MarketTransaction> = self.market_history
            .iter()
            .filter(|t| t.timestamp > recent_cutoff)
            .collect();

        // Update hardware prices
        self.update_hardware_prices(&recent_transactions);
        
        // Update software prices
        self.update_software_prices(&recent_transactions);

        // Update Bitcoin exchange rate (simulate market volatility)
        self.update_bitcoin_rate();

        Ok(())
    }

    /// Get user's bank account
    pub fn get_bank_account(&self, user_id: UserId) -> Option<&BankAccount> {
        self.bank_accounts.get(&user_id)
    }

    /// Get user's Bitcoin wallet
    pub fn get_bitcoin_wallet(&self, user_id: UserId) -> Option<&BitcoinWallet> {
        self.bitcoin_wallets.get(&user_id)
    }

    /// Get market listings
    pub fn get_hardware_listings(&self, category: Option<HardwareType>) -> Vec<HardwareListing> {
        self.hardware_market
            .values()
            .filter(|listing| {
                listing.status == ListingStatus::Active &&
                listing.expires_at > Utc::now() &&
                category.as_ref().map_or(true, |cat| &listing.hardware_type == cat)
            })
            .cloned()
            .collect()
    }

    /// Get software listings
    pub fn get_software_listings(&self, category: Option<SoftwareType>) -> Vec<SoftwareListing> {
        self.software_market
            .values()
            .filter(|listing| {
                listing.status == ListingStatus::Active &&
                listing.expires_at > Utc::now() &&
                category.as_ref().map_or(true, |cat| &listing.software_type == cat)
            })
            .cloned()
            .collect()
    }

    /// Calculate user's net worth
    pub fn calculate_net_worth(&self, user_id: UserId) -> i64 {
        let mut net_worth = 0;

        // Add bank balance
        if let Some(account) = self.bank_accounts.get(&user_id) {
            net_worth += account.balance;
        }

        // Add Bitcoin value
        if let Some(wallet) = self.bitcoin_wallets.get(&user_id) {
            let bitcoin_value = (wallet.balance * self.bitcoin_exchange_rate) as i64;
            net_worth += bitcoin_value;
        }

        // In a real implementation, would also add hardware/software values

        net_worth
    }

    // Private helper methods

    fn initialize_market_prices(&mut self) {
        self.current_prices = MarketPrices {
            hardware: HardwarePrices {
                cpu_base: 500,
                ram_base: 200,
                hdd_base: 100,
                net_base: 300,
                modifier_factor: 1.0,
            },
            software: SoftwarePrices {
                cracker_base: 1000,
                scanner_base: 800,
                virus_base: 1500,
                exploit_base: 2000,
                firewall_base: 1200,
                antivirus_base: 1000,
                modifier_factor: 1.0,
            },
        };
    }

    fn calculate_daily_transfers(&self, user_id: UserId) -> i64 {
        let today = Utc::now().date_naive();
        
        if let Some(account) = self.bank_accounts.get(&user_id) {
            account.transaction_history
                .iter()
                .filter(|t| {
                    t.timestamp.date_naive() == today &&
                    matches!(t.transaction_type, TransactionType::Transfer)
                })
                .map(|t| t.amount.abs())
                .sum()
        } else {
            0
        }
    }

    fn find_user_by_account_number(&self, account_number: &str) -> Option<UserId> {
        self.bank_accounts
            .iter()
            .find(|(_, account)| account.account_number == account_number)
            .map(|(user_id, _)| *user_id)
    }

    fn calculate_transfer_time(&self, amount: i64) -> u32 {
        // Larger transfers take longer to process
        match amount {
            0..=1000 => 60,        // 1 minute
            1001..=10000 => 300,   // 5 minutes
            10001..=50000 => 900,  // 15 minutes
            50001..=100000 => 1800, // 30 minutes
            _ => 3600,             // 1 hour
        }
    }

    fn generate_bitcoin_address(&self) -> String {
        let mut rng = thread_rng();
        format!("1{:>32}", 
            (0..31).map(|_| {
                let chars = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
                chars[rng.gen_range(0..chars.len())] as char
            }).collect::<String>()
        )
    }

    fn generate_private_key(&self) -> String {
        let mut rng = thread_rng();
        (0..64).map(|_| {
            let chars = b"0123456789abcdef";
            chars[rng.gen_range(0..chars.len())] as char
        }).collect()
    }

    fn calculate_bitcoin_mining_reward(&self, mining_power: u32, duration_hours: u32) -> f64 {
        // Simplified mining calculation
        let base_rate = 0.00001; // BTC per hour per unit of power
        let power_efficiency = (mining_power as f64).sqrt(); // Diminishing returns
        let time_factor = duration_hours as f64;
        
        base_rate * power_efficiency * time_factor
    }

    fn calculate_mining_efficiency(&self, mining_power: u32) -> f64 {
        // Efficiency decreases with very high power due to heat/complexity
        let optimal_power = 1000.0;
        let efficiency = if mining_power as f64 <= optimal_power {
            mining_power as f64 / optimal_power
        } else {
            1.0 - ((mining_power as f64 - optimal_power) / (optimal_power * 10.0))
        };
        
        efficiency.max(0.1).min(1.0)
    }

    fn get_current_bitcoin_rate(&self) -> f64 {
        self.bitcoin_exchange_rate
    }

    fn get_hardware_type(&self, _hardware_id: HardwareId) -> HardwareType {
        // Simplified - would look up actual hardware type
        HardwareType::CPU
    }

    fn get_hardware_specs(&self, _hardware_id: HardwareId) -> HardwareSpecs {
        // Simplified - would look up actual specs
        HardwareSpecs {
            power: 100,
            efficiency: 85,
            condition: 95,
        }
    }

    fn get_software_type(&self, _software_id: SoftwareId) -> SoftwareType {
        // Simplified - would look up actual software type
        SoftwareType::Cracker
    }

    fn get_software_version(&self, _software_id: SoftwareId) -> String {
        "1.0".to_string()
    }

    fn get_software_power(&self, _software_id: SoftwareId) -> u32 {
        100
    }

    fn transfer_marketplace_funds(&mut self, buyer_id: UserId, seller_id: UserId, total_price: i64, marketplace_fee: i64) -> HeResult<()> {
        let seller_amount = total_price - marketplace_fee;

        // Deduct from buyer
        if let Some(buyer_account) = self.bank_accounts.get_mut(&buyer_id) {
            buyer_account.balance -= total_price;
            buyer_account.transaction_history.push(BankTransaction {
                id: thread_rng().gen::<u64>(),
                transaction_type: TransactionType::Purchase,
                amount: -total_price,
                description: "Marketplace purchase".to_string(),
                timestamp: Utc::now(),
                balance_after: buyer_account.balance,
            });
        }

        // Add to seller
        if let Some(seller_account) = self.bank_accounts.get_mut(&seller_id) {
            seller_account.balance += seller_amount;
            seller_account.transaction_history.push(BankTransaction {
                id: thread_rng().gen::<u64>(),
                transaction_type: TransactionType::Sale,
                amount: seller_amount,
                description: format!("Marketplace sale (Fee: ${})", marketplace_fee),
                timestamp: Utc::now(),
                balance_after: seller_account.balance,
            });
        }

        Ok(())
    }

    fn record_market_transaction(&mut self, transaction: MarketTransaction) {
        self.market_history.push(transaction);
        
        // Keep only last 1000 transactions for performance
        if self.market_history.len() > 1000 {
            self.market_history.remove(0);
        }
    }

    fn update_hardware_prices(&mut self, recent_transactions: &[&MarketTransaction]) {
        // Analyze hardware transaction volume and adjust prices
        let hardware_transactions: Vec<&MarketTransaction> = recent_transactions
            .iter()
            .filter(|t| matches!(t.transaction_type, MarketTransactionType::Hardware))
            .cloned()
            .collect();

        if !hardware_transactions.is_empty() {
            let avg_price = hardware_transactions.iter().map(|t| t.price).sum::<i64>() / hardware_transactions.len() as i64;
            let volume_factor = (hardware_transactions.len() as f64 / 100.0).min(2.0);
            
            // Adjust base prices based on market activity
            self.current_prices.hardware.modifier_factor = (avg_price as f64 / 1000.0) * volume_factor;
        }
    }

    fn update_software_prices(&mut self, recent_transactions: &[&MarketTransaction]) {
        // Similar to hardware price updates
        let software_transactions: Vec<&MarketTransaction> = recent_transactions
            .iter()
            .filter(|t| matches!(t.transaction_type, MarketTransactionType::Software))
            .cloned()
            .collect();

        if !software_transactions.is_empty() {
            let avg_price = software_transactions.iter().map(|t| t.price).sum::<i64>() / software_transactions.len() as i64;
            let volume_factor = (software_transactions.len() as f64 / 100.0).min(2.0);
            
            self.current_prices.software.modifier_factor = (avg_price as f64 / 1500.0) * volume_factor;
        }
    }

    fn update_bitcoin_rate(&mut self) {
        // Simulate Bitcoin price volatility
        let mut rng = thread_rng();
        let volatility = rng.gen_range(0.95..=1.05); // ±5% change
        self.bitcoin_exchange_rate *= volatility;
        
        // Keep within reasonable bounds
        self.bitcoin_exchange_rate = self.bitcoin_exchange_rate.max(1000.0).min(10000.0);
    }
}

// Data structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankAccount {
    pub user_id: UserId,
    pub account_number: String,
    pub balance: i64,
    pub created_at: DateTime<Utc>,
    pub last_transaction: DateTime<Utc>,
    pub transaction_history: Vec<BankTransaction>,
    pub daily_transfer_limit: i64,
    pub interest_rate: f64,
    pub account_type: BankAccountType,
    pub frozen: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BankAccountType {
    Standard,
    Premium,
    Corporate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankTransaction {
    pub id: u64,
    pub transaction_type: TransactionType,
    pub amount: i64, // Negative for outgoing, positive for incoming
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub balance_after: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,
    Received,
    Purchase,
    Sale,
    Exchange,
    Interest,
    Fee,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinWallet {
    pub user_id: UserId,
    pub address: String,
    pub balance: f64, // Bitcoin amount
    pub created_at: DateTime<Utc>,
    pub transaction_history: Vec<BitcoinTransaction>,
    pub private_key: String,
    pub last_mining_attempt: Option<DateTime<Utc>>,
    pub mining_power: u32,
    pub total_mined: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinTransaction {
    pub id: u64,
    pub transaction_type: BitcoinTransactionType,
    pub amount: f64, // Bitcoin amount (negative for outgoing)
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub confirmations: u32,
    pub fee: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BitcoinTransactionType {
    Mining,
    Send,
    Receive,
    Exchange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankTransfer {
    pub id: u64,
    pub from_user_id: UserId,
    pub to_user_id: UserId,
    pub from_account: String,
    pub to_account: String,
    pub amount: i64,
    pub fee: i64,
    pub description: String,
    pub status: TransferStatus,
    pub initiated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub processing_time: u32, // in seconds
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransferStatus {
    Processing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareListing {
    pub id: u64,
    pub seller_id: UserId,
    pub hardware_id: HardwareId,
    pub hardware_type: HardwareType,
    pub specifications: HardwareSpecs,
    pub price: i64,
    pub description: String,
    pub listed_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: ListingStatus,
    pub views: u32,
    pub watchers: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HardwareType {
    CPU,
    RAM,
    HDD,
    Network,
    Firewall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSpecs {
    pub power: u32,
    pub efficiency: u32,
    pub condition: u32, // 0-100%
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareListing {
    pub id: u64,
    pub seller_id: UserId,
    pub software_id: SoftwareId,
    pub software_type: SoftwareType,
    pub version: String,
    pub power_rating: u32,
    pub price: i64,
    pub description: String,
    pub listed_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: ListingStatus,
    pub views: u32,
    pub watchers: u32,
    pub license_type: LicenseType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SoftwareType {
    Cracker,
    Scanner,
    Virus,
    Exploit,
    Firewall,
    Antivirus,
    LogRemover,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LicenseType {
    SingleUse,
    MultiUse,
    Unlimited,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ListingStatus {
    Active,
    Sold,
    Expired,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTransaction {
    pub id: u64,
    pub transaction_type: MarketTransactionType,
    pub item_id: u64,
    pub buyer_id: UserId,
    pub seller_id: UserId,
    pub price: i64,
    pub marketplace_fee: i64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketTransactionType {
    Hardware,
    Software,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarketPrices {
    pub hardware: HardwarePrices,
    pub software: SoftwarePrices,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HardwarePrices {
    pub cpu_base: i64,
    pub ram_base: i64,
    pub hdd_base: i64,
    pub net_base: i64,
    pub modifier_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SoftwarePrices {
    pub cracker_base: i64,
    pub scanner_base: i64,
    pub virus_base: i64,
    pub exploit_base: i64,
    pub firewall_base: i64,
    pub antivirus_base: i64,
    pub modifier_factor: f64,
}

// Result types

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferResult {
    pub transfer_id: u64,
    pub status: TransferStatus,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BitcoinMiningResult {
    pub reward: f64,
    pub mining_power: u32,
    pub duration_hours: u32,
    pub efficiency: f64,
    pub estimated_next_reward: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExchangeResult {
    pub bitcoin_amount: f64,
    pub exchange_rate: f64,
    pub gross_money: i64,
    pub fee: i64,
    pub net_money: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurchaseResult {
    pub item_type: ItemType,
    pub item_id: u64,
    pub price: i64,
    pub marketplace_fee: i64,
    pub seller_amount: i64,
    pub transaction_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ItemType {
    Hardware,
    Software,
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bank_account_creation() {
        let mut economy = EconomySystem::new();
        let account = economy.create_bank_account(1, 10000).unwrap();
        
        assert_eq!(account.user_id, 1);
        assert_eq!(account.balance, 10000);
        assert_eq!(account.account_number.len(), 10);
    }

    #[test]
    fn test_bitcoin_wallet_creation() {
        let mut economy = EconomySystem::new();
        let wallet = economy.create_bitcoin_wallet(1).unwrap();
        
        assert_eq!(wallet.user_id, 1);
        assert_eq!(wallet.balance, 0.0);
        assert!(wallet.address.starts_with('1'));
        assert_eq!(wallet.private_key.len(), 64);
    }

    #[test]
    fn test_money_transfer() {
        let mut economy = EconomySystem::new();
        
        // Create accounts
        economy.create_bank_account(1, 10000).unwrap();
        economy.create_bank_account(2, 1000).unwrap();
        
        let sender_account = economy.get_bank_account(1).unwrap().account_number.clone();
        let receiver_account = economy.get_bank_account(2).unwrap().account_number.clone();
        
        let transfer = economy.transfer_money(1, &receiver_account, 5000, None).unwrap();
        assert_eq!(transfer.amount, 5000);
        assert!(transfer.fee > 0);
    }

    #[test]
    fn test_bitcoin_mining() {
        let mut economy = EconomySystem::new();
        economy.create_bitcoin_wallet(1).unwrap();
        
        let result = economy.mine_bitcoin(1, 100, 1).unwrap();
        assert!(result.reward > 0.0);
        assert_eq!(result.mining_power, 100);
        assert_eq!(result.duration_hours, 1);
    }

    #[test]
    fn test_marketplace_listing() {
        let mut economy = EconomySystem::new();
        
        let listing = economy.list_hardware_for_sale(
            1, 
            123, 
            5000, 
            "High-performance CPU".to_string()
        ).unwrap();
        
        assert_eq!(listing.seller_id, 1);
        assert_eq!(listing.hardware_id, 123);
        assert_eq!(listing.price, 5000);
        assert_eq!(listing.status, ListingStatus::Active);
    }
}