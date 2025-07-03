//! Defines the HTTP routes for accessing node observability data.
//!
//! These routes map specific API paths to handler functions responsible for
//! serving channel statistics, node events, and other lightning-related information.

use axum::{routing::post, Router};
use super::handlers::authenticate_node;

pub async fn node_router() -> Router {
    let app = Router::new()
        .route("/auth", post(authenticate_node));
    app
}