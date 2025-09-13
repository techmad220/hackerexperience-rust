//! Notification actions

use uuid::Uuid;
use crate::model::NotificationClass;

/// Mark notification as read
pub async fn mark_notification_read(
    notification_id: Uuid,
    _account_id: Uuid,
    _class: NotificationClass,
) -> crate::NotificationResult<()> {
    tracing::info!("Marking notification as read: {}", notification_id);
    // Implementation would update database
    Ok(())
}