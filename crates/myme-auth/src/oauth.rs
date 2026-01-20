use anyhow::{Context, Result};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use oauth2::basic::{BasicClient, BasicTokenType};
use oauth2::reqwest::async_http_client;
use oauth2::{EmptyExtraTokenFields, StandardTokenResponse};
use std::sync::Arc;
use tokio::sync::oneshot;
use url::Url;
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
    /// Returns the authorization URL to open in browser
    fn authorize(&self) -> Result<(String, CsrfToken)> {
        let config = self.config();

        let client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new(config.auth_url.clone())
                .context("Invalid auth URL")?,
            Some(TokenUrl::new(config.token_url.clone())
                .context("Invalid token URL")?),
        )
        .set_redirect_uri(
            RedirectUrl::new(config.redirect_uri.clone())
                .context("Invalid redirect URI")?,
        );

        // Generate PKCE challenge for additional security
        let (pkce_challenge, _pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate authorization URL
        let mut auth_request = client
            .authorize_url(CsrfToken::new_random);

        // Add scopes
        for scope in &config.scopes {
            auth_request = auth_request.add_scope(Scope::new(scope.clone()));
        }

        let (auth_url, csrf_token) = auth_request
            .set_pkce_challenge(pkce_challenge)
            .url();

        Ok((auth_url.to_string(), csrf_token))
    }

    /// Complete the OAuth2 flow with authorization code
    ///
    /// # Arguments
    /// * `code` - Authorization code from callback
    /// * `state` - CSRF token for validation
    async fn exchange_code(&self, code: String, _state: String) -> Result<TokenSet> {
        let config = self.config();

        let client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new(config.auth_url.clone())?,
            Some(TokenUrl::new(config.token_url.clone())?),
        )
        .set_redirect_uri(RedirectUrl::new(config.redirect_uri.clone())?);

        // Exchange code for token
        let token_result = client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await
            .context("Failed to exchange authorization code")?;

        // Calculate expiration
        let expires_in = token_result.expires_in()
            .map(|d| d.as_secs() as i64)
            .unwrap_or(3600); // Default 1 hour
        let expires_at = chrono::Utc::now().timestamp() + expires_in;

        // Extract scopes
        let scopes = token_result.scopes()
            .map(|s| s.iter().map(|scope| scope.to_string()).collect())
            .unwrap_or_else(Vec::new);

        let token_set = TokenSet {
            access_token: token_result.access_token().secret().clone(),
            refresh_token: token_result.refresh_token()
                .map(|t| t.secret().clone()),
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
        // Generate authorization URL
        let (auth_url, csrf_token) = self.authorize()?;

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

                Ok::<_, warp::Rejection>(warp::reply::html(
                    "<html><body><h1>Authorization successful!</h1><p>You can close this window and return to MyMe.</p></body></html>"
                ))
            });

        // Start server in background
        let server = warp::serve(routes).bind(([127, 0, 0, 1], 8080));
        tokio::spawn(server);

        // Open browser
        webbrowser::open(&auth_url)
            .context("Failed to open browser")?;

        // Wait for callback
        let (code, state) = rx.await
            .context("Failed to receive OAuth callback")?;

        // Validate CSRF token
        if state != csrf_token.secret() {
            anyhow::bail!("CSRF token mismatch");
        }

        // Exchange code for token
        self.exchange_code(code, state).await
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
