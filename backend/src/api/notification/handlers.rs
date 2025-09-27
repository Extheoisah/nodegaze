//! Handler functions for notification management API endpoints.

use crate::api::common::{
    ApiResponse, PaginatedData, PaginationFilter, PaginationMeta, service_error_to_http,
};
use crate::database::models::{
    CreateNotificationRequest, EventResponse, Notification, UpdateNotificationRequest,
};
use crate::services::notification_service::NotificationService;
use crate::services::user_service::UserService;
use crate::utils::jwt::Claims;
use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::Json as ResponseJson,
};
use sqlx::SqlitePool;

/// Creates a new notification.
#[axum::debug_handler]
pub async fn create_notification(
    Extension(pool): Extension<SqlitePool>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateNotificationRequest>,
) -> Result<ResponseJson<ApiResponse<Notification>>, (StatusCode, String)> {
    let user_id = claims.sub.as_str();

    // Get user details
    let user_service = UserService::new(&pool);
    let user = user_service.get_user_required(user_id).await.map_err(|e| {
        tracing::error!("User not found for ID {}: {}", user_id, e);
        let error_response = ApiResponse::<()>::error("User not found", "user_not_found", None);
        (
            StatusCode::NOT_FOUND,
            serde_json::to_string(&error_response).unwrap(),
        )
    })?;

    let service = NotificationService::new(&pool);
    match service.create_notification(payload, &user).await {
        Ok(notification) => Ok(ResponseJson(ApiResponse::success(
            notification,
            "Notification created successfully",
        ))),
        Err(error) => Err(service_error_to_http(error)),
    }
}

/// Retrieves all notifications for the user's account.
#[axum::debug_handler]
pub async fn get_notifications(
    Extension(pool): Extension<SqlitePool>,
    Extension(claims): Extension<Claims>,
) -> Result<ResponseJson<ApiResponse<Vec<Notification>>>, (StatusCode, String)> {
    let account_id = claims.account_id();

    let service = NotificationService::new(&pool);
    match service.get_notifications_for_account(account_id).await {
        Ok(notifications) => Ok(ResponseJson(ApiResponse::success(
            notifications,
            "Notifications retrieved successfully",
        ))),
        Err(error) => Err(service_error_to_http(error)),
    }
}

/// Retrieves a notification by ID.
#[axum::debug_handler]
pub async fn get_notification_by_id(
    Extension(pool): Extension<SqlitePool>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<ResponseJson<ApiResponse<Notification>>, (StatusCode, String)> {
    let account_id = claims.account_id();

    let service = NotificationService::new(&pool);
    match service.get_notification_required(&id, account_id).await {
        Ok(notification) => Ok(ResponseJson(ApiResponse::success(
            notification,
            "Notification retrieved successfully",
        ))),
        Err(error) => Err(service_error_to_http(error)),
    }
}

/// Updates a notification.
#[axum::debug_handler]
pub async fn update_notification(
    Extension(pool): Extension<SqlitePool>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateNotificationRequest>,
) -> Result<ResponseJson<ApiResponse<Notification>>, (StatusCode, String)> {
    let account_id = claims.account_id();

    let service = NotificationService::new(&pool);
    match service.update_notification(&id, payload, account_id).await {
        Ok(notification) => Ok(ResponseJson(ApiResponse::success(
            notification,
            "Notification updated successfully",
        ))),
        Err(error) => Err(service_error_to_http(error)),
    }
}

/// Deletes a notification.
#[axum::debug_handler]
pub async fn delete_notification(
    Extension(pool): Extension<SqlitePool>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<ResponseJson<ApiResponse<()>>, (StatusCode, String)> {
    let account_id = claims.account_id();

    let service = NotificationService::new(&pool);
    match service.delete_notification(&id, account_id).await {
        Ok(_) => Ok(ResponseJson(ApiResponse::success(
            (),
            "Notification deleted successfully",
        ))),
        Err(error) => Err(service_error_to_http(error)),
    }
}

/// Retrieves events for a specific notification endpoint.
#[axum::debug_handler]
pub async fn get_notification_events(
    Extension(pool): Extension<SqlitePool>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Query(pagination): Query<PaginationFilter>,
) -> Result<ResponseJson<ApiResponse<PaginatedData<EventResponse>>>, (StatusCode, String)> {
    let account_id = claims.account_id();

    let service = NotificationService::new(&pool);

    // Get events for the notification
    match service
        .get_events_for_notification(
            &id,
            account_id,
            Some(pagination.limit()),
            Some(pagination.offset()),
        )
        .await
    {
        Ok(events) => {
            // Get total count
            let total_count = service
                .count_events_for_notification(&id, account_id)
                .await
                .map_err(service_error_to_http)?;

            let paginated_data = PaginatedData::new(events, total_count as u64);
            let pagination_meta = PaginationMeta::from_filter(&pagination, total_count as u64);

            Ok(ResponseJson(ApiResponse::ok_paginated(
                paginated_data,
                pagination_meta,
            )))
        }
        Err(error) => Err(service_error_to_http(error)),
    }
}
