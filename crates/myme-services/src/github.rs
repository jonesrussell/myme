// crates/myme-services/src/github.rs

use anyhow::{Context, Result};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;

const GITHUB_API_URL: &str = "https://api.github.com";

/// GitHub repository representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub html_url: String,
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

/// GitHub API client
#[derive(Debug, Clone)]
pub struct GitHubClient {
    base_url: Url,
    client: Arc<Client>,
    token: String,
}

impl GitHubClient {
    /// Create a new GitHub client with OAuth token
    pub fn new(token: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            base_url: Url::parse(GITHUB_API_URL).unwrap(),
            client: Arc::new(client),
            token,
        })
    }

    /// Build request with auth headers
    fn build_request(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.header(header::AUTHORIZATION, format!("Bearer {}", self.token))
            .header(header::ACCEPT, "application/vnd.github+json")
            .header(header::USER_AGENT, "myme-app")
            .header("X-GitHub-Api-Version", "2022-11-28")
    }

    /// Check response status and extract error
    async fn check_response(&self, response: reqwest::Response) -> Result<reqwest::Response> {
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error ({}): {}", status, error_text);
        }
        Ok(response)
    }

    /// List repositories for authenticated user
    pub async fn list_repos(&self) -> Result<Vec<GitHubRepo>> {
        tracing::debug!("Fetching user repositories");

        let url = self.base_url.join("user/repos")?;
        let request = self.build_request(
            self.client
                .get(url)
                .query(&[("sort", "updated"), ("per_page", "100")]),
        );

        let response = request.send().await.context("Failed to fetch repos")?;
        let response = self.check_response(response).await?;
        let repos: Vec<GitHubRepo> = response.json().await?;

        tracing::info!("Fetched {} repositories", repos.len());
        Ok(repos)
    }

    /// Get a specific repository
    pub async fn get_repo(&self, owner: &str, repo: &str) -> Result<GitHubRepo> {
        tracing::debug!("Fetching repository {}/{}", owner, repo);

        let url = self.base_url.join(&format!("repos/{}/{}", owner, repo))?;
        let request = self.build_request(self.client.get(url));

        let response = request.send().await.context("Failed to fetch repo")?;
        let response = self.check_response(response).await?;
        let repo: GitHubRepo = response.json().await?;

        Ok(repo)
    }

    /// Create a new repository
    pub async fn create_repo(&self, req: CreateRepoRequest) -> Result<GitHubRepo> {
        tracing::debug!("Creating repository: {}", req.name);

        let url = self.base_url.join("user/repos")?;
        let request = self.build_request(self.client.post(url).json(&req));

        let response = request.send().await.context("Failed to create repo")?;
        let response = self.check_response(response).await?;
        let repo: GitHubRepo = response.json().await?;

        tracing::info!("Created repository: {}", repo.full_name);
        Ok(repo)
    }
}

#[cfg(test)]
mod tests {
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
}
