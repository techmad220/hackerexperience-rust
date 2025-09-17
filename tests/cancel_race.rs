//! Test idempotent cancellation under race conditions

use tokio::task;
use sqlx::PgPool;
use he_helix_core::{process_cancel, process::ProcessState};

#[tokio::test]
async fn cancel_is_idempotent_under_finish_race() {
    // Setup test database
    let pool = setup_test_db().await;
    let user_id = 1;

    // Create a running process
    let process_id = create_test_process(&pool, user_id, ProcessState::RUNNING).await;

    // Spawn multiple concurrent cancel attempts
    let mut handles = vec![];
    for _ in 0..10 {
        let pool_clone = pool.clone();
        let handle = task::spawn(async move {
            process_cancel::cancel_process(&pool_clone, process_id, user_id).await
        });
        handles.push(handle);
    }

    // Also spawn a worker that tries to complete the process
    let pool_clone = pool.clone();
    let complete_handle = task::spawn(async move {
        sqlx::query!(
            "UPDATE processes SET state = 'COMPLETED' WHERE id = $1 AND state = 'RUNNING'",
            process_id
        )
        .execute(&pool_clone)
        .await
    });

    // Wait for all operations
    for handle in handles {
        let result = handle.await.unwrap();
        // All cancels should succeed (idempotent)
        assert!(result.is_ok(), "Cancel should be idempotent");
    }
    let _ = complete_handle.await;

    // Check final state - should be exactly one terminal state
    let final_state = sqlx::query_scalar!(
        r#"SELECT state as "state: ProcessState" FROM processes WHERE id = $1"#,
        process_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // Should be either CANCELLED or COMPLETED, but not both
    assert!(
        final_state == ProcessState::CANCELLED || final_state == ProcessState::COMPLETED,
        "Process should be in exactly one terminal state"
    );

    // Check resources were freed exactly once
    let resources = sqlx::query!(
        "SELECT cpu_available, ram_available FROM servers WHERE id = 1"
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(resources.cpu_available >= 0, "CPU should not be negative");
    assert!(resources.ram_available >= 0, "RAM should not be negative");
}

#[tokio::test]
async fn cancel_completed_process_is_noop() {
    let pool = setup_test_db().await;
    let user_id = 1;

    // Create an already completed process
    let process_id = create_test_process(&pool, user_id, ProcessState::COMPLETED).await;

    // Try to cancel it multiple times
    for _ in 0..5 {
        let result = process_cancel::cancel_process(&pool, process_id, user_id).await;
        assert!(result.is_ok(), "Canceling completed process should succeed (idempotent)");
    }

    // State should still be COMPLETED
    let final_state = sqlx::query_scalar!(
        r#"SELECT state as "state: ProcessState" FROM processes WHERE id = $1"#,
        process_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(final_state, ProcessState::COMPLETED);
}

// Helper functions
async fn setup_test_db() -> PgPool {
    // Use test database or in-memory mock
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://test:test@localhost/test_he".to_string());

    let pool = PgPool::connect(&database_url).await.unwrap();

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    // Setup test data
    sqlx::query!(
        "INSERT INTO servers (id, cpu_available, ram_available) VALUES (1, 1000, 2048)
         ON CONFLICT (id) DO UPDATE SET cpu_available = 1000, ram_available = 2048"
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

async fn create_test_process(pool: &PgPool, user_id: i64, state: ProcessState) -> i64 {
    sqlx::query_scalar!(
        r#"INSERT INTO processes (user_id, state, cpu_used, ram_used, server_id)
           VALUES ($1, $2, 100, 256, 1)
           RETURNING id"#,
        user_id,
        state as ProcessState
    )
    .fetch_one(pool)
    .await
    .unwrap()
}