//! Defines the HTTP routes for invite management.
//!
//! These routes provide endpoints for accessing and updating invite-specific requests

use super::handlers::{accept_invite, create_invite, get_invite_by_id, get_invites, resend_invite};
use crate::auth::middleware::{jwt_auth, node_credentials_required, optional_jwt_auth};
use axum::{
    Router, middleware,
    routing::{get, post},
};

pub async fn invite_router() -> Router {
    Router::new()
        // Protected routes (require JWT token with node credentials)
        .route(
            "/send-invite",
            post(create_invite)
                .layer(middleware::from_fn(node_credentials_required))
                .layer(middleware::from_fn(jwt_auth)),
        )
        .route(
            "/get-invites",
            get(get_invites)
                .layer(middleware::from_fn(node_credentials_required))
                .layer(middleware::from_fn(jwt_auth)),
        )
        .route(
            "/resend-invite/{id}",
            post(resend_invite)
                .layer(middleware::from_fn(node_credentials_required))
                .layer(middleware::from_fn(jwt_auth)),
        )
        .route(
            "/get-invite/{id}",
            get(get_invite_by_id)
                .layer(middleware::from_fn(node_credentials_required))
                .layer(middleware::from_fn(jwt_auth)),
        )
        .route(
            "/accept-invite",
            post(accept_invite)
                .layer(middleware::from_fn(node_credentials_required))
                .layer(middleware::from_fn(jwt_auth)),
        )
}
