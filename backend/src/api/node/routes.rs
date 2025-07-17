//! Defines the HTTP routes for accessing node observability data.
//!
//! These routes map specific API paths to handler functions responsible for
//! serving channel statistics, node events, and other lightning-related information.

use super::handlers::{authenticate_node, get_node_info, get_node_info_jwt};
use crate::auth::middleware::{jwt_auth, node_credentials_required, optional_jwt_auth};
use axum::{
    Router, middleware,
    routing::{get, post},
};

pub async fn node_router() -> Router {
    Router::new()
        // Node authentication - can work with or without JWT token
        .route(
            "/auth",
            post(authenticate_node).layer(middleware::from_fn(optional_jwt_auth)), // This adds Option<Claims>
        )
        // Public route (no authentication required)
        .route("/info", post(get_node_info))
        // Protected routes (require JWT token with node credentials)
        .route(
            "/info/jwt",
            get(get_node_info_jwt)
                .layer(middleware::from_fn(node_credentials_required))
                .layer(middleware::from_fn(jwt_auth)),
        )
}
