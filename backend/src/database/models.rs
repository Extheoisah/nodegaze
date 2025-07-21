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
    pub username: String,
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

    #[validate(length(
        min = 1,
        max = 255,
        message = "Username must be between 1-255 characters"
    ))]
    pub username: String,

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
    #[validate(length(min = 1, message = "User ID is required"))]
    pub id: String,
    #[validate(length(min = 1, message = "Account ID is required"))]
    pub account_id: String,

    #[validate(length(min = 1, message = "Role ID is required"))]
    pub role_id: String,

    #[validate(length(
        min = 1,
        max = 255,
        message = "Username must be between 1-255 characters"
    ))]
    pub username: String,

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
    #[validate(length(min = 1, message = "Credential ID is required"))]
    pub id: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Invite {
    pub id: String,
    pub account_id: String,
    pub inviter_id: String,
    pub invitee_email: String,
    pub token: String,
    pub invite_status: InviteStatus,
    pub is_active: bool,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "TEXT")] // Store as TEXT in SQLite
pub enum InviteStatus {
    Pending = 1,
    Accepted = 2,
}

impl std::fmt::Display for InviteStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InviteStatus::Pending => write!(f, "Pending"),
            InviteStatus::Accepted => write!(f, "Accepted"),
        }
    }
}

impl std::str::FromStr for InviteStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(InviteStatus::Pending),
            "Accepted" => Ok(InviteStatus::Accepted),
            _ => Err(format!("Invalid invite status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateInvite {
    #[validate(length(min = 1, message = "Invite ID is required"))]
    pub id: String,
    #[validate(length(min = 1, message = "Account ID is required"))]
    pub account_id: String,
    #[validate(length(min = 1, message = "Inviter ID is required"))]
    pub inviter_id: String,
    #[validate(email(message = "Must be a valid email"))]
    pub invitee_email: String,
    #[validate(length(min = 1, message = "Token is required"))]
    pub token: String,
    #[validate(custom(function = "validate_expiry_time"))]
    pub expires_at: DateTime<Utc>,
    pub invite_status: InviteStatus,
}

/// Validates that the expiry time is in the future
fn validate_expiry_time(expires_at: &DateTime<Utc>) -> Result<(), validator::ValidationError> {
    if expires_at <= &Utc::now() {
        return Err(validator::ValidationError::new(
            "expires_at must be in the future",
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateInviteRequest {
    #[validate(
        email(message = "Must be a valid email"),
        length(max = 255, message = "Email too long")
    )]
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AcceptInviteRequest {
    #[validate(length(min = 1, message = "Token is required"))]
    pub token: String,
    #[validate(length(
        min = 1,
        max = 255,
        message = "Username must be between 1-255 characters"
    ))]
    pub username: String,
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
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
