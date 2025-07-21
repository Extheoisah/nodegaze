//! Database repository for notification management operations.
//!
//! Provides CRUD operations for webhook and Discord notifications.

use crate::database::models::{CreateNotification, Notification};
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

/// Repository for notification database operations.
pub struct NotificationRepository<'a> {
    /// Shared SQLite connection pool
    pool: &'a SqlitePool,
}

impl<'a> NotificationRepository<'a> {
    /// Creates a new NotificationRepository instance.
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Creates a new notification in the database.
    pub async fn create_notification(
        &self,
        notification: CreateNotification,
    ) -> Result<Notification> {
        let notification = sqlx::query_as!(
            Notification,
            r#"
            INSERT INTO notifications (id, account_id, user_id, name, notification_type, url, is_active)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING
            id as "id!",
            account_id as "account_id!",
            user_id as "user_id!",
            name as "name!",
            notification_type as "notification_type: crate::database::models::NotificationType",
            url as "url!",
            is_active as "is_active!",
            created_at as "created_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            "#,
            notification.id,
            notification.account_id,
            notification.user_id,
            notification.name,
            notification.notification_type,
            notification.url,
            true
        )
        .fetch_one(self.pool)
        .await?;

        Ok(notification)
    }

    /// Retrieves a notification by its ID.
    pub async fn get_notification_by_id(&self, id: &str) -> Result<Option<Notification>> {
        let notification = sqlx::query_as!(
            Notification,
            r#"
            SELECT
            id as "id!",
            account_id as "account_id!",
            user_id as "user_id!",
            name as "name!",
            notification_type as "notification_type: crate::database::models::NotificationType",
            url as "url!",
            is_active as "is_active!",
            created_at as "created_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            FROM notifications WHERE id = ? AND is_deleted = 0
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(notification)
    }

    /// Retrieves all notifications for an account.
    pub async fn get_notifications_by_account_id(
        &self,
        account_id: &str,
    ) -> Result<Vec<Notification>> {
        let notifications = sqlx::query_as!(
            Notification,
            r#"
            SELECT
            id as "id!",
            account_id as "account_id!",
            user_id as "user_id!",
            name as "name!",
            notification_type as "notification_type: crate::database::models::NotificationType",
            url as "url!",
            is_active as "is_active!",
            created_at as "created_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>",
            is_deleted as "is_deleted!",
            deleted_at as "deleted_at?: DateTime<Utc>"
            FROM notifications
            WHERE account_id = ? AND is_deleted = 0
            ORDER BY created_at DESC
            "#,
            account_id
        )
        .fetch_all(self.pool)
        .await?;

        Ok(notifications)
    }

    /// Updates a notification.
    pub async fn update_notification(
        &self,
        id: &str,
        name: Option<&str>,
        url: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<bool> {
        // Build the query dynamically based on provided fields
        let mut set_clauses = Vec::new();
        let mut param_count = 0;

        if name.is_some() {
            param_count += 1;
            set_clauses.push(format!("name = ?{}", param_count));
        }
        if url.is_some() {
            param_count += 1;
            set_clauses.push(format!("url = ?{}", param_count));
        }
        if is_active.is_some() {
            param_count += 1;
            set_clauses.push(format!("is_active = ?{}", param_count));
        }

        if set_clauses.is_empty() {
            return Ok(false);
        }

        set_clauses.push("updated_at = CURRENT_TIMESTAMP".to_string());
        let query = format!(
            "UPDATE notifications SET {} WHERE id = ?{} AND is_deleted = 0",
            set_clauses.join(", "),
            param_count + 1
        );

        // Execute query with proper parameter binding
        let mut query_builder = sqlx::query(&query);

        if let Some(name) = name {
            query_builder = query_builder.bind(name);
        }
        if let Some(url) = url {
            query_builder = query_builder.bind(url);
        }
        if let Some(is_active) = is_active {
            query_builder = query_builder.bind(is_active);
        }
        query_builder = query_builder.bind(id);

        let rows_affected = query_builder.execute(self.pool).await?.rows_affected();

        Ok(rows_affected > 0)
    }

    /// Soft deletes a notification.
    pub async fn delete_notification(&self, id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE notifications
            SET is_deleted = 1, deleted_at = CURRENT_TIMESTAMP
            WHERE id = ? AND is_deleted = 0
            "#,
            id
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }
}
