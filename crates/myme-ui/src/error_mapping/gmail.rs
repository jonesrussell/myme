use crate::services::gmail_service::GmailError;
use myme_core::{AppError, AuthError, NetworkError};

impl From<GmailError> for AppError {
    fn from(e: GmailError) -> Self {
        match e {
            GmailError::Network(s) => {
                AppError::Network(NetworkError::ConnectionFailed(s))
            }
            GmailError::Auth(s) => AppError::Auth(AuthError::OAuthFailed(s)),
            GmailError::NotInitialized => {
                AppError::Service("Gmail service not initialized".into())
            }
        }
    }
}
