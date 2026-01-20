use crate::oauth::{OAuth2Config, OAuth2Provider};

/// GitHub OAuth2 authentication provider
pub struct GitHubAuth {
    config: OAuth2Config,
}

impl GitHubAuth {
    /// Create a new GitHub authentication provider
    ///
    /// # Arguments
    /// * `client_id` - GitHub OAuth App client ID
    /// * `client_secret` - GitHub OAuth App client secret
    pub fn new(client_id: String, client_secret: String) -> Self {
        let config = OAuth2Config {
            client_id,
            client_secret,
            auth_url: "https://github.com/login/oauth/authorize".to_string(),
            token_url: "https://github.com/login/oauth/access_token".to_string(),
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scopes: vec![
                "repo".to_string(),       // Full control of private repositories
                "read:user".to_string(),  // Read user profile data
                "user:email".to_string(), // Access user email addresses
            ],
        };

        Self { config }
    }

    /// Create with custom scopes
    ///
    /// # Arguments
    /// * `client_id` - GitHub OAuth App client ID
    /// * `client_secret` - GitHub OAuth App client secret
    /// * `scopes` - Custom scopes to request
    pub fn with_scopes(client_id: String, client_secret: String, scopes: Vec<String>) -> Self {
        let config = OAuth2Config {
            client_id,
            client_secret,
            auth_url: "https://github.com/login/oauth/authorize".to_string(),
            token_url: "https://github.com/login/oauth/access_token".to_string(),
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scopes,
        };

        Self { config }
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
