//! Financial system mechanics
//! 
//! Implements all financial calculations, economy balance, market dynamics,
//! and transaction processing from the original HackerExperience game.

use crate::{PlayerState, TargetInfo, Result, GameMechanicsError};
use crate::config::FinancialConfig;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use chrono::{DateTime, Utc, Duration};
use rand::Rng;

/// Calculate financial rewards for successful operations
/// 
/// Original HE formula:
/// - Base reward from target value
/// - Skill multiplier: +5% per player level  
/// - Difficulty bonus: +20% per difficulty point
/// - Reputation bonus: +10% per 1000 reputation points
/// - Random variation: ±15%
pub fn calculate_rewards(
    process_type: &str,
    success: bool,
    target: &TargetInfo,
    player: &PlayerState,
    config: &FinancialConfig,
) -> (i64, i64) {
    if !success {
        // Failed operations give minimal experience, no money
        let failed_exp = match process_type {
            "hack" | "password_crack" => target.difficulty_level as i64 * 25,
            "network_scan" | "port_scan" => target.difficulty_level as i64 * 10,
            _ => 50,
        };
        return (0, failed_exp);
    }
    
    let mut money_reward = target.reward_money;
    let mut experience_reward = target.difficulty_level as i64 * 100;
    
    // Skill multiplier based on player level
    let skill_multiplier = dec!(1.0) + (Decimal::from(player.level) * dec!(0.05));
    money_reward = (Decimal::from(money_reward) * skill_multiplier).to_i64().unwrap_or(money_reward);
    experience_reward = (Decimal::from(experience_reward) * skill_multiplier).to_i64().unwrap_or(experience_reward);
    
    // Difficulty bonus
    let difficulty_bonus = dec!(1.0) + (Decimal::from(target.difficulty_level) * dec!(0.20));
    money_reward = (Decimal::from(money_reward) * difficulty_bonus).to_i64().unwrap_or(money_reward);
    experience_reward = (Decimal::from(experience_reward) * difficulty_bonus).to_i64().unwrap_or(experience_reward);
    
    // Reputation bonus
    if let Some(hack_rep) = player.reputation.get("hacking") {
        let rep_bonus = dec!(1.0) + (Decimal::from(*hack_rep) * dec!(0.0001)); // +0.01% per rep point
        money_reward = (Decimal::from(money_reward) * rep_bonus).to_i64().unwrap_or(money_reward);
        experience_reward = (Decimal::from(experience_reward) * rep_bonus).to_i64().unwrap_or(experience_reward);
    }
    
    // Process-specific multipliers
    let process_multiplier = match process_type {
        "bank_hack" => dec!(2.0),        // Bank hacks give 2x money
        "government_hack" => dec!(1.5),  // Government hacks give 1.5x money, 2x exp
        "corporate_hack" => dec!(1.2),   // Corporate hacks give moderate bonus
        "password_crack" => dec!(0.8),   // Password cracking gives less money
        "vulnerability_scan" => dec!(0.3), // Scanning gives minimal money
        _ => dec!(1.0),
    };
    
    money_reward = (Decimal::from(money_reward) * process_multiplier).to_i64().unwrap_or(money_reward);
    
    if process_type == "government_hack" {
        experience_reward = (Decimal::from(experience_reward) * dec!(2.0)).to_i64().unwrap_or(experience_reward);
    }
    
    // Random variation (±15%)
    let mut rng = rand::thread_rng();
    let money_variation: f64 = rng.gen_range(0.85..=1.15);
    let exp_variation: f64 = rng.gen_range(0.85..=1.15);
    
    money_reward = (money_reward as f64 * money_variation) as i64;
    experience_reward = (experience_reward as f64 * exp_variation) as i64;
    
    // Apply reward multiplier cap
    let cap = config.reward_multiplier_cap.to_i64().unwrap_or(5);
    let base_money = target.reward_money;
    let base_exp = target.difficulty_level as i64 * 100;
    
    money_reward = money_reward.min(base_money * cap);
    experience_reward = experience_reward.min(base_exp * cap);
    
    (money_reward.max(0), experience_reward.max(1))
}

/// Calculate bank interest for player accounts
pub fn calculate_bank_interest(
    principal: i64,
    interest_rate: Decimal,
    days_elapsed: i32,
    account_type: &str,
) -> i64 {
    let rate_multiplier = match account_type {
        "premium" => dec!(2.0),    // Premium accounts get 2x interest
        "business" => dec!(1.5),   // Business accounts get 1.5x interest
        "standard" => dec!(1.0),   // Standard accounts get base rate
        "basic" => dec!(0.5),      // Basic accounts get half rate
        _ => dec!(1.0),
    };
    
    let daily_rate = interest_rate * rate_multiplier;
    let compound_factor = (dec!(1.0) + daily_rate).powu(days_elapsed as u64);
    let new_balance = Decimal::from(principal) * compound_factor;
    
    (new_balance.to_i64().unwrap_or(principal) - principal).max(0)
}

