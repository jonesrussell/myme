// crates/myme-services/src/github.rs

use anyhow::{Context, Result};
use reqwest::{header, Client, Response};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;

use crate::retry::{is_retryable_status, with_retry, RetryConfig, RetryDecision};

const GITHUB_API_URL: &str = "https://api.github.com";

/// GitHub repository representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub html_url: String,
    /// HTTPS clone URL (e.g. https://github.com/owner/repo.git)
    #[serde(default)]
    pub clone_url: Option<String>,
    /// SSH clone URL (e.g. git@github.com:owner/repo.git)
    #[serde(default)]
    pub ssh_url: Option<String>,
    pub private: bool,
    pub default_branch: String,
    #[serde(default)]
    pub open_issues_count: i32,
    pub updated_at: String,
}

/// GitHub issue representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub id: i64,
    pub number: i32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub html_url: String,
    pub labels: Vec<GitHubLabel>,
    pub created_at: String,
    pub updated_at: String,
}

/// GitHub label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubLabel {
    pub id: i64,
    pub name: String,
    pub color: String,
}

/// Request to create a new repo
#[derive(Debug, Serialize)]
pub struct CreateRepoRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub private: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_init: Option<bool>,
}

/// Request to create a new issue
#[derive(Debug, Serialize)]
pub struct CreateIssueRequest {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
}

/// Request to update an issue
#[derive(Debug, Serialize)]
pub struct UpdateIssueRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
}

/// Request to create a label
#[derive(Debug, Serialize)]
pub struct CreateLabelRequest {
    pub name: String,
    pub color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// GitHub Actions workflow (list repository workflows response item)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubWorkflow {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub state: String,
    #[serde(default)]
    pub html_url: Option<String>,
    #[serde(default)]
    pub badge_url: Option<String>,
}

/// Response from GET /repos/{owner}/{repo}/actions/workflows
#[derive(Debug, Deserialize)]
pub struct ListWorkflowsResponse {
    pub total_count: i32,
    pub workflows: Vec<GitHubWorkflow>,
}

/// GitHub API client
#[derive(Debug, Clone)]
pub struct GitHubClient {
    base_url: Url,
    client: Arc<Client>,
    token: String,
    retry_config: RetryConfig,
}

