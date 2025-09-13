//! Bank Actor System
//!
//! This module provides actor implementations for bank and financial operations,
//! including account management, transfers, and transaction processing.

use crate::models::{
    BankAccount, BankTransaction, BankTransfer, TransferStatus, TransactionType,
    CreateBankAccountParams, TransferRequest, DepositRequest, WithdrawalRequest,
    BalanceResponse, TransferResult, BankError
};
use he_helix_core::actors::{Actor, ActorContext, Handler, Message};
use he_helix_core::HelixError;
use he_core::id::AccountId;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::{Utc, Duration};
use std::sync::Arc;
use tracing::{info, error, warn, debug};
use validator::Validate;

/// Messages for Bank Actor
#[derive(Debug)]
pub struct CreateBankAccount {
    pub params: CreateBankAccountParams,
}

impl Message for CreateBankAccount {
    type Result = Result<BankAccount, BankError>;
}

#[derive(Debug)]
pub struct GetBankAccount {
    pub account_id: AccountId,
}

impl Message for GetBankAccount {
    type Result = Result<Option<BankAccount>, BankError>;
}

#[derive(Debug)]
pub struct GetBalance {
    pub account_id: AccountId,
}

impl Message for GetBalance {
    type Result = Result<BalanceResponse, BankError>;
}

#[derive(Debug)]
pub struct InitiateTransfer {
    pub request: TransferRequest,
}

impl Message for InitiateTransfer {
    type Result = Result<TransferResult, BankError>;
}

#[derive(Debug)]
pub struct ProcessTransfers;

impl Message for ProcessTransfers {
    type Result = Result<Vec<TransferResult>, BankError>;
}

#[derive(Debug)]
pub struct Deposit {
    pub request: DepositRequest,
}

impl Message for Deposit {
    type Result = Result<BankTransaction, BankError>;
}

#[derive(Debug)]
pub struct Withdraw {
    pub request: WithdrawalRequest,
}

impl Message for Withdraw {
    type Result = Result<BankTransaction, BankError>;
}

#[derive(Debug)]
pub struct GetTransactionHistory {
    pub account_id: AccountId,
    pub limit: Option<usize>,
}

impl Message for GetTransactionHistory {
    type Result = Result<Vec<BankTransaction>, BankError>;
}

#[derive(Debug)]
pub struct FreezeAccount {
    pub account_id: AccountId,
    pub reason: String,
}

impl Message for FreezeAccount {
    type Result = Result<(), BankError>;
}

#[derive(Debug)]
pub struct UnfreezeAccount {
    pub account_id: AccountId,
}

impl Message for UnfreezeAccount {
    type Result = Result<(), BankError>;
}

#[derive(Debug)]
pub struct GetTransfer {
    pub transfer_id: uuid::Uuid,
}

impl Message for GetTransfer {
    type Result = Result<Option<BankTransfer>, BankError>;
}

#[derive(Debug)]
pub struct CancelTransfer {
    pub transfer_id: uuid::Uuid,
    pub account_id: AccountId, // For authorization
}

impl Message for CancelTransfer {
    type Result = Result<(), BankError>;
}

/// Bank Actor - manages bank accounts and financial operations
#[derive(Debug)]
pub struct BankActor {
    /// Bank accounts storage (account_id -> BankAccount)
    accounts: Arc<RwLock<HashMap<AccountId, BankAccount>>>,
    /// Account number to account ID mapping
    account_number_index: Arc<RwLock<HashMap<String, AccountId>>>,
    /// Active transfers (transfer_id -> BankTransfer)
    active_transfers: Arc<RwLock<HashMap<uuid::Uuid, BankTransfer>>>,
    /// Pending outgoing amounts per account (for available balance calculation)
    pending_outgoing: Arc<RwLock<HashMap<AccountId, i64>>>,
}

impl BankActor {
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
            account_number_index: Arc::new(RwLock::new(HashMap::new())),
            active_transfers: Arc::new(RwLock::new(HashMap::new())),
            pending_outgoing: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Find account ID by account number
    async fn find_account_by_number(&self, account_number: &str) -> Option<AccountId> {
        let index = self.account_number_index.read().await;
        index.get(account_number).copied()
    }

