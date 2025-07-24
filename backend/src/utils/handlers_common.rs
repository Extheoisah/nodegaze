use crate::api::common::ApiResponse;
use crate::errors::LightningError;
use crate::utils::jwt::{Claims, NodeCredentials};
use axum::http::StatusCode;
use bitcoin::secp256k1::PublicKey;
use lightning::ln::PaymentHash;
use std::str::FromStr;

/// Extract credentials from claims
pub fn extract_node_credentials(claims: &Claims) -> Result<&NodeCredentials, (StatusCode, String)> {
    claims.node_credentials().ok_or_else(|| {
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

/// Parse hex string into PaymentHash
pub fn parse_payment_hash(payment_hash: &str) -> Result<PaymentHash, (StatusCode, String)> {
    let payment_hash_bytes = hex::decode(payment_hash).map_err(|e| {
        let error_response = ApiResponse::<()>::error(
            format!("Invalid payment hash format: {}", e),
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
            format!("Invalid node public key: {}", e),
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
pub fn extract_cln_tls_components(node_credentials: &NodeCredentials) -> Result<(String, String, String), (StatusCode, String)> {
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
        format!("Failed to {}: {}", operation, e),
        &format!("{}_error", operation.replace(' ', "_")),
        None,
    );
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        serde_json::to_string(&error_response).unwrap(),
    )
}