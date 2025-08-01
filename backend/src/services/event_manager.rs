//! Manages Events occurring on a lightning node.
//!
//! This module collects, aggregates and dispatches events occurring on a lightning node
//! in order to provide timely notifications for critical events.
//!
//! ## Event Streaming Architecture
//!
//! The event system has been designed to avoid blocking issues that can occur when
//! multiple concurrent streams are attempted on Lightning nodes (particularly LND).
//! Instead of creating separate streams for different event types, this implementation:
//!
//! 1. Uses a single unified event stream per node via `stream_events()`
//! 2. Applies client-side filtering to dispatch only relevant events
//! 3. Prevents multiple concurrent streams per node using an active streams tracker
//! 4. Provides efficient event routing without overwhelming the Lightning node
//!
//! ## Usage Example
//!
//! ```rust
//! // Create event collector with a channel
//! let (sender, receiver) = mpsc::channel(1000);
//! let collector = EventCollector::new(sender);
//!
//! // Start streaming all events for a node
//! collector.start_sending(node_id, lightning_client).await;
//!
//! // Or start streaming only specific event types
//! collector.start_filtered_events(
//!     node_id,
//!     vec![EventType::Channel, EventType::Invoice],
//!     lightning_client
//! ).await;
//! ```

use crate::services::node_manager::LightningClient;
use bitcoin::secp256k1::PublicKey;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use tokio;
use tokio::sync::{Mutex, mpsc};
use tracing;

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

