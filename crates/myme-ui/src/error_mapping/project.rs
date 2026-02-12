use crate::services::project_service::ProjectError;
use myme_core::{AppError, GitHubError};

impl From<ProjectError> for AppError {
    fn from(e: ProjectError) -> Self {
        match e {
            ProjectError::Network(s) => AppError::GitHub(GitHubError::message(s)),
            ProjectError::NotInitialized => {
                AppError::Service("Project service not initialized".into())
            }
        }
    }
}
