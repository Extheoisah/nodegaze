//! Data structures for authentication-related entities.
//!
//! This module defines models for users, user roles, sessions, JWT claims,
//! and refresh tokens, used for data transfer and internal representation within the
//! authentication flow.

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Login request payload
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1, message = "Username is required"))]
    pub username: String,

    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

/// Login response containing tokens and user info
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserInfo,
    pub expires_in: u64, // Token expiration in seconds
}

/// User information returned in login response
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub account_id: String,
    pub account_name: String,
    pub role: String,
    pub has_node_credentials: bool,
}

/// Token refresh request
#[derive(Debug, Deserialize, Validate)]
pub struct RefreshTokenRequest {
    #[validate(length(min = 1, message = "Refresh token is required"))]
    pub refresh_token: String,
}

/// Token refresh response
#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub expires_in: u64,
}

/// Response after revoking node credentials
#[derive(Debug, Serialize)]
pub struct RevokeNodeCredentialsResponse {
    pub access_token: String,
    pub revoked: bool,
    pub expires_in: u64,
}
