//! Manages connections and interactions with Lightning Network nodes (LND and CLN).
//!
//! This module defines connection structures (`LndConnection`, `ClnConnection`),
//! manages authenticated node instances (`LndNode`, `ClnNode`), handles their lifecycle,
//! and provides methods for interacting with the Lightning node RPCs.

use async_trait::async_trait;
use tokio::time::Duration;
use ::futures::stream;
use std::collections::HashSet;
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use bitcoin::secp256k1::PublicKey;
use bitcoin::Network;
use crate::utils::{self, NodeInfo, NodeId};
use tokio::{sync::Mutex, time::sleep};
use crate::errors::LightningError;
use lightning::ln::features::NodeFeatures;
use tonic_lnd::{Client, lnrpc::{
    GetInfoRequest, 
    NodeInfoRequest, 
    ListChannelsRequest, 
    ChannelEventSubscription,
    InvoiceSubscription, 
    channel_event_update::UpdateType as LndChannelUpdateType,
    channel_event_update::Channel as EventChannel,
    invoice::InvoiceState,
}};
use cln_grpc::pb::{node_client::NodeClient, GetinfoRequest, ListchannelsRequest, ListnodesRequest};
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, Error};
use tokio_stream::{Stream, StreamExt};
use std::pin::Pin;
use crate::services::event_manager::{NodeSpecificEvent, LNDEvent, CLNEvent};

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
                    LightningError::ConnectionError(format!(
                        "Cannot establish tls connection: {}",
                        err
                    ))
                })?
                .connect()
                .await
                .map_err(|err| {
                    LightningError::ConnectionError(format!(
                        "Cannot connect to gRPC server: {}",
                        err
                ))
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
    async fn list_channels(&self) -> Result<Vec<u64>, LightningError>;
    /// Returns a stream of raw events from the lightning node.
    async fn stream_events(
        &self,
    ) -> Pin<Box<dyn Stream<Item = NodeSpecificEvent> + Send>>;
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

    async fn list_channels(&self) -> Result<Vec<u64>, LightningError> {
        let mut client = self.client.lock().await;
        let channels = client
            .lightning()
            .list_channels(ListChannelsRequest {
                ..Default::default()
            })
            .await
            .map_err(|err| LightningError::ListChannelsError(err.to_string()))?
            .into_inner();

        // Convert capacity from satoshis to millisatoshis
        Ok(channels
            .channels
            .iter()
            .map(|channel| 1000 * channel.capacity as u64)
            .collect())
    }

   async fn stream_events(&self) -> Pin<Box<dyn Stream<Item = NodeSpecificEvent> + Send>> {
        let mut channel_event_stream = match self.client
            .lock()
            .await
            .lightning()
            .subscribe_channel_events(ChannelEventSubscription {})
            .await
        {
            Ok(response) => response.into_inner(),
            Err(e) => {
                eprintln!("Error subscribing to LND channel events: {:?}", e);
                return Box::pin(stream::empty());
            }
        };

        let mut invoice_event_stream = match self.client
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
                eprintln!("Error subscribing to LND channel events: {:?}", e);
                return Box::pin(stream::empty());
            }
        };

        let event_stream = async_stream::stream! {
            while let Some(result) = channel_event_stream.next().await {
                match result {
                    Ok(update) => {
                        let event_opt = match update.r#type() {
                            LndChannelUpdateType::OpenChannel => {
                                if let Some(event_channel) = update.channel {
                                    match event_channel {
                                        EventChannel::OpenChannel(chan) => {
                                            Some(NodeSpecificEvent::LND(LNDEvent::ChannelOpened {
                                                channel_id: chan.chan_id,
                                                counterparty_node_id: chan.remote_pubkey,
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
                                                channel_id: chan_close_sum.chan_id,
                                                counterparty_node_id: chan_close_sum.remote_pubkey,
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
                        };

                        if let Some(event) = event_opt {
                            yield event;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error receiving LND event: {:?}", e);
                        break;
                    }
                }
            }

            while let Some(result) = invoice_event_stream.next().await {
                match result {
                    Ok(invoice) => {
                        let event_opt = match invoice.state() {
                            InvoiceState::Open => {
                                Some(NodeSpecificEvent::LND(LNDEvent::InvoiceCreated {
                                        preimage: invoice.r_preimage,
                                        hash: invoice.r_hash,
                                        value_msat: invoice.value_msat,
                                        state: invoice.state,
                                        memo: invoice.memo,
                                        creation_date: invoice.creation_date,
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
                                }))
                            }
                        };

                        if let Some(event) = event_opt {
                            yield event;
                        }
                    },
                    Err(e) => {
                        eprintln!("Error receiving LND event: {:?}", e);
                        break;
                    }
                };
            }
        };

        Box::pin(event_stream)
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

    async fn list_channels(&self) -> Result<Vec<u64>, LightningError> {
        let mut node_channels = self.node_channels(true).await?;
        node_channels.extend(self.node_channels(false).await?);
        Ok(node_channels)
    }

    async fn stream_events(&self) -> Pin<Box<dyn Stream<Item = NodeSpecificEvent> + Send>> {
        let event_stream = async_stream::stream! {
            let mut counter = 0;
            loop {
                sleep(Duration::from_millis(60)).await;
                yield NodeSpecificEvent::CLN(CLNEvent::ChannelOpened {  });
                counter  = counter + 1;
            }
        };

        Box::pin(event_stream)
    }
}
