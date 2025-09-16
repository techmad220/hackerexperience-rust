//! Role-Based Access Control (RBAC) implementation

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use sqlx::{PgPool, postgres::PgPoolOptions};

/// Permission represents a specific action that can be performed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Permission {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub resource: String,
    pub action: String,
    pub scope: PermissionScope,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Permission {
    pub fn new(name: &str, description: &str) -> Self {
        let parts: Vec<&str> = name.split(':').collect();
        let (resource, action) = if parts.len() >= 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            (name.to_string(), "*".to_string())
        };

        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: description.to_string(),
            resource,
            action,
            scope: PermissionScope::Default,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn with_scope(mut self, scope: PermissionScope) -> Self {
        self.scope = scope;
        self
    }

    /// Check if this permission grants access to the requested permission
    pub fn grants(&self, requested: &str) -> bool {
        // Wildcard permission grants everything
        if self.name == "*" || self.name == "*:*" {
            return true;
        }

        // Resource wildcard
        if self.name.ends_with(":*") {
            let resource_prefix = &self.name[..self.name.len() - 2];
            return requested.starts_with(resource_prefix);
        }

        // Exact match
        self.name == requested
    }
}

/// Permission scope defines the extent of the permission
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PermissionScope {
    Global,         // System-wide permission
    Organization,   // Organization-level permission
    Team,          // Team-level permission
    User,          // User-level permission (own resources)
    Default,       // Default scope
}

/// Role represents a collection of permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub permissions: HashSet<Permission>,
    pub parent_roles: Vec<Uuid>,  // Role inheritance
    pub is_system: bool,           // System roles cannot be deleted
    pub priority: i32,             // Higher priority roles override lower ones
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Role {
    pub fn new(name: &str, display_name: &str, permissions: Vec<Permission>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            display_name: display_name.to_string(),
            description: String::new(),
            permissions: permissions.into_iter().collect(),
            parent_roles: Vec::new(),
            is_system: false,
            priority: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn with_parent_roles(mut self, parents: Vec<Uuid>) -> Self {
        self.parent_roles = parents;
        self
    }

    pub fn system_role(mut self) -> Self {
        self.is_system = true;
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Add a permission to the role
    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission);
        self.updated_at = Utc::now();
    }

    /// Remove a permission from the role
    pub fn remove_permission(&mut self, permission_name: &str) -> bool {
        let removed = self.permissions.retain(|p| p.name != permission_name);
        if removed {
            self.updated_at = Utc::now();
        }
        removed
    }

    /// Check if role has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p.grants(permission))
    }
}

/// User role assignment with temporal constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRoleAssignment {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub assigned_by: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub context: RoleContext,
    pub is_active: bool,
}

/// Context for role assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleContext {
    pub organization_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub metadata: HashMap<String, String>,
}

impl Default for RoleContext {
    fn default() -> Self {
        Self {
            organization_id: None,
            team_id: None,
            project_id: None,
            metadata: HashMap::new(),
        }
    }
}

/// Access control decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessDecision {
    pub granted: bool,
    pub reason: String,
    pub applied_rules: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// Role manager handles all RBAC operations
pub struct RoleManager {
    db_pool: Option<PgPool>,
    roles: Arc<RwLock<HashMap<Uuid, Role>>>,
    role_names: Arc<RwLock<HashMap<String, Uuid>>>,
    user_roles: Arc<RwLock<HashMap<Uuid, Vec<UserRoleAssignment>>>>,
    permission_cache: Arc<RwLock<HashMap<String, HashSet<Permission>>>>,
    access_log: Arc<RwLock<Vec<AccessLog>>>,
}

/// Access log entry for audit
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AccessLog {
    user_id: Uuid,
    resource: String,
    action: String,
    granted: bool,
    reason: String,
    timestamp: DateTime<Utc>,
}

