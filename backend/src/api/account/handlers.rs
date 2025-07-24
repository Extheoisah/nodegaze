//! Handler functions for account management API endpoints.
//!
//! These functions process requests for account data, interact with the database
//! or relevant services, and return account-specific information.

use crate::api::common::{
    ApiResponse, PaginatedData, PaginationFilter, PaginationMeta, service_error_to_http,
};
use crate::database::models::{Account, CreateNewAccount, User, UserWithAccount};
use crate::services::account_service::AccountService;
use crate::services::user_service::UserService;
use crate::utils::jwt::Claims;
use axum::extract::Query;
use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::Json as ResponseJson,
};
use sqlx::SqlitePool;

#[axum::debug_handler]
pub async fn create_account(
    Extension(pool): Extension<SqlitePool>,
    Json(payload): Json<CreateNewAccount>,
) -> Result<ResponseJson<ApiResponse<UserWithAccount>>, (StatusCode, String)> {
    tracing::info!("Creating new account with payload: {:?}", payload);

    let service = AccountService::new(&pool);

    match service.create_account(payload).await {
        Ok(account) => {
            tracing::debug!("Account created successfully: {:?}", account);
            Ok(ResponseJson(ApiResponse::success(
                account,
                "Account created successfully",
            )))
        }
        Err(error) => Err(service_error_to_http(error)),
    }
}

/// Retrieves an account by its ID.
#[axum::debug_handler]
pub async fn get_account(
    Extension(claims): Extension<Claims>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<ApiResponse<Account>>, (StatusCode, String)> {
    let account_id = claims.account_id.as_str().to_string();

    tracing::info!("Getting account Details: {}", account_id);

    let account_service = AccountService::new(&pool);
    let account = account_service
        .get_account_required(account_id.as_str())
        .await
        .map_err(|e| {
            tracing::error!("Account not found for ID {}: {}", account_id, e);
            let error_response = ApiResponse::<()>::error(
                "Account not found".to_string(),
                "account_not_found",
                None,
            );
            (
                StatusCode::NOT_FOUND,
                serde_json::to_string(&error_response).unwrap(),
            )
        })?;

    tracing::info!("Account found: {}", account.id);

    Ok(Json(ApiResponse::success(
        account,
        "Account retrieved successfully",
    )))
}

/// Retrieves an account admin user.
#[axum::debug_handler]
pub async fn get_account_admin_user(
    Extension(claims): Extension<Claims>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<ApiResponse<User>>, (StatusCode, String)> {
    let account_id = claims.account_id.as_str().to_string();
    let user_service = UserService::new(&pool);

    tracing::info!("Getting account admin user: {}", account_id);

    let user = user_service
        .get_admin_user_required(&account_id)
        .await
        .map_err(|e| {
            tracing::error!("Admin user not found for account ID {}: {}", account_id, e);
            let error_response = ApiResponse::<()>::error(
                "Admin user not found".to_string(),
                "admin_user_not_found",
                None,
            );
            (
                StatusCode::NOT_FOUND,
                serde_json::to_string(&error_response).unwrap(),
            )
        })?;

    tracing::info!("Admin user found: {}", user.id);

    Ok(Json(ApiResponse::success(
        user,
        "Admin user retrieved successfully",
    )))
}

/// Retrieves all users for an account.
#[axum::debug_handler]
pub async fn get_account_users(
    Extension(claims): Extension<Claims>,
    Extension(pool): Extension<SqlitePool>,
    Query(pagination): Query<PaginationFilter>,
) -> Result<Json<ApiResponse<PaginatedData<User>>>, (StatusCode, String)> {
    let account_id = claims.account_id.as_str().to_string();
    let user_service = UserService::new(&pool);

    tracing::info!(
        "Getting account users: {} with pagination: page={}, per_page={}",
        account_id,
        pagination.page(),
        pagination.per_page()
    );

    let (users, total_count) = user_service
        .get_account_users(&account_id, &pagination)
        .await
        .map_err(|e| {
            tracing::error!("Users not found for account ID {}: {}", account_id, e);
            let error_response =
                ApiResponse::<()>::error("Users not found".to_string(), "users_not_found", None);
            (
                StatusCode::NOT_FOUND,
                serde_json::to_string(&error_response).unwrap(),
            )
        })?;

    let paginated_data = PaginatedData::new(users, total_count);
    let pagination_meta = PaginationMeta::from_filter(&pagination, total_count);

    tracing::info!(
        "Users found: {} total, {} on current page",
        total_count,
        paginated_data.items.len()
    );

    Ok(Json(ApiResponse::ok_paginated(
        paginated_data,
        pagination_meta,
    )))
}
