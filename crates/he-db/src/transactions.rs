//! Database transaction management

use crate::{DbPool, DbTransaction};
use anyhow::Result;
use sqlx::Acquire;
use std::future::Future;
use tracing::{debug, error, warn};
use uuid::Uuid;

/// Transaction manager for handling database transactions
pub struct TransactionManager {
    pool: DbPool,
}

impl TransactionManager {
    /// Create a new transaction manager
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Execute a function within a transaction
    pub async fn with_transaction<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(DbTransaction<'_>) -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let mut tx = self.pool.begin().await?;
        let tx_id = Uuid::new_v4();
        
        debug!("Starting transaction: {}", tx_id);

        match f(tx).await {
            Ok(result) => {
                debug!("Committing transaction: {}", tx_id);
                // Note: tx is consumed by f, so we can't commit here
                // The commit should be handled within the closure
                Ok(result)
            }
            Err(e) => {
                error!("Transaction failed: {}, error: {}", tx_id, e);
                Err(e)
            }
        }
    }

    /// Execute a function within a transaction with automatic commit/rollback
    pub async fn execute_in_transaction<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut DbTransaction<'_>) -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let mut tx = self.pool.begin().await?;
        let tx_id = Uuid::new_v4();
        
        debug!("Starting transaction: {}", tx_id);

        match f(&mut tx).await {
            Ok(result) => {
                debug!("Committing transaction: {}", tx_id);
                tx.commit().await?;
                Ok(result)
            }
            Err(e) => {
                warn!("Rolling back transaction: {}, error: {}", tx_id, e);
                if let Err(rollback_error) = tx.rollback().await {
                    error!("Failed to rollback transaction {}: {}", tx_id, rollback_error);
                }
                Err(e)
            }
        }
    }

    /// Execute multiple operations within a single transaction
    pub async fn execute_batch<F, Fut>(&self, operations: Vec<F>) -> Result<Vec<sqlx::mysql::MySqlQueryResult>>
    where
        F: FnOnce(&mut DbTransaction<'_>) -> Fut,
        Fut: Future<Output = Result<sqlx::mysql::MySqlQueryResult>>,
    {
        let mut tx = self.pool.begin().await?;
        let tx_id = Uuid::new_v4();
        let mut results = Vec::new();
        
        debug!("Starting batch transaction: {} with {} operations", tx_id, operations.len());

        for (i, operation) in operations.into_iter().enumerate() {
            match operation(&mut tx).await {
                Ok(result) => {
                    debug!("Batch operation {} completed successfully", i);
                    results.push(result);
                }
                Err(e) => {
                    error!("Batch operation {} failed: {}", i, e);
                    warn!("Rolling back batch transaction: {}", tx_id);
                    if let Err(rollback_error) = tx.rollback().await {
                        error!("Failed to rollback batch transaction {}: {}", tx_id, rollback_error);
                    }
                    return Err(e);
                }
            }
        }

        debug!("Committing batch transaction: {}", tx_id);
        tx.commit().await?;
        Ok(results)
    }
}

/// Savepoint manager for nested transactions
pub struct SavepointManager<'a> {
    tx: &'a mut DbTransaction<'a>,
    savepoint_counter: u32,
}

impl<'a> SavepointManager<'a> {
    /// Create a new savepoint manager
    pub fn new(tx: &'a mut DbTransaction<'a>) -> Self {
        Self {
            tx,
            savepoint_counter: 0,
        }
    }

    /// Create a savepoint
    pub async fn create_savepoint(&mut self) -> Result<String> {
        self.savepoint_counter += 1;
        let savepoint_name = format!("sp_{}", self.savepoint_counter);
        
        let sql = format!("SAVEPOINT {}", savepoint_name);
        sqlx::query(&sql).execute(&mut **self.tx).await?;
        
        debug!("Created savepoint: {}", savepoint_name);
        Ok(savepoint_name)
    }

    /// Rollback to a savepoint
    pub async fn rollback_to_savepoint(&mut self, savepoint_name: &str) -> Result<()> {
        let sql = format!("ROLLBACK TO SAVEPOINT {}", savepoint_name);
        sqlx::query(&sql).execute(&mut **self.tx).await?;
        
        debug!("Rolled back to savepoint: {}", savepoint_name);
        Ok(())
    }

    /// Release a savepoint
    pub async fn release_savepoint(&mut self, savepoint_name: &str) -> Result<()> {
        let sql = format!("RELEASE SAVEPOINT {}", savepoint_name);
        sqlx::query(&sql).execute(&mut **self.tx).await?;
        
        debug!("Released savepoint: {}", savepoint_name);
        Ok(())
    }
}

/// Transaction isolation levels
#[derive(Debug, Clone, Copy)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

