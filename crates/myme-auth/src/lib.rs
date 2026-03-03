pub mod github;
pub mod google;
pub mod google_api_client;
pub mod oauth;
pub mod storage;

pub use github::GitHubAuth;
pub use google::{
    GoogleOAuth2Provider, GoogleTokenResponse, GoogleUserInfo, DEFAULT_GOOGLE_CLIENT_ID,
    DEFAULT_GOOGLE_CLIENT_SECRET,
};
pub use google_api_client::{GoogleApiClient, GoogleApiError};
pub use oauth::{OAuth2Config, OAuth2Provider};
pub use storage::{SecureStorage, TokenSet};

use anyhow::Result;

/// Initialize the auth module
pub fn init() -> Result<()> {
    tracing::info!("MyMe auth initialized");
    Ok(())
}
