//! Central module for application-wide configuration settings.
//!
//! This module handles loading and managing configuration parameters such as
//! database URLs, server port, and paths to sensitive files (macaroons, certs).

use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub max_connections: u32,
    pub acquire_timeout_seconds: u64,
    pub encryption_key: String,
}

impl Config {
    /// Loads configuration from environment variables.
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL").context("DATABASE_URL not set")?;

        let max_connections = env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u32>()
            .context("DB_MAX_CONNECTIONS must be a valid number")?;

        let acquire_timeout_seconds = env::var("DB_ACQUIRE_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "3".to_string())
            .parse::<u64>()
            .context("DB_ACQUIRE_TIMEOUT_SECONDS must be a valid number")?;

        let encryption_key = env::var("ENCRYPTION_KEY").context("ENCRYPTION_KEY not set")?;

        Ok(Config {
            database_url,
            max_connections,
            acquire_timeout_seconds,
            encryption_key,
        })
    }
}
