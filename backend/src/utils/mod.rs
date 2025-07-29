//! Collection of general utility functions and common traits.
//!
//! This module serves as a repository for small, reusable helper functions
//! or traits that do not fit into other specific domain modules.

use crate::errors::LightningError;
use bitcoin::secp256k1::PublicKey;
use bitcoin:: Txid;
use expanduser::expanduser;
use lightning::ln::features::NodeFeatures;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::time::SystemTime;

pub mod crypto;
pub mod generate_random_string;
pub mod handlers_common;
pub mod jwt;
pub mod sats_to_usd;

/// Represents a node id, either by its public key or alias.
#[derive(Serialize, Debug, Clone)]
pub enum NodeId {
    /// The node's public key.
    PublicKey(PublicKey),
    /// The node's alias (human-readable name).
    Alias(String),
}

impl NodeId {
    /// Validates that the provided node id matches the one returned by the backend.
    pub fn validate(&self, node_id: &PublicKey, alias: &mut String) -> Result<(), LightningError> {
        match self {
            NodeId::PublicKey(pk) => {
                if pk != node_id {
                    return Err(LightningError::ValidationError(format!(
                        "The provided node id does not match the one returned by the backend ({} != {}).",
                        pk, node_id
                    )));
                }
            }
            NodeId::Alias(a) => {
                if a != alias {
                    return Err(LightningError::ValidationError(format!(
                        "The provided alias does not match the one returned by the backend ({} != {}).",
                        a, alias
                    )));
                }
            }
        }
        Ok(())
    }

    /// Returns the public key of the node if it is a public key node id.
    pub fn get_pk(&self) -> Result<&PublicKey, String> {
        if let NodeId::PublicKey(pk) = self {
            Ok(pk)
        } else {
            Err("NodeId is not a PublicKey".to_string())
        }
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NodeId::PublicKey(pk) => pk.to_string(),
                NodeId::Alias(a) => a.to_owned(),
            }
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// The node's public key.
    pub pubkey: PublicKey,
    /// A human-readable name for the node (may be empty).
    pub alias: String,
    /// The node's supported protocol features and capabilities.
    #[serde(with = "node_features_serde")]
    pub features: NodeFeatures,
}

impl Display for NodeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let pk = self.pubkey.to_string();
        let pk_summary = format!("{}...{}", &pk[..6], &pk[pk.len() - 6..]);
        if self.alias.is_empty() {
            write!(f, "{}", pk_summary)
        } else {
            write!(f, "{}({})", self.alias, pk_summary)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelDetails {
    pub channel_id: ShortChannelID,
    pub local_balance_sat: u64,
    pub remote_balance_sat: u64,
    pub capacity_sat: u64,
    pub active: bool,
    pub private: bool,
    pub remote_pubkey: PublicKey,
    pub commit_fee_sat: Option<u64>,
    pub local_chan_reserve_sat: Option<u64>,
    pub remote_chan_reserve_sat: Option<u64>,
    pub num_updates: Option<u64>,
    pub total_satoshis_sent: Option<u64>,
    pub total_satoshis_received: Option<u64>,
    pub channel_age_blocks: Option<u32>,
    pub last_update: Option<SystemTime>,
    pub opening_cost_sat: Option<u64>,
    pub initiator: Option<bool>,
    pub txid: Option<Txid>,
    pub vout: Option<u32>,
    pub node1_policy: Option<NodePolicy>,
    pub node2_policy: Option<NodePolicy>,
}

#[derive(Debug, Serialize)]
pub struct ChannelSummary {
    pub chan_id: ShortChannelID,
    pub alias: Option<String>,
    pub channel_state: ChannelState,
    pub private: bool,
    pub remote_balance: u64,
    pub local_balance: u64,
    pub capacity: u64,
    pub creation_date: Option<i64>,
    pub uptime: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomInvoice {
    pub memo: String,
    pub payment_hash: String,
    pub payment_preimage: String,
    pub value: u64,
    pub value_msat: u64,
    pub settled: Option<bool>,
    pub creation_date: Option<i64>,
    pub settle_date: Option<i64>,
    pub payment_request: String,
    pub expiry: Option<u64>,
    pub state: InvoiceStatus,
    pub is_keysend: Option<bool>,
    pub is_amp: Option<bool>,
    pub payment_addr: Option<String>,
    pub htlcs: Option<Vec<InvoiceHtlc>>,
    pub features: Option<HashMap<u32, Feature>>,
}

/// Represents a node's routing policy for forwarding payments
#[derive(Debug, Serialize, Deserialize)]
pub struct NodePolicy {
    pub pubkey: PublicKey,
    pub fee_base_msat: u64,
    pub fee_rate_milli_msat: u64,
    pub min_htlc_msat: u64,
    pub max_htlc_msat: Option<u64>,
    pub time_lock_delta: u16,
    pub disabled: bool,
    pub last_update: Option<SystemTime>,
}

impl Display for NodePolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Policy(pubkey: {}, fee: {}+{}ppm, min_htlc: {}msat{})",
            self.pubkey,
            self.fee_base_msat,
            self.fee_rate_milli_msat,
            self.min_htlc_msat,
            match self.max_htlc_msat {
                Some(max) => format!(", max_htlc: {}msat", max),
                None => String::new(),
            }
        )
    }
}

