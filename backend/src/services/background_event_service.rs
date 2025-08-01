//! Background Event Service
//!
//! Manages Lightning Network event subscriptions in the background,
//! creating separate client instances for each event type to prevent blocking.

use crate::services::event_manager::{EventHandler, NodeSpecificEvent};
use crate::services::node_manager::{
    ClnConnection, ClnNode, ConnectionRequest, LightningClient, LndConnection, LndNode,
};
use crate::utils::jwt::Claims;
use crate::utils::{NodeId, NodeInfo};
use bitcoin::secp256k1::PublicKey;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, mpsc};
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;

#[derive(Debug, Clone)]
pub struct NodeCredentials {
    pub node_id: String,
    pub node_alias: String,
    pub node_type: String,
    pub address: String,
    pub macaroon: String,
    pub tls_cert: String,
    pub client_cert: Option<String>,
    pub client_key: Option<String>,
    pub ca_cert: Option<String>,
}

#[derive(Debug)]
pub struct EventSubscription {
    pub event_type: String,
    pub node_id: String,
    pub handle: JoinHandle<()>,
    pub is_active: bool,
}

pub struct BackgroundEventService {
    pool: SqlitePool,
    active_subscriptions: Arc<RwLock<HashMap<String, EventSubscription>>>,
    node_credentials: Arc<RwLock<HashMap<String, NodeCredentials>>>,
}

