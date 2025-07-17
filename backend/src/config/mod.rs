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
    pub jwt_secret: String,
    pub jwt_expires_in_seconds: u64,
    pub server_port: u16,
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

        let jwt_secret = env::var("JWT_SECRET").context("JWT_SECRET not set")?;

        let jwt_expires_in_seconds = env::var("JWT_EXPIRES_IN_SECONDS")
            .unwrap_or_else(|_| "86400".to_string())
            .parse::<u64>()
            .context("JWT_EXPIRES_IN_SECONDS must be a valid number")?;

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .context("SERVER_PORT must be a valid number")?;

        Ok(Config {
            database_url,
            max_connections,
            acquire_timeout_seconds,
            jwt_secret,
            jwt_expires_in_seconds,
            server_port,
        })
    }
}
