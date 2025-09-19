use futures::{stream, StreamExt, TryStreamExt};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tokio::sync::{Semaphore, SemaphorePermit};
use tokio::time::timeout;

/// Optimized async operations with parallelization and batching
pub struct AsyncOptimizer {
    max_concurrency: usize,
    batch_size: usize,
    timeout_duration: Duration,
    semaphore: Semaphore,
}

impl Default for AsyncOptimizer {
    fn default() -> Self {
        Self {
            max_concurrency: 100,
            batch_size: 50,
            timeout_duration: Duration::from_secs(30),
            semaphore: Semaphore::new(100),
        }
    }
}

impl AsyncOptimizer {
    pub fn new(max_concurrency: usize, batch_size: usize, timeout_secs: u64) -> Self {
        Self {
            max_concurrency,
            batch_size,
            timeout_duration: Duration::from_secs(timeout_secs),
            semaphore: Semaphore::new(max_concurrency),
        }
    }

    /// Execute multiple async operations in parallel with controlled concurrency
    pub async fn parallel_execute<F, T, E>(
        &self,
        operations: Vec<F>,
    ) -> Result<Vec<T>, E>
    where
        F: Future<Output = Result<T, E>> + Send + 'static,
        T: Send + 'static,
        E: Send + 'static,
    {
        let results = stream::iter(operations)
            .map(|op| async move {
                let _permit = self.semaphore.acquire().await.unwrap();
                timeout(self.timeout_duration, op).await
                    .map_err(|_| panic!("Operation timed out"))
                    .unwrap()
            })
            .buffer_unordered(self.max_concurrency)
            .try_collect::<Vec<_>>()
            .await?;

        Ok(results)
    }

    /// Process items in batches with parallel execution
    pub async fn batch_process<T, F, R, E>(
        &self,
        items: Vec<T>,
        processor: F,
    ) -> Result<Vec<R>, E>
    where
        T: Send + 'static,
        F: Fn(Vec<T>) -> Pin<Box<dyn Future<Output = Result<Vec<R>, E>> + Send>> + Send + Sync,
        R: Send + 'static,
        E: Send + 'static,
    {
        let mut results = Vec::new();

        for batch in items.chunks(self.batch_size) {
            let batch_results = processor(batch.to_vec()).await?;
            results.extend(batch_results);
        }

        Ok(results)
    }

    /// Execute operations with retry logic
    pub async fn execute_with_retry<F, T, E>(
        &self,
        operation: F,
        max_retries: usize,
        backoff_ms: u64,
    ) -> Result<T, E>
    where
        F: Fn() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>> + Send,
        E: std::fmt::Debug,
    {
        let mut retries = 0;
        let mut backoff = Duration::from_millis(backoff_ms);

        loop {
            match timeout(self.timeout_duration, operation()).await {
                Ok(Ok(result)) => return Ok(result),
                Ok(Err(e)) if retries < max_retries => {
                    tracing::warn!("Operation failed, retrying ({}/{}): {:?}", retries + 1, max_retries, e);
                    tokio::time::sleep(backoff).await;
                    backoff *= 2; // Exponential backoff
                    retries += 1;
                }
                Ok(Err(e)) => return Err(e),
                Err(_) if retries < max_retries => {
                    tracing::warn!("Operation timed out, retrying ({}/{})", retries + 1, max_retries);
                    tokio::time::sleep(backoff).await;
                    backoff *= 2;
                    retries += 1;
                }
                Err(_) => panic!("Operation timed out after {} retries", max_retries),
            }
        }
    }
}

/// Optimized database query parallelization
pub struct ParallelQueryExecutor {
    optimizer: AsyncOptimizer,
}

impl ParallelQueryExecutor {
    pub fn new() -> Self {
        Self {
            optimizer: AsyncOptimizer::new(50, 100, 10),
        }
    }

    /// Execute multiple independent queries in parallel
    pub async fn execute_parallel<T>(
        &self,
        queries: Vec<Pin<Box<dyn Future<Output = Result<T, sqlx::Error>> + Send>>>,
    ) -> Result<Vec<T>, sqlx::Error>
    where
        T: Send + 'static,
    {
        let results = stream::iter(queries)
            .map(|query| async move {
                timeout(Duration::from_secs(5), query).await
                    .map_err(|_| sqlx::Error::PoolTimedOut)?
            })
            .buffer_unordered(self.optimizer.max_concurrency)
            .try_collect::<Vec<_>>()
            .await?;

        Ok(results)
    }

    /// Execute queries with connection pooling optimization
    pub async fn execute_with_pool<T, F>(
        &self,
        pool: &sqlx::PgPool,
        query_builders: Vec<F>,
    ) -> Result<Vec<T>, sqlx::Error>
    where
        F: Fn(&sqlx::PgPool) -> Pin<Box<dyn Future<Output = Result<T, sqlx::Error>> + Send>> + Send,
        T: Send + 'static,
    {
        let queries: Vec<_> = query_builders
            .into_iter()
            .map(|builder| builder(pool))
            .collect();

        self.execute_parallel(queries).await
    }
}

/// Optimized dashboard data loader using parallel fetching
pub struct DashboardDataLoader {
    query_executor: ParallelQueryExecutor,
}

