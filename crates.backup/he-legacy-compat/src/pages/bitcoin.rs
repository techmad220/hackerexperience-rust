use axum::{
    extract::Form,
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use he_core::session::PhpSession;
use sqlx::PgPool;

/// Bitcoin API response structure - matches original PHP format
#[derive(Debug, Clone, Serialize)]
pub struct BitcoinResponse {
    pub status: String,
    pub redirect: String,
    pub msg: String,
    #[serde(flatten)]
    pub data: Option<Value>,
}

impl BitcoinResponse {
    pub fn success(msg: &str) -> Self {
        Self {
            status: "OK".to_string(),
            redirect: "".to_string(),
            msg: msg.to_string(),
            data: None,
        }
    }
    
    pub fn success_with_data(msg: &str, data: Value) -> Self {
        Self {
            status: "OK".to_string(),
            redirect: "".to_string(),
            msg: msg.to_string(),
            data: Some(data),
        }
    }
    
    pub fn error(msg: &str) -> Self {
        Self {
            status: "ERROR".to_string(),
            redirect: "".to_string(),
            msg: msg.to_string(),
            data: None,
        }
    }
    
    // Original PHP default error - preserved for authenticity
    pub fn default_error() -> Self {
        Self {
            status: "ERROR".to_string(),
            redirect: "".to_string(),
            msg: "STOP SPYING ON ME!".to_string(),
            data: None,
        }
    }
}

/// Bitcoin request payload structure
#[derive(Debug, Deserialize)]
pub struct BitcoinRequest {
    pub func: String,
    pub amount: Option<String>,
    pub destination: Option<String>,
    pub acc: Option<String>,
    pub addr: Option<String>,
    pub key: Option<String>,
    #[serde(flatten)]
    pub params: HashMap<String, String>,
}

/// Bitcoin handler - 1:1 port of bitcoin.php
/// 
/// Original: AJAX endpoint for all bitcoin-related operations
/// Features:
/// - Bitcoin wallet authentication and session management
/// - Bitcoin transfer operations between wallets
/// - Bitcoin buy/sell operations with bank account integration
/// - Wallet creation and registration
/// - Comprehensive validation and error handling
pub async fn bitcoin_handler(
    Extension(db): Extension<PgPool>,
    Extension(session): Extension<PhpSession>,
    Form(request): Form<BitcoinRequest>,
) -> Result<Json<BitcoinResponse>, StatusCode> {
    
    // Initialize default error response
    let mut result = BitcoinResponse::default_error();
    
    // Check if user is logged in (equivalent to $session->issetLogin())
    if !session.is_logged_in() {
        return Ok(Json(result));
    }
    
    result.status = "OK".to_string();
    
    // Main function dispatcher - matches original PHP switch statement
    let response = match request.func.as_str() {
        
        "btcTransfer" => btc_transfer_handler(&db, &session, &request).await,
        "btcBuy" => btc_buy_handler(&db, &session, &request).await,
        "btcSell" => btc_sell_handler(&db, &session, &request).await,
        "btcLogout" => btc_logout_handler(&session, &request).await,
        "btcLogin" => btc_login_handler(&db, &session, &request).await,
        "btcRegister" => btc_register_handler(&db, &session, &request).await,
        
        _ => {
            tracing::warn!("Unknown bitcoin function: {}", request.func);
            BitcoinResponse::error("Invalid function")
        }
    };
    
    Ok(Json(response))
}

/// Handle bitcoin transfer between wallets
async fn btc_transfer_handler(
    db: &PgPool,
    session: &PhpSession,
    request: &BitcoinRequest,
) -> BitcoinResponse {
    
    // Check if wallet session is active (equivalent to $session->issetWalletSession())
    if !session.has_wallet_session() {
        return BitcoinResponse::error("No wallet session active");
    }
    
    if let Some(amount_str) = &request.amount {
        // Process transfer request
        let amount_to_transfer = match amount_str.parse::<f64>() {
            Ok(amount) => (amount * 10000000.0).round() / 10000000.0, // Round to 7 decimal places
            Err(_) => return BitcoinResponse::error("Invalid amount."),
        };
        
        if amount_to_transfer < 0.0 {
            // Original PHP: exit() on negative amount
            return BitcoinResponse::error("Invalid amount");
        }
        
        let destination = match &request.destination {
            Some(dest) => dest,
            None => return BitcoinResponse::error("Missing destination address."),
        };
        
        if destination.len() != 34 {
            return BitcoinResponse::error("Invalid destination address.");
        }
        
        let wallet_addr = session.get_wallet_address().unwrap_or_default();
        
        if destination == &wallet_addr {
            return BitcoinResponse::error("LOL, you cant transfer to yourself.");
        }
        
        // Implement actual bitcoin transfer logic
        // Check if destination wallet exists
        let destination_exists = sqlx::query!(
            "SELECT COUNT(*) as count FROM bitcoin_wallets WHERE address = $1",
            destination
        )
        .fetch_one(db)
        .await
        .map(|row| row.count.unwrap_or(0) > 0)
        .unwrap_or(false);
        
        if !destination_exists {
            return BitcoinResponse::error("Destination wallet does not exist.");
        }
        
        // Check if user has sufficient balance
        let wallet_balance = sqlx::query!(
            "SELECT balance FROM bitcoin_wallets WHERE address = $1",
            &wallet_addr
        )
        .fetch_one(db)
        .await
        .map(|row| row.balance.unwrap_or(0.0))
        .unwrap_or(0.0);
        
        if wallet_balance < amount_to_transfer {
            return BitcoinResponse::error("Insufficient balance.");
        }
        
        // Perform transfer
        let mut tx = match db.begin().await {
            Ok(tx) => tx,
            Err(_) => return BitcoinResponse::error("Database error."),
        };
        
        // Subtract from source wallet
        if sqlx::query!(
            "UPDATE bitcoin_wallets SET balance = balance - $1 WHERE address = $2",
            amount_to_transfer,
            &wallet_addr
        )
        .execute(&mut *tx)
        .await
        .is_err() {
            let _ = tx.rollback().await;
            return BitcoinResponse::error("Transfer failed.");
        }
        
        // Add to destination wallet
        if sqlx::query!(
            "UPDATE bitcoin_wallets SET balance = balance + $1 WHERE address = $2",
            amount_to_transfer,
            destination
        )
        .execute(&mut *tx)
        .await
        .is_err() {
            let _ = tx.rollback().await;
            return BitcoinResponse::error("Transfer failed.");
        }
        
        // Add transaction log
        if sqlx::query!(
            "INSERT INTO bitcoin_transactions (from_address, to_address, amount, transaction_type, created_at) VALUES ($1, $2, $3, $4, NOW())",
            &wallet_addr,
            destination,
            amount_to_transfer,
            "transfer"
        )
        .execute(&mut *tx)
        .await
        .is_err() {
            let _ = tx.rollback().await;
            return BitcoinResponse::error("Transfer failed.");
        }
        
        if tx.commit().await.is_err() {
            return BitcoinResponse::error("Transfer failed.");
        }
        
        BitcoinResponse::success(&format!(
            "{} BTC transfered from {} to {}.",
            amount_to_transfer, wallet_addr, destination
        ))
        
    } else {
        // Show transfer form
        let wallet_addr = session.get_wallet_address().unwrap_or_default();
        
        // Get actual wallet balance
        let wallet_balance = sqlx::query!(
            "SELECT balance FROM bitcoin_wallets WHERE address = $1",
            &wallet_addr
        )
        .fetch_one(db)
        .await
        .map(|row| row.balance.unwrap_or(0.0))
        .unwrap_or(0.0);
        
        let wallet_amount = format!("{:.7}", wallet_balance);
        let btc_value = get_btc_value().await.unwrap_or("50000".to_string());
        
        let title = "Transfer bitcoins";
        let text = format!(
            r#"<div class="control-group"><div class="controls">From <span class="item">{}</span></div><br/><div class="controls"><input id="btc-amount" class="name" type="text" name="btc-amount" placeholder="BTC Amount" style="width: 80%;" value="{}"/></div></div><div class="control-group"><div class="controls"><input id="btc-to" class="name" type="text" name="btc-to" placeholder="Destination address" style="width: 80%;"/></div></div>"#,
            wallet_addr, wallet_amount
        );
        
        let response_data = json!([{
            "title": title,
            "text": text,
            "value": btc_value
        }]);
        
        BitcoinResponse::success_with_data("", response_data)
    }
}

/// Handle bitcoin purchase with bank account
async fn btc_buy_handler(
    db: &PgPool,
    session: &PhpSession,
    request: &BitcoinRequest,
) -> BitcoinResponse {
    
    if !session.has_wallet_session() {
        return BitcoinResponse::error("No wallet session active");
    }
    
    if let Some(amount_str) = &request.amount {
        // Process buy request
        let amount_to_buy = match amount_str.parse::<f64>() {
            Ok(amount) => (amount * 10000000.0).round() / 10000000.0, // Round to 7 decimal places
            Err(_) => return BitcoinResponse::error("Invalid amount"),
        };
        
        if amount_to_buy < 0.0 {
            return BitcoinResponse::error("Invalid amount");
        }
        
        let acc = match &request.acc {
            Some(acc) => acc,
            None => return BitcoinResponse::error("Missing bank account."),
        };
        
        if acc.is_empty() || !acc.chars().all(|c| c.is_ascii_digit()) {
            return BitcoinResponse::error("Invalid bank account.");
        }
        
        // TODO: Implement actual bitcoin buy logic
        // - Validate bank account exists and belongs to user
        // - Check if user has sufficient funds
        // - Calculate cost based on BTC value
        // - Perform purchase
        // - Add logs
        
        BitcoinResponse::success(&format!(
            "{} BTC bought for ${}.",
            amount_to_buy,
            (amount_to_buy * 50000.0) as u64 // TODO: Use actual BTC value
        ))
        
    } else {
        // Show buy form
        let btc_value = "50000"; // TODO: Get actual BTC value
        let text_acc = r#"<br/><div id="loading" class="pull-left" style="margin-left: 9%;"><img src="images/ajax-money.gif">Loading...</div><input type="hidden" id="accSelect" value=""><span id="desc-money" class="pull-left" style="margin-left: 9%;"></span>"#;
        
        let title = "Buy bitcoins";
        let text = format!(
            r#"<div class="control-group"><div class="controls"><input id="btc-amount" class="name" type="text" name="btc-amount" placeholder="BTC Amount" style="width: 80%;" value="1.0"/></div><div class="controls"><span class="pull-left" style="margin-left: 9%;"><span class="item">Rate: </span>1 BTC = ${}</span></div><br/><div class="controls"><span class="pull-left" style="margin-left: 9%;"><span class="item">Value: </span><span class="green">$<span id="btc-total"></span></span></span></div></div>{}"#,
            btc_value, text_acc
        );
        
        let response_data = json!([{
            "title": title,
            "text": text,
            "value": btc_value
        }]);
        
        BitcoinResponse::success_with_data("", response_data)
    }
}

/// Handle bitcoin sale to bank account
async fn btc_sell_handler(
    db: &PgPool,
    session: &PhpSession,
    request: &BitcoinRequest,
) -> BitcoinResponse {
    
    if !session.has_wallet_session() {
        return BitcoinResponse::error("No wallet session active");
    }
    
    if let Some(amount_str) = &request.amount {
        // Process sell request
        let amount_to_sell = match amount_str.parse::<f64>() {
            Ok(amount) => (amount * 10000000.0).round() / 10000000.0, // Round to 7 decimal places
            Err(_) => return BitcoinResponse::error("Invalid amount."),
        };
        
        if amount_to_sell < 1.0 {
            // Original PHP: exit() on amount < 1
            return BitcoinResponse::error("Minimum 1 BTC required");
        }
        
        let acc = match &request.acc {
            Some(acc) => acc,
            None => return BitcoinResponse::error("Missing bank account."),
        };
        
        if acc.is_empty() || !acc.chars().all(|c| c.is_ascii_digit()) {
            return BitcoinResponse::error("Invalid bank account.");
        }
        
        // TODO: Implement actual bitcoin sell logic
        // - Check if user has sufficient BTC balance
        // - Validate bank account
        // - Calculate sale value based on BTC rate
        // - Perform sale
        // - Add logs
        
        BitcoinResponse::success(&format!(
            "${} transfered to account #{}",
            (amount_to_sell * 50000.0).ceil() as u64, // TODO: Use actual BTC value
            acc
        ))
        
    } else {
        // Show sell form
        let wallet_amount = 0.0; // TODO: Get actual wallet balance
        let btc_value = "50000"; // TODO: Get actual BTC value
        
        let text_sell = if wallet_amount >= 1.0 {
            r#"<br/><div id="loading" class="pull-left" style="margin-left: 9%;"><img src="images/ajax-money.gif">Loading...</div><input type="hidden" id="accSelect" value=""><span id="desc-money" class="pull-left" style="margin-left: 9%;"></span>"#
        } else {
            r#"<br/><span class="pull-left red" style="margin-left: 9%;">You need at least 1 BTC in order to sell.</span>"#
        };
        
        // Format wallet amount to 7 decimal places (from original PHP)
        let formatted_amount = format!("{:.7}", wallet_amount);
        
        let title = "Sell bitcoins";
        let text = format!(
            r#"<div class="control-group"><div class="controls"><input id="btc-amount" class="name" type="text" name="btc-amount" placeholder="BTC Amount" style="width: 80%;" value="{}"/></div><div class="controls"><span class="pull-left" style="margin-left: 9%;"><span class="item">Rate: </span>1 BTC = ${}</span></div><br/><div class="controls"><span class="pull-left" style="margin-left: 9%;"><span class="item">Value: </span><span class="green">$<span id="btc-total"></span></span></span></div></div>{}"#,
            formatted_amount, btc_value, text_sell
        );
        
        let response_data = json!([{
            "title": title,
            "text": text,
            "value": btc_value,
            "amount": wallet_amount
        }]);
        
        BitcoinResponse::success_with_data("", response_data)
    }
}

/// Handle bitcoin wallet logout
async fn btc_logout_handler(
    session: &PhpSession,
    _request: &BitcoinRequest,
) -> BitcoinResponse {
    
    if session.has_wallet_session() {
        let addr = session.get_wallet_address().unwrap_or_default();
        session.delete_wallet_session();
        BitcoinResponse::success(&format!(
            "Logged out from address <strong>{}</strong>.",
            addr
        ))
    } else {
        BitcoinResponse::success("No active wallet session")
    }
}

/// Handle bitcoin wallet login
async fn btc_login_handler(
    db: &PgPool,
    session: &PhpSession,
    request: &BitcoinRequest,
) -> BitcoinResponse {
    
    let addr = match &request.addr {
        Some(addr) => addr,
        None => return BitcoinResponse::error("Missing information."),
    };
    
    let key = match &request.key {
        Some(key) => key,
        None => return BitcoinResponse::error("Missing information."),
    };
    
    if addr.len() != 34 {
        return BitcoinResponse::error("Invalid address.");
    }
    
    if key.len() != 64 {
        return BitcoinResponse::error("Invalid key.");
    }
    
    // TODO: Implement actual bitcoin login logic
    // - Check if wallet exists
    // - Validate wallet key
    // - Create wallet session
    // - Add logs for both player and bitcoin NPC
    
    session.create_wallet_session(addr);
    
    BitcoinResponse::success(&format!(
        "You logged in to the address <strong>{}</strong>.",
        addr
    ))
}

/// Handle bitcoin wallet registration
async fn btc_register_handler(
    db: &PgPool,
    session: &PhpSession,
    _request: &BitcoinRequest,
) -> BitcoinResponse {
    
    let user_id = match session.get_user_id() {
        Some(id) => id,
        None => return BitcoinResponse::error("Not logged in"),
    };
    
    // TODO: Implement actual bitcoin registration logic
    // - Check if user already has a wallet
    // - Create new bitcoin wallet
    // - Generate wallet address and private key
    
    // Check if user already has a wallet
    let existing_wallet = sqlx::query!(
        "SELECT COUNT(*) as count FROM bitcoin_wallets WHERE player_id = $1",
        user_id
    )
    .fetch_one(db)
    .await
    .map(|row| row.count.unwrap_or(0) > 0)
    .unwrap_or(false);
    
    if existing_wallet {
        return BitcoinResponse::error("You already have a bitcoin wallet.");
    }
    
    // Generate wallet address and private key
    let (wallet_address, private_key) = generate_bitcoin_wallet();
    
    // Create new bitcoin wallet
    if sqlx::query!(
        "INSERT INTO bitcoin_wallets (player_id, address, private_key, balance, created_at) VALUES ($1, $2, $3, $4, NOW())",
        user_id,
        &wallet_address,
        &private_key,
        0.0
    )
    .execute(db)
    .await
    .is_err() {
        return BitcoinResponse::error("Failed to create wallet.");
    }
    
    BitcoinResponse::success("Your wallet was created! You can find it's information on the Finances page.")
}

/// Get current BTC value from external API or cache
async fn get_btc_value() -> Option<String> {
    // TODO: Implement actual BTC value fetching
    // This would typically fetch from a cryptocurrency API
    Some("50000".to_string())
}

/// Generate a new bitcoin wallet address and private key
fn generate_bitcoin_wallet() -> (String, String) {
    use rand::{thread_rng, Rng};
    use rand::distributions::Alphanumeric;
    
    // Generate a 34-character bitcoin address (simplified)
    let address: String = "1".to_string() + &thread_rng()
        .sample_iter(&Alphanumeric)
        .take(33)
        .map(char::from)
        .collect::<String>();
    
    // Generate a 64-character private key (simplified)
    let private_key: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();
    
    (address, private_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bitcoin_response_creation() {
        let success = BitcoinResponse::success("Test message");
        assert_eq!(success.status, "OK");
        assert_eq!(success.msg, "Test message");
        
        let error = BitcoinResponse::error("Error message");
        assert_eq!(error.status, "ERROR");
        assert_eq!(error.msg, "Error message");
        
        let default_error = BitcoinResponse::default_error();
        assert_eq!(default_error.msg, "STOP SPYING ON ME!");
    }
}