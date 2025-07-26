//! Authentication module for managing user accounts, sessions, and access control.
//!
//! This module provides the public interface for user authentication-related functionalities
//! such as login, registration, token management, and authorization middleware.

pub mod routes;
pub mod handlers;
pub mod models;
pub mod middleware;
pub mod service;
pub mod errors;

// Re-exports for convenience
