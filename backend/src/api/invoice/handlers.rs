use crate::utils::handlers_common::{
    extract_cln_tls_components, extract_node_credentials, handle_node_error, parse_payment_hash,
    parse_public_key,
};
use crate::utils::jwt::Claims;
use crate::{
    api::common::ApiResponse,
    services::node_manager::{ClnConnection, ClnNode, LightningClient, LndConnection, LndNode},
    utils::{CustomInvoice, NodeId},
};
use axum::{
    Json,
    extract::{Extension, Path},
    http::StatusCode,
};

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

#[axum::debug_handler]
pub async fn list_invoices(
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<Vec<CustomInvoice>>>, (StatusCode, String)> {
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

            Ok(Json(ApiResponse::success(
                invoices,
                "Invoices retrieved successfully",
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

            let invoices = cln_node
                .list_invoices()
                .await
                .map_err(|e| handle_node_error(e, "list invoices"))?;

            Ok(Json(ApiResponse::success(
                invoices,
                "Invoices retrieved successfully",
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
