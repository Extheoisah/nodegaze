//! Database repository for event management operations.

use crate::database::models::{CreateEvent, Event, EventFilters, EventSeverity, EventType};
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

/// Repository for event database operations.
pub struct EventRepository<'a> {
    /// Shared SQLite connection pool
    pool: &'a SqlitePool,
}

impl<'a> EventRepository<'a> {
    /// Creates a new EventRepository instance.
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Creates a new event in the database.
    pub async fn create_event(&self, event: CreateEvent) -> Result<Event> {
        let event = sqlx::query_as!(
            Event,
            r#"
            INSERT INTO events (id, account_id, user_id, node_id, node_alias, event_type, severity, title, description, data, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING
            id as "id!",
            account_id as "account_id!",
            user_id as "user_id!",
            node_id as "node_id!",
            node_alias as "node_alias!",
            event_type as "event_type: EventType",
            severity as "severity: EventSeverity",
            title as "title!",
            description as "description!",
            data as "data!",
            timestamp as "timestamp!: DateTime<Utc>",
            created_at as "created_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            "#,
            event.id,
            event.account_id,
            event.user_id,
            event.node_id,
            event.node_alias,
            event.event_type,
            event.severity,
            event.title,
            event.description,
            event.data,
            event.timestamp
        )
        .fetch_one(self.pool)
        .await?;

        Ok(event)
    }

    /// Retrieves events by account ID with basic filtering.
    pub async fn get_events_by_account_id(
        &self,
        account_id: &str,
        filters: Option<EventFilters>,
    ) -> Result<Vec<Event>> {
        let filters = filters.unwrap_or(EventFilters {
            limit: None,
            offset: None,
            node_ids: None,
            event_types: None,
            severities: None,
            start_date: None,
            end_date: None,
        });

        // Simple implementation without complex dynamic queries
        let limit = filters.limit.unwrap_or(50).min(1000);
        let offset = filters.offset.unwrap_or(0);

        let events = sqlx::query_as!(
            Event,
            r#"
            SELECT
            id as "id!",
            account_id as "account_id!",
            user_id as "user_id!",
            node_id as "node_id!",
            node_alias as "node_alias!",
            event_type as "event_type: EventType",
            severity as "severity: EventSeverity",
            title as "title!",
            description as "description!",
            data as "data!",
            timestamp as "timestamp!: DateTime<Utc>",
            created_at as "created_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            FROM events
            WHERE account_id = ? AND is_deleted = 0
            ORDER BY timestamp DESC
            LIMIT ? OFFSET ?
            "#,
            account_id,
            limit,
            offset
        )
        .fetch_all(self.pool)
        .await?;

        Ok(events)
    }

    /// Gets event count by account ID.
    pub async fn count_events_by_account_id(
        &self,
        account_id: &str,
        _filters: Option<EventFilters>,
    ) -> Result<i64> {
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM events WHERE account_id = ? AND is_deleted = 0",
            account_id
        )
        .fetch_one(self.pool)
        .await?;

        Ok(result.count)
    }

    /// Gets events by account ID with specific event type filter.
    pub async fn get_events_by_account_and_type(
        &self,
        account_id: &str,
        event_type: &EventType,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Event>> {
        let events = sqlx::query_as!(
            Event,
            r#"
            SELECT
            id as "id!",
            account_id as "account_id!",
            user_id as "user_id!",
            node_id as "node_id!",
            node_alias as "node_alias!",
            event_type as "event_type: EventType",
            severity as "severity: EventSeverity",
            title as "title!",
            description as "description!",
            data as "data!",
            timestamp as "timestamp!: DateTime<Utc>",
            created_at as "created_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            FROM events
            WHERE account_id = ? AND event_type = ? AND is_deleted = 0
            ORDER BY timestamp DESC
            LIMIT ? OFFSET ?
            "#,
            account_id,
            event_type,
            limit,
            offset
        )
        .fetch_all(self.pool)
        .await?;

        Ok(events)
    }

    /// Gets events by account ID with specific severity filter.
    pub async fn get_events_by_account_and_severity(
        &self,
        account_id: &str,
        severity: &EventSeverity,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Event>> {
        let events = sqlx::query_as!(
            Event,
            r#"
            SELECT
            id as "id!",
            account_id as "account_id!",
            user_id as "user_id!",
            node_id as "node_id!",
            node_alias as "node_alias!",
            event_type as "event_type: EventType",
            severity as "severity: EventSeverity",
            title as "title!",
            description as "description!",
            data as "data!",
            timestamp as "timestamp!: DateTime<Utc>",
            created_at as "created_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            FROM events
            WHERE account_id = ? AND severity = ? AND is_deleted = 0
            ORDER BY timestamp DESC
            LIMIT ? OFFSET ?
            "#,
            account_id,
            severity,
            limit,
            offset
        )
        .fetch_all(self.pool)
        .await?;

        Ok(events)
    }

    /// Count events by severity.
    pub async fn count_events_by_account_and_severity(
        &self,
        account_id: &str,
        severity: &EventSeverity,
    ) -> Result<i64> {
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM events WHERE account_id = ? AND severity = ? AND is_deleted = 0",
            account_id,
            severity
        )
        .fetch_one(self.pool)
        .await?;

        Ok(result.count)
    }
}
