//! Error handling utilities for API responses.
//!
//! Provides structured error responses and conversion between service-layer errors
//! and HTTP responses. Includes:
//! - Standard error response format
//! - ServiceError to HTTP status code mapping
//! - Validation error formatting helpers
//! - Pagination support for list endpoints
//! - Flexible filtering system for different data types
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

use crate::errors::ServiceError;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

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

/// Capacity filter for numeric values
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CapacityFilter {
    /// The comparison operator
    pub operator: NumericOperator,
    /// The value to compare against
    pub value: i64,
}

/// Date range filter
#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct DateRangeFilter {
    /// Start date (inclusive)
    pub from: Option<DateTime<Utc>>,
    /// End date (inclusive)
    pub to: Option<DateTime<Utc>>,
}

/// Generic state filter that can work with any enum type
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StateFilter<T>
where
    T: Debug + Clone + Serialize + DeserializeOwned,
{
    /// List of states to filter by (OR logic)
    pub states: Vec<T>,
}

/// Base filter struct that other modules can extend
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BaseFilter {
    /// Capacity-based filtering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity: Option<CapacityFilter>,
    /// Date range filtering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_range: Option<DateRangeFilter>,
}

/// Complete filter combining pagination and filtering options
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct FilterRequest<T>
where
    T: Debug + Clone + Serialize + DeserializeOwned,
{
    /// Pagination parameters
    #[serde(flatten)]
    #[validate]
    pub pagination: PaginationFilter,
    /// Base filtering options
    #[serde(flatten)]
    pub base_filter: BaseFilter,
    /// Module-specific state filtering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<StateFilter<T>>,
}

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

impl DateRangeFilter {
    /// Create a new date range filter
    pub fn new(from: Option<DateTime<Utc>>, to: Option<DateTime<Utc>>) -> Self {
        Self { from, to }
    }

    /// Check if the filter has any constraints
    pub fn is_empty(&self) -> bool {
        self.from.is_none() && self.to.is_none()
    }

    /// Validate that from date is before to date
    pub fn is_valid(&self) -> bool {
        match (self.from, self.to) {
            (Some(from), Some(to)) => from <= to,
            _ => true,
        }
    }
}

impl<T> StateFilter<T>
where
    T: Debug + Clone + Serialize + DeserializeOwned,
{
    /// Create a new state filter
    pub fn new(states: Vec<T>) -> Self {
        Self { states }
    }

    /// Check if the filter has any states
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    /// Check if a given state matches any of the filter states
    pub fn matches(&self, state: &T) -> bool
    where
        T: PartialEq,
    {
        self.states.contains(state)
    }
}

impl BaseFilter {
    /// Create a new empty base filter
    pub fn new() -> Self {
        Self {
            capacity: None,
            date_range: None,
        }
    }

    /// Set capacity filter
    pub fn with_capacity(mut self, capacity: CapacityFilter) -> Self {
        self.capacity = Some(capacity);
        self
    }

    /// Set date range filter
    pub fn with_date_range(mut self, date_range: DateRangeFilter) -> Self {
        self.date_range = Some(date_range);
        self
    }
}

impl Default for BaseFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> FilterRequest<T>
where
    T: Debug + Clone + Serialize + DeserializeOwned,
{
    /// Create a new filter request
    pub fn new() -> Self {
        Self {
            pagination: PaginationFilter::default(),
            base_filter: BaseFilter::new(),
            state: None,
        }
    }

    /// Set state filter
    pub fn with_state_filter(mut self, state_filter: StateFilter<T>) -> Self {
        self.state = Some(state_filter);
        self
    }
}

impl<T> Default for FilterRequest<T>
where
    T: Debug + Clone + Serialize + DeserializeOwned,
{
    fn default() -> Self {
        Self::new()
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
}
