//! Account business logic service.
//!
//! Handles all account-related business operations

use crate::database::models::{
    Account, CreateAccount, CreateNewAccount, RoleAccessLevel, UserWithAccount,
};
use crate::errors::{ServiceError, ServiceResult};
use crate::repositories::account_repository::AccountRepository;
use crate::repositories::role_repository::RoleRepository;
use sqlx::SqlitePool;
use uuid::Uuid;
use validator::Validate;

/// Service layer for account operations.
pub struct AccountService<'a> {
    /// Shared database connection pool
    pool: &'a SqlitePool,
}

impl<'a> AccountService<'a> {
    /// Creates a new AccountService instance.
    ///
    /// # Arguments
    /// * `pool` - Reference to SQLite connection pool
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Creates a new account with full validation and setup.
    ///
    /// # Arguments
    /// * `create_account` - Account creation data transfer object
    ///
    /// # Returns
    /// Combined `UserWithAccount` containing both the new account and admin user
    ///
    /// # Errors
    /// Returns `ServiceError` for:
    /// - Validation failures
    /// - Duplicate account names, usernames, or emails
    /// - Missing required roles
    /// - Business rule violations
    pub async fn create_account(
        &self,
        create_account: CreateNewAccount,
    ) -> ServiceResult<UserWithAccount> {
        // Input validation using validator crate
        if let Err(validation_errors) = create_account.validate() {
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

        // Pre-validation checks
        let account_repo = AccountRepository::new(self.pool);
        let user_repo = crate::repositories::user_repository::UserRepository::new(self.pool);

        // Check if account name already exists
        if account_repo
            .account_name_exists(&create_account.name)
            .await?
        {
            return Err(ServiceError::already_exists(
                "Account",
                &create_account.name,
            ));
        }

        // Check if username already exists
        if user_repo.username_exists(&create_account.username).await? {
            return Err(ServiceError::already_exists(
                "User with username",
                &create_account.username,
            ));
        }

        // Check if email already exists
        if user_repo.email_exists(&create_account.email).await? {
            return Err(ServiceError::already_exists(
                "User with email",
                &create_account.email,
            ));
        }

        // Business validation
        self.validate_business_rules(&create_account)?;

        // Check if the Admin role exists
        let role_repo = RoleRepository::new(self.pool);
        let role = role_repo.get_role_by_name("Admin").await?;
        if role.is_none() {
            return Err(ServiceError::not_found("Role", "Admin"));
        }

        let role = role.unwrap();

        // Start a transaction for atomic account + user creation
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ServiceError::Database { source: e.into() })?;

        // Create the account
        let new_account = CreateAccount {
            name: create_account.name.clone(),
            username: create_account.username.clone(),
            email: create_account.email.clone(),
        };

        let account_id = Uuid::now_v7().to_string();
        // Insert the account into the database
        let account = sqlx::query_as!(
            crate::database::models::Account,
            r#"
            INSERT INTO accounts (id, name, is_active)
            VALUES (?, ?, ?)
            RETURNING
            id as "id!",
            name as "name!",
            is_active as "is_active!",
            created_at as "created_at!: chrono::DateTime<chrono::Utc>",
            updated_at as "updated_at!: chrono::DateTime<chrono::Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: chrono::DateTime<chrono::Utc>"
            "#,
            account_id,
            new_account.name,
            true
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("UNIQUE constraint failed: accounts.name") {
                ServiceError::already_exists("Account", &create_account.name)
            } else {
                ServiceError::Database { source: e.into() }
            }
        })?;

        // Create the admin user for the account
        let password_hash = bcrypt::hash(&create_account.password, bcrypt::DEFAULT_COST)
            .map_err(|e| ServiceError::validation(format!("Password hashing failed: {e}")))?;

        let user_id = Uuid::now_v7().to_string();
        // Insert the user into the database
        let user = sqlx::query_as!(
            crate::database::models::User,
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
            created_at as "created_at!: chrono::DateTime<chrono::Utc>",
            updated_at as "updated_at!: chrono::DateTime<chrono::Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: chrono::DateTime<chrono::Utc>"
            "#,
            user_id,
            account.id,
            role.id,
            RoleAccessLevel::ReadWrite,
            create_account.username,
            password_hash,
            create_account.email,
            true
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("UNIQUE constraint failed: users.username") {
                ServiceError::already_exists("User with username", &create_account.username)
            } else if error_msg.contains("UNIQUE constraint failed: users.email") {
                ServiceError::already_exists("User with email", &create_account.email)
            } else {
                ServiceError::Database { source: e.into() }
            }
        })?;

        // Commit the transaction
        tx.commit()
            .await
            .map_err(|e| ServiceError::Database { source: e.into() })?;

        // Return the created account and user
        let user_with_account = UserWithAccount { account, user };

        Ok(user_with_account)
    }

    /// Retrieves an account by ID, returning error if not found.
    ///
    /// # Arguments
    /// * `id` - Account ID (UUID format)
    ///
    /// # Returns
    /// The requested Account if found
    ///
    /// # Errors
    /// Returns `ServiceError::NotFound` if account doesn't exist
    pub async fn get_account_required(&self, id: &str) -> ServiceResult<Account> {
        let repo = AccountRepository::new(self.pool);
        let account = repo
            .get_account_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Account", id))?;
        Ok(account)
    }

    /// Business validation rules.
    fn validate_business_rules(&self, create_account: &CreateNewAccount) -> ServiceResult<()> {
        // Validate name doesn't start with numbers or special characters
        if create_account
            .name
            .chars()
            .next()
            .is_some_and(|c| c.is_numeric() || !c.is_alphanumeric())
        {
            return Err(ServiceError::validation(
                "Account name must start with a letter",
            ));
        }

        Ok(())
    }
}
