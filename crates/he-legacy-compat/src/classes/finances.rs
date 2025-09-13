// FINANCES.CLASS.PHP PORT - Banking system with Bitcoin integration
// Original: Complex financial system with bank accounts, Bitcoin wallets, and money transfers

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use sqlx::MySqlPool;
use serde::{Deserialize, Serialize};
use he_core::*;
use he_db::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankAccount {
    pub id: i64,
    pub bank_id: i64,
    pub bank_user: i64,
    pub account_number: String,
    pub balance: i64,
    pub password: String,
    pub creation_date: DateTime<Utc>,
    pub expire_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinWallet {
    pub id: i64,
    pub user_id: i64,
    pub npc_id: i64, // Bitcoin exchange NPC
    pub address: String,
    pub key: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoneyTransfer {
    pub id: i64,
    pub from_account: i64,
    pub to_account: i64,
    pub amount: i64,
    pub from_bank: i64,
    pub to_bank: i64,
    pub from_user: i64,
    pub to_user: i64,
    pub user_ip: String,
    pub transfer_date: DateTime<Utc>,
}

pub struct Finances {
    db_pool: MySqlPool,
}

impl Finances {
    pub fn new(db_pool: MySqlPool) -> Self {
        Self { db_pool }
    }
    
    // Original PHP: isUserRegisteredOnBank - Check if user has account at bank
    pub async fn is_user_registered_on_bank(&self, uid: i64, bank_id: i64) -> Result<bool, FinancesError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM bankAccounts WHERE bankID = ? AND bankUser = ?"
        )
        .bind(bank_id)
        .bind(uid)
        .fetch_one(&self.db_pool)
        .await
        .map_err(FinancesError::DatabaseError)?;
        
        Ok(count > 0)
    }
    
    // Original PHP: bitcoin_getValue - Get Bitcoin price from blockchain.info
    pub async fn bitcoin_get_value(&self) -> Result<i32, FinancesError> {
        // TODO: Make HTTP request to blockchain.info API
        // For now, return fallback price from original
        let price = 472; // Original fallback price
        
        // let response = reqwest::get("https://blockchain.info/q/24hrprice").await?;
        // let price_str = response.text().await?;
        // let price = price_str.parse::<f64>().unwrap_or(472.0) as i32;
        
        Ok(price.max(1)) // Ensure positive price
    }
    
    // Original PHP: bitcoin_getID - Get Bitcoin exchange NPC ID
    pub async fn bitcoin_get_id(&self) -> Result<i64, FinancesError> {
        let npc_id = sqlx::query_scalar::<_, Option<i64>>(
            "SELECT id FROM npc WHERE npcType = 40 LIMIT 1"
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(FinancesError::DatabaseError)?
        .flatten();
        
        npc_id.ok_or(FinancesError::BitcoinExchangeNotFound)
    }
    
    // Original PHP: bitcoin_sell - Sell Bitcoin for money
    pub async fn bitcoin_sell(&self, address: String, amount: f64, rate: f64, account_id: i64) -> Result<(), FinancesError> {
        // Update wallet balance
        sqlx::query("UPDATE bitcoin_wallets SET amount = amount - ? WHERE address = ?")
            .bind(amount)
            .bind(&address)
            .execute(&self.db_pool)
            .await
            .map_err(FinancesError::DatabaseError)?;
        
        // Add money to bank account
        let money_amount = (amount * rate).ceil() as i64;
        self.add_money(money_amount, account_id).await?;
        
        Ok(())
    }
    
    // Original PHP: bitcoin_buy - Buy Bitcoin with money
    pub async fn bitcoin_buy(&self, address: String, amount: f64, rate: f64, account_id: i64) -> Result<(), FinancesError> {
        let cost = (amount * rate).ceil() as i64;
        
        // Debit money from account
        if !self.debt_money(cost, account_id).await? {
            return Err(FinancesError::InsufficientFunds);
        }
        
        // Add Bitcoin to wallet
        sqlx::query("UPDATE bitcoin_wallets SET amount = amount + ? WHERE address = ?")
            .bind(amount)
            .bind(&address)
            .execute(&self.db_pool)
            .await
            .map_err(FinancesError::DatabaseError)?;
        
        Ok(())
    }
    
    // Original PHP: getWalletInfo - Get Bitcoin wallet information
    pub async fn get_wallet_info(&self, btc_id: i64, uid: Option<i64>) -> Result<BitcoinWallet, FinancesError> {
        let user_id = uid.unwrap_or_else(|| {
            // TODO: Get from session
            0 // placeholder
        });
        
        let wallet = sqlx::query_as::<_, BitcoinWallet>(
            "SELECT id, user_id, npc_id, address, key, amount FROM bitcoin_wallets WHERE npcID = ? AND userID = ? LIMIT 1"
        )
        .bind(btc_id)
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(FinancesError::DatabaseError)?;
        
        wallet.ok_or(FinancesError::WalletNotFound)
    }
    
    // Original PHP: getWalletInfoByAddress - Get wallet by address
    pub async fn get_wallet_info_by_address(&self, address: String) -> Result<BitcoinWallet, FinancesError> {
        let wallet = sqlx::query_as::<_, BitcoinWallet>(
            "SELECT id, user_id, npc_id, address, key, amount FROM bitcoin_wallets WHERE address = ? LIMIT 1"
        )
        .bind(&address)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(FinancesError::DatabaseError)?;
        
        wallet.ok_or(FinancesError::WalletNotFound)
    }
    
    // Original PHP: issetWallet - Check if wallet exists
    pub async fn wallet_exists(&self, address: String) -> Result<bool, FinancesError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM bitcoin_wallets WHERE address = ?"
        )
        .bind(&address)
        .fetch_one(&self.db_pool)
        .await
        .map_err(FinancesError::DatabaseError)?;
        
        Ok(count > 0)
    }
    
    // Original PHP: bitcoin_createAcc - Create Bitcoin wallet account
    pub async fn bitcoin_create_account(&self, btc_id: i64, user_id: i64) -> Result<String, FinancesError> {
        // Generate unique wallet address and key
        let address = Self::generate_bitcoin_address();
        let key = Self::generate_wallet_key();
        
        sqlx::query(
            "INSERT INTO bitcoin_wallets (userID, npcID, address, key, amount) VALUES (?, ?, ?, ?, 0.0)"
        )
        .bind(user_id)
        .bind(btc_id)
        .bind(&address)
        .bind(&key)
        .execute(&self.db_pool)
        .await
        .map_err(FinancesError::DatabaseError)?;
        
        Ok(address)
    }
    
    // Original PHP: bitcoin_transfer - Transfer Bitcoin between wallets
    pub async fn bitcoin_transfer(&self, address_from: String, amount: f64, address_to: String) -> Result<(), FinancesError> {
        // Check if both wallets exist
        if !self.wallet_exists(address_from.clone()).await? {
            return Err(FinancesError::WalletNotFound);
        }
        if !self.wallet_exists(address_to.clone()).await? {
            return Err(FinancesError::WalletNotFound);
        }
        
        // Check if sender has enough Bitcoin
        let from_wallet = self.get_wallet_info_by_address(address_from.clone()).await?;
        if from_wallet.amount < amount {
            return Err(FinancesError::InsufficientFunds);
        }
        
        // Perform transfer
        let mut tx = self.db_pool.begin().await.map_err(FinancesError::DatabaseError)?;
        
        sqlx::query("UPDATE bitcoin_wallets SET amount = amount - ? WHERE address = ?")
            .bind(amount)
            .bind(&address_from)
            .execute(&mut *tx)
            .await
            .map_err(FinancesError::DatabaseError)?;
            
        sqlx::query("UPDATE bitcoin_wallets SET amount = amount + ? WHERE address = ?")
            .bind(amount)
            .bind(&address_to)
            .execute(&mut *tx)
            .await
            .map_err(FinancesError::DatabaseError)?;
        
        tx.commit().await.map_err(FinancesError::DatabaseError)?;
        
        Ok(())
    }
    
    // Original PHP: userHaveWallet - Check if user has Bitcoin wallet
    pub async fn user_have_wallet(&self, uid: i64, btc_id: i64) -> Result<bool, FinancesError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM bitcoin_wallets WHERE userID = ? AND npcID = ?"
        )
        .bind(uid)
        .bind(btc_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(FinancesError::DatabaseError)?;
        
        Ok(count > 0)
    }
    
    // Original PHP: issetBankAccount - Check if bank account exists
    pub async fn bank_account_exists(&self, bank_acc: String, bank_id: i64) -> Result<bool, FinancesError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM bankAccounts WHERE id = ? AND bankID = ?"
        )
        .bind(&bank_acc)
        .bind(bank_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(FinancesError::DatabaseError)?;
        
        Ok(count > 0)
    }
    
    // Original PHP: setExpireDate - Set account expiration date
    pub async fn set_expire_date(&self, account_id: i64, delete_in_hours: i32) -> Result<(), FinancesError> {
        sqlx::query("UPDATE bankAccounts SET expireDate = DATE_ADD(NOW(), INTERVAL ? HOUR) WHERE id = ?")
            .bind(delete_in_hours)
            .bind(account_id)
            .execute(&self.db_pool)
            .await
            .map_err(FinancesError::DatabaseError)?;
        
        Ok(())
    }
    
    // Original PHP: closeAccount - Close bank account
    pub async fn close_account(&self, bank_acc: i64) -> Result<(), FinancesError> {
        sqlx::query("DELETE FROM bankAccounts WHERE id = ?")
            .bind(bank_acc)
            .execute(&self.db_pool)
            .await
            .map_err(FinancesError::DatabaseError)?;
        
        Ok(())
    }
    
    // Original PHP: listBankAccounts - Get user's bank accounts
    pub async fn list_bank_accounts(&self, uid: i64) -> Result<Vec<BankAccount>, FinancesError> {
        let accounts = sqlx::query_as::<_, BankAccount>(
            "SELECT id, bank_id, bank_user, account_number, balance, password, creation_date, expire_date 
             FROM bankAccounts 
             WHERE bankUser = ? 
             ORDER BY creation_date"
        )
        .bind(uid)
        .fetch_all(&self.db_pool)
        .await
        .map_err(FinancesError::DatabaseError)?;
        
        Ok(accounts)
    }
    
    // Original PHP: bankAccountInfo - Get bank account information
    pub async fn bank_account_info(&self, bank_acc: i64) -> Result<BankAccount, FinancesError> {
        let account = sqlx::query_as::<_, BankAccount>(
            "SELECT id, bank_id, bank_user, account_number, balance, password, creation_date, expire_date 
             FROM bankAccounts 
             WHERE id = ? 
             LIMIT 1"
        )
        .bind(bank_acc)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(FinancesError::DatabaseError)?;
        
        account.ok_or(FinancesError::AccountNotFound(bank_acc))
    }
    
    // Original PHP: transferMoney - Transfer money between accounts
    pub async fn transfer_money(
        &self,
        from_acc: i64,
        to_acc: i64,
        amount: i64,
        bank_from: i64,
        bank_to: i64,
        user_to: i64,
        user_ip: String,
    ) -> Result<(), FinancesError> {
        // Get current user ID from session
        // TODO: Get from session
        let user_from = 0i64; // placeholder
        
        // Verify accounts exist
        let from_account = self.bank_account_info(from_acc).await?;
        let to_account = self.bank_account_info(to_acc).await?;
        
        if from_account.balance < amount {
            return Err(FinancesError::InsufficientFunds);
        }
        
        // Perform transfer
        let mut tx = self.db_pool.begin().await.map_err(FinancesError::DatabaseError)?;
        
        // Debit from source account
        sqlx::query("UPDATE bankAccounts SET balance = balance - ? WHERE id = ?")
            .bind(amount)
            .bind(from_acc)
            .execute(&mut *tx)
            .await
            .map_err(FinancesError::DatabaseError)?;
        
        // Credit to destination account
        sqlx::query("UPDATE bankAccounts SET balance = balance + ? WHERE id = ?")
            .bind(amount)
            .bind(to_acc)
            .execute(&mut *tx)
            .await
            .map_err(FinancesError::DatabaseError)?;
        
        // Log transfer
        sqlx::query(
            "INSERT INTO money_transfers (from_account, to_account, amount, from_bank, to_bank, from_user, to_user, user_ip, transfer_date) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, NOW())"
        )
        .bind(from_acc)
        .bind(to_acc)
        .bind(amount)
        .bind(bank_from)
        .bind(bank_to)
        .bind(user_from)
        .bind(user_to)
        .bind(&user_ip)
        .execute(&mut *tx)
        .await
        .map_err(FinancesError::DatabaseError)?;
        
        tx.commit().await.map_err(FinancesError::DatabaseError)?;
        
        Ok(())
    }
    
    // Original PHP: addMoney - Add money to account
    pub async fn add_money(&self, amount: i64, bank_acc: i64) -> Result<(), FinancesError> {
        sqlx::query("UPDATE bankAccounts SET balance = balance + ? WHERE id = ?")
            .bind(amount)
            .bind(bank_acc)
            .execute(&self.db_pool)
            .await
            .map_err(FinancesError::DatabaseError)?;
        
        Ok(())
    }
    
    // Original PHP: debtMoney - Debit money from account
    pub async fn debt_money(&self, amount: i64, bank_acc: i64) -> Result<bool, FinancesError> {
        let account = self.bank_account_info(bank_acc).await?;
        
        if account.balance < amount {
            return Ok(false); // Insufficient funds
        }
        
        sqlx::query("UPDATE bankAccounts SET balance = balance - ? WHERE id = ?")
            .bind(amount)
            .bind(bank_acc)
            .execute(&self.db_pool)
            .await
            .map_err(FinancesError::DatabaseError)?;
        
        Ok(true)
    }
    
    // Original PHP: totalMoney - Get user's total money across all accounts
    pub async fn total_money(&self, uid: i64) -> Result<i64, FinancesError> {
        let total = sqlx::query_scalar::<_, Option<i64>>(
            "SELECT SUM(balance) FROM bankAccounts WHERE bankUser = ?"
        )
        .bind(uid)
        .fetch_one(&self.db_pool)
        .await
        .map_err(FinancesError::DatabaseError)?;
        
        Ok(total.unwrap_or(0))
    }
    
    // Create account for new user (called during registration)
    pub async fn create_account(&self, user_id: i64) -> Result<(), FinancesError> {
        // Create default bank account with starter money
        let bank_id = 1; // Default bank ID
        let account_number = Self::generate_account_number();
        let initial_balance = 1000; // Starter money
        let password = Self::generate_account_password();
        
        sqlx::query(
            "INSERT INTO bankAccounts (bankID, bankUser, account_number, balance, password, creation_date) 
             VALUES (?, ?, ?, ?, ?, NOW())"
        )
        .bind(bank_id)
        .bind(user_id)
        .bind(&account_number)
        .bind(initial_balance)
        .bind(&password)
        .execute(&self.db_pool)
        .await
        .map_err(FinancesError::DatabaseError)?;
        
        Ok(())
    }
    
    // Helper functions for generating unique identifiers
    fn generate_bitcoin_address() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let chars: Vec<char> = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".chars().collect();
        (0..34).map(|_| chars[rng.gen_range(0..chars.len())]).collect()
    }
    
    fn generate_wallet_key() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let chars: Vec<char> = "0123456789abcdef".chars().collect();
        (0..64).map(|_| chars[rng.gen_range(0..chars.len())]).collect()
    }
    
    fn generate_account_number() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("{:010}", rng.gen_range(1000000000..9999999999u64))
    }
    
    fn generate_account_password() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let chars: Vec<char> = "0123456789".chars().collect();
        (0..8).map(|_| chars[rng.gen_range(0..chars.len())]).collect()
    }
}

