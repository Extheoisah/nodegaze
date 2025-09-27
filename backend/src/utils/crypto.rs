//! String encryption/decryption using AES-256-GCM with direct key support.
//!
//! ## Usage
//!
//! ```rust
//! let encrypted = StringCrypto::encrypt("secret data")?;
//! let decrypted = StringCrypto::decrypt(&encrypted)?;
//! ```

use crate::config::Config;
use aes_gcm::aead::rand_core::{OsRng, RngCore};
use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit},
};
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug)]
pub enum CryptoError {
    InvalidKey,
    EncryptionFailed,
    DecryptionFailed,
    InvalidData,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::InvalidKey => write!(f, "Invalid encryption key"),
            CryptoError::EncryptionFailed => write!(f, "Encryption failed"),
            CryptoError::DecryptionFailed => write!(f, "Decryption failed"),
            CryptoError::InvalidData => write!(f, "Invalid data format"),
        }
    }
}

impl std::error::Error for CryptoError {}

/// AES-256-GCM encryption/decryption for strings.
pub struct StringCrypto;

impl StringCrypto {
    /// Create a cipher instance from the environment key
    fn create_cipher() -> Result<Aes256Gcm, CryptoError> {
        let config = Config::from_env().unwrap();
        let key_str = config.encryption_key;

        // Decode base64 key or use raw bytes
        let key_bytes = if key_str.len() == 44 {
            // Assume base64 encoded key
            general_purpose::STANDARD
                .decode(key_str)
                .map_err(|_| CryptoError::InvalidKey)?
        } else {
            // Use raw string bytes (pad or truncate to 32 bytes)
            let mut bytes = vec![0u8; 32];
            let input_bytes = key_str.as_bytes();
            let copy_len = std::cmp::min(input_bytes.len(), 32);
            bytes[..copy_len].copy_from_slice(&input_bytes[..copy_len]);
            bytes
        };

        if key_bytes.len() != 32 {
            return Err(CryptoError::InvalidKey);
        }

        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        Ok(cipher)
    }

    /// Encrypt a string and return base64 encoded result.
    /// Each encryption uses a unique nonce for security.
    pub fn encrypt(plaintext: &str) -> Result<String, CryptoError> {
        let cipher = Self::create_cipher()?;

        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the plaintext
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|_| CryptoError::EncryptionFailed)?;

        // Combine nonce + ciphertext
        let mut result = Vec::new();
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        // Return base64 encoded result
        Ok(general_purpose::STANDARD.encode(result))
    }

    /// Decrypt a base64 encoded string that was encrypted with `encrypt()`.
    pub fn decrypt(encrypted_data: &str) -> Result<String, CryptoError> {
        let cipher = Self::create_cipher()?;

        // Decode base64
        let data = general_purpose::STANDARD
            .decode(encrypted_data)
            .map_err(|_| CryptoError::InvalidData)?;

        if data.len() < 12 {
            return Err(CryptoError::InvalidData);
        }

        // Extract nonce and ciphertext
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| CryptoError::DecryptionFailed)?;

        // Convert back to string
        String::from_utf8(plaintext).map_err(|_| CryptoError::InvalidData)
    }
}

/// Generate a new base64-encoded 256-bit encryption key.
pub fn generate_key() -> String {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    general_purpose::STANDARD.encode(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_encrypt_decrypt() {
            let original = "Test message";

            let encrypted = StringCrypto::encrypt(original).unwrap();
            let decrypted = StringCrypto::decrypt(&encrypted).unwrap();

            assert_eq!(original, decrypted);
        }

        #[test]
        fn test_unique_nonces() {
            let msg = "Same message";
            let enc1 = StringCrypto::encrypt(msg).unwrap();
            let enc2 = StringCrypto::encrypt(msg).unwrap();

            // Same message should produce different ciphertext
            assert_ne!(enc1, enc2);
            // Same message should produce different ciphertext
            assert_ne!(enc1, enc2);

            // But both should decrypt correctly
            assert_eq!(StringCrypto::decrypt(&enc1).unwrap(), msg);
            assert_eq!(StringCrypto::decrypt(&enc2).unwrap(), msg);
        }
    }
}
