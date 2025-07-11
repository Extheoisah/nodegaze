//! Credential business logic service.
//!
//! Handles all account-related business operations

use crate::config::Config;
use crate::database::models::{CreateCredential, Credential};
use crate::errors::{ServiceError, ServiceResult};
use crate::repositories::account_repository::AccountRepository;
use crate::repositories::credential_repository::CredentialRepository;
use crate::repositories::role_repository::RoleRepository;
use crate::repositories::user_repository::UserRepository;
use crate::utils::crypto::StringCrypto;
use sqlx::SqlitePool;
use validator::Validate;

pub struct CredentialService<'a> {
    /// Shared database connection pool
    pool: &'a SqlitePool,
}

impl<'a> CredentialService<'a> {
    /// Creates a new CredentialService instance.
    ///
    /// # Arguments
    /// * `pool` - Reference to SQLite connection pool
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Creates new node credentials with full validation.
    ///
    /// # Arguments
    /// * `create_credential` - Credential creation DTO
    ///
    /// # Returns
    /// The newly created Credential with all fields populated
    ///
    /// # Errors
    /// Returns `ServiceError` for:
    /// - Validation failures
    /// - Non-existent account/user
    /// - Database errors
    pub async fn create_credential(
        &self,
        create_credential: CreateCredential,
    ) -> ServiceResult<Credential> {
        // Input validation using validator crate
        if let Err(validation_errors) = create_credential.validate() {
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
            .get_account_by_id(&create_credential.account_id)
            .await?
            .is_none()
        {
            return Err(ServiceError::not_found(
                "Account",
                &create_credential.account_id,
            ));
        }

        // Check if the user exists
        let user_repo = UserRepository::new(self.pool);
        if user_repo
            .get_user_by_id(&create_credential.user_id)
            .await?
            .is_none()
        {
            return Err(ServiceError::not_found("User", &create_credential.user_id));
        }

        // Encrypt the credential value
        let config = Config::from_env().unwrap();
        let crypto = StringCrypto::new(&config.encryption_key)
            .map_err(|e| ServiceError::validation(&format!("Crypto initialization failed: {e}")))?;

        let encrypted_macaroon = crypto
            .encrypt(&create_credential.macaroon)
            .map_err(|e| ServiceError::validation(&format!("Encryption failed: {e}")))?;

        let encrypted_tls_cert = crypto
            .encrypt(&create_credential.tls_cert)
            .map_err(|e| ServiceError::validation(&format!("Encryption failed: {e}")))?;

        let new_credential = CreateCredential {
            macaroon: encrypted_macaroon,
            tls_cert: encrypted_tls_cert,
            account_id: create_credential.account_id,
            user_id: create_credential.user_id,
            node_alias: create_credential.node_alias,
            node_id: create_credential.node_id,
            address: create_credential.address,
        };

        // Create the credential
        let repo = CredentialRepository::new(self.pool);
        let credential = repo.create_credential(new_credential).await?;
        Ok(credential)
    }

    /// Retrieves credentials by ID with existence verification.
    ///
    /// # Arguments
    /// * `id` - Credential ID (UUID format)
    ///
    /// # Returns
    /// The requested Credential if found
    ///
    /// # Errors
    /// Returns `ServiceError::NotFound` if credential doesn't exist
    pub async fn get_credential_required(&self, id: &str) -> ServiceResult<Credential> {
        let repo = CredentialRepository::new(self.pool);
        let credential = repo
            .get_credential_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Credential", id))?;
        Ok(credential)
    }
}
