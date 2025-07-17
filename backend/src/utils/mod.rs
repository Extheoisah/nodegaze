//! Collection of general utility functions and common traits.
//!
//! This module serves as a repository for small, reusable helper functions
//! or traits that do not fit into other specific domain modules.

use crate::errors::LightningError;
use bitcoin::secp256k1::PublicKey;
use bitcoin::{Address, OutPoint, ScriptBuf};
use expanduser::expanduser;
use lightning::ln::features::NodeFeatures;
use lightning::ln::{PaymentHash, PaymentPreimage, PaymentSecret};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub mod crypto;
pub mod jwt;

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

/// Represents a connected Lightning Network channel.
#[derive(Debug)]
pub struct Channel {
    pub channel_id: ShortChannelID,
    pub pubkeys: ChannelPublicKey,
    pub capacity_sat: u64,
    pub local_balance_sat: u64,
    pub remote_balance_sat: u64,
    pub status: ChannelStatus,
    pub active: bool,
    pub private: bool,
}

/// Represents a short channel ID.
#[derive(Debug, Serialize, Deserialize)]
pub struct ShortChannelID(u64);

/// Represents key and metadata for a Lightning Network channel.
#[derive(Debug)]
pub struct ChannelPublicKey {
    pub payment_point: PublicKey,
    pub initiator: bool,
    pub channel_point: OutPoint,
}

/// Represents a Hashed Timelock Contract (HTLC) for an invoice.
#[derive(Debug, Serialize, Deserialize)]
pub struct Htlc {
    amount_msat: u64,
    cltv_expiry: u32,
}

/// Represents a Lightning Network invoice for receiving payments.
#[derive(Debug)]
pub struct Invoice {
    pub payment_hash: PaymentHash,
    pub payment_preimage: Option<PaymentPreimage>,
    pub payment_secret: Option<PaymentSecret>,
    pub payment_request: String,
    pub memo: Option<String>,
    pub value_sat: u64,
    pub amt_paid_sat: Option<u64>,
    pub description_hash: Option<String>,
    pub status: InvoiceStatus,
    pub creation_date: u64,
    pub settle_date: Option<u64>,
    pub expiry: u64,
    pub htlcs: Vec<Htlc>,
}

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
#[derive(Debug)]
pub struct Payment {
    pub payment_hash: PaymentHash,
    pub value_sat: u64,
    pub fee_sat: u64,
    pub status: PaymentStatus,
    pub creation_date: u64,
    pub path: Vec<PublicKey>,
    pub payment_request: Option<String>,
}

/// Represents a connected peer in the Lightning Network.
#[derive(Debug)]
pub struct Peer {
    pub pubkey: PublicKey,
    pub address: Address,
    pub inbound: bool,
    pub bytes_recv: u64,
    pub bytes_sent: u64,
    pub ping_time: u64,
}

/// Represents an unspent transaction output (UTXO) in the node's wallet.
#[derive(Debug)]
pub struct UnspentOutput {
    pub outpoint: OutPoint,
    pub amount_sat: u64,
    pub address: Address,
    pub confirmations: u64,
    pub script_pubkey: ScriptBuf,
    pub witness_script: Option<ScriptBuf>,
}

/// Represents the on-chain Bitcoin wallet balance of the Lightning node.
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletBalance {
    pub total_sat: u64,
    pub confirmed_sat: u64,
    pub unconfirmed_sat: u64,
    pub reserved_sat: Option<u64>,
    pub locked_sat: Option<u64>,
}

/// The current status of a Lightning channel.
#[derive(Debug, Serialize, Deserialize)]
pub enum ChannelStatus {
    Opening,
    Open,
    Closing,
    Closed,
    ForceClosing,
    WaitingClose,
    Pending,
    Inactive,
    Unknown,
}

/// The status of a Lightning invoice.
#[derive(Debug, Serialize, Deserialize)]
pub enum InvoiceStatus {
    Open,
    Settled,
    Cancelled,
    Expired,
    Accepted,
}

/// The severity level of a log entry.
#[derive(Debug, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Unknown,
}

/// Status of a Lightning payment attempt.
#[derive(Debug, Serialize, Deserialize)]
pub enum PaymentStatus {
    Inflight,
    Succeeded,
    Failed,
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
