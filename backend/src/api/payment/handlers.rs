//! Handler functions for payment management API endpoints.
//!
//! These functions process requests for payment data and return payment-specific information.

use crate::api::common::{ApiResponse, PaginatedData, PaginationMeta, validation_error_response};
use crate::api::payment::models::{Payment, PaymentFilter, PaymentResponse, PaymentStatus};
use crate::utils::jwt::Claims;
use axum::{
    extract::{Extension, Json, Path, Query, RawQuery},
    http::StatusCode,
};
use serde::Serialize;
use sqlx::SqlitePool;
use validator::Validate;

/// Retrieves all payments for the connected node.
#[axum::debug_handler]
pub async fn get_payments(
    Extension(claims): Extension<Claims>,
    Query(filter): Query<PaymentFilter>,
    raw_query: RawQuery,
) -> Result<Json<ApiResponse<PaymentResponse>>, (StatusCode, String)> {
    if let Err(validation_errors) = filter.validate() {
        return Err(validation_error_response(validation_errors));
    }

    let user_id = claims.sub.as_str().to_string();
    tracing::info!("Getting all payments for user: {}", user_id);
    tracing::info!("Filter received: {:?}", filter);
    tracing::info!("Raw query string: {:?}", raw_query.0);
    // Simulate fetching payments
    let all_payments = create_mock_payments();
    tracing::info!("Total payments before filtering: {}", all_payments.len());

    // Apply filters step by step
    let mut filtered_payments = all_payments.clone();

    // Apply state filter if any states were provided
    if let Some(states) = &filter.states {
        filtered_payments.retain(|payment| states.contains(&payment.status));
    }

    // 2. Apply capacity filter (amount filter)
    if filter.operator.is_some() && filter.value.is_some() {
        let before_count = filtered_payments.len();
        filtered_payments.retain(|payment| {
            let amount_cents = payment.amount as i64;
            let filter_matches = match &filter.operator {
                Some(operator) => match operator {
                    crate::api::common::NumericOperator::Gte => {
                        amount_cents >= filter.value.unwrap()
                    }
                    crate::api::common::NumericOperator::Lte => {
                        amount_cents <= filter.value.unwrap()
                    }
                    crate::api::common::NumericOperator::Eq => {
                        amount_cents == filter.value.unwrap()
                    }
                    crate::api::common::NumericOperator::Gt => amount_cents > filter.value.unwrap(),
                    crate::api::common::NumericOperator::Lt => amount_cents < filter.value.unwrap(),
                },
                None => false,
            };
            if filter_matches {
                tracing::debug!(
                    "Payment {} (amount: {}) matches capacity filter",
                    payment.id,
                    payment.amount
                );
            }
            filter_matches
        });
        tracing::info!(
            "After capacity filter: {} -> {} payments",
            before_count,
            filtered_payments.len()
        );
    }

    // 3. Apply date range filter
    if filter.from.is_some() || filter.to.is_some() {
        let before_count = filtered_payments.len();

        if let Some(from_date) = filter.from {
            filtered_payments.retain(|payment| payment.created_at >= from_date);
        }

        if let Some(to_date) = filter.to {
            filtered_payments.retain(|payment| payment.created_at <= to_date);
        }

        tracing::info!(
            "After date filter: {} -> {} payments",
            before_count,
            filtered_payments.len()
        );
    }

    // Get total count after all filters but before pagination
    let total_filtered_count = filtered_payments.len() as u64;
    tracing::info!(
        "Total filtered count (before pagination): {}",
        total_filtered_count
    );

    // 4. Apply pagination
    let page = filter.page.unwrap_or(1);
    let per_page = filter.per_page.unwrap_or(20);
    let offset = ((page - 1) * per_page) as usize;
    let limit = per_page as usize;

    tracing::info!(
        "Pagination: page={}, per_page={}, offset={}, limit={}",
        page,
        per_page,
        offset,
        limit
    );

    let paginated_payments: Vec<Payment> = filtered_payments
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    tracing::info!("Final paginated count: {}", paginated_payments.len());

    // Create pagination metadata using the correct values
    let pagination_meta = PaginationMeta::new(page, per_page, total_filtered_count);

    // Create paginated data
    let paginated_data = PaginatedData::new(paginated_payments, total_filtered_count);

    let response = PaymentResponse {
        payments: paginated_data,
        outgoing_payments_amount: 300.0,
        incoming_payments_amount: 150.0,
        outgoing_payment_volume: 500.0,
        incoming_payment_volume: 250.0,
        forwarded_payments_amount: 100.0,
        forwarded_payment_volume: 200.0,
    };

    Ok(Json(ApiResponse::ok_paginated(response, pagination_meta)))
}

// Helper function to create mock payments
fn create_mock_payments() -> Vec<Payment> {
    use chrono::{Duration, Utc};
    let now = Utc::now();

    vec![
        Payment {
            id: "payment1".to_string(),
            amount: 100.0,
            status: PaymentStatus::Completed,
            created_at: now - Duration::days(1),
        },
        Payment {
            id: "payment2".to_string(),
            amount: 200.0,
            status: PaymentStatus::Pending,
            created_at: now - Duration::hours(2),
        },
        Payment {
            id: "payment3".to_string(),
            amount: 50.0,
            status: PaymentStatus::Failed,
            created_at: now - Duration::days(3),
        },
        Payment {
            id: "payment4".to_string(),
            amount: 300.0,
            status: PaymentStatus::Processing,
            created_at: now - Duration::hours(6),
        },
        Payment {
            id: "payment5".to_string(),
            amount: 150.0,
            status: PaymentStatus::Completed,
            created_at: now - Duration::days(2),
        },
    ]
}

/// Retrieves a payment by its ID.
#[axum::debug_handler]
pub async fn get_payment_by_id(
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Payment>>, (StatusCode, String)> {
    let node_credentials = claims.node_credentials().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "No node credentials found in token. Please authenticate your node first.".to_string(),
        )
    })?;

    let user_id = claims.sub.as_str().to_string();

    tracing::info!("Getting payment by ID: {} for user: {}", id, user_id);

    // Simulate fetching a payment by ID
    let payment = Payment {
        id: id.clone(),
        amount: 150.0,
        status: PaymentStatus::Completed,
        created_at: chrono::Utc::now() - chrono::Duration::days(1),
    };

    tracing::info!("Payment found: {}", payment.id);
    Ok(Json(ApiResponse::success(
        payment,
        "Payment retrieved successfully",
    )))
}
