//! Invite business logic service.
//!
//! Handles all account-related business operations

use crate::config::Config;
use crate::database::models::{
    AcceptInviteRequest, CreateInvite, CreateInviteRequest, Invite, InviteStatus, User,
};
use crate::errors::{ServiceError, ServiceResult};
use crate::repositories::account_repository::AccountRepository;
use crate::repositories::invite_repository::InviteRepository;
use crate::repositories::role_repository::RoleRepository;
use crate::repositories::user_repository::UserRepository;
use crate::services::email_service::EmailService;
use crate::utils::generate_random_string::generate_random_string;
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{Duration, Utc};
use sqlx::SqlitePool;
use validator::Validate;

pub struct InviteService<'a> {
    /// Shared database connection pool
    pool: &'a SqlitePool,
    /// Email service for sending invite emails
    email_service: Option<EmailService>,
}

impl<'a> InviteService<'a> {
    /// Creates a new InviteService instance.
    ///
    /// # Arguments
    /// * `pool` - Reference to SQLite connection pool
    pub fn new(pool: &'a SqlitePool, config: &Config) -> Self {
        let email_service = match config.email_config() {
            Some(email_config) => match EmailService::new(email_config) {
                Ok(service) => {
                    tracing::info!("Email service initialized successfully");
                    Some(service)
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to initialize email service: {}. Email notifications will be disabled.",
                        e
                    );
                    None
                }
            },
            None => {
                tracing::warn!(
                    "Email configuration not found. Email notifications will be disabled."
                );
                None
            }
        };

