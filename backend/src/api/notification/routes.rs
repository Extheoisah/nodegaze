//! Defines the HTTP routes for notification management.

use super::handlers::{
    create_notification, delete_notification, get_notification_by_id, get_notification_events,
    get_notifications, update_notification,
};
use crate::auth::middleware::jwt_auth;
use axum::{
    Router, middleware,
    routing::{delete, get, post, put},
};

pub async fn notification_router() -> Router {
    Router::new()
        .route("/", post(create_notification))
        .layer(middleware::from_fn(jwt_auth))
        .route("/", get(get_notifications))
        .layer(middleware::from_fn(jwt_auth))
        .route("/{id}", get(get_notification_by_id))
        .layer(middleware::from_fn(jwt_auth))
        .route("/{id}", put(update_notification))
        .layer(middleware::from_fn(jwt_auth))
        .route("/{id}", delete(delete_notification))
        .layer(middleware::from_fn(jwt_auth))
        .route("/{id}/events", get(get_notification_events))
        .layer(middleware::from_fn(jwt_auth))
}
