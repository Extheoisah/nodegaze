//! Middleware for protecting authenticated routes and handling authorization.
//!
//! This module contains logic for validating authentication tokens (e.g., JWTs)
//! and enforcing user permissions across the API endpoints.

use crate::utils::jwt::JwtUtils;
use axum::{
    extract::Request,
    http::{StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};

/// JWT authentication middleware
pub async fn jwt_auth(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if it's a Bearer token
    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..]; // Remove "Bearer " prefix

    // Validate JWT token
    let jwt_utils = JwtUtils::new().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match jwt_utils.validate_token(token) {
        Ok(claims) => {
            // Add claims to request extensions for use in handlers
            request.extensions_mut().insert(claims);
            Ok(next.run(request).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Optional JWT authentication middleware (doesn't fail if no token)
pub async fn optional_jwt_auth(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    let claims: Option<crate::utils::jwt::Claims> = if let Some(auth_header) = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
    {
        if auth_header.starts_with("Bearer ") {
            let token = &auth_header[7..];
            let jwt_utils = JwtUtils::new().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
pub async fn admin_auth(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    // Get claims from request extensions (should be set by jwt_auth middleware)
    let claims = request
        .extensions()
        .get::<crate::utils::jwt::Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if user has admin role
    if !claims.is_admin() {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

/// Node credentials required middleware
pub async fn node_credentials_required(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get claims from request extensions
    let claims = request
        .extensions()
        .get::<crate::utils::jwt::Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if user has node credentials
    if !claims.has_node_credentials() {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(next.run(request).await)
}
