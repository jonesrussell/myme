//! Google API client with transparent token lifecycle management.
//!
//! Wraps token state with proactive refresh before expiry.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use thiserror::Error;
use tracing::{debug, info, warn};

use crate::google::GoogleOAuth2Provider;
use crate::storage::{SecureStorage, TokenSet};

/// Errors specific to the Google API client middleware.
#[derive(Error, Debug)]
pub enum GoogleApiError {
    #[error("Not authenticated — no Google tokens found")]
    NotAuthenticated,

    #[error("Token refresh failed: {0}")]
    RefreshFailed(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

struct TokenState {
    access_token: String,
    refresh_token: String,
    expires_at: DateTime<Utc>,
}

/// Auth middleware for Google APIs.
///
/// - **Proactive refresh**: if token expires within 5 minutes, refreshes before sending.
/// - **Persistence**: writes refreshed tokens back to the system keyring.
/// - **Thread-safe**: `RwLock` on token state for concurrent async access.
pub struct GoogleApiClient {
    http: reqwest::Client,
    client_id: String,
    client_secret: String,
    token_state: RwLock<TokenState>,
}

impl GoogleApiClient {
    /// Create a client from an existing token set in the system keyring.
    ///
    /// Returns `GoogleApiError::NotAuthenticated` if no tokens are stored.
    pub fn from_keyring(
        client_id: String,
        client_secret: String,
    ) -> Result<Arc<Self>, GoogleApiError> {
        let token_set = SecureStorage::retrieve_token("google")
            .map_err(|_| GoogleApiError::NotAuthenticated)?;

        let refresh_token = token_set.refresh_token.ok_or(GoogleApiError::NotAuthenticated)?;

        let expires_at = DateTime::from_timestamp(token_set.expires_at, 0).unwrap_or_else(Utc::now);

        Ok(Arc::new(Self {
            http: reqwest::Client::new(),
            client_id,
            client_secret,
            token_state: RwLock::new(TokenState {
                access_token: token_set.access_token,
                refresh_token,
                expires_at,
            }),
        }))
    }

    /// Get the current access token, refreshing proactively if it expires within 5 minutes.
    pub async fn access_token(&self) -> Result<String, GoogleApiError> {
        {
            let state = self.token_state.read();
            let now = Utc::now();
            let buffer = chrono::Duration::minutes(5);
            if state.expires_at > now + buffer {
                return Ok(state.access_token.clone());
            }
        }
        // Token needs refresh
        self.force_refresh().await
    }

    /// Force a token refresh regardless of expiry.
    pub async fn force_refresh(&self) -> Result<String, GoogleApiError> {
        let refresh_token = { self.token_state.read().refresh_token.clone() };

        debug!("Refreshing Google access token");
        let provider =
            GoogleOAuth2Provider::new(self.client_id.clone(), self.client_secret.clone());

        let new_tokens = provider
            .refresh_token(&refresh_token)
            .await
            .map_err(|e| GoogleApiError::RefreshFailed(e.to_string()))?;

        let expires_at = Utc::now() + chrono::Duration::seconds(new_tokens.expires_in as i64);
        let new_access = new_tokens.access_token.clone();
        let new_refresh = new_tokens.refresh_token.unwrap_or_else(|| refresh_token.clone());

        // Update in-memory state
        {
            let mut state = self.token_state.write();
            state.access_token = new_access.clone();
            state.refresh_token = new_refresh.clone();
            state.expires_at = expires_at;
        }

        // Persist to keyring
        let token_set = TokenSet {
            access_token: new_access.clone(),
            refresh_token: Some(new_refresh),
            expires_at: expires_at.timestamp(),
            scopes: new_tokens
                .scope
                .as_deref()
                .unwrap_or("")
                .split(' ')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect(),
        };
        if let Err(e) = SecureStorage::store_token("google", &token_set) {
            warn!("Failed to persist refreshed token to keyring: {}", e);
        } else {
            info!("Persisted refreshed Google token to keyring");
        }

        Ok(new_access)
    }

    /// Get a reference to the inner HTTP client.
    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    /// Create a client with a fixed token (for testing — no refresh capability).
    pub fn new_for_test(access_token: &str) -> Arc<Self> {
        Arc::new(Self {
            http: reqwest::Client::new(),
            client_id: String::new(),
            client_secret: String::new(),
            token_state: RwLock::new(TokenState {
                access_token: access_token.to_string(),
                refresh_token: String::new(),
                expires_at: Utc::now() + chrono::Duration::hours(1),
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn test_new_for_test() {
        let client = GoogleApiClient::new_for_test("test_token");
        let state = client.token_state.read();
        assert_eq!(state.access_token, "test_token");
        assert!(state.expires_at > Utc::now());
    }

    #[tokio::test]
    async fn test_access_token_returns_valid_token() {
        let client = GoogleApiClient::new_for_test("my_token");
        let token = client.access_token().await.unwrap();
        assert_eq!(token, "my_token");
    }
}
