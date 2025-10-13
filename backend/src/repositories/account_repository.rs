//! Database repository for account management operations.
//!
//! Provides CRUD operations and business logic for accounts.

use crate::database::models::Account;
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
        // Shared SQLite connection pool
        Self { pool }
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
}
