//! Main entry point for the NodeGaze backend.
//!
//! This file initializes the Axum web server, sets up database connections,
//! and registers all API routes and middleware.
//! It orchestrates the application's startup and defines its overall structure.

mod utils;
mod errors;
mod services;
mod api;

use axum::{routing::get, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root_handler))
        .nest("/api/node", api::node::routes::node_router());
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root_handler() -> &'static str {
    "Welcome to NodeGaze!"
}
