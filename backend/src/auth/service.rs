//! Core business logic for the authentication system.

use crate::auth::models::*;
use crate::config::Config;
use crate::errors::{ServiceError, ServiceResult};
use crate::repositories::account_repository::AccountRepository;
use crate::repositories::credential_repository::CredentialRepository;
use crate::services::user_service::UserService;
use crate::utils::jwt::{Claims, JwtUtils, NodeCredentials};
use sqlx::SqlitePool;
use validator::Validate;

/// Authentication service for handling login, token generation, and user management
pub struct AuthService<'a> {
    pool: &'a SqlitePool,
    jwt_utils: JwtUtils,
    user_service: UserService<'a>,
}

impl<'a> AuthService<'a> {
    /// Create a new AuthService instance
    pub fn new(pool: &'a SqlitePool) -> ServiceResult<Self> {
        let jwt_utils = JwtUtils::new()?;
        let user_service = UserService::new(pool);

        Ok(AuthService {
            pool,
            jwt_utils,
            user_service,
        })
    }

    /// Authenticate user and generate JWT tokens with node credentials if available
    pub async fn login(&self, login_request: LoginRequest) -> ServiceResult<LoginResponse> {
        // Validate input
        if let Err(validation_errors) = login_request.validate() {
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

        // Authenticate user using UserService
        let user = self
            .user_service
            .authenticate_user(&login_request.username, &login_request.password)
            .await?;

        // Get account information
        let account_repo = AccountRepository::new(self.pool);
        let account = account_repo
            .get_account_by_id(&user.account_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Account", &user.account_id))?;

        // Check if account is active
        if !account.is_active {
            return Err(ServiceError::validation("Account is inactive".to_string()));
        }

        // Store user ID before potential moves
        let user_id = user.id.clone();
        let account_id = account.id.clone();
        let user_account_id = user.account_id.clone();
        let user_role_id = user.role_id.clone();

        // Check for existing node credentials and convert them to JWT format
        let credential_repo = CredentialRepository::new(self.pool);
        let node_credentials =
            if let Some(credential) = credential_repo.get_credential_by_user_id(&user_id).await? {
                Some(NodeCredentials {
                    node_id: credential.node_id,
                    node_alias: credential.node_alias,
                    node_type: credential.node_type.unwrap_or_else(|| "lnd".to_string()),
                    macaroon: credential.macaroon,
                    tls_cert: credential.tls_cert,
                    client_cert: credential.client_cert,
                    client_key: credential.client_key,
                    ca_cert: credential.ca_cert,
                    address: credential.address,
                })
            } else {
                None
            };

        // Get user role name
        let role_name = self.get_user_role_name(&user_role_id).await?;

        // Generate tokens with node credentials if available
        let access_token = self.jwt_utils.generate_token(
            user_id.clone(),
            account_id.clone(),
            role_name.clone(),
            node_credentials,
        )?;

        let refresh_token = self.jwt_utils.generate_refresh_token(user_id.clone())?;

        // Check if user has credentials for the response
        let has_node_credentials = credential_repo
            .get_credential_by_user_id(&user_id)
            .await?
            .is_some();

        let user_info = UserInfo {
            id: user_id,
            name: user.name,
            email: user.email,
            account_id: account_id,
            account_name: account.name,
            role: role_name,
            has_node_credentials,
        };

        Ok(LoginResponse {
            access_token,
            refresh_token,
            user: user_info,
            expires_in: 24 * 60 * 60, // 24 hours in seconds
        })
    }

    /// Store node credentials in database after authentication
    pub async fn store_node_credentials(
        &self,
        claims: Claims,
        request: StoreNodeCredentialsRequest,
    ) -> ServiceResult<StoreNodeCredentialsResponse> {
        // Validate input
        if let Err(validation_errors) = request.validate() {
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

        let credential_repo = CredentialRepository::new(self.pool);

        // Check if user already has credentials stored
        if credential_repo
            .get_credential_by_user_id(&claims.sub)
            .await?
            .is_some()
        {
            return Err(ServiceError::already_exists(
                "Node credentials for user",
                &claims.sub,
            ));
        }

        // Create credential record with all required fields
        let create_credential = crate::database::models::CreateCredential {
            user_id: claims.sub.clone(),
            account_id: claims.account_id.clone(),
            node_id: request.node_id.clone(),
            node_alias: request.node_alias.clone(),
            macaroon: request.macaroon.clone(),
            tls_cert: request.tls_cert.clone(),
            address: request.address.clone(),
            node_type: Some(request.node_type.clone()),
            client_cert: request.client_cert.clone(),
            client_key: request.client_key.clone(),
            ca_cert: request.ca_cert.clone(),
        };

        // Store in database
        let credential = credential_repo.create_credential(create_credential).await?;

        // Create node credentials for new token
        let node_credentials = NodeCredentials {
            node_id: request.node_id,
            node_alias: request.node_alias,
            node_type: request.node_type,
            macaroon: request.macaroon,
            tls_cert: request.tls_cert,
            client_cert: request.client_cert,
            client_key: request.client_key,
            ca_cert: request.ca_cert,
            address: request.address,
        };

        // Generate new token with node credentials
        let access_token = self.jwt_utils.generate_token(
            claims.sub,
            claims.account_id,
            claims.role,
            Some(node_credentials),
        )?;

        Ok(StoreNodeCredentialsResponse {
            access_token,
            credential_id: credential.id,
            expires_in: 24 * 60 * 60, // 24 hours
        })
    }

    /// Revoke stored node credentials for a user
    pub async fn revoke_node_credentials(
        &self,
        claims: Claims,
    ) -> ServiceResult<RevokeNodeCredentialsResponse> {
        let credential_repo = CredentialRepository::new(self.pool);

        // Check if user has credentials to revoke
        let credential = credential_repo
            .get_credential_by_user_id(&claims.sub)
            .await?
            .ok_or_else(|| ServiceError::not_found("Node credentials", &claims.sub))?;

        // Soft delete the credential
        credential_repo.delete_credential(&credential.id).await?;

        // Generate new token without node credentials
        let access_token = self.jwt_utils.generate_token(
            claims.sub,
            claims.account_id,
            claims.role,
            None, // No node credentials
        )?;

        Ok(RevokeNodeCredentialsResponse {
            access_token,
            revoked: true,
            expires_in: 24 * 60 * 60,
        })
    }

    /// Refresh access token with existing node credentials
    pub async fn refresh_token(
        &self,
        request: RefreshTokenRequest,
    ) -> ServiceResult<RefreshTokenResponse> {
        // Validate refresh token
        let claims = self.jwt_utils.validate_token(&request.refresh_token)?;

        // Get user to ensure they still exist and are active
        let user = self.user_service.get_user_required(&claims.sub).await?;

        if !user.is_active {
            return Err(ServiceError::validation(
                "User account is inactive".to_string(),
            ));
        }

        // Store needed values before potential moves
        let user_id = user.id.clone();
        let user_account_id = user.account_id.clone();
        let user_role_id = user.role_id.clone();

        // Check for existing node credentials
        let credential_repo = CredentialRepository::new(self.pool);
        let node_credentials =
            if let Some(credential) = credential_repo.get_credential_by_user_id(&user_id).await? {
                Some(NodeCredentials {
                    node_id: credential.node_id,
                    node_alias: credential.node_alias,
                    node_type: credential.node_type.unwrap_or_else(|| "lnd".to_string()),
                    macaroon: credential.macaroon,
                    tls_cert: credential.tls_cert,
                    client_cert: credential.client_cert,
                    client_key: credential.client_key,
                    ca_cert: credential.ca_cert,
                    address: credential.address,
                })
            } else {
                None
            };

        // Generate new access token with node credentials if available
        let access_token = self.jwt_utils.generate_token(
            user_id,
            user_account_id,
            self.get_user_role_name(&user_role_id).await?,
            node_credentials,
        )?;

        Ok(RefreshTokenResponse {
            access_token,
            expires_in: 24 * 60 * 60, // 24 hours
        })
    }

    /// Helper method to determine node type from macaroon or other stored data
    fn determine_node_type(&self, _macaroon: &str) -> ServiceResult<String> {
        // For now, return a default. You might want to store this explicitly in the database
        // or implement logic to detect from the macaroon format
        Ok("lnd".to_string())
    }

    /// Helper method to get user role name
    async fn get_user_role_name(&self, role_id: &str) -> ServiceResult<String> {
        let role = sqlx::query!(
            "SELECT name FROM roles WHERE id = ? AND is_deleted = 0",
            role_id
        )
        .fetch_optional(self.pool)
        .await
        .map_err(|e| ServiceError::Database { source: e.into() })?
        .ok_or_else(|| ServiceError::not_found("Role", role_id))?;

        Ok(role.name)
    }
}
