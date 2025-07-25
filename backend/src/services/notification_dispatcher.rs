//! Service for dispatching events to notification endpoints.

use crate::database::models::{Event, Notification, NotificationType};
use crate::repositories::notification_repository::NotificationRepository;
use reqwest::Client;
use serde_json::json;
use sqlx::SqlitePool;
use std::time::Duration;
use tracing::{error, info, warn};

/// Service for dispatching events to notification endpoints.
#[derive(Debug, Clone)]
pub struct NotificationDispatcher {
    http_client: Client,
}

impl NotificationDispatcher {
    /// Creates a new NotificationDispatcher instance.
    pub fn new() -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self { http_client }
    }

    /// Dispatches an event to all active notifications for the account.
    pub async fn dispatch_event(
        &self,
        pool: &SqlitePool,
        event: &Event,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let notification_repo = NotificationRepository::new(pool);
        let notifications = notification_repo
            .get_notifications_by_account_id(&event.account_id)
            .await?;

        let active_notifications: Vec<_> =
            notifications.into_iter().filter(|n| n.is_active).collect();

        if active_notifications.is_empty() {
            info!(
                "No active notifications found for account {}",
                event.account_id
            );
            return Ok(());
        }

        info!(
            "Dispatching event {} to {} notification(s)",
            event.id,
            active_notifications.len()
        );

        // Dispatch to all active notifications concurrently
        let dispatch_futures: Vec<_> = active_notifications
            .into_iter()
            .map(|notification| self.send_to_endpoint(event, notification))
            .collect();

        // Wait for all dispatches to complete
        let results = futures::future::join_all(dispatch_futures).await;

        // Log results
        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(_) => info!(
                    "Successfully dispatched event {} to endpoint {}",
                    event.id, i
                ),
                Err(e) => error!(
                    "Failed to dispatch event {} to endpoint {}: {}",
                    event.id, i, e
                ),
            }
        }

        Ok(())
    }

    /// Sends an event to a specific notification endpoint.
    async fn send_to_endpoint(
        &self,
        event: &Event,
        notification: Notification,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match notification.notification_type {
            NotificationType::Webhook => self.send_webhook(event, &notification).await,
            NotificationType::Discord => self.send_discord(event, &notification).await,
        }
    }

    /// Sends event to a webhook endpoint.
    async fn send_webhook(
        &self,
        event: &Event,
        notification: &Notification,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let payload = json!({
            "event_id": event.id,
            "timestamp": event.timestamp,
            "event_type": event.event_type,
            "severity": event.severity,
            "title": event.title,
            "description": event.description,
            "node_id": event.node_id,
            "node_alias": event.node_alias,
            "data": serde_json::from_str::<serde_json::Value>(&event.data).unwrap_or(json!({}))
        });

        let response = self
            .http_client
            .post(&notification.url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "NodeGaze/1.0")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            info!(
                "Webhook notification sent successfully to {}",
                notification.url
            );
        } else {
            warn!(
                "Webhook notification failed with status {}: {}",
                response.status(),
                notification.url
            );
        }

        Ok(())
    }

    /// Sends event to a Discord webhook.
    async fn send_discord(
        &self,
        event: &Event,
        notification: &Notification,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let color = match event.severity {
            crate::database::models::EventSeverity::Info => 0x00ff00, // Green
            crate::database::models::EventSeverity::Warning => 0xffff00, // Yellow
            crate::database::models::EventSeverity::Critical => 0xff0000, // Red
        };

        let embed = json!({
            "title": event.title,
            "description": event.description,
            "color": color,
            "timestamp": event.timestamp,
            "fields": [
                {
                    "name": "Event Type",
                    "value": event.event_type.to_string(),
                    "inline": true
                },
                {
                    "name": "Severity",
                    "value": event.severity.to_string(),
                    "inline": true
                },
                {
                    "name": "Node",
                    "value": if event.node_alias.is_empty() {
                        event.node_id.clone()
                    } else {
                        format!("{} ({})", event.node_alias, &event.node_id[..8])
                    },
                    "inline": true
                }
            ],
            "footer": {
                "text": "NodeGaze Lightning Monitor"
            }
        });

        let payload = json!({
            "embeds": [embed]
        });

        let response = self
            .http_client
            .post(&notification.url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "NodeGaze/1.0")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            info!(
                "Discord notification sent successfully to {}",
                notification.url
            );
        } else {
            warn!(
                "Discord notification failed with status {}: {}",
                response.status(),
                notification.url
            );
        }

        Ok(())
    }
}
