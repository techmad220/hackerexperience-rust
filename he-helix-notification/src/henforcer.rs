//! Notification henforcer for authorization checks

use he_helix_henforcer::{StandardResult, reply_ok, reply_error, HenforcerError};
use uuid::Uuid;

/// Check if user can read notification
pub async fn can_read_notification(
    account_id: Uuid,
    notification_id: Uuid,
) -> StandardResult {
    tracing::debug!("Checking notification read permission: account={}, notification={}", 
        account_id, notification_id);
    
    // Mock implementation - would check database
    reply_ok(std::collections::HashMap::new())
}