//! Handler functions for invite endpoints.
//!
//! These functions process requests for invite data, interact with the database
//! or relevant services, and return invite-specific information.

use crate::api::common::ApiResponse;
use crate::config::Config;
use crate::database::models::{AcceptInviteRequest, CreateInviteRequest, Invite, User};
use crate::services::invite_service::InviteService;
use crate::services::user_service::UserService;
use crate::utils::jwt::Claims;
use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
};
use sqlx::SqlitePool;

/// Handle invite creation request
#[axum::debug_handler]
pub async fn create_invite(
    Extension(pool): Extension<SqlitePool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateInviteRequest>,
) -> Result<Json<ApiResponse<Invite>>, (StatusCode, String)> {
    let config = Config::from_env().unwrap();
    let user_id = claims.sub.as_str().to_string();

    tracing::info!("Creating invite for user: {}", user_id);

    let user_service = UserService::new(&pool);
    let user = user_service
        .get_user_required(user_id.as_str())
        .await
        .map_err(|e| {
            tracing::error!("User not found for ID {}: {}", user_id, e);
            let error_response =
                ApiResponse::<()>::error("User not found".to_string(), "user_not_found", None);
            (
                StatusCode::NOT_FOUND,
                serde_json::to_string(&error_response).unwrap(),
            )
        })?;

    let service = InviteService::new(&pool, &config);

    let invite = service.create_invite(payload, user).await.map_err(|e| {
        tracing::error!("Failed to create invite for user {}: {}", user_id, e);
        let error_response = ApiResponse::<()>::error(
            format!("Failed to create invite: {}", e),
            "invite_creation_error",
            None,
        );
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::to_string(&error_response).unwrap(),
        )
    })?;

    tracing::info!("Invite created successfully: {}", invite.id);
    Ok(Json(ApiResponse::success(
        invite,
        "Invite created successfully",
    )))
}

/// Retrieves a invite by its ID.
#[axum::debug_handler]
pub async fn get_invite_by_id(
    Extension(claims): Extension<Claims>,
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Invite>>, (StatusCode, String)> {
    let config = Config::from_env().unwrap();
    let user_id = claims.sub.as_str().to_string();

    tracing::info!("Getting invite by ID: {} for user: {}", id, user_id);

    let user_service = UserService::new(&pool);
    let user = user_service
        .get_user_required(user_id.as_str())
        .await
        .map_err(|e| {
            tracing::error!("User not found for ID {}: {}", user_id, e);
            let error_response =
                ApiResponse::<()>::error("User not found".to_string(), "user_not_found", None);
            (
                StatusCode::NOT_FOUND,
                serde_json::to_string(&error_response).unwrap(),
            )
        })?;

    let service = InviteService::new(&pool, &config);
    let invite = service
        .get_invite_required(&id, &user.account_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to find invite {}: {}", id, e);
            let error_response = ApiResponse::<()>::error(
                format!("Failed to find invite: {}", e),
                "invite_not_found",
                None,
            );
            (
                StatusCode::NOT_FOUND,
                serde_json::to_string(&error_response).unwrap(),
            )
        })?;

    tracing::info!("Invite found: {}", invite.id);
    Ok(Json(ApiResponse::success(
        invite,
        "Invite retrieved successfully",
    )))
}

/// Retrieves all invites for the user's account.
#[axum::debug_handler]
pub async fn get_invites(
    Extension(claims): Extension<Claims>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<ApiResponse<Vec<Invite>>>, (StatusCode, String)> {
    let config = Config::from_env().unwrap();
    let user_id = claims.sub.as_str().to_string();

    tracing::info!("Getting all invites for user: {}", user_id);

    let user_service = UserService::new(&pool);
    let user = user_service
        .get_user_required(user_id.as_str())
        .await
        .map_err(|e| {
            tracing::error!("User not found for ID {}: {}", user_id, e);
            let error_response =
                ApiResponse::<()>::error("User not found".to_string(), "user_not_found", None);
            (
                StatusCode::NOT_FOUND,
                serde_json::to_string(&error_response).unwrap(),
            )
        })?;

    let service = InviteService::new(&pool, &config);
    let invites = service
        .get_invites_by_account_id(&user.account_id)
        .await
        .map_err(|e| {
            tracing::error!("No invites found for account {}: {}", user.account_id, e);
            let error_response = ApiResponse::<()>::error(
                format!("No invites found: {}", e),
                "invites_not_found",
                None,
            );
            (
                StatusCode::NOT_FOUND,
                serde_json::to_string(&error_response).unwrap(),
            )
        })?;

    tracing::info!("Found {} invites for user: {}", invites.len(), user_id);
    Ok(Json(ApiResponse::success(
        invites,
        "Invites retrieved successfully",
    )))
}

/// Resends an invite to the invitee's email.
#[axum::debug_handler]
pub async fn resend_invite(
    Extension(pool): Extension<SqlitePool>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Invite>>, (StatusCode, String)> {
    let config = Config::from_env().unwrap();
    let user_id = claims.sub.as_str().to_string();

    tracing::info!("Resending invite {} for user: {}", id, user_id);

    let user_service = UserService::new(&pool);
    let user = user_service
        .get_user_required(user_id.as_str())
        .await
        .map_err(|e| {
            tracing::error!("User not found for ID {}: {}", user_id, e);
            let error_response =
                ApiResponse::<()>::error("User not found".to_string(), "user_not_found", None);
            (
                StatusCode::NOT_FOUND,
                serde_json::to_string(&error_response).unwrap(),
            )
        })?;

    let service = InviteService::new(&pool, &config);
    let invite = service.resend_invite(&id, &user).await.map_err(|e| {
        tracing::error!("Failed to resend invite {} for user {}: {}", id, user_id, e);
        let error_response = ApiResponse::<()>::error(
            format!("Failed to resend invite: {}", e),
            "invite_resend_error",
            None,
        );
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::to_string(&error_response).unwrap(),
        )
    })?;

    tracing::info!("Invite resent successfully: {}", invite.id);
    Ok(Json(ApiResponse::success(
        invite,
        "Invite resent successfully",
    )))
}

/// Accepts an invite for the invited user.
#[axum::debug_handler]
pub async fn accept_invite(
    Extension(pool): Extension<SqlitePool>,
    Json(accept_invite): Json<AcceptInviteRequest>,
) -> Result<Json<ApiResponse<User>>, (StatusCode, String)> {
    let config = Config::from_env().unwrap();

    tracing::info!("Accepting invite for token: {}", accept_invite.token);

    let service = InviteService::new(&pool, &config);
    let user = service.accept_invite(&accept_invite).await.map_err(|e| {
        tracing::error!(
            "Failed to accept invitation for token {}: {}",
            accept_invite.token,
            e
        );
        let error_response = ApiResponse::<()>::error(
            format!("Failed to accept invitation: {}", e),
            "invite_accept_error",
            None,
        );
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::to_string(&error_response).unwrap(),
        )
    })?;

    tracing::info!(
        "Invite accepted successfully for token: {}",
        accept_invite.token
    );
    Ok(Json(ApiResponse::success(
        user,
        "Invite accepted successfully",
    )))
}