// Implement FromRow for BankAccount
impl sqlx::FromRow<'_, sqlx::mysql::MySqlRow> for BankAccount {
    fn from_row(row: &'_ sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(BankAccount {
            id: row.try_get("id")?,
            bank_id: row.try_get("bank_id")?,
            bank_user: row.try_get("bank_user")?,
            account_number: row.try_get("account_number")?,
            balance: row.try_get("balance")?,
            password: row.try_get("password")?,
            creation_date: row.try_get("creation_date")?,
            expire_date: row.try_get("expire_date")?,
        })
    }
}

// Implement FromRow for BitcoinWallet
impl sqlx::FromRow<'_, sqlx::mysql::MySqlRow> for BitcoinWallet {
    fn from_row(row: &'_ sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(BitcoinWallet {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            npc_id: row.try_get("npc_id")?,
            address: row.try_get("address")?,
            key: row.try_get("key")?,
            amount: row.try_get("amount")?,
        })
    }
}

#[derive(Debug)]
pub enum FinancesError {
    DatabaseError(sqlx::Error),
    AccountNotFound(i64),
    WalletNotFound,
    InsufficientFunds,
    BitcoinExchangeNotFound,
    InvalidAmount,
}

impl std::fmt::Display for FinancesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FinancesError::DatabaseError(e) => write!(f, "Database error: {}", e),
            FinancesError::AccountNotFound(id) => write!(f, "Account {} not found", id),
            FinancesError::WalletNotFound => write!(f, "Bitcoin wallet not found"),
            FinancesError::InsufficientFunds => write!(f, "Insufficient funds"),
            FinancesError::BitcoinExchangeNotFound => write!(f, "Bitcoin exchange NPC not found"),
            FinancesError::InvalidAmount => write!(f, "Invalid amount"),
        }
    }
}

impl std::error::Error for FinancesError {}