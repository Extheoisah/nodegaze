//! Handler functions for authentication-related API endpoints.
//!
//! These functions process incoming HTTP requests for user authentication (login, registration,
//! token refresh), parse request data, validate input, and interact with the
//! `auth::service` for core business logic.

use crate::api::common::service_error_to_http;
use crate::auth::models::*;
use crate::auth::service::AuthService;
use crate::repositories::credential_repository::CredentialRepository;
use crate::utils::jwt::Claims;
use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::Json as ResponseJson,
};
use sqlx::SqlitePool;

/// Handle user login request
#[axum::debug_handler]
pub async fn login(
    Extension(pool): Extension<SqlitePool>,
    Json(payload): Json<LoginRequest>,
) -> Result<ResponseJson<LoginResponse>, (StatusCode, String)> {
    let auth_service = match AuthService::new(&pool) {
        Ok(service) => service,
        Err(error) => return Err(service_error_to_http(error)),
    };

    match auth_service.login(payload).await {
        Ok(response) => Ok(ResponseJson(response)),
        Err(error) => Err(service_error_to_http(error)),
    }
}

/// Handle token refresh request
#[axum::debug_handler]
pub async fn refresh_token(
    Extension(pool): Extension<SqlitePool>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<ResponseJson<RefreshTokenResponse>, (StatusCode, String)> {
    let auth_service = match AuthService::new(&pool) {
        Ok(service) => service,
        Err(error) => return Err(service_error_to_http(error)),
    };

    match auth_service.refresh_token(payload).await {
        Ok(response) => Ok(ResponseJson(response)),
        Err(error) => Err(service_error_to_http(error)),
    }
}

/// Handle node credentials storage request
#[axum::debug_handler]
pub async fn store_node_credentials(
    Extension(pool): Extension<SqlitePool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<StoreNodeCredentialsRequest>,
) -> Result<ResponseJson<StoreNodeCredentialsResponse>, (StatusCode, String)> {
    let auth_service = match AuthService::new(&pool) {
        Ok(service) => service,
        Err(error) => return Err(service_error_to_http(error)),
    };

    match auth_service.store_node_credentials(claims, payload).await {
        Ok(response) => Ok(ResponseJson(response)),
        Err(error) => Err(service_error_to_http(error)),
    }
}

/// Handle node credentials revocation request
#[axum::debug_handler]
pub async fn revoke_node_credentials(
    Extension(pool): Extension<SqlitePool>,
    Extension(claims): Extension<Claims>,
) -> Result<ResponseJson<RevokeNodeCredentialsResponse>, (StatusCode, String)> {
    let credential_repo = CredentialRepository::new(&pool);

    // Check if user has credentials to revoke
    let credential = match credential_repo.get_credential_by_user_id(&claims.sub).await {
        Ok(Some(cred)) => cred,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                "No node credentials found".to_string(),
            ));
        }
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    };

    // Soft delete the credential
    if let Err(e) = credential_repo.delete_credential(&credential.id).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
    }

    // Generate new token without node credentials
    let jwt_utils = match crate::utils::jwt::JwtUtils::new() {
        Ok(utils) => utils,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("JWT error: {}", e),
            ));
        }
    };

    let access_token = match jwt_utils.generate_token(
        claims.sub,
        claims.account_id,
        claims.role,
        None, // No node credentials
    ) {
        Ok(token) => token,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Token generation failed: {}", e),
            ));
        }
    };

    let response = RevokeNodeCredentialsResponse {
        access_token,
        revoked: true,
        expires_in: 24 * 60 * 60,
    };

    Ok(ResponseJson(response))
}

/// Handle logout request (client-side token invalidation)
#[axum::debug_handler]
pub async fn logout() -> Result<ResponseJson<serde_json::Value>, (StatusCode, String)> {
    // For JWT tokens, logout is typically handled on the client side
    // by removing the token from storage. The server can maintain a blacklist
    // if we later need for enhanced security.
    Ok(ResponseJson(serde_json::json!({
        "message": "Logged out successfully"
    })))
}

/// Get current user information from token
#[axum::debug_handler]
pub async fn me(
    Extension(pool): Extension<SqlitePool>,
    Extension(claims): Extension<Claims>,
) -> Result<ResponseJson<UserInfo>, (StatusCode, String)> {
    // Get user information from database using claims
    let user = match sqlx::query!(
        r#"
        SELECT u.name, u.email, a.name as account_name, r.name as role_name
        FROM users u
        JOIN accounts a ON u.account_id = a.id
        JOIN roles r ON u.role_id = r.id
        WHERE u.id = ? AND u.is_deleted = 0
        "#,
        claims.sub
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(user)) => user,
        Ok(None) => return Err((StatusCode::NOT_FOUND, "User not found".to_string())),
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    };

    let user_info = UserInfo {
        id: claims.sub.clone(),
        name: user.name,
        email: user.email,
        account_id: claims.account_id.clone(),
        account_name: user.account_name,
        role: user.role_name,
        has_node_credentials: claims.has_node_credentials(),
    };

    Ok(ResponseJson(user_info))
}
