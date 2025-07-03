//! Handler functions for the node observability API.
//!
//! These functions process requests for lightning data, interact with the
//! `services::node_manager` and `services::data_aggregator` to retrieve and
//! process information, and format the responses.

use services::node_manager::{ConnectionRequest};
use crate::utils::NodeInfo;
use axum::{routing::post, Json, Router, StatusCode};

pub async fn authenticate_node(
    Json(payload): Json<ConnectionRequest>,
) -> Result<Json<NodeInfo>, (StatusCode, String)> {
    match payload {
        ConnectionRequest::Lnd(lnd_conn) => {
            tracing::info!("Attempting to authenticate LND node: {:?}", lnd_conn.id);
            match LndNode::new(lnd_conn).await {
                Ok(lnd_node) => {
                    tracing::info!("LND node authenticated: {:?}", lnd_node.info);
                    Ok(Json(lnd_node.info))
                }
                Err(e) => {
                    tracing::error!("Failed to authenticate LND node: {}", e);
                    Err((StatusCode::INTERNAL_SERVER_ERROR, format!("LND authentication failed: {}", e)))
                }
            }
        }
        ConnectionRequest::Cln(cln_conn) => {
            tracing::info!("Attempting to authenticate CLN node: {:?}", cln_conn.id);
            match ClnNode::new(cln_conn).await {
                Ok(cln_node) => {
                    tracing::info!("CLN node authenticated: {:?}", cln_node.info);
                    Ok(Json(cln_node.info))
                }
                Err(e) => {
                    tracing::error!("Failed to authenticate CLN node: {}", e);
                    Err((StatusCode::INTERNAL_SERVER_ERROR, format!("CLN authentication failed: {}", e)))
                }
            }
        }
    }
}