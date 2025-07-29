//! Handler functions for event management API endpoints.

use crate::api::common::{ApiResponse, PaginatedData, service_error_to_http};
use crate::database::models::EventResponse;
use crate::services::event_service::EventService;
use crate::utils::jwt::Claims;
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::Json as ResponseJson,
};
use sqlx::SqlitePool;

/// Retrieves events for the user's account.
#[axum::debug_handler]
pub async fn get_events(
    Extension(pool): Extension<SqlitePool>,
    Extension(claims): Extension<Claims>,
) -> Result<ResponseJson<ApiResponse<PaginatedData<EventResponse>>>, (StatusCode, String)> {
    let account_id = claims.account_id();

    let service = EventService::new(&pool);

    // Get all events for the account
    let events = service
        .get_events_for_account(&pool, account_id, None)
        .await
        .map_err(service_error_to_http)?;

    let total = events.len() as u64;
    let response = PaginatedData::new(events, total);

    Ok(ResponseJson(ApiResponse::success(
        response,
        "Events retrieved successfully",
    )))
}

/// Retrieves a specific event by ID.
#[axum::debug_handler]
pub async fn get_event_by_id(
    Extension(pool): Extension<SqlitePool>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<ResponseJson<ApiResponse<EventResponse>>, (StatusCode, String)> {
    let account_id = claims.account_id();

    let service = EventService::new(&pool);

    // Get all events for the account
    let events = service
        .get_events_for_account(&pool, account_id, None)
        .await
        .map_err(service_error_to_http)?;

    // Find the specific event by ID
    let event = events.into_iter().find(|e| e.id == id).ok_or_else(|| {
        let error_response = ApiResponse::<()>::error("Event not found", "not_found", None);
        (
            StatusCode::NOT_FOUND,
            serde_json::to_string(&error_response).unwrap(),
        )
    })?;

    Ok(ResponseJson(ApiResponse::success(
        event,
        "Event retrieved successfully",
    )))
}