    /// Process a completed transfer
    async fn complete_transfer(&self, transfer_id: uuid::Uuid) -> Result<TransferResult, BankError> {
        let mut transfers = self.active_transfers.write().await;
        let mut accounts = self.accounts.write().await;
        let mut pending = self.pending_outgoing.write().await;

        if let Some(transfer) = transfers.get_mut(&transfer_id) {
            if transfer.status != TransferStatus::Processing {
                return Err(BankError::InvalidTransferState);
            }

            // Find recipient account
            let recipient_id = self.find_account_by_number(&transfer.to_account_number).await
                .ok_or(BankError::InvalidAccountNumber)?;

            // Credit recipient account
            if let Some(recipient_account) = accounts.get_mut(&recipient_id) {
                let transaction = BankTransaction {
                    id: uuid::Uuid::new_v4(),
                    transaction_type: TransactionType::Received,
                    amount: transfer.amount,
                    description: format!("Transfer from account ending in {}", 
                        &transfer.from_account_id.to_string()[..4]),
                    timestamp: Utc::now(),
                    balance_after: recipient_account.balance + transfer.amount,
                    from_account: Some(transfer.from_account_id.to_string()),
                    to_account: Some(transfer.to_account_number.clone()),
                    reference: Some(transfer.id.to_string()),
                };

                recipient_account.add_transaction(transaction);
            }

            // Update transfer status
            transfer.status = TransferStatus::Completed;
            transfer.completed_at = Some(Utc::now());

            // Remove pending amount from sender
            if let Some(pending_amount) = pending.get_mut(&transfer.from_account_id) {
                *pending_amount -= transfer.total_amount();
                if *pending_amount <= 0 {
                    pending.remove(&transfer.from_account_id);
                }
            }

            info!("Transfer completed: {} -> {} for ${:.2}", 
                transfer.from_account_id, transfer.to_account_number, transfer.amount as f64 / 100.0);

            Ok(TransferResult {
                transfer_id,
                status: TransferStatus::Completed,
                message: "Transfer completed successfully".to_string(),
                estimated_completion: None,
            })
        } else {
            Err(BankError::TransferNotFound)
        }
    }

    /// Fail a transfer with reason
    async fn fail_transfer(&self, transfer_id: uuid::Uuid, reason: String) -> Result<TransferResult, BankError> {
        let mut transfers = self.active_transfers.write().await;
        let mut accounts = self.accounts.write().await;
        let mut pending = self.pending_outgoing.write().await;

        if let Some(transfer) = transfers.get_mut(&transfer_id) {
            // Refund sender account
            if let Some(sender_account) = accounts.get_mut(&transfer.from_account_id) {
                let refund_transaction = BankTransaction {
                    id: uuid::Uuid::new_v4(),
                    transaction_type: TransactionType::Received,
                    amount: transfer.total_amount(),
                    description: format!("Transfer refund: {}", reason),
                    timestamp: Utc::now(),
                    balance_after: sender_account.balance + transfer.total_amount(),
                    from_account: None,
                    to_account: Some(sender_account.account_number.clone()),
                    reference: Some(transfer.id.to_string()),
                };

                sender_account.add_transaction(refund_transaction);
            }

            // Update transfer status
            transfer.status = TransferStatus::Failed;
            transfer.completed_at = Some(Utc::now());
            transfer.failure_reason = Some(reason.clone());

            // Remove pending amount
            if let Some(pending_amount) = pending.get_mut(&transfer.from_account_id) {
                *pending_amount -= transfer.total_amount();
                if *pending_amount <= 0 {
                    pending.remove(&transfer.from_account_id);
                }
            }

            warn!("Transfer failed: {} - {}", transfer_id, reason);

            Ok(TransferResult {
                transfer_id,
                status: TransferStatus::Failed,
                message: format!("Transfer failed: {}", reason),
                estimated_completion: None,
            })
        } else {
            Err(BankError::TransferNotFound)
        }
    }

    /// Start background transfer processing
    async fn start_transfer_processing(&self) {
        let transfers = self.active_transfers.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                
                let transfers_to_process: Vec<uuid::Uuid> = {
                    let transfers_guard = transfers.read().await;
                    transfers_guard
                        .values()
                        .filter(|t| t.status == TransferStatus::Processing && t.is_ready_to_process())
                        .map(|t| t.id)
                        .collect()
                };

                for transfer_id in transfers_to_process {
                    // Note: In a real implementation, this would be handled by the actor
                    // For now, this is just a placeholder for the background processing logic
                    debug!("Transfer {} is ready for processing", transfer_id);
                }
            }
        });
    }
}

impl Actor for BankActor {
    fn started(&mut self, ctx: &mut ActorContext) {
        info!("BankActor started with process_id: {}", ctx.process_id);
        
        // Start background transfer processing
        let actor = self.clone();
        tokio::spawn(async move {
            actor.start_transfer_processing().await;
        });
    }

    fn stopping(&mut self, ctx: &mut ActorContext) {
        info!("BankActor stopping with process_id: {}", ctx.process_id);
    }

