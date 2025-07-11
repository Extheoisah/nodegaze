//! Collection of general utility functions and common traits.
//!
//! This module serves as a repository for small, reusable helper functions
//! or traits that do not fit into other specific domain modules.

use serde::{Serialize, Deserialize};
use bitcoin::secp256k1::PublicKey;
use std::fmt::{Display, Formatter};
use crate::errors::LightningError;
use expanduser::expanduser;
use lightning::ln::features::NodeFeatures;

pub mod crypto;

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
            },
            NodeId::Alias(a) => {
                if a != alias {
                    return Err(LightningError::ValidationError(format!(
                        "The provided alias does not match the one returned by the backend ({} != {}).",
                        a, alias
                    )));
                }
            },
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