        Self {
            pool,
            email_service,
        }
    }

    /// Creates and sends a new invite with full validation.
    ///
    /// # Arguments
    /// * `create_invite` - Invite creation data transfer object
    ///
    /// # Returns
    /// The newly created Invite with all fields populated
    ///
    /// # Errors
    /// Returns `ServiceError` for:
    /// - Validation failures
    /// - Duplicate invites
    pub async fn create_invite(
        &self,
        create_invite: CreateInviteRequest,
        user: User,
    ) -> ServiceResult<Invite> {
        let create_invite = CreateInvite {
            account_id: user.account_id.clone(),
            invitee_email: create_invite.email,
            inviter_id: user.id.clone(),
            invite_status: InviteStatus::Pending,
            token: generate_random_string(20),
            expires_at: Utc::now() + Duration::days(7),
        };

        // Input validation using validator crate
        if let Err(validation_errors) = create_invite.validate() {
            let error_messages: Vec<String> = validation_errors
                .field_errors()
                .into_iter()
                .flat_map(|(field, errors)| {
                    errors.iter().map(move |error| {
                        format!(
                            "{}: {}",
                            field,
                            error.message.as_ref().unwrap_or(&"Invalid value".into())
                        )
                    })
                })
                .collect();

            return Err(ServiceError::validation(error_messages.join(", ")));
        }

        let invite_repo = InviteRepository::new(self.pool);
        let user_repo = UserRepository::new(self.pool);
        let account_repo = AccountRepository::new(self.pool);

        // Check if invitee email is a user globally
        if user_repo.email_exists(&create_invite.invitee_email).await? {
            return Err(ServiceError::already_exists(
                "User with email",
                &create_invite.invitee_email,
            ));
        }

        // Check if invite already exists for the same account or has been accepted
        let existing_invite = invite_repo
            .get_invite_by_email(&create_invite.invitee_email, &create_invite.account_id)
            .await?;
        if let Some(ref invite) = existing_invite {
            match invite.invite_status {
                InviteStatus::Accepted => {
                    return Err(ServiceError::invalid_operation(
                        "Invitation already accepted",
                    ));
                }
                InviteStatus::Pending => {
                    return Err(ServiceError::invalid_operation("Invitation already sent"));
                }
            }
        }

        let invite = invite_repo
            .create_invite(create_invite.clone())
            .await
            .map_err(|e| {
                // Handle potential database constraint violations
                let error_msg = e.to_string();
                if error_msg.contains("UNIQUE constraint failed:") {
                    ServiceError::invalid_operation("Invitation already exists")
                } else {
                    ServiceError::Database { source: e }
                }
            })?;

        self.try_send_invite_email(&invite, &user).await;

        Ok(invite)
    }

    /// Attempts to send an invite email, logging but not failing if email service is unavailable
    async fn try_send_invite_email(&self, invite: &Invite, inviter: &User) {
        if let Some(ref email_service) = self.email_service {
            match self
                .send_invite_email_internal(email_service, invite, inviter)
                .await
            {
                Ok(_) => {
                    tracing::info!("Invite email sent successfully to {}", invite.invitee_email);
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to send invite email to {}: {}",
                        invite.invitee_email,
                        e
                    );
                }
            }
        } else {
            tracing::warn!(
                "Email service not configured. Invite email not sent to {}",
                invite.invitee_email
            );
        }
    }

    /// Internal method to send email when service is available
    async fn send_invite_email_internal(
        &self,
        email_service: &EmailService,
        invite: &Invite,
        inviter: &User,
    ) -> ServiceResult<()> {
        // Get account details
        let account_repo = AccountRepository::new(self.pool);
        let account = account_repo
            .get_account_by_id(&invite.account_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Account", &invite.account_id))?;

        email_service
            .send_invite_email(
                &invite.invitee_email,
                None,
                &invite.token.as_str(),
                &inviter.username,
                &account.name,
            )
            .await
    }

    pub async fn get_invites_by_account_id(&self, account_id: &str) -> ServiceResult<Vec<Invite>> {
        let repo = InviteRepository::new(self.pool);
        let invites = repo.get_invites_by_account_id(account_id).await?;

        Ok(invites)
    }

    pub async fn resend_invite(&self, invite_id: &str, user: &User) -> ServiceResult<Invite> {
        let repo = InviteRepository::new(self.pool);
        let mut invite = repo
            .get_invite_by_id(invite_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Invite", invite_id))?;

        // Verify that the invite belongs to the account
        if invite.account_id != user.account_id {
            return Err(ServiceError::not_found("Invite", &invite_id.to_string()));
        }

        let expires_at = Utc::now() + Duration::days(7);

        let rows_affected = sqlx::query!(
            r#"
            UPDATE invites 
            SET expires_at = ?, 
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ? AND is_deleted = 0
            "#,
            expires_at,
            invite.id
        )
        .execute(self.pool)
        .await
        .map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("FOREIGN KEY constraint failed") {
                ServiceError::validation("Invalid invite reference")
            } else if error_msg.contains("CHECK constraint failed") {
                ServiceError::validation("Invalid expires_at value")
            } else {
                ServiceError::Database { source: e.into() }
            }
        })?
        .rows_affected();

        // Check if the update actually affected any rows
        if rows_affected == 0 {
            return Err(ServiceError::not_found("Invitation not resent", &invite.id));
        }

        self.try_send_invite_email(&invite, &user).await;
        Ok(invite)
    }

    /// Retrieves a invite by ID with existence verification.
    ///
    /// # Arguments
    /// * `id` - Invite ID (UUID format)
    ///
    /// # Returns
    /// The requested Invite if found
    ///
    /// # Errors
    /// Returns `ServiceError::NotFound` if invite doesn't exist
    pub async fn get_invite_required(&self, id: &str, account_id: &str) -> ServiceResult<Invite> {
        let repo = InviteRepository::new(self.pool);
        let invite = repo
            .get_invite_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Invite", id))?;

        // Verify that the invite belongs to the account
        if invite.account_id != account_id {
            return Err(ServiceError::not_found("Invite", id));
        }

        Ok(invite)
    }

    /// Accepts an invite using the provided token.
    ///
    /// This method validates the invite token and processes the acceptance of the
    /// invitation. The token should be a valid, unexpired invite token that was
    /// previously generated.
    ///
    /// # Arguments
    ///
    /// * `token` - A reference to the invite token string to be accepted
    ///
    /// # Returns
    ///
    /// * `Ok(Invite)` - The accepted invite object containing invite details
    /// * `Err(ServiceError)` - If the token is invalid, expired, already used, or other processing errors occur
    ///
    pub async fn accept_invite(&self, accept_invite: &AcceptInviteRequest) -> ServiceResult<User> {
        let repo = InviteRepository::new(self.pool);

        // Get invite by token
        let mut invite = repo
            .get_invite_by_token(&accept_invite.token)
            .await?
            .ok_or_else(|| ServiceError::not_found("Invite", &accept_invite.token))?;

        // Check if invite is still valid
        if invite.invite_status != InviteStatus::Pending {
            return Err(ServiceError::validation(
                "Invite has already been processed",
            ));
        }

        // Check if invite hasn't expired
        if invite.expires_at <= Utc::now() {
            return Err(ServiceError::validation("Invite has expired"));
        }

        // Start a transaction for invite acceptance + user creation
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ServiceError::Database { source: e.into() })?;

        // Update invite status to accepted
        let rows_affected = sqlx::query!(
            r#"
            UPDATE invites 
            SET invite_status = ?, 
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ? AND is_deleted = 0
            "#,
            InviteStatus::Accepted,
            invite.id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("FOREIGN KEY constraint failed") {
                ServiceError::validation("Invalid invite reference")
            } else if error_msg.contains("CHECK constraint failed") {
                ServiceError::validation("Invalid expires_at value")
            } else {
                ServiceError::Database { source: e.into() }
            }
        })?
        .rows_affected();

        // Check if the update actually affected any rows
        if rows_affected == 0 {
            return Err(ServiceError::validation("Invitation not resent"));
        }

        // TODO: Create user account for the invitee
        // This would typically involve:
        // 1. Creating a new user with the invitee_email
        // 2. Associating them with the account_id
        // 3. Setting appropriate roles/permissions

        // Check if the Member role exists
        let role_repo = RoleRepository::new(self.pool);
        let role = role_repo.get_role_by_name("Member").await?;
        if role.is_none() {
            return Err(ServiceError::not_found("Role", "Member"));
        }

        let role = role.unwrap();

        let password_hash = bcrypt::hash(&accept_invite.password, bcrypt::DEFAULT_COST)
            .map_err(|e| ServiceError::validation(format!("Password hashing failed: {}", e)))?;

        let user = sqlx::query_as!(
            crate::database::models::User,
            r#"
            INSERT INTO users (account_id, role_id, username, password_hash, email, is_active)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING
            id as "id!",
            account_id as "account_id!",
            role_id as "role_id!",
            username as "username!",
            password_hash as "password_hash!",
            email as "email!",
            is_active as "is_active!",
            created_at as "created_at!: chrono::DateTime<chrono::Utc>",
            updated_at as "updated_at!: chrono::DateTime<chrono::Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: chrono::DateTime<chrono::Utc>"
            "#,
            invite.account_id,
            role.id,
            accept_invite.username,
            password_hash,
            invite.invitee_email,
            true
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("UNIQUE constraint failed: users.username") {
                ServiceError::already_exists("User with username", &accept_invite.username)
            } else if error_msg.contains("UNIQUE constraint failed: users.email") {
                ServiceError::already_exists("User with email", &invite.invitee_email)
            } else {
                ServiceError::Database { source: e.into() }
            }
        })?;

        // Commit the transaction
        tx.commit()
            .await
            .map_err(|e| ServiceError::Database { source: e.into() })?;

        Ok(user)
    }
}
