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
    pub encryption_key: String,

    // Email configuration
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub from_email: Option<String>,
    pub from_name: Option<String>,
    pub base_url: String,
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

        let encryption_key = env::var("ENCRYPTION_KEY").context("ENCRYPTION_KEY not set")?;

        // Optional email configuration
        let smtp_host = env::var("SMTP_HOST").ok();
        let smtp_port = env::var("SMTP_PORT").ok().and_then(|p| p.parse().ok());
        let smtp_username = env::var("SMTP_USERNAME").ok();
        let smtp_password = env::var("SMTP_PASSWORD").ok();
        let from_email = env::var("FROM_EMAIL").ok();
        let from_name = env::var("FROM_NAME").ok();
        // Base URL for the application, used in email links
        let base_url = env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

        Ok(Config {
            database_url,
            max_connections,
            acquire_timeout_seconds,
            jwt_secret,
            jwt_expires_in_seconds,
            server_port,
            encryption_key,
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            from_email,
            from_name,
            base_url,
        })
    }

    /// Returns email configuration if all required fields are present
    pub fn email_config(&self) -> Option<EmailConfig> {
        match (
            &self.smtp_host,
            &self.smtp_port,
            &self.smtp_username,
            &self.smtp_password,
        ) {
            (Some(host), Some(port), Some(username), Some(password)) => Some(EmailConfig {
                smtp_host: host.clone(),
                smtp_port: *port,
                smtp_username: username.clone(),
                smtp_password: password.clone(),
                from_email: self.from_email.clone().unwrap_or_else(|| username.clone()),
                from_name: self
                    .from_name
                    .clone()
                    .unwrap_or_else(|| "Your App".to_string()),
                base_url: self.base_url.clone(),
            }),
            _ => None,
        }
    }

    /// Check if email is configured
    pub fn is_email_configured(&self) -> bool {
        self.email_config().is_some()
    }
}

/// Email-specific configuration extracted from main Config
#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
    pub from_name: String,
    pub base_url: String,
}
