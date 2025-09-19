use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PremiumError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("User not found: {0}")]
    UserNotFound(u64),
    #[error("Plan not found: {0}")]
    PlanNotFound(String),
    #[error("Payment error: {0}")]
    Payment(String),
    #[error("Subscription not found: {0}")]
    SubscriptionNotFound(u64),
    #[error("Subscription already active")]
    SubscriptionAlreadyActive,
    #[error("Subscription expired")]
    SubscriptionExpired,
    #[error("Invalid plan configuration: {0}")]
    InvalidPlan(String),
    #[error("Permission denied")]
    PermissionDenied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanType {
    Basic,
    Premium,
    VIP,
    Ultimate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanDuration {
    Monthly,
    Quarterly,
    Yearly,
    Lifetime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubscriptionStatus {
    Active,
    Expired,
    Cancelled,
    Suspended,
    PendingPayment,
    PendingActivation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentStatus {
    Pending,
    Completed,
    Failed,
    Refunded,
    Disputed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PremiumPlan {
    pub id: String,
    pub name: String,
    pub description: String,
    pub plan_type: PlanType,
    pub duration: PlanDuration,
    pub price: u64, // Price in cents
    pub currency: String,
    pub features: Vec<String>,
    pub max_characters: Option<u32>,
    pub max_servers: Option<u32>,
    pub priority_support: bool,
    pub ad_free: bool,
    pub special_badges: Vec<String>,
    pub bonus_multiplier: f64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PremiumSubscription {
    pub id: u64,
    pub user_id: u64,
    pub plan_id: String,
    pub status: SubscriptionStatus,
    pub started_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub auto_renew: bool,
    pub payment_method: Option<String>,
    pub last_payment_at: Option<DateTime<Utc>>,
    pub next_payment_at: Option<DateTime<Utc>>,
    pub total_paid: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTransaction {
    pub id: u64,
    pub subscription_id: u64,
    pub user_id: u64,
    pub amount: u64,
    pub currency: String,
    pub status: PaymentStatus,
    pub payment_method: String,
    pub transaction_id: Option<String>,
    pub provider_response: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub failed_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPremiumStatus {
    pub user_id: u64,
    pub is_premium: bool,
    pub current_plan: Option<PremiumPlan>,
    pub subscription: Option<PremiumSubscription>,
    pub days_remaining: Option<i64>,
    pub features: Vec<String>,
    pub bonus_multiplier: f64,
    pub max_characters: Option<u32>,
    pub max_servers: Option<u32>,
    pub has_priority_support: bool,
    pub is_ad_free: bool,
    pub special_badges: Vec<String>,
}

/// Premium subscription management system ported from PHP Premium class
/// Handles premium plans, subscriptions, payments, and user benefits
pub struct Premium {
    current_user_id: Option<u64>,
}

impl Premium {
    /// Create new Premium instance
    pub fn new(current_user_id: Option<u64>) -> Self {
        Self { current_user_id }
    }

    /// Get all available premium plans
    pub fn get_plans(&self) -> Result<Vec<PremiumPlan>, PremiumError> {
        // Simulate database query
        let plans = vec![
            PremiumPlan {
                id: "basic_monthly".to_string(),
                name: "Basic Monthly".to_string(),
                description: "Basic premium features for one month".to_string(),
                plan_type: PlanType::Basic,
                duration: PlanDuration::Monthly,
                price: 999, // $9.99
                currency: "USD".to_string(),
                features: vec![
                    "Ad-free experience".to_string(),
                    "Priority support".to_string(),
                    "Basic badge".to_string(),
                ],
                max_characters: Some(5),
                max_servers: Some(10),
                priority_support: true,
                ad_free: true,
                special_badges: vec!["premium_basic".to_string()],
                bonus_multiplier: 1.5,
                is_active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            PremiumPlan {
                id: "premium_monthly".to_string(),
                name: "Premium Monthly".to_string(),
                description: "Premium features for one month".to_string(),
                plan_type: PlanType::Premium,
                duration: PlanDuration::Monthly,
                price: 1999, // $19.99
                currency: "USD".to_string(),
                features: vec![
                    "All Basic features".to_string(),
                    "Advanced tools".to_string(),
                    "Premium badge".to_string(),
                    "Exclusive content".to_string(),
                ],
                max_characters: Some(10),
                max_servers: Some(25),
                priority_support: true,
                ad_free: true,
                special_badges: vec!["premium_gold".to_string()],
                bonus_multiplier: 2.0,
                is_active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];

        Ok(plans.into_iter().filter(|p| p.is_active).collect())
    }

    /// Get premium plan by ID
    pub fn get_plan(&self, plan_id: &str) -> Result<PremiumPlan, PremiumError> {
        let plans = self.get_plans()?;
        plans
            .into_iter()
            .find(|p| p.id == plan_id)
            .ok_or_else(|| PremiumError::PlanNotFound(plan_id.to_string()))
    }

    /// Get user's premium status
    pub fn get_user_status(&self, user_id: u64) -> Result<UserPremiumStatus, PremiumError> {
        // Get active subscription
        let subscription = self.get_active_subscription(user_id).ok();
        
        let (is_premium, current_plan, days_remaining, features, bonus_multiplier, max_characters, max_servers, priority_support, ad_free, badges) = 
            if let Some(ref sub) = subscription {
                if matches!(sub.status, SubscriptionStatus::Active) && sub.expires_at > Utc::now() {
                    let plan = self.get_plan(&sub.plan_id)?;
                    let days_remaining = (sub.expires_at - Utc::now()).num_days();
                    
                    (
                        true,
                        Some(plan.clone()),
                        Some(days_remaining),
                        plan.features.clone(),
                        plan.bonus_multiplier,
                        plan.max_characters,
                        plan.max_servers,
                        plan.priority_support,
                        plan.ad_free,
                        plan.special_badges.clone(),
                    )
                } else {
                    (false, None, None, vec![], 1.0, None, None, false, false, vec![])
                }
            } else {
                (false, None, None, vec![], 1.0, None, None, false, false, vec![])
            };

        Ok(UserPremiumStatus {
            user_id,
            is_premium,
            current_plan,
            subscription,
            days_remaining,
            features,
            bonus_multiplier,
            max_characters,
            max_servers,
            has_priority_support: priority_support,
            is_ad_free: ad_free,
            special_badges: badges,
        })
    }

    /// Get user's active subscription
    pub fn get_active_subscription(&self, user_id: u64) -> Result<PremiumSubscription, PremiumError> {
        // Simulate database query
        // SELECT * FROM premium_subscriptions WHERE user_id = ? AND status = 'active' AND expires_at > NOW()
        
        // Mock subscription for testing
        Ok(PremiumSubscription {
            id: 1,
            user_id,
            plan_id: "premium_monthly".to_string(),
            status: SubscriptionStatus::Active,
            started_at: Utc::now() - Duration::days(15),
            expires_at: Utc::now() + Duration::days(15),
            cancelled_at: None,
            auto_renew: true,
            payment_method: Some("credit_card".to_string()),
            last_payment_at: Some(Utc::now() - Duration::days(15)),
            next_payment_at: Some(Utc::now() + Duration::days(15)),
            total_paid: 1999,
            created_at: Utc::now() - Duration::days(15),
            updated_at: Utc::now(),
        })
    }

    /// Create new subscription
    pub fn create_subscription(
        &self,
        user_id: u64,
        plan_id: &str,
        payment_method: &str,
    ) -> Result<PremiumSubscription, PremiumError> {
        // Check if user already has active subscription
        if self.get_active_subscription(user_id).is_ok() {
            return Err(PremiumError::SubscriptionAlreadyActive);
        }

        // Get plan details
        let plan = self.get_plan(plan_id)?;

        // Calculate expiration date
        let expires_at = match plan.duration {
            PlanDuration::Monthly => Utc::now() + Duration::days(30),
            PlanDuration::Quarterly => Utc::now() + Duration::days(90),
            PlanDuration::Yearly => Utc::now() + Duration::days(365),
            PlanDuration::Lifetime => Utc::now() + Duration::days(36500), // 100 years
        };

        let subscription = PremiumSubscription {
            id: self.generate_id(),
            user_id,
            plan_id: plan_id.to_string(),
            status: SubscriptionStatus::PendingPayment,
            started_at: Utc::now(),
            expires_at,
            cancelled_at: None,
            auto_renew: !matches!(plan.duration, PlanDuration::Lifetime),
            payment_method: Some(payment_method.to_string()),
            last_payment_at: None,
            next_payment_at: if matches!(plan.duration, PlanDuration::Lifetime) {
                None
            } else {
                Some(expires_at)
            },
            total_paid: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Create initial payment transaction
        let _payment = self.create_payment_transaction(&subscription, &plan)?;

        Ok(subscription)
    }

    /// Create payment transaction
    fn create_payment_transaction(
        &self,
        subscription: &PremiumSubscription,
        plan: &PremiumPlan,
    ) -> Result<PaymentTransaction, PremiumError> {
        let transaction = PaymentTransaction {
            id: self.generate_id(),
            subscription_id: subscription.id,
            user_id: subscription.user_id,
            amount: plan.price,
            currency: plan.currency.clone(),
            status: PaymentStatus::Pending,
            payment_method: subscription.payment_method.clone().unwrap_or_default(),
            transaction_id: None,
            provider_response: None,
            created_at: Utc::now(),
            completed_at: None,
            failed_reason: None,
        };

        // In real implementation, integrate with payment provider here
        // For now, simulate immediate success
        Ok(transaction)
    }

    /// Process payment (simulate payment gateway integration)
    pub fn process_payment(
        &self,
        transaction_id: u64,
        provider_transaction_id: &str,
    ) -> Result<PaymentTransaction, PremiumError> {
        // In real implementation, this would:
        // 1. Verify payment with provider
        // 2. Update transaction status
        // 3. Activate subscription if payment successful
        
        let mut transaction = PaymentTransaction {
            id: transaction_id,
            subscription_id: 1,
            user_id: 1,
            amount: 1999,
            currency: "USD".to_string(),
            status: PaymentStatus::Completed,
            payment_method: "credit_card".to_string(),
            transaction_id: Some(provider_transaction_id.to_string()),
            provider_response: Some("SUCCESS".to_string()),
            created_at: Utc::now(),
            completed_at: Some(Utc::now()),
            failed_reason: None,
        };

        // If payment successful, activate subscription
        if matches!(transaction.status, PaymentStatus::Completed) {
            self.activate_subscription(transaction.subscription_id)?;
        }

        Ok(transaction)
    }

    /// Activate subscription after successful payment
    fn activate_subscription(&self, subscription_id: u64) -> Result<(), PremiumError> {
        // Update subscription status to active
        // UPDATE premium_subscriptions SET status = 'active', last_payment_at = NOW() WHERE id = ?
        Ok(())
    }

    /// Cancel subscription
    pub fn cancel_subscription(&self, subscription_id: u64) -> Result<(), PremiumError> {
        let subscription = self.get_subscription(subscription_id)?;

        // Check permissions
        if let Some(user_id) = self.current_user_id {
            if subscription.user_id != user_id {
                return Err(PremiumError::PermissionDenied);
            }
        } else {
            return Err(PremiumError::PermissionDenied);
        }

        // Cancel subscription (but don't end immediately, let it expire)
        // UPDATE premium_subscriptions SET auto_renew = false, cancelled_at = NOW() WHERE id = ?
        Ok(())
    }

    /// Renew subscription
    pub fn renew_subscription(&self, subscription_id: u64) -> Result<PaymentTransaction, PremiumError> {
        let subscription = self.get_subscription(subscription_id)?;
        let plan = self.get_plan(&subscription.plan_id)?;

        // Check permissions
        if let Some(user_id) = self.current_user_id {
            if subscription.user_id != user_id {
                return Err(PremiumError::PermissionDenied);
            }
        } else {
            return Err(PremiumError::PermissionDenied);
        }

        // Create renewal payment transaction
        let transaction = PaymentTransaction {
            id: self.generate_id(),
            subscription_id: subscription.id,
            user_id: subscription.user_id,
            amount: plan.price,
            currency: plan.currency.clone(),
            status: PaymentStatus::Pending,
            payment_method: subscription.payment_method.clone().unwrap_or_default(),
            transaction_id: None,
            provider_response: None,
            created_at: Utc::now(),
            completed_at: None,
            failed_reason: None,
        };

        Ok(transaction)
    }

    /// Get subscription by ID
    pub fn get_subscription(&self, subscription_id: u64) -> Result<PremiumSubscription, PremiumError> {
        // Simulate database lookup
        if subscription_id == 0 {
            return Err(PremiumError::SubscriptionNotFound(subscription_id));
        }

        // Mock subscription
        Ok(PremiumSubscription {
            id: subscription_id,
            user_id: 1,
            plan_id: "premium_monthly".to_string(),
            status: SubscriptionStatus::Active,
            started_at: Utc::now() - Duration::days(15),
            expires_at: Utc::now() + Duration::days(15),
            cancelled_at: None,
            auto_renew: true,
            payment_method: Some("credit_card".to_string()),
            last_payment_at: Some(Utc::now() - Duration::days(15)),
            next_payment_at: Some(Utc::now() + Duration::days(15)),
            total_paid: 1999,
            created_at: Utc::now() - Duration::days(15),
            updated_at: Utc::now(),
        })
    }

    /// Get user's payment history
    pub fn get_payment_history(&self, user_id: u64) -> Result<Vec<PaymentTransaction>, PremiumError> {
        // Simulate database query
        let transactions = vec![]; // Would be populated from database
        Ok(transactions)
    }

    /// Check if user has specific premium feature
    pub fn has_feature(&self, user_id: u64, feature: &str) -> Result<bool, PremiumError> {
        let status = self.get_user_status(user_id)?;
        Ok(status.features.contains(&feature.to_string()))
    }

    /// Get user's bonus multiplier
    pub fn get_bonus_multiplier(&self, user_id: u64) -> Result<f64, PremiumError> {
        let status = self.get_user_status(user_id)?;
        Ok(status.bonus_multiplier)
    }

    /// Check if user has priority support
    pub fn has_priority_support(&self, user_id: u64) -> Result<bool, PremiumError> {
        let status = self.get_user_status(user_id)?;
        Ok(status.has_priority_support)
    }

    /// Get user's special badges
    pub fn get_special_badges(&self, user_id: u64) -> Result<Vec<String>, PremiumError> {
        let status = self.get_user_status(user_id)?;
        Ok(status.special_badges)
    }

    /// Process expired subscriptions (cleanup task)
    pub fn process_expired_subscriptions(&self) -> Result<u32, PremiumError> {
        // Find and update expired subscriptions
        // UPDATE premium_subscriptions SET status = 'expired' WHERE expires_at < NOW() AND status = 'active'
        
        // Return count of processed subscriptions
        Ok(0)
    }

    /// Process auto-renewals
    pub fn process_auto_renewals(&self) -> Result<u32, PremiumError> {
        // Find subscriptions due for renewal
        // SELECT * FROM premium_subscriptions WHERE auto_renew = true AND expires_at <= NOW() + INTERVAL 1 DAY
        
        // Process renewals
        let processed = 0;
        Ok(processed)
    }

    /// Calculate plan discount for longer durations
    pub fn calculate_discount(&self, plan: &PremiumPlan) -> f64 {
        match plan.duration {
            PlanDuration::Monthly => 0.0,
            PlanDuration::Quarterly => 0.1, // 10% discount
            PlanDuration::Yearly => 0.2,    // 20% discount
            PlanDuration::Lifetime => 0.5,  // 50% discount
        }
    }

    /// Generate unique ID
    fn generate_id(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow::anyhow!("Error: {}", e))?
            .as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_premium_creation() {
        let premium = Premium::new(Some(1));
        assert_eq!(premium.current_user_id, Some(1));
    }

    #[test]
    fn test_get_plans() {
        let premium = Premium::new(None);
        let result = premium.get_plans();
        assert!(result.is_ok());
        
        let plans = result.map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert!(!plans.is_empty());
        assert!(plans.iter().all(|p| p.is_active));
    }

    #[test]
    fn test_get_plan() {
        let premium = Premium::new(None);
        let result = premium.get_plan("basic_monthly");
        assert!(result.is_ok());

        let plan = result.map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert_eq!(plan.id, "basic_monthly");
        assert!(matches!(plan.plan_type, PlanType::Basic));
    }

    #[test]
    fn test_invalid_plan() {
        let premium = Premium::new(None);
        let result = premium.get_plan("invalid_plan");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PremiumError::PlanNotFound(_)));
    }

    #[test]
    fn test_user_status() {
        let premium = Premium::new(None);
        let result = premium.get_user_status(1);
        assert!(result.is_ok());

        let status = result.map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert_eq!(status.user_id, 1);
    }

    #[test]
    fn test_calculate_discount() {
        let premium = Premium::new(None);
        
        let monthly_plan = PremiumPlan {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            plan_type: PlanType::Basic,
            duration: PlanDuration::Monthly,
            price: 1000,
            currency: "USD".to_string(),
            features: vec![],
            max_characters: None,
            max_servers: None,
            priority_support: false,
            ad_free: false,
            special_badges: vec![],
            bonus_multiplier: 1.0,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(premium.calculate_discount(&monthly_plan), 0.0);
        
        let yearly_plan = PremiumPlan {
            duration: PlanDuration::Yearly,
            ..monthly_plan
        };
        
        assert_eq!(premium.calculate_discount(&yearly_plan), 0.2);
    }

    #[test]
    fn test_subscription_creation() {
        let premium = Premium::new(Some(1));
        let result = premium.create_subscription(1, "basic_monthly", "credit_card");
        assert!(result.is_ok());

        let subscription = result.map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert_eq!(subscription.user_id, 1);
        assert_eq!(subscription.plan_id, "basic_monthly");
        assert!(matches!(subscription.status, SubscriptionStatus::PendingPayment));
    }
}