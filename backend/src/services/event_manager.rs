//! Manages Events occuring on a lightning node.
//!
//! This module collects, aggregates and dispatches events occuring on a lightning node
//! in order to provide timely notifications for critical events.

use crate::services::node_manager::LightningClient;
use bitcoin::secp256k1::PublicKey;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use tokio;
use tokio::sync::{Mutex, mpsc};

use tokio_stream::StreamExt;

/// Represents different types of Lightning Network events that can be streamed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    /// Channel-related events (open, close, etc.)
    Channel,
    /// Invoice-related events (created, settled, etc.)
    Invoice,
    /// Payment-related events (sent, failed, etc.)
    Payment,
    /// Peer connection events
    Peer,
    /// Forward events (htlc forwarding)
    Forward,
    /// All event types
    All,
}

/// Configuration for filtering events during streaming
#[derive(Debug, Clone)]
pub struct EventFilter {
    /// Set of event types to include in the stream
    pub event_types: HashSet<EventType>,
    /// Whether to include all events (overrides event_types if true)
    pub include_all: bool,
}

impl EventFilter {
    /// Create a filter that includes all event types
    pub fn all() -> Self {
        Self {
            event_types: HashSet::new(),
            include_all: true,
        }
    }

    /// Create a filter for specific event types
    pub fn for_types(types: Vec<EventType>) -> Self {
        Self {
            event_types: types.into_iter().collect(),
            include_all: false,
        }
    }

    /// Create a filter for only channel events
    pub fn channels_only() -> Self {
        Self::for_types(vec![EventType::Channel])
    }

    /// Create a filter for only invoice events
    pub fn invoices_only() -> Self {
        Self::for_types(vec![EventType::Invoice])
    }

    /// Check if the filter should include a specific event type
    pub fn should_include(&self, event_type: &EventType) -> bool {
        self.include_all || self.event_types.contains(event_type)
    }

    /// Check if a NodeSpecificEvent should be included based on this filter
    pub fn matches_event(&self, event: &NodeSpecificEvent) -> bool {
        if self.include_all {
            return true;
        }

        match event {
            NodeSpecificEvent::LND(lnd_event) => match lnd_event {
                LNDEvent::ChannelOpened { .. } | LNDEvent::ChannelClosed { .. } => {
                    self.should_include(&EventType::Channel)
                }
                LNDEvent::InvoiceCreated { .. }
                | LNDEvent::InvoiceSettled { .. }
                | LNDEvent::InvoiceCancelled { .. }
                | LNDEvent::InvoiceAccepted { .. } => self.should_include(&EventType::Invoice),
            },
            NodeSpecificEvent::CLN(cln_event) => match cln_event {
                CLNEvent::ChannelOpened { .. } => self.should_include(&EventType::Channel),
            },
        }
    }
}

impl Default for EventFilter {
    fn default() -> Self {
        Self::all()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LNDEvent {
    ChannelOpened {
        active: bool,
        remote_pubkey: String,
        channel_point: String,
        chan_id: u64,
        capacity: i64,
        local_balance: i64,
        remote_balance: i64,
        total_satoshis_sent: i64,
        total_satoshis_received: i64,
    },
    ChannelClosed {
        channel_point: String,
        chan_id: u64,
        chain_hash: String,
        closing_tx_hash: String,
        remote_pubkey: String,
        capacity: i64,
        close_height: u32,
        settled_balance: i64,
        time_locked_balance: i64,
        close_type: i32,
        open_initiator: i32,
        close_initiator: i32,
    },
    InvoiceCreated {
        preimage: Vec<u8>,
        hash: Vec<u8>,
        value_msat: i64,
        state: i32,
        memo: String,
        creation_date: i64,
        payment_request: String,
    },
    InvoiceSettled {
        preimage: Vec<u8>,
        hash: Vec<u8>,
        value_msat: i64,
        state: i32,
        memo: String,
        creation_date: i64,
        payment_request: String,
    },
    InvoiceCancelled {
        preimage: Vec<u8>,
        hash: Vec<u8>,
        value_msat: i64,
        state: i32,
        memo: String,
        creation_date: i64,
        payment_request: String,
    },
    InvoiceAccepted {
        preimage: Vec<u8>,
        hash: Vec<u8>,
        value_msat: i64,
        state: i32,
        memo: String,
        creation_date: i64,
        payment_request: String,
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

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            node_id: PublicKey::from_str(
                "02000000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            filter: EventFilter::default(),
            buffer_size: Some(1000),
        }
    }
}

pub struct EventCollector {
    raw_event_sender: mpsc::Sender<NodeSpecificEvent>,
}

/// Configuration for starting event streams
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Node identifier for logging
    pub node_id: PublicKey,
    /// Event filter configuration
    pub filter: EventFilter,
    /// Buffer size for the event stream
    pub buffer_size: Option<usize>,
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
        // Start channel events stream
        self.start_channel_events_stream(node_id, lnd_node_.clone())
            .await;

        // Start invoice events stream
        self.start_invoice_events_stream(node_id, lnd_node_.clone())
            .await;
    }

    /// Start streaming only channel events for a node
    pub async fn start_channel_events(
        &self,
        node_id: PublicKey,
        lnd_node_: Arc<Mutex<Box<dyn LightningClient + Send + Sync + 'static>>>,
    ) {
        self.start_channel_events_stream(node_id, lnd_node_).await;
    }

