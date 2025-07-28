//! Manages connections and interactions with Lightning Network nodes (LND and CLN).
//!
//! This module defines connection structures (`LndConnection`, `ClnConnection`),
//! manages authenticated node instances (`LndNode`, `ClnNode`), handles their lifecycle,
//! and provides methods for interacting with the Lightning node RPCs.

use crate::{
    errors::LightningError,
    services::event_manager::{CLNEvent, LNDEvent, NodeSpecificEvent},
    utils::{
        self, ChannelDetails, ChannelState, ChannelSummary, CustomInvoice, Feature, Hop,
        InvoiceHtlc, InvoiceStatus, NodeId, NodeInfo, NodePolicy, PaymentDetails, PaymentHtlc,
        PaymentState, PaymentSummary, Route, ShortChannelID,
    },
};

use async_stream::stream;
use async_trait::async_trait;
use bitcoin::{Network, OutPoint, Txid, hashes::Hash, secp256k1::PublicKey};
use cln_grpc::pb::{
    GetinfoRequest, ListchannelsRequest, ListnodesRequest, ListpeerchannelsRequest,
    node_client::NodeClient,
};
use futures::stream::{SelectAll, StreamExt};
use hex;
use lightning::ln::{PaymentHash, features::NodeFeatures};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    convert::TryFrom,
    pin::Pin,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::time::Duration;
