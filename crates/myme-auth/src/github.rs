use crate::oauth::{OAuth2Config, OAuth2Provider};

const GITHUB_AUTH_URL: &str = "https://github.com/login/oauth/authorize";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const DEFAULT_REDIRECT_URI: &str = "http://localhost:8080/callback";

/// GitHub OAuth2 authentication provider
pub struct GitHubAuth {
    config: OAuth2Config,
}

impl GitHubAuth {
    /// Create a new GitHub authentication provider with default scopes
    ///
    /// Default scopes: repo, read:user, user:email
    pub fn new(client_id: String, client_secret: String) -> Self {
        let default_scopes = vec![
            "repo".to_string(),       // Full control of private repositories
            "read:user".to_string(),  // Read user profile data
            "user:email".to_string(), // Access user email addresses
        ];
        Self::with_scopes(client_id, client_secret, default_scopes)
    }

    /// Create with custom scopes
    pub fn with_scopes(client_id: String, client_secret: String, scopes: Vec<String>) -> Self {
        Self {
            config: OAuth2Config {
                client_id,
                client_secret,
                auth_url: GITHUB_AUTH_URL.to_string(),
                token_url: GITHUB_TOKEN_URL.to_string(),
                redirect_uri: DEFAULT_REDIRECT_URI.to_string(),
                scopes,
            },
        }
    }
}

impl OAuth2Provider for GitHubAuth {
    fn service_id(&self) -> &str {
        "github"
    }

    fn config(&self) -> &OAuth2Config {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn test_github_auth_creation() {
        let auth = GitHubAuth::new(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
        );

        assert_eq!(auth.service_id(), "github");
        assert_eq!(auth.config().scopes.len(), 3);
        assert!(auth.config().scopes.contains(&"repo".to_string()));
    }

    #[test]
    fn test_github_auth_custom_scopes() {
        let custom_scopes = vec!["public_repo".to_string()];
        let auth = GitHubAuth::with_scopes(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
            custom_scopes.clone(),
        );

        assert_eq!(auth.config().scopes, custom_scopes);
    }
}
