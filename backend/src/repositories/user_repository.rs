//! Database repository for user management operations.
//!
//! Provides CRUD operations for system users

use crate::{
    api::common::PaginationFilter,
    database::models::{CreateUser, RoleAccessLevel, User},
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

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
            INSERT INTO users (id, account_id, role_id, role_access_level, username, password_hash, email, is_active)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING
            id as "id!",
            account_id as "account_id!",
            role_id as "role_id!",
            role_access_level as "role_access_level: RoleAccessLevel",
            username as "username!",
            password_hash as "password_hash!",
            email as "email!",
            is_active as "is_active!",
            created_at as "created_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            "#,
            user.id,
            user.account_id,
            user.role_id,
            user.role_access_level,
            user.username,
            user.password_hash,
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
            role_access_level as "role_access_level: RoleAccessLevel",
            username as "username!",
            password_hash as "password_hash!",
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

    /// Retrieves a user by their username.
    ///
    /// # Arguments
    /// * `username` - Username to search for
    ///
    /// # Returns
    /// `Some(User)` if found and active, `None` otherwise
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT
            id as "id!",
            account_id as "account_id!",
            role_id as "role_id!",
            role_access_level as "role_access_level: RoleAccessLevel",
            username as "username!",
            password_hash as "password_hash!",
            email as "email!",
            is_active as "is_active!",
            created_at as "created_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            FROM users WHERE username = ? AND is_deleted = 0
            "#,
            username
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(user)
    }

    /// Retrieves a user by their email.
    ///
    /// # Arguments
    /// * `email` - Email to search for
    ///
    /// # Returns
    /// `Some(User)` if found and active, `None` otherwise
    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT
            id as "id!",
            account_id as "account_id!",
            role_id as "role_id!",
            role_access_level as "role_access_level: RoleAccessLevel",
            username as "username!",
            password_hash as "password_hash!",
            email as "email!",
            is_active as "is_active!",
            created_at as "created_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            FROM users WHERE email = ? AND is_deleted = 0
            "#,
            email
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
            u.role_access_level as "role_access_level: RoleAccessLevel",
            u.username as "username!",
            u.password_hash as "password_hash!",
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

    /// Checks if a username already exists in the system.
    ///
    /// # Arguments
    /// * `username` - Username to check
    ///
    /// # Returns
    /// `true` if a user with this username exists (and is not deleted)
    pub async fn username_exists(&self, username: &str) -> Result<bool> {
        let count = sqlx::query!(
            "SELECT COUNT(*) as count FROM users WHERE username = ? AND is_deleted = 0",
            username
        )
        .fetch_one(self.pool)
        .await?;

        Ok(count.count > 0)
    }

    /// Checks if an email already exists in the system.
    ///
    /// # Arguments
    /// * `email` - Email to check
    ///
    /// # Returns
    /// `true` if a user with this email exists (and is not deleted)
    pub async fn email_exists(&self, email: &str) -> Result<bool> {
        let count = sqlx::query!(
            "SELECT COUNT(*) as count FROM users WHERE email = ? AND is_deleted = 0",
            email
        )
        .fetch_one(self.pool)
        .await?;

        Ok(count.count > 0)
    }

    /// Checks if username exists excluding a specific user.
    ///
    /// # Arguments
    /// * `username` - Username to check
    /// * `exclude_user_id` - User ID to exclude from check
    ///
    /// # Returns
    /// `true` if another user with this username exists
    pub async fn username_exists_excluding(
        &self,
        username: &str,
        exclude_user_id: &str,
    ) -> Result<bool> {
        let count = sqlx::query!(
            "SELECT COUNT(*) as count FROM users WHERE username = ? AND id != ? AND is_deleted = 0",
            username,
            exclude_user_id
        )
        .fetch_one(self.pool)
        .await?;

        Ok(count.count > 0)
    }

    /// Checks if email exists excluding a specific user.
    ///
    /// # Arguments
    /// * `email` - Email to check
    /// * `exclude_user_id` - User ID to exclude from check
    ///
    /// # Returns
    /// `true` if another user with this email exists
    pub async fn email_exists_excluding(&self, email: &str, exclude_user_id: &str) -> Result<bool> {
        let count = sqlx::query!(
            "SELECT COUNT(*) as count FROM users WHERE email = ? AND id != ? AND is_deleted = 0",
            email,
            exclude_user_id
        )
        .fetch_one(self.pool)
        .await?;

        Ok(count.count > 0)
    }

    /// Retrieves the users for a specific account.
    ///
    /// # Arguments
    /// * `account_id` - Account ID (UUID format)
    ///
    /// # Returns
    /// `Some(User)` if users exist for account, `None` otherwise
    pub async fn get_users_by_account_id(
        &self,
        account_id: &str,
        pagination: &PaginationFilter,
    ) -> Result<Vec<User>> {
       let limit = pagination.limit();
        let offset = pagination.offset();

         let users = sqlx::query_as!(
            User,
            r#"
            SELECT
            id as "id!",
            account_id as "account_id!",
            role_id as "role_id!",
            role_access_level as "role_access_level: RoleAccessLevel",
            username as "username!",
            password_hash as "password_hash!",
            email as "email!",
            is_active as "is_active!",
            created_at as "created_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            FROM users
            WHERE account_id = ? AND is_deleted = 0
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
            account_id,
            limit,
            offset
        )
        .fetch_all(self.pool)
        .await?;

        Ok(users)
    }

    /// Get total count of users for an account
    pub async fn get_users_count_by_account_id(&self, account_id: &str) -> Result<u64> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) as count FROM users WHERE account_id = ? AND is_deleted = 0",
            account_id
        )
        .fetch_one(self.pool)
        .await?;

        Ok(count as u64)
    }
}
