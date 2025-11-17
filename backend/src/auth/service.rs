//! Core business logic for the authentication system.

use crate::api::common::ApiResponse;
use crate::auth::models::*;
use crate::config::Config;
use crate::database::models::Credential;
use crate::errors::{ServiceError, ServiceResult};
use crate::repositories::account_repository::AccountRepository;
use crate::repositories::credential_repository::CredentialRepository;
use crate::services::node_manager::ConnectionRequest;
use crate::services::node_manager::{ClnConnection, LndConnection};
use crate::services::user_service::UserService;
use crate::utils::NodeId;
use crate::utils::crypto::StringCrypto;
use crate::utils::handlers_common::connect_to_event_stream;
use crate::utils::handlers_common::extract_cln_tls_components;
use crate::utils::handlers_common::parse_public_key;
use crate::utils::jwt::JwtUtils;
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
        let node_credential_id = if let Some(node_credentials) = credential_repo
            .get_credential_by_account_id(&account_id)
            .await?
        {
            self.authenticate_event_handler(&node_credentials, &self.pool, &account.id, &user_id)
                .await?;

            Some(node_credentials.id)
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
            node_credential_id,
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

    async fn authenticate_event_handler(
        &self,
        node_credentials: &Credential,
        pool: &SqlitePool,
        account_id: &String,
        user_id: &String,
    ) -> ServiceResult<()> {
        let public_key = parse_public_key(&node_credentials.node_id).unwrap();
        let tls_cert = StringCrypto::decrypt(&node_credentials.tls_cert).unwrap();
        let macaroon = StringCrypto::decrypt(&node_credentials.macaroon).unwrap();

        match node_credentials.node_type.as_deref() {
            Some("lnd") => {
                let connection_payload = ConnectionRequest::Lnd(LndConnection {
                    id: NodeId::PublicKey(public_key),
                    address: node_credentials.address.clone(),
                    macaroon: macaroon.clone(),
                    cert: tls_cert.clone(),
                });

                connect_to_event_stream(
                    &connection_payload,
                    &Some(account_id.clone()),
                    &pool,
                    &Some(user_id.clone()),
                )
                .await
                .unwrap();
                Ok(())
            }
            Some("cln") => {
                tracing::info!(
                    "Attempting to authenticate CLN node: {:?}",
                    node_credentials.node_id
                );
                let (client_cert, client_key, ca_cert) =
                    extract_cln_tls_components(&node_credentials.id, pool)
                        .await
                        .unwrap();

                let connection_payload = ConnectionRequest::Cln(ClnConnection {
                    id: NodeId::PublicKey(public_key),
                    address: node_credentials.address.clone(),
                    ca_cert,
                    client_cert,
                    client_key,
                });
                connect_to_event_stream(
                    &connection_payload,
                    &Some(account_id.clone()),
                    &pool,
                    &Some(user_id.clone()),
                )
                .await
                .unwrap();
                Ok(())
            }
            Some(_) => {
                let error_response = ApiResponse::<()>::error(
                    "Unsupported node type".to_string(),
                    "unsupported_node_type",
                    None,
                );
                Err(ServiceError::invalid_operation(
                    serde_json::to_string(&error_response).unwrap(),
                ))
            }
            None => {
                let error_response = ApiResponse::<()>::error(
                    "Node type not specified".to_string(),
                    "missing_node_type",
                    None,
                );
                Err(ServiceError::invalid_operation(
                    serde_json::to_string(&error_response).unwrap(),
                ))
            }
        }
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
        let node_credential_id =
            if let Some(credential) = credential_repo.get_credential_by_user_id(&user_id).await? {
                Some(credential.id)
            } else {
                None
            };

        // Generate new access token with node credentials if available
        let access_token = self.jwt_utils.generate_token(
            user_id,
            user_account_id,
            self.get_user_role_name(&user_role_id).await?,
            role_access_level,
            node_credential_id,
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
