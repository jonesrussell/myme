use anyhow::Result;
use std::sync::Arc;

use crate::Config;

/// Plugin provider trait for extending application functionality
pub trait PluginProvider: Send + Sync {
    /// Unique identifier for this plugin
    fn id(&self) -> &str;

    /// Human-readable name
    fn name(&self) -> &str;

    /// Initialize the plugin with the given context
    fn initialize(&mut self, ctx: &PluginContext) -> Result<()>;

    /// Shutdown the plugin gracefully
    fn shutdown(&mut self) -> Result<()>;

    /// Get UI components provided by this plugin
    fn ui_components(&self) -> Vec<UiComponent>;
}

/// Context provided to plugins during initialization
pub struct PluginContext {
    pub config: Arc<Config>,
}

impl PluginContext {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
}

/// UI component types that plugins can provide
#[derive(Debug, Clone)]
pub enum UiComponent {
    /// A full page in the main navigation
    Page {
        qml_path: String,
        title: String,
        icon: String,
    },
    /// A widget that can be embedded
    Widget { qml_path: String },
}
