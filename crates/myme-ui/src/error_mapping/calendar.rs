use crate::services::calendar_service::CalendarError;
use myme_core::{AppError, AuthError, NetworkError};

impl From<CalendarError> for AppError {
    fn from(e: CalendarError) -> Self {
        match e {
            CalendarError::Network(s) => AppError::Network(NetworkError::ConnectionFailed(s)),
            CalendarError::Auth(s) => AppError::Auth(AuthError::OAuthFailed(s)),
            CalendarError::NotInitialized => {
                AppError::Service("Calendar service not initialized".into())
            }
        }
    }
}
