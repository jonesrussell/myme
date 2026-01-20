use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Application configuration directory
    pub config_dir: PathBuf,

    /// Service configurations
    pub services: ServiceConfig,

    /// UI preferences
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// URL for the Golang todo API
    pub todo_api_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Window width
    pub window_width: u32,

    /// Window height
    pub window_height: u32,

    /// Dark mode enabled
    pub dark_mode: bool,
}

impl Default for Config {
    fn default() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("myme");

        Self {
            config_dir,
            services: ServiceConfig {
                todo_api_url: "http://localhost:8080".to_string(),
            },
            ui: UiConfig {
                window_width: 1200,
                window_height: 800,
                dark_mode: false,
            },
        }
    }
}

impl Config {
    /// Load configuration from file, creating default if it doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let contents = std::fs::read_to_string(&config_path)
            .context("Failed to read config file")?;

        let config: Config = toml::from_str(&contents)
            .context("Failed to parse config file")?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        std::fs::write(&config_path, contents)
            .context("Failed to write config file")?;

        Ok(())
    }

    /// Get the path to the configuration file
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("myme");

        Ok(config_dir.join("config.toml"))
    }
}
