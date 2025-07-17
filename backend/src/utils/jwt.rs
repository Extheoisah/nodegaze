//! JWT token utilities for authentication and authorization.
//!
//! Provides secure token creation, validation, and claims management for
//! user authentication and node access control.

use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::errors::ServiceError;

/// JWT Claims structure containing user and node authentication data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// User ID
    pub sub: String,
    /// Account ID
    pub account_id: String,
    /// User role
    pub role: String,
    /// Node credentials (optional, now unencrypted)
    pub node_credentials: Option<NodeCredentials>,
    /// Token expiration timestamp
    pub exp: usize,
    /// Token issued at timestamp
    pub iat: usize,
}

/// Node credentials stored in JWT (now unencrypted for simplicity)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeCredentials {
    pub node_id: String,
    pub node_alias: String,
    pub node_type: String, // "lnd" or "cln"
    pub macaroon: String,
    pub tls_cert: String,
    pub client_cert: Option<String>, // For CLN
    pub client_key: Option<String>,  // For CLN
    pub ca_cert: Option<String>,     // For CLN
    pub address: String,
}

/// JWT token utility for creating and validating tokens
pub struct JwtUtils {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtUtils {
    /// Create a new JwtUtils instance with keys from environment
    pub fn new() -> Result<Self, ServiceError> {
        let config = crate::config::Config::from_env()
            .map_err(|e| ServiceError::validation(format!("Config error: {}", e)))?;

        let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());

        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        Ok(JwtUtils {
            encoding_key,
            decoding_key,
            validation,
        })
    }

    /// Generate a new JWT token with user and optional node credentials
    pub fn generate_token(
        &self,
        user_id: String,
        account_id: String,
        role: String,
        node_credentials: Option<NodeCredentials>,
    ) -> Result<String, ServiceError> {
        // Get expires_in from config
        let config = Config::from_env()
            .map_err(|e| ServiceError::validation(format!("Config error: {}", e)))?;
        let expires_in = config.jwt_expires_in_seconds;

        let now = Utc::now();
        let exp = now + Duration::seconds(expires_in as i64);

        let claims = Claims {
            sub: user_id,
            account_id,
            role,
            node_credentials,
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| ServiceError::validation(format!("Token generation failed: {}", e)))
    }

    /// Validate and decode a JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims, ServiceError> {
        decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map(|token_data| token_data.claims)
            .map_err(|e| ServiceError::validation(format!("Token validation failed: {}", e)))
    }

    /// Generate a refresh token (longer expiration)
    pub fn generate_refresh_token(&self, user_id: String) -> Result<String, ServiceError> {
        let now = Utc::now();
        let exp = now + Duration::days(30); // Refresh token expires in 30 days

        let claims = Claims {
            sub: user_id,
            account_id: String::new(), // Refresh tokens don't need account info
            role: String::new(),
            node_credentials: None,
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        encode(&Header::default(), &claims, &self.encoding_key).map_err(|e| {
            ServiceError::validation(format!("Refresh token generation failed: {}", e))
        })
    }
}

/// Extract user ID from JWT claims
impl Claims {
    pub fn user_id(&self) -> &str {
        &self.sub
    }

    pub fn account_id(&self) -> &str {
        &self.account_id
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub fn has_node_credentials(&self) -> bool {
        self.node_credentials.is_some()
    }

    pub fn node_credentials(&self) -> Option<&NodeCredentials> {
        self.node_credentials.as_ref()
    }

    /// Check if token has expired
    pub fn is_expired(&self) -> bool {
        let now = Utc::now().timestamp() as usize;
        now > self.exp
    }

    /// Check if user has specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.role == role
    }

    /// Check if user is admin
    pub fn is_admin(&self) -> bool {
        self.has_role("Admin")
    }
}