    fn error(&mut self, err: HelixError, ctx: &mut ActorContext) {
        error!("BankActor error on process_id {}: {}", ctx.process_id, err);
    }
}

// Note: We need to implement Clone for BankActor for the background task
impl Clone for BankActor {
    fn clone(&self) -> Self {
        Self {
            accounts: self.accounts.clone(),
            account_number_index: self.account_number_index.clone(),
            active_transfers: self.active_transfers.clone(),
            pending_outgoing: self.pending_outgoing.clone(),
        }
    }
}

#[async_trait]
impl Handler<CreateBankAccount> for BankActor {
    async fn handle(&mut self, msg: CreateBankAccount, _ctx: &mut ActorContext) -> Result<BankAccount, BankError> {
        info!("Creating bank account for account_id: {}", msg.params.account_id);
        
        msg.params.validate()?;

        let mut accounts = self.accounts.write().await;
        let mut account_index = self.account_number_index.write().await;

        // Check if account already exists
        if accounts.contains_key(&msg.params.account_id) {
            return Err(BankError::AccountAlreadyExists);
        }

        // Create new bank account
        let bank_account = BankAccount::new(
            msg.params.account_id,
            msg.params.initial_balance,
            msg.params.account_type,
        );

        // Store account and update index
        account_index.insert(bank_account.account_number.clone(), msg.params.account_id);
        accounts.insert(msg.params.account_id, bank_account.clone());

        info!("Bank account created: {} ({})", bank_account.account_number, msg.params.account_id);
        Ok(bank_account)
    }
}

#[async_trait]
impl Handler<GetBankAccount> for BankActor {
    async fn handle(&mut self, msg: GetBankAccount, _ctx: &mut ActorContext) -> Result<Option<BankAccount>, BankError> {
        let accounts = self.accounts.read().await;
        Ok(accounts.get(&msg.account_id).cloned())
    }
}

#[async_trait]
impl Handler<GetBalance> for BankActor {
    async fn handle(&mut self, msg: GetBalance, _ctx: &mut ActorContext) -> Result<BalanceResponse, BankError> {
        let accounts = self.accounts.read().await;
        let pending = self.pending_outgoing.read().await;

        let account = accounts.get(&msg.account_id).ok_or(BankError::AccountNotFound)?;
        let pending_amount = pending.get(&msg.account_id).copied().unwrap_or(0);

        Ok(BalanceResponse {
            account_id: msg.account_id,
            balance: account.balance,
            available_balance: account.available_balance(pending_amount),
            last_transaction: account.last_transaction,
        })
    }
}

#[async_trait]
impl Handler<InitiateTransfer> for BankActor {
    async fn handle(&mut self, msg: InitiateTransfer, _ctx: &mut ActorContext) -> Result<TransferResult, BankError> {
        info!("Initiating transfer from {} to {} for ${:.2}", 
            msg.request.from_account_id, msg.request.to_account_number, msg.request.amount as f64 / 100.0);

        msg.request.validate()?;

        let mut accounts = self.accounts.write().await;
        let mut transfers = self.active_transfers.write().await;
        let mut pending = self.pending_outgoing.write().await;

        // Verify sender account
        let sender_account = accounts.get_mut(&msg.request.from_account_id)
            .ok_or(BankError::AccountNotFound)?;

        if sender_account.frozen {
            return Err(BankError::AccountFrozen);
        }

        // Create transfer
        let mut transfer = BankTransfer::new(
            msg.request.from_account_id,
            msg.request.to_account_number.clone(),
            msg.request.amount,
            msg.request.description.unwrap_or_else(|| "Bank transfer".to_string()),
        );

        let total_amount = transfer.total_amount();

        // Check if sender has sufficient funds
        let current_pending = pending.get(&msg.request.from_account_id).copied().unwrap_or(0);
        if !sender_account.can_afford(total_amount, current_pending) {
            return Err(BankError::InsufficientFunds);
        }

        // Check daily transfer limit
        if !sender_account.can_transfer_today(msg.request.amount) {
            return Err(BankError::DailyLimitExceeded);
        }

        // Verify recipient account exists
        if self.find_account_by_number(&msg.request.to_account_number).await.is_none() {
            return Err(BankError::InvalidAccountNumber);
        }

        // Debit sender account immediately
        let debit_transaction = BankTransaction {
            id: uuid::Uuid::new_v4(),
            transaction_type: TransactionType::Transfer,
            amount: -(total_amount),
            description: format!("Transfer to {} (Fee: ${:.2})", 
                transfer.to_account_number, transfer.fee as f64 / 100.0),
            timestamp: Utc::now(),
            balance_after: sender_account.balance - total_amount,
            from_account: Some(sender_account.account_number.clone()),
            to_account: Some(transfer.to_account_number.clone()),
            reference: Some(transfer.id.to_string()),
        };

        sender_account.add_transaction(debit_transaction);

        // Update transfer status and store
        transfer.status = TransferStatus::Processing;
        let transfer_id = transfer.id;
        
        // Track pending amount
        *pending.entry(msg.request.from_account_id).or_insert(0) += total_amount;

        transfers.insert(transfer_id, transfer);

        info!("Transfer initiated: {} for ${:.2}", transfer_id, total_amount as f64 / 100.0);

        Ok(TransferResult {
            transfer_id,
            status: TransferStatus::Processing,
            message: "Transfer initiated successfully".to_string(),
            estimated_completion: Some(Utc::now() + Duration::seconds(msg.request.amount.min(3600) as i64)),
        })
    }
}

