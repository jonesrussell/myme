use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// GitHub repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Repository ID
    pub id: u64,

    /// Repository name
    pub name: String,

    /// Repository full name (owner/name)
    pub full_name: String,

    /// Repository description
    pub description: Option<String>,

    /// Repository URL
    pub html_url: String,

    /// Clone URL (HTTPS)
    pub clone_url: String,

    /// SSH URL
    pub ssh_url: String,

    /// Default branch
    pub default_branch: String,

    /// Is private repository
    pub private: bool,

    /// Is fork
    pub fork: bool,

    /// Star count
    pub stargazers_count: u32,

    /// Fork count
    pub forks_count: u32,

    /// Open issues count
    pub open_issues_count: u32,

    /// Created at timestamp
    pub created_at: String,

    /// Updated at timestamp
    pub updated_at: String,

    /// Pushed at timestamp
    pub pushed_at: Option<String>,
}

/// GitHub issue information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Issue ID
    pub id: u64,

    /// Issue number
    pub number: u32,

    /// Issue title
    pub title: String,

    /// Issue body
    pub body: Option<String>,

    /// Issue state (open, closed)
    pub state: String,

    /// Issue URL
    pub html_url: String,

    /// Issue author
    pub user: User,

    /// Labels
    pub labels: Vec<Label>,

    /// Created at timestamp
    pub created_at: String,

    /// Updated at timestamp
    pub updated_at: String,
}

/// GitHub user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID
    pub id: u64,

    /// Username
    pub login: String,

    /// Avatar URL
    pub avatar_url: String,
}

/// GitHub label information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    /// Label ID
    pub id: u64,

    /// Label name
    pub name: String,

    /// Label color (hex without #)
    pub color: String,
}

/// GitHub API client
pub struct GitHubClient {
    /// HTTP client
    client: reqwest::Client,

    /// Access token
    access_token: String,

    /// Base API URL
    base_url: String,
}

impl GitHubClient {
    /// Create a new GitHub client with access token
    ///
    /// # Arguments
    /// * `access_token` - GitHub OAuth access token
    pub fn new(access_token: String) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent("MyMe/0.1.0")
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client, access_token, base_url: "https://api.github.com".to_string() })
    }

    /// List repositories for the authenticated user
    ///
    /// # Arguments
    /// * `visibility` - Filter by visibility: "all", "public", or "private"
    /// * `sort` - Sort by: "created", "updated", "pushed", "full_name"
    pub async fn list_repositories(
        &self,
        visibility: Option<&str>,
        sort: Option<&str>,
    ) -> Result<Vec<Repository>> {
        let url = format!("{}/user/repos", self.base_url);
        let mut query_params = vec![("per_page", "100")];

        if let Some(v) = visibility {
            query_params.push(("visibility", v));
        }

        if let Some(s) = sort {
            query_params.push(("sort", s));
        }

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/vnd.github+json")
            .query(&query_params)
            .send()
            .await
            .context("Failed to send request to GitHub API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error {}: {}", status, body);
        }

        let repos: Vec<Repository> = response
            .json::<Vec<Repository>>()
            .await
            .context("Failed to parse GitHub API response")?;

        tracing::info!("Fetched {} repositories from GitHub", repos.len());
        Ok(repos)
    }

    /// List issues for a repository
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `state` - Filter by state: "open", "closed", or "all"
    pub async fn list_issues(
        &self,
        owner: &str,
        repo: &str,
        state: Option<&str>,
    ) -> Result<Vec<Issue>> {
        let url = format!("{}/repos/{}/{}/issues", self.base_url, owner, repo);
        let mut query_params = vec![("per_page", "100")];

        if let Some(s) = state {
            query_params.push(("state", s));
        }

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/vnd.github+json")
            .query(&query_params)
            .send()
            .await
            .context("Failed to send request to GitHub API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error {}: {}", status, body);
        }

        let issues: Vec<Issue> =
            response.json::<Vec<Issue>>().await.context("Failed to parse GitHub API response")?;

        tracing::info!("Fetched {} issues from {}/{}", issues.len(), owner, repo);
        Ok(issues)
    }

    /// Create a new repository
    ///
    /// # Arguments
    /// * `name` - Repository name
    /// * `description` - Repository description
    /// * `private` - Whether the repository is private
    pub async fn create_repository(
        &self,
        name: &str,
        description: Option<&str>,
        private: bool,
    ) -> Result<Repository> {
        let url = format!("{}/user/repos", self.base_url);

        let mut body = serde_json::json!({
            "name": name,
            "private": private,
            "auto_init": true,
        });

        if let Some(desc) = description {
            body["description"] = serde_json::json!(desc);
        }

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/vnd.github+json")
            .json(&body)
            .send()
            .await
            .context("Failed to send request to GitHub API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error {}: {}", status, body);
        }

        let repo: Repository =
            response.json::<Repository>().await.context("Failed to parse GitHub API response")?;

        tracing::info!("Created repository: {}", repo.full_name);
        Ok(repo)
    }

    /// Get authenticated user information
    pub async fn get_user(&self) -> Result<User> {
        let url = format!("{}/user", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .context("Failed to send request to GitHub API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error {}: {}", status, body);
        }

        let user: User =
            response.json::<User>().await.context("Failed to parse GitHub API response")?;

        tracing::info!("Authenticated as: {}", user.login);
        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn test_github_client_creation() {
        let client = GitHubClient::new("test_token".to_string()).unwrap();
        assert_eq!(client.access_token, "test_token");
        assert_eq!(client.base_url, "https://api.github.com");
    }
}