/// Represents a short channel ID.
#[derive(Debug, Clone, Serialize, Copy, Deserialize)]
pub struct ShortChannelID(pub u64);

/// Represents a log entry from the Lightning Network node.
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeLog {
    pub timestamp: String,
    pub level: Option<LogLevel>,
    pub message: String,
    pub subsystem: Option<String>,
}

// Aggregated metrics and statistics about a Lightning Network node.
///
/// Provides a comprehensive view of node performance, resource usage,
/// and operational health for monitoring and alerting purposes.
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub num_channels: u32,
    pub num_active_channels: u32,
    pub num_peers: u32,
    pub block_height: u32,
    pub uptime_seconds: u64,
    pub total_capacity: u64,
    pub total_local_balance: u64,
    pub total_remote_balance: u64,
    pub memory_usage: Option<u64>,
    pub cpu_usage: Option<u64>,
    pub disk_usage: Option<u64>,
}

/// Represents a Lightning Network payment initiated or received by the node.
#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentDetails {
    pub state: PaymentState,
    pub amount_sat: u64,
    pub amount_usd: f64,
    pub routing_fee: Option<u64>,
    pub network: Option<String>,
    pub description: Option<String>,
    pub creation_time: Option<SystemTime>,
    pub invoice: Option<String>,
    pub payment_hash: String,
    pub destination_pubkey: Option<PublicKey>,
    pub completed_at: Option<u64>,
    pub htlcs: Vec<PaymentHtlc>,
}

