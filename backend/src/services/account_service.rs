//! Account business logic service.
//!
//! Handles all account-related business operations

use crate::database::models::{Account, CreateAccount, CreateUser, UserWithAccount};
use crate::errors::{ServiceError, ServiceResult};
use crate::repositories::account_repository::AccountRepository;
use crate::repositories::role_repository::RoleRepository;
use crate::services::user_service::UserService;
use sqlx::SqlitePool;
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
    /// - Duplicate account names
    /// - Missing required roles
    /// - Business rule violations
    pub async fn create_account(
        &self,
        create_account: CreateAccount,
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

        let repo = AccountRepository::new(self.pool);

        // Check if account name already exists
        if repo.account_name_exists(&create_account.name).await? {
            return Err(ServiceError::already_exists(
                "Account",
                &create_account.name,
            ));
        }

        // Business validation
        self.validate_business_rules(&create_account)?;

        // Check if the role exists
        let role_repo = RoleRepository::new(self.pool);
        let role = role_repo.get_role_by_name("Admin").await?;
        if role.is_none() {
            return Err(ServiceError::not_found("Role", "Admin"));
        }

        // Create the account
        let account = repo.create_account(create_account.clone()).await?;

        // Create the admin user for the account
        let user_service = UserService::new(self.pool);
        let create_user = CreateUser {
            account_id: account.id.clone(),
            name: create_account.username.clone(),
            email: create_account.email.clone(),
            role_id: role.unwrap().id.clone(),
        };

        let user = user_service.create_user(create_user).await?;

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
    fn validate_business_rules(&self, create_account: &CreateAccount) -> ServiceResult<()> {
        // Validate name doesn't start with numbers or special characters
        if create_account
            .name
            .chars()
            .next()
            .map_or(false, |c| c.is_numeric() || !c.is_alphanumeric())
        {
            return Err(ServiceError::validation(
                "Account name must start with a letter",
            ));
        }

        Ok(())
    }
}
