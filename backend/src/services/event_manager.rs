//! Manages Events occuring on a lightning node.
//!
//! This module collects, aggregates and dispatches events occuring on a lightning node
//! in order to provide timely notifications for critical events.

use std::collections::HashMap;

use crate::services::node_manager::LightningClient;
use bitcoin::secp256k1::PublicKey;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;
use tokio;
use tokio::sync::{Mutex, mpsc};
use tokio_stream::Stream;
use tokio_stream::StreamExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LNDEvent {
    ChannelOpened {
        channel_id: u64,
        counterparty_node_id: String,
    },
    ChannelClosed {
        channel_id: u64,
        counterparty_node_id: String,
    },
    InvoiceCreated {
        preimage: Vec<u8>,
        hash: Vec<u8>,
        value_msat: i64,
        state: i32,
        memo: String,
        creation_date: i64,
    },
    InvoiceSettled {
        preimage: Vec<u8>,
        hash: Vec<u8>,
        value_msat: i64,
        state: i32,
        memo: String,
        creation_date: i64,
    },
    InvoiceCancelled {
        preimage: Vec<u8>,
        hash: Vec<u8>,
        value_msat: i64,
        state: i32,
        memo: String,
        creation_date: i64,
    },
    InvoiceAccepted {
        preimage: Vec<u8>,
        hash: Vec<u8>,
        value_msat: i64,
        state: i32,
        memo: String,
        creation_date: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CLNEvent {
    ChannelOpened {},
}

#[derive(Debug, Clone)]
pub enum NodeSpecificEvent {
    LND(LNDEvent),
    CLN(CLNEvent),
}

pub struct EventCollector {
    raw_event_sender: mpsc::Sender<NodeSpecificEvent>,
}

impl EventCollector {
    pub fn new(sender: mpsc::Sender<NodeSpecificEvent>) -> Self {
        EventCollector {
            raw_event_sender: sender,
        }
    }

    pub async fn start_sending(
        &self,
        node_id: PublicKey,
        lnd_node_: Arc<Mutex<Box<dyn LightningClient + Send + Sync + 'static>>>,
    ) {
        let sender = self.raw_event_sender.clone();
        let node_id_for_task = node_id.clone();

        tokio::spawn(async move {
            let mut lnd_node_guard = lnd_node_.lock().await;
            let mut event_stream: Pin<
                Box<dyn Stream<Item = NodeSpecificEvent> + std::marker::Send>,
            > = lnd_node_guard.stream_events().await;

            while let Some(event) = event_stream.next().await {
                if sender.send(event).await.is_err() {
                    eprintln!(
                        "Failed to send event for node {}. Receiver likely dropped.",
                        node_id_for_task
                    );
                    break;
                }
            }
            println!("Event stream for node {} ended.", node_id_for_task);
        });
    }
}

#[derive(Clone)]
pub struct EventHandler {
    pool: Option<sqlx::SqlitePool>,
    account_id: Option<String>,
    user_id: Option<String>,
    node_id: Option<String>,
    node_alias: Option<String>,
}

impl EventHandler {
    pub fn new() -> Self {
        EventHandler {
            pool: None,
            account_id: None,
            user_id: None,
            node_id: None,
            node_alias: None,
        }
    }

    pub fn start_receiving(self, mut receiver: mpsc::Receiver<NodeSpecificEvent>) {
        let handler = self.clone();
        tokio::spawn(async move {
            while let Some(raw_event) = receiver.recv().await {
                handler.dispatch_event(raw_event).await;
            }
        });
    }

    pub fn with_context(
        pool: sqlx::SqlitePool,
        account_id: String,
        user_id: String,
        node_id: String,
        node_alias: String,
    ) -> Self {
        EventHandler {
            pool: Some(pool),
            account_id: Some(account_id),
            user_id: Some(user_id),
            node_id: Some(node_id),
            node_alias: Some(node_alias),
        }
    }

    pub async fn dispatch_event(&self, raw_event: NodeSpecificEvent) {
        // Only process if we have database context
        if let (Some(pool), Some(account_id), Some(user_id), Some(node_id), Some(node_alias)) = (
            &self.pool,
            &self.account_id,
            &self.user_id,
            &self.node_id,
            &self.node_alias,
        ) {
            let event_service = crate::services::event_service::EventService::new();

            if let Err(e) = event_service
                .process_lightning_event(
                    pool,
                    account_id.clone(),
                    user_id.clone(),
                    node_id.clone(),
                    node_alias.clone(),
                    &raw_event,
                )
                .await
            {
                tracing::error!(
                    "Failed to process lightning event for node {}: {}. Event: {:?}",
                    node_id,
                    e,
                    raw_event
                );
            }
        } else {
            tracing::debug!("Skipping event dispatch - no database context available");
        }
    }
}
