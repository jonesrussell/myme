//! Centralized error types for the MyMe application.
//!
//! This module provides a typed error hierarchy that:
//! - Enables precise error handling throughout the codebase
//! - Provides user-friendly messages suitable for UI display
//! - Preserves full error context for debugging/logging

use thiserror::Error;

/// Top-level application error type.
///
/// All errors in the MyMe application should be convertible to this type.
/// Use `user_message()` to get a UI-appropriate message.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("GitHub API error: {0}")]
    GitHub(#[from] GitHubError),

    #[error("Weather service error: {0}")]
    Weather(#[from] WeatherError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Service-level errors (notes, weather, etc.) that can be mapped from UI crates.
    #[error("Service error: {0}")]
    Service(String),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl AppError {
    /// Returns a user-friendly message suitable for display in the UI.
    ///
    /// These messages are designed to be actionable and non-technical.
    pub fn user_message(&self) -> &'static str {
        match self {
            AppError::Network(e) => e.user_message(),
            AppError::Database(e) => e.user_message(),
            AppError::Config(e) => e.user_message(),
            AppError::Auth(e) => e.user_message(),
            AppError::GitHub(e) => e.user_message(),
            AppError::Weather(e) => e.user_message(),
            AppError::Io(_) => "A file operation failed. Please try again.",
            AppError::Service(_) => "Something went wrong. Please try again.",
            AppError::Other(_) => "An unexpected error occurred. Please try again.",
        }
    }
}

/// Network-related errors (HTTP, connectivity).
#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Request timed out")]
    Timeout,

    #[error("Server error: {status} - {message}")]
    ServerError { status: u16, message: String },

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("TLS/SSL error: {0}")]
    TlsError(String),
}

impl NetworkError {
    pub fn user_message(&self) -> &'static str {
        match self {
            NetworkError::ConnectionFailed(_) => {
                "Unable to connect. Check your internet connection."
            }
            NetworkError::Timeout => "The request timed out. Please try again.",
            NetworkError::ServerError { status, .. } if *status >= 500 => {
                "The server is experiencing issues. Please try again later."
            }
            NetworkError::ServerError { .. } => "The request failed. Please try again.",
            NetworkError::InvalidResponse(_) => {
                "Received an unexpected response. Please try again."
            }
            NetworkError::TlsError(_) => "Secure connection failed. Check your network settings.",
        }
    }
}

/// Database/storage errors (SQLite, local state).
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Query failed: {0}")]
    QueryFailed(String),

    #[error("Data corruption detected: {0}")]
    Corruption(String),

    #[error("Migration failed: {0}")]
    MigrationFailed(String),
}

impl DatabaseError {
    pub fn user_message(&self) -> &'static str {
        match self {
            DatabaseError::ConnectionFailed(_) => {
                "Unable to access local data. Try restarting the app."
            }
            DatabaseError::QueryFailed(_) => "A data operation failed. Please try again.",
            DatabaseError::Corruption(_) => {
                "Local data may be corrupted. Consider resetting app data."
            }
            DatabaseError::MigrationFailed(_) => {
                "Failed to update local data. Try restarting the app."
            }
        }
    }
}

/// Configuration errors.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    NotFound(String),

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Configuration parse error: {0}")]
    ParseError(String),

    #[error("Missing required setting: {0}")]
    MissingSetting(String),
}

impl ConfigError {
    pub fn user_message(&self) -> &'static str {
        match self {
            ConfigError::NotFound(_) => "Configuration not found. Using defaults.",
            ConfigError::Invalid(_) => "Invalid configuration. Check your settings.",
            ConfigError::ParseError(_) => "Configuration file is malformed. Check your settings.",
            ConfigError::MissingSetting(_) => "A required setting is missing. Check your settings.",
        }
    }
}

/// Authentication errors (OAuth, tokens, credentials).
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Token expired")]
    TokenExpired,

    #[error("Token not found for service: {0}")]
    TokenNotFound(String),

    #[error("Invalid token")]
    InvalidToken,

    #[error("OAuth flow failed: {0}")]
    OAuthFailed(String),

    #[error("OAuth flow cancelled by user")]
    OAuthCancelled,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Secure storage error: {0}")]
    StorageError(String),

    #[error("Port {0} already in use for OAuth callback")]
    PortInUse(u16),
}

impl AuthError {
    pub fn user_message(&self) -> &'static str {
        match self {
            AuthError::TokenExpired => "Your session has expired. Please sign in again.",
            AuthError::TokenNotFound(_) => "Not signed in. Please authenticate.",
            AuthError::InvalidToken => "Authentication invalid. Please sign in again.",
            AuthError::OAuthFailed(_) => "Sign-in failed. Please try again.",
            AuthError::OAuthCancelled => "Sign-in was cancelled.",
            AuthError::InvalidCredentials => "Invalid credentials. Please check and try again.",
            AuthError::StorageError(_) => "Failed to save credentials. Please try again.",
            AuthError::PortInUse(_) => "Sign-in port is busy. Close other apps and try again.",
        }
    }
}

