//! Database repository for invite management operations.
//!
//! Provides CRUD operations for system invites

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::models::{CreateInvite, Invite, InviteStatus};

/// Repository for invite database operations.
///
/// Handles all persistence operations for the Invite entity,
/// maintaining relationships with accounts and roles.
pub struct InviteRepository<'a> {
    /// Shared SQLite connection pool
    pool: &'a SqlitePool,
}

impl<'a> InviteRepository<'a> {
    /// Creates a new InviteRepository instance.
    ///
    /// # Arguments
    /// * `pool` - Reference to SQLite connection pool
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Creates a new invite in the database.
    ///
    /// # Arguments
    /// * `invite` - CreateInvite DTO containing invite details
    ///
    /// # Returns
    /// The newly created Invite with all fields populated
    pub async fn create_invite(&self, invite: CreateInvite) -> Result<Invite> {
        let invite = sqlx::query_as!(
            Invite,
            r#"
            INSERT INTO invites (account_id, inviter_id, invitee_email, token, invite_status, expires_at, is_active)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING 
            id as "id!",
            account_id as "account_id!",
            inviter_id as "inviter_id!",
            invitee_email as "invitee_email!",
            token as "token!",
            invite_status as "invite_status: InviteStatus",
            is_active as "is_active!",
            created_at as "created_at!: DateTime<Utc>",
            expires_at as "expires_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            "#,
            invite.account_id,
            invite.inviter_id,
            invite.invitee_email,
            invite.token,
            invite.invite_status,
            invite.expires_at,
            true
        )
        .fetch_one(self.pool)
        .await?;

        Ok(invite)
    }

    /// Updates an existing invite status in the database.
    ///
    /// # Arguments
    /// * `invite_id` - Unique identifier of the invite to update
    /// * `new_status` - New status to set for the invite
    /// # Returns
    /// `true` if the update was successful, `false` otherwise
    pub async fn update_invite_status(
        &self,
        invite_id: &str,
        new_status: InviteStatus,
    ) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            UPDATE invites 
            SET invite_status = ?, 
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ? AND is_deleted = 0
            "#,
            new_status,
            invite_id
        )
        .execute(self.pool)
        .await?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    /// Retrieves a invite by their unique identifier.
    ///
    /// # Arguments
    /// * `id` - Invite ID (UUID format)
    ///
    /// # Returns
    /// `Some(Invite)` if found and active, `None` otherwise
    pub async fn get_invite_by_id(&self, id: &str) -> Result<Option<Invite>> {
        let invite = sqlx::query_as!(
            Invite,
            r#"
            SELECT 
            id as "id!",
            account_id as "account_id!",
            inviter_id as "inviter_id!",
            invitee_email as "invitee_email!",
            token as "token!",
            invite_status as "invite_status: InviteStatus",
            is_active as "is_active!",
            created_at as "created_at!: DateTime<Utc>",
            expires_at as "expires_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            FROM invites WHERE id = ? AND is_deleted = 0
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(invite)
    }

    /// Retrieves a invite by their token.
    ///
    /// # Arguments
    /// * `token` - Invite token (String format)
    ///
    /// # Returns
    /// `Some(Invite)` if found and active, `None` otherwise
    pub async fn get_invite_by_token(&self, token: &str) -> Result<Option<Invite>> {
        let invite = sqlx::query_as!(
            Invite,
            r#"
            SELECT 
            id as "id!",
            account_id as "account_id!",
            inviter_id as "inviter_id!",
            invitee_email as "invitee_email!",
            token as "token!",
            invite_status as "invite_status: InviteStatus",
            is_active as "is_active!",
            created_at as "created_at!: DateTime<Utc>",
            expires_at as "expires_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            FROM invites WHERE token = ? AND is_deleted = 0
            "#,
            token
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(invite)
    }

    /// Retrieves a invite by the invitee email and account ID.
    ///
    /// # Arguments
    /// * `email` - Invitee email (UUID format)
    /// * `account_id` - Account ID (UUID format)
    ///
    /// # Returns
    /// `Some(Invite)` if found and active, `None` otherwise
    pub async fn get_invite_by_email(
        &self,
        email: &str,
        account_id: &str,
    ) -> Result<Option<Invite>> {
        let invite = sqlx::query_as!(
            Invite,
            r#"
            SELECT 
            id as "id!",
            account_id as "account_id!",
            inviter_id as "inviter_id!",
            invitee_email as "invitee_email!",
            token as "token!",
            invite_status as "invite_status: InviteStatus",
            is_active as "is_active!",
            created_at as "created_at!: DateTime<Utc>",
            expires_at as "expires_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            FROM invites WHERE invitee_email = ? AND account_id = ? AND is_deleted = 0
            "#,
            email,
            account_id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(invite)
    }

    /// Retrieves the invite for a specific account.
    ///
    /// # Arguments
    /// * `account_id` - Account ID (UUID format)
    ///
    /// # Returns
    /// `Some(Invite)` if invites exist for account, `None` otherwise
    pub async fn get_invites_by_account_id(&self, account_id: &str) -> Result<Vec<Invite>> {
        let invites = sqlx::query_as!(
            Invite,
            r#"
            SELECT 
            id as "id!",
            account_id as "account_id!",
            inviter_id as "inviter_id!",
            invitee_email as "invitee_email!",
            token as "token!",
            invite_status as "invite_status: InviteStatus",
            is_active as "is_active!",
            created_at as "created_at!: DateTime<Utc>",
            expires_at as "expires_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            FROM invites
            WHERE account_id = ? AND is_deleted = 0
            ORDER BY created_at DESC
            "#,
            account_id
        )
        .fetch_all(self.pool)
        .await?;

        Ok(invites)
    }
}
