//! Gmail-specific error types.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GmailError {
    #[error("Authentication required")]
    AuthRequired,

    #[error("Token expired")]
    TokenExpired,

    #[error("Rate limited, retry after {0} seconds")]
    RateLimited(u64),

    #[error("Message not found: {0}")]
    MessageNotFound(String),

    #[error("Label not found: {0}")]
    LabelNotFound(String),

    #[error("Invalid message format")]
    InvalidMessageFormat,

    #[error("Send failed: {0}")]
    SendFailed(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
}

impl GmailError {
    /// User-friendly error message for UI display.
    pub fn user_message(&self) -> String {
        match self {
            Self::AuthRequired => "Please sign in to your Google account".to_string(),
            Self::TokenExpired => "Your session has expired. Please sign in again.".to_string(),
            Self::RateLimited(secs) => format!("Too many requests. Please wait {} seconds.", secs),
            Self::MessageNotFound(_) => "Message not found".to_string(),
            Self::LabelNotFound(_) => "Label not found".to_string(),
            Self::InvalidMessageFormat => "Invalid email format".to_string(),
            Self::SendFailed(msg) => format!("Failed to send email: {}", msg),
            Self::ApiError(msg) => format!("Gmail error: {}", msg),
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
        let err = GmailError::AuthRequired;
        assert!(err.user_message().contains("sign in"));

        let err = GmailError::RateLimited(30);
        assert!(err.user_message().contains("30"));
    }

    #[test]
    fn test_should_refresh_token() {
        assert!(GmailError::TokenExpired.should_refresh_token());
        assert!(GmailError::AuthRequired.should_refresh_token());
        assert!(!GmailError::MessageNotFound("x".into()).should_refresh_token());
    }

    #[test]
    fn test_is_retryable() {
        assert!(GmailError::RateLimited(10).is_retryable());
        assert!(!GmailError::MessageNotFound("x".into()).is_retryable());
    }
}
