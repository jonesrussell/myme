//! Calendar-specific error types.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CalendarError {
    #[error("Authentication required")]
    AuthRequired,

    #[error("Token expired")]
    TokenExpired,

    #[error("Rate limited, retry after {0} seconds")]
    RateLimited(u64),

    #[error("Event not found: {0}")]
    EventNotFound(String),

    #[error("Calendar not found: {0}")]
    CalendarNotFound(String),

    #[error("Invalid event data: {0}")]
    InvalidEventData(String),

    #[error("Conflict: event was modified")]
    Conflict,

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
}

impl CalendarError {
    /// User-friendly error message for UI display.
    pub fn user_message(&self) -> String {
        match self {
            Self::AuthRequired => "Please sign in to your Google account".to_string(),
            Self::TokenExpired => "Your session has expired. Please sign in again.".to_string(),
            Self::RateLimited(secs) => format!("Too many requests. Please wait {} seconds.", secs),
            Self::EventNotFound(_) => "Event not found".to_string(),
            Self::CalendarNotFound(_) => "Calendar not found".to_string(),
            Self::InvalidEventData(msg) => format!("Invalid event: {}", msg),
            Self::Conflict => "The event was modified elsewhere. Please refresh.".to_string(),
            Self::ApiError(msg) => format!("Calendar error: {}", msg),
            Self::CacheError(_) => "Local cache error".to_string(),
            Self::NetworkError(_) => "Network error. Check your connection.".to_string(),
        }
    }

    /// Whether this error should trigger a token refresh.
    pub fn should_refresh_token(&self) -> bool {
        matches!(self, Self::TokenExpired | Self::AuthRequired)
    }

    /// Whether this error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::RateLimited(_) | Self::NetworkError(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_user_messages() {
        let err = CalendarError::AuthRequired;
        assert!(err.user_message().contains("sign in"));

        let err = CalendarError::RateLimited(30);
        assert!(err.user_message().contains("30"));

        let err = CalendarError::Conflict;
        assert!(err.user_message().contains("modified"));
    }

    #[test]
    fn test_should_refresh_token() {
        assert!(CalendarError::TokenExpired.should_refresh_token());
        assert!(CalendarError::AuthRequired.should_refresh_token());
        assert!(!CalendarError::EventNotFound("x".into()).should_refresh_token());
    }

    #[test]
    fn test_is_retryable() {
        assert!(CalendarError::RateLimited(10).is_retryable());
        assert!(!CalendarError::EventNotFound("x".into()).is_retryable());
        assert!(!CalendarError::Conflict.is_retryable());
    }
}
