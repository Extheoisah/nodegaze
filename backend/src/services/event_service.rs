//! Event business logic service.

use crate::database::models::{
    CreateEvent, Event, EventFilters, EventResponse, EventSeverity, EventType,
};
use crate::errors::{ServiceError, ServiceResult};
use crate::repositories::event_repository::EventRepository;
use crate::services::notification_dispatcher::NotificationDispatcher;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::SqlitePool;
use std::collections::HashMap;
use uuid::Uuid;

/// Service layer for event operations.
pub struct EventService {
    /// Notification dispatcher
    dispatcher: NotificationDispatcher,
}

impl EventService {
    /// Creates a new EventService instance.
    pub fn new() -> Self {
        Self {
            dispatcher: NotificationDispatcher::new(),
        }
    }

    /// Creates and dispatches a new event.
    pub async fn create_and_dispatch_event(
        &self,
        pool: &SqlitePool,
        account_id: String,
        user_id: String,
        node_id: String,
        node_alias: String,
        event_type: EventType,
        severity: EventSeverity,
        title: String,
        description: String,
        data: HashMap<String, Value>,
    ) -> ServiceResult<Event> {
        let event_data = serde_json::to_string(&data)
            .map_err(|e| ServiceError::validation(format!("Invalid event data: {}", e)))?;

        let create_event = CreateEvent {
            id: Uuid::now_v7().to_string(),
            account_id,
            user_id,
            node_id,
            node_alias,
            event_type,
            severity,
            title,
            description,
            data: event_data,
            timestamp: Utc::now(),
        };

        // Save event to database
        let repo = EventRepository::new(pool);
        let event = repo.create_event(create_event).await?;

        // Dispatch to notification endpoints (async, don't wait)
        let event_clone = event.clone();
        let dispatcher = self.dispatcher.clone();
        let pool_clone = pool.clone();

        tokio::spawn(async move {
            if let Err(e) = dispatcher.dispatch_event(&pool_clone, &event_clone).await {
                tracing::error!("Failed to dispatch event {}: {}", event_clone.id, e);
            }
        });

        Ok(event)
    }

    /// Retrieves events for an account with optional filters.
    pub async fn get_events_for_account(
        &self,
        pool: &SqlitePool,
        account_id: &str,
        filters: Option<EventFilters>,
    ) -> ServiceResult<Vec<EventResponse>> {
        let repo = EventRepository::new(pool);
        let events = repo.get_events_by_account_id(account_id, filters).await?;

        let event_responses: Vec<EventResponse> = events
            .into_iter()
            .filter_map(|event| {
                // Parse JSON data
                let data = match serde_json::from_str::<Value>(&event.data) {
                    Ok(data) => data,
                    Err(e) => {
                        tracing::warn!("Failed to parse event data for {}: {}", event.id, e);
                        serde_json::json!({})
                    }
                };

                Some(EventResponse {
                    id: event.id,
                    account_id: event.account_id,
                    user_id: event.user_id,
                    node_id: event.node_id,
                    node_alias: event.node_alias,
                    event_type: event.event_type,
                    severity: event.severity,
                    title: event.title,
                    description: event.description,
                    data,
                    timestamp: event.timestamp,
                    created_at: event.created_at,
                })
            })
            .collect();

        Ok(event_responses)
    }

    /// Gets event count for an account.
    pub async fn count_events_for_account(
        &self,
        pool: &SqlitePool,
        account_id: &str,
        filters: Option<EventFilters>,
    ) -> ServiceResult<i64> {
        let repo = EventRepository::new(pool);
        let count = repo.count_events_by_account_id(account_id, filters).await?;
        Ok(count)
    }

    /// Gets event statistics by severity.
    pub async fn get_event_stats_by_severity(
        &self,
        pool: &SqlitePool,
        account_id: &str,
    ) -> ServiceResult<(i64, i64, i64)> {
        let repo = EventRepository::new(pool);

        let info_count = repo
            .count_events_by_account_and_severity(account_id, &EventSeverity::Info)
            .await?;

        let warning_count = repo
            .count_events_by_account_and_severity(account_id, &EventSeverity::Warning)
            .await?;

        let critical_count = repo
            .count_events_by_account_and_severity(account_id, &EventSeverity::Critical)
            .await?;

        Ok((info_count, warning_count, critical_count))
    }

    /// Processes a Lightning node event and creates a standardized event.
    pub async fn process_lightning_event(
        &self,
        pool: &SqlitePool,
        account_id: String,
        user_id: String,
        node_id: String,
        node_alias: String,
        lightning_event: &crate::services::event_manager::NodeSpecificEvent,
    ) -> ServiceResult<Event> {
        let (event_type, severity, title, description, data) = match lightning_event {
            crate::services::event_manager::NodeSpecificEvent::LND(lnd_event) => {
                self.process_lnd_event(lnd_event)
            }
            crate::services::event_manager::NodeSpecificEvent::CLN(cln_event) => {
                self.process_cln_event(cln_event)
            }
        };

        self.create_and_dispatch_event(
            pool,
            account_id,
            user_id,
            node_id,
            node_alias,
            event_type,
            severity,
            title,
            description,
            data,
        )
        .await
    }

