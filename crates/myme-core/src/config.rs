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

    /// Weather settings
    #[serde(default)]
    pub weather: WeatherConfig,

    /// Projects settings
    #[serde(default)]
    pub projects: ProjectsConfig,

    /// Repos settings (local search path for git repositories)
    #[serde(default)]
    pub repos: ReposConfig,

    /// GitHub OAuth settings
    #[serde(default)]
    pub github: GitHubConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// URL for the Godo API
    pub todo_api_url: String,

    /// JWT token for Godo API authentication (optional, can be set via environment)
    pub jwt_token: Option<String>,
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

/// Temperature unit preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TemperatureUnit {
    #[default]
    Auto,
    Celsius,
    Fahrenheit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherConfig {
    /// Temperature unit preference
    pub temperature_unit: TemperatureUnit,

    /// Refresh interval in minutes
    pub refresh_minutes: u32,
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            temperature_unit: TemperatureUnit::Auto,
            refresh_minutes: 15,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectsConfig {
    /// Sync interval in minutes (default: 5)
    #[serde(default = "default_sync_interval")]
    pub sync_interval_minutes: u32,
    /// Auto-create status labels on repos (default: true)
    #[serde(default = "default_auto_create_labels")]
    pub auto_create_labels: bool,
}

fn default_sync_interval() -> u32 {
    5
}

fn default_auto_create_labels() -> bool {
    true
}

impl Default for ProjectsConfig {
    fn default() -> Self {
        Self {
            sync_interval_minutes: default_sync_interval(),
            auto_create_labels: default_auto_create_labels(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReposConfig {
    /// Base directory to search for local git repositories (e.g. ~/dev)
    #[serde(default = "default_repos_local_search_path_str")]
    pub local_search_path: String,
}

fn default_repos_local_search_path_str() -> String {
    default_repos_local_search_path()
        .to_string_lossy()
        .into_owned()
}

fn default_repos_local_search_path() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join("dev"))
        .unwrap_or_else(|| PathBuf::from("."))
}

impl Default for ReposConfig {
    fn default() -> Self {
        Self {
            local_search_path: default_repos_local_search_path_str(),
        }
    }
}

impl ReposConfig {
    /// Returns (effective_path, config_path_invalid).
    /// effective_path: valid directory to use (config path if valid, else fallback).
    /// config_path_invalid: true if configured path was invalid (missing or not a directory).
    pub fn effective_local_search_path(&self) -> (PathBuf, bool) {
        let configured = PathBuf::from(&self.local_search_path);
        let invalid = !configured.exists() || !configured.is_dir();
        if invalid {
            let fallback = default_repos_local_search_path();
            let fallback_valid = fallback.exists() && fallback.is_dir();
            let effective = if fallback_valid {
                fallback
            } else {
                PathBuf::from(".")
            };
            (effective, true)
        } else {
            (configured, false)
        }
    }
}

/// GitHub OAuth configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// GitHub OAuth App Client ID
    /// Create at: https://github.com/settings/developers
    pub client_id: String,
    /// GitHub OAuth App Client Secret
    pub client_secret: String,
}

impl GitHubConfig {
    /// Check if credentials are configured (not placeholders)
    pub fn is_configured(&self) -> bool {
        !self.client_id.is_empty()
            && !self.client_secret.is_empty()
            && !self.client_id.starts_with("YOUR_")
            && !self.client_secret.starts_with("YOUR_")
    }
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            client_id: "YOUR_GITHUB_CLIENT_ID".to_string(),
            client_secret: "YOUR_GITHUB_CLIENT_SECRET".to_string(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("myme");

        Self {
            config_dir,
            services: ServiceConfig {
                todo_api_url: "http://localhost:8008".to_string(), // Godo default port
                jwt_token: std::env::var("GODO_JWT_TOKEN").ok(),   // Read from environment
            },
            ui: UiConfig {
                window_width: 1200,
                window_height: 800,
                dark_mode: false,
            },
            weather: WeatherConfig::default(),
            projects: ProjectsConfig::default(),
            repos: ReposConfig::default(),
            github: GitHubConfig::default(),
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

        let contents =
            std::fs::read_to_string(&config_path).context("Failed to read config file")?;

        let config: Config = toml::from_str(&contents).context("Failed to parse config file")?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let contents = toml::to_string_pretty(self).context("Failed to serialize config")?;

        std::fs::write(&config_path, contents).context("Failed to write config file")?;

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
