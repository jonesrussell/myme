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

/// GitHub issue info (subset of what we need)
#[derive(Debug, Clone)]
pub struct IssueInfo {
    pub number: i32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub labels: Vec<String>,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Messages sent from async operations back to the UI thread
#[derive(Debug)]
pub enum ProjectServiceMessage {
    /// Result of fetching repo info from GitHub
    FetchRepoDone(Result<RepoInfo, ProjectError>),
    /// Result of fetching issues from GitHub
    FetchIssuesDone {
        project_id: String,
        result: Result<Vec<IssueInfo>, ProjectError>,
    },
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

/// Request to fetch issues asynchronously.
/// Sends `FetchIssuesDone` on the channel when complete.
pub fn request_fetch_issues(
    tx: &std::sync::mpsc::Sender<ProjectServiceMessage>,
    client: Arc<GitHubClient>,
    project_id: String,
    owner: String,
    repo: String,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(ProjectServiceMessage::FetchIssuesDone {
                project_id,
                result: Err(ProjectError::NotInitialized),
            });
            return;
        }
    };

    runtime.spawn(async move {
        let result = client
            .list_issues(&owner, &repo)
            .await
            .map(|issues| {
                issues
                    .into_iter()
                    .map(|issue| IssueInfo {
                        number: issue.number,
                        title: issue.title,
                        body: issue.body,
                        state: issue.state,
                        labels: issue.labels.into_iter().map(|l| l.name).collect(),
                        html_url: issue.html_url,
                        created_at: issue.created_at,
                        updated_at: issue.updated_at,
                    })
                    .collect()
            })
            .map_err(|e| ProjectError::Network(e.to_string()));
        let _ = tx.send(ProjectServiceMessage::FetchIssuesDone { project_id, result });
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
