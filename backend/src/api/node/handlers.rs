//! Handler functions for the node observability API.
use crate::api::common::ApiResponse;
use crate::database::models::CreateCredential;
use crate::errors::LightningError;
use crate::repositories::credential_repository::CredentialRepository;
use crate::services::node_manager::LightningClient;
use crate::services::node_manager::{
    ClnConnection, ClnNode, ConnectionRequest, LndConnection, LndNode,
};
use crate::services::event_manager::{EventCollector, EventProcessor, NodeSpecificEvent};
use crate::utils::jwt::Claims;
use crate::utils::{NodeId, NodeInfo};
use axum::{
    extract::{Extension, Json},
    http::StatusCode,
};
use sqlx::SqlitePool;

use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::Mutex;

use uuid::Uuid;


/// Node authentication response with stored credential info
#[derive(Debug, serde::Serialize)]
pub struct NodeAuthResponse {
    pub node_info: NodeInfo,
    pub credential_stored: bool,
    pub credential_id: Option<String>,
}

#[axum::debug_handler]
pub async fn authenticate_node(
    Extension(pool): Extension<SqlitePool>,
    Extension(claims): Extension<Option<Claims>>,
    Json(payload): Json<ConnectionRequest>,
) -> Result<Json<ApiResponse<NodeAuthResponse>>, (StatusCode, String)> {
    // First authenticate with the node
    let node_info = match &payload {
        ConnectionRequest::Lnd(lnd_conn) => {
            tracing::info!("Attempting to authenticate LND node: {:?}", lnd_conn.id);
            match LndNode::new(lnd_conn.clone()).await {
                Ok(lnd_node) => {
                    tracing::info!("LND node authenticated: {:?}", lnd_node.info);

                    let info = lnd_node.info.clone();

                    let (sender, receiver) = mpsc::channel::<NodeSpecificEvent>(32);    

                    let collector = EventCollector::new(sender);
                    let lnd_node_: Arc<Mutex<Box<dyn LightningClient  + Send + Sync + 'static>>>  = Arc::new(Mutex::new(Box::new(lnd_node)));

                    collector.start_sending(info.pubkey, lnd_node_).await;

                    let processor = EventProcessor::new();
                    processor.start_receiving(receiver);

                    info
                },
                Err(e) => {
                    tracing::error!("Failed to authenticate LND node: {}", e);
                    let error_response = ApiResponse::<()>::error(
                        format!("LND authentication failed: {}", e),
                        "node_authentication_error",
                        None,
                    );
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        serde_json::to_string(&error_response).unwrap(),
                    ));
                }
            }
        }
        ConnectionRequest::Cln(cln_conn) => {
            tracing::info!("Attempting to authenticate CLN node: {:?}", cln_conn.id);
            match ClnNode::new(cln_conn.clone()).await {
                Ok(cln_node) => {
                    tracing::info!("CLN node authenticated: {:?}", cln_node.info);
                    cln_node.info
                }
                Err(e) => {
                    tracing::error!("Failed to authenticate CLN node: {}", e);
                    let error_response = ApiResponse::<()>::error(
                        format!("CLN authentication failed: {}", e),
                        "node_authentication_error",
                        None,
                    );
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        serde_json::to_string(&error_response).unwrap(),
                    ));
                }
            }
        }
    };

    // If user is authenticated (has JWT token), store the credentials
    let (credential_stored, credential_id) = if let Some(user_claims) = claims {
        match store_node_credentials(&pool, &user_claims, &payload, &node_info).await {
            Ok(credential_id) => {
                tracing::info!("Node credentials stored for user: {}", user_claims.sub);
                (true, Some(credential_id))
            }
            Err(e) => {
                tracing::warn!("Failed to store credentials: {}", e);
                (false, None)
            }
        }
    } else {
        tracing::info!("No JWT token provided, skipping credential storage");
        (false, None)
    };

    let response_data = NodeAuthResponse {
        node_info,
        credential_stored,
        credential_id,
    };

    let message = if credential_stored {
        "Node authenticated and credentials stored successfully"
    } else {
        "Node authenticated successfully"
    };

    Ok(Json(ApiResponse::success(response_data, message)))
}

