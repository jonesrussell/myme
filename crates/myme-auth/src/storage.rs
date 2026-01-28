use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Token set for OAuth2 authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSet {
    /// Access token for API requests
    pub access_token: String,

    /// Optional refresh token for token renewal
    pub refresh_token: Option<String>,

    /// Token expiration timestamp (Unix timestamp)
    pub expires_at: i64,

    /// Scopes granted to this token
    pub scopes: Vec<String>,
}

impl TokenSet {
    /// Check if the token needs refresh (within 5 minutes of expiry)
    pub fn needs_refresh(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        now >= self.expires_at - 300 // 5 minute buffer
    }

    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        now >= self.expires_at
    }
}

/// Secure storage for OAuth tokens using file-based storage
/// Tokens are stored in the user's config directory
pub struct SecureStorage;

impl SecureStorage {
    /// Get the token file path for a service
    fn token_path(service: &str) -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("myme")
            .join("tokens");

        // Ensure directory exists
        fs::create_dir_all(&config_dir)
            .context("Failed to create tokens directory")?;

        Ok(config_dir.join(format!("{}.json", service)))
    }

    /// Store a token set securely
    ///
    /// # Arguments
    /// * `service` - Service identifier (e.g., "github", "google")
    /// * `token_set` - The token set to store
    pub fn store_token(service: &str, token_set: &TokenSet) -> Result<()> {
        let path = Self::token_path(service)?;

        let json = serde_json::to_string_pretty(token_set)
            .context("Failed to serialize token set")?;

        fs::write(&path, &json)
            .context("Failed to write token file")?;

        tracing::info!("Stored token for service: {} at {:?}", service, path);
        Ok(())
    }

    /// Retrieve a token set from secure storage
    ///
    /// # Arguments
    /// * `service` - Service identifier (e.g., "github", "google")
    pub fn retrieve_token(service: &str) -> Result<TokenSet> {
        let path = Self::token_path(service)?;

        let json = fs::read_to_string(&path)
            .context("Failed to read token file")?;

        let token_set: TokenSet = serde_json::from_str(&json)
            .context("Failed to deserialize token set")?;

        tracing::info!("Retrieved token for service: {}", service);
        Ok(token_set)
    }

    /// Delete a token set from secure storage
    ///
    /// # Arguments
    /// * `service` - Service identifier (e.g., "github", "google")
    pub fn delete_token(service: &str) -> Result<()> {
        let path = Self::token_path(service)?;

        if path.exists() {
            fs::remove_file(&path)
                .context("Failed to delete token file")?;
            tracing::info!("Deleted token for service: {}", service);
        }

        Ok(())
    }

    /// Check if a token exists for a service
    ///
    /// # Arguments
    /// * `service` - Service identifier (e.g., "github", "google")
    pub fn has_token(service: &str) -> bool {
        Self::retrieve_token(service).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_expiry() {
        let now = chrono::Utc::now().timestamp();

        // Expired token
        let expired = TokenSet {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: now - 3600, // 1 hour ago
            scopes: vec![],
        };
        assert!(expired.is_expired());
        assert!(expired.needs_refresh());

        // Valid token
        let valid = TokenSet {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: now + 3600, // 1 hour from now
            scopes: vec![],
        };
        assert!(!valid.is_expired());
        assert!(!valid.needs_refresh());

        // Needs refresh soon
        let soon = TokenSet {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_at: now + 200, // 3 minutes from now
            scopes: vec![],
        };
        assert!(!soon.is_expired());
        assert!(soon.needs_refresh());
    }
}
