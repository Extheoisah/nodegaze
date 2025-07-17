//! Handler functions for authentication-related API endpoints.
//!
//! These functions process incoming HTTP requests for user authentication (login, registration,
//! token refresh), parse request data, validate input, and interact with the
//! `auth::service` for core business logic.

use crate::api::common::{ApiResponse, service_error_to_http};
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
) -> Result<ResponseJson<ApiResponse<LoginResponse>>, (StatusCode, String)> {
    let auth_service = match AuthService::new(&pool) {
        Ok(service) => service,
        Err(error) => return Err(service_error_to_http(error)),
    };

    match auth_service.login(payload).await {
        Ok(response) => Ok(ResponseJson(ApiResponse::success(
            response,
            "Login successful",
        ))),
        Err(error) => Err(service_error_to_http(error)),
    }
}

/// Handle token refresh request
#[axum::debug_handler]
pub async fn refresh_token(
    Extension(pool): Extension<SqlitePool>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<ResponseJson<ApiResponse<RefreshTokenResponse>>, (StatusCode, String)> {
    let auth_service = match AuthService::new(&pool) {
        Ok(service) => service,
        Err(error) => return Err(service_error_to_http(error)),
    };

    match auth_service.refresh_token(payload).await {
        Ok(response) => Ok(ResponseJson(ApiResponse::success(
            response,
            "Token refreshed successfully",
        ))),
        Err(error) => Err(service_error_to_http(error)),
    }
}

/// Handle logout request (client-side token invalidation)
#[axum::debug_handler]
pub async fn logout() -> Result<ResponseJson<ApiResponse<serde_json::Value>>, (StatusCode, String)>
{
    Ok(ResponseJson(ApiResponse::success(
        serde_json::json!({ "logged_out": true }),
        "Logged out successfully",
    )))
}

/// Get current user information from token
#[axum::debug_handler]
pub async fn me(
    Extension(pool): Extension<SqlitePool>,
    Extension(claims): Extension<Claims>,
) -> Result<ResponseJson<ApiResponse<UserInfo>>, (StatusCode, String)> {
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
        Ok(None) => {
            let error_response = ApiResponse::<()>::error("User not found", "not_found", None);
            return Err((
                StatusCode::NOT_FOUND,
                serde_json::to_string(&error_response).unwrap(),
            ));
        }
        Err(e) => {
            let error_response = ApiResponse::<()>::error("Database error", "database_error", None);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                serde_json::to_string(&error_response).unwrap(),
            ));
        }
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

    Ok(ResponseJson(ApiResponse::success(
        user_info,
        "User information retrieved successfully",
    )))
}

/// Handle node credentials revocation request
#[axum::debug_handler]
pub async fn revoke_node_credentials(
    Extension(pool): Extension<SqlitePool>,
    Extension(claims): Extension<Claims>,
) -> Result<ResponseJson<ApiResponse<RevokeNodeCredentialsResponse>>, (StatusCode, String)> {
    let credential_repo = CredentialRepository::new(&pool);

    // Check if user has credentials to revoke
    let credential = match credential_repo.get_credential_by_user_id(&claims.sub).await {
        Ok(Some(cred)) => cred,
        Ok(None) => {
            let error_response =
                ApiResponse::<()>::error("No node credentials found", "not_found", None);
            return Err((
                StatusCode::NOT_FOUND,
                serde_json::to_string(&error_response).unwrap(),
            ));
        }
        Err(e) => {
            let error_response = ApiResponse::<()>::error("Database error", "database_error", None);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                serde_json::to_string(&error_response).unwrap(),
            ));
        }
    };

    // Soft delete the credential
    if let Err(e) = credential_repo.delete_credential(&credential.id).await {
        let error_response =
            ApiResponse::<()>::error("Failed to revoke credentials", "database_error", None);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::to_string(&error_response).unwrap(),
        ));
    }

    // Generate new token without node credentials
    let jwt_utils = match crate::utils::jwt::JwtUtils::new() {
        Ok(utils) => utils,
        Err(e) => {
            let error_response =
                ApiResponse::<()>::error("JWT configuration error", "configuration_error", None);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                serde_json::to_string(&error_response).unwrap(),
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
            let error_response =
                ApiResponse::<()>::error("Token generation failed", "token_error", None);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                serde_json::to_string(&error_response).unwrap(),
            ));
        }
    };

    let response = RevokeNodeCredentialsResponse {
        access_token,
        revoked: true,
        expires_in: 24 * 60 * 60,
    };

    Ok(ResponseJson(ApiResponse::success(
        response,
        "Node credentials revoked successfully",
    )))
}
