//! Module for core business logic services.
//!
//! This module encapsulates services that perform specific business operations
//! and orchestrate interactions between different parts of the application,
//! such as managing node connections or aggregating data.

pub mod account_service;
pub mod background_event_service;
pub mod credential_service;
pub mod data_aggregator;
pub mod email_service;
pub mod event_manager;

pub mod event_service;
pub mod invite_service;
pub mod node_manager;
pub mod notification_dispatcher;
pub mod notification_service;
pub mod user_service;
