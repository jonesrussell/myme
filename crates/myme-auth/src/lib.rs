pub mod github;
pub mod oauth;
pub mod storage;

pub use github::GitHubAuth;
pub use oauth::{OAuth2Config, OAuth2Provider};
pub use storage::{SecureStorage, TokenSet};

use anyhow::Result;

/// Initialize the auth module
pub fn init() -> Result<()> {
    tracing::info!("MyMe auth initialized");
    Ok(())
}
