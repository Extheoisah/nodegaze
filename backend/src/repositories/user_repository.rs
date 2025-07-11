//! Database repository for user management operations.
//!
//! Provides CRUD operations for system users

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::models::{CreateUser, User};

/// Repository for user database operations.
///
/// Handles all persistence operations for the User entity,
/// maintaining relationships with accounts and roles.
pub struct UserRepository<'a> {
    /// Shared SQLite connection pool
    pool: &'a SqlitePool,
}

impl<'a> UserRepository<'a> {
    /// Creates a new UserRepository instance.
    ///
    /// # Arguments
    /// * `pool` - Reference to SQLite connection pool
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Creates a new user in the database.
    ///
    /// # Arguments
    /// * `user` - CreateUser DTO containing user details
    ///
    /// # Returns
    /// The newly created User with all fields populated
    pub async fn create_user(&self, user: CreateUser) -> Result<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (account_id, role_id, name, email, is_active)
            VALUES (?, ?, ?, ?, ?)
            RETURNING 
            id as "id!",
            account_id as "account_id!",
            role_id as "role_id!",
            name as "name!",
            email as "email!",
            is_active as "is_active!",
            created_at as "created_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            "#,
            user.account_id,
            user.role_id,
            user.name,
            user.email,
            true
        )
        .fetch_one(self.pool)
        .await?;

        Ok(user)
    }

    /// Retrieves a user by their unique identifier.
    ///
    /// # Arguments
    /// * `id` - User ID (UUID format)
    ///
    /// # Returns
    /// `Some(User)` if found and active, `None` otherwise
    pub async fn get_user_by_id(&self, id: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
            id as "id!",
            account_id as "account_id!",
            role_id as "role_id!",
            name as "name!",
            email as "email!",
            is_active as "is_active!",
            created_at as "created_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            FROM users WHERE id = ? AND is_deleted = 0
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(user)
    }

    /// Retrieves the admin user for a specific account.
    ///
    /// # Arguments
    /// * `account_id` - Account ID (UUID format)
    ///
    /// # Returns
    /// `Some(User)` if admin user exists for account, `None` otherwise
    pub async fn get_admin_user_by_account_id(&self, account_id: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
            u.id as "id!",
            u.account_id as "account_id!",
            u.role_id as "role_id!",
            u.name as "name!",
            u.email as "email!",
            u.is_active as "is_active!",
            u.created_at as "created_at!: DateTime<Utc>",
            u.updated_at as "updated_at!: DateTime<Utc>",
            u.is_deleted as "is_deleted!",
            u.deleted_at as "deleted_at?: DateTime<Utc>"
            FROM users  u
            LEFT JOIN roles r ON u.role_id = r.id
            WHERE u.account_id = ? AND u.is_deleted = 0 AND r.name = 'Admin'
            "#,
            account_id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(user)
    }
}
