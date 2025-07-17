//! Handler functions for account management API endpoints.
//!
//! These functions process requests for account data, interact with the database
//! or relevant services, and return account-specific information.

use crate::api::common::{ApiResponse, service_error_to_http};
use crate::database::models::{CreateNewAccount, UserWithAccount};
use crate::services::account_service::AccountService;
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
    let service = AccountService::new(&pool);

    match service.create_account(payload).await {
        Ok(account) => Ok(ResponseJson(ApiResponse::success(
            account,
            "Account created successfully",
        ))),
        Err(error) => Err(service_error_to_http(error)),
    }
}
