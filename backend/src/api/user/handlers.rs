//! Handler functions for user profile and management API endpoints.
//!
//! These functions process requests for user data, interact with the database
//! or relevant services, and return user-specific information.

use crate::api::common::ApiResponse;
use crate::database::models::{User, UserWithAccount};
use crate::services::user_service::UserService;
use crate::utils::jwt::Claims;
use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
};
use sqlx::SqlitePool;

/// Retrieves a user by its ID.
#[axum::debug_handler]
pub async fn get_user_by_id(
    Extension(claims): Extension<Claims>,
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<User>>, (StatusCode, String)> {
    let user_id = claims.sub.as_str().to_string();

    tracing::info!("Getting user by ID: {} for user: {}", id, user_id);

    let user_service = UserService::new(&pool);
    let user = user_service
        .get_user_required(&id.as_str())
        .await
        .map_err(|e| {
            tracing::error!("User not found for ID {}: {}", id, e);
            let error_response =
                ApiResponse::<()>::error("User not found".to_string(), "user_not_found", None);
            (
                StatusCode::NOT_FOUND,
                serde_json::to_string(&error_response).unwrap(),
            )
        })?;

    tracing::info!("User found: {}", user.id);
    Ok(Json(ApiResponse::success(
        user,
        "User retrieved successfully",
    )))
}

/// Changes a user role access level.
#[axum::debug_handler]
pub async fn change_user_role_access_level(
    Extension(claims): Extension<Claims>,
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<User>>, (StatusCode, String)> {
    let user_role = claims.role.as_str().to_string();
    if user_role != "Admin" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only Admin users can change role access levels".to_string(),
        ));
    }

    let user_id = claims.sub.as_str().to_string();

    tracing::info!(
        "Changing role access level for user: {} by admin: {}",
        id,
        user_id
    );

    let user_service = UserService::new(&pool);
    let user = user_service
        .change_user_role_access(&id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to change role access level for ID {}: {}", id, e);
            let error_response = ApiResponse::<()>::error(
                "Failed to change role access level".to_string(),
                "role_change_failed",
                None,
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                serde_json::to_string(&error_response).unwrap(),
            )
        })?;

    Ok(Json(ApiResponse::success(
        user,
        "User role access level changed successfully",
    )))
}
