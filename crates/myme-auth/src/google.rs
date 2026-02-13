//! Google OAuth2 provider for Gmail and Calendar access.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";

// Scopes for Gmail and Calendar access
const GMAIL_SCOPE: &str = "https://www.googleapis.com/auth/gmail.modify";
const CALENDAR_SCOPE: &str = "https://www.googleapis.com/auth/calendar";
const USERINFO_SCOPE: &str = "https://www.googleapis.com/auth/userinfo.email";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub token_type: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleUserInfo {
    pub email: String,
    pub verified_email: bool,
    pub picture: Option<String>,
}

pub struct GoogleOAuth2Provider {
    pub client_id: String,
    pub client_secret: String,
}

impl GoogleOAuth2Provider {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self { client_id, client_secret }
    }

    /// Generate authorization URL for OAuth flow.
    /// Returns (url, state) where state should be verified on callback.
    pub fn authorization_url(&self, port: u16) -> (String, String) {
        let state = uuid::Uuid::new_v4().to_string();
        let redirect_uri = format!("http://localhost:{}/callback", port);
        let scopes = format!("{} {} {}", GMAIL_SCOPE, CALENDAR_SCOPE, USERINFO_SCOPE);

        let url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&access_type=offline&prompt=consent",
            GOOGLE_AUTH_URL,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&redirect_uri),
            urlencoding::encode(&scopes),
            urlencoding::encode(&state),
        );

        (url, state)
    }

    /// Exchange authorization code for tokens.
    #[tracing::instrument(skip(self, code), level = "info")]
    pub async fn exchange_code(&self, code: &str, port: u16) -> Result<GoogleTokenResponse> {
        let redirect_uri = format!("http://localhost:{}/callback", port);
        let client = reqwest::Client::new();

        let response = client
            .post(GOOGLE_TOKEN_URL)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("code", code),
                ("grant_type", "authorization_code"),
                ("redirect_uri", &redirect_uri),
            ])
            .send()
            .await
            .context("Failed to send token request")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Token exchange failed: {}", error_text);
        }

        response.json::<GoogleTokenResponse>().await.context("Failed to parse token response")
    }

    /// Refresh an expired access token.
    #[tracing::instrument(skip(self, refresh_token), level = "info")]
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<GoogleTokenResponse> {
        let client = reqwest::Client::new();

        let response = client
            .post(GOOGLE_TOKEN_URL)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("refresh_token", refresh_token),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await
            .context("Failed to send refresh request")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Token refresh failed: {}", error_text);
        }

        response.json::<GoogleTokenResponse>().await.context("Failed to parse refresh response")
    }

    /// Get user info (email) from access token.
    #[tracing::instrument(skip(self, access_token), level = "info")]
    pub async fn get_user_info(&self, access_token: &str) -> Result<GoogleUserInfo> {
        let client = reqwest::Client::new();

        let response = client
            .get(GOOGLE_USERINFO_URL)
            .bearer_auth(access_token)
            .send()
            .await
            .context("Failed to fetch user info")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("User info request failed: {}", error_text);
        }

        response.json::<GoogleUserInfo>().await.context("Failed to parse user info")
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn test_google_oauth_provider_creation() {
        let provider = GoogleOAuth2Provider::new(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
        );
        assert_eq!(provider.client_id, "test_client_id");
    }

    #[test]
    fn test_google_auth_url_contains_scopes() {
        let provider = GoogleOAuth2Provider::new(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
        );
        let (url, _state) = provider.authorization_url(8080);
        assert!(url.contains("scope="));
        assert!(url.contains("gmail"));
        assert!(url.contains("calendar"));
    }

    #[test]
    fn test_google_auth_url_contains_offline_access() {
        let provider = GoogleOAuth2Provider::new(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
        );
        let (url, _state) = provider.authorization_url(8080);
        assert!(url.contains("access_type=offline"));
        assert!(url.contains("prompt=consent"));
    }

    #[test]
    fn test_google_state_is_unique() {
        let provider = GoogleOAuth2Provider::new(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
        );
        let (_, state1) = provider.authorization_url(8080);
        let (_, state2) = provider.authorization_url(8080);
        assert_ne!(state1, state2);
    }
}
