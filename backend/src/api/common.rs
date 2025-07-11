//! Error handling utilities for API responses.
//!
//! Provides structured error responses and conversion between service-layer errors
//! and HTTP responses. Includes:
//! - Standard error response format
//! - ServiceError to HTTP status code mapping
//! - Validation error formatting helpers
//!
//! # Response Format
//! All errors return consistent JSON responses containing:
//! - `error`: Human-readable message
//! - `error_type`: Machine-readable error category
//! - `details`: Optional field-specific validation errors
//!
//! # Error Handling Flow
//! 1. Service layer returns domain-specific `ServiceError`
//! 2. `service_error_to_http` converts to appropriate HTTP response
//! 3. Validation errors are automatically formatted with field details

use crate::errors::ServiceError;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

/// Standard error response format for all API endpoints
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Human-readable error message
    pub error: String,
    /// Field-specific validation errors when applicable
    pub details: Option<Vec<FieldError>>,
    /// Machine-readable error type identifier
    pub error_type: String,
}

/// Field-specific validation error details
#[derive(Debug, Serialize, Deserialize)]
pub struct FieldError {
    /// Name of the field with validation error
    pub field: String,
    /// Description of the validation failure
    pub message: String,
}

/// Converts ServiceError to appropriate HTTP response
///
/// Maps service layer errors to:
/// - HTTP status codes
/// - Standardized error response format
/// - Proper error categorization
///
/// # Examples
/// ```
/// let error = ServiceError::NotFound { /* ... */ };
/// let (status, json) = service_error_to_http(error);
/// ```
pub fn service_error_to_http(error: ServiceError) -> (StatusCode, String) {
    let (status, error_type, message) = match error {
        ServiceError::Validation { message } => {
            (StatusCode::BAD_REQUEST, "validation_error", message)
        }
        ServiceError::NotFound { entity, identifier } => (
            StatusCode::NOT_FOUND,
            "not_found",
            format!("{} '{}' not found", entity, identifier),
        ),
        ServiceError::AlreadyExists { entity, identifier } => (
            StatusCode::CONFLICT,
            "already_exists",
            format!("{} '{}' already exists", entity, identifier),
        ),
        ServiceError::PermissionDenied { message } => {
            (StatusCode::FORBIDDEN, "permission_denied", message)
        }
        ServiceError::InvalidOperation { message } => {
            (StatusCode::BAD_REQUEST, "invalid_operation", message)
        }
        ServiceError::Database { source } => {
            eprintln!("Database error: {}", source);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "database_error",
                "Internal server error".to_string(),
            )
        }
        ServiceError::ExternalService { message } => {
            (StatusCode::BAD_GATEWAY, "external_service_error", message)
        }
    };

    let error_response = ErrorResponse {
        error: message,
        details: None,
        error_type: error_type.to_string(),
    };

    (status, serde_json::to_string(&error_response).unwrap())
}

/// Formats validator::ValidationErrors into field-specific error details
///
/// Transforms validation framework errors into our standard
/// field error format for API responses.
pub fn validation_errors_to_field_errors(errors: validator::ValidationErrors) -> Vec<FieldError> {
    errors
        .field_errors()
        .into_iter()
        .flat_map(|(field, errors)| {
            errors.iter().map(move |error| FieldError {
                field: field.to_string(),
                message: error
                    .message
                    .as_ref()
                    .unwrap_or(&"Invalid value".into())
                    .to_string(),
            })
        })
        .collect()
}
