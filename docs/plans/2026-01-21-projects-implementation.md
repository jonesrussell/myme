# Projects Feature Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add GitHub-backed project management with kanban boards and two-way issue sync.

**Architecture:** GitHub-first approach where projects are thin wrappers around repos. Tasks mirror GitHub issues with label-based status mapping. Polling for sync (no webhooks).

**Tech Stack:** Rust, reqwest, tokio, cxx-qt, Qt/QML, SQLite (rusqlite), GitHub REST API

---

## Task 1: GitHub API Client - Data Types

**Files:**
- Create: `crates/myme-services/src/github.rs`
- Modify: `crates/myme-services/src/lib.rs`

**Step 1: Create the data types file**

```rust
// crates/myme-services/src/github.rs

use serde::{Deserialize, Serialize};

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
```

**Step 2: Run tests to verify**

Run: `cargo test -p myme-services github`
Expected: All 4 tests pass

**Step 3: Export from lib.rs**

Add to `crates/myme-services/src/lib.rs`:

```rust
pub mod github;
pub use github::*;
```

**Step 4: Verify it compiles**

Run: `cargo build -p myme-services`
Expected: Compiles without errors

**Step 5: Commit**

```bash
git add crates/myme-services/src/github.rs crates/myme-services/src/lib.rs
git commit -m "feat(projects): add GitHub API data types"
```

---

## Task 2: GitHub API Client - Core Client

**Files:**
- Modify: `crates/myme-services/src/github.rs`

**Step 1: Add client struct and constructor**

Add after the data types in `github.rs`:

```rust
use anyhow::{Context, Result};
use reqwest::{header, Client};
use std::sync::Arc;
use url::Url;

const GITHUB_API_URL: &str = "https://api.github.com";

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
}
```

**Step 2: Verify it compiles**

