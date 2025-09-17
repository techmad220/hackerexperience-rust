//! Idempotent process cancellation with database row locking

use crate::process::ProcessState;
use sqlx::PgPool;

/// Cancel a process idempotently with proper state transitions
///
/// This function is safe to call multiple times and will:
/// - Lock the target row to serialize transitions
/// - Only transition from valid states (QUEUED or RUNNING)
/// - Silently succeed if already in terminal state (idempotent)
pub async fn cancel_process(pool: &PgPool, pid: i64, user_id: i64) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;

    // Lock the target row to serialize transitions
    // SKIP LOCKED ensures we don't wait on already-locked rows
    let row = sqlx::query!(
        r#"SELECT id, state as "state: ProcessState"
           FROM processes
           WHERE id = $1 AND user_id = $2
           FOR UPDATE SKIP LOCKED"#,
        pid, user_id
    )
    .fetch_optional(&mut *tx).await?;

    if let Some(r) = row {
        match r.state {
            ProcessState::QUEUED | ProcessState::RUNNING => {
                // Valid states for cancellation - transition to CANCELLING
                sqlx::query!(
                    r#"UPDATE processes
                       SET state = 'CANCELLING', updated_at = NOW()
                       WHERE id = $1"#,
                    pid
                )
                .execute(&mut *tx).await?;

                tracing::info!("Process {} transitioned to CANCELLING", pid);
            }
            // Already terminal or mid-cancel = idempotent success
            _ => {
                tracing::debug!("Process {} already in state {:?}, no-op", pid, r.state);
            }
        }
    } else {
        // Process doesn't exist or is locked by another transaction
        // This is fine for idempotent semantics
        tracing::debug!("Process {} not found or locked, treating as success", pid);
    }

    tx.commit().await?;
    Ok(())
}

/// Worker-side handler to complete cancellation
///
/// Should be called by the worker tick loop when it sees a CANCELLING process
pub async fn complete_cancellation(pool: &PgPool, pid: i64) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;

    // Re-check state under lock to ensure only one worker completes it
    let row = sqlx::query!(
        r#"SELECT id, state as "state: ProcessState", cpu_used, ram_used
           FROM processes
           WHERE id = $1 AND state = 'CANCELLING'
           FOR UPDATE"#,
        pid
    )
    .fetch_optional(&mut *tx).await?;

    if let Some(r) = row {
        // Free resources in the same transaction
        sqlx::query!(
            r#"UPDATE servers
               SET cpu_available = cpu_available + $1,
                   ram_available = ram_available + $2
               WHERE id = (SELECT server_id FROM processes WHERE id = $3)"#,
            r.cpu_used,
            r.ram_used,
            pid
        )
        .execute(&mut *tx).await?;

        // Mark as cancelled
        sqlx::query!(
            r#"UPDATE processes
               SET state = 'CANCELLED', updated_at = NOW()
               WHERE id = $1"#,
            pid
        )
        .execute(&mut *tx).await?;

        tracing::info!("Process {} cancelled, freed cpu={} ram={}", pid, r.cpu_used, r.ram_used);
    }

    tx.commit().await?;
    Ok(())
}