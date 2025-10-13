//! Defines the HTTP routes for user profile and management.
//!
//! These routes provide endpoints for accessing and updating user-specific
//! data beyond authentication credentials.

use crate::api::credential::handlers;
use crate::auth::middleware::jwt_auth;
use axum::{Router, middleware, routing::get};

/// Creates and returns the credential routes
pub fn credential_routes() -> Router {
    Router::new().route(
        "/status",
        get(handlers::get_user_credential_status).layer(middleware::from_fn(jwt_auth)),
    )
}
