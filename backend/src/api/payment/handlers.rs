//! Handler functions for payment management API endpoints.
//!
//! These functions process requests for payment data and return payment-specific information.

use crate::utils::handlers_common::{
    extract_cln_tls_components, extract_node_credentials, handle_node_error, parse_payment_hash,
    parse_public_key,
};
use crate::utils::jwt::Claims;
use crate::{
    api::common::{ApiResponse, PaginatedData, PaginationMeta, PaginationFilter, FilterRequest,
    NumericOperator, apply_pagination, get_filtered_count, validation_error_response},
    services::node_manager::{ClnConnection, ClnNode, LightningClient, LndConnection, LndNode},
    utils::{NodeId, PaymentDetails, PaymentSummary, PaymentState},
};
use axum::{
    Json,
    extract::{Extension, Path, Query},
    http::StatusCode,
};
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

    match node_credentials.node_type.as_str() {
        "lnd" => {
            let lnd_node = LndNode::new(LndConnection {
                id: NodeId::PublicKey(public_key),
                address: node_credentials.address.clone(),
                macaroon: node_credentials.macaroon.clone(),
                cert: node_credentials.tls_cert.clone(),
            })
            .await
            .map_err(|e| handle_node_error(e, "connect to LND node"))?;

            let payment_details = lnd_node
                .get_payment_details(&payment_hash)
                .await
                .map_err(|e| handle_node_error(e, "get payment details"))?;

            Ok(Json(ApiResponse::success(
                payment_details,
                "Payment details retrieved successfully",
            )))
        }

        "cln" => {
            let (client_cert, client_key, ca_cert) = extract_cln_tls_components(node_credentials)?;

            let cln_node = ClnNode::new(ClnConnection {
                id: NodeId::PublicKey(public_key),
                address: node_credentials.address.clone(),
                ca_cert,
                client_cert,
                client_key,
            })
            .await
            .map_err(|e| handle_node_error(e, "connect to CLN node"))?;

            let payment_details = cln_node
                .get_payment_details(&payment_hash)
                .await
                .map_err(|e| handle_node_error(e, "get payment details"))?;

            Ok(Json(ApiResponse::success(
                payment_details,
                "Payment details retrieved successfully",
            )))
        }

        _ => {
            let error_response = ApiResponse::<()>::error(
                "Unsupported node type".to_string(),
                "unsupported_node_type",
                None,
            );
            Err((
                StatusCode::BAD_REQUEST,
                serde_json::to_string(&error_response).unwrap(),
            ))
        }
    }
}

/// Handler for listing all payments
#[axum::debug_handler]
pub async fn list_payments(
    Extension(claims): Extension<Claims>,
    Query(filter): Query<PaymentFilter>,
) -> Result<Json<ApiResponse<PaginatedData<PaymentSummary>>>, (StatusCode, String)> {
    // Validate the filter using the built-in validation
    if let Err(validation_errors) = filter.validate() {
        return Err(validation_error_response(validation_errors));
    }

    let node_credentials = extract_node_credentials(&claims)?;
    let public_key = parse_public_key(&node_credentials.node_id)?;

    match node_credentials.node_type.as_str() {
        "lnd" => {
            let lnd_node = LndNode::new(LndConnection {
                id: NodeId::PublicKey(public_key),
                address: node_credentials.address.clone(),
                macaroon: node_credentials.macaroon.clone(),
                cert: node_credentials.tls_cert.clone(),
            })
            .await
            .map_err(|e| handle_node_error(e, "connect to LND node"))?;

            let all_payments = lnd_node
                .list_payments()
                .await
                .map_err(|e| handle_node_error(e, "list payments"))?;

            process_payments_with_filters(all_payments, &filter).await
        }

        "cln" => {
            let (client_cert, client_key, ca_cert) = extract_cln_tls_components(node_credentials)?;

            let cln_node = ClnNode::new(ClnConnection {
                id: NodeId::PublicKey(public_key),
                address: node_credentials.address.clone(),
                ca_cert,
                client_cert,
                client_key,
            })
            .await
            .map_err(|e| handle_node_error(e, "connect to CLN node"))?;

            let all_payments = cln_node
                .list_payments()
                .await
                .map_err(|e| handle_node_error(e, "list payments"))?;

            process_payments_with_filters(all_payments, &filter).await
        }

        _ => {
            let error_response = ApiResponse::<()>::error(
                "Unsupported node type",
                "unsupported_node_type",
                None,
            );
            Err((
                StatusCode::BAD_REQUEST,
                serde_json::to_string(&error_response).unwrap(),
            ))
        }
    }
}

pub type PaymentFilter = FilterRequest<PaymentState>;

impl FilterRequest<PaymentState> {
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

    // Apply state filter using the existing states field
    if let Some(filter_states) = &filter.states {
        payments.retain(|payment| {
            filter_states.iter().any(|state| {
                payment.state.as_str().to_lowercase() == state.as_str().to_lowercase()
            })
        });
    }

    // Apply amount filter
if let (Some(operator), Some(filter_value)) = (&filter.operator, filter.value) {
    
    if filter_value < 0 {
        // Negative filter values shouldn't match positive amounts
        payments.clear();
    } else {
        let filter_value_u64 = filter_value as u64;
        payments.retain(|payment| {
            match operator {
                NumericOperator::Gte => payment.amount_sat >= filter_value_u64,
                NumericOperator::Lte => payment.amount_sat <= filter_value_u64,
                NumericOperator::Eq => payment.amount_sat == filter_value_u64,
                NumericOperator::Gt => payment.amount_sat > filter_value_u64,
                NumericOperator::Lt => payment.amount_sat < filter_value_u64,
            }
        });
    }
}

    // Apply date range filter
    if filter.from.is_some() || filter.to.is_some() {

        if let Some(from_date) = filter.from {
            payments.retain(|payment| {
                payment.completed_at
                    .map(|completed_at| (completed_at as i64) >= from_date.timestamp())
                    .unwrap_or(false)
            });
        }

        if let Some(to_date) = filter.to {
            payments.retain(|payment| {
                payment.completed_at
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
    let total_filtered_count = get_filtered_count(&filtered_payments);
    let pagination_filter = filter.to_pagination_filter();
    let paginated_payments = apply_pagination(filtered_payments, &pagination_filter);
    let pagination_meta = PaginationMeta::from_filter(&pagination_filter, total_filtered_count);
    let paginated_data = PaginatedData::new(paginated_payments, total_filtered_count);

    Ok(Json(ApiResponse::ok_paginated(paginated_data, pagination_meta)))
}

