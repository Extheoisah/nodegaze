//! User business logic service.
//!
//! Handles all account-related business operations

use crate::api::account;
use crate::database::models::{CreateUser, User};
use crate::errors::{ServiceError, ServiceResult};
use crate::repositories::account_repository::AccountRepository;
use crate::repositories::role_repository::RoleRepository;
use crate::repositories::user_repository::UserRepository;
use sqlx::SqlitePool;
use validator::Validate;

pub struct UserService<'a> {
    /// Shared database connection pool
    pool: &'a SqlitePool,
}

impl<'a> UserService<'a> {
    /// Creates a new UserService instance.
    ///
    /// # Arguments
    /// * `pool` - Reference to SQLite connection pool
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Creates a new user with full validation.
    ///
    /// # Arguments
    /// * `create_user` - User creation data transfer object
    ///
    /// # Returns
    /// The newly created User with all fields populated
    ///
    /// # Errors
    /// Returns `ServiceError` for:
    /// - Validation failures
    /// - Non-existent account or role
    /// - Duplicate admin users for account
    /// - Business rule violations
    pub async fn create_user(&self, create_user: CreateUser) -> ServiceResult<User> {
        // Input validation using validator crate
        if let Err(validation_errors) = create_user.validate() {
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

        // Check if the account exists
        let account_repo = AccountRepository::new(self.pool);
        if account_repo
            .get_account_by_id(&create_user.account_id)
            .await?
            .is_none()
        {
            return Err(ServiceError::not_found("Account", &create_user.account_id));
        }

        let repo = UserRepository::new(self.pool);

        // Check if user already exists
        //todo: Implement user existence check
        if repo
            .get_admin_user_by_account_id(&create_user.account_id)
            .await?
            .is_some()
        {
            return Err(ServiceError::already_exists(
                "Admin User",
                &create_user.name,
            ));
        }

        // Business validation
        self.validate_business_rules(&create_user)?;

        // Check if role exists
        let role_repo = RoleRepository::new(self.pool);
        let role = role_repo.get_role_by_id(&create_user.role_id).await?;

        if role.is_none() {
            return Err(ServiceError::not_found("Role", &create_user.role_id));
        }

        // Create the user
        let user = repo.create_user(create_user).await?;
        Ok(user)
    }

    /// Retrieves a user by ID with existence verification.
    ///
    /// # Arguments
    /// * `id` - User ID (UUID format)
    ///
    /// # Returns
    /// The requested User if found
    ///
    /// # Errors
    /// Returns `ServiceError::NotFound` if user doesn't exist
    pub async fn get_user_required(&self, id: &str) -> ServiceResult<User> {
        let repo = UserRepository::new(self.pool);
        let user = repo
            .get_user_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::not_found("User", id))?;
        Ok(user)
    }

    /// Business validation rules.
    fn validate_business_rules(&self, create_user: &CreateUser) -> ServiceResult<()> {
        // Validate name doesn't start with numbers or special characters
        if create_user
            .name
            .chars()
            .next()
            .map_or(false, |c| c.is_numeric() || !c.is_alphanumeric())
        {
            return Err(ServiceError::validation(
                "User name must start with a letter",
            ));
        }

        Ok(())
    }
}
