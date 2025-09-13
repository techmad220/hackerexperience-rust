//! Public notification API

use uuid::Uuid;
use crate::model::{NotificationQuery, BaseNotification};

/// Get notifications for account
pub async fn get_notifications(
    _query: NotificationQuery,
) -> crate::NotificationResult<Vec<BaseNotification>> {
    // Mock implementation
    Ok(vec![])
}