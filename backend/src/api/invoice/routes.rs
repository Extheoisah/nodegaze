use super::handlers::{get_invoice_details, list_invoices};
use crate::auth::middleware::{jwt_auth, node_credentials_required};
use axum::{Router, middleware, routing::get};

pub async fn invoice_router() -> Router {
    Router::new()
        .route(
            "/{payment_hash}",
            get(get_invoice_details)
                .layer(middleware::from_fn(node_credentials_required))
                .layer(middleware::from_fn(jwt_auth)),
        )
        .route(
            "/",
            get(list_invoices)
                .layer(middleware::from_fn(node_credentials_required))
                .layer(middleware::from_fn(jwt_auth)),
        )
}
