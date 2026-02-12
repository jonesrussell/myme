//! Workflow backend: async fetch of GitHub Actions workflows for project-linked repos.
//! All network work runs off the UI thread; results sent via mpsc.

use std::sync::Arc;

use myme_services::{GitHubClient, GitHubWorkflow};

use crate::bridge;

/// Error type for workflow operations
#[derive(Debug, Clone)]
pub enum WorkflowError {
    Network(String),
    NotInitialized,
}

impl std::fmt::Display for WorkflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowError::Network(s) => write!(f, "Workflow error: {}", s),
            WorkflowError::NotInitialized => write!(f, "Workflow service not initialized"),
        }
    }
}

impl std::error::Error for WorkflowError {}

/// Workflows for a single repo (owner/repo)
#[derive(Debug, Clone)]
pub struct RepoWorkflows {
    pub repo_id: String,
    pub workflows: Vec<GitHubWorkflow>,
}

/// Messages sent from async operations back to the UI thread
#[derive(Debug)]
pub enum WorkflowServiceMessage {
    /// Result of fetching workflows for all linked repos
    FetchWorkflowsDone(Result<Vec<RepoWorkflows>, WorkflowError>),
}

/// Request to fetch workflows for the given repo_ids (owner/repo format).
/// Sorts repo_ids before fetching. Sends `FetchWorkflowsDone` on the channel when complete.
pub fn request_fetch_workflows(
    tx: &std::sync::mpsc::Sender<WorkflowServiceMessage>,
    client: Arc<GitHubClient>,
    mut repo_ids: Vec<String>,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(WorkflowServiceMessage::FetchWorkflowsDone(Err(
                WorkflowError::NotInitialized,
            )));
            return;
        }
    };

    repo_ids.sort();

    runtime.spawn(async move {
        let mut results = Vec::with_capacity(repo_ids.len());
        for repo_id in repo_ids {
            let parts: Vec<&str> = repo_id.splitn(2, '/').collect();
            let (owner, repo) = match parts.as_slice() {
                [o, r] => (*o, *r),
                _ => {
                    results.push(RepoWorkflows {
                        repo_id,
                        workflows: vec![],
                    });
                    continue;
                }
            };
            match client.list_workflows(owner, repo).await {
                Ok(workflows) => {
                    results.push(RepoWorkflows { repo_id, workflows });
                }
                Err(e) => {
                    let _ = tx.send(WorkflowServiceMessage::FetchWorkflowsDone(Err(
                        WorkflowError::Network(e.to_string()),
                    )));
                    return;
                }
            }
        }
        let _ = tx.send(WorkflowServiceMessage::FetchWorkflowsDone(Ok(results)));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflow_error_display() {
        assert!(format!("{}", WorkflowError::Network("timeout".into())).contains("Workflow"));
        assert!(format!("{}", WorkflowError::NotInitialized).contains("not initialized"));
    }
}