impl RoleManager {
    /// Create a new role manager
    pub async fn new() -> Result<Self> {
        Ok(Self {
            db_pool: None,
            roles: Arc::new(RwLock::new(HashMap::new())),
            role_names: Arc::new(RwLock::new(HashMap::new())),
            user_roles: Arc::new(RwLock::new(HashMap::new())),
            permission_cache: Arc::new(RwLock::new(HashMap::new())),
            access_log: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Create with database connection
    pub async fn with_database(database_url: &str) -> Result<Self> {
        let db_pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        let mut manager = Self::new().await?;
        manager.db_pool = Some(db_pool);
        manager.initialize_database().await?;
        manager.load_from_database().await?;

        Ok(manager)
    }

    /// Initialize database tables
    async fn initialize_database(&self) -> Result<()> {
        if let Some(pool) = &self.db_pool {
            sqlx::query(
                r#"
                CREATE TABLE IF NOT EXISTS permissions (
                    id UUID PRIMARY KEY,
                    name VARCHAR(255) UNIQUE NOT NULL,
                    description TEXT,
                    resource VARCHAR(255) NOT NULL,
                    action VARCHAR(255) NOT NULL,
                    scope VARCHAR(50) NOT NULL,
                    created_at TIMESTAMPTZ NOT NULL,
                    updated_at TIMESTAMPTZ NOT NULL
                );

                CREATE TABLE IF NOT EXISTS roles (
                    id UUID PRIMARY KEY,
                    name VARCHAR(255) UNIQUE NOT NULL,
                    display_name VARCHAR(255) NOT NULL,
                    description TEXT,
                    is_system BOOLEAN DEFAULT FALSE,
                    priority INTEGER DEFAULT 0,
                    created_at TIMESTAMPTZ NOT NULL,
                    updated_at TIMESTAMPTZ NOT NULL
                );

                CREATE TABLE IF NOT EXISTS role_permissions (
                    role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
                    permission_id UUID REFERENCES permissions(id) ON DELETE CASCADE,
                    PRIMARY KEY (role_id, permission_id)
                );

                CREATE TABLE IF NOT EXISTS role_hierarchy (
                    child_role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
                    parent_role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
                    PRIMARY KEY (child_role_id, parent_role_id)
                );

                CREATE TABLE IF NOT EXISTS user_roles (
                    user_id UUID NOT NULL,
                    role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
                    assigned_by UUID NOT NULL,
                    assigned_at TIMESTAMPTZ NOT NULL,
                    expires_at TIMESTAMPTZ,
                    context JSONB,
                    is_active BOOLEAN DEFAULT TRUE,
                    PRIMARY KEY (user_id, role_id)
                );

                CREATE INDEX IF NOT EXISTS idx_user_roles_user ON user_roles(user_id);
                CREATE INDEX IF NOT EXISTS idx_user_roles_active ON user_roles(is_active) WHERE is_active = TRUE;
                CREATE INDEX IF NOT EXISTS idx_permissions_resource ON permissions(resource);
                CREATE INDEX IF NOT EXISTS idx_permissions_action ON permissions(action);
                "#
            )
            .execute(pool)
            .await?;

            info!("RBAC database tables initialized");
        }
        Ok(())
    }

    /// Load roles and permissions from database
    async fn load_from_database(&self) -> Result<()> {
        if let Some(pool) = &self.db_pool {
            // Load permissions
            let permissions = sqlx::query(
                r#"
                SELECT id, name, description, resource, action, scope, created_at, updated_at
                FROM permissions
                "#
            )
            .fetch_all(pool)
            .await?;

            // Load roles
            let db_roles = sqlx::query(
                r#"
                SELECT id, name, display_name, description, is_system, priority, created_at, updated_at
                FROM roles
                "#
            )
            .fetch_all(pool)
            .await?;

            let mut roles = self.roles.write().await;
            let mut role_names = self.role_names.write().await;

            for db_role in db_roles {
                let role_permissions = sqlx::query(
                    r#"
                    SELECT p.*
                    FROM permissions p
                    JOIN role_permissions rp ON p.id = rp.permission_id
                    WHERE rp.role_id = $1
                    "#
                )
                .bind(db_role.id)
                .fetch_all(pool)
                .await?;

                let permissions_set: HashSet<Permission> = role_permissions
                    .into_iter()
                    .map(|p| Permission {
                        id: p.id,
                        name: p.name,
                        description: p.description.unwrap_or_default(),
                        resource: p.resource,
                        action: p.action,
                        scope: PermissionScope::Default,
                        created_at: p.created_at,
                        updated_at: p.updated_at,
                    })
                    .collect();

                let role = Role {
                    id: db_role.id,
                    name: db_role.name.clone(),
                    display_name: db_role.display_name,
                    description: db_role.description.unwrap_or_default(),
                    permissions: permissions_set,
                    parent_roles: Vec::new(),
                    is_system: db_role.is_system,
                    priority: db_role.priority,
                    created_at: db_role.created_at,
                    updated_at: db_role.updated_at,
                };

                role_names.insert(db_role.name, db_role.id);
                roles.insert(db_role.id, role);
            }

            // Load user role assignments
            let assignments = sqlx::query(
                r#"
                SELECT user_id, role_id, assigned_by, assigned_at, expires_at, context, is_active
                FROM user_roles
                WHERE is_active = TRUE
                "#
            )
            .fetch_all(pool)
            .await?;

            let mut user_roles = self.user_roles.write().await;
            for assignment in assignments {
                let user_assignment = UserRoleAssignment {
                    user_id: assignment.user_id,
                    role_id: assignment.role_id,
                    assigned_by: assignment.assigned_by,
                    assigned_at: assignment.assigned_at,
                    expires_at: assignment.expires_at,
                    context: serde_json::from_value(assignment.context.unwrap_or_default())
                        .unwrap_or_default(),
                    is_active: assignment.is_active,
                };

                user_roles
                    .entry(assignment.user_id)
                    .or_insert_with(Vec::new)
                    .push(user_assignment);
            }

            info!("Loaded {} roles from database", roles.len());
        }
        Ok(())
    }

    /// Create a new role
    pub async fn create_role(&self, mut role: Role) -> Result<Uuid> {
        // Check if role name already exists
        {
            let role_names = self.role_names.read().await;
            if role_names.contains_key(&role.name) {
                return Err(anyhow!("Role with name '{}' already exists", role.name));
            }
        }

        let role_id = role.id;

        // Save to database if available
        if let Some(pool) = &self.db_pool {
            sqlx::query(
                r#"
                INSERT INTO roles (id, name, display_name, description, is_system, priority, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#
            )
            .bind(role.id)
            .bind(role.name.clone())
            .bind(role.display_name.clone())
            .bind(role.description.clone())
            .bind(role.is_system)
            .bind(role.priority)
            .bind(role.created_at)
            .bind(role.updated_at)
            .execute(pool)
            .await?;

            // Save permissions
            for permission in &role.permissions {
                // Insert permission if not exists
                sqlx::query(
                    r#"
                    INSERT INTO permissions (id, name, description, resource, action, scope, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                    ON CONFLICT (name) DO NOTHING
                    "#
                )
                .bind(permission.id)
                .bind(permission.name.clone())
                .bind(permission.description.clone())
                .bind(permission.resource.clone())
                .bind(permission.action.clone())
                .bind("Default")
                .bind(permission.created_at)
                .bind(permission.updated_at)
                .execute(pool)
                .await?;

                // Link permission to role
                sqlx::query(
                    r#"
                    INSERT INTO role_permissions (role_id, permission_id)
                    VALUES ($1, $2)
                    "#
                )
                .bind(role.id)
                .bind(permission.id)
                .execute(pool)
                .await?;
            }
        }

        // Store in memory
        let mut roles = self.roles.write().await;
        let mut role_names = self.role_names.write().await;

        role_names.insert(role.name.clone(), role_id);
        roles.insert(role_id, role);

        info!("Created role: {}", role_id);
        Ok(role_id)
    }

    /// Get role by ID
    pub async fn get_role(&self, role_id: &Uuid) -> Option<Role> {
        let roles = self.roles.read().await;
        roles.get(role_id).cloned()
    }

    /// Get role by name
    pub async fn get_role_by_name(&self, name: &str) -> Option<Role> {
        let role_names = self.role_names.read().await;
        if let Some(role_id) = role_names.get(name) {
            let roles = self.roles.read().await;
            roles.get(role_id).cloned()
        } else {
            None
        }
    }

    /// Update role
    pub async fn update_role(&self, role_id: &Uuid, updates: RoleUpdate) -> Result<()> {
        let mut roles = self.roles.write().await;
        let role = roles.get_mut(role_id)
            .ok_or_else(|| anyhow!("Role not found"))?;

        if role.is_system {
            return Err(anyhow!("Cannot modify system role"));
        }

        if let Some(display_name) = updates.display_name {
            role.display_name = display_name;
        }
        if let Some(description) = updates.description {
            role.description = description;
        }
        if let Some(permissions) = updates.permissions {
            role.permissions = permissions.into_iter().collect();
        }

        role.updated_at = Utc::now();

        // Update in database
        if let Some(pool) = &self.db_pool {
            sqlx::query(
                r#"
                UPDATE roles
                SET display_name = $2, description = $3, updated_at = $4
                WHERE id = $1
                "#
            )
            .bind(role_id)
            .bind(role.display_name.clone())
            .bind(role.description.clone())
            .bind(role.updated_at)
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    /// Delete role
    pub async fn delete_role(&self, role_id: &Uuid) -> Result<()> {
        let role = {
            let roles = self.roles.read().await;
            roles.get(role_id).cloned()
        };

        if let Some(role) = role {
            if role.is_system {
                return Err(anyhow!("Cannot delete system role"));
            }

            // Remove from database
            if let Some(pool) = &self.db_pool {
                sqlx::query("DELETE FROM roles WHERE id = $1")
                    .bind(role_id)
                    .execute(pool)
                    .await?;
            }

            // Remove from memory
            let mut roles = self.roles.write().await;
            let mut role_names = self.role_names.write().await;

            role_names.remove(&role.name);
            roles.remove(role_id);

            info!("Deleted role: {}", role_id);
            Ok(())
        } else {
            Err(anyhow!("Role not found"))
        }
    }

    /// Assign role to user
    pub async fn assign_role_to_user(
        &self,
        user_id: &Uuid,
        role_id: &Uuid,
        assigned_by: &Uuid,
        expires_at: Option<DateTime<Utc>>,
        context: RoleContext,
    ) -> Result<()> {
        // Check if role exists
        {
            let roles = self.roles.read().await;
            if !roles.contains_key(role_id) {
                return Err(anyhow!("Role not found"));
            }
        }

        let assignment = UserRoleAssignment {
            user_id: *user_id,
            role_id: *role_id,
            assigned_by: *assigned_by,
            assigned_at: Utc::now(),
            expires_at,
            context,
            is_active: true,
        };

        // Save to database
        if let Some(pool) = &self.db_pool {
            sqlx::query(
                r#"
                INSERT INTO user_roles (user_id, role_id, assigned_by, assigned_at, expires_at, context, is_active)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (user_id, role_id)
                DO UPDATE SET
                    assigned_by = $3,
                    assigned_at = $4,
                    expires_at = $5,
                    context = $6,
                    is_active = $7
                "#
            )
            .bind(user_id)
            .bind(role_id)
            .bind(assigned_by)
            .bind(assignment.assigned_at)
            .bind(assignment.expires_at)
            .bind(serde_json::to_value(&assignment.context)?)
            .bind(true)
            .execute(pool)
            .await?;
        }

        // Store in memory
        let mut user_roles = self.user_roles.write().await;
        user_roles
            .entry(*user_id)
            .or_insert_with(Vec::new)
            .push(assignment);

        // Clear permission cache for user
        let mut cache = self.permission_cache.write().await;
        cache.remove(&user_id.to_string());

        info!("Assigned role {} to user {}", role_id, user_id);
        Ok(())
    }

    /// Revoke role from user
    pub async fn revoke_role_from_user(&self, user_id: &Uuid, role_id: &Uuid) -> Result<()> {
        // Update in database
        if let Some(pool) = &self.db_pool {
            sqlx::query(
                r#"
                UPDATE user_roles
                SET is_active = FALSE
                WHERE user_id = $1 AND role_id = $2
                "#
            )
            .bind(user_id)
            .bind(role_id)
            .execute(pool)
            .await?;
        }

        // Remove from memory
        let mut user_roles = self.user_roles.write().await;
        if let Some(assignments) = user_roles.get_mut(user_id) {
            assignments.retain(|a| a.role_id != *role_id);
        }

        // Clear permission cache
        let mut cache = self.permission_cache.write().await;
        cache.remove(&user_id.to_string());

        info!("Revoked role {} from user {}", role_id, user_id);
        Ok(())
    }

    /// Get all roles for a user
    pub async fn get_user_roles(&self, user_id: &Uuid) -> Vec<Role> {
        let user_roles = self.user_roles.read().await;
        let roles = self.roles.read().await;

        if let Some(assignments) = user_roles.get(user_id) {
            assignments
                .iter()
                .filter(|a| {
                    a.is_active &&
                    a.expires_at.map_or(true, |exp| exp > Utc::now())
                })
                .filter_map(|a| roles.get(&a.role_id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all permissions for a user (with caching)
    pub async fn get_user_permissions(&self, user_id: &Uuid) -> HashSet<Permission> {
        // Check cache first
        {
            let cache = self.permission_cache.read().await;
            if let Some(cached) = cache.get(&user_id.to_string()) {
                return cached.clone();
            }
        }

        // Collect all permissions from user's roles
        let mut all_permissions = HashSet::new();
        let user_roles = self.get_user_roles(user_id).await;

        for role in user_roles {
            // Add role's direct permissions
            all_permissions.extend(role.permissions.clone());

            // Add inherited permissions from parent roles
            for parent_id in &role.parent_roles {
                if let Some(parent_role) = self.get_role(parent_id).await {
                    all_permissions.extend(parent_role.permissions);
                }
            }
        }

        // Cache the result
        {
            let mut cache = self.permission_cache.write().await;
            cache.insert(user_id.to_string(), all_permissions.clone());
        }

        all_permissions
    }

    /// Check if user has specific permission
    pub async fn user_has_permission(&self, user_id: &Uuid, permission: &str) -> bool {
        let permissions = self.get_user_permissions(user_id).await;
        permissions.iter().any(|p| p.grants(permission))
    }

    /// Check if role has specific permission
    pub async fn role_has_permission(&self, role_name: &str, permission: &str) -> Result<bool> {
        if let Some(role) = self.get_role_by_name(role_name).await {
            Ok(role.has_permission(permission))
        } else {
            Ok(false)
        }
    }

    /// Evaluate access control
    pub async fn evaluate_access(
        &self,
        user_id: &Uuid,
        resource: &str,
        action: &str,
    ) -> AccessDecision {
        let permission_string = format!("{}:{}", resource, action);
        let has_access = self.user_has_permission(user_id, &permission_string).await;

        let decision = AccessDecision {
            granted: has_access,
            reason: if has_access {
                format!("User has permission for {}:{}", resource, action)
            } else {
                format!("User lacks permission for {}:{}", resource, action)
            },
            applied_rules: vec![permission_string.clone()],
            timestamp: Utc::now(),
        };

        // Log access attempt
        {
            let mut log = self.access_log.write().await;
            log.push(AccessLog {
                user_id: *user_id,
                resource: resource.to_string(),
                action: action.to_string(),
                granted: decision.granted,
                reason: decision.reason.clone(),
                timestamp: decision.timestamp,
            });

            // Keep only last 10000 entries
            if log.len() > 10000 {
                log.drain(0..5000);
            }
        }

        decision
    }

    /// Get access log for audit
    pub async fn get_access_log(&self, user_id: Option<&Uuid>, limit: usize) -> Vec<AccessLog> {
        let log = self.access_log.read().await;

        let filtered: Vec<AccessLog> = if let Some(uid) = user_id {
            log.iter()
                .filter(|entry| entry.user_id == *uid)
                .cloned()
                .collect()
        } else {
            log.clone()
        };

        filtered.into_iter().rev().take(limit).collect()
    }

    /// Clean up expired role assignments
    pub async fn cleanup_expired_assignments(&self) -> Result<usize> {
        let now = Utc::now();
        let mut expired_count = 0;

        let mut user_roles = self.user_roles.write().await;
        for assignments in user_roles.values_mut() {
            let before_count = assignments.len();
            assignments.retain(|a| {
                a.expires_at.map_or(true, |exp| exp > now)
            });
            expired_count += before_count - assignments.len();
        }

        if expired_count > 0 && self.db_pool.is_some() {
            let pool = self.db_pool.as_ref().unwrap();
            sqlx::query(
                r#"
                UPDATE user_roles
                SET is_active = FALSE
                WHERE expires_at < $1 AND is_active = TRUE
                "#
            )
            .bind(now)
            .execute(pool)
            .await?;
        }

        info!("Cleaned up {} expired role assignments", expired_count);
        Ok(expired_count)
    }

    /// Get role statistics
    pub async fn get_statistics(&self) -> RbacStatistics {
        let roles = self.roles.read().await;
        let user_roles = self.user_roles.read().await;
        let access_log = self.access_log.read().await;

        let total_permissions: usize = roles
            .values()
            .map(|r| r.permissions.len())
            .sum();

        let system_roles = roles
            .values()
            .filter(|r| r.is_system)
            .count();

        RbacStatistics {
            total_roles: roles.len(),
            system_roles,
            custom_roles: roles.len() - system_roles,
            total_permissions,
            total_users_with_roles: user_roles.len(),
            total_assignments: user_roles.values().map(|v| v.len()).sum(),
            access_log_entries: access_log.len(),
        }
    }
}

/// Role update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleUpdate {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<Vec<Permission>>,
}

/// RBAC statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbacStatistics {
    pub total_roles: usize,
    pub system_roles: usize,
    pub custom_roles: usize,
    pub total_permissions: usize,
    pub total_users_with_roles: usize,
    pub total_assignments: usize,
    pub access_log_entries: usize,
}

/// Access control helper for common operations
pub struct AccessControl;

impl AccessControl {
    /// Check read permission
    pub async fn can_read(
        manager: &RoleManager,
        user_id: &Uuid,
        resource: &str,
    ) -> bool {
        manager.user_has_permission(user_id, &format!("{}:read", resource)).await
    }

    /// Check write permission
    pub async fn can_write(
        manager: &RoleManager,
        user_id: &Uuid,
        resource: &str,
    ) -> bool {
        manager.user_has_permission(user_id, &format!("{}:write", resource)).await
    }

    /// Check delete permission
    pub async fn can_delete(
        manager: &RoleManager,
        user_id: &Uuid,
        resource: &str,
    ) -> bool {
        manager.user_has_permission(user_id, &format!("{}:delete", resource)).await
    }

    /// Check admin permission
    pub async fn is_admin(manager: &RoleManager, user_id: &Uuid) -> bool {
        manager.user_has_permission(user_id, "*").await ||
        manager.user_has_permission(user_id, "admin:*").await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_permission_grants() {
        let perm = Permission::new("game:play", "Play the game");
        assert!(perm.grants("game:play"));
        assert!(!perm.grants("game:admin"));

        let wildcard = Permission::new("game:*", "All game permissions");
        assert!(wildcard.grants("game:play"));
        assert!(wildcard.grants("game:admin"));
        assert!(!wildcard.grants("admin:users"));

        let global = Permission::new("*", "Global admin");
        assert!(global.grants("game:play"));
        assert!(global.grants("admin:users"));
    }

    #[tokio::test]
    async fn test_role_creation() {
        let manager = RoleManager::new().await.unwrap();

        let permissions = vec![
            Permission::new("game:play", "Play game"),
            Permission::new("chat:send", "Send messages"),
        ];

        let role = Role::new("player", "Player", permissions);
        let role_id = manager.create_role(role).await.unwrap();

        let retrieved = manager.get_role(&role_id).await.unwrap();
        assert_eq!(retrieved.name, "player");
        assert_eq!(retrieved.permissions.len(), 2);
    }

    #[tokio::test]
    async fn test_user_role_assignment() {
        let manager = RoleManager::new().await.unwrap();

        let role = Role::new("player", "Player", vec![
            Permission::new("game:play", "Play game"),
        ]);
        let role_id = manager.create_role(role).await.unwrap();

        let user_id = Uuid::new_v4();
        let assigned_by = Uuid::new_v4();

        manager.assign_role_to_user(
            &user_id,
            &role_id,
            &assigned_by,
            None,
            RoleContext::default(),
        ).await.unwrap();

        let user_roles = manager.get_user_roles(&user_id).await;
        assert_eq!(user_roles.len(), 1);
        assert_eq!(user_roles[0].name, "player");
    }

    #[tokio::test]
    async fn test_permission_checking() {
        let manager = RoleManager::new().await.unwrap();

        let role = Role::new("moderator", "Moderator", vec![
            Permission::new("chat:moderate", "Moderate chat"),
            Permission::new("player:warn", "Warn players"),
        ]);
        let role_id = manager.create_role(role).await.unwrap();

        let user_id = Uuid::new_v4();
        manager.assign_role_to_user(
            &user_id,
            &role_id,
            &user_id,
            None,
            RoleContext::default(),
        ).await.unwrap();

        assert!(manager.user_has_permission(&user_id, "chat:moderate").await);
        assert!(manager.user_has_permission(&user_id, "player:warn").await);
        assert!(!manager.user_has_permission(&user_id, "admin:users").await);
    }

    #[tokio::test]
    async fn test_role_revocation() {
        let manager = RoleManager::new().await.unwrap();

        let role = Role::new("test", "Test", vec![]);
        let role_id = manager.create_role(role).await.unwrap();

        let user_id = Uuid::new_v4();
        manager.assign_role_to_user(
            &user_id,
            &role_id,
            &user_id,
            None,
            RoleContext::default(),
        ).await.unwrap();

        assert_eq!(manager.get_user_roles(&user_id).await.len(), 1);

        manager.revoke_role_from_user(&user_id, &role_id).await.unwrap();
        assert_eq!(manager.get_user_roles(&user_id).await.len(), 0);
    }
}