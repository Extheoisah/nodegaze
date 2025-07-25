//! Middleware for protecting authenticated routes and handling authorization.
//!
//! This module contains logic for validating authentication tokens (e.g., JWTs)
//! and enforcing user permissions across the API endpoints.

use crate::api::common::ApiResponse;
use crate::utils::jwt::JwtUtils;
use axum::response::IntoResponse;
use axum::{
    extract::Request,
    http::{StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::{Json, Response},
};

/// JWT authentication middleware
pub async fn jwt_auth(mut request: Request, next: Next) -> Result<Response, Response> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = match auth_header {
        Some(header) => header,
        None => {
            let error_response = ApiResponse::<()>::error(
                "Missing authorization header",
                "authentication_error",
                None,
            );
            return Err((StatusCode::UNAUTHORIZED, Json(error_response)).into_response());
        }
    };

    // Check if it's a Bearer token
    if !auth_header.starts_with("Bearer ") {
        let error_response = ApiResponse::<()>::error(
            "Invalid authorization header format. Expected: Bearer <token>",
            "authentication_error",
            None,
        );
        return Err((StatusCode::UNAUTHORIZED, Json(error_response)).into_response());
    }

    let token = &auth_header[7..]; // Remove "Bearer " prefix

    // Validate JWT token
    let jwt_utils = match JwtUtils::new() {
        Ok(utils) => utils,
        Err(_) => {
            let error_response =
                ApiResponse::<()>::error("Internal server error", "server_error", None);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response());
        }
    };

    match jwt_utils.validate_token(token) {
        Ok(claims) => {
            // Add claims to request extensions for use in handlers
            request.extensions_mut().insert(claims);
            Ok(next.run(request).await)
        }
        Err(e) => {
            let error_response = ApiResponse::<()>::error(
                format!("Invalid or expired token: {}", e),
                "authentication_error",
                None,
            );
            Err((StatusCode::UNAUTHORIZED, Json(error_response)).into_response())
        }
    }
}

/// Optional JWT authentication middleware (doesn't fail if no token)
pub async fn optional_jwt_auth(mut request: Request, next: Next) -> Result<Response, Response> {
    let claims: Option<crate::utils::jwt::Claims> = if let Some(auth_header) = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
    {
        if auth_header.starts_with("Bearer ") {
            let token = &auth_header[7..];
            let jwt_utils = match JwtUtils::new() {
                Ok(utils) => utils,
                Err(_) => {
                    let error_response =
                        ApiResponse::<()>::error("Internal server error", "server_error", None);
                    return Err(
                        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
                    );
                }
            };

            match jwt_utils.validate_token(token) {
                Ok(claims) => Some(claims),
                Err(_) => None,
            }
        } else {
            None
        }
    } else {
        None
    };

    // Always insert the Option<Claims>, even if it's None
    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}

/// Admin role authorization middleware
pub async fn admin_auth(request: Request, next: Next) -> Result<Response, Response> {
    // Get claims from request extensions (should be set by jwt_auth middleware)
    let claims = request.extensions().get::<crate::utils::jwt::Claims>();

    let claims = match claims {
        Some(claims) => claims,
        None => {
            let error_response =
                ApiResponse::<()>::error("Authentication required", "authentication_error", None);
            return Err((StatusCode::UNAUTHORIZED, Json(error_response)).into_response());
        }
    };

    // Check if user has admin role
    if !claims.is_admin() {
        let error_response =
            ApiResponse::<()>::error("Admin privileges required", "authorization_error", None);
        return Err((StatusCode::FORBIDDEN, Json(error_response)).into_response());
    }

    Ok(next.run(request).await)
}

/// Node credentials required middleware
pub async fn node_credentials_required(
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Get claims from request extensions
    let claims = request.extensions().get::<crate::utils::jwt::Claims>();

    let claims = match claims {
        Some(claims) => claims,
        None => {
            let error_response =
                ApiResponse::<()>::error("Authentication required", "authentication_error", None);
            return Err((StatusCode::UNAUTHORIZED, Json(error_response)).into_response());
        }
    };

    // Check if user has node credentials
    if !claims.has_node_credentials() {
        let error_response = ApiResponse::<()>::error(
            "Node credentials required. Please authenticate your node first.",
            "node_credentials_required",
            None,
        );
        return Err((StatusCode::BAD_REQUEST, Json(error_response)).into_response());
    }

    Ok(next.run(request).await)
}
