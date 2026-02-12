use crate::services::workflow_service::WorkflowError;
use myme_core::{AppError, GitHubError};

impl From<WorkflowError> for AppError {
    fn from(e: WorkflowError) -> Self {
        match e {
            WorkflowError::Network(s) => AppError::GitHub(GitHubError::message(s)),
            WorkflowError::NotInitialized => {
                AppError::Service("Workflow service not initialized".into())
            }
        }
    }
}
