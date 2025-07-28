use crate::utils::handlers_common::{
    extract_cln_tls_components, extract_node_credentials, handle_node_error, parse_payment_hash,
    parse_public_key,
};
use crate::utils::jwt::Claims;
use crate::{
    api::common::{ApiResponse, PaginatedData, PaginationMeta, PaginationFilter, FilterRequest,
    NumericOperator, apply_pagination, get_filtered_count, validation_error_response},
    services::node_manager::{ClnConnection, ClnNode, LightningClient, LndConnection, LndNode},
    utils::{CustomInvoice, NodeId, InvoiceStatus},
};
use axum::{
    Json,
    extract::{Extension, Path, Query},
    http::StatusCode,
};
use validator::Validate;

/// Handler for getting payment details
#[axum::debug_handler]
pub async fn get_invoice_details(
    Extension(claims): Extension<Claims>,
    Path(payment_hash): Path<String>,
) -> Result<Json<ApiResponse<CustomInvoice>>, (StatusCode, String)> {
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

            let invoice_details = lnd_node
                .get_invoice_details(&payment_hash)
                .await
                .map_err(|e| handle_node_error(e, "get invoice details"))?;

            Ok(Json(ApiResponse::success(
                invoice_details,
                "Invoice details retrieved successfully",
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
                .get_invoice_details(&payment_hash)
                .await
                .map_err(|e| handle_node_error(e, "get invoice details"))?;

            Ok(Json(ApiResponse::success(
                payment_details,
                "Invoice details retrieved successfully",
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

/// Handler for listing all invoices with filtering and pagination
#[axum::debug_handler]
pub async fn list_invoices(
    Extension(claims): Extension<Claims>,
    Query(filter): Query<InvoiceFilter>,
) -> Result<Json<ApiResponse<PaginatedData<CustomInvoice>>>, (StatusCode, String)> {
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

             let invoices = lnd_node
                .list_invoices()
                .await
                .map_err(|e| handle_node_error(e, "list invoices"))?;

            process_invoices_with_filters(invoices, &filter).await
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

            let invoices = cln_node
                .list_invoices()
                .await
                .map_err(|e| handle_node_error(e, "list invoices"))?;

            process_invoices_with_filters(invoices, &filter).await
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

pub type InvoiceFilter = FilterRequest<InvoiceStatus>;

impl FilterRequest<InvoiceStatus> {
    pub fn to_pagination_filter(&self) -> PaginationFilter {
        PaginationFilter {
            page: self.page,
            per_page: self.per_page,
        }
    }
}

/// Apply all filters to a collection of invoices
fn apply_invoice_filters(
    mut invoices: Vec<CustomInvoice>,
    filter: &InvoiceFilter,
) -> Vec<CustomInvoice> {
    // Apply state filter
    if let Some(filter_states) = &filter.states {
        let normalized_filter_states: std::collections::HashSet<String> = filter_states
            .iter()
            .map(|state| state.to_string().to_lowercase())
            .collect();
        
        invoices.retain(|invoice| {
            normalized_filter_states.contains(&invoice.state.to_string().to_lowercase())
        });
    }

    // Apply amount filter (using value field)
    if let (Some(operator), Some(filter_value)) = (&filter.operator, filter.value) {
        if filter_value < 0 {
            // Negative filter values shouldn't match positive amounts
            invoices.clear();
        } else {
            let filter_value_u64 = filter_value as u64;
            invoices.retain(|invoice| {
                match operator {
                    NumericOperator::Gte => invoice.value >= filter_value_u64,
                    NumericOperator::Lte => invoice.value <= filter_value_u64,
                    NumericOperator::Eq => invoice.value == filter_value_u64,
                    NumericOperator::Gt => invoice.value > filter_value_u64,
                    NumericOperator::Lt => invoice.value < filter_value_u64,
                }
            });
        }
    }

    // Apply date range filter (for invoice creation dates)
    if filter.from.is_some() || filter.to.is_some() {
        if let Some(from_date) = filter.from {
            invoices.retain(|invoice| {
                invoice.creation_date
                    .map(|creation_date| creation_date >= from_date.timestamp())
                    .unwrap_or(false)
            });
        }

        if let Some(to_date) = filter.to {
            invoices.retain(|invoice| {
                invoice.creation_date
                    .map(|creation_date| creation_date <= to_date.timestamp())
                    .unwrap_or(false)
            });
        }
    }

    invoices
}

/// Process invoices with filters and pagination
async fn process_invoices_with_filters(
    all_invoices: Vec<CustomInvoice>,
    filter: &InvoiceFilter,
) -> Result<Json<ApiResponse<PaginatedData<CustomInvoice>>>, (StatusCode, String)> {
    let filtered_invoices = apply_invoice_filters(all_invoices, filter);
    let total_filtered_count = filtered_invoices.len() as u64;
    let pagination_filter = filter.to_pagination_filter();
    let paginated_invoices = apply_pagination(filtered_invoices, &pagination_filter);
    let pagination_meta = PaginationMeta::from_filter(&pagination_filter, total_filtered_count);
    let paginated_data = PaginatedData::new(paginated_invoices, total_filtered_count);

    Ok(Json(ApiResponse::ok_paginated(paginated_data, pagination_meta)))
}
