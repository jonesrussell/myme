use crate::services::kanban_service::KanbanError;
use myme_core::{AppError, GitHubError};

impl From<KanbanError> for AppError {
    fn from(e: KanbanError) -> Self {
        match e {
            KanbanError::Network(s) => AppError::GitHub(GitHubError::message(s)),
            KanbanError::NotInitialized => {
                AppError::Service("Kanban service not initialized".into())
            }
        }
    }
}
