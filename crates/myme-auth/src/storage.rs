use anyhow::{Context, Result};
use keyring::Entry;
use serde::{Deserialize, Serialize};

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

/// Secure storage for OAuth tokens using platform keyring
pub struct SecureStorage;

impl SecureStorage {
    /// Store a token set securely
    ///
    /// # Arguments
    /// * `service` - Service identifier (e.g., "github", "google")
    /// * `token_set` - The token set to store
    pub fn store_token(service: &str, token_set: &TokenSet) -> Result<()> {
        let entry = Entry::new("myme", service)
            .context("Failed to create keyring entry")?;

        let json = serde_json::to_string(token_set)
            .context("Failed to serialize token set")?;

        entry.set_password(&json)
            .context("Failed to store token in keyring")?;

        tracing::info!("Stored token for service: {}", service);
        Ok(())
    }

    /// Retrieve a token set from secure storage
    ///
    /// # Arguments
    /// * `service` - Service identifier (e.g., "github", "google")
    pub fn retrieve_token(service: &str) -> Result<TokenSet> {
        let entry = Entry::new("myme", service)
            .context("Failed to create keyring entry")?;

        let json = entry.get_password()
            .context("Failed to retrieve token from keyring")?;

        let token_set: TokenSet = serde_json::from_str(&json)
            .context("Failed to deserialize token set")?;

        tracing::debug!("Retrieved token for service: {}", service);
        Ok(token_set)
    }

    /// Delete a token set from secure storage
    ///
    /// # Arguments
    /// * `service` - Service identifier (e.g., "github", "google")
    pub fn delete_token(service: &str) -> Result<()> {
        let entry = Entry::new("myme", service)
            .context("Failed to create keyring entry")?;

        entry.delete_password()
            .context("Failed to delete token from keyring")?;

        tracing::info!("Deleted token for service: {}", service);
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