#[async_trait]
impl Handler<ProcessTransfers> for BankActor {
    async fn handle(&mut self, _msg: ProcessTransfers, _ctx: &mut ActorContext) -> Result<Vec<TransferResult>, BankError> {
        let mut results = Vec::new();
        
        // Get transfers ready for processing
        let transfers_to_process: Vec<uuid::Uuid> = {
            let transfers = self.active_transfers.read().await;
            transfers
                .values()
                .filter(|t| t.status == TransferStatus::Processing && t.is_ready_to_process())
                .map(|t| t.id)
                .collect()
        };

        // Process each ready transfer
        for transfer_id in transfers_to_process {
            match self.complete_transfer(transfer_id).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    error!("Failed to process transfer {}: {}", transfer_id, e);
                    if let Ok(failed_result) = self.fail_transfer(transfer_id, e.to_string()).await {
                        results.push(failed_result);
                    }
                }
            }
        }

        if !results.is_empty() {
            info!("Processed {} transfers", results.len());
        }

        Ok(results)
    }
}

#[async_trait]
impl Handler<Deposit> for BankActor {
    async fn handle(&mut self, msg: Deposit, _ctx: &mut ActorContext) -> Result<BankTransaction, BankError> {
        info!("Processing deposit for account {} of ${:.2}", 
            msg.request.account_id, msg.request.amount as f64 / 100.0);

        msg.request.validate()?;

        let mut accounts = self.accounts.write().await;
        let account = accounts.get_mut(&msg.request.account_id)
            .ok_or(BankError::AccountNotFound)?;

        let transaction = BankTransaction {
            id: uuid::Uuid::new_v4(),
            transaction_type: TransactionType::Deposit,
            amount: msg.request.amount,
            description: msg.request.description,
            timestamp: Utc::now(),
            balance_after: account.balance + msg.request.amount,
            from_account: None,
            to_account: Some(account.account_number.clone()),
            reference: None,
        };

        account.add_transaction(transaction.clone());

        info!("Deposit completed: ${:.2} to account {}", 
            msg.request.amount as f64 / 100.0, msg.request.account_id);

        Ok(transaction)
    }
}

#[async_trait]
impl Handler<Withdraw> for BankActor {
    async fn handle(&mut self, msg: Withdraw, _ctx: &mut ActorContext) -> Result<BankTransaction, BankError> {
        info!("Processing withdrawal for account {} of ${:.2}", 
            msg.request.account_id, msg.request.amount as f64 / 100.0);

        msg.request.validate()?;

        let mut accounts = self.accounts.write().await;
        let pending = self.pending_outgoing.read().await;

        let account = accounts.get_mut(&msg.request.account_id)
            .ok_or(BankError::AccountNotFound)?;

        if account.frozen {
            return Err(BankError::AccountFrozen);
        }

        let pending_amount = pending.get(&msg.request.account_id).copied().unwrap_or(0);
        if !account.can_afford(msg.request.amount, pending_amount) {
            return Err(BankError::InsufficientFunds);
        }

        let transaction = BankTransaction {
            id: uuid::Uuid::new_v4(),
            transaction_type: TransactionType::Withdrawal,
            amount: -(msg.request.amount),
            description: msg.request.description,
            timestamp: Utc::now(),
            balance_after: account.balance - msg.request.amount,
            from_account: Some(account.account_number.clone()),
            to_account: None,
            reference: None,
        };

        account.add_transaction(transaction.clone());

        info!("Withdrawal completed: ${:.2} from account {}", 
            msg.request.amount as f64 / 100.0, msg.request.account_id);

        Ok(transaction)
    }
}

