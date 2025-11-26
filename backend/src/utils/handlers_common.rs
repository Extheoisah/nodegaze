use crate::api::common::ApiResponse;
use crate::errors::LightningError;
use crate::services::credential_service::CredentialService;
use crate::services::event_manager::{EventCollector, EventHandler, NodeSpecificEvent};
use crate::services::node_manager::{
    ClnConnection, ClnNode, ConnectionRequest, LightningClient, LndConnection, LndNode,
};
use crate::utils::jwt::Claims;
use crate::utils::{NodeId, NodeInfo};
use axum::http::StatusCode;
use bitcoin::secp256k1::PublicKey;
use lightning::ln::PaymentHash;
use std::result::Result::Ok;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

/// Extract credentials from claims
pub fn extract_node_credential_id(claims: &Claims) -> Result<&String, (StatusCode, String)> {
    claims.node_credential_id().ok_or_else(|| {
        let error_response = ApiResponse::<()>::error(
            "No node credentials found in token".to_string(),
            "missing_credentials",
            None,
        );
        (
            StatusCode::UNAUTHORIZED,
            serde_json::to_string(&error_response).unwrap(),
        )
    })
}

/// Creates and returns a Lightning client (LND or CLN) based on the provided credentials.
pub async fn create_node_client(
    node_credential_id: &String,
    pool: &sqlx::SqlitePool,
) -> Result<Box<dyn LightningClient>, (StatusCode, String)> {
    let service = CredentialService::new(&pool);

    let node_credentials = service
        .get_credential_required(&node_credential_id.as_str())
        .await
        .map_err(|e| {
            tracing::error!("Node credential not found {}: {}", node_credential_id, e);
            let error_response = ApiResponse::<()>::error(
                "Node credential not found".to_string(),
                "node_credential_not_found",
                None,
            );
            (
                StatusCode::NOT_FOUND,
                serde_json::to_string(&error_response).unwrap(),
            )
        })?;

    let public_key = parse_public_key(&node_credentials.node_id)?;

    match node_credentials.node_type.as_deref() {
        Some("lnd") => {
            let lnd_node = LndNode::new(LndConnection {
                id: NodeId::PublicKey(public_key),
                address: node_credentials.address.clone(),
                macaroon: node_credentials.macaroon.clone(),
                cert: node_credentials.tls_cert.clone(),
            })
            .await
            .map_err(|e| handle_node_error(e, "connect to LND node"))?;

            Ok(Box::new(lnd_node))
        }
        Some("cln") => {
            let (client_cert, client_key, ca_cert) =
                extract_cln_tls_components(node_credential_id, pool).await?;

            let cln_node = ClnNode::new(ClnConnection {
                id: NodeId::PublicKey(public_key),
                address: node_credentials.address.clone(),
                ca_cert,
                client_cert,
                client_key,
            })
            .await
            .map_err(|e| handle_node_error(e, "connect to CLN node"))?;

            Ok(Box::new(cln_node))
        }
        Some(_) => {
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
        None => {
            let error_response = ApiResponse::<()>::error(
                "Node type not specified".to_string(),
                "missing_node_type",
                None,
            );
            Err((
                StatusCode::BAD_REQUEST,
                serde_json::to_string(&error_response).unwrap(),
            ))
        }
    }
}

/// Parse hex string into PaymentHash
pub fn parse_payment_hash(payment_hash: &str) -> Result<PaymentHash, (StatusCode, String)> {
    let payment_hash_bytes = hex::decode(payment_hash).map_err(|e| {
        let error_response = ApiResponse::<()>::error(
            format!("Invalid payment hash format: {e}"),
            "invalid_payment_hash",
            None,
        );
        (
            StatusCode::BAD_REQUEST,
            serde_json::to_string(&error_response).unwrap(),
        )
    })?;

    if payment_hash_bytes.len() != 32 {
        let error_response = ApiResponse::<()>::error(
            "Payment hash must be 32 bytes".to_string(),
            "invalid_payment_hash_length",
            None,
        );
        return Err((
            StatusCode::BAD_REQUEST,
            serde_json::to_string(&error_response).unwrap(),
        ));
    }

    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&payment_hash_bytes);
    Ok(PaymentHash(hash_array))
}

/// Parse node_id into PublicKey
pub fn parse_public_key(node_id: &str) -> Result<PublicKey, (StatusCode, String)> {
    PublicKey::from_str(node_id).map_err(|e| {
        let error_response = ApiResponse::<()>::error(
            format!("Invalid node public key: {e}"),
            "invalid_public_key",
            None,
        );
        (
            StatusCode::BAD_REQUEST,
            serde_json::to_string(&error_response).unwrap(),
        )
    })
}

