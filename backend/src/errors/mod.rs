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
    /// Error that occurred while sending a payment.
    #[error("Send payment error: {0}")]
    SendPaymentError(String),
    /// Error that occurred while tracking a payment.
    #[error("Track payment error: {0}")]
    TrackPaymentError(String),
    /// Error that occurred when a payment hash is invalid.
    #[error("Invalid payment hash")]
    InvalidPaymentHash,
    /// Error that occurred while retrieving information about a specific node.
    #[error("Get node info error: {0}")]
    GetNodeInfoError(String),
    /// Error that occurred during configuration validation.
    #[error("Config validation failed: {0}")]
    ValidationError(String),
    /// Error that represents a permanent failure condition.
    #[error("Permanent error: {0:?}")]
    PermanentError(String),
    /// Error that occurred while listing channels.
    #[error("List channels error: {0}")]
    ListChannelsError(String),
    /// Error that occurred while getting graph.
    #[error("Get graph error: {0}")]
    GetGraphError(String),
}