/// Calculate transaction fees for money transfers
pub fn calculate_transaction_fee(
    amount: i64,
    transfer_type: &str,
    sender_account_type: &str,
    config: &FinancialConfig,
) -> i64 {
    let base_fee_rate = config.transaction_fee_rate;
    
    // Transfer type modifiers
    let type_multiplier = match transfer_type {
        "instant" => dec!(3.0),      // Instant transfers cost 3x more
        "priority" => dec!(2.0),     // Priority transfers cost 2x more
        "standard" => dec!(1.0),     // Standard transfers use base rate
        "economy" => dec!(0.5),      // Economy transfers cost half
        _ => dec!(1.0),
    };
    
    // Account type modifiers
    let account_multiplier = match sender_account_type {
        "premium" => dec!(0.5),      // Premium accounts get 50% discount
        "business" => dec!(0.7),     // Business accounts get 30% discount
        "standard" => dec!(1.0),     // Standard accounts pay full fee
        "basic" => dec!(1.5),        // Basic accounts pay 50% more
        _ => dec!(1.0),
    };
    
    let fee_rate = base_fee_rate * type_multiplier * account_multiplier;
    let fee = Decimal::from(amount) * fee_rate;
    
    // Minimum fee of $10, maximum fee of $10,000
    fee.to_i64().unwrap_or(0).max(10).min(10000)
}

/// Calculate cryptocurrency exchange rates with volatility
pub fn calculate_crypto_price(
    base_price: Decimal,
    time_elapsed: Duration,
    volatility: Decimal,
) -> Decimal {
    let mut rng = rand::thread_rng();
    
    // Calculate number of volatility periods (each hour)
    let hours_elapsed = time_elapsed.num_hours() as f64;
    let volatility_periods = (hours_elapsed / 1.0).max(1.0);
    
    // Apply compound volatility
    let mut current_price = base_price;
    
    for _ in 0..(volatility_periods as i32) {
        // Random price change: ±volatility%
        let change_factor: f64 = rng.gen_range(-1.0..=1.0);
        let price_change = volatility * Decimal::from_f64_retain(change_factor).unwrap_or(dec!(0.0));
        current_price *= (dec!(1.0) + price_change);
        
        // Prevent price from going negative or too extreme
        current_price = current_price.max(base_price * dec!(0.1)).min(base_price * dec!(10.0));
    }
    
    current_price
}