impl GitHubClient {
    /// Create a new GitHub client with OAuth token
    pub fn new(token: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            base_url: Url::parse(GITHUB_API_URL).context("constant GITHUB_API_URL is valid")?,
            client: Arc::new(client),
            token,
            retry_config: RetryConfig::default(),
        })
    }

    /// Set custom retry configuration
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Build request with auth headers
    fn build_request(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.header(header::AUTHORIZATION, format!("Bearer {}", self.token))
            .header(header::ACCEPT, "application/vnd.github+json")
            .header(header::USER_AGENT, "myme-app")
            .header("X-GitHub-Api-Version", "2022-11-28")
    }

    /// Send a request with retry logic for transient failures.
    ///
    /// This wraps the request with exponential backoff retry for:
    /// - Timeout errors
    /// - 5xx server errors
    /// - 429 rate limit errors (GitHub has strict rate limits)
    /// - Connection resets
    ///
    /// It does NOT retry 4xx client errors (bad requests, auth failures, etc.)
    async fn send_with_retry<F>(&self, build_request: F) -> Result<Response>
    where
        F: Fn() -> reqwest::RequestBuilder,
    {
        let response = with_retry(self.retry_config.clone(), || async {
            build_request().send().await
        })
        .await
        .context("Failed to send request after retries")?;

        let status = response.status();

        // Check for non-retryable error status codes (4xx except rate limit)
        if !status.is_success() && is_retryable_status(status) == RetryDecision::NoRetry {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error ({}): {}", status, error_text);
        }

        Ok(response)
    }

    /// List repositories for authenticated user
    #[tracing::instrument(skip(self), level = "info")]
    pub async fn list_repos(&self) -> Result<Vec<GitHubRepo>> {
        tracing::debug!("Fetching user repositories");

        let url = self.base_url.join("user/repos")?;
        let response = self
            .send_with_retry(|| {
                self.build_request(
                    self.client
                        .get(url.clone())
                        .query(&[("sort", "updated"), ("per_page", "100")]),
                )
            })
            .await?;

        let repos: Vec<GitHubRepo> = response.json().await?;

        tracing::info!("Fetched {} repositories", repos.len());
        Ok(repos)
    }

    /// Get a specific repository
    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn get_repo(&self, owner: &str, repo: &str) -> Result<GitHubRepo> {
        tracing::debug!("Fetching repository {}/{}", owner, repo);

        let url = self.base_url.join(&format!("repos/{}/{}", owner, repo))?;
        let response = self
            .send_with_retry(|| self.build_request(self.client.get(url.clone())))
            .await?;

        let repo: GitHubRepo = response.json().await?;
        Ok(repo)
    }

    /// Create a new repository
    #[tracing::instrument(skip(self, req), fields(repo_name = %req.name), level = "info")]
    pub async fn create_repo(&self, req: CreateRepoRequest) -> Result<GitHubRepo> {
        tracing::debug!("Creating repository: {}", req.name);

        let url = self.base_url.join("user/repos")?;
        let request_json = serde_json::to_value(&req).context("Failed to serialize request")?;

        let response = self
            .send_with_retry(|| {
                self.build_request(self.client.post(url.clone()).json(&request_json))
            })
            .await?;

        let repo: GitHubRepo = response.json().await?;

        tracing::info!("Created repository: {}", repo.full_name);
        Ok(repo)
    }

    /// List issues for a repository
    #[tracing::instrument(skip(self), level = "info")]
    pub async fn list_issues(&self, owner: &str, repo: &str) -> Result<Vec<GitHubIssue>> {
        tracing::debug!("Fetching issues for {}/{}", owner, repo);

        let url = self
            .base_url
            .join(&format!("repos/{}/{}/issues", owner, repo))?;
        let response = self
            .send_with_retry(|| {
                self.build_request(
                    self.client
                        .get(url.clone())
                        .query(&[("state", "all"), ("per_page", "100")]),
                )
            })
            .await?;

        let issues: Vec<GitHubIssue> = response.json().await?;

        tracing::info!("Fetched {} issues for {}/{}", issues.len(), owner, repo);
        Ok(issues)
    }

    /// List issues updated since a timestamp
    pub async fn list_issues_since(
        &self,
        owner: &str,
        repo: &str,
        since: &str,
    ) -> Result<Vec<GitHubIssue>> {
        tracing::debug!("Fetching issues for {}/{} since {}", owner, repo, since);

        let url = self
            .base_url
            .join(&format!("repos/{}/{}/issues", owner, repo))?;
        let since_owned = since.to_string();
        let response = self
            .send_with_retry(|| {
                self.build_request(self.client.get(url.clone()).query(&[
                    ("state", "all"),
                    ("since", since_owned.as_str()),
                    ("per_page", "100"),
                ]))
            })
            .await?;

        let issues: Vec<GitHubIssue> = response.json().await?;

        Ok(issues)
    }

    /// Create a new issue
    #[tracing::instrument(skip(self, req), fields(title = %req.title), level = "info")]
    pub async fn create_issue(
        &self,
        owner: &str,
        repo: &str,
        req: CreateIssueRequest,
    ) -> Result<GitHubIssue> {
        tracing::debug!("Creating issue in {}/{}: {}", owner, repo, req.title);

        let url = self
            .base_url
            .join(&format!("repos/{}/{}/issues", owner, repo))?;
        let request_json = serde_json::to_value(&req).context("Failed to serialize request")?;

        let response = self
            .send_with_retry(|| {
                self.build_request(self.client.post(url.clone()).json(&request_json))
            })
            .await?;

        let issue: GitHubIssue = response.json().await?;

        tracing::info!("Created issue #{} in {}/{}", issue.number, owner, repo);
        Ok(issue)
    }

    /// Update an issue
    #[tracing::instrument(skip(self, req), level = "info")]
    pub async fn update_issue(
        &self,
        owner: &str,
        repo: &str,
        issue_number: i32,
        req: UpdateIssueRequest,
    ) -> Result<GitHubIssue> {
        tracing::debug!("Updating issue #{} in {}/{}", issue_number, owner, repo);

        let url = self
            .base_url
            .join(&format!("repos/{}/{}/issues/{}", owner, repo, issue_number))?;
        let request_json = serde_json::to_value(&req).context("Failed to serialize request")?;

        let response = self
            .send_with_retry(|| {
                self.build_request(self.client.patch(url.clone()).json(&request_json))
            })
            .await?;

        let issue: GitHubIssue = response.json().await?;

        Ok(issue)
    }

    /// Close an issue
    pub async fn close_issue(
        &self,
        owner: &str,
        repo: &str,
        issue_number: i32,
    ) -> Result<GitHubIssue> {
        self.update_issue(
            owner,
            repo,
            issue_number,
            UpdateIssueRequest {
                title: None,
                body: None,
                state: Some("closed".to_string()),
                labels: None,
            },
        )
        .await
    }

    /// Reopen an issue
    pub async fn reopen_issue(
        &self,
        owner: &str,
        repo: &str,
        issue_number: i32,
    ) -> Result<GitHubIssue> {
        self.update_issue(
            owner,
            repo,
            issue_number,
            UpdateIssueRequest {
                title: None,
                body: None,
                state: Some("open".to_string()),
                labels: None,
            },
        )
        .await
    }

    /// List labels for a repository
    pub async fn list_labels(&self, owner: &str, repo: &str) -> Result<Vec<GitHubLabel>> {
        tracing::debug!("Fetching labels for {}/{}", owner, repo);

        let url = self
            .base_url
            .join(&format!("repos/{}/{}/labels", owner, repo))?;
        let response = self
            .send_with_retry(|| self.build_request(self.client.get(url.clone())))
            .await?;

        let labels: Vec<GitHubLabel> = response.json().await?;

        Ok(labels)
    }

    /// Create a label
    pub async fn create_label(
        &self,
        owner: &str,
        repo: &str,
        req: CreateLabelRequest,
    ) -> Result<GitHubLabel> {
        tracing::debug!("Creating label {} in {}/{}", req.name, owner, repo);

        let url = self
            .base_url
            .join(&format!("repos/{}/{}/labels", owner, repo))?;
        let request_json = serde_json::to_value(&req).context("Failed to serialize request")?;

        let response = self
            .send_with_retry(|| {
                self.build_request(self.client.post(url.clone()).json(&request_json))
            })
            .await?;

        let label: GitHubLabel = response.json().await?;

        Ok(label)
    }

    /// Set labels on an issue (replaces existing)
    pub async fn set_issue_labels(
        &self,
        owner: &str,
        repo: &str,
        issue_number: i32,
        labels: Vec<String>,
    ) -> Result<Vec<GitHubLabel>> {
        tracing::debug!(
            "Setting labels on issue #{} in {}/{}",
            issue_number,
            owner,
            repo
        );

        let url = self.base_url.join(&format!(
            "repos/{}/{}/issues/{}/labels",
            owner, repo, issue_number
        ))?;

        #[derive(Serialize)]
        struct SetLabelsRequest {
            labels: Vec<String>,
        }

        let request_json = serde_json::to_value(&SetLabelsRequest { labels })
            .context("Failed to serialize request")?;

        let response = self
            .send_with_retry(|| {
                self.build_request(self.client.put(url.clone()).json(&request_json))
            })
            .await?;

        let labels: Vec<GitHubLabel> = response.json().await?;

        Ok(labels)
    }

    /// List GitHub Actions workflows for a repository
    #[tracing::instrument(skip(self), level = "info")]
    pub async fn list_workflows(&self, owner: &str, repo: &str) -> Result<Vec<GitHubWorkflow>> {
        tracing::debug!("Fetching workflows for {}/{}", owner, repo);

        let url = self
            .base_url
            .join(&format!("repos/{}/{}/actions/workflows", owner, repo))?;
        let response = self
            .send_with_retry(|| {
                self.build_request(self.client.get(url.clone()).query(&[("per_page", "100")]))
            })
            .await?;

        let body: ListWorkflowsResponse = response.json().await?;
        tracing::info!(
            "Fetched {} workflows for {}/{}",
            body.workflows.len(),
            owner,
            repo
        );
        Ok(body.workflows)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn test_repo_deserialization() {
        let json = r#"{
            "id": 123,
            "name": "test-repo",
            "full_name": "user/test-repo",
            "description": "A test repo",
            "html_url": "https://github.com/user/test-repo",
            "private": false,
            "default_branch": "main",
            "open_issues_count": 5,
            "updated_at": "2026-01-21T00:00:00Z"
        }"#;
        let repo: GitHubRepo = serde_json::from_str(json).unwrap();
        assert_eq!(repo.name, "test-repo");
        assert_eq!(repo.open_issues_count, 5);
    }

    #[test]
    fn test_issue_deserialization() {
        let json = r#"{
            "id": 456,
            "number": 42,
            "title": "Test issue",
            "body": "Issue body",
            "state": "open",
            "html_url": "https://github.com/user/repo/issues/42",
            "labels": [{"id": 1, "name": "bug", "color": "ff0000"}],
            "created_at": "2026-01-21T00:00:00Z",
            "updated_at": "2026-01-21T00:00:00Z"
        }"#;
        let issue: GitHubIssue = serde_json::from_str(json).unwrap();
        assert_eq!(issue.number, 42);
        assert_eq!(issue.labels.len(), 1);
    }

    #[test]
    fn test_create_issue_serialization() {
        let req = CreateIssueRequest {
            title: "New issue".to_string(),
            body: Some("Description".to_string()),
            labels: Some(vec!["todo".to_string()]),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("New issue"));
        assert!(json.contains("todo"));
    }

    #[test]
    fn test_update_issue_skips_none() {
        let req = UpdateIssueRequest {
            title: None,
            body: None,
            state: Some("closed".to_string()),
            labels: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("title"));
        assert!(json.contains("closed"));
    }

    #[test]
    fn test_workflow_deserialization() {
        let json = r#"{
            "total_count": 2,
            "workflows": [
                {
                    "id": 161335,
                    "node_id": "MDg6V29ya2Zsb3cxNjEzMzU=",
                    "name": "CI",
                    "path": ".github/workflows/blank.yaml",
                    "state": "active",
                    "created_at": "2020-01-08T23:48:37.000-08:00",
                    "updated_at": "2020-01-08T23:50:21.000-08:00",
                    "url": "https://api.github.com/repos/octo-org/octo-repo/actions/workflows/161335",
                    "html_url": "https://github.com/octo-org/octo-repo/blob/master/.github/workflows/blank.yaml",
                    "badge_url": "https://github.com/octo-org/octo-repo/workflows/CI/badge.svg"
                },
                {
                    "id": 269289,
                    "name": "Linter",
                    "path": ".github/workflows/linter.yaml",
                    "state": "active"
                }
            ]
        }"#;
        let response: ListWorkflowsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.total_count, 2);
        assert_eq!(response.workflows.len(), 2);
        assert_eq!(response.workflows[0].name, "CI");
        assert_eq!(response.workflows[0].path, ".github/workflows/blank.yaml");
        assert_eq!(response.workflows[0].state, "active");
        assert!(response.workflows[0].html_url.is_some());
        assert_eq!(response.workflows[1].name, "Linter");
        assert!(response.workflows[1].html_url.is_none());
    }
}
