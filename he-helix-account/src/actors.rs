//! Account Actor System
//!
//! This module provides actor implementations for account management,
//! following the Erlang/OTP actor model patterns.

use crate::models::{Account, CreateAccountParams, UpdateAccountParams, LoginRequest, LoginResponse, PublicAccount, AccountError};
use he_helix_core::actors::{Actor, ActorContext, Handler, Message};
use he_core::id::AccountId;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::Utc;
use std::sync::Arc;
use tracing::{info, error, warn};

/// Messages for Account Actor
#[derive(Debug)]
pub struct CreateAccount {
    pub params: CreateAccountParams,
}

impl Message for CreateAccount {
    type Result = Result<Account, AccountError>;
}

#[derive(Debug)]
pub struct GetAccount {
    pub account_id: AccountId,
}

impl Message for GetAccount {
    type Result = Result<Option<Account>, AccountError>;
}

#[derive(Debug)]
pub struct UpdateAccount {
    pub account_id: AccountId,
    pub params: UpdateAccountParams,
}

impl Message for UpdateAccount {
    type Result = Result<Account, AccountError>;
}

#[derive(Debug)]
pub struct LoginAccount {
    pub request: LoginRequest,
}

impl Message for LoginAccount {
    type Result = Result<LoginResponse, AccountError>;
}

#[derive(Debug)]
pub struct DeleteAccount {
    pub account_id: AccountId,
}

impl Message for DeleteAccount {
    type Result = Result<(), AccountError>;
}

#[derive(Debug)]
pub struct GetAccountByUsername {
    pub username: String,
}

impl Message for GetAccountByUsername {
    type Result = Result<Option<Account>, AccountError>;
}

#[derive(Debug)]
pub struct GetAccountByEmail {
    pub email: String,
}

impl Message for GetAccountByEmail {
    type Result = Result<Option<Account>, AccountError>;
}

/// Account Actor - manages account lifecycle and operations
#[derive(Debug)]
pub struct AccountActor {
    /// In-memory account storage (in production, this would be database-backed)
    accounts: Arc<RwLock<HashMap<AccountId, Account>>>,
    username_index: Arc<RwLock<HashMap<String, AccountId>>>,
    email_index: Arc<RwLock<HashMap<String, AccountId>>>,
}

impl AccountActor {
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
            username_index: Arc::new(RwLock::new(HashMap::new())),
            email_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Hash a password using bcrypt
    fn hash_password(&self, password: &str) -> Result<String, AccountError> {
        bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(AccountError::from)
    }

    /// Generate a new unique account ID
    fn generate_account_id(&self) -> AccountId {
        AccountId::new()
    }

    /// Generate auth token (simplified - in production would use proper JWT)
    fn generate_auth_token(&self, account_id: AccountId) -> String {
        format!("token_{}_{}", account_id, Utc::now().timestamp())
    }
}

impl Actor for AccountActor {
    fn started(&mut self, ctx: &mut ActorContext) {
        info!("AccountActor started with process_id: {}", ctx.process_id);
    }

    fn stopping(&mut self, ctx: &mut ActorContext) {
        info!("AccountActor stopping with process_id: {}", ctx.process_id);
    }

    fn error(&mut self, err: he_helix_core::HelixError, ctx: &mut ActorContext) {
        error!("AccountActor error on process_id {}: {}", ctx.process_id, err);
    }
}

#[async_trait]
impl Handler<CreateAccount> for AccountActor {
    async fn handle(&mut self, msg: CreateAccount, _ctx: &mut ActorContext) -> Result<Account, AccountError> {
        info!("Creating account for username: {}", msg.params.username);
        
        // Validate input
        msg.params.validate()?;
        
        let mut accounts = self.accounts.write().await;
        let mut username_index = self.username_index.write().await;
        let mut email_index = self.email_index.write().await;
        
        // Check if username already exists
        if username_index.contains_key(&msg.params.username) {
            return Err(AccountError::AlreadyExists);
        }
        
        // Check if email already exists
        if email_index.contains_key(&msg.params.email) {
            return Err(AccountError::AlreadyExists);
        }
        
        // Hash password
        let hashed_password = self.hash_password(&msg.params.password)?;
        
        // Create new account
        let account_id = self.generate_account_id();
        let now = Utc::now();
        
        let account = Account {
            account_id,
            email: msg.params.email.clone(),
            username: msg.params.username.clone(),
            display_name: msg.params.username.clone(), // Default to username
            password: hashed_password,
            confirmed: false,
            inserted_at: now,
            updated_at: now,
        };
        
        // Store account and update indices
        accounts.insert(account_id, account.clone());
        username_index.insert(msg.params.username, account_id);
        email_index.insert(msg.params.email, account_id);
        
        info!("Account created successfully: {}", account_id);
        Ok(account)
    }
}

#[async_trait]
impl Handler<GetAccount> for AccountActor {
    async fn handle(&mut self, msg: GetAccount, _ctx: &mut ActorContext) -> Result<Option<Account>, AccountError> {
        let accounts = self.accounts.read().await;
        Ok(accounts.get(&msg.account_id).cloned())
    }
}