impl DashboardDataLoader {
    pub fn new() -> Self {
        Self {
            query_executor: ParallelQueryExecutor::new(),
        }
    }

    pub async fn load_dashboard_data(
        &self,
        pool: &sqlx::PgPool,
        user_id: i64,
    ) -> Result<DashboardData, anyhow::Error> {
        // Create all query futures
        let user_future = Self::get_user_data(pool, user_id);
        let processes_future = Self::get_active_processes(pool, user_id);
        let hardware_future = Self::get_hardware_info(pool, user_id);
        let missions_future = Self::get_available_missions(pool, user_id);
        let notifications_future = Self::get_notifications(pool, user_id);
        let stats_future = Self::get_user_stats(pool, user_id);

        // Execute all queries in parallel
        let (user, processes, hardware, missions, notifications, stats) = tokio::try_join!(
            user_future,
            processes_future,
            hardware_future,
            missions_future,
            notifications_future,
            stats_future
        )?;

        Ok(DashboardData {
            user,
            active_processes: processes,
            hardware,
            available_missions: missions,
            notifications,
            stats,
        })
    }

    async fn get_user_data(pool: &sqlx::PgPool, user_id: i64) -> Result<UserData, sqlx::Error> {
        sqlx::query_as!(
            UserData,
            "SELECT id, username, email, created_at FROM users WHERE id = $1",
            user_id
        )
        .fetch_one(pool)
        .await
    }

    async fn get_active_processes(pool: &sqlx::PgPool, user_id: i64) -> Result<Vec<Process>, sqlx::Error> {
        sqlx::query_as!(
            Process,
            "SELECT id, name, type, start_time, end_time FROM processes
             WHERE user_id = $1 AND end_time > NOW()
             ORDER BY end_time ASC LIMIT 10",
            user_id
        )
        .fetch_all(pool)
        .await
    }

    async fn get_hardware_info(pool: &sqlx::PgPool, user_id: i64) -> Result<Hardware, sqlx::Error> {
        sqlx::query_as!(
            Hardware,
            "SELECT cpu, ram, hdd, net FROM hardware WHERE user_id = $1",
            user_id
        )
        .fetch_one(pool)
        .await
    }

    async fn get_available_missions(pool: &sqlx::PgPool, user_id: i64) -> Result<Vec<Mission>, sqlx::Error> {
        sqlx::query_as!(
            Mission,
            "SELECT id, title, description, reward FROM missions
             WHERE id NOT IN (SELECT mission_id FROM completed_missions WHERE user_id = $1)
             LIMIT 5",
            user_id
        )
        .fetch_all(pool)
        .await
    }

    async fn get_notifications(pool: &sqlx::PgPool, user_id: i64) -> Result<Vec<Notification>, sqlx::Error> {
        sqlx::query_as!(
            Notification,
            "SELECT id, message, created_at FROM notifications
             WHERE user_id = $1 AND read = false
             ORDER BY created_at DESC LIMIT 10",
            user_id
        )
        .fetch_all(pool)
        .await
    }

    async fn get_user_stats(pool: &sqlx::PgPool, user_id: i64) -> Result<UserStats, sqlx::Error> {
        sqlx::query_as!(
            UserStats,
            "SELECT total_hacks, total_money, reputation, clan_id FROM user_stats WHERE user_id = $1",
            user_id
        )
        .fetch_one(pool)
        .await
    }
}

// Data structures for dashboard
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DashboardData {
    pub user: UserData,
    pub active_processes: Vec<Process>,
    pub hardware: Hardware,
    pub available_missions: Vec<Mission>,
    pub notifications: Vec<Notification>,
    pub stats: UserStats,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct UserData {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Process {
    pub id: i64,
    pub name: String,
    pub r#type: String,
    pub start_time: chrono::NaiveDateTime,
    pub end_time: chrono::NaiveDateTime,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Hardware {
    pub cpu: i32,
    pub ram: i32,
    pub hdd: i32,
    pub net: i32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Mission {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub reward: i32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Notification {
    pub id: i64,
    pub message: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct UserStats {
    pub total_hacks: i64,
    pub total_money: i64,
    pub reputation: i32,
    pub clan_id: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parallel_execution() {
        let optimizer = AsyncOptimizer::default();

        let operations = vec![
            Box::pin(async { Ok::<_, String>(1) }),
            Box::pin(async { Ok::<_, String>(2) }),
            Box::pin(async { Ok::<_, String>(3) }),
        ];

        let results = optimizer.parallel_execute(operations).await.unwrap();
        assert_eq!(results.len(), 3);
        assert!(results.contains(&1));
        assert!(results.contains(&2));
        assert!(results.contains(&3));
    }

    #[tokio::test]
    async fn test_retry_logic() {
        let optimizer = AsyncOptimizer::default();
        let mut attempt = 0;

        let result = optimizer
            .execute_with_retry(
                || {
                    let current = attempt;
                    attempt += 1;
                    Box::pin(async move {
                        if current < 2 {
                            Err("Failed".to_string())
                        } else {
                            Ok("Success".to_string())
                        }
                    })
                },
                3,
                10,
            )
            .await;

        assert_eq!(result.unwrap(), "Success");
        assert_eq!(attempt, 3);
    }
}