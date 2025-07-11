//! Main entry point for the NodeGaze backend.
//!
//! This file initializes the Axum web server, sets up database connections,
//! and registers all API routes and middleware.
//! It orchestrates the application's startup and defines its overall structure.

mod api;
mod config;
mod database;
mod errors;
mod repositories;
mod services;
mod utils;

use axum::{Extension, Router, response::Json, routing::get};
use config::Config;
use database::Database;
use serde_json::{Value, json};

#[tokio::main]
async fn main() {
    let config = Config::from_env().unwrap();
    let db = Database::new(&config).await.unwrap();
    let pool = db.pool().clone();

    let app = Router::new()
        .route("/", get(root_handler))
        .nest("/api/node", api::node::routes::node_router().await)
        .nest("/api/account", api::account::routes::account_router().await)
        .layer(Extension(pool));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root_handler() -> Json<Value> {
    Json(json!({ "message": "Welcome to  NodeGaze" }))
}
