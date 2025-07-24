use super::handlers::{get_channel_info, list_channels};
use crate::auth::middleware::{jwt_auth, node_credentials_required};
use axum::{Router, middleware, routing::get};

pub async fn channel_router() -> Router {
    Router::new()
        .route(
            "/{channel_id}",
            get(get_channel_info)
                .layer(middleware::from_fn(node_credentials_required))
                .layer(middleware::from_fn(jwt_auth)),
        )
        .route(
            "/",
            get(list_channels)
                .layer(middleware::from_fn(node_credentials_required))
                .layer(middleware::from_fn(jwt_auth)),
        )
}
