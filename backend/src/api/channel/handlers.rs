use crate::utils::handlers_common::{
    create_node_client, extract_node_credentials, handle_node_error, parse_public_key,
};
use crate::utils::jwt::Claims;
use crate::{
    api::common::{
        ApiResponse, FilterRequest, NumericOperator, PaginatedData, PaginationFilter,
        PaginationMeta, apply_pagination, validation_error_response,
    },
    utils::{ChannelDetails, ChannelState, ChannelSummary, ShortChannelID},
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

    let node_client = create_node_client(&node_credentials, public_key).await?;

    let channel_details = node_client
        .get_channel_info(&scid)
        .await
        .map_err(|e| handle_node_error(e, "get channel info"))?;

    Ok(Json(ApiResponse::success(
        channel_details,
        "Channel details retrieved successfully",
    )))
}

/// Handler for listing all channels with filtering and pagination
#[axum::debug_handler]
pub async fn list_channels(
    Extension(claims): Extension<Claims>,
    Query(filter): Query<ChannelFilter>,
) -> Result<Json<ApiResponse<PaginatedData<ChannelSummary>>>, (StatusCode, String)> {
    if let Err(validation_errors) = filter.validate() {
        return Err(validation_error_response(validation_errors));
    }

    let node_credentials = extract_node_credentials(&claims)?;
    let public_key = parse_public_key(&node_credentials.node_id)?;

    let node_client = create_node_client(&node_credentials, public_key).await?;

    let channels = node_client
        .list_channels()
        .await
        .map_err(|e| handle_node_error(e, "list channels"))?;

    process_channels_with_filters(channels, &filter).await
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
            channels.retain(|channel| match operator {
                NumericOperator::Gte => channel.capacity >= filter_value_u64,
                NumericOperator::Lte => channel.capacity <= filter_value_u64,
                NumericOperator::Eq => channel.capacity == filter_value_u64,
                NumericOperator::Gt => channel.capacity > filter_value_u64,
                NumericOperator::Lt => channel.capacity < filter_value_u64,
            });
        }
    }

    // Apply date range filter (for channel creation dates)
if filter.from.is_some() || filter.to.is_some() {
    if let Some(from_date) = filter.from {
        channels.retain(|channel| {
            channel
                .last_update
                .map(|creation_date| {
                    // Safely convert i64 timestamp to u64 (clamping negative values to 0)
                    let from_ts = from_date.timestamp().max(0) as u64;
                    creation_date >= from_ts
                })
                .unwrap_or(false)
        });
    }

    if let Some(to_date) = filter.to {
        channels.retain(|channel| {
            channel
                .last_update
                .map(|creation_date| {
                    // Safely convert i64 timestamp to u64 (negative becomes 0)
                    let to_ts = to_date.timestamp().max(0) as u64;
                    creation_date <= to_ts
                })
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

    Ok(Json(ApiResponse::ok_paginated(
        paginated_data,
        pagination_meta,
    )))
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