/// Collects and streams Lightning Network events from nodes.
///
/// The EventCollector manages event streaming from Lightning nodes while preventing
/// concurrent stream conflicts. It maintains a single unified stream per node and
/// applies client-side filtering to ensure efficient event processing.
pub struct EventCollector {
    /// Channel sender for dispatching processed events
    raw_event_sender: mpsc::Sender<NodeSpecificEvent>,
    /// Tracks which nodes currently have active event streams to prevent conflicts
    active_streams: Arc<Mutex<HashSet<PublicKey>>>,
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
    /// Creates a new EventCollector with the given event sender.
    ///
    /// # Arguments
    /// * `sender` - Channel sender for dispatching collected events
    pub fn new(sender: mpsc::Sender<NodeSpecificEvent>) -> Self {
        EventCollector {
            raw_event_sender: sender,
            active_streams: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Starts streaming all event types for the specified node.
    ///
    /// This method initiates a unified event stream that captures all Lightning
    /// events (channels, invoices, payments) for the given node.
    ///
    /// # Arguments
    /// * `node_id` - Public key identifier of the Lightning node
    /// * `lnd_node_` - Arc-wrapped Lightning client for the node
    pub async fn start_sending(
        &self,
        node_id: PublicKey,
        lnd_node_: Arc<Mutex<Box<dyn LightningClient + Send + Sync + 'static>>>,
    ) {
        self.start_unified_event_stream(node_id, lnd_node_, EventFilter::all())
            .await;
    }

    /// Start streaming only channel events for a node
    pub async fn start_channel_events(
        &self,
        node_id: PublicKey,
        lnd_node_: Arc<Mutex<Box<dyn LightningClient + Send + Sync + 'static>>>,
    ) {
        self.start_unified_event_stream(node_id, lnd_node_, EventFilter::channels_only())
            .await;
    }

    /// Start streaming only invoice events for a node
    pub async fn start_invoice_events(
        &self,
        node_id: PublicKey,
        lnd_node_: Arc<Mutex<Box<dyn LightningClient + Send + Sync + 'static>>>,
    ) {
        self.start_unified_event_stream(node_id, lnd_node_, EventFilter::invoices_only())
            .await;
    }

    /// Start streaming specific event types for a node
    pub async fn start_filtered_events(
        &self,
        node_id: PublicKey,
        event_types: Vec<EventType>,
        lnd_node_: Arc<Mutex<Box<dyn LightningClient + Send + Sync + 'static>>>,
    ) {
        let filter = EventFilter::for_types(event_types);
        self.start_unified_event_stream(node_id, lnd_node_, filter)
            .await;
    }

    /// Start streaming payment events for a node
    pub async fn start_payment_events(
        &self,
        node_id: PublicKey,
        lnd_node_: Arc<Mutex<Box<dyn LightningClient + Send + Sync + 'static>>>,
    ) {
        let filter = EventFilter::for_types(vec![EventType::Payment]);
        self.start_unified_event_stream(node_id, lnd_node_, filter)
            .await;
    }

    /// Starts a unified event stream that handles all event types and filters them client-side.
    ///
    /// This is the core method that prevents blocking issues by using a single stream
    /// per node instead of multiple concurrent streams. It:
    ///
    /// 1. Checks if a stream is already active for the node
    /// 2. Creates a single unified stream using `stream_events()`
    /// 3. Applies client-side filtering based on the provided EventFilter
    /// 4. Spawns a background task to handle events without blocking
    ///
    /// # Arguments
    /// * `node_id` - Public key identifier of the Lightning node
    /// * `lnd_node_` - Arc-wrapped Lightning client for the node
    /// * `filter` - Event filter to determine which events to process
    async fn start_unified_event_stream(
        &self,
        node_id: PublicKey,
        lnd_node_: Arc<Mutex<Box<dyn LightningClient + Send + Sync + 'static>>>,
        filter: EventFilter,
    ) {
        // Check if a stream is already active for this node
        {
            let mut active_streams = self.active_streams.lock().await;
            if active_streams.contains(&node_id) {
                tracing::warn!("Event stream already active for node {}", node_id);
                return;
            }
            active_streams.insert(node_id);
        }

        let sender = self.raw_event_sender.clone();
        let node_id_for_task = node_id;
        let active_streams = self.active_streams.clone();

        tokio::spawn(async move {
            // Get the unified events stream by temporarily acquiring the lock
            let event_stream = {
                let mut lnd_node_guard = lnd_node_.lock().await;
                match lnd_node_guard.stream_events().await {
                    Ok(stream) => stream,
                    Err(e) => {
                        tracing::error!(
                            "Failed to start unified event stream for node {}: {:?}",
                            node_id_for_task,
                            e
                        );
                        // Remove from active streams on error
                        let mut active_streams = active_streams.lock().await;
                        active_streams.remove(&node_id_for_task);
                        return;
                    }
                }
                // Lock is released here when lnd_node_guard goes out of scope
            };

            // Now stream events without holding the lock
            let mut stream = event_stream;
            while let Some(event) = stream.next().await {
                // Apply client-side filtering
                if filter.matches_event(&event) {
                    if sender.send(event).await.is_err() {
                        tracing::error!(
                            "Failed to send event for node {}. Receiver likely dropped.",
                            node_id_for_task
                        );
                        break;
                    }
                }
            }

            // Remove from active streams when done
            let mut active_streams = active_streams.lock().await;
            active_streams.remove(&node_id_for_task);
            tracing::info!("Unified event stream for node {} ended.", node_id_for_task);
        });
    }

    /// Stop the event stream for a specific node
    pub async fn stop_event_stream(&self, node_id: PublicKey) -> bool {
        let mut active_streams = self.active_streams.lock().await;
        let was_active = active_streams.remove(&node_id);
        if was_active {
            tracing::info!("Stopped event stream for node {}", node_id);
        } else {
            tracing::warn!("No active event stream found for node {}", node_id);
        }
        was_active
    }

    /// Check if an event stream is active for a specific node
    pub async fn is_stream_active(&self, node_id: PublicKey) -> bool {
        let active_streams = self.active_streams.lock().await;
        active_streams.contains(&node_id)
    }

    /// Get the list of nodes with active event streams
    pub async fn get_active_streams(&self) -> Vec<PublicKey> {
        let active_streams = self.active_streams.lock().await;
        active_streams.iter().cloned().collect()
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
