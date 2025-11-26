//! Credential business logic service.
//!
//! Handles all credential-related business operations

use crate::database::models::Credential;
use crate::errors::{ServiceError, ServiceResult};
use crate::repositories::credential_repository::CredentialRepository;
use crate::utils::crypto::StringCrypto;
use sqlx::SqlitePool;

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
        let mut credential = repo
            .get_credential_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Credential", id))?;

        credential.tls_cert = StringCrypto::decrypt(&credential.tls_cert).unwrap();
        credential.macaroon = StringCrypto::decrypt(&credential.macaroon).unwrap();
        Ok(credential)
    }
}
