//! Defines the HTTP routes for event management.

use super::handlers::{get_event_by_id, get_events};
use crate::auth::middleware::jwt_auth;
use axum::{Router, middleware, routing::get};

pub async fn event_router() -> Router {
    Router::new()
        .route("/", get(get_events))
        .route("/{id}", get(get_event_by_id))
        .layer(middleware::from_fn(jwt_auth))
}
