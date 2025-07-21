//! Database repository for account management operations.
//!
//! Provides CRUD operations and business logic for accounts.

use crate::database::models::{Account, CreateAccount};
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

pub struct AccountRepository<'a> {
    pool: &'a SqlitePool,
}

/// Repository for account database operations.
///
/// Handles all persistence operations for the Account entity,
/// enforcing business rules and maintaining data consistency.
impl<'a> AccountRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        /// Shared SQLite connection pool
        Self { pool }
    }

    /// Creates a new account in the database.
    ///
    /// # Arguments
    /// * `account` - CreateAccount DTO containing account details
    ///
    /// # Returns
    /// The newly created Account with all fields populated
    pub async fn create_account(&self, account: CreateAccount) -> Result<Account> {
        let account = sqlx::query_as!(
            Account,
            r#"
            INSERT INTO accounts (name, is_active)
            VALUES (?, ?)
            RETURNING 
            id as "id!",
            name as "name!",
            is_active as "is_active!",
            created_at as "created_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            "#,
            account.name,
            true
        )
        .fetch_one(self.pool)
        .await?;

        Ok(account)
    }

    /// Retrieves an account by its ID.
    ///
    /// # Arguments
    /// * `id` - Account ID (UUID format)
    ///
    /// # Returns
    /// `Some(Account)` if found and not deleted, `None` otherwise
    pub async fn get_account_by_id(&self, id: &str) -> Result<Option<Account>> {
        let account = sqlx::query_as!(
            Account,
            r#"
            SELECT 
            id as "id!",
            name as "name!",
            is_active as "is_active!",
            created_at as "created_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            FROM accounts WHERE id = ? AND is_deleted = 0
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(account)
    }

    /// Retrieves an account by its name.
    ///
    /// # Arguments
    /// * `name` - Exact account name to search for
    ///
    /// # Returns
    /// `Some(Account)` if found and not deleted, `None` otherwise
    pub async fn get_account_by_name(&self, name: &str) -> Result<Option<Account>> {
        let account = sqlx::query_as!(
            Account,
            r#"
            SELECT 
            id as "id!",
            name as "name!",
            is_active as "is_active!",
            created_at as "created_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            FROM accounts WHERE name = ? AND is_deleted = 0
            "#,
            name
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(account)
    }

    /// Retrieves all accounts with pagination and search.
    ///
    /// # Arguments
    /// * `limit` - Maximum number of accounts to return (default: 10)
    /// * `search` - Optional name search pattern (uses SQL LIKE)
    ///
    /// # Returns
    /// Vector of accounts sorted by creation date (newest first)
    pub async fn get_all_accounts(
        &self,
        limit: Option<i64>,
        search: Option<String>,
    ) -> Result<Vec<Account>> {
        let limit = limit.unwrap_or(10);
        let search_pattern = format!("%{}%", search.unwrap_or_default());

        let accounts = sqlx::query_as!(
            Account,
            r#"
            SELECT 
            id as "id!",
            name as "name!",
            is_active as "is_active!",
            created_at as "created_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            FROM accounts WHERE is_deleted = 0
            AND name LIKE ?
            ORDER BY created_at DESC
            LIMIT ?
            "#,
            search_pattern,
            limit
        )
        .fetch_all(self.pool)
        .await?;

        Ok(accounts)
    }

    /// Counts all active, non-deleted accounts.
    ///
    /// # Returns
    /// Total count of active accounts
    pub async fn count_active_accounts(&self) -> Result<i64> {
        let count = sqlx::query!(
            "SELECT COUNT(*) as count FROM accounts WHERE is_deleted = 0 AND is_active = 1"
        )
        .fetch_one(self.pool)
        .await?;

        Ok(count.count)
    }

    /// Checks if an account name already exists.
    ///
    /// # Arguments
    /// * `name` - Name to check
    ///
    /// # Returns
    /// `true` if an active account with this name exists
    pub async fn account_name_exists(&self, name: &str) -> Result<bool> {
        let count = sqlx::query!(
            "SELECT COUNT(*) as count FROM accounts WHERE name = ? AND is_deleted = 0",
            name
        )
        .fetch_one(self.pool)
        .await?;

        Ok(count.count > 0)
    }

    /// Checks if account name exists excluding a specific account.
    ///
    /// # Arguments
    /// * `name` - Name to check
    /// * `exclude_id` - Account ID to exclude from check
    ///
    /// # Returns
    /// `true` if another active account with this name exists
    ///
    /// # Use Case
    /// Useful for update operations to prevent name collisions
    pub async fn account_name_exists_excluding(
        &self,
        name: &str,
        exclude_id: &str,
    ) -> Result<bool> {
        let count = sqlx::query!(
            "SELECT COUNT(*) as count FROM accounts WHERE name = ? AND id != ? AND is_deleted = 0",
            name,
            exclude_id
        )
        .fetch_one(self.pool)
        .await?;

        Ok(count.count > 0)
    }

    /// Soft deletes an account by marking it as deleted.
    ///
    /// # Arguments
    /// * `id` - Account ID to delete
    ///
    /// # Effects
    /// - Sets `is_deleted` to 1
    /// - Updates `deleted_at` timestamp
    /// - Account remains in database but won't appear in queries
    pub async fn delete_account(&self, id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE accounts
            SET is_deleted = 1, deleted_at = CURRENT_TIMESTAMP
            WHERE id = ? AND is_deleted = 0
            "#,
            id
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }
}
