//! Defines the HTTP routes for payment management.
//!
//! These routes provide endpoints for accessing and updating payment-specific
//! data.

use super::handlers::{get_payment_by_id, get_payment_details, get_payments, list_payments};
use crate::auth::middleware::{jwt_auth, node_credentials_required};
use axum::{Router, middleware, routing::get};

pub async fn payment_router() -> Router {
    Router::new()
        // Protected routes (require JWT token with node credentials)
        .route(
            "/get-payments",
            get(get_payments)
                .layer(middleware::from_fn(node_credentials_required))
                .layer(middleware::from_fn(jwt_auth)),
        )
        .route(
            "/get-payment/{id}",
            get(get_payment_by_id)
                .layer(middleware::from_fn(node_credentials_required))
                .layer(middleware::from_fn(jwt_auth)),
        )
        .route(
            "/{payment_hash}",
            get(get_payment_details)
                .layer(middleware::from_fn(node_credentials_required))
                .layer(middleware::from_fn(jwt_auth)),
        )
        .route(
            "/",
            get(list_payments)
                .layer(middleware::from_fn(node_credentials_required))
                .layer(middleware::from_fn(jwt_auth)),
        )
}
