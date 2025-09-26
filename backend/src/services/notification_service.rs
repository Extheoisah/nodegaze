//! Notification business logic service.
//!
//! Handles all notification-related business operations

use crate::database::models::{
    CreateNotification, CreateNotificationRequest, EventResponse, Notification,
    UpdateNotificationRequest, User,
};
use crate::errors::{ServiceError, ServiceResult};
use crate::repositories::event_repository::EventRepository;
use crate::repositories::notification_repository::NotificationRepository;
use chrono::Utc;
use reqwest::Client;
use serde_json::json;
use sqlx::SqlitePool;
use std::time::Duration;
use uuid::Uuid;
use validator::Validate;

pub struct NotificationService<'a> {
    /// Shared database connection pool
    pool: &'a SqlitePool,
}

impl<'a> NotificationService<'a> {
    /// Creates a new NotificationService instance.
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Creates a new notification with full validation.
    pub async fn create_notification(
        &self,
        create_request: CreateNotificationRequest,
        user: &User,
    ) -> ServiceResult<Notification> {
        // Input validation using validator crate
        if let Err(validation_errors) = create_request.validate() {
            let error_messages: Vec<String> = validation_errors
                .field_errors()
                .into_iter()
                .flat_map(|(field, errors)| {
                    errors.iter().map(move |error| {
                        format!(
                            "{}: {}",
                            field,
                            error.message.as_ref().unwrap_or(&"Invalid value".into())
                        )
                    })
                })
                .collect();

            return Err(ServiceError::validation(error_messages.join(", ")));
        }

        // Validate URL based on notification type
        self.validate_url(&create_request.url, &create_request.notification_type)
            .await?;

        let create_notification = CreateNotification {
            id: Uuid::now_v7().to_string(),
            account_id: user.account_id.clone(),
            user_id: user.id.clone(),
            name: create_request.name,
            notification_type: create_request.notification_type,
            url: create_request.url,
        };

        let repo = NotificationRepository::new(self.pool);
        let notification = repo.create_notification(create_notification).await?;

        Ok(notification)
    }

    /// Retrieves all notifications for a user's account.
    pub async fn get_notifications_for_account(
        &self,
        account_id: &str,
    ) -> ServiceResult<Vec<Notification>> {
        let repo = NotificationRepository::new(self.pool);
        let notifications = repo.get_notifications_by_account_id(account_id).await?;
        Ok(notifications)
    }

    /// Retrieves a notification by ID with account verification.
    pub async fn get_notification_required(
        &self,
        id: &str,
        account_id: &str,
    ) -> ServiceResult<Notification> {
        let repo = NotificationRepository::new(self.pool);
        let notification = repo
            .get_notification_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Notification", id))?;

        // Verify that the notification belongs to the account
        if notification.account_id != account_id {
            return Err(ServiceError::not_found("Notification", id));
        }

        Ok(notification)
    }

    /// Updates a notification with validation.
    pub async fn update_notification(
        &self,
        id: &str,
        update_request: UpdateNotificationRequest,
        account_id: &str,
    ) -> ServiceResult<Notification> {
        // Input validation
        if let Err(validation_errors) = update_request.validate() {
            let error_messages: Vec<String> = validation_errors
                .field_errors()
                .into_iter()
                .flat_map(|(field, errors)| {
                    errors.iter().map(move |error| {
                        format!(
                            "{}: {}",
                            field,
                            error.message.as_ref().unwrap_or(&"Invalid value".into())
                        )
                    })
                })
                .collect();

            return Err(ServiceError::validation(error_messages.join(", ")));
        }

        // First verify the notification exists and belongs to the account
        let existing = self.get_notification_required(id, account_id).await?;

        // Validate URL if provided
        if let Some(ref url) = update_request.url {
            self.validate_url(url, &existing.notification_type).await?;
        }

        let repo = NotificationRepository::new(self.pool);
        let updated = repo
            .update_notification(
                id,
                update_request.name.as_deref(),
                update_request.url.as_deref(),
                update_request.is_active,
            )
            .await?;

        if !updated {
            return Err(ServiceError::not_found("Notification", id));
        }

        // Return updated notification
        self.get_notification_required(id, account_id).await
    }

    /// Deletes a notification.
    pub async fn delete_notification(&self, id: &str, account_id: &str) -> ServiceResult<()> {
        // Verify the notification exists and belongs to the account
        self.get_notification_required(id, account_id).await?;

        let repo = NotificationRepository::new(self.pool);
        repo.delete_notification(id).await?;

        Ok(())
    }

    /// Gets events dispatched to a specific notification endpoint.
    pub async fn get_events_for_notification(
        &self,
        notifications_id: &str,
        account_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> ServiceResult<Vec<EventResponse>> {
        // First verify the notification belongs to the account
        self.get_notification_required(notifications_id, account_id)
            .await?;

        let limit = limit.unwrap_or(50).min(1000);
        let offset = offset.unwrap_or(0);

        let event_repo = EventRepository::new(self.pool);
        let events = event_repo
            .get_events_by_notification_id(notifications_id, limit, offset)
            .await?;

        Ok(events)
    }

    /// Gets count of events for a notification endpoint.
    pub async fn count_events_for_notification(
        &self,
        notifications_id: &str,
        account_id: &str,
    ) -> ServiceResult<i64> {
        // First verify the notification belongs to the account
        self.get_notification_required(notifications_id, account_id)
            .await?;

        let event_repo = EventRepository::new(self.pool);
        let count = event_repo
            .count_events_by_notification_id(notifications_id)
            .await?;

        Ok(count)
    }

    /// Validates URL based on notification type.
    async fn validate_url(
        &self,
        url: &str,
        notification_type: &crate::database::models::NotificationType,
    ) -> ServiceResult<()> {
        match notification_type {
            crate::database::models::NotificationType::Discord => {
                if !url.contains("discord.com/api/webhooks/") {
                    return Err(ServiceError::validation(
                        "Discord URLs must be valid Discord webhook URLs",
                    ));
                }
            }
            crate::database::models::NotificationType::Webhook => {
                self.test_webhook_connection(url).await?;
            }
        }
        Ok(())
    }

    async fn test_webhook_connection(&self, url: &str) -> ServiceResult<()> {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|err| ServiceError::InternalError {
                message: format!("HTTP client setup failed: {}", err),
            })?;

        let response = client
            .post(url)
            .json(&json!({
                "event": "Ping",
                "timestamp": Utc::now().to_rfc3339()
            }))
            .send()
            .await
            .map_err(|err| ServiceError::ExternalService {
                message: match err {
                    err if err.is_timeout() => "Webhook timeout after 5 seconds".into(),
                    err if err.is_connect() => "Could not connect to webhook server".into(),
                    _ => format!("Webhook communication failed: {}", err),
                },
            })?;

        if !response.status().is_success() {
            return Err(ServiceError::ExternalService {
                message: format!("Webhook test failed with status {}", response.status()),
            });
        }

        Ok(())
    }
}
