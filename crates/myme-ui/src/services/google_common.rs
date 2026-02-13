//! Shared helpers for Google services (Gmail, Calendar).
//! Reduces duplication of token refresh, config loading, and cache paths.

use std::path::PathBuf;

use myme_auth::{GoogleOAuth2Provider, SecureStorage};

/// Returns (client_id, client_secret) from config if Google OAuth is configured.
pub fn get_google_config() -> Option<(String, String)> {
    match myme_core::Config::load() {
        Ok(config) => {
            let client_id = config.google.as_ref()?.client_id.clone()?;
            let client_secret = config.google.as_ref()?.client_secret.clone()?;
            Some((client_id, client_secret))
        }
        Err(_) => None,
    }
}

/// Get a valid Google access token, refreshing if expired.
/// Returns None if not authenticated or refresh fails.
pub fn get_google_access_token() -> Option<String> {
    let token_set = SecureStorage::retrieve_token("google").ok()?;

    if token_set.is_expired() {
        let refresh_token = token_set.refresh_token.as_ref()?;
        let (client_id, client_secret) = get_google_config()?;
        let rt = tokio::runtime::Runtime::new().ok()?;
        let provider = GoogleOAuth2Provider::new(client_id, client_secret);

        let new_tokens = rt.block_on(provider.refresh_token(refresh_token)).ok()?;
        let expires_at = chrono::Utc::now().timestamp() + new_tokens.expires_in as i64;
        let new_token_set = myme_auth::TokenSet {
            access_token: new_tokens.access_token.clone(),
            refresh_token: new_tokens.refresh_token.or(token_set.refresh_token.clone()),
            expires_at,
            scopes: new_tokens.scope.split(' ').map(|s| s.to_string()).collect(),
        };
        let _ = SecureStorage::store_token("google", &new_token_set);
        return Some(new_tokens.access_token);
    }

    Some(token_set.access_token)
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
