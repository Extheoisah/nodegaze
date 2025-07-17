//! Main entry point for the NodeGaze backend.
//!
//! This file initializes the Axum web server, sets up database connections,
//! and registers all API routes and middleware.
//! It orchestrates the application's startup and defines its overall structure.

mod api;
mod auth;
mod config;
mod database;
mod errors;
mod repositories;
mod services;
mod utils;

use crate::api::common::ApiResponse;
use axum::{Extension, Router, response::Json, routing::get};
use config::Config;
use database::Database;
use serde_json::{Value, json};
use tracing::info;
use tracing_subscriber::fmt::init;

#[tokio::main]
async fn main() {
    init();

    let config = Config::from_env().unwrap();
    let db = Database::new(&config).await.unwrap();
    let pool = db.pool().clone();

    let app = Router::new()
        .route("/", get(root_handler))
        .nest("/api/node", api::node::routes::node_router().await)
        .nest("/api/account", api::account::routes::account_router().await)
        .nest("/auth", auth::routes::auth_router())
        .nest("/api/invite", api::invite::routes::invite_router().await)
        .layer(Extension(pool));

    let bind_address = format!("0.0.0.0:{}", config.server_port);
    let listener = tokio::net::TcpListener::bind(&bind_address).await.unwrap();

    info!("Starting NodeGaze server on port {}", config.server_port);
    axum::serve(listener, app).await.unwrap();
}

async fn root_handler() -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse::success(
        serde_json::json!({
            "service": "NodeGaze Backend",
            "version": "0.1.0"
        }),
        "Welcome to NodeGaze API",
    ))
}
