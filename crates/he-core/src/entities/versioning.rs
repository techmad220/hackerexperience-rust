use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VersioningError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Version not found: {0}")]
    VersionNotFound(String),
    #[error("Invalid version format: {0}")]
    InvalidVersion(String),
    #[error("Migration error: {0}")]
    Migration(String),
    #[error("Schema error: {0}")]
    Schema(String),
    #[error("Rollback error: {0}")]
    Rollback(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationType {
    Schema,
    Data,
    Config,
    Hotfix,
    Feature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationStatus {
    Pending,
    Running,
    Completed,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub build: Option<u32>,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            build: None,
        }
    }

    pub fn with_build(major: u32, minor: u32, patch: u32, build: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            build: Some(build),
        }
    }

    pub fn from_string(version_str: &str) -> Result<Self, VersioningError> {
        let parts: Vec<&str> = version_str.split('.').collect();
        
        if parts.len() < 3 || parts.len() > 4 {
            return Err(VersioningError::InvalidVersion(version_str.to_string()));
        }

        let major = parts[0].parse()
            .map_err(|_| VersioningError::InvalidVersion(version_str.to_string()))?;
        let minor = parts[1].parse()
            .map_err(|_| VersioningError::InvalidVersion(version_str.to_string()))?;
        let patch = parts[2].parse()
            .map_err(|_| VersioningError::InvalidVersion(version_str.to_string()))?;

        let build = if parts.len() == 4 {
            Some(parts[3].parse()
                .map_err(|_| VersioningError::InvalidVersion(version_str.to_string()))?)
        } else {
            None
        };

        Ok(Self {
            major,
            minor,
            patch,
            build,
        })
    }

    pub fn to_string(&self) -> String {
        if let Some(build) = self.build {
            format!("{}.{}.{}.{}", self.major, self.minor, self.patch, build)
        } else {
            format!("{}.{}.{}", self.major, self.minor, self.patch)
        }
    }

    pub fn is_compatible_with(&self, other: &Version) -> bool {
        // Same major version is compatible
        self.major == other.major
    }

    pub fn increment_patch(&mut self) {
        self.patch += 1;
    }

    pub fn increment_minor(&mut self) {
        self.minor += 1;
        self.patch = 0;
    }

    pub fn increment_major(&mut self) {
        self.major += 1;
        self.minor = 0;
        self.patch = 0;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMigration {
    pub id: u64,
    pub version: Version,
    pub name: String,
    pub description: String,
    pub migration_type: MigrationType,
    pub status: MigrationStatus,
    pub up_sql: String,
    pub down_sql: String,
    pub checksum: String,
    pub applied_at: Option<DateTime<Utc>>,
    pub rolled_back_at: Option<DateTime<Utc>>,
    pub execution_time_ms: Option<u64>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaVersion {
    pub version: Version,
    pub description: String,
    pub migrations: Vec<DatabaseMigration>,
    pub release_date: DateTime<Utc>,
    pub is_stable: bool,
    pub breaking_changes: Vec<String>,
    pub new_features: Vec<String>,
    pub bug_fixes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHistory {
    pub current_version: Version,
    pub target_version: Option<Version>,
    pub installed_versions: Vec<SchemaVersion>,
    pub pending_migrations: Vec<DatabaseMigration>,
    pub last_migration_at: Option<DateTime<Utc>>,
}

/// Database versioning and migration system ported from PHP Versioning class
/// Handles schema migrations, version tracking, and database evolution
pub struct Versioning {
    current_version: Option<Version>,
    migrations_table: String,
    version_table: String,
}

impl Versioning {
    /// Create new Versioning instance
    pub fn new() -> Self {
        Self {
            current_version: None,
            migrations_table: "schema_migrations".to_string(),
            version_table: "schema_versions".to_string(),
        }
    }

    /// Initialize versioning system
    pub fn initialize(&mut self) -> Result<(), VersioningError> {
        // Create versioning tables if they don't exist
        self.create_versioning_tables()?;
        
        // Load current version
        self.current_version = Some(self.get_current_version()?);
        
        Ok(())
    }

    /// Create versioning tables
    fn create_versioning_tables(&self) -> Result<(), VersioningError> {
        // In real implementation, execute SQL to create tables
        let _migrations_sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id BIGINT PRIMARY KEY AUTO_INCREMENT,
                version VARCHAR(50) NOT NULL,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                migration_type ENUM('Schema', 'Data', 'Config', 'Hotfix', 'Feature') NOT NULL,
                status ENUM('Pending', 'Running', 'Completed', 'Failed', 'RolledBack') NOT NULL,
                up_sql LONGTEXT NOT NULL,
                down_sql LONGTEXT NOT NULL,
                checksum VARCHAR(64) NOT NULL,
                applied_at DATETIME NULL,
                rolled_back_at DATETIME NULL,
                execution_time_ms BIGINT NULL,
                error_message TEXT NULL,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                UNIQUE KEY unique_version_name (version, name),
                INDEX idx_version (version),
                INDEX idx_status (status)
            )", self.migrations_table
        );

        let _versions_sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                version VARCHAR(50) PRIMARY KEY,
                description TEXT,
                release_date DATETIME NOT NULL,
                is_stable BOOLEAN NOT NULL DEFAULT FALSE,
                breaking_changes JSON,
                new_features JSON,
                bug_fixes JSON,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )", self.version_table
        );

        // Simulate table creation
        Ok(())
    }

    /// Get current database version
    pub fn get_current_version(&self) -> Result<Version, VersioningError> {
        // Query database for current version
        // In real implementation: SELECT MAX(version) FROM migrations WHERE status = 'Completed'
        
        // Return default version if no migrations exist
        Ok(Version::new(1, 0, 0))
    }

    /// Get version history
    pub fn get_version_history(&self) -> Result<VersionHistory, VersioningError> {
        let current = self.get_current_version()?;
        let pending = self.get_pending_migrations()?;
        
        Ok(VersionHistory {
            current_version: current,
            target_version: None,
            installed_versions: vec![],
            pending_migrations: pending,
            last_migration_at: None,
        })
    }

    /// Get pending migrations
    pub fn get_pending_migrations(&self) -> Result<Vec<DatabaseMigration>, VersioningError> {
        // Query database for pending migrations
        // SELECT * FROM migrations WHERE status = 'Pending' ORDER BY version, id
        Ok(vec![])
    }

    /// Get applied migrations
    pub fn get_applied_migrations(&self) -> Result<Vec<DatabaseMigration>, VersioningError> {
        // Query database for applied migrations
        // SELECT * FROM migrations WHERE status = 'Completed' ORDER BY version, id
        Ok(vec![])
    }

    /// Create new migration
    pub fn create_migration(
        &self,
        version: Version,
        name: String,
        description: String,
        migration_type: MigrationType,
        up_sql: String,
        down_sql: String,
    ) -> Result<DatabaseMigration, VersioningError> {
        // Validate migration
        if name.trim().is_empty() {
            return Err(VersioningError::Migration("Migration name cannot be empty".to_string()));
        }

        if up_sql.trim().is_empty() {
            return Err(VersioningError::Migration("Up SQL cannot be empty".to_string()));
        }

        // Calculate checksum
        let checksum = self.calculate_checksum(&up_sql, &down_sql);

        let migration = DatabaseMigration {
            id: self.generate_id(),
            version,
            name,
            description,
            migration_type,
            status: MigrationStatus::Pending,
            up_sql,
            down_sql,
            checksum,
            applied_at: None,
            rolled_back_at: None,
            execution_time_ms: None,
            error_message: None,
            created_at: Utc::now(),
        };

        // Save migration to database
        self.save_migration(&migration)?;

        Ok(migration)
    }

    /// Apply pending migrations
    pub fn migrate(&mut self) -> Result<Vec<DatabaseMigration>, VersioningError> {
        let pending_migrations = self.get_pending_migrations()?;
        let mut applied = vec![];

        for migration in pending_migrations {
            match self.apply_migration(&migration) {
                Ok(applied_migration) => {
                    applied.push(applied_migration);
                }
                Err(e) => {
                    // Stop on first failure
                    return Err(e);
                }
            }
        }

        // Update current version
        if let Some(last_migration) = applied.last() {
            self.current_version = Some(last_migration.version.clone());
        }

        Ok(applied)
    }

    /// Apply specific migration
    fn apply_migration(&self, migration: &DatabaseMigration) -> Result<DatabaseMigration, VersioningError> {
        let start_time = std::time::Instant::now();
        
        // Update status to running
        let mut updated_migration = migration.clone();
        updated_migration.status = MigrationStatus::Running;
        self.save_migration(&updated_migration)?;

        // Execute up SQL
        match self.execute_sql(&migration.up_sql) {
            Ok(_) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                
                // Update status to completed
                updated_migration.status = MigrationStatus::Completed;
                updated_migration.applied_at = Some(Utc::now());
                updated_migration.execution_time_ms = Some(execution_time);
                
                self.save_migration(&updated_migration)?;
                Ok(updated_migration)
            }
            Err(e) => {
                // Update status to failed
                updated_migration.status = MigrationStatus::Failed;
                updated_migration.error_message = Some(e.to_string());
                
                self.save_migration(&updated_migration)?;
                Err(VersioningError::Migration(format!("Migration {} failed: {}", migration.name, e)))
            }
        }
    }

    /// Rollback to specific version
    pub fn rollback_to_version(&mut self, target_version: Version) -> Result<Vec<DatabaseMigration>, VersioningError> {
        let current = self.get_current_version()?;
        
        if target_version >= current {
            return Err(VersioningError::Rollback("Target version must be lower than current version".to_string()));
        }

        // Get migrations to rollback (in reverse order)
        let migrations_to_rollback = self.get_migrations_between_versions(&target_version, &current)?;
        let mut rolled_back = vec![];

        for migration in migrations_to_rollback.into_iter().rev() {
            match self.rollback_migration(&migration) {
                Ok(rolled_back_migration) => {
                    rolled_back.push(rolled_back_migration);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        // Update current version
        self.current_version = Some(target_version);

        Ok(rolled_back)
    }

    /// Rollback specific migration
    fn rollback_migration(&self, migration: &DatabaseMigration) -> Result<DatabaseMigration, VersioningError> {
        if migration.down_sql.trim().is_empty() {
            return Err(VersioningError::Rollback(format!("Migration {} has no down SQL", migration.name)));
        }

        let start_time = std::time::Instant::now();
        
        // Execute down SQL
        match self.execute_sql(&migration.down_sql) {
            Ok(_) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                
                // Update migration status
                let mut updated_migration = migration.clone();
                updated_migration.status = MigrationStatus::RolledBack;
                updated_migration.rolled_back_at = Some(Utc::now());
                updated_migration.execution_time_ms = Some(execution_time);
                
                self.save_migration(&updated_migration)?;
                Ok(updated_migration)
            }
            Err(e) => {
                Err(VersioningError::Rollback(format!("Rollback of migration {} failed: {}", migration.name, e)))
            }
        }
    }

    /// Get migrations between two versions
    fn get_migrations_between_versions(
        &self,
        from_version: &Version,
        to_version: &Version,
    ) -> Result<Vec<DatabaseMigration>, VersioningError> {
        // Query database for migrations between versions
        // SELECT * FROM migrations WHERE version > ? AND version <= ? AND status = 'Completed' ORDER BY version, id
        Ok(vec![])
    }

    /// Verify migration integrity
    pub fn verify_migrations(&self) -> Result<bool, VersioningError> {
        let applied_migrations = self.get_applied_migrations()?;
        
        for migration in applied_migrations {
            // Recalculate checksum
            let current_checksum = self.calculate_checksum(&migration.up_sql, &migration.down_sql);
            
            if current_checksum != migration.checksum {
                return Err(VersioningError::Migration(
                    format!("Migration {} has been modified after application", migration.name)
                ));
            }
        }

        Ok(true)
    }

    /// Get schema version info
    pub fn get_schema_version(&self, version: &Version) -> Result<SchemaVersion, VersioningError> {
        // Query database for schema version
        Ok(SchemaVersion {
            version: version.clone(),
            description: "Schema version".to_string(),
            migrations: vec![],
            release_date: Utc::now(),
            is_stable: true,
            breaking_changes: vec![],
            new_features: vec![],
            bug_fixes: vec![],
        })
    }

    /// Create schema version
    pub fn create_schema_version(
        &self,
        version: Version,
        description: String,
        is_stable: bool,
        breaking_changes: Vec<String>,
        new_features: Vec<String>,
        bug_fixes: Vec<String>,
    ) -> Result<SchemaVersion, VersioningError> {
        let schema_version = SchemaVersion {
            version,
            description,
            migrations: vec![],
            release_date: Utc::now(),
            is_stable,
            breaking_changes,
            new_features,
            bug_fixes,
        };

        // Save to database
        Ok(schema_version)
    }

    /// Execute SQL (mock implementation)
    fn execute_sql(&self, sql: &str) -> Result<(), VersioningError> {
        // In real implementation, execute SQL against database
        if sql.trim().is_empty() {
            return Err(VersioningError::Migration("Empty SQL statement".to_string()));
        }

        // Simulate execution
        Ok(())
    }

    /// Save migration to database
    fn save_migration(&self, migration: &DatabaseMigration) -> Result<(), VersioningError> {
        // In real implementation, save to database
        Ok(())
    }

    /// Calculate checksum for migration
    fn calculate_checksum(&self, up_sql: &str, down_sql: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        up_sql.hash(&mut hasher);
        down_sql.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Generate unique ID
    fn generate_id(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow::anyhow!("Error: {}", e))?
            .as_millis() as u64
    }

    /// Get database backup before migration
    pub fn create_backup(&self, version: &Version) -> Result<String, VersioningError> {
        // In real implementation, create database backup
        let backup_name = format!("backup_{}_{}", version.to_string(), Utc::now().format("%Y%m%d_%H%M%S"));
        Ok(backup_name)
    }

    /// Restore from backup
    pub fn restore_backup(&self, backup_name: &str) -> Result<(), VersioningError> {
        // In real implementation, restore database from backup
        Ok(())
    }
}

impl Default for Versioning {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_creation() {
        let version = Version::new(1, 2, 3);
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.build, None);
    }

    #[test]
    fn test_version_from_string() {
        let version = Version::from_string("1.2.3").map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);

        let version_with_build = Version::from_string("1.2.3.4").map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert_eq!(version_with_build.build, Some(4));
    }

    #[test]
    fn test_version_to_string() {
        let version = Version::new(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");

        let version_with_build = Version::with_build(1, 2, 3, 4);
        assert_eq!(version_with_build.to_string(), "1.2.3.4");
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(1, 1, 0);
        let v3 = Version::new(2, 0, 0);

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1.is_compatible_with(&v2));
        assert!(!v1.is_compatible_with(&v3));
    }

    #[test]
    fn test_version_increment() {
        let mut version = Version::new(1, 2, 3);
        
        version.increment_patch();
        assert_eq!(version.to_string(), "1.2.4");
        
        version.increment_minor();
        assert_eq!(version.to_string(), "1.3.0");
        
        version.increment_major();
        assert_eq!(version.to_string(), "2.0.0");
    }

    #[test]
    fn test_versioning_creation() {
        let versioning = Versioning::new();
        assert_eq!(versioning.migrations_table, "schema_migrations");
        assert_eq!(versioning.version_table, "schema_versions");
    }

    #[test]
    fn test_create_migration() {
        let versioning = Versioning::new();
        let version = Version::new(1, 1, 0);
        
        let result = versioning.create_migration(
            version,
            "add_user_table".to_string(),
            "Add user table".to_string(),
            MigrationType::Schema,
            "CREATE TABLE users (id INT PRIMARY KEY)".to_string(),
            "DROP TABLE users".to_string(),
        );
        
        assert!(result.is_ok());
        let migration = result.map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert_eq!(migration.name, "add_user_table");
        assert!(matches!(migration.status, MigrationStatus::Pending));
    }

    #[test]
    fn test_invalid_version_string() {
        let result = Version::from_string("invalid");
        assert!(result.is_err());

        let result = Version::from_string("1.2");
        assert!(result.is_err());

        let result = Version::from_string("1.2.3.4.5");
        assert!(result.is_err());
    }
}