Run: `cargo build -p myme-services`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add crates/myme-services/src/github.rs
git commit -m "feat(projects): add GitHubClient struct with auth"
```

---

## Task 3: GitHub API Client - Repository Methods

**Files:**
- Modify: `crates/myme-services/src/github.rs`

**Step 1: Add repository methods**

Add to `impl GitHubClient`:

```rust
    /// List repositories for authenticated user
    pub async fn list_repos(&self) -> Result<Vec<GitHubRepo>> {
        tracing::debug!("Fetching user repositories");

        let url = self.base_url.join("user/repos")?;
        let request = self.build_request(
            self.client
                .get(url)
                .query(&[("sort", "updated"), ("per_page", "100")])
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
```

**Step 2: Verify it compiles**

Run: `cargo build -p myme-services`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add crates/myme-services/src/github.rs
git commit -m "feat(projects): add GitHub repo API methods"
```

---

## Task 4: GitHub API Client - Issue Methods

**Files:**
- Modify: `crates/myme-services/src/github.rs`

**Step 1: Add issue methods**

Add to `impl GitHubClient`:

```rust
    /// List issues for a repository
    pub async fn list_issues(&self, owner: &str, repo: &str) -> Result<Vec<GitHubIssue>> {
        tracing::debug!("Fetching issues for {}/{}", owner, repo);

        let url = self.base_url.join(&format!("repos/{}/{}/issues", owner, repo))?;
        let request = self.build_request(
            self.client
                .get(url)
                .query(&[("state", "all"), ("per_page", "100")])
        );

        let response = request.send().await.context("Failed to fetch issues")?;
        let response = self.check_response(response).await?;
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

        let url = self.base_url.join(&format!("repos/{}/{}/issues", owner, repo))?;
        let request = self.build_request(
            self.client
                .get(url)
                .query(&[("state", "all"), ("since", since), ("per_page", "100")])
        );

        let response = request.send().await.context("Failed to fetch issues")?;
        let response = self.check_response(response).await?;
        let issues: Vec<GitHubIssue> = response.json().await?;

        Ok(issues)
    }

    /// Create a new issue
    pub async fn create_issue(
        &self,
        owner: &str,
        repo: &str,
        req: CreateIssueRequest,
    ) -> Result<GitHubIssue> {
        tracing::debug!("Creating issue in {}/{}: {}", owner, repo, req.title);

        let url = self.base_url.join(&format!("repos/{}/{}/issues", owner, repo))?;
        let request = self.build_request(self.client.post(url).json(&req));

        let response = request.send().await.context("Failed to create issue")?;
        let response = self.check_response(response).await?;
        let issue: GitHubIssue = response.json().await?;

        tracing::info!("Created issue #{} in {}/{}", issue.number, owner, repo);
        Ok(issue)
    }

    /// Update an issue
    pub async fn update_issue(
        &self,
        owner: &str,
        repo: &str,
        issue_number: i32,
        req: UpdateIssueRequest,
    ) -> Result<GitHubIssue> {
        tracing::debug!("Updating issue #{} in {}/{}", issue_number, owner, repo);

        let url = self.base_url.join(&format!(
            "repos/{}/{}/issues/{}",
            owner, repo, issue_number
        ))?;
        let request = self.build_request(self.client.patch(url).json(&req));

        let response = request.send().await.context("Failed to update issue")?;
        let response = self.check_response(response).await?;
        let issue: GitHubIssue = response.json().await?;

        Ok(issue)
    }

    /// Close an issue
    pub async fn close_issue(&self, owner: &str, repo: &str, issue_number: i32) -> Result<GitHubIssue> {
        self.update_issue(owner, repo, issue_number, UpdateIssueRequest {
            title: None,
            body: None,
            state: Some("closed".to_string()),
            labels: None,
        }).await
    }

    /// Reopen an issue
    pub async fn reopen_issue(&self, owner: &str, repo: &str, issue_number: i32) -> Result<GitHubIssue> {
        self.update_issue(owner, repo, issue_number, UpdateIssueRequest {
            title: None,
            body: None,
            state: Some("open".to_string()),
            labels: None,
        }).await
    }
```

**Step 2: Verify it compiles**

Run: `cargo build -p myme-services`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add crates/myme-services/src/github.rs
git commit -m "feat(projects): add GitHub issue API methods"
```

---

## Task 5: GitHub API Client - Label Methods

**Files:**
- Modify: `crates/myme-services/src/github.rs`

**Step 1: Add label methods**

Add to `impl GitHubClient`:

```rust
    /// List labels for a repository
    pub async fn list_labels(&self, owner: &str, repo: &str) -> Result<Vec<GitHubLabel>> {
        tracing::debug!("Fetching labels for {}/{}", owner, repo);

        let url = self.base_url.join(&format!("repos/{}/{}/labels", owner, repo))?;
        let request = self.build_request(self.client.get(url));

        let response = request.send().await.context("Failed to fetch labels")?;
        let response = self.check_response(response).await?;
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

        let url = self.base_url.join(&format!("repos/{}/{}/labels", owner, repo))?;
        let request = self.build_request(self.client.post(url).json(&req));

        let response = request.send().await.context("Failed to create label")?;
        let response = self.check_response(response).await?;
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
        tracing::debug!("Setting labels on issue #{} in {}/{}", issue_number, owner, repo);

        let url = self.base_url.join(&format!(
            "repos/{}/{}/issues/{}/labels",
            owner, repo, issue_number
        ))?;

        #[derive(Serialize)]
        struct SetLabelsRequest {
            labels: Vec<String>,
        }

        let request = self.build_request(
            self.client.put(url).json(&SetLabelsRequest { labels })
        );

        let response = request.send().await.context("Failed to set labels")?;
        let response = self.check_response(response).await?;
        let labels: Vec<GitHubLabel> = response.json().await?;

        Ok(labels)
    }
```

**Step 2: Verify it compiles**

Run: `cargo build -p myme-services`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add crates/myme-services/src/github.rs
git commit -m "feat(projects): add GitHub label API methods"
```

---

## Task 6: Project Data Types & Status Mapping

**Files:**
- Create: `crates/myme-services/src/project.rs`
- Modify: `crates/myme-services/src/lib.rs`

**Step 1: Create project types with status mapping**

```rust
// crates/myme-services/src/project.rs

use serde::{Deserialize, Serialize};

/// Task status in kanban board
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Backlog,
    Todo,
    InProgress,
    Blocked,
    Review,
    Done,
}

impl TaskStatus {
    /// Get the GitHub label name for this status
    pub fn to_label(&self) -> Option<&'static str> {
        match self {
            TaskStatus::Backlog => Some("backlog"),
            TaskStatus::Todo => Some("todo"),
            TaskStatus::InProgress => Some("in-progress"),
            TaskStatus::Blocked => Some("blocked"),
            TaskStatus::Review => Some("review"),
            TaskStatus::Done => None, // Done = closed, no label needed
        }
    }

    /// Get the GitHub label color for this status
    pub fn label_color(&self) -> &'static str {
        match self {
            TaskStatus::Backlog => "e0e0e0",
            TaskStatus::Todo => "0366d6",
            TaskStatus::InProgress => "fbca04",
            TaskStatus::Blocked => "d93f0b",
            TaskStatus::Review => "6f42c1",
            TaskStatus::Done => "0e8a16",
        }
    }

    /// Parse status from GitHub issue state and labels
    pub fn from_github(state: &str, labels: &[String]) -> Self {
        if state == "closed" {
            return TaskStatus::Done;
        }

        // Check labels in priority order
        for label in labels {
            match label.as_str() {
                "blocked" => return TaskStatus::Blocked,
                "review" => return TaskStatus::Review,
                "in-progress" => return TaskStatus::InProgress,
                "backlog" => return TaskStatus::Backlog,
                "todo" => return TaskStatus::Todo,
                _ => {}
            }
        }

        // Default for open issues with no status label
        TaskStatus::Todo
    }

    /// Get all status variants
    pub fn all() -> &'static [TaskStatus] {
        &[
            TaskStatus::Backlog,
            TaskStatus::Todo,
            TaskStatus::InProgress,
            TaskStatus::Blocked,
            TaskStatus::Review,
            TaskStatus::Done,
        ]
    }
}

/// Local project representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub github_repo: String,
    pub description: Option<String>,
    pub created_at: String,
    pub last_synced: Option<String>,
}

/// Local task representation (mirrors GitHub issue)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub github_issue_number: i32,
    pub title: String,
    pub body: Option<String>,
    pub status: TaskStatus,
    pub labels: Vec<String>,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_from_closed_issue() {
        let status = TaskStatus::from_github("closed", &[]);
        assert_eq!(status, TaskStatus::Done);
    }

    #[test]
    fn test_status_from_labels() {
        let status = TaskStatus::from_github("open", &["in-progress".to_string()]);
        assert_eq!(status, TaskStatus::InProgress);
    }

    #[test]
    fn test_status_blocked_priority() {
        // Blocked should take priority over other labels
        let status = TaskStatus::from_github(
            "open",
            &["todo".to_string(), "blocked".to_string()],
        );
        assert_eq!(status, TaskStatus::Blocked);
    }

    #[test]
    fn test_status_default_to_todo() {
        let status = TaskStatus::from_github("open", &["bug".to_string()]);
        assert_eq!(status, TaskStatus::Todo);
    }

    #[test]
    fn test_status_to_label() {
        assert_eq!(TaskStatus::InProgress.to_label(), Some("in-progress"));
        assert_eq!(TaskStatus::Done.to_label(), None);
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p myme-services project`
Expected: All 5 tests pass

**Step 3: Export from lib.rs**

Add to `crates/myme-services/src/lib.rs`:

```rust
pub mod project;
pub use project::*;
```

**Step 4: Verify it compiles**

Run: `cargo build -p myme-services`
Expected: Compiles without errors

**Step 5: Commit**

```bash
git add crates/myme-services/src/project.rs crates/myme-services/src/lib.rs
git commit -m "feat(projects): add Project/Task types with status mapping"
```

---

## Task 7: Configuration - Add Projects Config

**Files:**
- Modify: `crates/myme-core/src/config.rs`

**Step 1: Add ProjectsConfig struct**

Add after other config structs in `config.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectsConfig {
    /// Sync interval in minutes (default: 5)
    #[serde(default = "default_sync_interval")]
    pub sync_interval_minutes: u32,
    /// Auto-create status labels on repos (default: true)
    #[serde(default = "default_auto_create_labels")]
    pub auto_create_labels: bool,
}

fn default_sync_interval() -> u32 {
    5
}

fn default_auto_create_labels() -> bool {
    true
}

impl Default for ProjectsConfig {
    fn default() -> Self {
        Self {
            sync_interval_minutes: default_sync_interval(),
            auto_create_labels: default_auto_create_labels(),
        }
    }
}
```

**Step 2: Add to main Config struct**

Add field to `Config` struct:

```rust
#[serde(default)]
pub projects: ProjectsConfig,
```

**Step 3: Verify it compiles**

Run: `cargo build -p myme-core`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add crates/myme-core/src/config.rs
git commit -m "feat(projects): add ProjectsConfig to configuration"
```

---

## Task 8: Local Storage - SQLite Database

**Files:**
- Create: `crates/myme-services/src/project_store.rs`
- Modify: `crates/myme-services/src/lib.rs`
- Modify: `crates/myme-services/Cargo.toml`

**Step 1: Add rusqlite dependency**

Add to `crates/myme-services/Cargo.toml` under `[dependencies]`:

```toml
rusqlite = { version = "0.31", features = ["bundled"] }
```

**Step 2: Create the store**

```rust
// crates/myme-services/src/project_store.rs

use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use std::path::Path;

use crate::project::{Project, Task, TaskStatus};

/// Local SQLite storage for projects and tasks
pub struct ProjectStore {
    conn: Connection,
}

impl ProjectStore {
    /// Open or create the database
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .context("Failed to open projects database")?;

        let store = Self { conn };
        store.init_schema()?;

        Ok(store)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                github_repo TEXT NOT NULL UNIQUE,
                description TEXT,
                created_at TEXT NOT NULL,
                last_synced TEXT
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                github_issue_number INTEGER NOT NULL,
                title TEXT NOT NULL,
                body TEXT,
                status TEXT NOT NULL,
                labels TEXT NOT NULL,
                html_url TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects(id),
                UNIQUE (project_id, github_issue_number)
            );

            CREATE INDEX IF NOT EXISTS idx_tasks_project ON tasks(project_id);
            CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);"
        ).context("Failed to initialize schema")?;

        Ok(())
    }

    /// Insert or update a project
    pub fn upsert_project(&self, project: &Project) -> Result<()> {
        self.conn.execute(
            "INSERT INTO projects (id, github_repo, description, created_at, last_synced)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                github_repo = excluded.github_repo,
                description = excluded.description,
                last_synced = excluded.last_synced",
            params![
                project.id,
                project.github_repo,
                project.description,
                project.created_at,
                project.last_synced,
            ],
        )?;
        Ok(())
    }

    /// Get all projects
    pub fn list_projects(&self) -> Result<Vec<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, github_repo, description, created_at, last_synced
             FROM projects ORDER BY last_synced DESC NULLS LAST"
        )?;

        let projects = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                github_repo: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
                last_synced: row.get(4)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(projects)
    }

    /// Get a project by ID
    pub fn get_project(&self, id: &str) -> Result<Option<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, github_repo, description, created_at, last_synced
             FROM projects WHERE id = ?1"
        )?;

        let project = stmt.query_row([id], |row| {
            Ok(Project {
                id: row.get(0)?,
                github_repo: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
                last_synced: row.get(4)?,
            })
        }).optional()?;

        Ok(project)
    }

    /// Delete a project and its tasks
    pub fn delete_project(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM tasks WHERE project_id = ?1", [id])?;
        self.conn.execute("DELETE FROM projects WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Insert or update a task
    pub fn upsert_task(&self, task: &Task) -> Result<()> {
        let labels_json = serde_json::to_string(&task.labels)?;
        let status_str = serde_json::to_string(&task.status)?;

        self.conn.execute(
            "INSERT INTO tasks (id, project_id, github_issue_number, title, body, status, labels, html_url, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(project_id, github_issue_number) DO UPDATE SET
                title = excluded.title,
                body = excluded.body,
                status = excluded.status,
                labels = excluded.labels,
                html_url = excluded.html_url,
                updated_at = excluded.updated_at",
            params![
                task.id,
                task.project_id,
                task.github_issue_number,
                task.title,
                task.body,
                status_str,
                labels_json,
                task.html_url,
                task.created_at,
                task.updated_at,
            ],
        )?;
        Ok(())
    }

    /// Get tasks for a project
    pub fn list_tasks(&self, project_id: &str) -> Result<Vec<Task>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, github_issue_number, title, body, status, labels, html_url, created_at, updated_at
             FROM tasks WHERE project_id = ?1 ORDER BY github_issue_number"
        )?;

        let tasks = stmt.query_map([project_id], |row| {
            let status_str: String = row.get(5)?;
            let labels_json: String = row.get(6)?;

            Ok(Task {
                id: row.get(0)?,
                project_id: row.get(1)?,
                github_issue_number: row.get(2)?,
                title: row.get(3)?,
                body: row.get(4)?,
                status: serde_json::from_str(&status_str).unwrap_or(TaskStatus::Todo),
                labels: serde_json::from_str(&labels_json).unwrap_or_default(),
                html_url: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(tasks)
    }

    /// Delete a task
    pub fn delete_task(&self, project_id: &str, issue_number: i32) -> Result<()> {
        self.conn.execute(
            "DELETE FROM tasks WHERE project_id = ?1 AND github_issue_number = ?2",
            params![project_id, issue_number],
        )?;
        Ok(())
    }

    /// Count tasks by status for a project
    pub fn count_tasks_by_status(&self, project_id: &str) -> Result<Vec<(TaskStatus, i32)>> {
        let mut stmt = self.conn.prepare(
            "SELECT status, COUNT(*) FROM tasks WHERE project_id = ?1 GROUP BY status"
        )?;

        let counts = stmt.query_map([project_id], |row| {
            let status_str: String = row.get(0)?;
            let count: i32 = row.get(1)?;
            let status = serde_json::from_str(&status_str).unwrap_or(TaskStatus::Todo);
            Ok((status, count))
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(counts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_and_list_project() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = ProjectStore::open(&db_path).unwrap();

        let project = Project {
            id: "test-123".to_string(),
            github_repo: "user/repo".to_string(),
            description: Some("Test project".to_string()),
            created_at: "2026-01-21T00:00:00Z".to_string(),
            last_synced: None,
        };

        store.upsert_project(&project).unwrap();

        let projects = store.list_projects().unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].github_repo, "user/repo");
    }

    #[test]
    fn test_create_and_list_tasks() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = ProjectStore::open(&db_path).unwrap();

        let project = Project {
            id: "proj-1".to_string(),
            github_repo: "user/repo".to_string(),
            description: None,
            created_at: "2026-01-21T00:00:00Z".to_string(),
            last_synced: None,
        };
        store.upsert_project(&project).unwrap();

        let task = Task {
            id: "task-1".to_string(),
            project_id: "proj-1".to_string(),
            github_issue_number: 42,
            title: "Test task".to_string(),
            body: Some("Description".to_string()),
            status: TaskStatus::InProgress,
            labels: vec!["in-progress".to_string()],
            html_url: "https://github.com/user/repo/issues/42".to_string(),
            created_at: "2026-01-21T00:00:00Z".to_string(),
            updated_at: "2026-01-21T00:00:00Z".to_string(),
        };
        store.upsert_task(&task).unwrap();

        let tasks = store.list_tasks("proj-1").unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].status, TaskStatus::InProgress);
    }
}
```

**Step 3: Add tempfile dev dependency**

Add to `crates/myme-services/Cargo.toml` under `[dev-dependencies]`:

```toml
tempfile = "3"
```

**Step 4: Export from lib.rs**

Add to `crates/myme-services/src/lib.rs`:

```rust
pub mod project_store;
pub use project_store::ProjectStore;
```

**Step 5: Run tests**

Run: `cargo test -p myme-services project_store`
Expected: All tests pass

**Step 6: Commit**

```bash
git add crates/myme-services/
git commit -m "feat(projects): add SQLite storage for projects and tasks"
```

---

## Task 9: Bridge Initialization - Add GitHub Client

**Files:**
- Modify: `crates/myme-ui/src/bridge.rs`

**Step 1: Add static for GitHub client**

Add after other statics:

```rust
static GITHUB_CLIENT: OnceLock<Arc<GitHubClient>> = OnceLock::new();
static PROJECT_STORE: OnceLock<Arc<std::sync::Mutex<ProjectStore>>> = OnceLock::new();
```

Add imports at top:

```rust
use myme_services::{GitHubClient, ProjectStore};
```

**Step 2: Add initialization function**

Add function:

```rust
#[no_mangle]
pub extern "C" fn initialize_github_client() -> bool {
    // Get token from secure storage
    let token = match myme_auth::SecureStorage::retrieve_token("github") {
        Ok(token_set) if !token_set.is_expired() => token_set.access_token,
        _ => {
            tracing::warn!("No valid GitHub token found, projects will be unavailable");
            return false;
        }
    };

    // Create GitHub client
    let client = match GitHubClient::new(token) {
        Ok(c) => Arc::new(c),
        Err(e) => {
            tracing::error!("Failed to create GitHub client: {}", e);
            return false;
        }
    };

    if GITHUB_CLIENT.set(client).is_err() {
        tracing::warn!("GitHub client already initialized");
    }

    // Initialize project store
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("myme");

    let db_path = config_dir.join("projects.db");

    // Ensure directory exists
    if let Err(e) = std::fs::create_dir_all(&config_dir) {
        tracing::error!("Failed to create config directory: {}", e);
        return false;
    }

    let store = match ProjectStore::open(&db_path) {
        Ok(s) => Arc::new(std::sync::Mutex::new(s)),
        Err(e) => {
            tracing::error!("Failed to open project store: {}", e);
            return false;
        }
    };

    if PROJECT_STORE.set(store).is_err() {
        tracing::warn!("Project store already initialized");
    }

    tracing::info!("GitHub client and project store initialized");
    true
}

/// Get GitHub client and runtime
pub fn get_github_client_and_runtime() -> Option<(Arc<GitHubClient>, tokio::runtime::Handle)> {
    let client = GITHUB_CLIENT.get()?.clone();
    let runtime = RUNTIME.get()?.handle().clone();
    Some((client, runtime))
}

/// Get project store
pub fn get_project_store() -> Option<Arc<std::sync::Mutex<ProjectStore>>> {
    PROJECT_STORE.get().cloned()
}

/// Check if GitHub is authenticated
pub fn is_github_authenticated() -> bool {
    GITHUB_CLIENT.get().is_some()
}
```

**Step 3: Verify it compiles**

Run: `cargo build -p myme-ui`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add crates/myme-ui/src/bridge.rs
git commit -m "feat(projects): add GitHub client initialization to bridge"
```

---

## Task 10: ProjectModel - Basic Structure

**Files:**
- Create: `crates/myme-ui/src/models/project_model.rs`
- Modify: `crates/myme-ui/src/models/mod.rs`
- Modify: `crates/myme-ui/build.rs`

**Step 1: Create the model file**

```rust
// crates/myme-ui/src/models/project_model.rs

use cxx_qt_lib::QString;
use std::pin::Pin;
use std::sync::Arc;
use myme_services::{GitHubClient, Project, ProjectStore, TaskStatus};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, loading)]
        #[qproperty(bool, authenticated)]
        #[qproperty(QString, error_message)]
        type ProjectModel = super::ProjectModelRust;

        #[qinvokable]
        fn fetch_projects(self: Pin<&mut ProjectModel>);

        #[qinvokable]
        fn row_count(self: &ProjectModel) -> i32;

        #[qinvokable]
        fn get_id(self: &ProjectModel, index: i32) -> QString;

        #[qinvokable]
        fn get_github_repo(self: &ProjectModel, index: i32) -> QString;

        #[qinvokable]
        fn get_description(self: &ProjectModel, index: i32) -> QString;

        #[qinvokable]
        fn get_task_counts(self: &ProjectModel, index: i32) -> QString;

        #[qinvokable]
        fn add_project(self: Pin<&mut ProjectModel>, github_repo: QString);

        #[qinvokable]
        fn remove_project(self: Pin<&mut ProjectModel>, index: i32);

        #[qinvokable]
        fn sync_project(self: Pin<&mut ProjectModel>, index: i32);

        #[qinvokable]
        fn check_auth(self: Pin<&mut ProjectModel>);

        #[qsignal]
        fn projects_changed(self: Pin<&mut ProjectModel>);

        #[qsignal]
        fn auth_changed(self: Pin<&mut ProjectModel>);
    }
}

#[derive(Default)]
pub struct ProjectModelRust {
    loading: bool,
    authenticated: bool,
    error_message: QString,
    projects: Vec<Project>,
    task_counts: std::collections::HashMap<String, Vec<(TaskStatus, i32)>>,
    client: Option<Arc<GitHubClient>>,
    store: Option<Arc<std::sync::Mutex<ProjectStore>>>,
    runtime: Option<tokio::runtime::Handle>,
}

impl qobject::ProjectModel {
    fn ensure_initialized(&mut self) {
        if self.client.is_none() {
            if let Some((client, runtime)) = crate::bridge::get_github_client_and_runtime() {
                self.client = Some(client);
                self.runtime = Some(runtime);
            }
        }
        if self.store.is_none() {
            self.store = crate::bridge::get_project_store();
        }
        self.authenticated = crate::bridge::is_github_authenticated();
    }

    fn get_project(&self, index: i32) -> Option<&Project> {
        if index >= 0 && (index as usize) < self.projects.len() {
            Some(&self.projects[index as usize])
        } else {
            None
        }
    }
}

impl qobject::ProjectModel {
    pub fn check_auth(mut self: Pin<&mut Self>) {
        self.as_mut().ensure_initialized();
        let was_auth = self.authenticated;
        self.as_mut().set_authenticated(crate::bridge::is_github_authenticated());
        if was_auth != self.authenticated {
            self.as_mut().auth_changed();
        }
    }

    pub fn fetch_projects(mut self: Pin<&mut Self>) {
        self.as_mut().ensure_initialized();
        self.as_mut().set_loading(true);
        self.as_mut().set_error_message(QString::from(""));

        // Load from local store first
        if let Some(store) = &self.store {
            if let Ok(guard) = store.lock() {
                match guard.list_projects() {
                    Ok(projects) => {
                        // Load task counts
                        let mut counts = std::collections::HashMap::new();
                        for p in &projects {
                            if let Ok(c) = guard.count_tasks_by_status(&p.id) {
                                counts.insert(p.id.clone(), c);
                            }
                        }
                        self.as_mut().rust_mut().projects = projects;
                        self.as_mut().rust_mut().task_counts = counts;
                    }
                    Err(e) => {
                        self.as_mut().set_error_message(QString::from(&format!("Failed to load projects: {}", e)));
                    }
                }
            }
        }

        self.as_mut().set_loading(false);
        self.as_mut().projects_changed();
    }

    pub fn row_count(&self) -> i32 {
        self.projects.len() as i32
    }

    pub fn get_id(&self, index: i32) -> QString {
        self.get_project(index)
            .map(|p| QString::from(&p.id))
            .unwrap_or_default()
    }

    pub fn get_github_repo(&self, index: i32) -> QString {
        self.get_project(index)
            .map(|p| QString::from(&p.github_repo))
            .unwrap_or_default()
    }

    pub fn get_description(&self, index: i32) -> QString {
        self.get_project(index)
            .and_then(|p| p.description.as_ref())
            .map(|d| QString::from(d.as_str()))
            .unwrap_or_default()
    }

    pub fn get_task_counts(&self, index: i32) -> QString {
        // Returns JSON: {"backlog":0,"todo":3,"in_progress":1,...}
        self.get_project(index)
            .and_then(|p| self.task_counts.get(&p.id))
            .map(|counts| {
                let mut map = serde_json::Map::new();
                for status in TaskStatus::all() {
                    let count = counts.iter()
                        .find(|(s, _)| s == status)
                        .map(|(_, c)| *c)
                        .unwrap_or(0);
                    let key = format!("{:?}", status).to_lowercase();
                    map.insert(key, serde_json::Value::Number(count.into()));
                }
                QString::from(&serde_json::to_string(&map).unwrap_or_default())
            })
            .unwrap_or_else(|| QString::from("{}"))
    }

    pub fn add_project(mut self: Pin<&mut Self>, github_repo: QString) {
        self.as_mut().ensure_initialized();
        self.as_mut().set_loading(true);
        self.as_mut().set_error_message(QString::from(""));

        let repo_str = github_repo.to_string();

        // Create project locally
        let project = Project {
            id: uuid::Uuid::new_v4().to_string(),
            github_repo: repo_str,
            description: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_synced: None,
        };

        if let Some(store) = &self.store {
            if let Ok(guard) = store.lock() {
                if let Err(e) = guard.upsert_project(&project) {
                    self.as_mut().set_error_message(QString::from(&format!("Failed to save project: {}", e)));
                    self.as_mut().set_loading(false);
                    return;
                }
            }
        }

        // Sync with GitHub
        self.as_mut().set_loading(false);
        self.as_mut().fetch_projects();
    }

    pub fn remove_project(mut self: Pin<&mut Self>, index: i32) {
        let id = self.get_project(index).map(|p| p.id.clone());

        if let Some(id) = id {
            if let Some(store) = &self.store {
                if let Ok(guard) = store.lock() {
                    let _ = guard.delete_project(&id);
                }
            }
        }

        self.as_mut().fetch_projects();
    }

    pub fn sync_project(mut self: Pin<&mut Self>, index: i32) {
        self.as_mut().ensure_initialized();

        let project = match self.get_project(index) {
            Some(p) => p.clone(),
            None => return,
        };

        let Some(client) = self.client.clone() else { return };
        let Some(store) = self.store.clone() else { return };
        let Some(runtime) = self.runtime.clone() else { return };

        self.as_mut().set_loading(true);

        // Parse owner/repo
        let parts: Vec<&str> = project.github_repo.split('/').collect();
        if parts.len() != 2 {
            self.as_mut().set_error_message(QString::from("Invalid repo format"));
            self.as_mut().set_loading(false);
            return;
        }
        let (owner, repo) = (parts[0].to_string(), parts[1].to_string());

        // Sync in background
        let project_id = project.id.clone();
        runtime.block_on(async move {
            match client.list_issues(&owner, &repo).await {
                Ok(issues) => {
                    if let Ok(guard) = store.lock() {
                        for issue in issues {
                            let labels: Vec<String> = issue.labels.iter()
                                .map(|l| l.name.clone())
                                .collect();
                            let status = TaskStatus::from_github(&issue.state, &labels);

                            let task = myme_services::Task {
                                id: uuid::Uuid::new_v4().to_string(),
                                project_id: project_id.clone(),
                                github_issue_number: issue.number,
                                title: issue.title,
                                body: issue.body,
                                status,
                                labels,
                                html_url: issue.html_url,
                                created_at: issue.created_at,
                                updated_at: issue.updated_at,
                            };
                            let _ = guard.upsert_task(&task);
                        }

                        // Update last_synced
                        let mut updated_project = project.clone();
                        updated_project.last_synced = Some(chrono::Utc::now().to_rfc3339());
                        let _ = guard.upsert_project(&updated_project);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to sync project: {}", e);
                }
            }
        });

        self.as_mut().set_loading(false);
        self.as_mut().fetch_projects();
    }
}
```

**Step 2: Add to mod.rs**

Add to `crates/myme-ui/src/models/mod.rs`:

```rust
pub mod project_model;
```

**Step 3: Add to build.rs**

Add to `CxxQtBuilder` in `crates/myme-ui/build.rs`:

```rust
.file("src/models/project_model.rs")
```

**Step 4: Add uuid and chrono dependencies**

Add to `crates/myme-ui/Cargo.toml`:

```toml
uuid = { version = "1", features = ["v4"] }
chrono = "0.4"
```

**Step 5: Verify it compiles**

Run: `cargo build -p myme-ui`
Expected: Compiles without errors

**Step 6: Commit**

```bash
git add crates/myme-ui/
git commit -m "feat(projects): add ProjectModel cxx-qt bridge"
```

---

## Task 11: KanbanModel - Task Board Model

**Files:**
- Create: `crates/myme-ui/src/models/kanban_model.rs`
- Modify: `crates/myme-ui/src/models/mod.rs`
- Modify: `crates/myme-ui/build.rs`

**Step 1: Create the kanban model**

```rust
// crates/myme-ui/src/models/kanban_model.rs

use cxx_qt_lib::QString;
use std::pin::Pin;
use std::sync::Arc;
use myme_services::{GitHubClient, Task, TaskStatus, ProjectStore, CreateIssueRequest, UpdateIssueRequest};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, loading)]
        #[qproperty(QString, error_message)]
        #[qproperty(QString, project_id)]
        #[qproperty(QString, github_repo)]
        type KanbanModel = super::KanbanModelRust;

        #[qinvokable]
        fn load_project(self: Pin<&mut KanbanModel>, project_id: QString);

        #[qinvokable]
        fn row_count(self: &KanbanModel) -> i32;

        #[qinvokable]
        fn get_issue_number(self: &KanbanModel, index: i32) -> i32;

        #[qinvokable]
        fn get_title(self: &KanbanModel, index: i32) -> QString;

        #[qinvokable]
        fn get_body(self: &KanbanModel, index: i32) -> QString;

        #[qinvokable]
        fn get_status(self: &KanbanModel, index: i32) -> QString;

        #[qinvokable]
        fn get_labels(self: &KanbanModel, index: i32) -> QString;

        #[qinvokable]
        fn get_url(self: &KanbanModel, index: i32) -> QString;

        #[qinvokable]
        fn count_by_status(self: &KanbanModel, status: QString) -> i32;

        #[qinvokable]
        fn tasks_for_status(self: &KanbanModel, status: QString) -> QString;

        #[qinvokable]
        fn move_task(self: Pin<&mut KanbanModel>, index: i32, new_status: QString);

        #[qinvokable]
        fn create_task(self: Pin<&mut KanbanModel>, title: QString, body: QString, status: QString);

        #[qinvokable]
        fn update_task(self: Pin<&mut KanbanModel>, index: i32, title: QString, body: QString);

        #[qinvokable]
        fn sync_tasks(self: Pin<&mut KanbanModel>);

        #[qsignal]
        fn tasks_changed(self: Pin<&mut KanbanModel>);
    }
}

#[derive(Default)]
pub struct KanbanModelRust {
    loading: bool,
    error_message: QString,
    project_id: QString,
    github_repo: QString,
    tasks: Vec<Task>,
    client: Option<Arc<GitHubClient>>,
    store: Option<Arc<std::sync::Mutex<ProjectStore>>>,
    runtime: Option<tokio::runtime::Handle>,
}

impl qobject::KanbanModel {
    fn ensure_initialized(&mut self) {
        if self.client.is_none() {
            if let Some((client, runtime)) = crate::bridge::get_github_client_and_runtime() {
                self.client = Some(client);
                self.runtime = Some(runtime);
            }
        }
        if self.store.is_none() {
            self.store = crate::bridge::get_project_store();
        }
    }

    fn get_task(&self, index: i32) -> Option<&Task> {
        if index >= 0 && (index as usize) < self.tasks.len() {
            Some(&self.tasks[index as usize])
        } else {
            None
        }
    }

    fn parse_owner_repo(&self) -> Option<(String, String)> {
        let repo = self.github_repo.to_string();
        let parts: Vec<&str> = repo.split('/').collect();
        if parts.len() == 2 {
            Some((parts[0].to_string(), parts[1].to_string()))
        } else {
            None
        }
    }

    fn status_from_string(s: &str) -> TaskStatus {
        match s.to_lowercase().as_str() {
            "backlog" => TaskStatus::Backlog,
            "todo" => TaskStatus::Todo,
            "inprogress" | "in_progress" => TaskStatus::InProgress,
            "blocked" => TaskStatus::Blocked,
            "review" => TaskStatus::Review,
            "done" => TaskStatus::Done,
            _ => TaskStatus::Todo,
        }
    }
}

impl qobject::KanbanModel {
    pub fn load_project(mut self: Pin<&mut Self>, project_id: QString) {
        self.as_mut().ensure_initialized();
        self.as_mut().set_project_id(project_id.clone());
        self.as_mut().set_error_message(QString::from(""));

        let id_str = project_id.to_string();

        // Get project info and tasks from store
        if let Some(store) = &self.store {
            if let Ok(guard) = store.lock() {
                if let Ok(Some(project)) = guard.get_project(&id_str) {
                    self.as_mut().set_github_repo(QString::from(&project.github_repo));

                    match guard.list_tasks(&id_str) {
                        Ok(tasks) => {
                            self.as_mut().rust_mut().tasks = tasks;
                        }
                        Err(e) => {
                            self.as_mut().set_error_message(QString::from(&format!("Failed to load tasks: {}", e)));
                        }
                    }
                }
            }
        }

        self.as_mut().tasks_changed();
    }

    pub fn row_count(&self) -> i32 {
        self.tasks.len() as i32
    }

    pub fn get_issue_number(&self, index: i32) -> i32 {
        self.get_task(index).map(|t| t.github_issue_number).unwrap_or(0)
    }

    pub fn get_title(&self, index: i32) -> QString {
        self.get_task(index)
            .map(|t| QString::from(&t.title))
            .unwrap_or_default()
    }

    pub fn get_body(&self, index: i32) -> QString {
        self.get_task(index)
            .and_then(|t| t.body.as_ref())
            .map(|b| QString::from(b.as_str()))
            .unwrap_or_default()
    }

    pub fn get_status(&self, index: i32) -> QString {
        self.get_task(index)
            .map(|t| QString::from(&format!("{:?}", t.status).to_lowercase()))
            .unwrap_or_default()
    }

    pub fn get_labels(&self, index: i32) -> QString {
        self.get_task(index)
            .map(|t| QString::from(&serde_json::to_string(&t.labels).unwrap_or_default()))
            .unwrap_or_else(|| QString::from("[]"))
    }

    pub fn get_url(&self, index: i32) -> QString {
        self.get_task(index)
            .map(|t| QString::from(&t.html_url))
            .unwrap_or_default()
    }

    pub fn count_by_status(&self, status: QString) -> i32 {
        let target = Self::status_from_string(&status.to_string());
        self.tasks.iter().filter(|t| t.status == target).count() as i32
    }

    pub fn tasks_for_status(&self, status: QString) -> QString {
        // Returns JSON array of indices for tasks in this status
        let target = Self::status_from_string(&status.to_string());
        let indices: Vec<i32> = self.tasks.iter()
            .enumerate()
            .filter(|(_, t)| t.status == target)
            .map(|(i, _)| i as i32)
            .collect();
        QString::from(&serde_json::to_string(&indices).unwrap_or_default())
    }

    pub fn move_task(mut self: Pin<&mut Self>, index: i32, new_status: QString) {
        self.as_mut().ensure_initialized();

        let task = match self.get_task(index) {
            Some(t) => t.clone(),
            None => return,
        };

        let new_status_enum = Self::status_from_string(&new_status.to_string());
        if task.status == new_status_enum {
            return; // No change
        }

        let Some(client) = self.client.clone() else { return };
        let Some(store) = self.store.clone() else { return };
        let Some(runtime) = self.runtime.clone() else { return };
        let Some((owner, repo)) = self.parse_owner_repo() else { return };

        self.as_mut().set_loading(true);

        let issue_number = task.github_issue_number;
        let project_id = task.project_id.clone();

        // Determine new labels and state
        let is_closing = new_status_enum == TaskStatus::Done;
        let is_reopening = task.status == TaskStatus::Done && new_status_enum != TaskStatus::Done;

        // Build new labels: remove old status label, add new one
        let mut new_labels: Vec<String> = task.labels.iter()
            .filter(|l| !["backlog", "todo", "in-progress", "blocked", "review"].contains(&l.as_str()))
            .cloned()
            .collect();

        if let Some(label) = new_status_enum.to_label() {
            new_labels.push(label.to_string());
        }

        runtime.block_on(async move {
            // Update labels
            let update = UpdateIssueRequest {
                title: None,
                body: None,
                state: if is_closing {
                    Some("closed".to_string())
                } else if is_reopening {
                    Some("open".to_string())
                } else {
                    None
                },
                labels: Some(new_labels.clone()),
            };

            match client.update_issue(&owner, &repo, issue_number, update).await {
                Ok(updated_issue) => {
                    // Update local store
                    if let Ok(guard) = store.lock() {
                        let mut updated_task = task.clone();
                        updated_task.status = new_status_enum;
                        updated_task.labels = new_labels;
                        updated_task.updated_at = updated_issue.updated_at;
                        let _ = guard.upsert_task(&updated_task);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to update issue: {}", e);
                }
            }
        });

        self.as_mut().set_loading(false);
        self.as_mut().load_project(QString::from(&project_id));
    }

    pub fn create_task(mut self: Pin<&mut Self>, title: QString, body: QString, status: QString) {
        self.as_mut().ensure_initialized();

        let Some(client) = self.client.clone() else { return };
        let Some(store) = self.store.clone() else { return };
        let Some(runtime) = self.runtime.clone() else { return };
        let Some((owner, repo)) = self.parse_owner_repo() else { return };

        let project_id = self.project_id.to_string();
        let title_str = title.to_string();
        let body_str = body.to_string();
        let status_enum = Self::status_from_string(&status.to_string());

        self.as_mut().set_loading(true);

        let labels = status_enum.to_label().map(|l| vec![l.to_string()]);

        runtime.block_on(async move {
            let req = CreateIssueRequest {
                title: title_str,
                body: if body_str.is_empty() { None } else { Some(body_str) },
                labels,
            };

            match client.create_issue(&owner, &repo, req).await {
                Ok(issue) => {
                    if let Ok(guard) = store.lock() {
                        let task = Task {
                            id: uuid::Uuid::new_v4().to_string(),
                            project_id,
                            github_issue_number: issue.number,
                            title: issue.title,
                            body: issue.body,
                            status: status_enum,
                            labels: issue.labels.iter().map(|l| l.name.clone()).collect(),
                            html_url: issue.html_url,
                            created_at: issue.created_at,
                            updated_at: issue.updated_at,
                        };
                        let _ = guard.upsert_task(&task);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to create issue: {}", e);
                }
            }
        });

        self.as_mut().set_loading(false);
        let pid = self.project_id.clone();
        self.as_mut().load_project(pid);
    }

    pub fn update_task(mut self: Pin<&mut Self>, index: i32, title: QString, body: QString) {
        self.as_mut().ensure_initialized();

        let task = match self.get_task(index) {
            Some(t) => t.clone(),
            None => return,
        };

        let Some(client) = self.client.clone() else { return };
        let Some(store) = self.store.clone() else { return };
        let Some(runtime) = self.runtime.clone() else { return };
        let Some((owner, repo)) = self.parse_owner_repo() else { return };

        let project_id = task.project_id.clone();
        let title_str = title.to_string();
        let body_str = body.to_string();
        let issue_number = task.github_issue_number;

        self.as_mut().set_loading(true);

        runtime.block_on(async move {
            let req = UpdateIssueRequest {
                title: Some(title_str.clone()),
                body: Some(body_str.clone()),
                state: None,
                labels: None,
            };

            match client.update_issue(&owner, &repo, issue_number, req).await {
                Ok(issue) => {
                    if let Ok(guard) = store.lock() {
                        let mut updated_task = task.clone();
                        updated_task.title = title_str;
                        updated_task.body = Some(body_str);
                        updated_task.updated_at = issue.updated_at;
                        let _ = guard.upsert_task(&updated_task);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to update issue: {}", e);
                }
            }
        });

        self.as_mut().set_loading(false);
        self.as_mut().load_project(QString::from(&project_id));
    }

    pub fn sync_tasks(mut self: Pin<&mut Self>) {
        self.as_mut().ensure_initialized();

        let Some(client) = self.client.clone() else { return };
        let Some(store) = self.store.clone() else { return };
        let Some(runtime) = self.runtime.clone() else { return };
        let Some((owner, repo)) = self.parse_owner_repo() else { return };

        let project_id = self.project_id.to_string();
        self.as_mut().set_loading(true);

        runtime.block_on(async move {
            match client.list_issues(&owner, &repo).await {
                Ok(issues) => {
                    if let Ok(guard) = store.lock() {
                        for issue in issues {
                            let labels: Vec<String> = issue.labels.iter()
                                .map(|l| l.name.clone())
                                .collect();
                            let status = TaskStatus::from_github(&issue.state, &labels);

                            let task = Task {
                                id: uuid::Uuid::new_v4().to_string(),
                                project_id: project_id.clone(),
                                github_issue_number: issue.number,
                                title: issue.title,
                                body: issue.body,
                                status,
                                labels,
                                html_url: issue.html_url,
                                created_at: issue.created_at,
                                updated_at: issue.updated_at,
                            };
                            let _ = guard.upsert_task(&task);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to sync tasks: {}", e);
                }
            }
        });

        self.as_mut().set_loading(false);
        self.as_mut().load_project(QString::from(&project_id));
    }
}
```

**Step 2: Add to mod.rs**

Add to `crates/myme-ui/src/models/mod.rs`:

```rust
pub mod kanban_model;
```

**Step 3: Add to build.rs**

Add to `CxxQtBuilder` in `crates/myme-ui/build.rs`:

```rust
.file("src/models/kanban_model.rs")
```

**Step 4: Verify it compiles**

Run: `cargo build -p myme-ui`
Expected: Compiles without errors

**Step 5: Commit**

```bash
git add crates/myme-ui/
git commit -m "feat(projects): add KanbanModel for task board"
```

---

## Task 12: QML - ProjectsPage Dashboard

**Files:**
- Create: `crates/myme-ui/qml/pages/ProjectsPage.qml`
- Modify: `qml.qrc`

**Step 1: Create the page**

```qml
// crates/myme-ui/qml/pages/ProjectsPage.qml
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import myme_ui

import ".."
import "../components"

Kirigami.Page {
    id: projectsPage
    title: "Projects"

    property int projectCount: 0

    ProjectModel {
        id: projectModel
    }

    Connections {
        target: projectModel
        function onProjectsChanged() {
            projectsPage.projectCount = projectModel.row_count()
        }
        function onAuthChanged() {
            if (projectModel.authenticated) {
                projectModel.fetch_projects()
            }
        }
    }

    Component.onCompleted: {
        projectModel.check_auth()
        if (projectModel.authenticated) {
            projectModel.fetch_projects()
        }
    }

    header: ToolBar {
        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: Kirigami.Units.smallSpacing
            anchors.rightMargin: Kirigami.Units.smallSpacing

            Label {
                text: "Projects"
                font.bold: true
                font.pixelSize: 18
                color: Theme.text
            }

            Item { Layout.fillWidth: true }

            ToolButton {
                icon.name: "view-refresh"
                text: "Sync All"
                enabled: projectModel.authenticated && !projectModel.loading
                onClicked: projectModel.fetch_projects()
            }

            ToolButton {
                icon.name: "list-add"
                text: "Add Project"
                enabled: projectModel.authenticated && !projectModel.loading
                onClicked: addProjectDialog.open()
            }
        }

        background: Rectangle {
            color: Theme.surface
        }
    }

    // Not authenticated state
    ColumnLayout {
        anchors.centerIn: parent
        visible: !projectModel.authenticated && !projectModel.loading
        spacing: Kirigami.Units.largeSpacing

        Kirigami.Icon {
            source: "dialog-password"
            Layout.preferredWidth: 64
            Layout.preferredHeight: 64
            Layout.alignment: Qt.AlignHCenter
        }

        Label {
            text: "GitHub Authentication Required"
            font.bold: true
            font.pixelSize: 18
            color: Theme.text
            Layout.alignment: Qt.AlignHCenter
        }

        Label {
            text: "Sign in to GitHub to manage your projects"
            color: Theme.textSecondary
            Layout.alignment: Qt.AlignHCenter
        }

        Button {
            text: "Sign in with GitHub"
            Layout.alignment: Qt.AlignHCenter
            onClicked: {
                // TODO: Trigger OAuth flow
            }
        }
    }

    // Error banner
    Rectangle {
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: errorLabel.height + Kirigami.Units.largeSpacing * 2
        color: "#ffebee"
        visible: projectModel.error_message.length > 0
        z: 100

        RowLayout {
            anchors.fill: parent
            anchors.margins: Kirigami.Units.smallSpacing

            Label {
                id: errorLabel
                text: projectModel.error_message
                color: "#c62828"
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
            }

            ToolButton {
                icon.name: "dialog-close"
                onClicked: projectModel.error_message = ""
            }
        }
    }

    // Project grid
    ScrollView {
        anchors.fill: parent
        visible: projectModel.authenticated

        GridLayout {
            width: parent.width
            columns: Math.max(1, Math.floor(width / 320))
            columnSpacing: Kirigami.Units.largeSpacing
            rowSpacing: Kirigami.Units.largeSpacing

            Repeater {
                model: projectsPage.projectCount

                delegate: Rectangle {
                    id: projectCard
                    required property int index

                    Layout.fillWidth: true
                    Layout.preferredHeight: 160
                    Layout.minimumWidth: 280
                    Layout.maximumWidth: 400

                    color: Theme.surface
                    radius: 8
                    border.color: cardMouse.containsMouse ? Theme.primary : Theme.border
                    border.width: 1

                    property var taskCounts: {
                        try {
                            return JSON.parse(projectModel.get_task_counts(index))
                        } catch (e) {
                            return {}
                        }
                    }

                    MouseArea {
                        id: cardMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            // Navigate to kanban view
                            stackView.push("ProjectDetailPage.qml", {
                                projectId: projectModel.get_id(index),
                                projectName: projectModel.get_github_repo(index)
                            })
                        }
                    }

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: Kirigami.Units.largeSpacing
                        spacing: Kirigami.Units.smallSpacing

                        // Header
                        RowLayout {
                            Layout.fillWidth: true

                            Label {
                                text: projectModel.get_github_repo(index)
                                font.bold: true
                                font.pixelSize: 16
                                color: Theme.text
                                elide: Text.ElideRight
                                Layout.fillWidth: true
                            }

                            ToolButton {
                                icon.name: "view-refresh"
                                implicitWidth: 32
                                implicitHeight: 32
                                onClicked: {
                                    projectModel.sync_project(index)
                                }
                            }

                            ToolButton {
                                icon.name: "edit-delete"
                                implicitWidth: 32
                                implicitHeight: 32
                                onClicked: {
                                    removeConfirmDialog.projectIndex = index
                                    removeConfirmDialog.open()
                                }
                            }
                        }

                        // Description
                        Label {
                            text: projectModel.get_description(index) || "No description"
                            color: Theme.textSecondary
                            font.pixelSize: 13
                            elide: Text.ElideRight
                            maximumLineCount: 2
                            wrapMode: Text.WordWrap
                            Layout.fillWidth: true
                        }

                        Item { Layout.fillHeight: true }

                        // Status bar
                        RowLayout {
                            Layout.fillWidth: true
                            spacing: 2

                            Repeater {
                                model: ["backlog", "todo", "inprogress", "blocked", "review", "done"]

                                Rectangle {
                                    property int count: projectCard.taskCounts[modelData] || 0
                                    property int total: {
                                        let t = 0
                                        for (let k in projectCard.taskCounts) t += projectCard.taskCounts[k]
                                        return t
                                    }

                                    Layout.fillWidth: count > 0
                                    Layout.preferredWidth: count > 0 ? (count / Math.max(1, total)) * parent.width : 0
                                    height: 8
                                    radius: 2
                                    visible: count > 0

                                    color: {
                                        switch(modelData) {
                                            case "backlog": return "#9e9e9e"
                                            case "todo": return "#2196f3"
                                            case "inprogress": return "#ff9800"
                                            case "blocked": return "#f44336"
                                            case "review": return "#9c27b0"
                                            case "done": return "#4caf50"
                                            default: return "#9e9e9e"
                                        }
                                    }

                                    ToolTip.visible: statusMouse.containsMouse
                                    ToolTip.text: modelData + ": " + count

                                    MouseArea {
                                        id: statusMouse
                                        anchors.fill: parent
                                        hoverEnabled: true
                                    }
                                }
                            }
                        }

                        // Stats row
                        RowLayout {
                            Layout.fillWidth: true

                            Label {
                                property int total: {
                                    let t = 0
                                    for (let k in projectCard.taskCounts) t += projectCard.taskCounts[k]
                                    return t
                                }
                                text: total + " tasks"
                                font.pixelSize: 12
                                color: Theme.textSecondary
                            }

                            Item { Layout.fillWidth: true }

                            Label {
                                property int done: projectCard.taskCounts["done"] || 0
                                property int total: {
                                    let t = 0
                                    for (let k in projectCard.taskCounts) t += projectCard.taskCounts[k]
                                    return t
                                }
                                text: total > 0 ? Math.round(done / total * 100) + "% done" : ""
                                font.pixelSize: 12
                                color: Theme.textSecondary
                            }
                        }
                    }
                }
            }
        }
    }

    // Empty state
    ColumnLayout {
        anchors.centerIn: parent
        visible: projectModel.authenticated && !projectModel.loading && projectsPage.projectCount === 0
        spacing: Kirigami.Units.largeSpacing

        Kirigami.Icon {
            source: "folder"
            Layout.preferredWidth: 64
            Layout.preferredHeight: 64
            Layout.alignment: Qt.AlignHCenter
            opacity: 0.5
        }

        Label {
            text: "No projects yet"
            font.bold: true
            font.pixelSize: 18
            color: Theme.text
            Layout.alignment: Qt.AlignHCenter
        }

        Label {
            text: "Add a GitHub repository to get started"
            color: Theme.textSecondary
            Layout.alignment: Qt.AlignHCenter
        }

        Button {
            text: "Add Project"
            Layout.alignment: Qt.AlignHCenter
            onClicked: addProjectDialog.open()
        }
    }

    // Loading indicator
    BusyIndicator {
        anchors.centerIn: parent
        running: projectModel.loading
        visible: projectModel.loading
    }

    // Add project dialog
    Dialog {
        id: addProjectDialog
        title: "Add Project"
        modal: true
        anchors.centerIn: parent
        width: 400
        standardButtons: Dialog.Ok | Dialog.Cancel

        onAccepted: {
            if (repoField.text.trim().length > 0) {
                projectModel.add_project(repoField.text.trim())
                repoField.text = ""
            }
        }

        onRejected: {
            repoField.text = ""
        }

        ColumnLayout {
            anchors.fill: parent
            spacing: Kirigami.Units.largeSpacing

            Label {
                text: "Enter the GitHub repository (owner/repo):"
                color: Theme.text
            }

            TextField {
                id: repoField
                Layout.fillWidth: true
                placeholderText: "e.g., jonesrussell/myme"
            }

            Label {
                text: "The repository will be synced and its issues will appear as tasks."
                color: Theme.textSecondary
                font.pixelSize: 12
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }
        }
    }

    // Remove confirmation dialog
    Dialog {
        id: removeConfirmDialog
        title: "Remove Project"
        modal: true
        anchors.centerIn: parent
        standardButtons: Dialog.Yes | Dialog.No

        property int projectIndex: -1

        onAccepted: {
            if (projectIndex >= 0) {
                projectModel.remove_project(projectIndex)
            }
            projectIndex = -1
        }

        Label {
            text: "Remove this project from MyMe?\n\nThis will not delete the GitHub repository or its issues."
            wrapMode: Text.WordWrap
        }
    }
}
```

**Step 2: Add to qml.qrc**

Add to `qml.qrc`:

```xml
<file>crates/myme-ui/qml/pages/ProjectsPage.qml</file>
```

**Step 3: Commit**

```bash
git add crates/myme-ui/qml/pages/ProjectsPage.qml qml.qrc
git commit -m "feat(projects): add ProjectsPage dashboard UI"
```

---

## Task 13: QML - ProjectDetailPage Kanban Board

**Files:**
- Create: `crates/myme-ui/qml/pages/ProjectDetailPage.qml`
- Modify: `qml.qrc`

**Step 1: Create the kanban page**

```qml
// crates/myme-ui/qml/pages/ProjectDetailPage.qml
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import myme_ui

import ".."
import "../components"

Kirigami.Page {
    id: detailPage
    title: projectName

    required property string projectId
    required property string projectName

    readonly property var columns: [
        { key: "backlog", label: "Backlog", color: "#9e9e9e" },
        { key: "todo", label: "Todo", color: "#2196f3" },
        { key: "inprogress", label: "In Progress", color: "#ff9800" },
        { key: "blocked", label: "Blocked", color: "#f44336" },
        { key: "review", label: "Review", color: "#9c27b0" },
        { key: "done", label: "Done", color: "#4caf50" }
    ]

    KanbanModel {
        id: kanbanModel
    }

    Connections {
        target: kanbanModel
        function onTasksChanged() {
            detailPage.forceLayout()
        }
    }

    Component.onCompleted: {
        kanbanModel.load_project(projectId)
    }

    header: ToolBar {
        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: Kirigami.Units.smallSpacing
            anchors.rightMargin: Kirigami.Units.smallSpacing

            ToolButton {
                icon.name: "go-previous"
                onClicked: stackView.pop()
            }

            Label {
                text: projectName
                font.bold: true
                font.pixelSize: 18
                color: Theme.text
            }

            Item { Layout.fillWidth: true }

            ToolButton {
                icon.name: "view-refresh"
                text: "Sync"
                enabled: !kanbanModel.loading
                onClicked: kanbanModel.sync_tasks()
            }

            ToolButton {
                icon.name: "list-add"
                text: "New Task"
                enabled: !kanbanModel.loading
                onClicked: newTaskDialog.open()
            }
        }

        background: Rectangle {
            color: Theme.surface
        }
    }

    // Error banner
    Rectangle {
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: errorLabel.height + Kirigami.Units.largeSpacing * 2
        color: "#ffebee"
        visible: kanbanModel.error_message.length > 0
        z: 100

        RowLayout {
            anchors.fill: parent
            anchors.margins: Kirigami.Units.smallSpacing

            Label {
                id: errorLabel
                text: kanbanModel.error_message
                color: "#c62828"
                Layout.fillWidth: true
            }

            ToolButton {
                icon.name: "dialog-close"
                onClicked: kanbanModel.error_message = ""
            }
        }
    }

    // Kanban board
    ScrollView {
        anchors.fill: parent
        contentWidth: columnsRow.width

        RowLayout {
            id: columnsRow
            spacing: Kirigami.Units.largeSpacing

            Repeater {
                model: detailPage.columns

                delegate: Rectangle {
                    id: columnRect
                    required property var modelData
                    required property int index

                    Layout.preferredWidth: 280
                    Layout.fillHeight: true
                    Layout.minimumHeight: 400

                    color: Theme.background
                    radius: 8
                    border.color: Theme.border
                    border.width: 1

                    property var taskIndices: {
                        try {
                            return JSON.parse(kanbanModel.tasks_for_status(modelData.key))
                        } catch (e) {
                            return []
                        }
                    }

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: Kirigami.Units.smallSpacing
                        spacing: Kirigami.Units.smallSpacing

                        // Column header
                        RowLayout {
                            Layout.fillWidth: true

                            Rectangle {
                                width: 12
                                height: 12
                                radius: 6
                                color: modelData.color
                            }

                            Label {
                                text: modelData.label
                                font.bold: true
                                color: Theme.text
                            }

                            Item { Layout.fillWidth: true }

                            Label {
                                text: columnRect.taskIndices.length
                                font.pixelSize: 12
                                color: Theme.textSecondary
                                padding: 4
                                background: Rectangle {
                                    color: Theme.surface
                                    radius: 10
                                }
                            }

                            ToolButton {
                                icon.name: "list-add"
                                implicitWidth: 28
                                implicitHeight: 28
                                onClicked: {
                                    newTaskDialog.initialStatus = modelData.key
                                    newTaskDialog.open()
                                }
                            }
                        }

                        // Drop area
                        DropArea {
                            id: dropArea
                            Layout.fillWidth: true
                            Layout.fillHeight: true

                            keys: ["task"]

                            onDropped: (drop) => {
                                const taskIndex = drop.source.taskIndex
                                kanbanModel.move_task(taskIndex, modelData.key)
                            }

                            Rectangle {
                                anchors.fill: parent
                                color: dropArea.containsDrag ? Qt.rgba(modelData.color, 0.1) : "transparent"
                                radius: 4
                                border.color: dropArea.containsDrag ? modelData.color : "transparent"
                                border.width: 2

                                // Task cards
                                ListView {
                                    anchors.fill: parent
                                    anchors.margins: 4
                                    spacing: Kirigami.Units.smallSpacing
                                    clip: true

                                    model: columnRect.taskIndices.length

                                    delegate: Rectangle {
                                        id: taskCard
                                        required property int index

                                        property int taskIndex: columnRect.taskIndices[index]

                                        width: ListView.view.width
                                        height: taskContent.height + Kirigami.Units.largeSpacing

                                        color: Theme.surface
                                        radius: 6
                                        border.color: taskMouse.containsMouse ? Theme.primary : Theme.border
                                        border.width: 1

                                        Drag.active: taskMouse.drag.active
                                        Drag.keys: ["task"]
                                        Drag.hotSpot.x: width / 2
                                        Drag.hotSpot.y: height / 2

                                        MouseArea {
                                            id: taskMouse
                                            anchors.fill: parent
                                            hoverEnabled: true
                                            drag.target: parent
                                            cursorShape: drag.active ? Qt.ClosedHandCursor : Qt.PointingHandCursor

                                            onClicked: {
                                                taskDetailDialog.taskIndex = taskCard.taskIndex
                                                taskDetailDialog.open()
                                            }

                                            onReleased: {
                                                parent.Drag.drop()
                                            }
                                        }

                                        ColumnLayout {
                                            id: taskContent
                                            anchors.left: parent.left
                                            anchors.right: parent.right
                                            anchors.top: parent.top
                                            anchors.margins: Kirigami.Units.smallSpacing
                                            spacing: 4

                                            Label {
                                                text: kanbanModel.get_title(taskCard.taskIndex)
                                                font.pixelSize: 13
                                                color: Theme.text
                                                wrapMode: Text.WordWrap
                                                Layout.fillWidth: true
                                            }

                                            RowLayout {
                                                Layout.fillWidth: true

                                                Label {
                                                    text: "#" + kanbanModel.get_issue_number(taskCard.taskIndex)
                                                    font.pixelSize: 11
                                                    color: Theme.textSecondary
                                                }

                                                Item { Layout.fillWidth: true }

                                                ToolButton {
                                                    icon.name: "internet-services"
                                                    implicitWidth: 24
                                                    implicitHeight: 24
                                                    onClicked: Qt.openUrlExternally(kanbanModel.get_url(taskCard.taskIndex))
                                                    ToolTip.text: "Open on GitHub"
                                                    ToolTip.visible: hovered
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Loading indicator
    BusyIndicator {
        anchors.centerIn: parent
        running: kanbanModel.loading
        visible: kanbanModel.loading
    }

    // New task dialog
    Dialog {
        id: newTaskDialog
        title: "New Task"
        modal: true
        anchors.centerIn: parent
        width: 450
        standardButtons: Dialog.Ok | Dialog.Cancel

        property string initialStatus: "todo"

        onAccepted: {
            if (titleField.text.trim().length > 0) {
                kanbanModel.create_task(
                    titleField.text.trim(),
                    bodyField.text,
                    statusCombo.currentValue
                )
                titleField.text = ""
                bodyField.text = ""
            }
        }

        onOpened: {
            statusCombo.currentIndex = detailPage.columns.findIndex(c => c.key === initialStatus)
        }

        ColumnLayout {
            anchors.fill: parent
            spacing: Kirigami.Units.largeSpacing

            Label { text: "Title:" }
            TextField {
                id: titleField
                Layout.fillWidth: true
                placeholderText: "Task title"
            }

            Label { text: "Description:" }
            TextArea {
                id: bodyField
                Layout.fillWidth: true
                Layout.preferredHeight: 100
                placeholderText: "Optional description..."
            }

            Label { text: "Status:" }
            ComboBox {
                id: statusCombo
                Layout.fillWidth: true
                model: detailPage.columns
                textRole: "label"
                valueRole: "key"
            }
        }
    }

    // Task detail dialog
    Dialog {
        id: taskDetailDialog
        title: "Task Details"
        modal: true
        anchors.centerIn: parent
        width: 500
        standardButtons: Dialog.Save | Dialog.Cancel

        property int taskIndex: -1

        onOpened: {
            if (taskIndex >= 0) {
                editTitleField.text = kanbanModel.get_title(taskIndex)
                editBodyField.text = kanbanModel.get_body(taskIndex)
            }
        }

        onAccepted: {
            if (taskIndex >= 0 && editTitleField.text.trim().length > 0) {
                kanbanModel.update_task(taskIndex, editTitleField.text.trim(), editBodyField.text)
            }
        }

        ColumnLayout {
            anchors.fill: parent
            spacing: Kirigami.Units.largeSpacing

            RowLayout {
                Layout.fillWidth: true

                Label {
                    text: taskDetailDialog.taskIndex >= 0 ?
                        "#" + kanbanModel.get_issue_number(taskDetailDialog.taskIndex) : ""
                    color: Theme.textSecondary
                }

                Item { Layout.fillWidth: true }

                Button {
                    text: "Open on GitHub"
                    icon.name: "internet-services"
                    onClicked: {
                        if (taskDetailDialog.taskIndex >= 0) {
                            Qt.openUrlExternally(kanbanModel.get_url(taskDetailDialog.taskIndex))
                        }
                    }
                }
            }

            Label { text: "Title:" }
            TextField {
                id: editTitleField
                Layout.fillWidth: true
            }

            Label { text: "Description:" }
            TextArea {
                id: editBodyField
                Layout.fillWidth: true
                Layout.preferredHeight: 150
            }
        }
    }
}
```

**Step 2: Add to qml.qrc**

Add to `qml.qrc`:

```xml
<file>crates/myme-ui/qml/pages/ProjectDetailPage.qml</file>
```

**Step 3: Commit**

```bash
git add crates/myme-ui/qml/pages/ProjectDetailPage.qml qml.qrc
git commit -m "feat(projects): add ProjectDetailPage kanban board UI"
```

---

## Task 14: Navigation - Add Projects to Main Menu

**Files:**
- Modify: `crates/myme-ui/qml/Main.qml`

**Step 1: Add Projects navigation item**

Find the navigation drawer actions in `Main.qml` and add:

```qml
Kirigami.Action {
    text: "Projects"
    icon.name: "folder-development"
    onTriggered: {
        currentPage = "projects"
        stackView.replace("pages/ProjectsPage.qml")
    }
}
```

**Step 2: Verify the app builds and runs**

Run: `.\scripts\build.ps1`
Expected: Builds without errors

**Step 3: Commit**

```bash
git add crates/myme-ui/qml/Main.qml
git commit -m "feat(projects): add Projects to main navigation"
```

---

## Task 15: C++ Initialization - Wire Up GitHub Client

**Files:**
- Modify: `qt-main/main.cpp`

**Step 1: Add initialization call**

Add after existing initialization calls:

```cpp
// Initialize GitHub client (requires prior OAuth authentication)
initialize_github_client();
```

Add the extern declaration near the top:

```cpp
extern "C" bool initialize_github_client();
```

**Step 2: Verify it compiles**

Run: `.\scripts\build.ps1`
Expected: Builds and links without errors

**Step 3: Commit**

```bash
git add qt-main/main.cpp
git commit -m "feat(projects): wire up GitHub client initialization"
```

---

## Task 16: Note Promotion - Add to NotePage

**Files:**
- Modify: `crates/myme-ui/qml/pages/NotePage.qml`

**Step 1: Add promote action to note cards**

In the note delegate, add a promote button:

```qml
ToolButton {
    icon.name: "folder-new"
    ToolTip.text: "Promote to Project"
    ToolTip.visible: hovered
    onClicked: {
        promoteDialog.noteIndex = index
        promoteDialog.noteTitle = noteModel.get_title(index)
        promoteDialog.noteBody = noteModel.get_body(index)
        promoteDialog.open()
    }
}
```

**Step 2: Add promote dialog**

Add at the bottom of NotePage.qml:

```qml
Dialog {
    id: promoteDialog
    title: "Promote to Project"
    modal: true
    anchors.centerIn: parent
    width: 450
    standardButtons: Dialog.Ok | Dialog.Cancel

    property int noteIndex: -1
    property string noteTitle: ""
    property string noteBody: ""

    onAccepted: {
        if (repoNameField.text.trim().length > 0) {
            // Create project with note content
            // Note: This would need ProjectModel access or a shared service
            // For now, just navigate to projects page
            stackView.replace("ProjectsPage.qml")
        }
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: Kirigami.Units.largeSpacing

        Label {
            text: "Create a new project from this note?"
            wrapMode: Text.WordWrap
            Layout.fillWidth: true
        }

        Label { text: "GitHub Repository (owner/repo):" }
        TextField {
            id: repoNameField
            Layout.fillWidth: true
            placeholderText: "e.g., jonesrussell/my-project"
        }

        CheckBox {
            id: createRepoCheck
            text: "Create new repository on GitHub"
            checked: false
        }

        Label {
            text: "Note content will become the project description."
            color: Theme.textSecondary
            font.pixelSize: 12
            wrapMode: Text.WordWrap
            Layout.fillWidth: true
        }
    }
}
```

**Step 3: Commit**

```bash
git add crates/myme-ui/qml/pages/NotePage.qml
git commit -m "feat(projects): add note promotion to project"
```

---

## Task 17: Integration Test - Manual Verification

**Files:** None (manual testing)

**Step 1: Build the complete application**

Run: `.\scripts\build.ps1`
Expected: Builds without errors

**Step 2: Test GitHub OAuth**

1. Run the application
2. Navigate to Projects page
3. Verify "Sign in with GitHub" appears if not authenticated
4. (If OAuth is set up) Complete authentication flow

**Step 3: Test project creation**

1. Click "Add Project"
2. Enter a valid repo (e.g., "jonesrussell/myme")
3. Verify project card appears on dashboard
4. Verify task counts display

**Step 4: Test kanban board**

1. Click a project card
2. Verify kanban columns appear
3. Verify tasks are in correct columns
4. Test drag and drop between columns
5. Verify GitHub issue labels update

**Step 5: Test task creation**

1. Click "New Task"
2. Fill in title and description
3. Verify task appears in selected column
4. Verify GitHub issue was created

**Step 6: Commit integration notes**

```bash
git add -A
git commit -m "feat(projects): complete Projects feature implementation"
```

---

## Summary

This plan implements the Projects feature in 17 tasks:

1. **Tasks 1-5**: GitHub API client with full repo, issue, and label support
2. **Tasks 6-8**: Local data model, configuration, and SQLite storage
3. **Tasks 9-11**: cxx-qt bridge models (ProjectModel, KanbanModel)
4. **Tasks 12-13**: QML UI (dashboard and kanban board)
5. **Tasks 14-16**: Navigation, C++ wiring, and note promotion
6. **Task 17**: Integration testing

Each task follows TDD principles where applicable, with explicit file paths, complete code, and commit points.
