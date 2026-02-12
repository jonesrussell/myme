pub mod app;
pub mod config;
pub mod error;

pub use app::App;
pub use config::{Config, GitHubConfig, NotesConfig, TemperatureUnit, WeatherConfig};
pub use error::{
    AppError, AuthError, ConfigError, DatabaseError, GitHubError, NetworkError, WeatherError,
};

use anyhow::Result;

/// Initialize the core application
pub fn init() -> Result<()> {
    // Initialize tracing/logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("MyMe core initialized");
    Ok(())
}
