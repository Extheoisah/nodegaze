//! Error handling utilities for API responses.
//!
//! Provides structured error responses and conversion between service-layer errors
//! and HTTP responses. Includes:
//! - Standard error response format
//! - ServiceError to HTTP status code mapping
//! - Validation error formatting helpers
//! - Pagination support for list endpoints
//! - Flexible filtering system for different data types
//! - In-memory filtering capabilities
//!
//! # Response Format
//! All errors return consistent JSON responses containing:
//! - `error`: Human-readable message
//! - `error_type`: Machine-readable error category
//! - `details`: Optional field-specific validation errors
//!
//! Paginated responses include:
//! - `pagination`: Metadata about current page, total items, etc.
//!
//! # Error Handling Flow
//! 1. Service layer returns domain-specific `ServiceError`
//! 2. `service_error_to_http` converts to appropriate HTTP response
//! 3. Validation errors are automatically formatted with field details
//!
//! # Filtering System
//! The filtering system supports:
//! - Pagination with page/per_page
//! - Numeric filtering (capacity) with multiple operators
//! - Date range filtering with from/to dates
//! - Generic state filtering that works with any enum
//! - In-memory filtering for collections

use crate::errors::ServiceError;
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{
    Deserialize, Serialize,
    de::{DeserializeOwned, Deserializer, Error as DeError},
};
use std::fmt::Debug;
use std::str::FromStr;
use validator::Validate;

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
    /// Pagination metadata (present for paginated responses)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationMeta>,
    /// Request timestamp
    pub timestamp: String,
}

/// Pagination metadata for list responses
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationMeta {
    /// Current page number (1-indexed)
    pub current_page: u32,
    /// Number of items per page
    pub per_page: u32,
    /// Total number of items across all pages
    pub total_items: u64,
    /// Total number of pages
    pub total_pages: u32,
    /// Whether there is a next page
    pub has_next: bool,
    /// Whether there is a previous page
    pub has_prev: bool,
    /// Next page number (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page: Option<u32>,
    /// Previous page number (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_page: Option<u32>,
}

/// Paginated response wrapper containing items and pagination metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedData<T> {
    /// List of items for current page
    pub items: Vec<T>,
    /// Total count of items (redundant with pagination.total_items but convenient)
    pub total: u64,
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

/// Pagination parameters for requests
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PaginationFilter {
    /// Page number (1-indexed)
    #[validate(range(min = 1))]
    pub page: Option<u32>,
    /// Number of items per page
    #[validate(range(min = 1, max = 100))]
    pub per_page: Option<u32>,
}

// Numeric comparison operators for filtering
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum NumericOperator {
    /// Greater than or equal to
    Gte,
    /// Less than or equal to
    Lte,
    /// Equal to
    Eq,
    /// Greater than
    Gt,
    /// Less than
    Lt,
}

/// Complete filter combining pagination and filtering options
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct FilterRequest<T>
where
    T: Debug + Clone + Serialize + DeserializeOwned + FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    /// Page number (1-indexed)
    #[validate(range(min = 1))]
    pub page: Option<u32>,

    /// Number of items per page
    #[validate(range(min = 1, max = 100))]
    pub per_page: Option<u32>,

    /// The comparison operator
    pub operator: Option<NumericOperator>,

    /// The value to compare against
    pub value: Option<i64>,

    /// Start date (inclusive)
    pub from: Option<DateTime<Utc>>,

    /// End date (inclusive)
    pub to: Option<DateTime<Utc>>,

    #[serde(default, deserialize_with = "deserialize_states")]
    pub states: Option<Vec<T>>,
}

fn deserialize_states<'de, D, T>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned + FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    use serde::de::Error;

    // First try to deserialize as an optional string
    let opt_string: Option<String> = Option::deserialize(deserializer)?;

    match opt_string {
        Some(s) if s.trim().is_empty() => Ok(None),
        Some(s) => {
            // Split by comma and parse each state
            let states = s
                .split(',')
                .map(|state| state.trim())
                .filter(|state| !state.is_empty())
                .map(|state| {
                    T::from_str(state)
                        .map_err(|e| Error::custom(format!("Invalid state '{}': {}", state, e)))
                })
                .collect::<Result<Vec<T>, _>>()?;

            if states.is_empty() {
                Ok(None)
            } else {
                Ok(Some(states))
            }
        }
        None => Ok(None),
    }
}

// ============================================================================
// Implementation Details
// ============================================================================

