use anyhow::Result;
use std::sync::Arc;

use crate::{Config, PluginContext, PluginProvider};

/// Main application state and lifecycle manager
pub struct App {
    config: Arc<Config>,
    plugins: Vec<Box<dyn PluginProvider>>,
    context: PluginContext,
}

impl App {
    /// Create a new application instance
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let config = Arc::new(config);
        let context = PluginContext::new(config.clone());

        Ok(Self {
            config,
            plugins: Vec::new(),
            context,
        })
    }

    /// Register a plugin with the application
    pub fn register_plugin(&mut self, plugin: Box<dyn PluginProvider>) {
        tracing::info!("Registering plugin: {}", plugin.name());
        self.plugins.push(plugin);
    }

    /// Initialize all registered plugins
    pub fn initialize(&mut self) -> Result<()> {
        tracing::info!(
            "Initializing application with {} plugins",
            self.plugins.len()
        );

        for plugin in &mut self.plugins {
            tracing::debug!("Initializing plugin: {}", plugin.name());
            plugin.initialize(&self.context)?;
        }

        tracing::info!("Application initialized successfully");
        Ok(())
    }

    /// Shutdown the application and all plugins
    pub fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down application");

        for plugin in &mut self.plugins {
            tracing::debug!("Shutting down plugin: {}", plugin.name());
            if let Err(e) = plugin.shutdown() {
                tracing::error!("Error shutting down plugin {}: {}", plugin.name(), e);
            }
        }

        Ok(())
    }

    /// Get reference to application config
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get list of all registered plugins
    pub fn plugins(&self) -> &[Box<dyn PluginProvider>] {
        &self.plugins
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new().expect("Failed to create default App")
    }
}