/// Helper function to store node credentials in database
async fn store_node_credentials(
    pool: &SqlitePool,
    claims: &Claims,
    connection_request: &ConnectionRequest,
    node_info: &NodeInfo,
) -> Result<String, String> {
    let credential_repo = CredentialRepository::new(pool);

    // Check if user already has credentials - if so, update them
    if let Some(existing_credential) = credential_repo
        .get_credential_by_user_id(&claims.sub)
        .await
        .map_err(|e| format!("Database error: {}", e))?
    {
        // Delete old credential (soft delete)
        credential_repo
            .delete_credential(&existing_credential.id)
            .await
            .map_err(|e| format!("Failed to delete old credential: {}", e))?;
    }

    // Extract connection details based on type
    let (node_type, macaroon, tls_cert, address, client_cert, client_key, ca_cert) =
        match connection_request {
            ConnectionRequest::Lnd(lnd_conn) => (
                Some("lnd".to_string()),
                lnd_conn.macaroon.clone(),
                lnd_conn.cert.clone(),
                lnd_conn.address.clone(),
                None,
                None,
                None,
            ),
            ConnectionRequest::Cln(cln_conn) => (
                Some("cln".to_string()),
                "".to_string(), // CLN doesn't use macaroons in the same way
                "".to_string(), // TLS cert is handled differently in CLN
                cln_conn.address.clone(),
                Some(cln_conn.client_cert.clone()),
                Some(cln_conn.client_key.clone()),
                Some(cln_conn.ca_cert.clone()),
            ),
        };

    // Create new credential record with all required fields
    let create_credential = CreateCredential {
        id: Uuid::now_v7().to_string(),
        user_id: claims.sub.clone(),
        account_id: claims.account_id.clone(),
        node_id: node_info.pubkey.to_string(),
        node_alias: node_info.alias.clone(),
        macaroon,
        tls_cert,
        address,
        node_type,
        client_cert,
        client_key,
        ca_cert,
    };

    let credential = credential_repo
        .create_credential(create_credential)
        .await
        .map_err(|e| format!("Failed to store credential: {}", e))?;

    Ok(credential.id)
}

/// Get node info using JWT token credentials
#[axum::debug_handler]
pub async fn get_node_info_jwt(
    Extension(claims): Extension<Claims>,
) -> Result<Json<NodeInfo>, (StatusCode, String)> {
    let node_credentials = claims.node_credentials().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "No node credentials found in token. Please authenticate your node first.".to_string(),
        )
    })?;

    // Create connection request based on node type
    match node_credentials.node_type.as_str() {
        "lnd" => {
            let lnd_conn =
                LndConnection {
                    id: NodeId::PublicKey(node_credentials.node_id.parse().map_err(|e| {
                        (StatusCode::BAD_REQUEST, format!("Invalid node ID: {}", e))
                    })?),
                    address: node_credentials.address.clone(),
                    macaroon: node_credentials.macaroon.clone(),
                    cert: node_credentials.tls_cert.clone(),
                };

            match LndNode::new(lnd_conn).await {
                Ok(lnd_node) => Ok(Json(lnd_node.info)),
                Err(e) => {
                    tracing::error!("Failed to connect to LND node: {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("LND connection failed: {}", e),
                    ))
                }
            }
        }
        "cln" => {
            let client_cert = node_credentials.client_cert.as_ref().ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    "Missing client certificate for CLN".to_string(),
                )
            })?;

            let client_key = node_credentials.client_key.as_ref().ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    "Missing client key for CLN".to_string(),
                )
            })?;

            let ca_cert = node_credentials.ca_cert.as_ref().ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    "Missing CA certificate for CLN".to_string(),
                )
            })?;

            let cln_conn =
                ClnConnection {
                    id: NodeId::PublicKey(node_credentials.node_id.parse().map_err(|e| {
                        (StatusCode::BAD_REQUEST, format!("Invalid node ID: {}", e))
                    })?),
                    address: node_credentials.address.clone(),
                    ca_cert: ca_cert.clone(),
                    client_cert: client_cert.clone(),
                    client_key: client_key.clone(),
                };

            match ClnNode::new(cln_conn).await {
                Ok(cln_node) => Ok(Json(cln_node.info)),
                Err(e) => {
                    tracing::error!("Failed to connect to CLN node: {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("CLN connection failed: {}", e),
                    ))
                }
            }
        }
        _ => Err((StatusCode::BAD_REQUEST, "Unsupported node type".to_string())),
    }
}

// Keep existing functions...
pub async fn connect_lightning(
    conn: ConnectionRequest,
) -> Result<Box<dyn LightningClient + Send>, LightningError> {
    match conn {
        ConnectionRequest::Lnd(lnd_conn) => {
            let node = LndNode::new(lnd_conn).await?;
            Ok(Box::new(node))
        }
        ConnectionRequest::Cln(cln_conn) => {
            let node = ClnNode::new(cln_conn).await?;
            Ok(Box::new(node))
        }
    }
}

#[axum::debug_handler]
pub async fn get_node_info(
    Json(payload): Json<ConnectionRequest>,
) -> Result<Json<NodeInfo>, (StatusCode, String)> {
    match connect_lightning(payload).await {
        Ok(client) => Ok(Json(client.get_info().clone())),
        Err(e) => {
            tracing::error!("Failed to get node info: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}