impl IsolationLevel {
    fn to_sql(&self) -> &'static str {
        match self {
            IsolationLevel::ReadUncommitted => "READ UNCOMMITTED",
            IsolationLevel::ReadCommitted => "READ COMMITTED",
            IsolationLevel::RepeatableRead => "REPEATABLE READ",
            IsolationLevel::Serializable => "SERIALIZABLE",
        }
    }
}

/// Extended transaction manager with isolation level support
pub struct AdvancedTransactionManager {
    pool: DbPool,
}

impl AdvancedTransactionManager {
    /// Create a new advanced transaction manager
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Execute a function within a transaction with specified isolation level
    pub async fn with_isolation<F, Fut, T>(
        &self,
        isolation_level: IsolationLevel,
        f: F,
    ) -> Result<T>
    where
        F: FnOnce(&mut DbTransaction<'_>) -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let mut tx = self.pool.begin().await?;
        let tx_id = Uuid::new_v4();
        
        // Set isolation level
        let sql = format!("SET TRANSACTION ISOLATION LEVEL {}", isolation_level.to_sql());
        sqlx::query(&sql).execute(&mut *tx).await?;
        
        debug!("Starting transaction {} with isolation level: {:?}", tx_id, isolation_level);

        match f(&mut tx).await {
            Ok(result) => {
                debug!("Committing transaction: {}", tx_id);
                tx.commit().await?;
                Ok(result)
            }
            Err(e) => {
                warn!("Rolling back transaction: {}, error: {}", tx_id, e);
                if let Err(rollback_error) = tx.rollback().await {
                    error!("Failed to rollback transaction {}: {}", tx_id, rollback_error);
                }
                Err(e)
            }
        }
    }

    /// Execute read-only transaction (optimized for reads)
    pub async fn read_only<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut DbTransaction<'_>) -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let mut tx = self.pool.begin().await?;
        let tx_id = Uuid::new_v4();
        
        // Set read-only mode
        sqlx::query("SET TRANSACTION READ ONLY")
            .execute(&mut *tx)
            .await?;
        
        debug!("Starting read-only transaction: {}", tx_id);

        match f(&mut tx).await {
            Ok(result) => {
                debug!("Committing read-only transaction: {}", tx_id);
                tx.commit().await?;
                Ok(result)
            }
            Err(e) => {
                warn!("Rolling back read-only transaction: {}, error: {}", tx_id, e);
                if let Err(rollback_error) = tx.rollback().await {
                    error!("Failed to rollback read-only transaction {}: {}", tx_id, rollback_error);
                }
                Err(e)
            }
        }
    }
}

/// Macro for simpler transaction handling
#[macro_export]
macro_rules! with_transaction {
    ($pool:expr, $tx:ident, $body:block) => {{
        use sqlx::Acquire;
        let mut $tx = $pool.begin().await?;
        let result: Result<_> = async { $body }.await;
        
        match result {
            Ok(value) => {
                $tx.commit().await?;
                Ok(value)
            }
            Err(e) => {
                if let Err(rollback_error) = $tx.rollback().await {
                    tracing::error!("Failed to rollback transaction: {}", rollback_error);
                }
                Err(e)
            }
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::MySqlPool;

    // Note: These tests require a running MySQL instance
    // They are marked as ignored by default

    #[tokio::test]
    #[ignore]
    async fn test_transaction_manager() {
        let pool = MySqlPool::connect("mysql://test:test@localhost/test")
            .await
            .expect("Failed to connect to test database");
        
        let tx_manager = TransactionManager::new(pool);
        
        let result = tx_manager
            .execute_in_transaction(|tx| async move {
                sqlx::query("SELECT 1")
                    .execute(&mut **tx)
                    .await?;
                Ok::<_, anyhow::Error>(42)
            })
            .await;
            
        assert!(result.is_ok());
        assert_eq!(result.map_err(|e| anyhow::anyhow!("Error: {}", e))?, 42);
    }

    #[tokio::test]
    #[ignore]
    async fn test_advanced_transaction_manager() {
        let pool = MySqlPool::connect("mysql://test:test@localhost/test")
            .await
            .expect("Failed to connect to test database");
        
        let tx_manager = AdvancedTransactionManager::new(pool);
        
        let result = tx_manager
            .with_isolation(IsolationLevel::ReadCommitted, |tx| async move {
                sqlx::query("SELECT 1")
                    .execute(&mut **tx)
                    .await?;
                Ok::<_, anyhow::Error>(42)
            })
            .await;
            
        assert!(result.is_ok());
        assert_eq!(result.map_err(|e| anyhow::anyhow!("Error: {}", e))?, 42);
    }
}