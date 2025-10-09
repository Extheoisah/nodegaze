use bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};
use std::str::FromStr;

use backend::utils::*;

fn create_test_pubkey() -> PublicKey {
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(&[0xcd; 32]).unwrap();
    PublicKey::from_secret_key(&secp, &secret_key)
}

fn create_alt_test_pubkey() -> PublicKey {
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(&[0xab; 32]).unwrap();
    PublicKey::from_secret_key(&secp, &secret_key)
}

#[test]
fn test_node_id_validation() {
    let pk = create_test_pubkey();
    let pk2 = create_alt_test_pubkey();
    let mut alias = String::from("test_alias");

    // Valid pubkey validation
    let node_id = NodeId::PublicKey(pk);
    assert!(node_id.validate(&pk, &mut alias).is_ok());

    // Mismatched pubkey should fail
    assert!(node_id.validate(&pk2, &mut alias).is_err());

    // Valid alias validation
    let node_id = NodeId::Alias("test_alias".to_string());
    assert!(node_id.validate(&pk, &mut alias).is_ok());

    // Mismatched alias should fail
    alias = String::from("wrong_alias");
    assert!(node_id.validate(&pk, &mut alias).is_err());
}

#[test]
fn test_payment_state_parsing() {
    // Case-insensitive parsing
    assert_eq!(
        PaymentState::from_str("inflight").unwrap(),
        PaymentState::Inflight
    );
    assert_eq!(
        PaymentState::from_str("FAILED").unwrap(),
        PaymentState::Failed
    );
    assert_eq!(
        PaymentState::from_str("Settled").unwrap(),
        PaymentState::Settled
    );

    // Invalid input should fail
    assert!(PaymentState::from_str("invalid").is_err());
}

#[test]
fn test_payment_type_parsing() {
    // Case-insensitive parsing
    assert_eq!(
        PaymentType::from_str("OUTGOING").unwrap().as_str(),
        "outgoing"
    );
    assert_eq!(
        PaymentType::from_str("incoming").unwrap().as_str(),
        "incoming"
    );
    assert_eq!(
        PaymentType::from_str("Forwarded").unwrap().as_str(),
        "forwarded"
    );

    // Invalid input should fail
    assert!(PaymentType::from_str("invalid").is_err());
}

#[test]
fn test_channel_state_parsing() {
    // Case-insensitive parsing
    assert_eq!(
        ChannelState::from_str("ACTIVE").unwrap(),
        ChannelState::Active
    );
    assert_eq!(
        ChannelState::from_str("disabled").unwrap(),
        ChannelState::Disabled
    );

    // Invalid input should fail
    assert!(ChannelState::from_str("invalid").is_err());
}
