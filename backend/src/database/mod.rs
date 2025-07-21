//! Module for database connection setup and common utilities.
//!
//! This module is responsible for initializing the database connection pool
//! and providing a central point for database-related configurations and helpers.

use crate::config::Config;
use anyhow::Result;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::time::Duration;

pub mod models;

pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    /// Initializes the database connection pool.
    pub async fn new(config: &Config) -> Result<Self> {
        let database_url = &config.database_url;

        // Create a new SqlitePool with a timeout of 30 seconds
        let pool = SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .acquire_timeout(Duration::from_secs(config.acquire_timeout_seconds))
            .connect(database_url)
            .await?;

        Ok(Database { pool })
    }

    /// Returns a reference to the database connection pool.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Closes the database connection pool.
    pub async fn close(&self) {
        self.pool.close().await;
        println!("Database connection pool closed.");
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Database {
            pool: self.pool.clone(),
        }
    }
}
