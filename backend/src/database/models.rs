//! Rust structs that represent database table mappings.
//!
//! These models define the structure of data as it is stored in and retrieved
//! from the database, often used by an ORM. Note that these may differ from
//! API-specific models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateAccount {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Account name must be between 1-255 characters"
    ))]
    pub name: String,
    #[validate(length(
        min = 1,
        max = 255,
        message = "User's name must be between 1-255 characters"
    ))]
    pub username: String,
    #[validate(
        email(message = "Must be a valid email"),
        length(max = 255, message = "Email too long")
    )]
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateNewAccount {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Account name must be between 1-255 characters"
    ))]
    pub name: String,
    #[validate(length(
        min = 1,
        max = 255,
        message = "User's name must be between 1-255 characters"
    ))]
    pub username: String,
    #[validate(
        email(message = "Must be a valid email"),
        length(max = 255, message = "Email too long")
    )]
    pub email: String,

    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub password_hash: String,
    pub email: String,
    pub role_id: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateNewUser {
    #[validate(length(min = 1, message = "Account ID is required"))]
    pub account_id: String,

    #[validate(length(min = 1, message = "Role ID is required"))]
    pub role_id: String,

    #[validate(length(min = 1, max = 255, message = "Name must be between 1-255 characters"))]
    pub name: String,

    // #[validate(
    //     email(message = "Must be a valid email"),
    //     length(max = 255, message = "Email too long")
    // )]
    pub email: String,

    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateUser {
    #[validate(length(min = 1, message = "Account ID is required"))]
    pub account_id: String,

    #[validate(length(min = 1, message = "Role ID is required"))]
    pub role_id: String,

    #[validate(length(min = 1, max = 255, message = "Name must be between 1-255 characters"))]
    pub name: String,

    #[validate(
        email(message = "Must be a valid email"),
        length(max = 255, message = "Email too long")
    )]
    pub email: String,

    #[validate(length(min = 1, message = "Password hash is required"))]
    pub password_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateRole {
    #[validate(length(min = 1, max = 255, message = "Name must be between 1-255 characters"))]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Credential {
    pub id: String,
    pub user_id: String,
    pub account_id: String,
    pub node_id: String,
    pub node_alias: String,
    pub macaroon: String,
    pub tls_cert: String,
    pub address: String,
    pub node_type: Option<String>,   // "lnd" or "cln"
    pub client_cert: Option<String>, // For CLN
    pub client_key: Option<String>,  // For CLN
    pub ca_cert: Option<String>,     // For CLN
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCredential {
    #[validate(length(min = 1, message = "User ID is required"))]
    pub user_id: String,

    #[validate(length(min = 1, message = "Account ID is required"))]
    pub account_id: String,

    #[validate(length(min = 1, message = "Node ID is required"))]
    pub node_id: String,

    #[validate(length(min = 1, max = 255, message = "Node alias must be 1-255 characters"))]
    pub node_alias: String,

    #[validate(length(min = 1, message = "Macaroon is required"))]
    pub macaroon: String,

    #[validate(length(min = 1, message = "TLS certificate is required"))]
    pub tls_cert: String,

    #[validate(
        length(min = 1, max = 255, message = "Address must be 1-255 characters"),
        custom(function = "validate_socket_address")
    )]
    pub address: String,

    pub node_type: Option<String>,
    pub client_cert: Option<String>,
    pub client_key: Option<String>,
    pub ca_cert: Option<String>,
}

// Custom validation function
fn validate_socket_address(address: &str) -> Result<(), validator::ValidationError> {
    if !address.contains(':') {
        return Err(validator::ValidationError::new(
            "Address must contain port (format: host:port)",
        ));
    }
    Ok(())
}

// View models for API responses (with joined data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountWithUsers {
    pub account: Account,
    pub users: Vec<User>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserWithAccount {
    pub user: User,
    pub account: Account,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserWithRoleAndPermissions {
    pub user: User,
    pub role: Role,
}
