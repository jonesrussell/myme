pub mod app;
pub mod config;
pub mod plugin;

pub use app::App;
pub use config::{Config, GitHubConfig, TemperatureUnit, WeatherConfig};
pub use plugin::{PluginContext, PluginProvider, UiComponent};

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