use tokio::{
    fs::File,
    io::{AsyncReadExt, Error},
    sync::Mutex,
    time::sleep,
};
use tokio_stream::Stream;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};
use tonic_lnd::{
    Client,
    lnrpc::{
        ChannelEventSubscription, ChannelEventUpdate, ChannelGraphRequest, GetInfoRequest, Invoice,
        InvoiceSubscription, ListChannelsRequest, ListPaymentsRequest, NodeInfoRequest,
        channel_event_update::{Channel as EventChannel, UpdateType as LndChannelUpdateType},
        invoice::InvoiceState,
        payment::PaymentStatus,
    },
    tonic::Streaming,
};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ConnectionRequest {
    Lnd(LndConnection),
    Cln(ClnConnection),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LndConnection {
    #[serde(with = "utils::serde_node_id")]
    pub id: NodeId,
    #[serde(with = "utils::serde_address")]
    pub address: String,
    #[serde(deserialize_with = "utils::deserialize_path")]
    pub macaroon: String,
    #[serde(deserialize_with = "utils::deserialize_path")]
    pub cert: String,
}

pub struct LndNode {
    pub client: Mutex<Client>,
    pub info: NodeInfo,
    network: Network,
}

/// Parses the node features from the format returned by LND gRPC to LDK NodeFeatures
fn parse_node_features(features: HashSet<u32>) -> NodeFeatures {
    let mut flags = vec![0; 256];

    for f in features.into_iter() {
        let byte_offset = (f / 8) as usize;
        let mask = 1 << (f % 8);
        if flags.len() <= byte_offset {
            flags.resize(byte_offset + 1, 0u8);
        }

        flags[byte_offset] |= mask
    }

    NodeFeatures::from_le_bytes(flags)
}

impl LndNode {
    pub async fn new(connection: LndConnection) -> Result<Self, LightningError> {
        let mut client =
            tonic_lnd::connect(connection.address, connection.cert, connection.macaroon)
                .await
                .map_err(|err| LightningError::ConnectionError(err.to_string()))?;

        let info = client
            .lightning()
            .get_info(GetInfoRequest {})
            .await
            .map_err(|err| LightningError::GetInfoError(err.to_string()))?
            .into_inner();

        let mut alias = info.alias;
        let pubkey = PublicKey::from_str(&info.identity_pubkey)
            .map_err(|err| LightningError::GetInfoError(err.to_string()))?;
        connection.id.validate(&pubkey, &mut alias)?;

        let network = {
            if info.chains.is_empty() {
                return Err(LightningError::GetInfoError(
                    "node is not connected to any chain".to_string(),
                ));
            } else if info.chains.len() > 1 {
                return Err(LightningError::GetInfoError(format!(
                    "node is connected to more than one chain: {:?}",
                    info.chains.iter().map(|c| c.chain.to_string())
                )));
            }

            Network::from_str(match info.chains[0].network.as_str() {
                "mainnet" => "bitcoin",
                x => x,
            })
            .map_err(|e| LightningError::GetInfoError(e.to_string()))?
        };

        Ok(Self {
            client: Mutex::new(client),
            info: NodeInfo {
                pubkey,
                features: parse_node_features(info.features.keys().cloned().collect()),
                alias,
            },
            network,
        })
    }

    async fn stream_channel_events(&self) -> Result<Streaming<ChannelEventUpdate>, LightningError> {
        println!("Attempting to subscribe to LND channel events...");
        let channel_event_stream: Streaming<ChannelEventUpdate> = match self
            .client
            .lock()
            .await
            .lightning()
            .subscribe_channel_events(ChannelEventSubscription {})
            .await
        {
            Ok(response) => {
                println!("LND channel events subscription successful: {:?}", response);
                response.into_inner()
            }
            Err(e) => {
                eprintln!("Error subscribing to LND channel events: {:?}", e);
                return Err(LightningError::StreamingError(format!("{}", e)));
            }
        };
        println!("Finished channel events subscription block.");
        Ok(channel_event_stream)
    }

    async fn stream_invoice_events(&self) -> Result<Streaming<Invoice>, LightningError> {
        println!("Attempting to subscribe to LND invoice events...");
        let invoice_event_stream = match self
            .client
            .lock()
            .await
            .lightning()
            .subscribe_invoices(InvoiceSubscription {
                add_index: 0,
                settle_index: 0,
            })
            .await
        {
            Ok(response) => response.into_inner(),
            Err(e) => {
                eprintln!("Error subscribing to LND invoice events: {:?}", e);
                return Err(LightningError::StreamingError(format!("{}", e)));
            }
        };
        println!("Finished invoice events subscription block.");
        Ok(invoice_event_stream)
    }

    async fn get_lightning_stub(&self) -> tonic_lnd::LightningClient {
        let mut client = self.client.lock().await;
        client.lightning().clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClnConnection {
    #[serde(with = "utils::serde_node_id")]
    pub id: NodeId,
    #[serde(with = "utils::serde_address")]
    pub address: String,
    #[serde(deserialize_with = "utils::deserialize_path")]
    pub ca_cert: String,
    #[serde(deserialize_with = "utils::deserialize_path")]
    pub client_cert: String,
    #[serde(deserialize_with = "utils::deserialize_path")]
    pub client_key: String,
}

pub struct ClnNode {
    pub client: Mutex<NodeClient<Channel>>,
    pub info: NodeInfo,
    network: Network,
}

impl ClnNode {
    pub async fn new(connection: ClnConnection) -> Result<Self, LightningError> {
        let tls = ClientTlsConfig::new()
            .domain_name("cln")
            .identity(Identity::from_pem(
                reader(&connection.client_cert).await.map_err(|err| {
                    LightningError::ConnectionError(format!(
                        "Cannot load client certificate: {}",
                        err
                    ))
                })?,
                reader(&connection.client_key).await.map_err(|err| {
                    LightningError::ConnectionError(format!("Cannot load client key: {}", err))
                })?,
            ))
            .ca_certificate(Certificate::from_pem(
                reader(&connection.ca_cert).await.map_err(|err| {
                    LightningError::ConnectionError(format!("Cannot load CA certificate: {}", err))
                })?,
            ));

        let grpc_connection = Channel::from_shared(connection.address)
            .map_err(|err| LightningError::ConnectionError(err.to_string()))?
            .tls_config(tls)
            .map_err(|err| {
                LightningError::ConnectionError(format!("Cannot establish tls connection: {}", err))
            })?
            .connect()
            .await
            .map_err(|err| {
                LightningError::ConnectionError(format!("Cannot connect to gRPC server: {}", err))
            })?;
        let client = Mutex::new(NodeClient::new(grpc_connection));
        let info = client
            .lock()
            .await
            .getinfo(GetinfoRequest {})
            .await
            .map_err(|err| LightningError::GetInfoError(err.to_string()))?
            .into_inner();

        let pubkey = PublicKey::from_slice(&info.id)
            .map_err(|err| LightningError::GetInfoError(err.to_string()))?;
        let mut alias = info.alias.unwrap_or_default();
        connection.id.validate(&pubkey, &mut alias)?;

        let features = match info.our_features {
            Some(features) => NodeFeatures::from_be_bytes(features.node),
            None => NodeFeatures::empty(),
        };

        let network = Network::from_core_arg(&info.network)
            .map_err(|err| LightningError::GetInfoError(err.to_string()))?;

        Ok(Self {
            client,
            info: NodeInfo {
                pubkey,
                features,
                alias,
            },
            network,
        })
    }

    /// Fetch channels belonging to the local node, initiated locally if is_source is true, and initiated remotely if
    /// is_source is false. Introduced as a helper function because CLN doesn't have a single API to list all of our
    /// node's channels.
    async fn node_channels(&self, is_source: bool) -> Result<Vec<u64>, LightningError> {
        let req = if is_source {
            ListchannelsRequest {
                source: Some(self.info.pubkey.serialize().to_vec()),
                ..Default::default()
            }
        } else {
            ListchannelsRequest {
                destination: Some(self.info.pubkey.serialize().to_vec()),
                ..Default::default()
            }
        };

        let resp = self
            .client
            .lock()
            .await
            .list_channels(req)
            .await
            .map_err(|err| LightningError::ListChannelsError(err.to_string()))?
            .into_inner();

        Ok(resp
            .channels
            .into_iter()
            .map(|channel| channel.amount_msat.unwrap_or_default().msat)
            .collect())
    }

    async fn get_client_stub(&self) -> NodeClient<Channel> {
        self.client.lock().await.clone()
    }
}

async fn reader(filename: &str) -> Result<Vec<u8>, Error> {
    let mut file = File::open(filename).await?;
    let mut contents = vec![];
    file.read_to_end(&mut contents).await?;
    Ok(contents)
}

/// Unified interface for Lightning Network node operations across different implementations.
#[async_trait]
pub trait LightningClient: Send {
    /// Returns information about the node.
    fn get_info(&self) -> &NodeInfo;
    /// Retrieves the Bitcoin network the node is connected to.
    async fn get_network(&self) -> Result<Network, LightningError>;
    /// Fetches public information about a Lightning node by its public key.
    async fn get_node_info(&self, node_id: &PublicKey) -> Result<NodeInfo, LightningError>;
    /// Lists all channels, returning only their capacities in millisatoshis.
    async fn list_channels(&self) -> Result<Vec<ChannelSummary>, LightningError>;
    /// Gets detailed information about a specific channel.
    async fn get_channel_info(
        &self,
        channel_id: &ShortChannelID,
    ) -> Result<ChannelDetails, LightningError>;
    /// Gets detailed information about a specific payment by its hash.
    async fn get_payment_details(
        &self,
        payment_hash: &PaymentHash,
    ) -> Result<PaymentDetails, LightningError>;
    async fn list_payments(&self) -> Result<Vec<PaymentSummary>, LightningError>;
    /// Returns a stream of raw events from the lightning node.
    async fn stream_events(
        &mut self,
    ) -> Result<Pin<Box<dyn Stream<Item = NodeSpecificEvent> + Send>>, LightningError>;
    /// Lists all invoices.
    async fn list_invoices(&self) -> Result<Vec<CustomInvoice>, LightningError>;
    /// Gets detailed information about a specific invoice by its payment hash.
    async fn get_invoice_details(
        &self,
        payment_hash: &PaymentHash,
    ) -> Result<CustomInvoice, LightningError>;
}

#[async_trait]
impl LightningClient for LndNode {
    /// Returns cached node information (node_id, alias, features) that was retrieved
    /// during node initialization. This avoids repeated RPC calls for static node data.
    fn get_info(&self) -> &NodeInfo {
        &self.info
    }

    async fn get_network(&self) -> Result<Network, LightningError> {
        let mut client = self.client.lock().await;
        let info = client
            .lightning()
            .get_info(GetInfoRequest {})
            .await
            .map_err(|err| LightningError::GetInfoError(err.to_string()))?
            .into_inner();

        if info.chains.is_empty() {
            return Err(LightningError::ValidationError(format!(
                "{} is not connected any chain",
                self.get_info()
            )));
        } else if info.chains.len() > 1 {
            return Err(LightningError::ValidationError(format!(
                "{} is connected to more than one chain: {:?}",
                self.get_info(),
                info.chains.iter().map(|c| c.chain.to_string())
            )));
        }

        Ok(Network::from_str(match info.chains[0].network.as_str() {
            "mainnet" => "bitcoin",
            x => x,
        })
        .map_err(|err| LightningError::ValidationError(err.to_string()))?)
    }

    async fn get_node_info(&self, node_id: &PublicKey) -> Result<NodeInfo, LightningError> {
        let mut client = self.client.lock().await;
        let node_info = client
            .lightning()
            .get_node_info(NodeInfoRequest {
                pub_key: node_id.to_string(),
                include_channels: false,
            })
            .await
            .map_err(|err| LightningError::GetNodeInfoError(err.to_string()))?
            .into_inner();

        if let Some(node_info) = node_info.node {
            Ok(NodeInfo {
                pubkey: *node_id,
                alias: node_info.alias,
                features: parse_node_features(node_info.features.keys().cloned().collect()),
            })
        } else {
            Err(LightningError::GetNodeInfoError(
                "Node not found".to_string(),
            ))
        }
    }

    async fn list_channels(&self) -> Result<Vec<ChannelSummary>, LightningError> {
        let mut lightning_stub = self.get_lightning_stub().await;
        let response = lightning_stub
            .list_channels(ListChannelsRequest::default())
            .await
            .map_err(|e| LightningError::RpcError(e.to_string()))?
            .into_inner();

        Ok(response
            .channels
            .into_iter()
            .map(|channel| {
                let channel_state = match channel.active {
                    true => ChannelState::Active,
                    false => ChannelState::Disabled,
                };

                ChannelSummary {
                    chan_id: ShortChannelID(channel.chan_id),
                    alias: None,
                    channel_state,
                    private: channel.private,
                    remote_balance: channel.remote_balance.try_into().unwrap_or(0),
                    local_balance: channel.local_balance.try_into().unwrap_or(0),
                    capacity: channel.capacity.try_into().unwrap_or(0),
                    creation_date: None,
                    uptime: Some(channel.uptime as u64),
                }
            })
            .collect())
    }

    async fn get_channel_info(
        &self,
        channel_id: &ShortChannelID,
    ) -> Result<ChannelDetails, LightningError> {
        let mut lightning_stub = self.get_lightning_stub().await;

        // Fetch basic channel info
        let response = lightning_stub
            .list_channels(ListChannelsRequest {
                active_only: false,
                ..Default::default()
            })
            .await
            .map_err(|e| LightningError::ChannelError(format!("LND list_channels error: {}", e)))?;

        let channel_opt = response
            .into_inner()
            .channels
            .into_iter()
            .find(|c| c.chan_id == channel_id.0);

        match channel_opt {
            Some(c) => {
                let channel_point = parse_channel_point(&c.channel_point)?;
                let remote_pubkey = PublicKey::from_str(&c.remote_pubkey).map_err(|e| {
                    LightningError::ChannelError(format!("Invalid remote pubkey: {}", e))
                })?;

                // Get policies from describe_graph
                let (node1_policy, node2_policy) = match lightning_stub
                    .describe_graph(ChannelGraphRequest {
                        include_unannounced: false,
                    })
                    .await
                {
                    Ok(graph_response) => {
                        let edges = graph_response.into_inner().edges;
                        if let Some(e) = edges.into_iter().find(|e| e.channel_id == channel_id.0) {
                            let node1_pubkey =
                                PublicKey::from_str(&e.node1_pub).unwrap_or(remote_pubkey);
                            let node2_pubkey =
                                PublicKey::from_str(&e.node2_pub).unwrap_or(self.info.pubkey);

                            let p1 = e.node1_policy.as_ref().map(|p| NodePolicy {
                                pubkey: node1_pubkey,
                                fee_base_msat: p.fee_base_msat as u64,
                                fee_rate_milli_msat: p.fee_rate_milli_msat as u64,
                                min_htlc_msat: p.min_htlc as u64,
                                max_htlc_msat: if p.max_htlc_msat > 0 {
                                    Some(p.max_htlc_msat as u64)
                                } else {
                                    None
                                },
                                time_lock_delta: p.time_lock_delta as u16,
                                disabled: p.disabled,
                                last_update: Some(
                                    SystemTime::UNIX_EPOCH
                                        + Duration::from_secs(p.last_update as u64),
                                ),
                            });

                            let p2 = e.node2_policy.as_ref().map(|p| NodePolicy {
                                pubkey: node2_pubkey,
                                fee_base_msat: p.fee_base_msat as u64,
                                fee_rate_milli_msat: p.fee_rate_milli_msat as u64,
                                min_htlc_msat: p.min_htlc as u64,
                                max_htlc_msat: if p.max_htlc_msat > 0 {
                                    Some(p.max_htlc_msat as u64)
                                } else {
                                    None
                                },
                                time_lock_delta: p.time_lock_delta as u16,
                                disabled: p.disabled,
                                last_update: Some(
                                    SystemTime::UNIX_EPOCH
                                        + Duration::from_secs(p.last_update as u64),
                                ),
                            });

                            (p1, p2)
                        } else {
                            (None, None)
                        }
                    }
                    Err(_) => (None, None),
                };

                Ok(ChannelDetails {
                    channel_id: ShortChannelID(c.chan_id),
                    local_balance_sat: c.local_balance.try_into().unwrap_or(0),
                    remote_balance_sat: c.remote_balance.try_into().unwrap_or(0),
                    capacity_sat: c.capacity.try_into().unwrap_or(0),
                    active: c.active,
                    private: c.private,
                    remote_pubkey,
                    commit_fee_sat: Some(c.commit_fee as u64),
                    local_chan_reserve_sat: Some(
                        c.local_constraints
                            .as_ref()
                            .map(|lc| lc.chan_reserve_sat)
                            .unwrap_or(0),
                    ),
                    remote_chan_reserve_sat: Some(
                        c.remote_constraints
                            .as_ref()
                            .map(|rc| rc.chan_reserve_sat)
                            .unwrap_or(0),
                    ),
                    num_updates: Some(c.num_updates),
                    total_satoshis_sent: Some(c.total_satoshis_sent as u64),
                    total_satoshis_received: Some(c.total_satoshis_received as u64),
                    channel_age_blocks: c.lifetime.try_into().ok(),
                    last_update: None,
                    opening_cost_sat: None,
                    initiator: Some(c.initiator),
                    txid: Some(channel_point.txid),
                    vout: Some(channel_point.vout),
                    node1_policy,
                    node2_policy,
                })
            }
            None => Err(LightningError::ChannelError(
                "Channel not found".to_string(),
            )),
        }
    }

    async fn get_payment_details(
        &self,
        payment_hash: &PaymentHash,
    ) -> Result<PaymentDetails, LightningError> {
        let mut lightning_stub = self.get_lightning_stub().await;
        let response = lightning_stub
            .list_payments(ListPaymentsRequest {
                include_incomplete: true,
                ..Default::default()
            })
            .await
            .map_err(|e| {
                tracing::error!("list_payments RPC failed: {}", e);
                LightningError::RpcError(format!("LND list_payments error: {}", e))
            })?
            .into_inner();

        let hex_hash = hex::encode(payment_hash.0);

        let Some(payment) = response
            .payments
            .into_iter()
            .find(|p| p.payment_hash == hex_hash)
        else {
            return Err(LightningError::NotFound(format!(
                "Payment {} not found",
                hex_hash
            )));
        };

        let state = match PaymentStatus::try_from(payment.status).unwrap_or(PaymentStatus::Unknown)
        {
            PaymentStatus::Unknown | PaymentStatus::InFlight => PaymentState::Inflight,
            PaymentStatus::Succeeded => PaymentState::Settled,
            PaymentStatus::Failed => PaymentState::Failed,
        };

        let creation_time = payment
            .creation_time_ns
            .try_into()
            .ok()
            .map(|ns: u64| UNIX_EPOCH + Duration::from_nanos(ns));

        // Process HTLCs and extract destination pubkey from the last hop
        let (htlcs, destination_pubkey) = {
            let mut destination_pubkey = None;
            let htlcs = payment
                .htlcs
                .into_iter()
                .map(|htlc| {
                    let route = htlc.route.map(|r| {
                        // Get destination pubkey from last hop if available
                        if let Some(last_hop) = r.hops.last() {
                            if let Ok(pubkey) = PublicKey::from_str(&last_hop.pub_key) {
                                destination_pubkey = Some(pubkey);
                            }
                        }

                        Route {
                            total_time_lock: r.total_time_lock,
                            total_fees: (r.total_fees_msat / 1000).try_into().unwrap_or(0),
                            total_amt: (r.total_amt_msat / 1000).try_into().unwrap_or(0),
                            hops: r
                                .hops
                                .into_iter()
                                .map(|hop| Hop {
                                    pubkey: PublicKey::from_str(&hop.pub_key)
                                        .unwrap_or(self.info.pubkey),
                                    chan_id: ShortChannelID(hop.chan_id.try_into().unwrap_or(0)),
                                    amount_to_forward: (hop.amt_to_forward_msat / 1000) as u64,
                                    fee: Some((hop.fee_msat / 1000) as u64),
                                    expiry: Some(hop.expiry.into()),
                                })
                                .collect(),
                        }
                    });

                    PaymentHtlc {
                        routes: route.map_or_else(Vec::new, |r| vec![r]),
                        attempt_id: htlc.attempt_id,
                        attempt_time: Some(
                            UNIX_EPOCH + Duration::from_nanos(htlc.attempt_time_ns as u64),
                        ),
                        resolve_time: Some(
                            UNIX_EPOCH + Duration::from_nanos(htlc.resolve_time_ns as u64),
                        ),
                        failure_reason: htlc.failure.as_ref().map(|f| format!("{:?}", f.code())),
                        failure_code: htlc.failure.as_ref().map(|f| f.code() as u16),
                    }
                })
                .collect();

            (htlcs, destination_pubkey)
        };

        let network = self
            .get_network()
            .await
            .map(|n| Some(n.to_string()))
            .unwrap_or(None);

        Ok(PaymentDetails {
            state,
            amount: payment.value_sat.try_into().unwrap_or(0),
            routing_fee: Some(payment.fee_sat.try_into().unwrap_or(0)),
            network,
            description: None,
            creation_time,
            invoice: payment.payment_request.into(),
            payment_hash: payment.payment_hash,
            destination_pubkey,
            completed_at: None,
            htlcs,
        })
    }

    async fn list_payments(&self) -> Result<Vec<PaymentSummary>, LightningError> {
        let mut lightning_stub = self.get_lightning_stub().await;
        let response = lightning_stub
            .list_payments(ListPaymentsRequest::default())
            .await
            .map_err(|e| LightningError::RpcError(e.to_string()))?
            .into_inner();

        Ok(response
            .payments
            .into_iter()
            .map(|payment| {
                use std::convert::TryFrom;
                let state = match PaymentStatus::try_from(payment.status)
                    .unwrap_or(PaymentStatus::Unknown)
                {
                    PaymentStatus::Unknown | PaymentStatus::InFlight => PaymentState::Inflight,
                    PaymentStatus::Succeeded => PaymentState::Settled,
                    PaymentStatus::Failed => PaymentState::Failed,
                };

                PaymentSummary {
                    state,
                    amount_sat: payment.value_sat.try_into().unwrap_or(0),
                    amount_usd: 0,
                    routing_fee: payment.fee_sat.try_into().ok(),
                    creation_time: payment
                        .creation_time_ns
                        .try_into()
                        .ok()
                        .map(|ns: u64| SystemTime::UNIX_EPOCH + Duration::from_nanos(ns)),
                    invoice: Some(payment.payment_request),
                    payment_hash: payment.payment_hash,
                    completed_at: payment.htlcs.last().map(|htlc| {
                        (htlc.resolve_time_ns / 1_000_000_000)
                            .try_into()
                            .unwrap_or_default()
                    }),
                }
            })
            .collect())
    }

    async fn stream_events(
        &mut self,
    ) -> Result<Pin<Box<dyn Stream<Item = NodeSpecificEvent> + Send>>, LightningError> {
        let channel_events_stream = self.stream_channel_events().await?;
        let invoice_events_stream = self.stream_invoice_events().await?;

        let event_stream = stream! {
            let channel_events_filtered = channel_events_stream.filter_map(|result| {
                let event_opt = match result {
                    Ok(update) => {
                        match update.r#type() {
                            LndChannelUpdateType::OpenChannel => {
                                if let Some(event_channel) = update.channel {
                                    match event_channel {
                                        EventChannel::OpenChannel(chan) => {
                                            Some(NodeSpecificEvent::LND(LNDEvent::ChannelOpened {
                                                active: chan.active,
                                                remote_pubkey: chan.remote_pubkey,
                                                channel_point: chan.channel_point,
                                                chan_id: chan.chan_id,
                                                capacity: chan.capacity,
                                                local_balance: chan.local_balance,
                                                remote_balance: chan.remote_balance,
                                                total_satoshis_sent: chan.total_satoshis_sent,
                                                total_satoshis_received: chan.total_satoshis_received,
                                            }))
                                        }
                                        _ => {
                                            eprintln!("Unexpected channel variant for OpenChannel event");
                                            None
                                        }
                                    }
                                } else {
                                    None
                                }
                            },
                            LndChannelUpdateType::ClosedChannel => {
                                if let Some(event_channel) = update.channel {
                                    match event_channel {
                                        EventChannel::ClosedChannel(chan_close_sum) => {
                                            Some(NodeSpecificEvent::LND(LNDEvent::ChannelClosed {
                                                channel_point: chan_close_sum.channel_point,
                                                chan_id:  chan_close_sum.chan_id,
                                                chain_hash:  chan_close_sum.chain_hash,
                                                closing_tx_hash:  chan_close_sum.closing_tx_hash,
                                                remote_pubkey:  chan_close_sum.remote_pubkey,
                                                capacity:  chan_close_sum.capacity,
                                                close_height:  chan_close_sum.close_height,
                                                settled_balance:  chan_close_sum.settled_balance,
                                                time_locked_balance:  chan_close_sum.time_locked_balance,
                                                close_type:  chan_close_sum.close_type,
                                                open_initiator:  chan_close_sum.open_initiator,
                                                close_initiator:  chan_close_sum.close_initiator,
                                            }))
                                        }
                                        _ => {
                                            eprintln!("Unexpected channel variant for ClosedChannel event");
                                            None
                                        }
                                    }
                                } else {
                                    None
                                }
                            },
                            _ => None,
                        }
                    }
                    Err(e) => {
                        eprintln!("Error receiving LND channel event: {:?}", e);
                        None
                    }
                };
                futures::future::ready(event_opt)
            });

            let invoice_events_filtered = invoice_events_stream.filter_map(|result| {
                let event_opt = match result {
                    Ok(invoice) => {
                        match invoice.state() {
                            InvoiceState::Open => {
                                Some(NodeSpecificEvent::LND(LNDEvent::InvoiceCreated {
                                        preimage: invoice.r_preimage,
                                        hash: invoice.r_hash,
                                        value_msat: invoice.value_msat,
                                        state: invoice.state,
                                        memo: invoice.memo,
                                        creation_date: invoice.creation_date,
                                        payment_request: invoice.payment_request,
                                }))
                            },
                            InvoiceState::Settled => {
                                  Some(NodeSpecificEvent::LND(LNDEvent::InvoiceSettled {
                                        preimage: invoice.r_preimage,
                                        hash: invoice.r_hash,
                                        value_msat: invoice.value_msat,
                                        state: invoice.state,
                                        memo: invoice.memo,
                                        creation_date: invoice.creation_date,
                                        payment_request: invoice.payment_request,
                                }))
                            },
                            InvoiceState::Canceled => {
                                  Some(NodeSpecificEvent::LND(LNDEvent::InvoiceCancelled {
                                        preimage: invoice.r_preimage,
                                        hash: invoice.r_hash,
                                        value_msat: invoice.value_msat,
                                        state: invoice.state,
                                        memo: invoice.memo,
                                        creation_date: invoice.creation_date,
                                        payment_request: invoice.payment_request,
                                }))
                            },
                            InvoiceState::Accepted => {
                                  Some(NodeSpecificEvent::LND(LNDEvent::InvoiceAccepted {
                                        preimage: invoice.r_preimage,
                                        hash: invoice.r_hash,
                                        value_msat: invoice.value_msat,
                                        state: invoice.state,
                                        memo: invoice.memo,
                                        creation_date: invoice.creation_date,
                                        payment_request: invoice.payment_request,
                                }))
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("Error subscribing to LND channel events: {:?}", e);
                        None
                    }
                };
                futures::future::ready(event_opt)
            });

            let mut merged_stream = SelectAll::new();
            merged_stream.push(channel_events_filtered.boxed());
            merged_stream.push(invoice_events_filtered.boxed());

            while let Some(event) = merged_stream.next().await {
                yield event;
            }
        };

        Ok(Box::pin(event_stream))
    }

    async fn list_invoices(&self) -> Result<Vec<CustomInvoice>, LightningError> {
        let mut client = self.client.lock().await;
        let request = tonic_lnd::lnrpc::ListInvoiceRequest {
            pending_only: false,
            ..Default::default()
        };

        let response = client
            .lightning()
            .list_invoices(request)
            .await
            .map_err(|e| LightningError::RpcError(e.to_string()))?
            .into_inner();

        let invoices = response
            .invoices
            .into_iter()
            .map(|invoice| {
                // Map tonic's InvoiceState to your InvoiceStatus enum
                let state =
                    match InvoiceState::try_from(invoice.state).unwrap_or(InvoiceState::Open) {
                        InvoiceState::Open => InvoiceStatus::Open,
                        InvoiceState::Settled => InvoiceStatus::Settled,
                        InvoiceState::Canceled => InvoiceStatus::Failed,
                        InvoiceState::Accepted => InvoiceStatus::Open,
                    };
                let htlcs = Some(
                    invoice
                        .htlcs
                        .into_iter()
                        .map(|htlc| InvoiceHtlc {
                            chan_id: Some(htlc.chan_id),
                            htlc_index: Some(htlc.htlc_index),
                            amt_msat: Some(htlc.amt_msat),
                            accept_time: Some(htlc.accept_time),
                            resolve_time: Some(htlc.resolve_time),
                            expiry_height: htlc.expiry_height.try_into().ok(),
                            mpp_total_amt_msat: Some(htlc.mpp_total_amt_msat),
                        })
                        .collect(),
                );

                let features = Some(
                    invoice
                        .features
                        .into_iter()
                        .map(|(k, v)| {
                            (
                                k,
                                Feature {
                                    name: Some(v.name),
                                    is_known: Some(v.is_known),
                                    is_required: Some(v.is_required),
                                },
                            )
                        })
                        .collect(),
                );

                CustomInvoice {
                    memo: invoice.memo,
                    payment_hash: hex::encode(invoice.r_hash),
                    payment_preimage: Some(hex::encode(invoice.r_preimage))
                        .filter(|s| !s.is_empty())
                        .unwrap_or_default(),
                    value: invoice.value as u64,
                    value_msat: invoice.value_msat as u64,
                    settled: Some(invoice.settled),
                    creation_date: Some(invoice.creation_date),
                    settle_date: Some(invoice.settle_date),
                    payment_request: invoice.payment_request,
                    expiry: Some(invoice.expiry as u64),
                    state,
                    is_keysend: Some(invoice.is_keysend),
                    is_amp: Some(invoice.is_amp),
                    payment_addr: Some(hex::encode(invoice.payment_addr)).filter(|s| !s.is_empty()),
                    htlcs,
                    features,
                }
            })
            .collect();

        Ok(invoices)
    }

    async fn get_invoice_details(
        &self,
        payment_hash: &PaymentHash,
    ) -> Result<CustomInvoice, LightningError> {
        let mut client = self.get_lightning_stub().await;

        let request = tonic_lnd::lnrpc::PaymentHash {
            r_hash_str: payment_hash.to_string(),
            ..Default::default()
        };

        let response = client
            .lookup_invoice(request)
            .await
            .map_err(|e| LightningError::RpcError(e.to_string()))?
            .into_inner();

        let state = match InvoiceState::try_from(response.state).unwrap_or(InvoiceState::Open) {
            InvoiceState::Open => InvoiceStatus::Open,
            InvoiceState::Settled => InvoiceStatus::Settled,
            InvoiceState::Canceled => InvoiceStatus::Failed,
            InvoiceState::Accepted => InvoiceStatus::Open,
        };

        Ok(CustomInvoice {
            memo: response.memo,
            payment_hash: hex::encode(response.r_hash),
            payment_preimage: Some(hex::encode(response.r_preimage))
                .filter(|s| !s.is_empty())
                .unwrap_or_default(),
            value: response.value as u64,
            value_msat: response.value_msat as u64,
            settled: Some(response.settled),
            creation_date: Some(response.creation_date),
            settle_date: Some(response.settle_date),
            payment_request: response.payment_request,
            expiry: Some(response.expiry as u64),
            state,
            is_keysend: Some(response.is_keysend),
            is_amp: Some(response.is_amp),
            payment_addr: Some(hex::encode(response.payment_addr)).filter(|s| !s.is_empty()),
            htlcs: None,
            features: None,
        })
    }
}

#[async_trait]
impl LightningClient for ClnNode {
    fn get_info(&self) -> &NodeInfo {
        &self.info
    }

    async fn get_network(&self) -> Result<Network, LightningError> {
        let mut client = self.client.lock().await;
        let info = client
            .getinfo(GetinfoRequest {})
            .await
            .map_err(|err| LightningError::GetInfoError(err.to_string()))?
            .into_inner();

        Ok(Network::from_core_arg(&info.network)
            .map_err(|err| LightningError::ValidationError(err.to_string()))?)
    }

    async fn get_node_info(&self, node_id: &PublicKey) -> Result<NodeInfo, LightningError> {
        let mut client = self.client.lock().await;
        let mut nodes: Vec<cln_grpc::pb::ListnodesNodes> = client
            .list_nodes(ListnodesRequest {
                id: Some(node_id.serialize().to_vec()),
            })
            .await
            .map_err(|err| LightningError::GetNodeInfoError(err.to_string()))?
            .into_inner()
            .nodes;

        if let Some(node) = nodes.pop() {
            Ok(NodeInfo {
                pubkey: *node_id,
                alias: node.alias.unwrap_or(String::new()),
                features: node
                    .features
                    .clone()
                    .map_or(NodeFeatures::empty(), NodeFeatures::from_be_bytes),
            })
        } else {
            Err(LightningError::GetNodeInfoError(
                "Node not found".to_string(),
            ))
        }
    }

    async fn list_channels(&self) -> Result<Vec<ChannelSummary>, LightningError> {
        let mut client = self.get_client_stub().await;
        let response = client
            .list_peer_channels(ListpeerchannelsRequest { id: None })
            .await
            .map_err(|e| LightningError::RpcError(e.to_string()))?
            .into_inner();

        let _now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let summaries = response
            .channels
            .into_iter()
            .filter_map(|channel| {
                let chan_id = channel.short_channel_id.as_ref()?.parse().ok()?; // u64

                let capacity: u64 = channel.total_msat.as_ref().map(|a| a.msat).unwrap_or(0) / 1000;

                let local_balance: u64 =
                    channel.to_us_msat.as_ref().map(|a| a.msat).unwrap_or(0) / 1000;

                let remote_balance = capacity.saturating_sub(local_balance);

                let channel_state = match channel.state {
                    0 => ChannelState::Opening,  // OPENINGD
                    1 => ChannelState::Opening,  // CHANNELD_AWAITING_LOCKIN
                    2 => ChannelState::Active,   // CHANNELD_NORMAL
                    3 => ChannelState::Closing,  // CHANNELD_SHUTTING_DOWN
                    4 => ChannelState::Closing,  // CLOSINGD_SIGEXCHANGE
                    5 => ChannelState::Closing,  // CLOSINGD_COMPLETE
                    8 => ChannelState::Closed,   // ONCHAIN
                    9 => ChannelState::Opening,  // DUALOPEND_OPEN_INIT
                    10 => ChannelState::Opening, // DUALOPEND_AWAITING_LOCKIN
                    _ => ChannelState::Disabled, // default/fallback
                };
                let alias = channel.alias.as_ref().and_then(|a| a.remote.clone());

                Some(ChannelSummary {
                    chan_id,
                    alias,
                    channel_state,
                    private: false,
                    remote_balance,
                    local_balance,
                    capacity,
                    creation_date: None,
                    uptime: None,
                })
            })
            .collect();

        Ok(summaries)
    }

    async fn get_channel_info(
        &self,
        channel_id: &ShortChannelID,
    ) -> Result<ChannelDetails, LightningError> {
        let mut client = self.get_client_stub().await;

        // First get the channel information
        let channels_response = client
            .list_channels(ListchannelsRequest {
                short_channel_id: Some(channel_id.0.to_string()),
                ..Default::default()
            })
            .await
            .map_err(|e| LightningError::ChannelError(format!("CLN list_channels error: {}", e)))?
            .into_inner();

        let chan_opt = channels_response
            .channels
            .into_iter()
            .find(|c| c.short_channel_id == channel_id.0.to_string());

        if let Some(chan) = chan_opt {
            let remote_pubkey = PublicKey::from_slice(&chan.destination).map_err(|e| {
                LightningError::ChannelError(format!("Invalid remote pubkey bytes: {}", e))
            })?;

            let amount_msat = chan.amount_msat.as_ref();
            let capacity_sat = amount_msat.map(|amt| amt.msat / 1000).unwrap_or(0);
            let local_balance_sat = amount_msat.map(|amt| amt.msat / 2000).unwrap_or(0);
            let remote_balance_sat = capacity_sat.saturating_sub(local_balance_sat);

            // Get node policies
            let node1_policy = if chan.source == self.info.pubkey.serialize().to_vec() {
                Some(NodePolicy {
                    pubkey: self.info.pubkey,
                    fee_base_msat: chan.base_fee_millisatoshi as u64,
                    fee_rate_milli_msat: chan.fee_per_millionth as u64,
                    min_htlc_msat: chan.htlc_minimum_msat.as_ref().map_or(0, |amt| amt.msat),
                    max_htlc_msat: chan.htlc_maximum_msat.as_ref().map(|amt| amt.msat),
                    time_lock_delta: chan.delay as u16,
                    disabled: !chan.active,
                    last_update: None,
                })
            } else {
                None
            };

            let node2_policy = if chan.destination == self.info.pubkey.serialize().to_vec() {
                Some(NodePolicy {
                    pubkey: remote_pubkey,
                    fee_base_msat: chan.base_fee_millisatoshi as u64,
                    fee_rate_milli_msat: chan.fee_per_millionth as u64,
                    min_htlc_msat: chan.htlc_minimum_msat.as_ref().map_or(0, |amt| amt.msat),
                    max_htlc_msat: chan.htlc_maximum_msat.as_ref().map(|amt| amt.msat),
                    time_lock_delta: chan.delay as u16,
                    disabled: !chan.active,
                    last_update: None,
                })
            } else {
                None
            };

            Ok(ChannelDetails {
                channel_id: *channel_id,
                local_balance_sat,
                remote_balance_sat,
                capacity_sat,
                active: chan.active,
                private: !chan.public,
                remote_pubkey,
                commit_fee_sat: None,
                local_chan_reserve_sat: None,
                remote_chan_reserve_sat: None,
                num_updates: None,
                total_satoshis_sent: None,
                total_satoshis_received: None,
                channel_age_blocks: None,
                last_update: None,
                opening_cost_sat: None,
                initiator: None,
                txid: None,
                vout: None,
                node1_policy,
                node2_policy,
            })
        } else {
            Err(LightningError::ChannelError(
                "Channel not found in CLN".to_string(),
            ))
        }
    }

    async fn get_payment_details(
        &self,
        payment_hash: &PaymentHash,
    ) -> Result<PaymentDetails, LightningError> {
        let mut client = self.get_client_stub().await;

        let response = client
            .list_pays(cln_grpc::pb::ListpaysRequest {
                payment_hash: Some(payment_hash.0.to_vec()),
                ..Default::default()
            })
            .await
            .map_err(|e| LightningError::RpcError(format!("CLN listpays error: {}", e)))?
            .into_inner();

        let Some(payment) = response.pays.into_iter().last() else {
            return Err(LightningError::NotFound("Payment not found".to_string()));
        };

        let state = match payment.status {
            0 => PaymentState::Inflight, // pending
            1 => PaymentState::Settled,  // complete
            2 => PaymentState::Failed,   // failed
            _ => PaymentState::Failed,
        };

        // Calculate amounts
        let amount = payment.amount_msat.map(|amt| amt.msat / 1000).unwrap_or(0);
        let sent_amount = payment
            .amount_sent_msat
            .map(|amt| amt.msat / 1000)
            .unwrap_or(0);
        let routing_fee = sent_amount.checked_sub(amount);

        // Get destination pubkey
        let destination_pubkey = match &payment.destination {
            Some(hex_str) => {
                let hex_str = String::from_utf8(hex_str.clone()).map_err(|e| {
                    LightningError::Parse(format!("Invalid destination string: {}", e))
                })?;
                let pubkey = PublicKey::from_str(&hex_str).map_err(|e| {
                    LightningError::Parse(format!("Invalid destination pubkey: {}", e))
                })?;
                Some(pubkey)
            }
            None => None,
        };

        // Convert timestamps
        let creation_time = payment
            .created_at
            .try_into()
            .ok()
            .map(|ts: u64| UNIX_EPOCH + Duration::from_secs(ts));

        let network = self
            .get_network()
            .await
            .map(|n| Some(n.to_string()))
            .unwrap_or(None);

        Ok(PaymentDetails {
            state,
            amount,
            routing_fee,
            network,
            description: payment.description,
            creation_time,
            invoice: payment.bolt11,
            payment_hash: hex::encode(&payment.payment_hash),
            destination_pubkey,
            completed_at: payment.completed_at,
            htlcs: vec![],
        })
    }

    async fn list_payments(&self) -> Result<Vec<PaymentSummary>, LightningError> {
        let mut client = self.get_client_stub().await;
        let response = client
            .list_pays(cln_grpc::pb::ListpaysRequest::default())
            .await
            .map_err(|e| LightningError::RpcError(e.to_string()))?
            .into_inner();

        let summaries = response
            .pays
            .into_iter()
            .filter_map(|payment| {
                let state = match payment.status {
                    0 => PaymentState::Inflight, // pending
                    1 => PaymentState::Settled,  // complete
                    2 => PaymentState::Failed,   // failed
                    _ => PaymentState::Failed,
                };

                let amount_sat = payment
                    .amount_msat
                    .as_ref()
                    .map(|msat| (msat.msat / 1000).try_into().unwrap_or(0))
                    .unwrap_or(0);

                let routing_fee = match (
                    payment.amount_sent_msat.as_ref(),
                    payment.amount_msat.as_ref(),
                ) {
                    (Some(sent), Some(received)) => {
                        Some(((sent.msat - received.msat) / 1000).try_into().unwrap())
                    }
                    _ => None,
                };

                let creation_time = payment
                    .created_at
                    .try_into()
                    .ok()
                    .map(|ts: u64| UNIX_EPOCH + Duration::from_secs(ts));

                Some(PaymentSummary {
                    state,
                    amount_sat,
                    amount_usd: 0,
                    routing_fee,
                    creation_time,
                    invoice: payment.bolt11,
                    payment_hash: hex::encode(&payment.payment_hash),
                    completed_at: payment.completed_at.map(|ts| ts.try_into().ok()).flatten(),
                })
            })
            .collect();

        Ok(summaries)
    }

    async fn stream_events(
        &mut self,
    ) -> Result<Pin<Box<dyn Stream<Item = NodeSpecificEvent> + Send>>, LightningError> {
        let event_stream = async_stream::stream! {
            let mut counter = 0;
            loop {
                sleep(Duration::from_millis(60)).await;
                yield NodeSpecificEvent::CLN(CLNEvent::ChannelOpened {  });
                counter  = counter + 1;
            }
        };

        Ok(Box::pin(event_stream))
    }

    async fn list_invoices(&self) -> Result<Vec<CustomInvoice>, LightningError> {
        let mut client = self.get_client_stub().await;
        let response = client
            .list_invoices(cln_grpc::pb::ListinvoicesRequest::default())
            .await
            .map_err(|e| LightningError::RpcError(e.to_string()))?
            .into_inner();

        let now = chrono::Utc::now().timestamp() as u64;

        let invoices = response
            .invoices
            .into_iter()
            .map(|invoice| {
                let amount_msat = invoice.amount_msat.as_ref().map(|ms| ms.msat).unwrap_or(0);
                let amount_sats = amount_msat / 1000;

                let expires_at = invoice.expires_at;

                let state = match invoice.status {
                    1 => InvoiceStatus::Settled, // paid
                    2 => InvoiceStatus::Expired, // expired
                    3 => InvoiceStatus::Failed,  // failed
                    _ => {
                        if invoice.expires_at <= now {
                            InvoiceStatus::Expired
                        } else {
                            InvoiceStatus::Open
                        }
                    }
                };

                CustomInvoice {
                    memo: invoice.description.unwrap_or_default(),
                    payment_hash: hex::encode(invoice.payment_hash),
                    payment_preimage: invoice
                        .payment_preimage
                        .map(hex::encode)
                        .unwrap_or_default(),
                    value: amount_sats,
                    value_msat: amount_msat,
                    settled: None,
                    creation_date: None,
                    settle_date: invoice.paid_at.map(|t| t as i64),
                    payment_request: invoice.bolt11.unwrap_or_default(),
                    expiry: Some(expires_at),
                    state,
                    is_keysend: None,
                    is_amp: None,
                    payment_addr: None,
                    htlcs: None,
                    features: None,
                }
            })
            .collect();

        Ok(invoices)
    }

    async fn get_invoice_details(
        &self,
        payment_hash: &PaymentHash,
    ) -> Result<CustomInvoice, LightningError> {
        let mut client = self.get_client_stub().await;

        let request = cln_grpc::pb::ListinvoicesRequest {
            payment_hash: Some(payment_hash.0.to_vec()),
            ..Default::default()
        };

        let response = client
            .list_invoices(request)
            .await
            .map_err(|e| LightningError::RpcError(format!("CLN listinvoices error: {}", e)))?
            .into_inner();

        let invoice = response
            .invoices
            .into_iter()
            .next()
            .ok_or_else(|| LightningError::NotFound("Invoice not found".into()))?;

        let state = match invoice.status {
            1 => InvoiceStatus::Settled, // paid
            2 => InvoiceStatus::Expired, // expired
            3 => InvoiceStatus::Failed,  // failed
            _ => {
                let now = chrono::Utc::now().timestamp() as u64;

                if invoice.expires_at <= now {
                    InvoiceStatus::Expired
                } else {
                    InvoiceStatus::Open
                }
            }
        };

        let amount_msat = invoice.amount_msat.as_ref().map(|ms| ms.msat).unwrap_or(0);
        let amount_sats = amount_msat / 1000;

        Ok(CustomInvoice {
            memo: invoice.description.unwrap_or_default(),
            payment_hash: hex::encode(invoice.payment_hash),
            payment_preimage: invoice
                .payment_preimage
                .map(hex::encode)
                .unwrap_or_default(),
            value: amount_sats,
            value_msat: amount_msat,
            settled: None,
            creation_date: None,
            settle_date: invoice.paid_at.map(|t| t as i64),
            payment_request: invoice.bolt11.unwrap_or_default(),
            expiry: Some(invoice.expires_at),
            state,
            is_keysend: None,
            is_amp: None,
            payment_addr: None,
            htlcs: None,
            features: None,
        })
    }
}
pub fn parse_channel_point(s: &str) -> Result<OutPoint, LightningError> {
    let mut parts = s.split(':');
    let txid_str = parts
        .next()
        .ok_or_else(|| LightningError::ValidationError("Missing txid".into()))?;
    let vout_str = parts
        .next()
        .ok_or_else(|| LightningError::ValidationError("Missing vout".into()))?;

    let txid = Txid::from_str(txid_str)
        .map_err(|e| LightningError::ValidationError(format!("Invalid txid: {e}")))?;
    let vout = vout_str
        .parse::<u32>()
        .map_err(|e| LightningError::ValidationError(format!("Invalid vout: {e}")))?;

    Ok(OutPoint { txid, vout })
}
