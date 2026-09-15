use backend::utils::handlers_common::{parse_payment_hash, parse_public_key};
use bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};

fn valid_public_key() -> String {
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(&[0xcd; 32]).expect("valid secret key");
    PublicKey::from_secret_key(&secp, &secret_key).to_string()
}

const VALID_HASH: &str = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2";

#[test]
fn parse_payment_hash_validates_length_and_format() {
    let (_, body) = parse_payment_hash("a1b2").unwrap_err();
    assert!(body.contains("invalid_payment_hash_length"));

    let (_, body) = parse_payment_hash("xyz!").unwrap_err();
    assert!(body.contains("invalid_payment_hash"));

    assert!(parse_payment_hash(VALID_HASH).is_ok());
}

#[test]
fn parse_public_key_rejects_invalid_and_accepts_valid() {
    let (_, body) = parse_public_key("not-a-key").unwrap_err();
    assert!(body.contains("invalid_public_key"));

    assert!(parse_public_key(&valid_public_key()).is_ok());
}