impl PaginationMeta {
    /// Create pagination metadata from page parameters and total count
    pub fn new(current_page: u32, per_page: u32, total_items: u64) -> Self {
        let total_pages = if total_items == 0 {
            1
        } else {
            ((total_items - 1) / per_page as u64 + 1) as u32
        };

        let has_next = current_page < total_pages;
        let has_prev = current_page > 1;

        Self {
            current_page,
            per_page,
            total_items,
            total_pages,
            has_next,
            has_prev,
            next_page: if has_next {
                Some(current_page + 1)
            } else {
                None
            },
            prev_page: if has_prev {
                Some(current_page - 1)
            } else {
                None
            },
        }
    }

    pub fn from_filter(filter: &PaginationFilter, total_items: u64) -> Self {
        Self::new(filter.page(), filter.per_page(), total_items)
    }
}

impl<T> PaginatedData<T> {
    /// Create a new paginated data wrapper
    pub fn new(items: Vec<T>, total: u64) -> Self {
        Self { items, total }
    }
}

impl<T> ApiResponse<T> {
    /// Create a successful response
    pub fn success(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: message.into(),
            error: None,
            pagination: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create a successful response with default message
    pub fn ok(data: T) -> Self {
        Self::success(data, "Request successful")
    }

    /// Create a successful paginated response
    pub fn paginated(data: T, pagination: PaginationMeta, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: message.into(),
            error: None,
            pagination: Some(pagination),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create a successful paginated response with default message
    pub fn ok_paginated(data: T, pagination: PaginationMeta) -> Self {
        Self::paginated(data, pagination, "Request successful")
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
            pagination: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl PaginationFilter {
    /// Get page number with default
    pub fn page(&self) -> u32 {
        self.page.unwrap_or(1)
    }

    /// Get per_page with default
    pub fn per_page(&self) -> u32 {
        self.per_page.unwrap_or(20)
    }

    /// Calculate offset for database queries
    pub fn offset(&self) -> u64 {
        ((self.page() - 1) * self.per_page()) as u64
    }

    /// Get limit for database queries
    pub fn limit(&self) -> u64 {
        self.per_page() as u64
    }
}

impl Default for PaginationFilter {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(20),
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

/// Apply pagination to a collection
pub fn apply_pagination<T>(items: Vec<T>, pagination: &PaginationFilter) -> Vec<T> {
    let offset = pagination.offset() as usize;
    let limit = pagination.limit() as usize;

    items.into_iter().skip(offset).take(limit).collect()
}

/// Get filtered count without pagination
pub fn get_filtered_count<T>(items: &[T]) -> u64 {
    items.len() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_meta_calculation() {
        // Test normal pagination
        let meta = PaginationMeta::new(2, 10, 25);
        assert_eq!(meta.current_page, 2);
        assert_eq!(meta.per_page, 10);
        assert_eq!(meta.total_items, 25);
        assert_eq!(meta.total_pages, 3);
        assert!(meta.has_next);
        assert!(meta.has_prev);
        assert_eq!(meta.next_page, Some(3));
        assert_eq!(meta.prev_page, Some(1));

        // Test first page
        let meta = PaginationMeta::new(1, 10, 25);
        assert!(!meta.has_prev);
        assert!(meta.has_next);
        assert_eq!(meta.prev_page, None);
        assert_eq!(meta.next_page, Some(2));

        // Test last page
        let meta = PaginationMeta::new(3, 10, 25);
        assert!(meta.has_prev);
        assert!(!meta.has_next);
        assert_eq!(meta.prev_page, Some(2));
        assert_eq!(meta.next_page, None);

        // Test empty result set
        let meta = PaginationMeta::new(1, 10, 0);
        assert_eq!(meta.total_pages, 1);
        assert!(!meta.has_next);
        assert!(!meta.has_prev);
    }

    #[test]
    fn test_pagination_filter() {
        let filter = PaginationFilter {
            page: Some(2),
            per_page: Some(50),
        };
        assert_eq!(filter.page(), 2);
        assert_eq!(filter.per_page(), 50);
        assert_eq!(filter.offset(), 50);
        assert_eq!(filter.limit(), 50);
    }

    #[test]
    fn test_pagination_helper() {
        let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let pagination = PaginationFilter {
            page: Some(2),
            per_page: Some(3),
        };

        let paginated = apply_pagination(items, &pagination);
        assert_eq!(paginated, vec![4, 5, 6]); // Skip 3, take 3
    }
}
