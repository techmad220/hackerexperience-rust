//! Bank and financial system models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;
use he_core::id::AccountId;

/// Bank account model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankAccount {
    pub account_id: AccountId,
    pub account_number: String,
    pub balance: i64, // Balance in cents to avoid floating point issues
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

/// Bank transaction model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankTransaction {
    pub id: uuid::Uuid,
    pub transaction_type: TransactionType,
    pub amount: i64, // Negative for outgoing, positive for incoming
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub balance_after: i64,
    pub from_account: Option<String>,
    pub to_account: Option<String>,
    pub reference: Option<String>,
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
    Deposit,
    Withdrawal,
}

/// Bank transfer model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankTransfer {
    pub id: uuid::Uuid,
    pub from_account_id: AccountId,
    pub to_account_number: String,
    pub amount: i64,
    pub fee: i64,
    pub description: String,
    pub status: TransferStatus,
    pub initiated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub processing_time: u32, // in seconds
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransferStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

/// Bank account creation parameters
#[derive(Debug, Clone, Validate)]
pub struct CreateBankAccountParams {
    pub account_id: AccountId,
    pub initial_balance: i64,
    pub account_type: BankAccountType,
}

/// Transfer request parameters
#[derive(Debug, Clone, Validate)]
pub struct TransferRequest {
    pub from_account_id: AccountId,
    pub to_account_number: String,
    #[validate(range(min = 1))]
    pub amount: i64,
    pub description: Option<String>,
}

/// Deposit request parameters
#[derive(Debug, Clone, Validate)]
pub struct DepositRequest {
    pub account_id: AccountId,
    #[validate(range(min = 1))]
    pub amount: i64,
    pub description: String,
}

/// Withdrawal request parameters
#[derive(Debug, Clone, Validate)]
pub struct WithdrawalRequest {
    pub account_id: AccountId,
    #[validate(range(min = 1))]
    pub amount: i64,
    pub description: String,
}

/// Bank account balance response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub account_id: AccountId,
    pub balance: i64,
    pub available_balance: i64, // Balance minus pending transactions
    pub last_transaction: DateTime<Utc>,
}

/// Transfer result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResult {
    pub transfer_id: uuid::Uuid,
    pub status: TransferStatus,
    pub message: String,
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Bank operation errors
#[derive(Debug, thiserror::Error)]
pub enum BankError {
    #[error("Account not found")]
    AccountNotFound,
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Account is frozen")]
    AccountFrozen,
    #[error("Daily transfer limit exceeded")]
    DailyLimitExceeded,
    #[error("Invalid account number")]
    InvalidAccountNumber,
    #[error("Transfer not found")]
    TransferNotFound,
    #[error("Invalid transfer state")]
    InvalidTransferState,
    #[error("Account already exists")]
    AccountAlreadyExists,
    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl BankAccount {
    pub fn new(
        account_id: AccountId,
        initial_balance: i64,
        account_type: BankAccountType,
    ) -> Self {
        let now = Utc::now();
        let account_number = Self::generate_account_number();
        
        Self {
            account_id,
            account_number,
            balance: initial_balance,
            created_at: now,
            last_transaction: now,
            transaction_history: Vec::new(),
            daily_transfer_limit: match account_type {
                BankAccountType::Standard => 50_000_00, // $50,000
                BankAccountType::Premium => 200_000_00, // $200,000
                BankAccountType::Corporate => 1_000_000_00, // $1,000,000
            },
            interest_rate: match account_type {
                BankAccountType::Standard => 0.01,
                BankAccountType::Premium => 0.015,
                BankAccountType::Corporate => 0.02,
            },
            account_type,
            frozen: false,
        }
    }

    fn generate_account_number() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("{:010}", rng.gen::<u32>())
    }

    /// Calculate available balance (balance minus pending outgoing transfers)
    pub fn available_balance(&self, pending_outgoing: i64) -> i64 {
        self.balance - pending_outgoing
    }

    /// Check if account can afford a transaction
    pub fn can_afford(&self, amount: i64, pending_outgoing: i64) -> bool {
        self.available_balance(pending_outgoing) >= amount
    }

    /// Add a transaction to the account history
    pub fn add_transaction(&mut self, transaction: BankTransaction) {
        self.balance = transaction.balance_after;
        self.last_transaction = transaction.timestamp;
        self.transaction_history.push(transaction);

        // Keep only the last 1000 transactions to manage memory
        if self.transaction_history.len() > 1000 {
            self.transaction_history.remove(0);
        }
    }

    /// Calculate daily transfer amount for today
    pub fn daily_transfer_amount(&self) -> i64 {
        let today = Utc::now().date_naive();
        
        self.transaction_history
            .iter()
            .filter(|t| {
                t.timestamp.date_naive() == today &&
                matches!(t.transaction_type, TransactionType::Transfer) &&
                t.amount < 0 // Outgoing transfers
            })
            .map(|t| t.amount.abs())
            .sum()
    }

    /// Check if daily limit allows this transfer
    pub fn can_transfer_today(&self, amount: i64) -> bool {
        self.daily_transfer_amount() + amount <= self.daily_transfer_limit
    }
}

impl BankTransfer {
    pub fn new(
        from_account_id: AccountId,
        to_account_number: String,
        amount: i64,
        description: String,
    ) -> Self {
        let fee = Self::calculate_fee(amount);
        let processing_time = Self::calculate_processing_time(amount);

        Self {
            id: uuid::Uuid::new_v4(),
            from_account_id,
            to_account_number,
            amount,
            fee,
            description,
            status: TransferStatus::Pending,
            initiated_at: Utc::now(),
            completed_at: None,
            processing_time,
            failure_reason: None,
        }
    }

    /// Calculate transfer fee based on amount
    fn calculate_fee(amount: i64) -> i64 {
        // 0.1% fee with minimum $1.00 and maximum $100.00
        let fee = (amount as f64 * 0.001) as i64;
        fee.max(100).min(10_000) // $1.00 to $100.00 in cents
    }

    /// Calculate processing time based on amount
    fn calculate_processing_time(amount: i64) -> u32 {
        match amount {
            0..=1_000_00 => 60,        // $0-$1,000: 1 minute
            1_000_01..=10_000_00 => 300,   // $1,000-$10,000: 5 minutes
            10_000_01..=50_000_00 => 900,  // $10,000-$50,000: 15 minutes
            50_000_01..=100_000_00 => 1800, // $50,000-$100,000: 30 minutes
            _ => 3600,                     // >$100,000: 1 hour
        }
    }

    /// Get estimated completion time
    pub fn estimated_completion(&self) -> DateTime<Utc> {
        self.initiated_at + chrono::Duration::seconds(self.processing_time as i64)
    }

    /// Check if transfer is ready to be processed
    pub fn is_ready_to_process(&self) -> bool {
        self.status == TransferStatus::Processing &&
        Utc::now() >= self.estimated_completion()
    }

    /// Total amount including fee
    pub fn total_amount(&self) -> i64 {
        self.amount + self.fee
    }
}