impl BackgroundEventService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            active_subscriptions: Arc::new(RwLock::new(HashMap::new())),
            node_credentials: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store node credentials for later use
    pub async fn store_node_credentials(&self, node_id: String, credentials: NodeCredentials) {
        let mut creds = self.node_credentials.write().await;
        creds.insert(node_id, credentials);
    }

    /// Subscribe to channel events for a specific node (internal use only)
    pub(crate) async fn subscribe_to_channel_events(
        &self,
        node_id: String,
        user_claims: Option<Claims>,
    ) -> Result<(), String> {
        let subscription_key = format!("{}_channels", node_id);

        // Check if already subscribed
        {
            let subscriptions = self.active_subscriptions.read().await;
            if let Some(sub) = subscriptions.get(&subscription_key) {
                if sub.is_active {
                    tracing::info!("Channel events already subscribed for node: {}", node_id);
                    return Ok(());
                }
            }
        }

        let credentials = {
            let creds = self.node_credentials.read().await;
            creds.get(&node_id).cloned()
        };

        let credentials =
            credentials.ok_or_else(|| format!("No credentials found for node: {}", node_id))?;

        let client = self.create_client_instance(&credentials).await?;

        let (sender, receiver) = mpsc::channel::<NodeSpecificEvent>(100);

        // Create event handler
        let handler = if let Some(claims) = user_claims {
            EventHandler::with_context(
                self.pool.clone(),
                claims.account_id,
                claims.sub,
                node_id.clone(),
                credentials.node_alias.clone(),
            )
        } else {
            EventHandler::new()
        };

        // Start processing events in background (this spawns its own task)
        handler.start_receiving(receiver);

        // Start the subscription in a separate task
        let client_arc = Arc::new(Mutex::new(client));
        let node_id_for_task = node_id.clone();

        let handle = tokio::spawn(async move {
            tracing::info!(
                "Starting channel events subscription for node: {}",
                node_id_for_task
            );

            // Directly stream channel events to bypass EventCollector limitations
            let event_stream = {
                let mut client_guard = client_arc.lock().await;
                match client_guard.stream_channel_events_only().await {
                    Ok(stream) => stream,
                    Err(e) => {
                        tracing::error!(
                            "Failed to start channel events stream for node {}: {:?}",
                            node_id_for_task,
                            e
                        );
                        return;
                    }
                }
            };

            // Stream events directly
            let mut stream = event_stream;
            while let Some(event) = stream.next().await {
                if sender.send(event).await.is_err() {
                    tracing::error!(
                        "Failed to send channel event for node {}. Receiver likely dropped.",
                        node_id_for_task
                    );
                    break;
                }
            }

            tracing::info!(
                "Channel events subscription ended for node: {}",
                node_id_for_task
            );
        });

        // Store the subscription
        let subscription = EventSubscription {
            event_type: "channels".to_string(),
            node_id: node_id.clone(),
            handle,
            is_active: true,
        };

        let mut subscriptions = self.active_subscriptions.write().await;
        subscriptions.insert(subscription_key, subscription);

        tracing::info!("Channel events subscription started for node: {}", node_id);
        Ok(())
    }

    /// Subscribe to invoice events for a specific node (internal use only)
    pub(crate) async fn subscribe_to_invoice_events(
        &self,
        node_id: String,
        user_claims: Option<Claims>,
    ) -> Result<(), String> {
        let subscription_key = format!("{}_invoices", node_id);

        // Check if already subscribed
        {
            let subscriptions = self.active_subscriptions.read().await;
            if let Some(sub) = subscriptions.get(&subscription_key) {
                if sub.is_active {
                    tracing::info!("Invoice events already subscribed for node: {}", node_id);
                    return Ok(());
                }
            }
        }

        let credentials = {
            let creds = self.node_credentials.read().await;
            creds.get(&node_id).cloned()
        };

        let credentials =
            credentials.ok_or_else(|| format!("No credentials found for node: {}", node_id))?;

        let client = self.create_client_instance(&credentials).await?;

        let (sender, receiver) = mpsc::channel::<NodeSpecificEvent>(100);

        // Create event handler
        let handler = if let Some(claims) = user_claims {
            EventHandler::with_context(
                self.pool.clone(),
                claims.account_id,
                claims.sub,
                node_id.clone(),
                credentials.node_alias.clone(),
            )
        } else {
            EventHandler::new()
        };

        // Start processing events in background (this spawns its own task)
        handler.start_receiving(receiver);

        // Start the subscription in a separate task
        let client_arc = Arc::new(Mutex::new(client));
        let node_id_for_task = node_id.clone();

        let handle = tokio::spawn(async move {
            tracing::info!(
                "Starting invoice events subscription for node: {}",
                node_id_for_task
            );

            // Directly stream invoice events to bypass EventCollector limitations
            let event_stream = {
                let mut client_guard = client_arc.lock().await;
                match client_guard.stream_invoice_events_only().await {
                    Ok(stream) => stream,
                    Err(e) => {
                        tracing::error!(
                            "Failed to start invoice events stream for node {}: {:?}",
                            node_id_for_task,
                            e
                        );
                        return;
                    }
                }
            };

            // Stream events directly
            let mut stream = event_stream;
            while let Some(event) = stream.next().await {
                if sender.send(event).await.is_err() {
                    tracing::error!(
                        "Failed to send invoice event for node {}. Receiver likely dropped.",
                        node_id_for_task
                    );
                    break;
                }
            }

            tracing::info!(
                "Invoice events subscription ended for node: {}",
                node_id_for_task
            );
        });

        // Store the subscription
        let subscription = EventSubscription {
            event_type: "invoices".to_string(),
            node_id: node_id.clone(),
            handle,
            is_active: true,
        };

        let mut subscriptions = self.active_subscriptions.write().await;
        subscriptions.insert(subscription_key, subscription);

        tracing::info!("Invoice events subscription started for node: {}", node_id);
        Ok(())
    }

    /// Subscribe to payment events for a specific node (internal use only)
    pub(crate) async fn subscribe_to_payment_events(
        &self,
        node_id: String,
        user_claims: Option<Claims>,
    ) -> Result<(), String> {
        let subscription_key = format!("{}_payments", node_id);

        // Check if already subscribed
        {
            let subscriptions = self.active_subscriptions.read().await;
            if let Some(sub) = subscriptions.get(&subscription_key) {
                if sub.is_active {
                    tracing::info!("Payment events already subscribed for node: {}", node_id);
                    return Ok(());
                }
            }
        }

        let credentials = {
            let creds = self.node_credentials.read().await;
            creds.get(&node_id).cloned()
        };

        let credentials =
            credentials.ok_or_else(|| format!("No credentials found for node: {}", node_id))?;

        let client = self.create_client_instance(&credentials).await?;

        let (sender, receiver) = mpsc::channel::<NodeSpecificEvent>(100);

        // Create event handler
        let handler = if let Some(claims) = user_claims {
            EventHandler::with_context(
                self.pool.clone(),
                claims.account_id,
                claims.sub,
                node_id.clone(),
                credentials.node_alias.clone(),
            )
        } else {
            EventHandler::new()
        };

        // Start processing events in background (this spawns its own task)
        handler.start_receiving(receiver);

        // Start the subscription in a separate task
        let client_arc = Arc::new(Mutex::new(client));
        let node_id_for_task = node_id.clone();

        let handle = tokio::spawn(async move {
            tracing::info!(
                "Starting payment events subscription for node: {}",
                node_id_for_task
            );

            // Directly stream payment events to bypass EventCollector limitations
            let event_stream = {
                let mut client_guard = client_arc.lock().await;
                match client_guard.stream_payment_events_only().await {
                    Ok(stream) => stream,
                    Err(e) => {
                        tracing::error!(
                            "Failed to start payment events stream for node {}: {:?}",
                            node_id_for_task,
                            e
                        );
                        return;
                    }
                }
            };

            // Stream events directly
            let mut stream = event_stream;
            while let Some(event) = stream.next().await {
                if sender.send(event).await.is_err() {
                    tracing::error!(
                        "Failed to send payment event for node {}. Receiver likely dropped.",
                        node_id_for_task
                    );
                    break;
                }
            }

            tracing::info!(
                "Payment events subscription ended for node: {}",
                node_id_for_task
            );
        });

        // Store the subscription
        let subscription = EventSubscription {
            event_type: "payments".to_string(),
            node_id: node_id.clone(),
            handle,
            is_active: true,
        };

        let mut subscriptions = self.active_subscriptions.write().await;
        subscriptions.insert(subscription_key, subscription);

        tracing::info!("Payment events subscription started for node: {}", node_id);
        Ok(())
    }

    /// Unsubscribe from specific event type for a node (internal use only)
    pub(crate) async fn unsubscribe_from_events(
        &self,
        node_id: String,
        event_type: String,
    ) -> Result<(), String> {
        let subscription_key = format!("{}_{}", node_id, event_type);

        let mut subscriptions = self.active_subscriptions.write().await;
        if let Some(mut subscription) = subscriptions.remove(&subscription_key) {
            subscription.handle.abort();
            subscription.is_active = false;
            tracing::info!(
                "Unsubscribed from {} events for node: {}",
                event_type,
                node_id
            );
            Ok(())
        } else {
            Err(format!(
                "No active subscription found for {} events on node: {}",
                event_type, node_id
            ))
        }
    }

    /// Get all active subscriptions for a node
    pub async fn get_active_subscriptions(&self, node_id: String) -> Vec<String> {
        let subscriptions = self.active_subscriptions.read().await;
        subscriptions
            .values()
            .filter(|sub| sub.node_id == node_id && sub.is_active)
            .map(|sub| sub.event_type.clone())
            .collect()
    }

    /// Create a new client instance for event subscriptions
    async fn create_client_instance(
        &self,
        credentials: &NodeCredentials,
    ) -> Result<Box<dyn LightningClient + Send + Sync>, String> {
        match credentials.node_type.as_str() {
            "lnd" => {
                let pubkey = credentials
                    .node_id
                    .parse::<PublicKey>()
                    .map_err(|e| format!("Invalid pubkey: {}", e))?;

                let connection = LndConnection {
                    id: NodeId::PublicKey(pubkey),
                    address: credentials.address.clone(),
                    macaroon: credentials.macaroon.clone(),
                    cert: credentials.tls_cert.clone(),
                };

                let node = LndNode::new(connection)
                    .await
                    .map_err(|e| format!("Failed to create LND client: {}", e))?;

                Ok(Box::new(node))
            }
            "cln" => {
                let pubkey = credentials
                    .node_id
                    .parse::<PublicKey>()
                    .map_err(|e| format!("Invalid pubkey: {}", e))?;

                let connection = ClnConnection {
                    id: NodeId::PublicKey(pubkey),
                    address: credentials.address.clone(),
                    ca_cert: credentials
                        .ca_cert
                        .as_ref()
                        .unwrap_or(&String::new())
                        .clone(),
                    client_cert: credentials
                        .client_cert
                        .as_ref()
                        .unwrap_or(&String::new())
                        .clone(),
                    client_key: credentials
                        .client_key
                        .as_ref()
                        .unwrap_or(&String::new())
                        .clone(),
                };

                let node = ClnNode::new(connection)
                    .await
                    .map_err(|e| format!("Failed to create CLN client: {}", e))?;

                Ok(Box::new(node))
            }
            _ => Err(format!("Unsupported node type: {}", credentials.node_type)),
        }
    }

    /// Initialize background service after node authentication
    pub async fn initialize_for_node(
        &self,
        node_info: &NodeInfo,
        connection_request: &ConnectionRequest,
        user_claims: Option<Claims>,
    ) -> Result<(), String> {
        let node_id = node_info.pubkey.to_string();

        // Store credentials for later use
        let credentials = self.extract_credentials(connection_request, node_info);
        self.store_node_credentials(node_id.clone(), credentials)
            .await;

        // Start background task to monitor for event triggers
        let service = Arc::new(self.clone());
        let node_id_clone = node_id.clone();

        tokio::spawn(async move {
            tracing::info!(
                "Background event monitor started for node: {}",
                node_id_clone
            );

            // Simulate event triggers - in practice, these would be triggered by actual events
            // For now, we'll start channel and invoice subscriptions immediately but separately

            // Subscribe to channel events
            if let Err(e) = service
                .subscribe_to_channel_events(node_id_clone.clone(), user_claims.clone())
                .await
            {
                tracing::error!("Failed to subscribe to channel events: {}", e);
            }

            // Subscribe to invoice events (in a separate task to prevent blocking)
            let service_clone = service.clone();
            let node_id_clone2 = node_id_clone.clone();
            let user_claims_clone = user_claims.clone();
            tokio::spawn(async move {
                if let Err(e) = service_clone
                    .subscribe_to_invoice_events(node_id_clone2, user_claims_clone)
                    .await
                {
                    tracing::error!("Failed to subscribe to invoice events: {}", e);
                }
            });

            // Subscribe to payment events (in another separate task)
            let service_clone2 = service.clone();
            let node_id_clone3 = node_id_clone.clone();
            let user_claims_clone2 = user_claims.clone();
            tokio::spawn(async move {
                if let Err(e) = service_clone2
                    .subscribe_to_payment_events(node_id_clone3, user_claims_clone2)
                    .await
                {
                    tracing::error!("Failed to subscribe to payment events: {}", e);
                }
            });
        });

        Ok(())
    }

    /// Extract credentials from connection request
    fn extract_credentials(
        &self,
        connection_request: &ConnectionRequest,
        node_info: &NodeInfo,
    ) -> NodeCredentials {
        match connection_request {
            ConnectionRequest::Lnd(lnd_conn) => NodeCredentials {
                node_id: node_info.pubkey.to_string(),
                node_alias: node_info.alias.clone(),
                node_type: "lnd".to_string(),
                address: lnd_conn.address.clone(),
                macaroon: lnd_conn.macaroon.clone(),
                tls_cert: lnd_conn.cert.clone(),
                client_cert: None,
                client_key: None,
                ca_cert: None,
            },
            ConnectionRequest::Cln(cln_conn) => NodeCredentials {
                node_id: node_info.pubkey.to_string(),
                node_alias: node_info.alias.clone(),
                node_type: "cln".to_string(),
                address: cln_conn.address.clone(),
                macaroon: String::new(),
                tls_cert: String::new(),
                client_cert: Some(cln_conn.client_cert.clone()),
                client_key: Some(cln_conn.client_key.clone()),
                ca_cert: Some(cln_conn.ca_cert.clone()),
            },
        }
    }

    /// Subscribe to specific event type on-demand

    /// Cleanup all subscriptions for a node (internal use only)
    pub(crate) async fn cleanup_node_subscriptions(&self, node_id: String) -> Result<(), String> {
        let event_types = vec!["channels", "invoices", "payments"];

        for event_type in event_types {
            if let Err(e) = self
                .unsubscribe_from_events(node_id.clone(), event_type.to_string())
                .await
            {
                tracing::warn!("Failed to unsubscribe from {} events: {}", event_type, e);
            }
        }

        // Remove credentials
        let mut creds = self.node_credentials.write().await;
        creds.remove(&node_id);

        tracing::info!("Cleaned up all subscriptions for node: {}", node_id);
        Ok(())
    }
}

impl Clone for BackgroundEventService {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            active_subscriptions: self.active_subscriptions.clone(),
            node_credentials: self.node_credentials.clone(),
        }
    }
}
