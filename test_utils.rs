// Test utilities for HackerExperience

use once_cell::sync::Lazy;
use sqlx::PgPool;

/// Test database pool
pub static TEST_DB: Lazy<PgPool> = Lazy::new(|| {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            let database_url = std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://test:test@localhost:5432/test_he".to_string());
            PgPool::connect(&database_url)
                .await
                .expect("Failed to connect to test database")
        })
});

/// Create test user
pub async fn create_test_user(username: &str) -> i64 {
    // Implementation here
    1
}

/// Create test process
pub async fn create_test_process(user_id: i64, process_type: &str) -> i64 {
    // Implementation here
    1
}

/// Clean up test data
pub async fn cleanup_test_data() {
    // Implementation here
}

/// Test fixture macro
#[macro_export]
macro_rules! test_fixture {
    ($name:ident, $body:block) => {
        #[tokio::test]
        async fn $name() {
            // Setup
            let _guard = TEST_DB.clone();

            // Test
            $body

            // Cleanup
            cleanup_test_data().await;
        }
    };
}
