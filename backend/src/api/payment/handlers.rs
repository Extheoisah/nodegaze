//! Handler functions for payment management API endpoints.
//!
//! These functions process requests for payment data and return payment-specific information.

use crate::utils::handlers_common::{
    create_node_client, extract_node_credentials, handle_node_error, parse_payment_hash,
    parse_public_key,
};
use crate::utils::jwt::Claims;
use crate::{
    api::common::{
        ApiResponse, NumericOperator, PaginatedData, PaginationFilter, PaginationMeta,
        apply_pagination, deserialize_states, validation_error_response,
    },
    utils::{PaymentDetails, PaymentState, PaymentSummary, PaymentType, deserialize_payment_types},
};
use axum::{
    Json,
    extract::{Extension, Path, Query},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Handler for getting payment details
#[axum::debug_handler]
pub async fn get_payment_details(
    Extension(claims): Extension<Claims>,
    Path(payment_hash): Path<String>,
) -> Result<Json<ApiResponse<PaymentDetails>>, (StatusCode, String)> {
    let payment_hash = parse_payment_hash(&payment_hash)?;
    let node_credentials = extract_node_credentials(&claims)?;
    let public_key = parse_public_key(&node_credentials.node_id)?;

    let node_client = create_node_client(node_credentials, public_key).await?;

    let payment_details = node_client
        .get_payment_details(&payment_hash)
        .await
        .map_err(|e| handle_node_error(e, "get payment details"))?;

    Ok(Json(ApiResponse::success(
        payment_details,
        "Payment details retrieved successfully",
    )))
}

/// Handler for listing all payments
#[axum::debug_handler]
pub async fn list_payments(
    Extension(claims): Extension<Claims>,
    Query(filter): Query<PaymentFilter>,
) -> Result<Json<ApiResponse<PaginatedData<PaymentSummary>>>, (StatusCode, String)> {
    if let Err(validation_errors) = filter.validate() {
        return Err(validation_error_response(validation_errors));
    }

    let node_credentials = extract_node_credentials(&claims)?;
    let public_key = parse_public_key(&node_credentials.node_id)?;

    let node_client = create_node_client(node_credentials, public_key).await?;

    let all_payments = node_client
        .list_payments()
        .await
        .map_err(|e| handle_node_error(e, "list payments"))?;

    process_payments_with_filters(all_payments, &filter).await
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PaymentFilterRequest {
    /// Page number (1-indexed)
    #[validate(range(min = 1))]
    pub page: Option<u32>,

    /// Number of items per page
    #[validate(range(min = 1, max = 100))]
    pub per_page: Option<u32>,

    /// The comparison operator
    pub operator: Option<NumericOperator>,

    /// The value to compare against
    pub value: Option<i64>,

    /// Start date (inclusive)
    pub from: Option<DateTime<Utc>>,

    /// End date (inclusive)
    pub to: Option<DateTime<Utc>>,

    /// Payment states filter
    #[serde(default, deserialize_with = "deserialize_states")]
    pub states: Option<Vec<PaymentState>>,

    /// Payment type filter (NEW - only for payments)
    #[serde(default, deserialize_with = "deserialize_payment_types")]
    pub payment_types: Option<Vec<PaymentType>>,
}

pub type PaymentFilter = PaymentFilterRequest;

impl PaymentFilterRequest {
    pub fn to_pagination_filter(&self) -> PaginationFilter {
        PaginationFilter {
            page: self.page,
            per_page: self.per_page,
        }
    }
}

/// Apply all filters to a collection of payments
fn apply_payment_filters(
    mut payments: Vec<PaymentSummary>,
    filter: &PaymentFilter,
) -> Vec<PaymentSummary> {
    // Apply state filter
    if let Some(filter_states) = &filter.states {
        payments.retain(|payment| {
            filter_states
                .iter()
                .any(|state| payment.state.as_str().to_lowercase() == state.as_str().to_lowercase())
        });
    }

    // Apply payment type filter
    if let Some(filter_payment_types) = &filter.payment_types {
        payments.retain(|payment| {
            filter_payment_types.iter().any(|pt| {
                payment.payment_type.as_str().to_lowercase() == pt.as_str().to_lowercase()
            })
        });
    }

    // Apply amount filter
    if let (Some(operator), Some(filter_value)) = (&filter.operator, filter.value) {
        if filter_value < 0 {
            payments.clear();
        } else {
            let filter_value_u64 = filter_value as u64;
            payments.retain(|payment| match operator {
                NumericOperator::Gte => payment.amount_sat >= filter_value_u64,
                NumericOperator::Lte => payment.amount_sat <= filter_value_u64,
                NumericOperator::Eq => payment.amount_sat == filter_value_u64,
                NumericOperator::Gt => payment.amount_sat > filter_value_u64,
                NumericOperator::Lt => payment.amount_sat < filter_value_u64,
            });
        }
    }

    // Apply date range filter
    if filter.from.is_some() || filter.to.is_some() {
        if let Some(from_date) = filter.from {
            payments.retain(|payment| {
                payment
                    .completed_at
                    .map(|completed_at| (completed_at as i64) >= from_date.timestamp())
                    .unwrap_or(false)
            });
        }

        if let Some(to_date) = filter.to {
            payments.retain(|payment| {
                payment
                    .completed_at
                    .map(|completed_at| (completed_at as i64) <= to_date.timestamp())
                    .unwrap_or(false)
            });
        }
    }
    payments
}

/// Process payments with filters and pagination
async fn process_payments_with_filters(
    all_payments: Vec<PaymentSummary>,
    filter: &PaymentFilter,
) -> Result<Json<ApiResponse<PaginatedData<PaymentSummary>>>, (StatusCode, String)> {
    let filtered_payments = apply_payment_filters(all_payments, filter);
    let total_filtered_count = filtered_payments.len() as u64;
    let pagination_filter = filter.to_pagination_filter();
    let paginated_payments = apply_pagination(filtered_payments, &pagination_filter);
    let pagination_meta = PaginationMeta::from_filter(&pagination_filter, total_filtered_count);
    let paginated_data = PaginatedData::new(paginated_payments, total_filtered_count);

    Ok(Json(ApiResponse::ok_paginated(
        paginated_data,
        pagination_meta,
    )))
}
