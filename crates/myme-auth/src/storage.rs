//! Secure storage for OAuth tokens using the system keyring.
//!
//! Uses the system keyring for secure token storage:
//! - Windows: Windows Credential Manager
//! - macOS: Keychain
//! - Linux: Secret Service (requires libsecret)
//!
//! Includes automatic migration from legacy plaintext file storage.

use anyhow::{Context, Result};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Service name used for keyring entries
const KEYRING_SERVICE: &str = "myme";

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

/// Secure storage for OAuth tokens using the system keyring.
///
/// # Security
/// Tokens are stored in the platform's secure credential store:
/// - Windows: Credential Manager
/// - macOS: Keychain
/// - Linux: Secret Service (gnome-keyring, KDE Wallet, etc.)
///
/// # Migration
/// On first access, the storage automatically migrates any existing
/// plaintext tokens from the legacy `~/.config/myme/tokens/` directory.
pub struct SecureStorage;

impl SecureStorage {
    /// Get the legacy token file path for a service (for migration)
    fn legacy_token_path(service: &str) -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("myme")
            .join("tokens");

        Ok(config_dir.join(format!("{}.json", service)))
    }

    /// Migrate a token from legacy file storage to keyring.
    /// Returns Ok(Some(token)) if migration was successful.
    /// Returns Ok(None) if no legacy token exists.
    fn migrate_from_file(service: &str) -> Result<Option<TokenSet>> {
        let path = Self::legacy_token_path(service)?;

        if !path.exists() {
            return Ok(None);
        }

        tracing::info!("Found legacy token file for {}, migrating to keyring...", service);

        // Read the legacy token
        let json = fs::read_to_string(&path)
            .context("Failed to read legacy token file")?;

        let token_set: TokenSet = serde_json::from_str(&json)
            .context("Failed to deserialize legacy token")?;

        // Store in keyring
        if let Err(e) = Self::store_in_keyring(service, &token_set) {
            tracing::error!("Failed to store token in keyring during migration: {}", e);
            // Keep the file if migration fails
            return Err(e);
        }

        // Delete the legacy file after successful migration
        if let Err(e) = fs::remove_file(&path) {
            tracing::warn!("Failed to delete legacy token file after migration: {}", e);
            // Don't fail the migration if we can't delete the file
        } else {
            tracing::info!("Successfully migrated {} token to keyring and deleted legacy file", service);
        }

        Ok(Some(token_set))
    }

    /// Store a token in the system keyring.
    fn store_in_keyring(service: &str, token_set: &TokenSet) -> Result<()> {
        let entry = Entry::new(KEYRING_SERVICE, service)
            .context("Failed to create keyring entry")?;

        let json = serde_json::to_string(token_set)
            .context("Failed to serialize token set")?;

        entry.set_password(&json)
            .context("Failed to store token in keyring")?;

        Ok(())
    }

    /// Retrieve a token from the system keyring.
    fn retrieve_from_keyring(service: &str) -> Result<TokenSet> {
        let entry = Entry::new(KEYRING_SERVICE, service)
            .context("Failed to create keyring entry")?;

        let json = entry.get_password()
            .context("Failed to retrieve token from keyring")?;

        let token_set: TokenSet = serde_json::from_str(&json)
            .context("Failed to deserialize token from keyring")?;

        Ok(token_set)
    }

    /// Store a token set securely in the system keyring.
    ///
    /// # Arguments
    /// * `service` - Service identifier (e.g., "github", "google")
    /// * `token_set` - The token set to store
    pub fn store_token(service: &str, token_set: &TokenSet) -> Result<()> {
        Self::store_in_keyring(service, token_set)?;
        tracing::info!("Stored token for service: {} in system keyring", service);
        Ok(())
    }

    /// Retrieve a token set from secure storage.
    ///
    /// This method first checks the system keyring. If no token is found,
    /// it checks for a legacy plaintext file and automatically migrates it.
    ///
    /// # Arguments
    /// * `service` - Service identifier (e.g., "github", "google")
    pub fn retrieve_token(service: &str) -> Result<TokenSet> {
        // Try to get from keyring first
        match Self::retrieve_from_keyring(service) {
            Ok(token_set) => {
                tracing::debug!("Retrieved token for {} from keyring", service);
                return Ok(token_set);
            }
            Err(e) => {
                tracing::debug!("Token not in keyring for {}: {}", service, e);
            }
        }

        // Try to migrate from legacy file storage
        if let Ok(Some(token_set)) = Self::migrate_from_file(service) {
            return Ok(token_set);
        }

        anyhow::bail!("No token found for service: {}", service)
    }

    /// Delete a token set from secure storage.
    ///
    /// This deletes from both the keyring and any legacy file storage.
    ///
    /// # Arguments
    /// * `service` - Service identifier (e.g., "github", "google")
    pub fn delete_token(service: &str) -> Result<()> {
        // Delete from keyring
        match Entry::new(KEYRING_SERVICE, service) {
            Ok(entry) => {
                // Note: keyring v2 uses delete_password(), v3 uses delete_credential()
                match entry.delete_password() {
                    Ok(()) => {
                        tracing::info!("Deleted token for {} from keyring", service);
                    }
                    Err(keyring::Error::NoEntry) => {
                        tracing::debug!("No token in keyring for {}", service);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to delete token from keyring: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to create keyring entry for deletion: {}", e);
            }
        }

        // Also delete any legacy file
        if let Ok(path) = Self::legacy_token_path(service) {
            if path.exists() {
                if let Err(e) = fs::remove_file(&path) {
                    tracing::warn!("Failed to delete legacy token file: {}", e);
                } else {
                    tracing::info!("Deleted legacy token file for {}", service);
                }
            }
        }

        Ok(())
    }

    /// Check if a token exists for a service.
    ///
    /// # Arguments
    /// * `service` - Service identifier (e.g., "github", "google")
    pub fn has_token(service: &str) -> bool {
        // Check keyring first
        if let Ok(entry) = Entry::new(KEYRING_SERVICE, service) {
            if entry.get_password().is_ok() {
                return true;
            }
        }

        // Check legacy file
        if let Ok(path) = Self::legacy_token_path(service) {
            if path.exists() {
                return true;
            }
        }

        false
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

    #[test]
    fn test_keyring_service_name() {
        assert_eq!(KEYRING_SERVICE, "myme");
    }
}
