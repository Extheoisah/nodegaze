use crate::utils::handlers_common::{
    extract_cln_tls_components, extract_node_credentials, handle_node_error, parse_public_key,
};
use crate::utils::jwt::Claims;
use crate::{
    api::common::{ApiResponse, PaginatedData, PaginationMeta, PaginationFilter, FilterRequest,
    NumericOperator, apply_pagination, validation_error_response},
    services::node_manager::{ClnConnection, ClnNode, LightningClient, LndConnection, LndNode},
    utils::{ChannelDetails, ChannelSummary, NodeId, ShortChannelID, ChannelState},
};
use axum::{
    Json,
    extract::{Extension, Path, Query},
    http::StatusCode,
};
use std::str::FromStr;
use validator::Validate;

#[axum::debug_handler]
pub async fn get_channel_info(
    Extension(claims): Extension<Claims>,
    Path(channel_id): Path<String>,
) -> Result<Json<ApiResponse<ChannelDetails>>, (StatusCode, String)> {
    let scid = parse_short_channel_id(&channel_id)?;
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

            let channel_details = lnd_node
                .get_channel_info(&scid)
                .await
                .map_err(|e| handle_node_error(e, "get channel info"))?;

            Ok(Json(ApiResponse::success(
                channel_details,
                "Channel details retrieved successfully",
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

            let channel_details = cln_node
                .get_channel_info(&scid)
                .await
                .map_err(|e| handle_node_error(e, "get channel info"))?;

            Ok(Json(ApiResponse::success(
                channel_details,
                "Channel details retrieved successfully",
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

/// Handler for listing all channels with filtering and pagination
#[axum::debug_handler]
pub async fn list_channels(
    Extension(claims): Extension<Claims>,
    Query(filter): Query<ChannelFilter>,
) -> Result<Json<ApiResponse<PaginatedData<ChannelSummary>>>, (StatusCode, String)> {
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

            let channels = lnd_node
                .list_channels()
                .await
                .map_err(|e| handle_node_error(e, "list channels"))?;

            process_channels_with_filters(channels, &filter).await
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

            let channels = cln_node
                .list_channels()
                .await
                .map_err(|e| handle_node_error(e, "list channels"))?;

            process_channels_with_filters(channels, &filter).await
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

pub type ChannelFilter = FilterRequest<ChannelState>;

impl FilterRequest<ChannelState> {
    pub fn to_pagination_filter(&self) -> PaginationFilter {
        PaginationFilter {
            page: self.page,
            per_page: self.per_page,
        }
    }
}

/// Apply all filters to a collection of channels
fn apply_channel_filters(
    mut channels: Vec<ChannelSummary>,
    filter: &ChannelFilter,
) -> Vec<ChannelSummary> {
    // Apply state filter
    if let Some(filter_states) = &filter.states {
        let normalized_filter_states: std::collections::HashSet<String> = filter_states
            .iter()
            .map(|state| state.to_string().to_lowercase())
            .collect();
        
        channels.retain(|channel| {
            normalized_filter_states.contains(&channel.channel_state.to_string().to_lowercase())
        });
    }

    // Apply capacity filter
    if let (Some(operator), Some(filter_value)) = (&filter.operator, filter.value) {
        if filter_value < 0 {
            // Negative filter values shouldn't match positive amounts
            channels.clear();
        } else {
            let filter_value_u64 = filter_value as u64;
            channels.retain(|channel| {
                match operator {
                    NumericOperator::Gte => channel.capacity >= filter_value_u64,
                    NumericOperator::Lte => channel.capacity <= filter_value_u64,
                    NumericOperator::Eq => channel.capacity == filter_value_u64,
                    NumericOperator::Gt => channel.capacity > filter_value_u64,
                    NumericOperator::Lt => channel.capacity < filter_value_u64,
                }
            });
        }
    }

    // Apply date range filter (for channel creation dates)
    if filter.from.is_some() || filter.to.is_some() {
        if let Some(from_date) = filter.from {
            channels.retain(|channel| {
                channel.creation_date
                    .map(|creation_date| creation_date >= from_date.timestamp())
                    .unwrap_or(false)
            });
        }

        if let Some(to_date) = filter.to {
            channels.retain(|channel| {
                channel.creation_date
                    .map(|creation_date| creation_date <= to_date.timestamp())
                    .unwrap_or(false)
            });
        }
    }

    channels
}

/// Process channels with filters and pagination
async fn process_channels_with_filters(
    all_channels: Vec<ChannelSummary>,
    filter: &ChannelFilter,
) -> Result<Json<ApiResponse<PaginatedData<ChannelSummary>>>, (StatusCode, String)> {
    let filtered_channels = apply_channel_filters(all_channels, filter);
    let total_filtered_count = filtered_channels.len() as u64;
    let pagination_filter = filter.to_pagination_filter();
    let paginated_channels = apply_pagination(filtered_channels, &pagination_filter);
    let pagination_meta = PaginationMeta::from_filter(&pagination_filter, total_filtered_count);
    let paginated_data = PaginatedData::new(paginated_channels, total_filtered_count);

    Ok(Json(ApiResponse::ok_paginated(paginated_data, pagination_meta)))
}

fn parse_short_channel_id(channel_id: &str) -> Result<ShortChannelID, (StatusCode, String)> {
    ShortChannelID::from_str(channel_id).map_err(|e| {
        let error_response = ApiResponse::<()>::error(
            format!("Invalid channel ID format: {}", e),
            "invalid_channel_id",
            None,
        );
        (
            StatusCode::BAD_REQUEST,
            serde_json::to_string(&error_response).unwrap(),
        )
    })
}