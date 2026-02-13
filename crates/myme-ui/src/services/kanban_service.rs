//! Kanban backend: async GitHub operations for tasks.
//! All network work runs off the UI thread; results sent via mpsc.

use std::sync::Arc;

use myme_services::{CreateIssueRequest, GitHubClient, UpdateIssueRequest};

use crate::bridge;

/// Error type for kanban operations
#[derive(Debug, Clone)]
pub enum KanbanError {
    Network(String),
    NotInitialized,
}

impl std::fmt::Display for KanbanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KanbanError::Network(s) => write!(f, "Kanban error: {}", s),
            KanbanError::NotInitialized => write!(f, "Kanban service not initialized"),
        }
    }
}

impl std::error::Error for KanbanError {}

/// GitHub issue info (subset of what we need)
#[derive(Debug, Clone)]
pub struct IssueResult {
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
pub enum KanbanServiceMessage {
    /// Result of updating an issue (move_task, update_task)
    UpdateIssueDone { index: i32, result: Result<IssueResult, KanbanError> },
    /// Result of creating an issue
    CreateIssueDone(Result<IssueResult, KanbanError>),
    /// Result of syncing one repo (fetching issues)
    SyncDone { repo_id: String, result: Result<Vec<IssueResult>, KanbanError> },
}

/// Request to update an issue asynchronously.
pub fn request_update_issue(
    tx: &std::sync::mpsc::Sender<KanbanServiceMessage>,
    client: Arc<GitHubClient>,
    index: i32,
    owner: String,
    repo: String,
    issue_number: i32,
    update_req: UpdateIssueRequest,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(KanbanServiceMessage::UpdateIssueDone {
                index,
                result: Err(KanbanError::NotInitialized),
            });
            return;
        }
    };

    runtime.spawn(async move {
        let result = client
            .update_issue(&owner, &repo, issue_number, update_req)
            .await
            .map(|issue| IssueResult {
                number: issue.number,
                title: issue.title,
                body: issue.body,
                state: issue.state,
                labels: issue.labels.into_iter().map(|l| l.name).collect(),
                html_url: issue.html_url,
                created_at: issue.created_at,
                updated_at: issue.updated_at,
            })
            .map_err(|e| KanbanError::Network(e.to_string()));
        let _ = tx.send(KanbanServiceMessage::UpdateIssueDone { index, result });
    });
}

/// Request to create an issue asynchronously.
pub fn request_create_issue(
    tx: &std::sync::mpsc::Sender<KanbanServiceMessage>,
    client: Arc<GitHubClient>,
    owner: String,
    repo: String,
    create_req: CreateIssueRequest,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ =
                tx.send(KanbanServiceMessage::CreateIssueDone(Err(KanbanError::NotInitialized)));
            return;
        }
    };

    runtime.spawn(async move {
        let result = client
            .create_issue(&owner, &repo, create_req)
            .await
            .map(|issue| IssueResult {
                number: issue.number,
                title: issue.title,
                body: issue.body,
                state: issue.state,
                labels: issue.labels.into_iter().map(|l| l.name).collect(),
                html_url: issue.html_url,
                created_at: issue.created_at,
                updated_at: issue.updated_at,
            })
            .map_err(|e| KanbanError::Network(e.to_string()));
        let _ = tx.send(KanbanServiceMessage::CreateIssueDone(result));
    });
}

/// Request to sync one repo (fetch all issues) asynchronously.
pub fn request_sync(
    tx: &std::sync::mpsc::Sender<KanbanServiceMessage>,
    client: Arc<GitHubClient>,
    repo_id: String,
    owner: String,
    repo: String,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(KanbanServiceMessage::SyncDone {
                repo_id,
                result: Err(KanbanError::NotInitialized),
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
                    .map(|issue| IssueResult {
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
            .map_err(|e| KanbanError::Network(e.to_string()));
        let _ = tx.send(KanbanServiceMessage::SyncDone { repo_id, result });
    });
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn kanban_error_display() {
        assert!(format!("{}", KanbanError::Network("timeout".into())).contains("Kanban"));
        assert!(format!("{}", KanbanError::NotInitialized).contains("not initialized"));
    }

    #[test]
    fn kanban_service_message_variants() {
        let _sync_err: KanbanServiceMessage = KanbanServiceMessage::SyncDone {
            repo_id: "owner/repo".into(),
            result: Err(KanbanError::NotInitialized),
        };
    }
}
