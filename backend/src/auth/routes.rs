//! Defines the HTTP routes specifically for authentication.
//!
//! These routes handle endpoints like user login, registration, and token refreshing.
//! These are designed to be integrated into the main Axum router.

use crate::auth::handlers::*;
use crate::auth::middleware::*;
use axum::{
    Router, middleware,
    routing::{delete, get, post},
};

/// Creates the authentication router with all auth-related routes
pub fn auth_router() -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout))
        .route("/me", get(me).layer(middleware::from_fn(jwt_auth)))
        .route(
            "/revoke-node-credentials",
            delete(revoke_node_credentials).layer(middleware::from_fn(jwt_auth)),
        )
}