/// Extract TLS fields for CLN
pub async fn extract_cln_tls_components(
    node_credential_id: &String,
    pool: &sqlx::SqlitePool,
) -> Result<(String, String, String), (StatusCode, String)> {
    let service = CredentialService::new(&pool);

    let node_credentials = service
        .get_credential_required(&node_credential_id.as_str())
        .await
        .map_err(|e| {
            tracing::error!("Node credential not found {}: {}", node_credential_id, e);
            let error_response = ApiResponse::<()>::error(
                "Node credential not found".to_string(),
                "node_credential_not_found",
                None,
            );
            (
                StatusCode::NOT_FOUND,
                serde_json::to_string(&error_response).unwrap(),
            )
        })?;

    let client_cert = node_credentials.client_cert.as_ref().ok_or_else(|| {
        let error_response = ApiResponse::<()>::error(
            "Missing client certificate for CLN".to_string(),
            "missing_client_cert",
            None,
        );
        (
            StatusCode::BAD_REQUEST,
            serde_json::to_string(&error_response).unwrap(),
        )
    })?;

    let client_key = node_credentials.client_key.as_ref().ok_or_else(|| {
        let error_response = ApiResponse::<()>::error(
            "Missing client key for CLN".to_string(),
            "missing_client_key",
            None,
        );
        (
            StatusCode::BAD_REQUEST,
            serde_json::to_string(&error_response).unwrap(),
        )
    })?;

    let ca_cert = node_credentials.ca_cert.as_ref().ok_or_else(|| {
        let error_response = ApiResponse::<()>::error(
            "Missing CA certificate for CLN".to_string(),
            "missing_ca_cert",
            None,
        );
        (
            StatusCode::BAD_REQUEST,
            serde_json::to_string(&error_response).unwrap(),
        )
    })?;

    Ok((client_cert.clone(), client_key.clone(), ca_cert.clone()))
}

/// Handle node operation errors
pub fn handle_node_error(e: LightningError, operation: &str) -> (StatusCode, String) {
    tracing::error!("{} failed: {}", operation, e);
    let error_response = ApiResponse::<()>::error(
        format!("Failed to {operation}: {e}"),
        format!("{}_error", operation.replace(' ', "_")),
        None,
    );
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        serde_json::to_string(&error_response).unwrap(),
    )
}

/// Connect to event stream
pub async fn connect_to_event_stream(
    connection_request: &ConnectionRequest,
    account_id: &Option<String>,
    pool: &sqlx::SqlitePool,
    user_id: &Option<String>,
) -> Result<NodeInfo, (StatusCode, String)> {
    match &connection_request {
        ConnectionRequest::Lnd(lnd_conn) => {
            tracing::info!("Attempting to authenticate LND node: {:?}", lnd_conn.id);
            match LndNode::new(lnd_conn.clone()).await {
                Ok(lnd_node) => {
                    tracing::info!("LND node authenticated: {:?}", lnd_node.info);

                    let info = lnd_node.info.clone();

                    let (sender, receiver) = mpsc::channel::<NodeSpecificEvent>(32);

                    let collector = EventCollector::new(sender);
                    let lnd_node_: Arc<Mutex<Box<dyn LightningClient + Send + Sync + 'static>>> =
                        Arc::new(Mutex::new(Box::new(lnd_node)));

                    collector.start_sending(info.pubkey, lnd_node_).await;

                    // Start processing events with database context
                    let handler = match (account_id, user_id) {
                        (Some(acc_id), Some(u_id)) => {
                            tracing::info!(
                                "Creating handler with database context for user: {}",
                                u_id
                            );
                            EventHandler::with_context(
                                pool.clone(),
                                acc_id.clone(),
                                u_id.clone(),
                                info.pubkey.to_string(),
                                info.alias.clone(),
                            )
                        }
                        _ => {
                            tracing::info!("Creating handler without database context");
                            EventHandler::new()
                        }
                    };

                    handler.start_receiving(receiver);

                    Ok(info)
                }
                Err(e) => {
                    tracing::error!("Failed to authenticate LND node: {}", e);
                    let error_response = ApiResponse::<()>::error(
                        format!("LND authentication failed: {e}"),
                        "node_authentication_error",
                        None,
                    );
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        serde_json::to_string(&error_response).unwrap(),
                    ))
                }
            }
        }
        ConnectionRequest::Cln(cln_conn) => {
            tracing::info!("Attempting to authenticate CLN node: {:?}", cln_conn.id);
            match ClnNode::new(cln_conn.clone()).await {
                Ok(cln_node) => {
                    tracing::info!("CLN node authenticated: {:?}", cln_node.info);

                    let info = cln_node.info.clone();

                    let (sender, receiver) = mpsc::channel::<NodeSpecificEvent>(32);

                    let collector = EventCollector::new(sender);
                    let cln_node_: Arc<Mutex<Box<dyn LightningClient + Send + Sync + 'static>>> =
                        Arc::new(Mutex::new(Box::new(cln_node)));

                    collector.start_sending(info.pubkey, cln_node_).await;

                    // Start processing events with database context
                    let handler = match (account_id, user_id) {
                        (Some(acc_id), Some(u_id)) => {
                            tracing::info!(
                                "Creating handler with database context for user: {}",
                                u_id
                            );
                            EventHandler::with_context(
                                pool.clone(),
                                acc_id.clone(),
                                u_id.clone(),
                                info.pubkey.to_string(),
                                info.alias.clone(),
                            )
                        }
                        _ => {
                            tracing::info!("Creating handler without database context");
                            EventHandler::new()
                        }
                    };

                    handler.start_receiving(receiver);

                    Ok(info)
                }
                Err(e) => {
                    tracing::error!("Failed to authenticate CLN node: {}", e);
                    let error_response = ApiResponse::<()>::error(
                        format!("CLN authentication failed: {e}"),
                        "node_authentication_error",
                        None,
                    );
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        serde_json::to_string(&error_response).unwrap(),
                    ))
                }
            }
        }
    }
}
