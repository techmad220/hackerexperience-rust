//! Database migration support for Helix

use crate::connection::DatabaseConnection;
use he_helix_core::{HelixError, HelixResult};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Migration metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    /// Migration ID/version
    pub id: String,
    /// Migration name
    pub name: String,
    /// SQL to apply the migration
    pub up_sql: String,
    /// SQL to rollback the migration
    pub down_sql: String,
    /// Dependencies (other migrations that must be applied first)
    pub dependencies: Vec<String>,
    /// Database this migration applies to
    pub database: String,
}

impl Migration {
    pub fn new<S: Into<String>>(
        id: S,
        name: S,
        database: S,
        up_sql: S,
        down_sql: S,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            up_sql: up_sql.into(),
            down_sql: down_sql.into(),
            dependencies: Vec::new(),
            database: database.into(),
        }
    }

    pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies = dependencies;
        self
    }
}

/// Migration manager for handling database schema changes
#[derive(Debug)]
pub struct MigrationManager {
    /// All available migrations
    migrations: HashMap<String, Vec<Migration>>,
}

impl MigrationManager {
    /// Create a new migration manager
    pub fn new() -> Self {
        Self {
            migrations: HashMap::new(),
        }
    }

    /// Add migrations for a database
    pub fn add_migrations(&mut self, database: String, migrations: Vec<Migration>) {
        self.migrations.insert(database, migrations);
    }

    /// Add a single migration
    pub fn add_migration(&mut self, migration: Migration) {
        let database = migration.database.clone();
        self.migrations
            .entry(database)
            .or_insert_with(Vec::new)
            .push(migration);
    }

    /// Get migrations for a database
    pub fn get_migrations(&self, database: &str) -> Option<&Vec<Migration>> {
        self.migrations.get(database)
    }

    /// Apply all pending migrations for a database
    pub async fn migrate(&self, database: &str, connection: &DatabaseConnection) -> HelixResult<()> {
        // First ensure the migration table exists
        self.ensure_migration_table(connection).await?;

        let migrations = self
            .get_migrations(database)
            .ok_or_else(|| HelixError::not_found(format!("No migrations found for database: {}", database)))?;

        // Get applied migrations
        let applied = self.get_applied_migrations(connection).await?;

        // Sort migrations by dependencies and ID
        let sorted_migrations = self.sort_migrations(migrations)?;

        // Apply pending migrations
        for migration in sorted_migrations {
            if !applied.contains(&migration.id) {
                self.apply_migration(&migration, connection).await?;
                tracing::info!("Applied migration: {} - {}", migration.id, migration.name);
            }
        }

        Ok(())
    }

    /// Rollback the last migration for a database
    pub async fn rollback(&self, database: &str, connection: &DatabaseConnection) -> HelixResult<()> {
        let migrations = self
            .get_migrations(database)
            .ok_or_else(|| HelixError::not_found(format!("No migrations found for database: {}", database)))?;

        // Get applied migrations
        let applied = self.get_applied_migrations(connection).await?;

        if applied.is_empty() {
            return Err(HelixError::validation("No migrations to rollback"));
        }

        // Find the last applied migration
        let mut sorted_migrations = self.sort_migrations(migrations)?;
        sorted_migrations.reverse();

        let last_migration = sorted_migrations
            .iter()
            .find(|m| applied.contains(&m.id))
            .ok_or_else(|| HelixError::not_found("Last applied migration not found"))?;

        // Apply rollback
        self.rollback_migration(last_migration, connection).await?;
        tracing::info!("Rolled back migration: {} - {}", last_migration.id, last_migration.name);

        Ok(())
    }

    /// Get the status of migrations for a database
    pub async fn status(&self, database: &str, connection: &DatabaseConnection) -> HelixResult<Vec<MigrationStatus>> {
        let migrations = self
            .get_migrations(database)
            .ok_or_else(|| HelixError::not_found(format!("No migrations found for database: {}", database)))?;

        let applied = self.get_applied_migrations(connection).await?;
        let sorted_migrations = self.sort_migrations(migrations)?;

        let status: Vec<MigrationStatus> = sorted_migrations
            .into_iter()
            .map(|migration| MigrationStatus {
                id: migration.id.clone(),
                name: migration.name.clone(),
                applied: applied.contains(&migration.id),
                database: migration.database.clone(),
            })
            .collect();

        Ok(status)
    }

    /// Ensure the migration table exists
    async fn ensure_migration_table(&self, connection: &DatabaseConnection) -> HelixResult<()> {
        let sql = match connection {
            DatabaseConnection::PostgreSQL(_) => {
                r#"
                CREATE TABLE IF NOT EXISTS _helix_migrations (
                    id VARCHAR(255) PRIMARY KEY,
                    name VARCHAR(255) NOT NULL,
                    applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
                );
                "#
            }
            DatabaseConnection::MySQL(_) => {
                r#"
                CREATE TABLE IF NOT EXISTS _helix_migrations (
                    id VARCHAR(255) PRIMARY KEY,
                    name VARCHAR(255) NOT NULL,
                    applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );
                "#
            }
            DatabaseConnection::SQLite(_) => {
                r#"
                CREATE TABLE IF NOT EXISTS _helix_migrations (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
                );
                "#
            }
        };

        match connection {
            DatabaseConnection::PostgreSQL(pool) => {
                sqlx::query(sql).execute(pool).await.map_err(HelixError::Database)?;
            }
            DatabaseConnection::MySQL(pool) => {
                sqlx::query(sql).execute(pool).await.map_err(HelixError::Database)?;
            }
            DatabaseConnection::SQLite(pool) => {
                sqlx::query(sql).execute(pool).await.map_err(HelixError::Database)?;
            }
        }

        Ok(())
    }

