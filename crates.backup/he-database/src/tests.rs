//! Database layer tests

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::models::*;
    use crate::queries::*;
    use anyhow::Result;
    use chrono::Utc;
    use sqlx::{PgPool, postgres::PgPoolOptions};
    use uuid::Uuid;

    // Test database URL - should be set in environment
    fn get_test_database_url() -> String {
        std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://test:test@localhost:5432/test_he".to_string())
    }

    async fn create_test_pool() -> Result<PgPool> {
        let database_url = get_test_database_url();
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;
        Ok(pool)
    }

    async fn cleanup_test_user(pool: &PgPool, email: &str) {
        let _ = sqlx::query!("DELETE FROM users WHERE email = $1", email)
            .execute(pool)
            .await;
    }

    mod user_tests {
        use super::*;

        #[tokio::test]
        async fn test_create_user() {
            let pool = match create_test_pool().await {
                Ok(p) => p,
                Err(_) => return, // Skip if no test database
            };

            let email = format!("test_{}@example.com", Uuid::new_v4());
            cleanup_test_user(&pool, &email).await;

            let result = UserQueries::create_user(
                &pool,
                "testuser",
                &email,
                "password123"
            ).await;

            assert!(result.is_ok());

            let user = result.unwrap();
            assert_eq!(user.login, "testuser");
            assert_eq!(user.email, email);
            assert_ne!(user.pwd, "password123"); // Should be hashed

            cleanup_test_user(&pool, &email).await;
        }

        #[tokio::test]
        async fn test_get_user_by_email() {
            let pool = match create_test_pool().await {
                Ok(p) => p,
                Err(_) => return;
            };

            let email = format!("test_{}@example.com", Uuid::new_v4());
            cleanup_test_user(&pool, &email).await;

            // Create user first
            let created = UserQueries::create_user(
                &pool,
                "findme",
                &email,
                "password123"
            ).await.unwrap();

            // Find by email
            let found = UserQueries::get_user_by_email(&pool, &email).await.unwrap();
            assert!(found.is_some());
            assert_eq!(found.unwrap().id, created.id);

            // Try non-existent email
            let not_found = UserQueries::get_user_by_email(&pool, "nonexistent@example.com").await.unwrap();
            assert!(not_found.is_none());

            cleanup_test_user(&pool, &email).await;
        }

        #[tokio::test]
        async fn test_password_verification() {
            let pool = match create_test_pool().await {
                Ok(p) => p,
                Err(_) => return;
            };

            let email = format!("test_{}@example.com", Uuid::new_v4());
            cleanup_test_user(&pool, &email).await;

            let user = UserQueries::create_user(
                &pool,
                "pwtest",
                &email,
                "correct_password"
            ).await.unwrap();

            // Test correct password
            let valid = UserQueries::verify_password(&user, "correct_password").await.unwrap();
            assert!(valid);

            // Test wrong password
            let invalid = UserQueries::verify_password(&user, "wrong_password").await.unwrap();
            assert!(!invalid);

            cleanup_test_user(&pool, &email).await;
        }

        #[tokio::test]
        async fn test_update_last_login() {
            let pool = match create_test_pool().await {
                Ok(p) => p,
                Err(_) => return;
            };

            let email = format!("test_{}@example.com", Uuid::new_v4());
            cleanup_test_user(&pool, &email).await;

            let user = UserQueries::create_user(
                &pool,
                "logintest",
                &email,
                "password123"
            ).await.unwrap();

            let old_login_time = user.last_login;

            // Update last login
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            UserQueries::update_last_login(&pool, user.id, "192.168.1.1").await.unwrap();

            // Fetch updated user
            let updated = UserQueries::get_user_by_id(&pool, user.id).await.unwrap().unwrap();
            assert!(updated.last_login > old_login_time);
            assert_eq!(updated.last_ip, "192.168.1.1");
            assert!(updated.online);

            cleanup_test_user(&pool, &email).await;
        }
    }

    mod process_tests {
        use super::*;

        async fn create_test_user_for_process(pool: &PgPool) -> i64 {
            let email = format!("test_{}@example.com", Uuid::new_v4());
            let user = UserQueries::create_user(
                pool,
                "processuser",
                &email,
                "password123"
            ).await.unwrap();
            user.id
        }

        #[tokio::test]
        async fn test_create_process() {
            let pool = match create_test_pool().await {
                Ok(p) => p,
                Err(_) => return;
            };

            let user_id = create_test_user_for_process(&pool).await;

            let process = ProcessQueries::create_process(
                &pool,
                user_id,
                "hack",
                &format!("pc_{}", user_id),
                Some("target_pc_123".to_string())
            ).await;

            assert!(process.is_ok());

            let p = process.unwrap();
            assert_eq!(p.user_id, user_id);
            assert_eq!(p.process_type, "hack");
            assert!(p.end_time > p.start_time);

            // Cleanup
            let _ = sqlx::query!("DELETE FROM processes WHERE user_id = $1", user_id)
                .execute(&pool)
                .await;
            let _ = sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
                .execute(&pool)
                .await;
        }

        #[tokio::test]
        async fn test_create_process_with_duration() {
            let pool = match create_test_pool().await {
                Ok(p) => p,
                Err(_) => return;
            };

            let user_id = create_test_user_for_process(&pool).await;

            let duration_seconds = 300; // 5 minutes
            let process = ProcessQueries::create_process_with_duration(
                &pool,
                user_id,
                "download",
                &format!("pc_{}", user_id),
                None,
                duration_seconds
            ).await;

            assert!(process.is_ok());

            let p = process.unwrap();
            let expected_duration = chrono::Duration::seconds(duration_seconds as i64);
            let actual_duration = p.end_time - p.start_time;

            // Allow 1 second variance for processing time
            assert!((actual_duration.num_seconds() - expected_duration.num_seconds()).abs() <= 1);

            // Cleanup
            let _ = sqlx::query!("DELETE FROM processes WHERE user_id = $1", user_id)
                .execute(&pool)
                .await;
            let _ = sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
                .execute(&pool)
                .await;
        }

        #[tokio::test]
        async fn test_get_user_processes() {
            let pool = match create_test_pool().await {
                Ok(p) => p,
                Err(_) => return;
            };

            let user_id = create_test_user_for_process(&pool).await;

            // Create multiple processes
            for i in 0..3 {
                ProcessQueries::create_process(
                    &pool,
                    user_id,
                    &format!("process_{}", i),
                    &format!("pc_{}", user_id),
                    None
                ).await.unwrap();
            }

            let processes = ProcessQueries::get_user_processes(&pool, user_id).await.unwrap();
            assert_eq!(processes.len(), 3);

            // Cleanup
            let _ = sqlx::query!("DELETE FROM processes WHERE user_id = $1", user_id)
                .execute(&pool)
                .await;
            let _ = sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
                .execute(&pool)
                .await;
        }

        #[tokio::test]
        async fn test_cancel_process() {
            let pool = match create_test_pool().await {
                Ok(p) => p,
                Err(_) => return;
            };

            let user_id = create_test_user_for_process(&pool).await;

            let process = ProcessQueries::create_process(
                &pool,
                user_id,
                "scan",
                &format!("pc_{}", user_id),
                None
            ).await.unwrap();

            // Cancel the process
            let cancelled = ProcessQueries::cancel_process(&pool, process.pid, user_id).await.unwrap();
            assert!(cancelled);

            // Try to cancel non-existent process
            let not_cancelled = ProcessQueries::cancel_process(&pool, 999999, user_id).await.unwrap();
            assert!(!not_cancelled);

            // Cleanup
            let _ = sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
                .execute(&pool)
                .await;
        }
    }

    mod hardware_tests {
        use super::*;

        #[tokio::test]
        async fn test_get_user_hardware() {
            let pool = match create_test_pool().await {
                Ok(p) => p,
                Err(_) => return;
            };

            let email = format!("test_{}@example.com", Uuid::new_v4());
            let user = UserQueries::create_user(
                &pool,
                "hwuser",
                &email,
                "password123"
            ).await.unwrap();

            // Get hardware (should create default if not exists)
            let hardware = HardwareQueries::get_user_hardware(&pool, user.id).await;

            assert!(hardware.is_ok());
            let hw = hardware.unwrap();
            assert_eq!(hw.user_id, user.id);
            assert_eq!(hw.cpu_mhz, 1000); // Default values
            assert_eq!(hw.ram_mb, 1024);

            // Cleanup
            let _ = sqlx::query!("DELETE FROM hardware WHERE user_id = $1", user.id)
                .execute(&pool)
                .await;
            let _ = sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
                .execute(&pool)
                .await;
        }
    }

    mod bank_tests {
        use super::*;

        async fn create_test_bank_accounts(pool: &PgPool) -> (String, String, i64) {
            let email1 = format!("test_{}@example.com", Uuid::new_v4());
            let user1 = UserQueries::create_user(
                pool,
                "banker1",
                &email1,
                "password123"
            ).await.unwrap();

            let email2 = format!("test_{}@example.com", Uuid::new_v4());
            let user2 = UserQueries::create_user(
                pool,
                "banker2",
                &email2,
                "password123"
            ).await.unwrap();

            // Create bank accounts
            let acc1 = format!("ACC{}", user1.id);
            let acc2 = format!("ACC{}", user2.id);

            let _ = sqlx::query!(
                "INSERT INTO bank_accounts (user_id, account_number, balance) VALUES ($1, $2, $3)",
                user1.id,
                &acc1,
                10000i64
            ).execute(pool).await;

            let _ = sqlx::query!(
                "INSERT INTO bank_accounts (user_id, account_number, balance) VALUES ($1, $2, $3)",
                user2.id,
                &acc2,
                5000i64
            ).execute(pool).await;

            (acc1, acc2, user1.id)
        }

        #[tokio::test]
        async fn test_money_transfer() {
            let pool = match create_test_pool().await {
                Ok(p) => p,
                Err(_) => return;
            };

            let (acc1, acc2, user_id) = create_test_bank_accounts(&pool).await;

            // Transfer money
            let success = BankQueries::transfer_money(&pool, &acc1, &acc2, 1000).await.unwrap();
            assert!(success);

            // Check balances
            let balance1: i64 = sqlx::query_scalar!(
                "SELECT balance FROM bank_accounts WHERE account_number = $1",
                &acc1
            ).fetch_one(&pool).await.unwrap();

            let balance2: i64 = sqlx::query_scalar!(
                "SELECT balance FROM bank_accounts WHERE account_number = $1",
                &acc2
            ).fetch_one(&pool).await.unwrap();

            assert_eq!(balance1, 9000);
            assert_eq!(balance2, 6000);

            // Try to transfer more than available
            let failed = BankQueries::transfer_money(&pool, &acc1, &acc2, 20000).await.unwrap();
            assert!(!failed);

            // Cleanup
            let _ = sqlx::query!("DELETE FROM bank_accounts WHERE account_number LIKE 'ACC%'")
                .execute(&pool)
                .await;
            let _ = sqlx::query!("DELETE FROM users WHERE login IN ('banker1', 'banker2')")
                .execute(&pool)
                .await;
        }
    }

    mod transaction_tests {
        use super::*;

        #[tokio::test]
        async fn test_transaction_rollback() {
            let pool = match create_test_pool().await {
                Ok(p) => p,
                Err(_) => return;
            };

            let email = format!("test_{}@example.com", Uuid::new_v4());

            // Start transaction
            let mut tx = pool.begin().await.unwrap();

            // Create user in transaction
            let user_id = sqlx::query_scalar!(
                r#"
                INSERT INTO users (login, pwd, email, online, last_login, created, last_act, last_ip)
                VALUES ($1, $2, $3, false, NOW(), NOW(), NOW(), '127.0.0.1')
                RETURNING id
                "#,
                "txuser",
                "hashed_password",
                &email
            )
            .fetch_one(&mut *tx)
            .await
            .unwrap();

            // Rollback transaction
            tx.rollback().await.unwrap();

            // User should not exist
            let user = UserQueries::get_user_by_id(&pool, user_id).await.unwrap();
            assert!(user.is_none());
        }

        #[tokio::test]
        async fn test_transaction_commit() {
            let pool = match create_test_pool().await {
                Ok(p) => p,
                Err(_) => return;
            };

            let email = format!("test_{}@example.com", Uuid::new_v4());
            cleanup_test_user(&pool, &email).await;

            // Start transaction
            let mut tx = pool.begin().await.unwrap();

            // Create user in transaction
            let user_id = sqlx::query_scalar!(
                r#"
                INSERT INTO users (login, pwd, email, online, last_login, created, last_act, last_ip)
                VALUES ($1, $2, $3, false, NOW(), NOW(), NOW(), '127.0.0.1')
                RETURNING id
                "#,
                "txcommituser",
                "hashed_password",
                &email
            )
            .fetch_one(&mut *tx)
            .await
            .unwrap();

            // Commit transaction
            tx.commit().await.unwrap();

            // User should exist
            let user = UserQueries::get_user_by_id(&pool, user_id).await.unwrap();
            assert!(user.is_some());

            cleanup_test_user(&pool, &email).await;
        }
    }

    mod performance_tests {
        use super::*;

        #[tokio::test]
        async fn test_connection_pool() {
            let pool = match create_test_pool().await {
                Ok(p) => p,
                Err(_) => return;
            };

            // Test concurrent queries
            let mut handles = vec![];

            for i in 0..10 {
                let pool_clone = pool.clone();
                let handle = tokio::spawn(async move {
                    let email = format!("concurrent_{}@example.com", i);
                    UserQueries::get_user_by_email(&pool_clone, &email).await
                });
                handles.push(handle);
            }

            // All queries should complete
            for handle in handles {
                assert!(handle.await.unwrap().is_ok());
            }
        }

        #[tokio::test]
        async fn test_bulk_insert_performance() {
            let pool = match create_test_pool().await {
                Ok(p) => p,
                Err(_) => return;
            };

            let start = std::time::Instant::now();

            // Insert multiple users
            for i in 0..100 {
                let email = format!("perf_test_{}@example.com", i);
                let _ = UserQueries::create_user(
                    &pool,
                    &format!("perfuser{}", i),
                    &email,
                    "password123"
                ).await;
            }

            let elapsed = start.elapsed();

            // Should complete in reasonable time (< 10 seconds)
            assert!(elapsed.as_secs() < 10);

            // Cleanup
            let _ = sqlx::query!("DELETE FROM users WHERE email LIKE 'perf_test_%@example.com'")
                .execute(&pool)
                .await;
        }
    }
}