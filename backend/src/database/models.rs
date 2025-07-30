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
    pub role_access_level: RoleAccessLevel,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, PartialOrd)]
#[sqlx(type_name = "TEXT")] // Store as TEXT in SQLite
pub enum RoleAccessLevel {
    Read = 1,
    ReadWrite = 2,
}

impl std::fmt::Display for RoleAccessLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoleAccessLevel::Read => write!(f, "Read"),
            RoleAccessLevel::ReadWrite => write!(f, "ReadWrite"),
        }
    }
}

impl std::str::FromStr for RoleAccessLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Read" => Ok(RoleAccessLevel::Read),
            "ReadWrite" => Ok(RoleAccessLevel::ReadWrite),
            _ => Err(format!("Invalid role access level: {}", s)),
        }
    }
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

    #[validate(
        email(message = "Must be a valid email"),
        length(max = 255, message = "Email too long")
    )]
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
    pub role_access_level: RoleAccessLevel,
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: String,
    pub account_id: String,
    pub user_id: String,
    pub name: String,
    pub notification_type: NotificationType,
    pub url: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "TEXT")]
pub enum NotificationType {
    Webhook,
    Discord,
}

impl std::fmt::Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationType::Webhook => write!(f, "webhook"),
            NotificationType::Discord => write!(f, "discord"),
        }
    }
}

impl std::str::FromStr for NotificationType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "webhook" => Ok(NotificationType::Webhook),
            "discord" => Ok(NotificationType::Discord),
            _ => Err(format!("Invalid notification type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateNotification {
    #[validate(length(min = 1, message = "Notification ID is required"))]
    pub id: String,
    #[validate(length(min = 1, message = "Account ID is required"))]
    pub account_id: String,
    #[validate(length(min = 1, message = "User ID is required"))]
    pub user_id: String,
    #[validate(length(min = 1, max = 255, message = "Name must be between 1-255 characters"))]
    pub name: String,
    pub notification_type: NotificationType,
    #[validate(url(message = "Must be a valid URL"))]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateNotificationRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be between 1-255 characters"))]
    pub name: String,
    pub notification_type: NotificationType,
    #[validate(url(message = "Must be a valid URL"))]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateNotificationRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be between 1-255 characters"))]
    pub name: Option<String>,
    #[validate(url(message = "Must be a valid URL"))]
    pub url: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: String,
    pub account_id: String,
    pub user_id: String,
    pub node_id: String,
    pub node_alias: String,
    pub event_type: EventType,
    pub severity: EventSeverity,
    pub title: String,
    pub description: String,
    pub data: String, // JSON string
    pub notifications_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl From<Event> for EventResponse {
    fn from(event: Event) -> Self {
        Self {
            id: event.id,
            account_id: event.account_id,
            user_id: event.user_id,
            node_id: event.node_id,
            node_alias: event.node_alias,
            event_type: event.event_type,
            severity: event.severity,
            title: event.title,
            description: event.description,
            data: serde_json::from_str(&event.data).unwrap_or(serde_json::Value::Null),
            timestamp: event.timestamp,
            notifications_id: event.notifications_id,
            created_at: event.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "TEXT")]
pub enum EventType {
    ChannelOpened,
    ChannelClosed,
    InvoiceCreated,
    InvoiceSettled,
    InvoiceCancelled,
    InvoiceAccepted,
    PaymentSent,
    PaymentReceived,
    PaymentFailed,
    NodeConnected,
    NodeDisconnected,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::ChannelOpened => write!(f, "channel_opened"),
            EventType::ChannelClosed => write!(f, "channel_closed"),
            EventType::InvoiceCreated => write!(f, "invoice_created"),
            EventType::InvoiceSettled => write!(f, "invoice_settled"),
            EventType::InvoiceCancelled => write!(f, "invoice_cancelled"),
            EventType::InvoiceAccepted => write!(f, "invoice_accepted"),
            EventType::PaymentSent => write!(f, "payment_sent"),
            EventType::PaymentReceived => write!(f, "payment_received"),
            EventType::PaymentFailed => write!(f, "payment_failed"),
            EventType::NodeConnected => write!(f, "node_connected"),
            EventType::NodeDisconnected => write!(f, "node_disconnected"),
        }
    }
}

impl std::str::FromStr for EventType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "channel_opened" => Ok(EventType::ChannelOpened),
            "channel_closed" => Ok(EventType::ChannelClosed),
            "invoice_created" => Ok(EventType::InvoiceCreated),
            "invoice_settled" => Ok(EventType::InvoiceSettled),
            "invoice_cancelled" => Ok(EventType::InvoiceCancelled),
            "invoice_accepted" => Ok(EventType::InvoiceAccepted),
            "payment_sent" => Ok(EventType::PaymentSent),
            "payment_received" => Ok(EventType::PaymentReceived),
            "payment_failed" => Ok(EventType::PaymentFailed),
            "node_connected" => Ok(EventType::NodeConnected),
            "node_disconnected" => Ok(EventType::NodeDisconnected),
            _ => Err(format!("Invalid event type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "TEXT")]
pub enum EventSeverity {
    Info,
    Warning,
    Critical,
}

impl std::fmt::Display for EventSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventSeverity::Info => write!(f, "info"),
            EventSeverity::Warning => write!(f, "warning"),
            EventSeverity::Critical => write!(f, "critical"),
        }
    }
}

impl std::str::FromStr for EventSeverity {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "info" => Ok(EventSeverity::Info),
            "warning" => Ok(EventSeverity::Warning),
            "critical" => Ok(EventSeverity::Critical),
            _ => Err(format!("Invalid event severity: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateEvent {
    #[validate(length(min = 1, message = "Event ID is required"))]
    pub id: String,
    #[validate(length(min = 1, message = "Account ID is required"))]
    pub account_id: String,
    #[validate(length(min = 1, message = "User ID is required"))]
    pub user_id: String,
    #[validate(length(min = 1, message = "Node ID is required"))]
    pub node_id: String,
    pub node_alias: String,
    pub event_type: EventType,
    pub severity: EventSeverity,
    #[validate(length(min = 1, max = 255, message = "Title must be between 1-255 characters"))]
    pub title: String,
    #[validate(length(min = 1, message = "Description is required"))]
    pub description: String,
    pub data: String, // JSON string
    pub notifications_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventResponse {
    pub id: String,
    pub account_id: String,
    pub user_id: String,
    pub node_id: String,
    pub node_alias: String,
    pub event_type: EventType,
    pub severity: EventSeverity,
    pub title: String,
    pub description: String,
    pub notifications_id: Option<String>,
    pub data: serde_json::Value, // Parsed JSON
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilters {
    pub event_types: Option<Vec<EventType>>,
    pub severities: Option<Vec<EventSeverity>>,
    pub node_ids: Option<Vec<String>>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
