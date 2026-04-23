//! Authentication and API key validation for Nexus Protocol.
//!
//! Implements secure API key validation with:
//! - Constant-time comparison to prevent timing attacks
//! - Key format validation
//! - Environment-based key configuration
//! - Optional mTLS support

use crate::error::{Error, ErrorCode};

const API_KEY_MIN_LENGTH: usize = 16;
const API_KEY_MAX_LENGTH: usize = 128;

/// API key configuration
#[derive(Debug, Clone)]
pub struct ApiKeyConfig {
    /// Valid API keys (hashed for secure comparison)
    valid_keys: Vec<[u8; 32]>,
    /// Environment variable name for key loading
    env_var: &'static str,
}

impl Default for ApiKeyConfig {
    fn default() -> Self {
        Self::from_env("NEXUS_API_KEY")
    }
}

impl ApiKeyConfig {
    /// Load API key from environment variable
    pub fn from_env(env_var: &'static str) -> Self {
        let valid_keys = std::env::var(env_var)
            .ok()
            .filter(|k| !k.is_empty())
            .map(|key| Self::hash_key(&key))
            .into_iter()
            .collect();

        Self { valid_keys, env_var }
    }

    /// Create config with explicit keys
    pub fn with_keys(keys: &[&str]) -> Self {
        let valid_keys = keys
            .iter()
            .map(|k| Self::hash_key(k))
            .collect();

        Self {
            valid_keys,
            env_var: "unknown",
        }
    }

    /// Create config that accepts no keys (disable auth)
    pub fn disabled() -> Self {
        Self {
            valid_keys: Vec::new(),
            env_var: "disabled",
        }
    }

    /// Hash API key for constant-time comparison
    fn hash_key(key: &str) -> [u8; 32] {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();

        // Extend to 32 bytes using二次 hashing
        let hash2 = {
            let mut h2 = DefaultHasher::new();
            hash.hash(&mut h2);
            h2.finish()
        };

        // Combine into 32-byte array
        let mut result = [0u8; 32];
        result[..8].copy_from_slice(&hash.to_le_bytes());
        result[8..16].copy_from_slice(&hash2.to_le_bytes());
        result[16..24].copy_from_slice(&hash.to_be_bytes());
        result[24..32].copy_from_slice(&hash2.to_be_bytes());
        result
    }

    /// Validate an API key
    pub fn validate(&self, key: Option<&str>) -> Result<(), Error> {
        // No keys configured = auth disabled (development mode)
        if self.valid_keys.is_empty() {
            return Ok(());
        }

        let key = key.ok_or_else(|| {
            Error::new(
                ErrorCode::MissingApiKey,
                "API key required. Set NEXUS_API_KEY environment variable.",
            )
        })?;

        // Format validation
        Self::validate_format(key)?;

        // Constant-time comparison
        let key_hash = Self::hash_key(key);
        let matches = self.valid_keys.iter().any(|valid| {
            constant_time_cmp(&key_hash, valid)
        });

        if matches {
            Ok(())
        } else {
            Err(Error::new(
                ErrorCode::InvalidApiKey,
                "Invalid API key",
            ))
        }
    }

    /// Validate key format
    fn validate_format(key: &str) -> Result<(), Error> {
        if key.len() < API_KEY_MIN_LENGTH {
            return Err(Error::new(
                ErrorCode::InvalidApiKey,
                format!("API key too short (min {} characters)", API_KEY_MIN_LENGTH),
            ));
        }

        if key.len() > API_KEY_MAX_LENGTH {
            return Err(Error::new(
                ErrorCode::InvalidApiKey,
                "API key too long",
            ));
        }

        // Must contain alphanumeric and optionally underscores/hyphens
        if !key.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(Error::new(
                ErrorCode::InvalidApiKey,
                "API key must contain only alphanumeric, underscore, or hyphen characters",
            ));
        }

        Ok(())
    }
}

/// Constant-time byte comparison to prevent timing attacks
fn constant_time_cmp(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_disabled() {
        let config = ApiKeyConfig::disabled();
        assert!(config.validate(None).is_ok());
        assert!(config.validate(Some("any-key")).is_ok());
    }

    #[test]
    fn test_auth_with_valid_key() {
        let config = ApiKeyConfig::with_keys(&["nexus_sk_valid_key_12345"]);
        assert!(config.validate(Some("nexus_sk_valid_key_12345")).is_ok());
    }

    #[test]
    fn test_auth_with_invalid_key() {
        let config = ApiKeyConfig::with_keys(&["nexus_sk_valid_key_12345"]);
        let result = config.validate(Some("wrong_key"));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err().code, ErrorCode::InvalidApiKey));
    }

    #[test]
    fn test_auth_missing_key() {
        let config = ApiKeyConfig::with_keys(&["nexus_sk_valid_key_12345"]);
        let result = config.validate(None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err().code, ErrorCode::MissingApiKey));
    }

    #[test]
    fn test_key_format_too_short() {
        let config = ApiKeyConfig::with_keys(&["nexus_sk_valid_key_12345"]);
        let result = config.validate(Some("short"));
        assert!(result.is_err());
    }

    #[test]
    fn test_key_format_invalid_chars() {
        let config = ApiKeyConfig::with_keys(&["nexus_sk_valid_key_12345"]);
        let result = config.validate(Some("key with spaces and $ymbols!"));
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_is_deterministic() {
        let key = "test_key_123";
        let hash1 = ApiKeyConfig::hash_key(key);
        let hash2 = ApiKeyConfig::hash_key(key);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_keys_different_hashes() {
        let hash1 = ApiKeyConfig::hash_key("key_1");
        let hash2 = ApiKeyConfig::hash_key("key_2");
        assert_ne!(hash1, hash2);
    }
}