    /// Processes LND-specific events.
    fn process_lnd_event(
        &self,
        lnd_event: &crate::services::event_manager::LNDEvent,
    ) -> (
        EventType,
        EventSeverity,
        String,
        String,
        HashMap<String, Value>,
    ) {
        match lnd_event {
            crate::services::event_manager::LNDEvent::ChannelOpened {
                channel_id,
                counterparty_node_id,
            } => (
                EventType::ChannelOpened,
                EventSeverity::Info,
                "Channel Opened".to_string(),
                format!("New channel opened with {}", counterparty_node_id),
                HashMap::from([
                    (
                        "channel_id".to_string(),
                        Value::Number((*channel_id).into()),
                    ),
                    (
                        "counterparty_node_id".to_string(),
                        Value::String(counterparty_node_id.clone()),
                    ),
                ]),
            ),
            crate::services::event_manager::LNDEvent::ChannelClosed {
                channel_id,
                counterparty_node_id,
            } => (
                EventType::ChannelClosed,
                EventSeverity::Warning,
                "Channel Closed".to_string(),
                format!("Channel closed with {}", counterparty_node_id),
                HashMap::from([
                    (
                        "channel_id".to_string(),
                        Value::Number((*channel_id).into()),
                    ),
                    (
                        "counterparty_node_id".to_string(),
                        Value::String(counterparty_node_id.clone()),
                    ),
                ]),
            ),
            crate::services::event_manager::LNDEvent::InvoiceSettled {
                hash,
                value_msat,
                memo,
                ..
            } => (
                EventType::InvoiceSettled,
                EventSeverity::Info,
                "Invoice Settled".to_string(),
                format!("Invoice settled for {} msat", value_msat),
                HashMap::from([
                    ("hash".to_string(), Value::String(hex::encode(hash))),
                    (
                        "value_msat".to_string(),
                        Value::Number((*value_msat).into()),
                    ),
                    ("memo".to_string(), Value::String(memo.clone())),
                ]),
            ),
            // we will add other LND event types...
            _ => (
                EventType::InvoiceCreated,
                EventSeverity::Info,
                "Lightning Event".to_string(),
                "Lightning node event occurred".to_string(),
                HashMap::new(),
            ),
        }
    }

    /// Processes CLN-specific events.
    fn process_cln_event(
        &self,
        cln_event: &crate::services::event_manager::CLNEvent,
    ) -> (
        EventType,
        EventSeverity,
        String,
        String,
        HashMap<String, Value>,
    ) {
        match cln_event {
            crate::services::event_manager::CLNEvent::ChannelOpened {} => (
                EventType::ChannelOpened,
                EventSeverity::Info,
                "Channel Opened".to_string(),
                "New channel opened".to_string(),
                HashMap::new(),
            ),
            // crate::services::event_manager::CLNEvent::ChannelClosed {} => (
            //     EventType::ChannelClosed,
            //     EventSeverity::Warning,
            //     "Channel Closed".to_string(),
            //     "Channel closed".to_string(),
            //     HashMap::new(),
            // ),
            // crate::services::event_manager::CLNEvent::InvoiceSettled {} => (
            //     EventType::InvoiceSettled,
            //     EventSeverity::Info,
            //     "Invoice Settled".to_string(),
            //     "Invoice has been settled".to_string(),
            //     HashMap::new(),
            // ),
            // crate::services::event_manager::CLNEvent::InvoiceCreated {} => (
            //     EventType::InvoiceCreated,
            //     EventSeverity::Info,
            //     "Invoice Created".to_string(),
            //     "New invoice created".to_string(),
            //     HashMap::new(),
            // ),
            // crate::services::event_manager::CLNEvent::InvoiceCancelled {} => (
            //     EventType::InvoiceCancelled,
            //     EventSeverity::Warning,
            //     "Invoice Cancelled".to_string(),
            //     "Invoice has been cancelled".to_string(),
            //     HashMap::new(),
            // ),
            // crate::services::event_manager::CLNEvent::InvoiceAccepted {} => (
            //     EventType::InvoiceAccepted,
            //     EventSeverity::Info,
            //     "Invoice Accepted".to_string(),
            //     "Invoice has been accepted".to_string(),
            //     HashMap::new(),
            // ),
        }
    }

    /// Tests a notification endpoint.
    pub async fn test_notification(
        &self,
        pool: &SqlitePool,
        notification_id: &str,
        account_id: &str,
    ) -> ServiceResult<bool> {
        let notification_repo =
            crate::repositories::notification_repository::NotificationRepository::new(pool);
        let notification = notification_repo
            .get_notification_by_id(notification_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Notification", notification_id))?;

        // Verify that the notification belongs to the account
        if notification.account_id != account_id {
            return Err(ServiceError::not_found("Notification", notification_id));
        }

        let result = self
            .dispatcher
            .test_notification(&notification)
            .await
            .map_err(|e| ServiceError::validation(format!("Test failed: {}", e)))?;

        Ok(result)
    }
}
