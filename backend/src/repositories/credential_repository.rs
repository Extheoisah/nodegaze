//! Database repository for credential management operations.
//!
//! Provides CRUD operations for node credentials.
use crate::database::models::{CreateCredential, Credential};
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

/// Repository for credential database operations.
///
/// Manages persistence of node authentication credentials including:
/// - Lightning node connection details
/// - Macaroon authentication tokens
/// - TLS certificates
/// - Node addressing information
pub struct CredentialRepository<'a> {
    /// Shared SQLite connection pool
    pool: &'a SqlitePool,
}

impl<'a> CredentialRepository<'a> {
    /// Creates a new CredentialRepository instance.
    ///
    /// # Arguments
    /// * `pool` - Reference to SQLite connection pool
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Stores new node credentials in the database.
    ///
    /// # Arguments
    /// * `credential` - CreateCredential DTO containing all authentication details
    ///
    /// # Returns
    /// The newly created Credential with all fields populated
    ///
    /// # Security
    /// - Sets `is_active` to true by default for new credentials
    /// - Stores sensitive data (macaroon, TLS cert) encrypted at rest
    pub async fn create_credential(&self, credential: CreateCredential) -> Result<Credential> {
        let credential = sqlx::query_as!(
            Credential,
            r#"
            INSERT INTO credentials (id, user_id, account_id, node_id, node_alias, macaroon, tls_cert, address, node_type, client_cert, client_key, ca_cert, is_active)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING
            id as "id!",
            user_id as "user_id!",
            account_id as "account_id!",
            node_id as "node_id!",
            node_alias as "node_alias!",
            macaroon as "macaroon!",
            tls_cert as "tls_cert!",
            address as "address!",
            node_type as "node_type?",
            client_cert as "client_cert?",
            client_key as "client_key?",
            ca_cert as "ca_cert?",
            is_active as "is_active!",
            created_at as "created_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>"
            "#,
            credential.id,
            credential.user_id,
            credential.account_id,
            credential.node_id,
            credential.node_alias,
            credential.macaroon,
            credential.tls_cert,
            credential.address,
            credential.node_type,
            credential.client_cert,
            credential.client_key,
            credential.ca_cert,
            true
        )
        .fetch_one(self.pool)
        .await?;

        Ok(credential)
    }

    /// Retrieves credentials associated with a specific user.
    ///
    /// # Arguments
    /// * `user_id` - User ID (UUID format)
    ///
    /// # Returns
    /// `Some(Credential)` if found, `None` otherwise
    pub async fn get_credential_by_user_id(&self, user_id: &str) -> Result<Option<Credential>> {
        let credential = sqlx::query_as!(
            Credential,
            r#"
                SELECT
                id as "id!",
                user_id as "user_id!",
                account_id as "account_id!",
                node_id as "node_id!",
                node_alias as "node_alias!",
                macaroon as "macaroon!",
                tls_cert as "tls_cert!",
                address as "address!",
                node_type as "node_type?",
                client_cert as "client_cert?",
                client_key as "client_key?",
                ca_cert as "ca_cert?",
                is_active as "is_active!",
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at!: DateTime<Utc>"
                FROM credentials WHERE user_id = ?
                "#,
            user_id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(credential)
    }

    /// Retrieves credentials associated with a specific account.
    ///
    /// # Arguments
    /// * `account_id` - Account ID (UUID format)
    ///
    /// # Returns
    /// `Some(Credential)` if found, `None` otherwise
    pub async fn get_credential_by_account_id(
        &self,
        account_id: &str,
    ) -> Result<Option<Credential>> {
        let credential = sqlx::query_as!(
            Credential,
            r#"
                SELECT
                id as "id!",
                user_id as "user_id!",
                account_id as "account_id!",
                node_id as "node_id!",
                node_alias as "node_alias!",
                macaroon as "macaroon!",
                tls_cert as "tls_cert!",
                address as "address!",
                node_type as "node_type?",
                client_cert as "client_cert?",
                client_key as "client_key?",
                ca_cert as "ca_cert?",
                is_active as "is_active!",
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at!: DateTime<Utc>"
                FROM credentials WHERE account_id = ?
                "#,
            account_id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(credential)
    }

    /// Deletes a credential permanently from the database.
    ///
    /// # Arguments
    /// * `id` - Credential ID to delete
    ///
    /// # Effects
    /// - Permanently removes credential from database
    /// - Cannot be recovered after deletion
    ///
    /// # Security
    /// - Ensures credential cannot be used after deletion
    pub async fn delete_credential(&self, id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM credentials
            WHERE id = ?
            "#,
            id
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }
}
