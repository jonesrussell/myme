use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

/// Configuration validation errors
#[derive(Debug, Clone)]
pub struct ConfigValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Result of config validation
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    pub errors: Vec<ConfigValidationError>,
    pub warnings: Vec<ConfigValidationError>,
}

impl ValidationResult {
    /// Returns true if there are no errors (warnings are OK)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Add an error
    pub fn add_error(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.errors.push(ConfigValidationError { field: field.into(), message: message.into() });
    }

    /// Add a warning
    pub fn add_warning(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.warnings.push(ConfigValidationError { field: field.into(), message: message.into() });
    }

    /// Get a user-friendly message summarizing all errors
    pub fn error_summary(&self) -> String {
        if self.errors.is_empty() {
            return String::new();
        }
        self.errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; ")
    }
}

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

    /// Google OAuth settings
    #[serde(default)]
    pub google: Option<GoogleConfig>,

    /// Notes storage settings
    #[serde(default)]
    pub notes: NotesConfig,
}

/// Service-related config. Reserved for future use.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceConfig {}

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
        Self { temperature_unit: TemperatureUnit::Auto, refresh_minutes: 15 }
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
    default_repos_local_search_path().to_string_lossy().into_owned()
}

fn default_repos_local_search_path() -> PathBuf {
    dirs::home_dir().map(|h| h.join("dev")).unwrap_or_else(|| PathBuf::from("."))
}

impl Default for ReposConfig {
    fn default() -> Self {
        Self { local_search_path: default_repos_local_search_path_str() }
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
            let effective = if fallback_valid { fallback } else { PathBuf::from(".") };
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

/// Google OAuth configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GoogleConfig {
    /// Google OAuth Client ID
    /// Create at: https://console.cloud.google.com/apis/credentials
    pub client_id: Option<String>,
    /// Google OAuth Client Secret
    pub client_secret: Option<String>,
}

impl GoogleConfig {
    /// Check if credentials are configured
    pub fn is_configured(&self) -> bool {
        self.client_id.as_ref().map(|s| !s.is_empty()).unwrap_or(false)
            && self.client_secret.as_ref().map(|s| !s.is_empty()).unwrap_or(false)
    }
}

/// Notes storage configuration (SQLite only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotesConfig {
    /// Path to SQLite database file
    /// Default: ~/.config/myme/notes.db
    #[serde(default = "default_notes_sqlite_path")]
    pub sqlite_path: String,
}

fn default_notes_sqlite_path() -> String {
    dirs::config_dir()
        .map(|d| d.join("myme").join("notes.db"))
        .unwrap_or_else(|| PathBuf::from("notes.db"))
        .to_string_lossy()
        .into_owned()
}

impl Default for NotesConfig {
    fn default() -> Self {
        Self { sqlite_path: default_notes_sqlite_path() }
    }
}

impl NotesConfig {
    /// Get the effective SQLite path (expanded)
    pub fn sqlite_path(&self) -> PathBuf {
        expand_path(&self.sqlite_path)
    }
}

/// Expand ~ in paths to home directory
fn expand_path(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }
    PathBuf::from(path)
}

impl Default for Config {
    fn default() -> Self {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("myme");

        Self {
            config_dir,
            services: ServiceConfig::default(),
            ui: UiConfig { window_width: 1200, window_height: 800, dark_mode: false },
            weather: WeatherConfig::default(),
            projects: ProjectsConfig::default(),
            repos: ReposConfig::default(),
            github: GitHubConfig::default(),
            google: Some(GoogleConfig::default()),
            notes: NotesConfig::default(),
        }
    }
}

/// Cached config for hot paths. Initialized on first access.
static CACHED_CONFIG: OnceLock<Arc<Config>> = OnceLock::new();

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

    /// Load configuration with caching for hot paths.
    /// Uses process-lifecycle cache; for fresh config after settings change, use `load()`.
    pub fn load_cached() -> Arc<Self> {
        CACHED_CONFIG
            .get_or_init(|| {
                Arc::new(Self::load().unwrap_or_else(|e| {
                    tracing::warn!("Config load failed, using default: {}", e);
                    Self::default()
                }))
            })
            .clone()
    }

    /// Load configuration and validate it
    ///
    /// Returns the config along with any validation warnings.
    /// Returns an error if validation fails with critical errors.
    pub fn load_validated() -> Result<(Self, ValidationResult)> {
        let config = Self::load()?;
        let validation = config.validate();

        if !validation.is_valid() {
            anyhow::bail!("Configuration validation failed: {}", validation.error_summary());
        }

        if !validation.warnings.is_empty() {
            for warning in &validation.warnings {
                tracing::warn!("Config warning: {}", warning);
            }
        }

        Ok((config, validation))
    }

    /// Validate the configuration
    ///
    /// Returns a ValidationResult containing any errors or warnings.
    pub fn validate(&self) -> ValidationResult {
        let mut result = ValidationResult::default();

        // Validate window dimensions
        if self.ui.window_width == 0 {
            result.add_error("ui.window_width", "Window width must be greater than 0");
        } else if self.ui.window_width > 10000 {
            result.add_warning("ui.window_width", "Window width is unusually large (>10000)");
        }

        if self.ui.window_height == 0 {
            result.add_error("ui.window_height", "Window height must be greater than 0");
        } else if self.ui.window_height > 10000 {
            result.add_warning("ui.window_height", "Window height is unusually large (>10000)");
        }

        // Validate weather refresh interval
        if self.weather.refresh_minutes == 0 {
            result.add_warning("weather.refresh_minutes", "Weather refresh disabled (0 minutes)");
        } else if self.weather.refresh_minutes > 1440 {
            result.add_warning(
                "weather.refresh_minutes",
                "Weather refresh interval is more than 24 hours",
            );
        }

        // Validate projects sync interval
        if self.projects.sync_interval_minutes == 0 {
            result
                .add_warning("projects.sync_interval_minutes", "Project sync disabled (0 minutes)");
        }

        // Validate repos path
        let repos_path = PathBuf::from(&self.repos.local_search_path);
        if !repos_path.exists() {
            result.add_warning(
                "repos.local_search_path",
                format!("Path does not exist: {}", repos_path.display()),
            );
        } else if !repos_path.is_dir() {
            result.add_error(
                "repos.local_search_path",
                format!("Path is not a directory: {}", repos_path.display()),
            );
        }

        // Validate GitHub config (just warn if not configured)
        if !self.github.is_configured() {
            result.add_warning(
                "github",
                "GitHub OAuth not configured - some features will be unavailable",
            );
        }

        result
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
        let config_dir = dirs::config_dir().context("Failed to get config directory")?.join("myme");

        Ok(config_dir.join("config.toml"))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn test_valid_default_config() {
        let config = Config::default();
        let result = config.validate();
        // Default config should be valid (only warnings, no errors)
        assert!(result.is_valid(), "Default config should be valid: {:?}", result.errors);
    }

    #[test]
    fn test_zero_window_dimensions() {
        let mut config = Config::default();
        config.ui.window_width = 0;
        let result = config.validate();
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.field == "ui.window_width"));
    }

    #[test]
    fn test_github_not_configured_is_warning() {
        let config = Config::default();
        let result = config.validate();
        // GitHub not configured should be a warning, not an error
        assert!(result.is_valid());
        assert!(result.warnings.iter().any(|w| w.field == "github"));
    }

    #[test]
    fn test_validation_result_error_summary() {
        let mut result = ValidationResult::default();
        result.add_error("field1", "error1");
        result.add_error("field2", "error2");
        let summary = result.error_summary();
        assert!(summary.contains("field1"));
        assert!(summary.contains("field2"));
    }
}
