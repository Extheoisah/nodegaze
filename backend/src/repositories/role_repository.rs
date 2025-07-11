//! Database repository for role management operations.
//!
//! Provides read-only access to system roles with:
//! - Role lookup by ID or name
//! - Complete role listing
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::models::Role;

/// Repository for role database operations.
///
/// Handles all read operations for the Role entity,
/// enforcing data consistency and access patterns.
pub struct RoleRepository<'a> {
    /// Shared SQLite connection pool
    pool: &'a SqlitePool,
}

impl<'a> RoleRepository<'a> {
    /// Creates a new RoleRepository instance.
    ///
    /// # Arguments
    /// * `pool` - Reference to SQLite connection pool
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Retrieves a role by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - Role ID (UUID format)
    ///
    /// # Returns
    /// `Some(Role)` if found and active, `None` otherwise
    pub async fn get_role_by_id(&self, id: &str) -> Result<Option<Role>> {
        let role = sqlx::query_as!(
            Role,
            r#"
            SELECT 
            id as "id!",
            name as "name!",
            is_active as "is_active!",
            created_at as "created_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            FROM roles WHERE id = ? AND is_deleted = 0
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(role)
    }

    /// Retrieves a role by its exact name.
    ///
    /// # Arguments
    /// * `name` - Exact role name to search for
    ///
    /// # Returns
    /// `Some(Role)` if found and active, `None` otherwise
    ///
    /// # Use Case
    /// Useful for permission checks and role-based access control
    pub async fn get_role_by_name(&self, name: &str) -> Result<Option<Role>> {
        let role = sqlx::query_as!(
            Role,
            r#"
            SELECT 
            id as "id!",
            name as "name!",
            is_active as "is_active!",
            created_at as "created_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            FROM roles WHERE name = ? AND is_deleted = 0
            "#,
            name
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(role)
    }

    /// Retrieves all active roles in the system.
    ///
    /// # Returns
    /// Vector of all non-deleted roles
    pub async fn get_all_roles(&self) -> Result<Vec<Role>> {
        let roles = sqlx::query_as!(
            Role,
            r#"
            SELECT 
            id as "id!",
            name as "name!",
            is_active as "is_active!",
            created_at as "created_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            FROM roles WHERE is_deleted = 0
            "#
        )
        .fetch_all(self.pool)
        .await?;

        Ok(roles)
    }
}
