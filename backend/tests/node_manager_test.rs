use backend::services::node_manager::{parse_channel_point, parse_node_features};
use std::collections::HashSet;
use std::str::FromStr;
use bitcoin::Txid;

#[test]
fn test_parse_node_features_empty() {
    let features = HashSet::new();
    let parsed = parse_node_features(features);
    assert!(!parsed.supports_basic_mpp());
    assert!(!parsed.requires_payment_secret());
}

#[test]
fn test_parse_channel_point_valid() {
    let txid = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2";
    let channel_point = format!("{}:1", txid);
    let result = parse_channel_point(&channel_point).unwrap();

    assert_eq!(result.vout, 1);
    assert_eq!(result.txid, Txid::from_str(txid).unwrap());
}

#[test]
fn test_parse_channel_point_invalid_formats() {
    assert!(parse_channel_point(":1").is_err());
    assert!(parse_channel_point("txid:").is_err());
    assert!(parse_channel_point("txid").is_err());
    assert!(parse_channel_point("invalid_txid:1").is_err());
}