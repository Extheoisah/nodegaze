//! Core business logic for the authentication system.

use crate::auth::models::*;
use crate::config::Config;
use crate::errors::{ServiceError, ServiceResult};
use crate::repositories::account_repository::AccountRepository;
use crate::repositories::credential_repository::CredentialRepository;
use crate::services::user_service::UserService;
use crate::utils::jwt::{JwtUtils, NodeCredentials};
use sqlx::SqlitePool;
use validator::Validate;

/// Authentication service for handling login, token generation, and user management
pub struct AuthService<'a> {
    pool: &'a SqlitePool,
    jwt_utils: JwtUtils,
    user_service: UserService<'a>,
    config: Config,
}

impl<'a> AuthService<'a> {
    /// Create a new AuthService instance
    pub fn new(pool: &'a SqlitePool) -> ServiceResult<Self> {
        let jwt_utils = JwtUtils::new()?;
        let user_service = UserService::new(pool);
        let config = Config::from_env()?;

        Ok(AuthService {
            pool,
            jwt_utils,
            user_service,
            config,
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
        let _user_account_id = user.account_id.clone();
        let user_role_id = user.role_id.clone();
        let role_access_level = user.role_access_level.clone();

        // Check for existing node credentials and convert them to JWT format
        let credential_repo = CredentialRepository::new(self.pool);
        let node_credentials =
            if let Some(credential) = credential_repo.get_credential_by_account_id(&account_id).await? {
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
            role_access_level.clone(),
            node_credentials,
        )?;

        let refresh_token = self
            .jwt_utils
            .generate_refresh_token(user_id.clone(), role_access_level.clone())?;

        // Check if user has credentials for the response
        let has_node_credentials = credential_repo
            .get_credential_by_user_id(&user_id)
            .await?
            .is_some();

        // Get expires_in from config
        let expires_in = self.config.jwt_expires_in_seconds;

        let user_info = UserInfo {
            id: user_id,
            username: user.username,
            email: user.email,
            account_id,
            account_name: account.name,
            role: role_name,
            has_node_credentials,
        };

        Ok(LoginResponse {
            access_token,
            refresh_token,
            user: user_info,
            expires_in,
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
        let role_access_level = user.role_access_level.clone();

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
            role_access_level,
            node_credentials,
        )?;

        Ok(RefreshTokenResponse {
            access_token,
            expires_in: self.config.jwt_expires_in_seconds,
        })
    }

    /// Helper method to get user role name
    async fn get_user_role_name(&self, role_id: &str) -> ServiceResult<String> {
        let role_repo = crate::repositories::role_repository::RoleRepository::new(self.pool);
        let role = role_repo
            .get_role_by_id(role_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Role", role_id))?;

        Ok(role.name)
    }
}
