use crate::services::repo_service::RepoError;
use myme_core::{AppError, ConfigError, GitHubError};
use std::io;

impl From<RepoError> for AppError {
    fn from(e: RepoError) -> Self {
        match e {
            RepoError::Git(s) => AppError::Service(format!("Git operation failed: {}", s)),
            RepoError::GitHub(s) => AppError::GitHub(GitHubError::message(s)),
            RepoError::Io(s) => AppError::Io(io::Error::new(io::ErrorKind::Other, s)),
            RepoError::Config(s) => AppError::Config(ConfigError::Invalid(s)),
            RepoError::Cancelled => AppError::Service("Operation cancelled".into()),
        }
    }
}