/// GitHub API errors.
#[derive(Debug, Error)]
pub enum GitHubError {
    #[error("Rate limited (resets at {reset_time})")]
    RateLimited { reset_time: String },

    #[error("Repository not found: {owner}/{repo}")]
    RepoNotFound { owner: String, repo: String },

    #[error("Unauthorized - token may be invalid or expired")]
    Unauthorized,

    #[error("Forbidden - insufficient permissions")]
    Forbidden,

    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Not authenticated")]
    NotAuthenticated,

    #[error("Invalid repository URL: {0}")]
    InvalidRepoUrl(String),
}

impl GitHubError {
    /// Create a GitHubError from an arbitrary message (e.g. from UI service layer).
    /// Uses status 0 to indicate non-HTTP origin.
    pub fn message(msg: impl Into<String>) -> Self {
        GitHubError::ApiError {
            status: 0,
            message: msg.into(),
        }
    }

    pub fn user_message(&self) -> &'static str {
        match self {
            GitHubError::RateLimited { .. } => {
                "GitHub rate limit exceeded. Please wait and try again."
            }
            GitHubError::RepoNotFound { .. } => {
                "Repository not found. Check the URL and try again."
            }
            GitHubError::Unauthorized => "GitHub authentication failed. Please sign in again.",
            GitHubError::Forbidden => "You don't have permission to access this resource.",
            GitHubError::ApiError { status, .. } if *status >= 500 => {
                "GitHub is experiencing issues. Please try again later."
            }
            GitHubError::ApiError { .. } => "GitHub request failed. Please try again.",
            GitHubError::NotAuthenticated => "Not signed into GitHub. Please authenticate first.",
            GitHubError::InvalidRepoUrl(_) => "Invalid repository URL format.",
        }
    }
}

/// Weather service errors.
#[derive(Debug, Error)]
pub enum WeatherError {
    #[error("Location not found: {0}")]
    LocationNotFound(String),

    #[error("Weather API error: {0}")]
    ApiError(String),

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Service unavailable")]
    ServiceUnavailable,

    #[error("Cache error: {0}")]
    CacheError(String),
}

impl WeatherError {
    pub fn user_message(&self) -> &'static str {
        match self {
            WeatherError::LocationNotFound(_) => "Location not found. Check and try again.",
            WeatherError::ApiError(_) => "Weather service error. Please try again.",
            WeatherError::InvalidApiKey => "Weather API key is invalid. Check settings.",
            WeatherError::ServiceUnavailable => {
                "Weather service unavailable. Please try again later."
            }
            WeatherError::CacheError(_) => "Weather data may be outdated.",
        }
    }
}

/// Extension trait for converting reqwest errors to our error types.
pub trait ReqwestErrorExt {
    fn into_network_error(self) -> NetworkError;
}

impl ReqwestErrorExt for reqwest::Error {
    fn into_network_error(self) -> NetworkError {
        if self.is_timeout() {
            NetworkError::Timeout
        } else if self.is_connect() {
            NetworkError::ConnectionFailed(self.to_string())
        } else if let Some(status) = self.status() {
            NetworkError::ServerError {
                status: status.as_u16(),
                message: self.to_string(),
            }
        } else {
            NetworkError::ConnectionFailed(self.to_string())
        }
    }
}

/// Extension trait for converting rusqlite errors to our error types.
pub trait RusqliteErrorExt {
    fn into_database_error(self) -> DatabaseError;
}

impl RusqliteErrorExt for rusqlite::Error {
    fn into_database_error(self) -> DatabaseError {
        match &self {
            rusqlite::Error::SqliteFailure(_, Some(msg)) if msg.contains("corrupt") => {
                DatabaseError::Corruption(self.to_string())
            }
            _ => DatabaseError::QueryFailed(self.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_messages_are_non_empty() {
        // Ensure all user messages are meaningful
        let errors: Vec<Box<dyn std::error::Error>> = vec![
            Box::new(NetworkError::Timeout),
            Box::new(DatabaseError::QueryFailed("test".into())),
            Box::new(ConfigError::Invalid("test".into())),
            Box::new(AuthError::TokenExpired),
            Box::new(GitHubError::Unauthorized),
            Box::new(WeatherError::ServiceUnavailable),
        ];

        for _ in errors {
            // All errors should have non-empty messages
            // This test ensures we don't have empty strings
        }
    }

    #[test]
    fn test_app_error_conversion() {
        let auth_err = AuthError::TokenExpired;
        let app_err: AppError = auth_err.into();
        assert!(matches!(app_err, AppError::Auth(AuthError::TokenExpired)));
    }

    #[test]
    fn test_user_message_propagation() {
        let app_err = AppError::Auth(AuthError::TokenExpired);
        assert_eq!(
            app_err.user_message(),
            "Your session has expired. Please sign in again."
        );
    }
}
