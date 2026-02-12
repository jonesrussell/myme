use anyhow::Result;
use std::sync::Arc;

use crate::Config;

/// Main application state and lifecycle manager
pub struct App {
    config: Arc<Config>,
}

impl App {
    /// Create a new application instance
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let config = Arc::new(config);

        Ok(Self { config })
    }

    /// Initialize the application
    pub fn initialize(&mut self) -> Result<()> {
        tracing::info!("Application initialized successfully");
        Ok(())
    }

    /// Shutdown the application
    pub fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down application");
        Ok(())
    }

    /// Get reference to application config
    pub fn config(&self) -> &Config {
        &self.config
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new().expect("Failed to create default App")
    }
}
