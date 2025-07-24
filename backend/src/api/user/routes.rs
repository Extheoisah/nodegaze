//! Defines the HTTP routes for user profile and management.
//!
//! These routes provide endpoints for accessing and updating user-specific
//! data beyond authentication credentials.

use super::handlers::{change_user_role_access_level, get_user_by_id};
use crate::auth::middleware::jwt_auth;
use axum::{
    Router, middleware,
    routing::{get, post},
};

pub async fn user_router() -> Router {
    Router::new()
        .route(
            "/get-user/{id}",
            get(get_user_by_id).layer(middleware::from_fn(jwt_auth)),
        )
        .route(
            "/change-user-role-access-level/{id}",
            post(change_user_role_access_level).layer(middleware::from_fn(jwt_auth)),
        )
}
