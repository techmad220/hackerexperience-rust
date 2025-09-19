//! Utility functions for game mechanics

use rand::Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Apply random variation to a value
pub fn apply_random_variation(base_value: i64, variation_percent: f64) -> i64 {
    let mut rng = rand::thread_rng();
    let variation_factor = rng.gen_range((1.0 - variation_percent)..=(1.0 + variation_percent));
    (base_value as f64 * variation_factor) as i64
}

/// Clamp a decimal value between min and max
pub fn clamp_decimal(value: Decimal, min: Decimal, max: Decimal) -> Decimal {
    value.max(min).min(max)
}

/// Calculate weighted average
pub fn weighted_average(values: &[(i32, Decimal)]) -> Decimal {
    let total_weight: Decimal = values.iter().map(|(_, weight)| weight).sum();
    if total_weight == dec!(0.0) {
        return dec!(0.0);
    }
    
    let weighted_sum: Decimal = values.iter()
        .map(|(value, weight)| Decimal::from(*value) * weight)
        .sum();
    
    weighted_sum / total_weight
}