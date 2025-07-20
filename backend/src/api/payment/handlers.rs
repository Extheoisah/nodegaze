//! Handler functions for payment management API endpoints.
//!
//! These functions process requests for payment data and return payment-specific information.

use crate::api::common::ApiResponse;
use crate::api::payment::models::{Payment, PaymentResponse};
use crate::utils::jwt::Claims;
use aes_gcm::aead::rand_core::le;
use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
};
use serde::Serialize;
use sqlx::SqlitePool;

/// Retrieves all payments for the connected node.
#[axum::debug_handler]
pub async fn get_payments(
    Extension(claims): Extension<Claims>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<ApiResponse<PaymentResponse>>, (StatusCode, String)> {
    let node_credentials = claims.node_credentials().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "No node credentials found in token. Please authenticate your node first.".to_string(),
        )
    })?;

    let user_id = claims.sub.as_str().to_string();

    tracing::info!("Getting all payments for user: {}", user_id);

    // Simulate fetching payments
    let payments = vec![
        Payment {
            id: "payment1".to_string(),
            amount: 100.0,
        },
        Payment {
            id: "payment2".to_string(),
            amount: 200.0,
        },
    ];

    let response = PaymentResponse {
        payments,
        outgoing_payments_amount: 300.0,
        incoming_payments_amount: 150.0,
        outgoing_payment_volume: 500.0,
        incoming_payment_volume: 250.0,
        forwarded_payments_amount: 100.0,
        forwarded_payment_volume: 200.0,
    };

    Ok(Json(ApiResponse::success(
        response,
        "Payments retrieved successfully",
    )))
}

/// Retrieves a payment by its ID.
#[axum::debug_handler]
pub async fn get_payment_by_id(
    Extension(claims): Extension<Claims>,
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Payment>>, (StatusCode, String)> {
    let node_credentials = claims.node_credentials().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "No node credentials found in token. Please authenticate your node first.".to_string(),
        )
    })?;

    let user_id = claims.sub.as_str().to_string();

    tracing::info!("Getting payment by ID: {} for user: {}", id, user_id);

    // Simulate fetching a payment by ID
    let payment = Payment {
        id: id.clone(),
        amount: 150.0,
    };

    tracing::info!("Payment found: {}", payment.id);
    Ok(Json(ApiResponse::success(
        payment,
        "Payment retrieved successfully",
    )))
}

/// Export payments data to a CSV file.
#[axum::debug_handler]
pub async fn export_payments(
    Extension(claims): Extension<Claims>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, String)> {
    let node_credentials = claims.node_credentials().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "No node credentials found in token. Please authenticate your node first.".to_string(),
        )
    })?;

    let user_id = claims.sub.as_str().to_string();

    tracing::info!("Exporting payments for user: {}", user_id);

    // Simulate exporting payments to CSV
    let csv_data = "id,amount\npayment1,100.0\npayment2,200.0".to_string();

    Ok(Json(ApiResponse::success(
        csv_data,
        "Payments exported successfully",
    )))
}
