//! Authentication module for managing user accounts, sessions, and access control.
//!
//! This module provides the public interface for user authentication-related functionalities
//! such as login, registration, token management, and authorization middleware.

pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod service;

// Re-exports for convenience
