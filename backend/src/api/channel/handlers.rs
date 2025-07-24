use crate::utils::jwt::Claims;
use crate::{
    api::common::ApiResponse,
    services::node_manager::{ClnConnection, ClnNode, LightningClient, LndConnection, LndNode},
    utils::{ChannelDetails, ChannelSummary, NodeId, ShortChannelID},
};
use crate::utils::handlers_common::{
    extract_node_credentials, parse_public_key, extract_cln_tls_components, handle_node_error,
};
use axum::{
    Json,
    extract::{Extension, Path},
    http::StatusCode,
};
use std::str::FromStr;

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

            let channel_details = lnd_node.get_channel_info(&scid)
                .await
                .map_err(|e| handle_node_error(e, "get channel info"))?;

            Ok(Json(ApiResponse::success(channel_details, "Channel details retrieved successfully")))
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

            let channel_details = cln_node.get_channel_info(&scid)
                .await
                .map_err(|e| handle_node_error(e, "get channel info"))?;

            Ok(Json(ApiResponse::success(channel_details, "Channel details retrieved successfully")))
        }

        _ => {
            let error_response = ApiResponse::<()>::error(
                "Unsupported node type".to_string(),
                "unsupported_node_type",
                None,
            );
            Err((StatusCode::BAD_REQUEST, serde_json::to_string(&error_response).unwrap()))
        }
    }
}

#[axum::debug_handler]
pub async fn list_channels(
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<Vec<ChannelSummary>>>, (StatusCode, String)> {
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

            let channels = lnd_node.list_channels()
                .await
                .map_err(|e| handle_node_error(e, "list channels"))?;

            Ok(Json(ApiResponse::success(channels, "Channels retrieved successfully")))
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

            let channels = cln_node.list_channels()
                .await
                .map_err(|e| handle_node_error(e, "list channels"))?;

            Ok(Json(ApiResponse::success(channels, "Channels retrieved successfully")))
        }

        _ => {
            let error_response = ApiResponse::<()>::error(
                "Unsupported node type".to_string(),
                "unsupported_node_type",
                None,
            );
            Err((StatusCode::BAD_REQUEST, serde_json::to_string(&error_response).unwrap()))
        }
    }
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