#[async_trait]
impl Handler<GetTransactionHistory> for BankActor {
    async fn handle(&mut self, msg: GetTransactionHistory, _ctx: &mut ActorContext) -> Result<Vec<BankTransaction>, BankError> {
        let accounts = self.accounts.read().await;
        let account = accounts.get(&msg.account_id).ok_or(BankError::AccountNotFound)?;

        let mut transactions = account.transaction_history.clone();
        transactions.reverse(); // Most recent first

        if let Some(limit) = msg.limit {
            transactions.truncate(limit);
        }

        Ok(transactions)
    }
}

#[async_trait]
impl Handler<FreezeAccount> for BankActor {
    async fn handle(&mut self, msg: FreezeAccount, _ctx: &mut ActorContext) -> Result<(), BankError> {
        let mut accounts = self.accounts.write().await;
        let account = accounts.get_mut(&msg.account_id).ok_or(BankError::AccountNotFound)?;

        account.frozen = true;
        
        let freeze_transaction = BankTransaction {
            id: uuid::Uuid::new_v4(),
            transaction_type: TransactionType::Fee,
            amount: 0,
            description: format!("Account frozen: {}", msg.reason),
            timestamp: Utc::now(),
            balance_after: account.balance,
            from_account: None,
            to_account: Some(account.account_number.clone()),
            reference: None,
        };

        account.add_transaction(freeze_transaction);

        warn!("Account frozen: {} - {}", msg.account_id, msg.reason);
        Ok(())
    }
}

#[async_trait]
impl Handler<UnfreezeAccount> for BankActor {
    async fn handle(&mut self, msg: UnfreezeAccount, _ctx: &mut ActorContext) -> Result<(), BankError> {
        let mut accounts = self.accounts.write().await;
        let account = accounts.get_mut(&msg.account_id).ok_or(BankError::AccountNotFound)?;

        account.frozen = false;

        let unfreeze_transaction = BankTransaction {
            id: uuid::Uuid::new_v4(),
            transaction_type: TransactionType::Fee,
            amount: 0,
            description: "Account unfrozen".to_string(),
            timestamp: Utc::now(),
            balance_after: account.balance,
            from_account: None,
            to_account: Some(account.account_number.clone()),
            reference: None,
        };

        account.add_transaction(unfreeze_transaction);

        info!("Account unfrozen: {}", msg.account_id);
        Ok(())
    }
}

#[async_trait]
impl Handler<GetTransfer> for BankActor {
    async fn handle(&mut self, msg: GetTransfer, _ctx: &mut ActorContext) -> Result<Option<BankTransfer>, BankError> {
        let transfers = self.active_transfers.read().await;
        Ok(transfers.get(&msg.transfer_id).cloned())
    }
}

#[async_trait]
impl Handler<CancelTransfer> for BankActor {
    async fn handle(&mut self, msg: CancelTransfer, _ctx: &mut ActorContext) -> Result<(), BankError> {
        let mut transfers = self.active_transfers.write().await;
        
        if let Some(transfer) = transfers.get_mut(&msg.transfer_id) {
            // Check authorization
            if transfer.from_account_id != msg.account_id {
                return Err(BankError::InvalidAccountNumber);
            }

            // Can only cancel pending or processing transfers
            if !matches!(transfer.status, TransferStatus::Pending | TransferStatus::Processing) {
                return Err(BankError::InvalidTransferState);
            }

            transfer.status = TransferStatus::Cancelled;
            transfer.completed_at = Some(Utc::now());

            // Refund the sender (handled by fail_transfer logic)
            self.fail_transfer(msg.transfer_id, "Cancelled by user".to_string()).await?;

            info!("Transfer cancelled: {}", msg.transfer_id);
            Ok(())
        } else {
            Err(BankError::TransferNotFound)
        }
    }
}

/// Bank Supervisor - manages bank actor and provides supervision
#[derive(Debug)]
pub struct BankSupervisor {
    bank_actor: Option<he_helix_core::actors::ActorAddress>,
}

impl BankSupervisor {
    pub fn new() -> Self {
        Self {
            bank_actor: None,
        }
    }
    
    pub async fn start(&mut self) -> Result<he_helix_core::actors::ActorAddress, HelixError> {
        let mut supervisor = he_helix_core::actors::ActorSupervisor::new();
        let bank_actor = BankActor::new();
        let address = supervisor.spawn(bank_actor);
        
        self.bank_actor = Some(address.clone());
        info!("BankSupervisor started successfully");
        
        Ok(address)
    }
    
    pub fn get_bank_actor(&self) -> Option<&he_helix_core::actors::ActorAddress> {
        self.bank_actor.as_ref()
    }
}

impl Default for BankSupervisor {
    fn default() -> Self {
        Self::new()
    }
}