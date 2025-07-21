//! Central module for organizing the application's main API endpoints.
//!
//! This module acts as a top-level container for different API domains,
//! such as node observability data and user profiles, excluding core
//! authentication routes which are handled separately.

pub mod account;
pub mod common;
pub mod credential;
pub mod event;
pub mod invite;
pub mod node;
pub mod notification;