#[async_trait]
impl Handler<UpdateAccount> for AccountActor {
    async fn handle(&mut self, msg: UpdateAccount, _ctx: &mut ActorContext) -> Result<Account, AccountError> {
        msg.params.validate()?;
        
        let mut accounts = self.accounts.write().await;
        let mut email_index = self.email_index.write().await;
        
        let account = accounts.get_mut(&msg.account_id)
            .ok_or(AccountError::NotFound)?;
        
        // Update fields if provided
        if let Some(email) = msg.params.email {
            // Remove old email from index
            email_index.remove(&account.email);
            // Update email and add to index
            account.email = email.clone();
            email_index.insert(email, msg.account_id);
        }
        
        if let Some(password) = msg.params.password {
            account.password = self.hash_password(&password)?;
        }
        
        if let Some(confirmed) = msg.params.confirmed {
            account.confirmed = confirmed;
        }
        
        account.updated_at = Utc::now();
        
        info!("Account updated: {}", msg.account_id);
        Ok(account.clone())
    }
}

#[async_trait]
impl Handler<LoginAccount> for AccountActor {
    async fn handle(&mut self, msg: LoginAccount, _ctx: &mut ActorContext) -> Result<LoginResponse, AccountError> {
        let accounts = self.accounts.read().await;
        let username_index = self.username_index.read().await;
        let email_index = self.email_index.read().await;
        
        // Find account by username or email
        let account_id = username_index.get(&msg.request.email_or_username)
            .or_else(|| email_index.get(&msg.request.email_or_username))
            .ok_or(AccountError::InvalidCredentials)?;
        
        let account = accounts.get(account_id)
            .ok_or(AccountError::InvalidCredentials)?;
        
        // Verify password
        if !account.check_password(&msg.request.password) {
            warn!("Failed login attempt for account: {}", account.account_id);
            return Err(AccountError::InvalidCredentials);
        }
        
        // Generate token and response
        let token = self.generate_auth_token(account.account_id);
        let expires_at = Utc::now() + chrono::Duration::hours(24); // 24 hour token
        
        info!("Successful login for account: {}", account.account_id);
        
        Ok(LoginResponse {
            account: PublicAccount::from(account.clone()),
            token,
            expires_at,
        })
    }
}

#[async_trait]
impl Handler<DeleteAccount> for AccountActor {
    async fn handle(&mut self, msg: DeleteAccount, _ctx: &mut ActorContext) -> Result<(), AccountError> {
        let mut accounts = self.accounts.write().await;
        let mut username_index = self.username_index.write().await;
        let mut email_index = self.email_index.write().await;
        
        if let Some(account) = accounts.remove(&msg.account_id) {
            // Remove from indices
            username_index.remove(&account.username);
            email_index.remove(&account.email);
            
            info!("Account deleted: {}", msg.account_id);
            Ok(())
        } else {
            Err(AccountError::NotFound)
        }
    }
}

#[async_trait]
impl Handler<GetAccountByUsername> for AccountActor {
    async fn handle(&mut self, msg: GetAccountByUsername, _ctx: &mut ActorContext) -> Result<Option<Account>, AccountError> {
        let accounts = self.accounts.read().await;
        let username_index = self.username_index.read().await;
        
        if let Some(account_id) = username_index.get(&msg.username) {
            Ok(accounts.get(account_id).cloned())
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl Handler<GetAccountByEmail> for AccountActor {
    async fn handle(&mut self, msg: GetAccountByEmail, _ctx: &mut ActorContext) -> Result<Option<Account>, AccountError> {
        let accounts = self.accounts.read().await;
        let email_index = self.email_index.read().await;
        
        if let Some(account_id) = email_index.get(&msg.email) {
            Ok(accounts.get(account_id).cloned())
        } else {
            Ok(None)
        }
    }
}

/// Account Supervisor - manages multiple account actors and provides supervision
#[derive(Debug)]
pub struct AccountSupervisor {
    account_actor: Option<he_helix_core::actors::ActorAddress>,
}

impl AccountSupervisor {
    pub fn new() -> Self {
        Self {
            account_actor: None,
        }
    }
    
    pub async fn start(&mut self) -> Result<he_helix_core::actors::ActorAddress, he_helix_core::HelixError> {
        let mut supervisor = he_helix_core::actors::ActorSupervisor::new();
        let account_actor = AccountActor::new();
        let address = supervisor.spawn(account_actor);
        
        self.account_actor = Some(address.clone());
        info!("AccountSupervisor started successfully");
        
        Ok(address)
    }
    
    pub fn get_account_actor(&self) -> Option<&he_helix_core::actors::ActorAddress> {
        self.account_actor.as_ref()
    }
}

impl Default for AccountSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

/// Entity Actor - placeholder for entity management
pub struct EntityActor;

/// Component Actor - placeholder for component management
pub struct ComponentActor;