    /// Get list of applied migration IDs
    async fn get_applied_migrations(&self, connection: &DatabaseConnection) -> HelixResult<Vec<String>> {
        let sql = "SELECT id FROM _helix_migrations ORDER BY applied_at";

        match connection {
            DatabaseConnection::PostgreSQL(pool) => {
                let rows = sqlx::query_as::<_, (String,)>(sql)
                    .fetch_all(pool)
                    .await
                    .map_err(HelixError::Database)?;
                Ok(rows.into_iter().map(|(id,)| id).collect())
            }
            DatabaseConnection::MySQL(pool) => {
                let rows = sqlx::query_as::<_, (String,)>(sql)
                    .fetch_all(pool)
                    .await
                    .map_err(HelixError::Database)?;
                Ok(rows.into_iter().map(|(id,)| id).collect())
            }
            DatabaseConnection::SQLite(pool) => {
                let rows = sqlx::query_as::<_, (String,)>(sql)
                    .fetch_all(pool)
                    .await
                    .map_err(HelixError::Database)?;
                Ok(rows.into_iter().map(|(id,)| id).collect())
            }
        }
    }

    /// Apply a single migration
    async fn apply_migration(&self, migration: &Migration, connection: &DatabaseConnection) -> HelixResult<()> {
        // Execute the migration SQL
        match connection {
            DatabaseConnection::PostgreSQL(pool) => {
                sqlx::query(&migration.up_sql).execute(pool).await.map_err(HelixError::Database)?;
            }
            DatabaseConnection::MySQL(pool) => {
                sqlx::query(&migration.up_sql).execute(pool).await.map_err(HelixError::Database)?;
            }
            DatabaseConnection::SQLite(pool) => {
                sqlx::query(&migration.up_sql).execute(pool).await.map_err(HelixError::Database)?;
            }
        }

        // Record the migration as applied
        let sql = "INSERT INTO _helix_migrations (id, name) VALUES (?, ?)";
        match connection {
            DatabaseConnection::PostgreSQL(pool) => {
                sqlx::query(sql)
                    .bind(&migration.id)
                    .bind(&migration.name)
                    .execute(pool)
                    .await
                    .map_err(HelixError::Database)?;
            }
            DatabaseConnection::MySQL(pool) => {
                sqlx::query(sql)
                    .bind(&migration.id)
                    .bind(&migration.name)
                    .execute(pool)
                    .await
                    .map_err(HelixError::Database)?;
            }
            DatabaseConnection::SQLite(pool) => {
                sqlx::query(sql)
                    .bind(&migration.id)
                    .bind(&migration.name)
                    .execute(pool)
                    .await
                    .map_err(HelixError::Database)?;
            }
        }

        Ok(())
    }

    /// Rollback a single migration
    async fn rollback_migration(&self, migration: &Migration, connection: &DatabaseConnection) -> HelixResult<()> {
        // Execute the rollback SQL
        match connection {
            DatabaseConnection::PostgreSQL(pool) => {
                sqlx::query(&migration.down_sql).execute(pool).await.map_err(HelixError::Database)?;
            }
            DatabaseConnection::MySQL(pool) => {
                sqlx::query(&migration.down_sql).execute(pool).await.map_err(HelixError::Database)?;
            }
            DatabaseConnection::SQLite(pool) => {
                sqlx::query(&migration.down_sql).execute(pool).await.map_err(HelixError::Database)?;
            }
        }

        // Remove the migration record
        let sql = "DELETE FROM _helix_migrations WHERE id = ?";
        match connection {
            DatabaseConnection::PostgreSQL(pool) => {
                sqlx::query(sql)
                    .bind(&migration.id)
                    .execute(pool)
                    .await
                    .map_err(HelixError::Database)?;
            }
            DatabaseConnection::MySQL(pool) => {
                sqlx::query(sql)
                    .bind(&migration.id)
                    .execute(pool)
                    .await
                    .map_err(HelixError::Database)?;
            }
            DatabaseConnection::SQLite(pool) => {
                sqlx::query(sql)
                    .bind(&migration.id)
                    .execute(pool)
                    .await
                    .map_err(HelixError::Database)?;
            }
        }

        Ok(())
    }

    /// Sort migrations by dependencies
    fn sort_migrations(&self, migrations: &[Migration]) -> HelixResult<Vec<Migration>> {
        let mut sorted = Vec::new();
        let mut remaining: Vec<_> = migrations.iter().collect();
        
        // Simple topological sort
        while !remaining.is_empty() {
            let initial_len = remaining.len();
            
            remaining.retain(|migration| {
                // Check if all dependencies are satisfied
                for dep in &migration.dependencies {
                    if !sorted.iter().any(|m| m.id == *dep) {
                        return true; // Keep in remaining
                    }
                }
                
                // All dependencies satisfied, add to sorted
                sorted.push((*migration).clone());
                false // Remove from remaining
            });
            
            // Check for circular dependencies
            if remaining.len() == initial_len {
                return Err(HelixError::validation("Circular dependency detected in migrations"));
            }
        }
        
        Ok(sorted)
    }
}

impl Default for MigrationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Status of a migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStatus {
    pub id: String,
    pub name: String,
    pub applied: bool,
    pub database: String,
}