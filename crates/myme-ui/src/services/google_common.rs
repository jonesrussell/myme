//! Shared helpers for Google services (Gmail, Calendar).
//! Reduces duplication of token refresh, config loading, and cache paths.

use std::path::PathBuf;
use std::sync::Arc;

use myme_auth::{GoogleApiClient, GoogleApiError, DEFAULT_GOOGLE_CLIENT_ID, DEFAULT_GOOGLE_CLIENT_SECRET};

/// Returns (client_id, client_secret) from config, falling back to compiled defaults.
pub fn get_google_credentials() -> (String, String) {
    match myme_core::Config::load() {
        Ok(config) => {
            let client_id = config
                .google
                .as_ref()
                .and_then(|g| g.client_id.clone())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| DEFAULT_GOOGLE_CLIENT_ID.to_string());
            let client_secret = config
                .google
                .as_ref()
                .and_then(|g| g.client_secret.clone())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| DEFAULT_GOOGLE_CLIENT_SECRET.to_string());
            (client_id, client_secret)
        }
        Err(_) => (
            DEFAULT_GOOGLE_CLIENT_ID.to_string(),
            DEFAULT_GOOGLE_CLIENT_SECRET.to_string(),
        ),
    }
}

/// Create a GoogleApiClient from the keyring, using config credentials.
/// Returns None if not authenticated.
pub fn create_google_api_client() -> Option<Arc<GoogleApiClient>> {
    let (client_id, client_secret) = get_google_credentials();
    match GoogleApiClient::from_keyring(client_id, client_secret) {
        Ok(client) => Some(client),
        Err(GoogleApiError::NotAuthenticated) => {
            tracing::debug!("Google not authenticated");
            None
        }
        Err(e) => {
            tracing::warn!("Failed to create Google API client: {}", e);
            None
        }
    }
}

/// Get the configured sync interval in seconds (default: 300).
pub fn get_sync_interval_secs() -> u64 {
    match myme_core::Config::load() {
        Ok(config) => config
            .google
            .as_ref()
            .map(|g| g.sync_interval_secs)
            .unwrap_or(300),
        Err(_) => 300,
    }
}

/// Config directory for MyMe (e.g. ~/.config/myme on Linux).
fn myme_config_dir() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("myme")
}

/// Path to a Google-related cache file under the config directory.
/// Example: `get_google_cache_path("gmail_cache.db")` -> `.../myme/gmail_cache.db`.
pub fn get_google_cache_path(name: &str) -> PathBuf {
    myme_config_dir().join(name)
}
