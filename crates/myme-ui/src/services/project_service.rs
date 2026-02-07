//! Project backend: async GitHub operations for projects.
//! All network work runs off the UI thread; results sent via mpsc.

use std::sync::Arc;

use myme_services::GitHubClient;

use crate::bridge;

/// Error type for project operations
#[derive(Debug, Clone)]
pub enum ProjectError {
    Network(String),
    NotInitialized,
}

impl std::fmt::Display for ProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectError::Network(s) => write!(f, "Project error: {}", s),
            ProjectError::NotInitialized => write!(f, "Project service not initialized"),
        }
    }
}

impl std::error::Error for ProjectError {}

/// GitHub repository info (subset of what we need)
#[derive(Debug, Clone)]
pub struct RepoInfo {
    pub full_name: String,
    pub description: Option<String>,
}

/// Messages sent from async operations back to the UI thread
#[derive(Debug)]
pub enum ProjectServiceMessage {
    /// Result of fetching repo info from GitHub (when adding repo to project)
    FetchRepoDone(Result<RepoInfo, ProjectError>),
}

/// Request to fetch repo info asynchronously.
/// Sends `FetchRepoDone` on the channel when complete.
pub fn request_fetch_repo(
    tx: &std::sync::mpsc::Sender<ProjectServiceMessage>,
    client: Arc<GitHubClient>,
    owner: String,
    repo: String,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(ProjectServiceMessage::FetchRepoDone(Err(
                ProjectError::NotInitialized,
            )));
            return;
        }
    };

    runtime.spawn(async move {
        let result = client
            .get_repo(&owner, &repo)
            .await
            .map(|repo| RepoInfo {
                full_name: repo.full_name,
                description: repo.description,
            })
            .map_err(|e| ProjectError::Network(e.to_string()));
        let _ = tx.send(ProjectServiceMessage::FetchRepoDone(result));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_error_display() {
        assert!(format!("{}", ProjectError::Network("timeout".into())).contains("Project"));
        assert!(format!("{}", ProjectError::NotInitialized).contains("not initialized"));
    }

    #[test]
    fn project_service_message_variants() {
        let _fetch_err: ProjectServiceMessage =
            ProjectServiceMessage::FetchRepoDone(Err(ProjectError::NotInitialized));
    }
}
