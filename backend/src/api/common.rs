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

/// Standard API response wrapper for all endpoints
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Indicates if the request was successful
    pub success: bool,
    /// Response data (present on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// Human-readable message
    pub message: String,
    /// Error details (present on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetails>,
    /// Request timestamp
    pub timestamp: String,
}

/// Error details for failed requests
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetails {
    /// Machine-readable error type identifier
    pub error_type: String,
    /// Field-specific validation errors when applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<FieldError>>,
}

/// Field-specific validation error details
#[derive(Debug, Serialize, Deserialize)]
pub struct FieldError {
    /// Name of the field with validation error
    pub field: String,
    /// Description of the validation failure
    pub message: String,
}

impl<T> ApiResponse<T> {
    /// Create a successful response
    pub fn success(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: message.into(),
            error: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create a successful response with default message
    pub fn ok(data: T) -> Self {
        Self::success(data, "Request successful")
    }

    /// Create an error response
    pub fn error(
        message: impl Into<String>,
        error_type: impl Into<String>,
        details: Option<Vec<FieldError>>,
    ) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            message: message.into(),
            error: Some(ErrorDetails {
                error_type: error_type.into(),
                details,
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Converts ServiceError to appropriate HTTP response with standard format
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
            tracing::error!("Database error: {}", source);
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

    let error_response = ApiResponse::<()>::error(message, error_type, None);
    (status, serde_json::to_string(&error_response).unwrap())
}

/// Formats validator::ValidationErrors into field-specific error details
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

/// Helper to create validation error response
pub fn validation_error_response(errors: validator::ValidationErrors) -> (StatusCode, String) {
    let field_errors = validation_errors_to_field_errors(errors);
    let error_response =
        ApiResponse::<()>::error("Validation failed", "validation_error", Some(field_errors));
    (
        StatusCode::BAD_REQUEST,
        serde_json::to_string(&error_response).unwrap(),
    )
}