/// Represents a Lightning Network payment initiated or received by the node.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentSummary {
    pub state: PaymentState,
    pub amount_sat: u64,
    pub amount_usd: f64,
    pub routing_fee: Option<u64>,
    pub creation_time: Option<SystemTime>,
    pub invoice: Option<String>,
    pub payment_hash: String,
    pub completed_at: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentHtlc {
    pub routes: Vec<Route>,
    pub attempt_id: u64,
    pub attempt_time: Option<SystemTime>,
    pub resolve_time: Option<SystemTime>,
    pub failure_reason: Option<String>,
    pub failure_code: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceHtlc {
    pub chan_id: Option<u64>,
    pub htlc_index: Option<u64>,
    pub amt_msat: Option<u64>,
    pub accept_time: Option<i64>,
    pub resolve_time: Option<i64>,
    pub expiry_height: Option<u32>,
    pub mpp_total_amt_msat: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Feature {
    pub name: Option<String>,
    pub is_known: Option<bool>,
    pub is_required: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Route {
    pub total_time_lock: u32,
    pub total_fees: u64,
    pub total_amt: u64,
    pub hops: Vec<Hop>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Hop {
    pub pubkey: PublicKey,
    pub chan_id: ShortChannelID,
    pub amount_to_forward: u64,
    pub fee: Option<u64>,
    pub expiry: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum PaymentState {
    Inflight,
    Failed,
    #[default]
    Settled,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PaymentType {
    Outgoing,
    Incoming,
    Forwarded,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum InvoiceStatus {
    #[default]
    Settled,
    Open,
    Expired,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum ChannelState {
    Opening,  // funding tx not confirmed
    #[default]
    Active,   // normal / available
    Disabled, // temporarily disabled
    Closing,  // cooperative or force close initiated
    Closed,   // channel is closed
    Failed,   // failed or on-chain resolved
}

/// The severity level of a log entry.
#[derive(Debug, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Unknown,
}

pub mod serde_node_id {
    use super::*;
    use std::str::FromStr;

    use NodeId;
    use bitcoin::secp256k1::PublicKey;

    pub fn serialize<S>(id: &NodeId, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&match id {
            NodeId::PublicKey(p) => p.to_string(),
            NodeId::Alias(s) => s.to_string(),
        })
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NodeId, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if let Ok(pk) = PublicKey::from_str(&s) {
            Ok(NodeId::PublicKey(pk))
        } else {
            Ok(NodeId::Alias(s))
        }
    }
}

pub mod serde_address {
    use super::*;

    pub fn serialize<S>(address: &str, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(address)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.starts_with("https://") || s.starts_with("http://") {
            Ok(s)
        } else {
            Ok(format!("https://{}", s))
        }
    }
}

pub fn deserialize_path<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(expanduser(s)
        .map_err(serde::de::Error::custom)?
        .display()
        .to_string())
}

mod node_features_serde {
    use super::*;
    pub fn serialize<S>(features: &NodeFeatures, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&features.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NodeFeatures, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let flags = Vec::deserialize(deserializer)?;
        Ok(NodeFeatures::from_le_bytes(flags))
    }
}

impl ShortChannelID {
    pub fn to_u64(&self) -> u64 {
        self.0
    }
}

impl FromStr for ShortChannelID {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse::<u64>()?;
        Ok(Self(id))
    }
}

impl Display for ShortChannelID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for ShortChannelID {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<ShortChannelID> for u64 {
    fn from(id: ShortChannelID) -> u64 {
        id.0
    }
}

impl FromStr for PaymentState {
    type Err = String;
    
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "inflight" => Ok(PaymentState::Inflight),
            "failed" => Ok(PaymentState::Failed),
            "settled" => Ok(PaymentState::Settled),
            _ => Err(format!("Invalid payment state: {}", input)),
        }
    }
}

impl Display for PaymentState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let state = match self {
            PaymentState::Inflight => "inflight",
            PaymentState::Failed => "failed",
            PaymentState::Settled => "settled",
        };
        write!(f, "{}", state)
    }
}

impl PaymentState {
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentState::Inflight => "inflight",
            PaymentState::Failed => "failed",
            PaymentState::Settled => "settled",
        }
    }
}

impl Display for InvoiceStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let status = match self {
            InvoiceStatus::Settled => "settled",
            InvoiceStatus::Open => "open",
            InvoiceStatus::Expired => "expired",
            InvoiceStatus::Failed => "failed",
        };
        write!(f, "{}", status)
    }
}

impl FromStr for InvoiceStatus {
    type Err = String;
    
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "settled" => Ok(InvoiceStatus::Settled),
            "open" => Ok(InvoiceStatus::Open),
            "expired" => Ok(InvoiceStatus::Expired),
            "failed" => Ok(InvoiceStatus::Failed),
            _ => Err(format!("Invalid invoice status: {}", input)),
        }
    }
}

impl Display for ChannelState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let state = match self {
            ChannelState::Opening => "opening",
            ChannelState::Active => "active",
            ChannelState::Disabled => "disabled",
            ChannelState::Closing => "closing",
            ChannelState::Closed => "closed",
            ChannelState::Failed => "failed",
        };
        write!(f, "{}", state)
    }
}

impl FromStr for ChannelState {
    type Err = String;
    
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "opening" => Ok(ChannelState::Opening),
            "active" => Ok(ChannelState::Active),
            "disabled" => Ok(ChannelState::Disabled),
            "closing" => Ok(ChannelState::Closing),
            "closed" => Ok(ChannelState::Closed),
            "failed" => Ok(ChannelState::Failed),
            _ => Err(format!("Invalid channel state: {}", input)),
        }
    }
}
