// Allow async fn in traits - we use these internally and Send bounds are acceptable
#![allow(async_fn_in_trait)]

use anyhow::{Context, Result};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use std::sync::Arc;
use tokio::sync::oneshot;
use warp::Filter;

use crate::storage::{SecureStorage, TokenSet};

/// OAuth2 configuration
#[derive(Debug, Clone)]
pub struct OAuth2Config {
    /// Client ID from OAuth provider
    pub client_id: String,

    /// Client secret from OAuth provider
    pub client_secret: String,

    /// Authorization endpoint URL
    pub auth_url: String,

    /// Token endpoint URL
    pub token_url: String,

    /// Redirect URI for OAuth callback
    pub redirect_uri: String,

    /// Scopes to request
    pub scopes: Vec<String>,
}

/// OAuth2 provider trait
pub trait OAuth2Provider: Send + Sync {
    /// Get the service identifier (e.g., "github", "google")
    fn service_id(&self) -> &str;

    /// Get the OAuth2 configuration
    fn config(&self) -> &OAuth2Config;

    /// Start the OAuth2 authorization flow
    ///
    /// Returns the authorization URL, CSRF token, and PKCE verifier
    fn authorize(&self) -> Result<(String, CsrfToken, PkceCodeVerifier)> {
        let config = self.config();

        let client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new(config.auth_url.clone()).context("Invalid auth URL")?,
            Some(TokenUrl::new(config.token_url.clone()).context("Invalid token URL")?),
        )
        .set_redirect_uri(
            RedirectUrl::new(config.redirect_uri.clone()).context("Invalid redirect URI")?,
        );

        // Generate PKCE challenge for additional security
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate authorization URL
        let mut auth_request = client.authorize_url(CsrfToken::new_random);

        // Add scopes
        for scope in &config.scopes {
            auth_request = auth_request.add_scope(Scope::new(scope.clone()));
        }

        let (auth_url, csrf_token) = auth_request.set_pkce_challenge(pkce_challenge).url();

        Ok((auth_url.to_string(), csrf_token, pkce_verifier))
    }

    /// Complete the OAuth2 flow with authorization code
    ///
    /// # Arguments
    /// * `code` - Authorization code from callback
    /// * `pkce_verifier` - PKCE verifier from authorization step
    async fn exchange_code(
        &self,
        code: String,
        pkce_verifier: PkceCodeVerifier,
    ) -> Result<TokenSet> {
        let config = self.config();

        let client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new(config.auth_url.clone())?,
            Some(TokenUrl::new(config.token_url.clone())?),
        )
        .set_redirect_uri(RedirectUrl::new(config.redirect_uri.clone())?);

        // Exchange code for token with PKCE verifier
        let token_result = client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
            .context("Failed to exchange authorization code")?;

        // Calculate expiration
        // GitHub OAuth tokens don't expire, so default to 1 year if no expiration provided
        let expires_in = token_result
            .expires_in()
            .map(|d| d.as_secs() as i64)
            .unwrap_or(365 * 24 * 3600); // Default 1 year for non-expiring tokens
        let expires_at = chrono::Utc::now().timestamp() + expires_in;

        // Extract scopes
        let scopes = token_result
            .scopes()
            .map(|s| s.iter().map(|scope| scope.to_string()).collect())
            .unwrap_or_else(Vec::new);

        let token_set = TokenSet {
            access_token: token_result.access_token().secret().clone(),
            refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
            expires_at,
            scopes,
        };

        // Store token securely
        SecureStorage::store_token(self.service_id(), &token_set)?;

        tracing::info!("OAuth2 flow completed for {}", self.service_id());
        Ok(token_set)
    }

    /// Perform full OAuth2 flow with browser and local callback server
    async fn authenticate(&self) -> Result<TokenSet> {
        // Generate authorization URL and PKCE verifier
        let (auth_url, csrf_token, pkce_verifier) = self.authorize()?;

        tracing::info!("Opening browser for OAuth2 authorization...");
        tracing::info!("Auth URL: {}", auth_url);

        // Start local callback server
        let (tx, rx) = oneshot::channel();
        let tx = Arc::new(tokio::sync::Mutex::new(Some(tx)));

        let routes = warp::get()
            .and(warp::path("callback"))
            .and(warp::query::<std::collections::HashMap<String, String>>())
            .and(warp::any().map(move || tx.clone()))
            .and_then(|params: std::collections::HashMap<String, String>, tx: Arc<tokio::sync::Mutex<Option<oneshot::Sender<(String, String)>>>>| async move {
                let code = params.get("code").cloned().unwrap_or_default();
                let state = params.get("state").cloned().unwrap_or_default();

                if let Some(sender) = tx.lock().await.take() {
                    let _ = sender.send((code, state));
                }

                Ok::<_, warp::Rejection>(warp::reply::html(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MyMe - Authorization Successful</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            color: #e4e6eb;
        }
        .container {
            text-align: center;
            padding: 3rem;
            background: rgba(255, 255, 255, 0.05);
            border-radius: 16px;
            border: 1px solid rgba(255, 255, 255, 0.1);
            backdrop-filter: blur(10px);
            max-width: 420px;
        }
        .logo {
            width: 80px;
            height: 80px;
            background: #6c63ff;
            border-radius: 20px;
            display: flex;
            align-items: center;
            justify-content: center;
            margin: 0 auto 1.5rem;
            font-size: 2.5rem;
            font-weight: bold;
            color: white;
        }
        h1 {
            font-size: 1.75rem;
            margin-bottom: 0.5rem;
            color: #4ade80;
        }
        .subtitle {
            font-size: 1rem;
            color: #a8a8b3;
            margin-bottom: 1.5rem;
        }
        .message {
            background: rgba(74, 222, 128, 0.1);
            border: 1px solid rgba(74, 222, 128, 0.3);
            border-radius: 8px;
            padding: 1rem;
            color: #4ade80;
        }
        .close-hint {
            margin-top: 1.5rem;
            font-size: 0.875rem;
            color: #6c6c7a;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="logo">M</div>
        <h1>Authorization Successful</h1>
        <p class="subtitle">GitHub account connected</p>
        <div class="message">
            ✓ You can close this window and return to MyMe
        </div>
        <p class="close-hint">This window can be safely closed</p>
    </div>
</body>
</html>"#))
            });

        // Start server in background
        let server = warp::serve(routes).bind(([127, 0, 0, 1], 8080));
        tokio::spawn(server);

        // Open browser
        webbrowser::open(&auth_url).context("Failed to open browser")?;

        // Wait for callback
        let (code, state) = rx.await.context("Failed to receive OAuth callback")?;

        // Validate CSRF token
        if state != *csrf_token.secret() {
            anyhow::bail!("CSRF token mismatch");
        }

        tracing::info!("Received OAuth callback, exchanging code for token...");

        // Exchange code for token with PKCE verifier
        self.exchange_code(code, pkce_verifier).await
    }

    /// Get stored token, or None if not authenticated
    fn get_token(&self) -> Option<TokenSet> {
        SecureStorage::retrieve_token(self.service_id()).ok()
    }

    /// Check if authenticated and token is valid
    fn is_authenticated(&self) -> bool {
        self.get_token()
            .map(|token| !token.is_expired())
            .unwrap_or(false)
    }

    /// Sign out (delete stored token)
    fn sign_out(&self) -> Result<()> {
        SecureStorage::delete_token(self.service_id())
    }
}
