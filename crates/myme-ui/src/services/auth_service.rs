//! Auth backend: async OAuth authentication.
//! OAuth flow runs off the UI thread; results sent via mpsc.

use std::sync::Arc;

use myme_auth::{GitHubAuth, OAuth2Provider, TokenSet};

use crate::bridge;

/// Error type for auth operations
#[derive(Debug, Clone)]
pub enum AuthError {
    OAuth(String),
    NotConfigured,
    NotInitialized,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::OAuth(s) => write!(f, "Authentication failed: {}", s),
            AuthError::NotConfigured => {
                write!(f, "GitHub OAuth not configured in config.toml")
            }
            AuthError::NotInitialized => write!(f, "Auth service not initialized"),
        }
    }
}

impl std::error::Error for AuthError {}

/// Messages sent from async operations back to the UI thread
#[derive(Debug)]
pub enum AuthServiceMessage {
    /// Result of OAuth authentication
    AuthenticateDone(Result<TokenSet, AuthError>),
}

/// Request to start OAuth authentication asynchronously.
/// Sends `AuthenticateDone` on the channel when complete.
pub fn request_authenticate(
    tx: &std::sync::mpsc::Sender<AuthServiceMessage>,
    provider: Arc<GitHubAuth>,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(AuthServiceMessage::AuthenticateDone(Err(
                AuthError::NotInitialized,
            )));
            return;
        }
    };

    runtime.spawn(async move {
        let result = provider
            .authenticate()
            .await
            .map_err(|e| AuthError::OAuth(e.to_string()));
        let _ = tx.send(AuthServiceMessage::AuthenticateDone(result));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_error_display() {
        assert!(format!("{}", AuthError::OAuth("timeout".into())).contains("Authentication"));
        assert!(format!("{}", AuthError::NotConfigured).contains("not configured"));
        assert!(format!("{}", AuthError::NotInitialized).contains("not initialized"));
    }

    #[test]
    fn auth_service_message_variants() {
        let _auth_err: AuthServiceMessage =
            AuthServiceMessage::AuthenticateDone(Err(AuthError::NotInitialized));
    }
}
