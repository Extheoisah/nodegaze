//! Global application error types and handlers.
//!
//! This module defines custom error types that are used across the entire
//! backend application and provides mechanisms for consistent error handling
//! and response formatting.

use thiserror::Error;

/// Represents errors that can occur during Lightning Network operations.
#[derive(Debug, Error)]
pub enum LightningError {
    /// Error that occurred while connecting to a Lightning node.
    #[error("Node connection error: {0}")]
    ConnectionError(String),
    /// Error that occurred while retrieving node information.
    #[error("Get info error: {0}")]
    GetInfoError(String),
    /// Error that occurred while retrieving payments.
    #[error("Error while retrieving payments: {0}")]
    PaymentError(String),
    /// Error that occurred while retrieving invoices.
    #[error("Error while retrieving invoices: {0}")]
    InvoiceError(String),
    /// Error that occurred while retrieving information about a specific node.
    #[error("Get node info error: {0}")]
    GetNodeInfoError(String),
    /// Error that occurred during configuration validation.
    #[error("Config validation failed: {0}")]
    ValidationError(String),
    /// Error that occurred while getting graph.
    #[error("Get graph error: {0}")]
    GetGraphError(String),
    /// Error that occurred while streaming events.
    #[error("Streaming error: {0}")]
    StreamingError(String),
    /// Channel-related error.
    #[error("Channel error: {0}")]
    ChannelError(String),
    /// Generic not found error.
    #[error("Not found: {0}")]
    NotFound(String),
    /// Parse error for things like pubkeys or strings.
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Network error: {0}")]
    /// Network error.
    NetworkError(String),
}

/// Generic service error that can be used across all entities
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("{entity} not found: {identifier}")]
    NotFound { entity: String, identifier: String },

    #[error("{entity} already exists: {identifier}")]
    AlreadyExists { entity: String, identifier: String },

    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },

    #[error("Invalid operation: {message}")]
    InvalidOperation { message: String },

    #[error("Database error: {source}")]
    Database {
        #[from]
        source: anyhow::Error,
    },
    #[error("External service error: {message}")]
    ExternalService { message: String },
    #[error("Internal error: {message}")]
    InternalError { message: String },
}

pub type ServiceResult<T> = Result<T, ServiceError>;

impl ServiceError {
    // Helper constructors for common patterns

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    pub fn not_found(entity: impl Into<String>, identifier: impl Into<String>) -> Self {
        Self::NotFound {
            entity: entity.into(),
            identifier: identifier.into(),
        }
    }

    pub fn already_exists(entity: impl Into<String>, identifier: impl Into<String>) -> Self {
        Self::AlreadyExists {
            entity: entity.into(),
            identifier: identifier.into(),
        }
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::PermissionDenied {
            message: message.into(),
        }
    }

    pub fn invalid_operation(message: impl Into<String>) -> Self {
        Self::InvalidOperation {
            message: message.into(),
        }
    }

    pub fn external_service(message: impl Into<String>) -> Self {
        Self::ExternalService {
            message: message.into(),
        }
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
        }
    }
}
