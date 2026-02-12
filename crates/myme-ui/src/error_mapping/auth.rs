use crate::services::auth_service::AuthError as UiAuthError;
use myme_core::{AppError, AuthError, ConfigError};

impl From<UiAuthError> for AppError {
    fn from(e: UiAuthError) -> Self {
        match e {
            UiAuthError::OAuth(s) => AppError::Auth(AuthError::OAuthFailed(s)),
            UiAuthError::NotConfigured => AppError::Config(ConfigError::MissingSetting(
                "GitHub OAuth (client_id, client_secret)".into(),
            )),
            UiAuthError::NotInitialized => AppError::Service("Auth service not initialized".into()),
        }
    }
}