    /// Start streaming only invoice events for a node
    pub async fn start_invoice_events(
        &self,
        node_id: PublicKey,
        lnd_node_: Arc<Mutex<Box<dyn LightningClient + Send + Sync + 'static>>>,
    ) {
        self.start_invoice_events_stream(node_id, lnd_node_).await;
    }

    /// Start streaming specific event types for a node
    pub async fn start_filtered_events(
        &self,
        node_id: PublicKey,
        event_types: Vec<EventType>,
        lnd_node_: Arc<Mutex<Box<dyn LightningClient + Send + Sync + 'static>>>,
    ) {
        // Start individual streams based on requested event types
        for event_type in event_types {
            match event_type {
                EventType::Channel => {
                    self.start_channel_events_stream(node_id, lnd_node_.clone())
                        .await;
                }
                EventType::Invoice => {
                    self.start_invoice_events_stream(node_id, lnd_node_.clone())
                        .await;
                }
                EventType::Payment => {
                    self.start_payment_events_stream(node_id, lnd_node_.clone())
                        .await;
                }
                EventType::All => {
                    self.start_channel_events_stream(node_id, lnd_node_.clone())
                        .await;
                    self.start_invoice_events_stream(node_id, lnd_node_.clone())
                        .await;
                    self.start_payment_events_stream(node_id, lnd_node_.clone())
                        .await;
                }
                _ => {
                    // Future event types can be added here
                    eprintln!("Event type {:?} not yet implemented", event_type);
                }
            }
        }
    }

    /// Start streaming payment events for a node
    pub async fn start_payment_events(
        &self,
        node_id: PublicKey,
        lnd_node_: Arc<Mutex<Box<dyn LightningClient + Send + Sync + 'static>>>,
    ) {
        self.start_payment_events_stream(node_id, lnd_node_).await;
    }

    async fn start_channel_events_stream(
        &self,
        node_id: PublicKey,
        lnd_node_: Arc<Mutex<Box<dyn LightningClient + Send + Sync + 'static>>>,
    ) {
        let sender = self.raw_event_sender.clone();
        let node_id_for_task = node_id;

        tokio::spawn(async move {
            // Get the channel events stream by temporarily acquiring the lock
            let channel_stream = {
                let mut lnd_node_guard = lnd_node_.lock().await;
                match lnd_node_guard.stream_channel_events_only().await {
                    Ok(stream) => stream,
                    Err(e) => {
                        eprintln!(
                            "Failed to start channel event stream for node {}: {:?}",
                            node_id_for_task, e
                        );
                        return;
                    }
                }
                // Lock is released here when lnd_node_guard goes out of scope
            };

            // Now stream events without holding the lock
            let mut event_stream = channel_stream;
            while let Some(event) = event_stream.next().await {
                if sender.send(event).await.is_err() {
                    eprintln!(
                        "Failed to send channel event for node {}. Receiver likely dropped.",
                        node_id_for_task
                    );
                    break;
                }
            }
            println!("Channel event stream for node {} ended.", node_id_for_task);
        });
    }

    async fn start_invoice_events_stream(
        &self,
        node_id: PublicKey,
        lnd_node_: Arc<Mutex<Box<dyn LightningClient + Send + Sync + 'static>>>,
    ) {
        let sender = self.raw_event_sender.clone();
        let node_id_for_task = node_id;

        tokio::spawn(async move {
            // Get the invoice events stream by temporarily acquiring the lock
            let invoice_stream = {
                let mut lnd_node_guard = lnd_node_.lock().await;
                match lnd_node_guard.stream_invoice_events_only().await {
                    Ok(stream) => stream,
                    Err(e) => {
                        eprintln!(
                            "Failed to start invoice event stream for node {}: {:?}",
                            node_id_for_task, e
                        );
                        return;
                    }
                }
                // Lock is released here when lnd_node_guard goes out of scope
            };

            // Now stream events without holding the lock
            let mut event_stream = invoice_stream;
            while let Some(event) = event_stream.next().await {
                if sender.send(event).await.is_err() {
                    eprintln!(
                        "Failed to send invoice event for node {}. Receiver likely dropped.",
                        node_id_for_task
                    );
                    break;
                }
            }
            println!("Invoice event stream for node {} ended.", node_id_for_task);
        });
    }

    async fn start_payment_events_stream(
        &self,
        node_id: PublicKey,
        lnd_node_: Arc<Mutex<Box<dyn LightningClient + Send + Sync + 'static>>>,
    ) {
        let sender = self.raw_event_sender.clone();
        let node_id_for_task = node_id;

        tokio::spawn(async move {
            // Get the payment events stream by temporarily acquiring the lock
            let payment_stream = {
                let mut lnd_node_guard = lnd_node_.lock().await;
                match lnd_node_guard.stream_payment_events_only().await {
                    Ok(stream) => stream,
                    Err(e) => {
                        eprintln!(
                            "Failed to start payment event stream for node {}: {:?}",
                            node_id_for_task, e
                        );
                        return;
                    }
                }
                // Lock is released here when lnd_node_guard goes out of scope
            };

            // Now stream events without holding the lock
            let mut event_stream = payment_stream;
            while let Some(event) = event_stream.next().await {
                if sender.send(event).await.is_err() {
                    eprintln!(
                        "Failed to send payment event for node {}. Receiver likely dropped.",
                        node_id_for_task
                    );
                    break;
                }
            }
            println!("Payment event stream for node {} ended.", node_id_for_task);
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
