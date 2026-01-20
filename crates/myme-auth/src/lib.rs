pub mod storage;
pub mod oauth;
pub mod github;

pub use storage::{SecureStorage, TokenSet};
pub use oauth::{OAuth2Provider, OAuth2Config};
pub use github::GitHubAuth;

use anyhow::Result;

/// Initialize the auth module
pub fn init() -> Result<()> {
    tracing::info!("MyMe auth initialized");
    Ok(())
}
