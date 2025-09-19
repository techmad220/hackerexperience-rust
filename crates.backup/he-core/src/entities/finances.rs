use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FinanceError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Insufficient funds: need {needed}, have {available}")]
    InsufficientFunds { needed: u64, available: u64 },
    #[error("Invalid amount: {0}")]
    InvalidAmount(i64),
    #[error("Account not found: {0}")]
    AccountNotFound(u64),
    #[error("Transaction not found: {0}")]
    TransactionNotFound(u64),
    #[error("Invalid transaction type: {0}")]
    InvalidTransactionType(String),
    #[error("Permission denied")]
    PermissionDenied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,
    Income,
    Expense,
    Mission,
    Purchase,
    Sale,
    Hack,
    Penalty,
    Bonus,
    Interest,
    Refund,
    Premium,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Completed,
    Failed,
    Cancelled,
    Disputed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinanceAccount {
    pub user_id: u64,
    pub balance: u64,
    pub frozen_balance: u64,
    pub total_income: u64,
    pub total_expenses: u64,
    pub last_updated: DateTime<Utc>,
    pub is_suspended: bool,
    pub daily_limit: Option<u64>,
    pub monthly_limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: u64,
    pub from_user_id: Option<u64>,
    pub to_user_id: Option<u64>,
    pub amount: u64,
    pub transaction_type: TransactionType,
    pub status: TransactionStatus,
    pub description: String,
    pub reference_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub fee: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub total_transactions: u64,
    pub total_income: u64,
    pub total_expenses: u64,
    pub net_amount: i64,
    pub average_transaction: u64,
    pub largest_transaction: u64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinanceFilter {
    pub transaction_type: Option<TransactionType>,
    pub status: Option<TransactionStatus>,
    pub from_user_id: Option<u64>,
    pub to_user_id: Option<u64>,
    pub min_amount: Option<u64>,
    pub max_amount: Option<u64>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub reference_id: Option<String>,
}

/// Financial management system ported from PHP Finances class
/// Handles user balances, transactions, and financial operations
pub struct Finances {
    current_user_id: Option<u64>,
}

impl Finances {
    /// Create new Finances instance
    pub fn new(current_user_id: Option<u64>) -> Self {
        Self { current_user_id }
    }

    /// Get user's financial account
    pub fn get_account(&self, user_id: u64) -> Result<FinanceAccount, FinanceError> {
        // Simulate database lookup
        Ok(FinanceAccount {
            user_id,
            balance: 1000, // Mock balance
            frozen_balance: 0,
            total_income: 5000,
            total_expenses: 4000,
            last_updated: Utc::now(),
            is_suspended: false,
            daily_limit: Some(10000),
            monthly_limit: Some(100000),
        })
    }

    /// Get current user's account
    pub fn get_my_account(&self) -> Result<FinanceAccount, FinanceError> {
        match self.current_user_id {
            Some(user_id) => self.get_account(user_id),
            None => Err(FinanceError::PermissionDenied),
        }
    }

    /// Transfer money between users
    pub fn transfer(
        &self,
        from_user_id: u64,
        to_user_id: u64,
        amount: u64,
        description: String,
        reference_id: Option<String>,
    ) -> Result<Transaction, FinanceError> {
        if amount == 0 {
            return Err(FinanceError::InvalidAmount(0));
        }

        // Check permissions
        if let Some(current_user) = self.current_user_id {
            if current_user != from_user_id {
                return Err(FinanceError::PermissionDenied);
            }
        } else {
            return Err(FinanceError::PermissionDenied);
        }

        // Get accounts
        let from_account = self.get_account(from_user_id)?;
        let to_account = self.get_account(to_user_id)?;

        // Check if accounts are suspended
        if from_account.is_suspended || to_account.is_suspended {
            return Err(FinanceError::PermissionDenied);
        }

        // Check sufficient funds
        if from_account.balance < amount {
            return Err(FinanceError::InsufficientFunds {
                needed: amount,
                available: from_account.balance,
            });
        }

        // Create transaction
        let transaction = Transaction {
            id: self.generate_transaction_id(),
            from_user_id: Some(from_user_id),
            to_user_id: Some(to_user_id),
            amount,
            transaction_type: TransactionType::Transfer,
            status: TransactionStatus::Pending,
            description,
            reference_id,
            created_at: Utc::now(),
            completed_at: None,
            fee: self.calculate_transfer_fee(amount),
            metadata: HashMap::new(),
        };

        // Execute transfer (in real implementation, this would be atomic)
        self.execute_transaction(&transaction)?;

        Ok(transaction)
    }

    /// Add money to user account (income)
    pub fn add_income(
        &self,
        user_id: u64,
        amount: u64,
        income_type: TransactionType,
        description: String,
        reference_id: Option<String>,
    ) -> Result<Transaction, FinanceError> {
        if amount == 0 {
            return Err(FinanceError::InvalidAmount(0));
        }

        let transaction = Transaction {
            id: self.generate_transaction_id(),
            from_user_id: None,
            to_user_id: Some(user_id),
            amount,
            transaction_type: income_type,
            status: TransactionStatus::Pending,
            description,
            reference_id,
            created_at: Utc::now(),
            completed_at: None,
            fee: 0,
            metadata: HashMap::new(),
        };

        self.execute_transaction(&transaction)?;
        Ok(transaction)
    }

    /// Deduct money from user account (expense)
    pub fn add_expense(
        &self,
        user_id: u64,
        amount: u64,
        expense_type: TransactionType,
        description: String,
        reference_id: Option<String>,
    ) -> Result<Transaction, FinanceError> {
        if amount == 0 {
            return Err(FinanceError::InvalidAmount(0));
        }

        // Check permissions
        if let Some(current_user) = self.current_user_id {
            if current_user != user_id {
                return Err(FinanceError::PermissionDenied);
            }
        } else {
            return Err(FinanceError::PermissionDenied);
        }

        let account = self.get_account(user_id)?;

        // Check sufficient funds
        if account.balance < amount {
            return Err(FinanceError::InsufficientFunds {
                needed: amount,
                available: account.balance,
            });
        }

        let transaction = Transaction {
            id: self.generate_transaction_id(),
            from_user_id: Some(user_id),
            to_user_id: None,
            amount,
            transaction_type: expense_type,
            status: TransactionStatus::Pending,
            description,
            reference_id,
            created_at: Utc::now(),
            completed_at: None,
            fee: 0,
            metadata: HashMap::new(),
        };

        self.execute_transaction(&transaction)?;
        Ok(transaction)
    }

    /// Get transaction by ID
    pub fn get_transaction(&self, transaction_id: u64) -> Result<Transaction, FinanceError> {
        // Simulate database lookup
        if transaction_id == 0 {
            return Err(FinanceError::TransactionNotFound(transaction_id));
        }

        // Mock transaction
        Ok(Transaction {
            id: transaction_id,
            from_user_id: Some(1),
            to_user_id: Some(2),
            amount: 100,
            transaction_type: TransactionType::Transfer,
            status: TransactionStatus::Completed,
            description: "Test transaction".to_string(),
            reference_id: None,
            created_at: Utc::now(),
            completed_at: Some(Utc::now()),
            fee: 1,
            metadata: HashMap::new(),
        })
    }

    /// Get user's transaction history
    pub fn get_transaction_history(
        &self,
        user_id: u64,
        filter: Option<FinanceFilter>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Transaction>, FinanceError> {
        let limit = if limit > 100 { 100 } else { limit };
        
        // Simulate database query with filtering
        let transactions = vec![]; // Would be populated from database

        Ok(transactions)
    }

    /// Get transaction summary for a period
    pub fn get_transaction_summary(
        &self,
        user_id: u64,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<TransactionSummary, FinanceError> {
        let filter = FinanceFilter {
            date_from: Some(period_start),
            date_to: Some(period_end),
            ..Default::default()
        };

        let transactions = self.get_transaction_history(user_id, Some(filter), 1000, 0)?;

        let mut total_income = 0;
        let mut total_expenses = 0;
        let mut largest_transaction = 0;

        for transaction in &transactions {
            match transaction.to_user_id {
                Some(to_id) if to_id == user_id => {
                    total_income += transaction.amount;
                }
                _ => {}
            }

            match transaction.from_user_id {
                Some(from_id) if from_id == user_id => {
                    total_expenses += transaction.amount;
                }
                _ => {}
            }

            if transaction.amount > largest_transaction {
                largest_transaction = transaction.amount;
            }
        }

        let net_amount = total_income as i64 - total_expenses as i64;
        let average_transaction = if !transactions.is_empty() {
            (total_income + total_expenses) / transactions.len() as u64
        } else {
            0
        };

        Ok(TransactionSummary {
            total_transactions: transactions.len() as u64,
            total_income,
            total_expenses,
            net_amount,
            average_transaction,
            largest_transaction,
            period_start,
            period_end,
        })
    }

    /// Freeze/unfreeze funds
    pub fn freeze_funds(&self, user_id: u64, amount: u64) -> Result<(), FinanceError> {
        // Check permissions
        if let Some(current_user) = self.current_user_id {
            if current_user != user_id {
                return Err(FinanceError::PermissionDenied);
            }
        } else {
            return Err(FinanceError::PermissionDenied);
        }

        let account = self.get_account(user_id)?;

        if account.balance < amount {
            return Err(FinanceError::InsufficientFunds {
                needed: amount,
                available: account.balance,
            });
        }

        // Simulate freezing funds
        // UPDATE accounts SET balance = balance - amount, frozen_balance = frozen_balance + amount
        Ok(())
    }

    /// Unfreeze funds
    pub fn unfreeze_funds(&self, user_id: u64, amount: u64) -> Result<(), FinanceError> {
        // Check permissions
        if let Some(current_user) = self.current_user_id {
            if current_user != user_id {
                return Err(FinanceError::PermissionDenied);
            }
        } else {
            return Err(FinanceError::PermissionDenied);
        }

        let account = self.get_account(user_id)?;

        if account.frozen_balance < amount {
            return Err(FinanceError::InsufficientFunds {
                needed: amount,
                available: account.frozen_balance,
            });
        }

        // Simulate unfreezing funds
        // UPDATE accounts SET balance = balance + amount, frozen_balance = frozen_balance - amount
        Ok(())
    }

    /// Calculate transfer fee
    fn calculate_transfer_fee(&self, amount: u64) -> u64 {
        // Basic fee calculation: 1% with minimum 1 and maximum 100
        let fee = (amount as f64 * 0.01) as u64;
        fee.max(1).min(100)
    }

    /// Execute transaction (simulate atomic operation)
    fn execute_transaction(&self, transaction: &Transaction) -> Result<(), FinanceError> {
        // In real implementation, this would:
        // 1. Start database transaction
        // 2. Update account balances
        // 3. Insert transaction record
        // 4. Commit or rollback on error

        // Simulate successful execution
        Ok(())
    }

    /// Generate unique transaction ID
    fn generate_transaction_id(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    /// Get daily spending for user
    pub fn get_daily_spending(&self, user_id: u64) -> Result<u64, FinanceError> {
        let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        let today_end = today_start + chrono::Duration::days(1);

        let summary = self.get_transaction_summary(user_id, today_start, today_end)?;
        Ok(summary.total_expenses)
    }

    /// Get monthly spending for user
    pub fn get_monthly_spending(&self, user_id: u64) -> Result<u64, FinanceError> {
        let now = Utc::now();
        let month_start = now.date_naive().with_day(1).unwrap().and_hms_opt(0, 0, 0).unwrap().and_utc();
        let month_end = if month_start.month() == 12 {
            month_start.with_year(month_start.year() + 1).unwrap().with_month(1).unwrap()
        } else {
            month_start.with_month(month_start.month() + 1).unwrap()
        };

        let summary = self.get_transaction_summary(user_id, month_start, month_end)?;
        Ok(summary.total_expenses)
    }

    /// Check if user can afford amount
    pub fn can_afford(&self, user_id: u64, amount: u64) -> Result<bool, FinanceError> {
        let account = self.get_account(user_id)?;
        Ok(account.balance >= amount && !account.is_suspended)
    }
}

impl Default for FinanceFilter {
    fn default() -> Self {
        Self {
            transaction_type: None,
            status: None,
            from_user_id: None,
            to_user_id: None,
            min_amount: None,
            max_amount: None,
            date_from: None,
            date_to: None,
            reference_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finances_creation() {
        let finances = Finances::new(Some(1));
        assert_eq!(finances.current_user_id, Some(1));
    }

    #[test]
    fn test_get_account() {
        let finances = Finances::new(None);
        let result = finances.get_account(1);
        assert!(result.is_ok());

        let account = result.unwrap();
        assert_eq!(account.user_id, 1);
        assert_eq!(account.balance, 1000);
    }

    #[test]
    fn test_calculate_transfer_fee() {
        let finances = Finances::new(None);
        
        assert_eq!(finances.calculate_transfer_fee(50), 1); // Minimum fee
        assert_eq!(finances.calculate_transfer_fee(500), 5); // 1% fee
        assert_eq!(finances.calculate_transfer_fee(20000), 100); // Maximum fee
    }

    #[test]
    fn test_can_afford() {
        let finances = Finances::new(None);
        let result = finances.can_afford(1, 500);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Mock account has 1000 balance

        let result = finances.can_afford(1, 2000);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Not enough balance
    }

    #[test]
    fn test_transfer_without_permission() {
        let finances = Finances::new(None);
        let result = finances.transfer(1, 2, 100, "Test".to_string(), None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FinanceError::PermissionDenied));
    }

    #[test]
    fn test_invalid_amount() {
        let finances = Finances::new(Some(1));
        let result = finances.transfer(1, 2, 0, "Test".to_string(), None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FinanceError::InvalidAmount(0)));
    }
}