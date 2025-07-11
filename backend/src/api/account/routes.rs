//! Defines the HTTP routes for account management.
//!
//! These routes provide endpoints for accessing and updating account-specific
//! data.

use super::handlers::create_account;
use axum::{Router, routing::post};

pub async fn account_router() -> Router {
    let app = Router::new().route("/create-account", post(create_account));
    app
}
