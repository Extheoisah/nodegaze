//! User business logic service.
//!
//! Handles all account-related business operations

use crate::api::common::PaginationFilter;
use crate::database::models::{RoleAccessLevel, User};
use crate::errors::{ServiceError, ServiceResult};
use crate::repositories::role_repository::RoleRepository;
use crate::repositories::user_repository::UserRepository;
use bcrypt::{DEFAULT_COST, hash, verify};
use sqlx::SqlitePool;

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

    /// Function to hash a password before storing in database
    ///
    /// # Arguments
    /// * `password` - Plain text password to hash
    ///
    /// # Returns
    /// Hashed password string
    ///
    /// # Errors
    /// Returns `ServiceError` if hashing fails
    fn hash_password(&self, password: &str) -> ServiceResult<String> {
        hash(password, DEFAULT_COST)
            .map_err(|e| ServiceError::validation(format!("Password hashing failed: {e}")))
    }

    /// Function to verify a password against the stored hash
    ///
    /// # Arguments
    /// * `password` - Plain text password to verify
    /// * `hash` - Stored password hash
    ///
    /// # Returns
    /// `true` if password matches hash, `false` otherwise
    ///
    /// # Errors
    /// Returns `ServiceError` if verification process fails
    fn verify_password(&self, password: &str, hash: &str) -> ServiceResult<bool> {
        verify(password, hash)
            .map_err(|e| ServiceError::validation(format!("Password verification failed: {e}")))
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

    /// Retrieves an admin user by Account ID.
    ///
    /// # Arguments
    /// * `id` - Account ID (UUID format)
    ///
    /// # Returns
    /// The requested Admin User if found
    ///
    /// # Errors
    /// Returns `ServiceError::NotFound` if admin user doesn't exist
    pub async fn get_admin_user_required(&self, id: &str) -> ServiceResult<User> {
        let repo = UserRepository::new(self.pool);
        let user = repo
            .get_admin_user_by_account_id(id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Admin User", id))?;
        Ok(user)
    }

    /// Retrieves all users by Account ID.
    ///
    /// # Arguments
    /// * `id` - Account ID (UUID format)
    ///
    /// # Returns
    /// The requested Users if found
    ///
    /// # Errors
    /// Returns `ServiceError::NotFound` if users don't exist
    pub async fn get_account_users(
        &self,
        id: &str,
        pagination: &PaginationFilter,
    ) -> ServiceResult<(Vec<User>, u64)> {
        let repo = UserRepository::new(self.pool);

        // Get total count first
        let total_count = repo.get_users_count_by_account_id(id).await?;

        if total_count == 0 {
            return Err(ServiceError::not_found("Users", id));
        }

        let users = repo.get_users_by_account_id(id, pagination).await?;

        Ok((users, total_count))
    }

    /// Authenticates a user with username and password.
    ///
    /// # Arguments
    /// * `username` - Username to authenticate
    /// * `password` - Plain text password to verify
    ///
    /// # Returns
    /// The authenticated User if credentials are valid
    ///
    /// # Errors
    /// Returns `ServiceError` for:
    /// - Invalid username or password
    /// - Inactive user accounts
    /// - Database errors
    pub async fn authenticate_user(&self, username: &str, password: &str) -> ServiceResult<User> {
        let repo = UserRepository::new(self.pool);

        // Get user by username
        let user = repo
            .get_user_by_username(username)
            .await?
            .ok_or_else(|| ServiceError::validation("Invalid username or password".to_string()))?;

        // Check if user is active
        if !user.is_active {
            return Err(ServiceError::validation(
                "User account is inactive".to_string(),
            ));
        }

        // Verify password
        if !self.verify_password(password, &user.password_hash)? {
            return Err(ServiceError::validation(
                "Invalid username or password".to_string(),
            ));
        }

        Ok(user)
    }

    /// Changes a user's role access.
    ///
    pub async fn change_user_role_access(&self, user_id: &str) -> ServiceResult<User> {
        let repo = UserRepository::new(self.pool);
        let mut user = repo
            .get_user_by_id(user_id)
            .await?
            .ok_or_else(|| ServiceError::not_found(" User", user_id))?;

        let role_repo = RoleRepository::new(self.pool);
        let role = role_repo.get_role_by_name("Admin").await?;
        if role.is_none() {
            return Err(ServiceError::not_found("Role", "Admin"));
        }

        let role = role.unwrap();

        if user.role_id == role.id {
            return Err(ServiceError::validation("User already has Admin role"));
        }

        match user.role_access_level {
            RoleAccessLevel::Read => {
                user.role_access_level = RoleAccessLevel::ReadWrite;
            }
            RoleAccessLevel::ReadWrite => {
                user.role_access_level = RoleAccessLevel::Read;
            }
        }

        // Update user in the database
        let rows_affected = sqlx::query!(
            r#"
            UPDATE users
            SET role_access_level = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ? AND is_deleted = 0
            "#,
            user.role_access_level,
            user.id
        )
        .execute(self.pool)
        .await
        .map_err(|e| ServiceError::Database { source: e.into() })?
        .rows_affected();

        // Check if the update actually affected any rows
        if rows_affected == 0 {
            return Err(ServiceError::validation("User role access not changed"));
        }

        Ok(user)
    }
}
