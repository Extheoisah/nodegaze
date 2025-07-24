//! Defines the HTTP routes for account management.
//!
//! These routes provide endpoints for accessing and updating account-specific
//! data.

use super::handlers::{create_account, get_account, get_account_admin_user, get_account_users};
use crate::auth::middleware::jwt_auth;
use axum::{
    Router, middleware,
    routing::{get, post},
};

pub async fn account_router() -> Router {
    let app = Router::new()
        .route("/create-account", post(create_account))
        .route(
            "/get-account",
            get(get_account).layer(middleware::from_fn(jwt_auth)),
        )
        .route(
            "/get-account-admin-user",
            get(get_account_admin_user).layer(middleware::from_fn(jwt_auth)),
        )
        .route(
            "/get-account-users",
            get(get_account_users).layer(middleware::from_fn(jwt_auth)),
        );
    app
}