/// Calculate market maker fees for cryptocurrency trading
pub fn calculate_crypto_trading_fee(
    trade_amount: i64,
    trade_type: &str,
    user_trading_volume: i64,
    config: &FinancialConfig,
) -> i64 {
    let base_fee = config.market_maker_fee;\n    
    // Volume-based fee reduction
    let volume_tier_discount = if user_trading_volume > 1_000_000 {\n        dec!(0.3)  // 30% discount for high-volume traders\n    } else if user_trading_volume > 100_000 {\n        dec!(0.2)  // 20% discount for medium-volume traders\n    } else if user_trading_volume > 10_000 {\n        dec!(0.1)  // 10% discount for regular traders\n    } else {\n        dec!(0.0)  // No discount for low-volume traders\n    };
    
    // Trade type modifiers
    let type_multiplier = match trade_type {
        "market_buy" | "market_sell" => dec!(1.0),     // Market orders use base fee
        "limit_buy" | "limit_sell" => dec!(0.8),       // Limit orders get 20% discount
        "stop_loss" => dec!(1.2),                      // Stop losses cost 20% more
        _ => dec!(1.0),
    };
    
    let effective_fee_rate = base_fee * (dec!(1.0) - volume_tier_discount) * type_multiplier;
    let fee = Decimal::from(trade_amount) * effective_fee_rate;
    
    // Minimum fee of $1, maximum of 1% of trade amount
    fee.to_i64().unwrap_or(0).max(1).min(trade_amount / 100)
}

/// Calculate loan interest and payment schedules
pub fn calculate_loan_payment(
    principal: i64,
    annual_interest_rate: Decimal,
    term_months: i32,
) -> (i64, i64) { // (monthly_payment, total_interest)
    if term_months <= 0 {
        return (principal, 0);
    }
    
    let monthly_rate = annual_interest_rate / dec!(12.0);
    let num_payments = Decimal::from(term_months);
    
    if monthly_rate == dec!(0.0) {
        // Interest-free loan
        let monthly_payment = principal / term_months as i64;
        return (monthly_payment, 0);
    }
    
    // Calculate monthly payment using loan payment formula
    let rate_plus_one = dec!(1.0) + monthly_rate;
    let rate_power = rate_plus_one.powu(term_months as u64);
    
    let monthly_payment = Decimal::from(principal) * monthly_rate * rate_power / (rate_power - dec!(1.0));
    let total_payments = monthly_payment * num_payments;
    let total_interest = total_payments - Decimal::from(principal);
    
    (
        monthly_payment.to_i64().unwrap_or(0),
        total_interest.to_i64().unwrap_or(0),
    )
}

/// Calculate inflation effects on game economy
pub fn apply_inflation(
    base_amount: i64,
    inflation_rate: Decimal,
    time_period_days: i32,
) -> i64 {
    let daily_inflation = inflation_rate;
    let inflation_factor = (dec!(1.0) + daily_inflation).powu(time_period_days as u64);
    
    (Decimal::from(base_amount) * inflation_factor).to_i64().unwrap_or(base_amount)
}

/// Calculate bankruptcy penalties and asset liquidation
pub fn calculate_bankruptcy_recovery(
    debt_amount: i64,
    asset_value: i64,
    liquidation_penalty: Decimal,
) -> (i64, i64) { // (recovered_amount, remaining_debt)
    let liquidation_value = Decimal::from(asset_value) * (dec!(1.0) - liquidation_penalty);
    let recovered = liquidation_value.to_i64().unwrap_or(0);
    let remaining_debt = (debt_amount - recovered).max(0);
    
    (recovered, remaining_debt)
}

/// Validate financial transaction
pub fn validate_transaction(
    sender_balance: i64,
    amount: i64,
    fee: i64,
    sender_account_status: &str,
    receiver_account_status: &str,
) -> Result<()> {
    // Check sender has sufficient balance
    let total_cost = amount + fee;
    if sender_balance < total_cost {
        return Err(GameMechanicsError::InsufficientResources(
            format!("Insufficient balance: need ${}, have ${}", total_cost, sender_balance)
        ));
    }
    
    // Check account statuses
    if sender_account_status == "frozen" {
        return Err(GameMechanicsError::PreconditionFailed("Sender account is frozen".to_string()));
    }
    
    if receiver_account_status == "closed" {
        return Err(GameMechanicsError::PreconditionFailed("Receiver account is closed".to_string()));
    }
    
    // Check transaction amount limits
    if amount <= 0 {
        return Err(GameMechanicsError::InvalidParameter("Transaction amount must be positive".to_string()));
    }
    
    if amount > 10_000_000 {
        return Err(GameMechanicsError::InvalidParameter("Transaction amount exceeds limit ($10,000,000)".to_string()));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PlayerState, TargetInfo, HardwareSpecs};
    use std::collections::HashMap;
    
    fn create_test_player() -> PlayerState {
        PlayerState {
            user_id: 1,
            level: 15,
            experience: 25000,
            money: 100000,
            reputation: {
                let mut rep = HashMap::new();
                rep.insert("hacking".to_string(), 750);
                rep
            },
            hardware_specs: HardwareSpecs {
                cpu: 2800,
                ram: 16384,
                hdd: 1024000,
                net: 200,
                security_level: 6,
                performance_rating: 85,
            },
            software_installed: vec![],
            active_processes: vec![],
            clan_membership: None,
            last_updated: Utc::now(),
        }
    }
    
    fn create_test_target() -> TargetInfo {
        TargetInfo {
            ip_address: "203.0.113.25".to_string(),
            target_type: "bank".to_string(),
            difficulty_level: 7,
            security_rating: 75,
            reward_money: 50000,
            defense_systems: vec![],
        }
    }
    
    #[test]
    fn test_reward_calculation() {
        let player = create_test_player();
        let target = create_test_target();
        let config = FinancialConfig::default();
        
        let (money, exp) = calculate_rewards("bank_hack", true, &target, &player, &config);
        assert!(money > target.reward_money); // Should be higher due to bonuses
        assert!(exp > 0);
        
        let (failed_money, failed_exp) = calculate_rewards("bank_hack", false, &target, &player, &config);
        assert_eq!(failed_money, 0);
        assert!(failed_exp > 0);
    }
    
    #[test]
    fn test_bank_interest() {
        let interest = calculate_bank_interest(100000, dec!(0.001), 30, "premium");
        assert!(interest > 0);
    }
    
    #[test]
    fn test_transaction_fee() {
        let config = FinancialConfig::default();
        let fee = calculate_transaction_fee(10000, "instant", "premium", &config);
        assert!(fee > 0);
        assert!(fee >= 10); // Minimum fee
    }
    
    #[test]
    fn test_crypto_price_volatility() {
        let base_price = dec!(50000.0);
        let time_elapsed = Duration::hours(24);
        let volatility = dec!(0.05);
        
        let new_price = calculate_crypto_price(base_price, time_elapsed, volatility);
        assert!(new_price > dec!(0.0));
        assert_ne!(new_price, base_price); // Price should have changed
    }
    
    #[test]
    fn test_loan_calculation() {
        let (monthly_payment, total_interest) = calculate_loan_payment(100000, dec!(0.05), 12);
        assert!(monthly_payment > 0);
        assert!(total_interest > 0);
        assert!(monthly_payment * 12 == 100000 + total_interest);
    }
    
    #[test]
    fn test_transaction_validation() {
        // Valid transaction
        assert!(validate_transaction(10000, 5000, 100, "active", "active").is_ok());
        
        // Insufficient balance
        assert!(validate_transaction(1000, 5000, 100, "active", "active").is_err());
        
        // Frozen account
        assert!(validate_transaction(10000, 5000, 100, "frozen", "active").is_err());
    }
}