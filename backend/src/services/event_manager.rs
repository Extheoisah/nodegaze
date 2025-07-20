//! Manages Events occuring on a lightning node.
//! 
//! This module collects, aggregates and dispatches events occuring on a lightning node
//! in order to provide timely notifications for critical events.

use std::collections::HashMap;

use bitcoin::secp256k1::PublicKey;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use tokio;
use tokio::{sync::{mpsc, Mutex}};
use tokio_stream::StreamExt;
use tokio_stream::Stream;
use std::pin::Pin;
use std::sync::Arc;
use crate::services::node_manager::LightningClient;

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
    ChannelOpened { },
    ChannelClosed { },
    InvoiceCreated { },
    InvoiceSettled { },
    InvoiceCancelled { },
    InvoiceAccepted { },
}

#[derive(Debug)]
pub enum NodeSpecificEvent {
    LND(LNDEvent),
    CLN(CLNEvent),
}

pub struct EventCollector {
    raw_event_sender: mpsc::Sender<NodeSpecificEvent>,
}

impl EventCollector {
    pub fn new(sender: mpsc::Sender<NodeSpecificEvent>) -> Self {
        EventCollector { raw_event_sender: sender }
    }

    pub async fn start_sending(
        &self,
        node_id: PublicKey,
        lnd_node_: Arc<Mutex<Box<dyn LightningClient  + Send + Sync + 'static>>>,
    ) {
        let sender = self.raw_event_sender.clone();
        let node_id_for_task = node_id.clone();

        tokio::spawn(async move {
            let mut lnd_node_guard = lnd_node_.lock().await;
            let mut event_stream: Pin<Box<dyn Stream<Item = NodeSpecificEvent> + std::marker::Send>> = lnd_node_guard.stream_events().await;

            while let Some(event) = event_stream.next().await {
                if sender.send(event).await.is_err() {
                    eprintln!("Failed to send event for node {}. Receiver likely dropped.", node_id_for_task);
                    break;
                }
            }
            println!("Event stream for node {} ended.", node_id_for_task);
        });
    }
}

pub struct EventProcessor {}

impl EventProcessor {
    pub fn new() -> Self {
        EventProcessor { }
    }

    pub fn start_receiving(&self, mut receiver: mpsc::Receiver<NodeSpecificEvent>) {
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                println!("Received event: {:?}", event);
            }
            println!("Receiver task for channel ended.");
        });
    }
}

pub struct EventDispatcher {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub severity: EventSeverity,
    pub node_id: String,
    pub node_alias: Option<String>,
    pub data: HashMap<String, serde_json::Value>,
}


