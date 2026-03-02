# Data Services Specification

Covers `crates/myme-services/`, `crates/myme-integrations/`, and `crates/myme-organizations/`.

## File Map

| File | Purpose |
|------|---------|
| `crates/myme-services/src/lib.rs` | Re-exports: `GitHubClient`, `NoteClient`, `SqliteNoteStore`, `ProjectStore`, `Todo`, retry utils |
| `crates/myme-services/src/todo.rs` | `Todo`, `TodoCreateRequest`, `TodoUpdateRequest` data types |
| `crates/myme-services/src/note_client.rs` | Async `NoteClient` wrapping `SqliteNoteStore` via `spawn_blocking` |
| `crates/myme-services/src/note_store.rs` | `SqliteNoteStore` - raw SQLite note CRUD |
| `crates/myme-services/src/note_backend.rs` | `NoteBackend` trait, `NoteBackendError` |
| `crates/myme-services/src/github.rs` | `GitHubClient` - GitHub REST API with retry logic |
| `crates/myme-services/src/retry.rs` | `RetryConfig`, `with_retry()`, exponential backoff |
| `crates/myme-services/src/project.rs` | `Project`, `Task`, `TaskStatus` (kanban), `ProjectRepo` |
| `crates/myme-services/src/project_store.rs` | `ProjectStore` - SQLite for projects/tasks with schema migrations |
| `crates/myme-integrations/src/lib.rs` | Re-exports: `GitOperations`, `LocalRepo`, `GitHubClient` (integrations), `RepoEntry` |
| `crates/myme-integrations/src/github/mod.rs` | `GitHubClient` (integrations), `Repository`, `Issue`, `User`, `Label` |
| `crates/myme-integrations/src/git/mod.rs` | `GitOperations` - local git via git2: discover, clone, fetch, pull, push |
| `crates/myme-integrations/src/repo.rs` | `RepoEntry`, `RepoId`, `RepoState`, `match_repos()` |
| `crates/myme-integrations/src/repo_url.rs` | `normalize_github_url()` |
| `crates/myme-organizations/src/lib.rs` | Re-exports: `Organization`, `Prospect`, `ProspectStage`, `OrganizationStore` |
| `crates/myme-organizations/src/models.rs` | `Organization`, `Prospect`, `ProspectStage` data types |
| `crates/myme-organizations/src/store.rs` | `OrganizationStore` - SQLite for organizations/prospects |

## Interface Signatures

### myme-services::todo (Data Types)

```rust
pub struct Todo {
    pub id: i64,
    pub content: String,
    pub done: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub color: Option<String>,
    pub pinned: bool,
    pub archived: bool,
    pub labels: Vec<String>,
    pub is_checklist: bool,
    pub reminder: Option<DateTime<Utc>>,
}

pub struct TodoCreateRequest { pub content: String, pub is_checklist: bool }

pub struct TodoUpdateRequest {
    pub content: Option<String>,
    pub color: Option<Option<String>>,  // Some(None) = clear color
    pub pinned: Option<bool>,
    pub archived: Option<bool>,
    pub labels: Option<Vec<String>>,
    pub is_checklist: Option<bool>,
    pub reminder: Option<Option<DateTime<Utc>>>,
    pub done: Option<bool>,
}
```

### myme-services::note_client

```rust
pub struct NoteClient(Arc<Mutex<SqliteNoteStore>>);  // parking_lot::Mutex
impl NoteClient {
    pub fn sqlite(store: SqliteNoteStore) -> Self;
    pub async fn list_todos(&self) -> Result<Vec<Todo>>;         // pinned first, then updated_at DESC
    pub async fn list_archived(&self) -> Result<Vec<Todo>>;
    pub async fn list_by_label(&self, label: &str) -> Result<Vec<Todo>>;
    pub async fn list_with_reminders(&self) -> Result<Vec<Todo>>;
    pub async fn get_todo(&self, id: i64) -> Result<Todo>;
    pub async fn create_todo(&self, request: TodoCreateRequest) -> Result<Todo>;
    pub async fn update_todo(&self, id: i64, request: TodoUpdateRequest) -> Result<Todo>;
    pub async fn delete_todo(&self, id: i64) -> Result<()>;
    pub async fn mark_done(&self, id: i64) -> Result<Todo>;
    pub async fn mark_undone(&self, id: i64) -> Result<Todo>;
    pub async fn toggle_done(&self, id: i64) -> Result<Todo>;
    pub async fn health_check(&self) -> Result<bool>;            // always true for SQLite
    pub fn sqlite_store(&self) -> Arc<Mutex<SqliteNoteStore>>;
}
```

### myme-services::github (GitHubClient with retry)

```rust
pub struct GitHubClient { base_url: Url, client: Arc<Client>, token: String, retry_config: RetryConfig }
impl GitHubClient {
    pub fn new(token: String) -> Result<Self>;            // 30s timeout
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self;
    pub async fn list_repos(&self) -> Result<Vec<GitHubRepo>>;
    pub async fn get_repo(&self, owner: &str, repo: &str) -> Result<GitHubRepo>;
    pub async fn create_repo(&self, req: CreateRepoRequest) -> Result<GitHubRepo>;
    pub async fn list_issues(&self, owner: &str, repo: &str) -> Result<Vec<GitHubIssue>>;
    pub async fn list_issues_since(&self, owner: &str, repo: &str, since: &str) -> Result<Vec<GitHubIssue>>;
    pub async fn create_issue(&self, owner: &str, repo: &str, req: CreateIssueRequest) -> Result<GitHubIssue>;
    pub async fn update_issue(&self, owner: &str, repo: &str, issue_number: i32, req: UpdateIssueRequest) -> Result<GitHubIssue>;
    pub async fn close_issue(&self, owner: &str, repo: &str, issue_number: i32) -> Result<GitHubIssue>;
    pub async fn reopen_issue(&self, owner: &str, repo: &str, issue_number: i32) -> Result<GitHubIssue>;
    pub async fn list_labels(&self, owner: &str, repo: &str) -> Result<Vec<GitHubLabel>>;
    pub async fn create_label(&self, owner: &str, repo: &str, req: CreateLabelRequest) -> Result<GitHubLabel>;
    pub async fn set_issue_labels(&self, owner: &str, repo: &str, issue_number: i32, labels: Vec<String>) -> Result<Vec<GitHubLabel>>;
    pub async fn list_workflows(&self, owner: &str, repo: &str) -> Result<Vec<GitHubWorkflow>>;
}

pub struct GitHubRepo { pub id: i64, pub name: String, pub full_name: String, pub description: Option<String>, pub html_url: String, pub clone_url: Option<String>, pub ssh_url: Option<String>, pub private: bool, pub default_branch: String, pub open_issues_count: i32, pub updated_at: String }
pub struct GitHubIssue { pub id: i64, pub number: i32, pub title: String, pub body: Option<String>, pub state: String, pub html_url: String, pub labels: Vec<GitHubLabel>, pub created_at: String, pub updated_at: String }
pub struct GitHubWorkflow { pub id: i64, pub name: String, pub path: String, pub state: String, pub html_url: Option<String>, pub badge_url: Option<String> }
```

### myme-services::retry

```rust
pub struct RetryConfig { pub max_retries: u32, pub initial_delay: Duration, pub max_delay: Duration }
impl Default for RetryConfig { /* max_retries: 3, initial_delay: 100ms, max_delay: 5000ms */ }
impl RetryConfig {
    pub fn new(max_retries: u32, initial_delay_ms: u64, max_delay_ms: u64) -> Self;
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration; // 2^attempt * initial, capped at max
}

pub enum RetryDecision { Retry, NoRetry }
pub fn is_retryable_error(error: &reqwest::Error) -> RetryDecision;
pub fn is_retryable_status(status: StatusCode) -> RetryDecision;
pub async fn with_retry<F, Fut>(config: RetryConfig, operation: F) -> Result<Response, reqwest::Error>
where F: Fn() -> Fut, Fut: Future<Output = Result<Response, reqwest::Error>>;
```

### myme-integrations::git

```rust
pub struct LocalRepo {
    pub path: PathBuf, pub name: String, pub current_branch: Option<String>,
    pub is_clean: bool, pub remote_url: Option<String>,
    pub uncommitted_changes: usize, pub last_commit: Option<String>,
    pub last_commit_time: Option<String>,
}

pub struct GitOperations;
impl GitOperations {
    pub fn discover_repositories(base_path: &Path, max_depth: Option<usize>) -> Result<Vec<LocalRepo>>;
    pub fn get_repository_info(path: &Path) -> Result<LocalRepo>;
    pub fn clone_repository(url: &str, target_path: &Path) -> Result<LocalRepo>;
    pub fn fetch(path: &Path) -> Result<()>;
    pub fn pull(path: &Path) -> Result<()>;    // fetch + fast-forward or merge
    pub fn push(path: &Path) -> Result<()>;
    pub fn get_uncommitted_files(path: &Path) -> Result<Vec<(String, String)>>; // (path, status)
}
```

### myme-services::project

```rust
pub enum TaskStatus { Backlog, Todo, InProgress, Blocked, Review, Done }
impl TaskStatus {
    pub fn to_label(&self) -> Option<&'static str>;      // Done = None
    pub fn label_color(&self) -> &'static str;            // hex color without #
    pub fn from_github(state: &str, labels: &[String]) -> Self;  // priority: blocked > review > in-progress > backlog > todo
    pub fn all() -> &'static [TaskStatus];
}

pub struct Project { pub id: String, pub name: String, pub description: Option<String>, pub created_at: String }
pub struct Task { pub id: String, pub project_id: String, pub title: String, pub body: Option<String>, pub status: TaskStatus, pub created_at: String, pub updated_at: String }
```

### myme-services::project_store

```rust
pub struct ProjectStore { conn: Connection }
impl ProjectStore {
    pub fn open(path: &Path) -> Result<Self>;           // auto-migrates schema
    pub fn upsert_project(&self, project: &Project) -> Result<()>;
    pub fn list_projects(&self) -> Result<Vec<Project>>;
    pub fn get_project(&self, id: &str) -> Result<Option<Project>>;
    pub fn delete_project(&self, id: &str) -> Result<()>;  // cascades tasks + repos
    pub fn add_repo_to_project(&self, project_id: &str, repo_id: &str) -> Result<()>;
    pub fn remove_repo_from_project(&self, project_id: &str, repo_id: &str) -> Result<()>;
    pub fn list_repos_for_project(&self, project_id: &str) -> Result<Vec<String>>;
    pub fn list_all_linked_repo_ids(&self) -> Result<Vec<String>>;
    pub fn list_projects_for_repo(&self, repo_id: &str) -> Result<Vec<Project>>;
    pub fn upsert_task(&self, task: &Task) -> Result<()>;
    pub fn list_tasks_for_project(&self, project_id: &str) -> Result<Vec<Task>>;
    pub fn delete_task(&self, task_id: &str) -> Result<()>;
    pub fn count_tasks_by_status(&self, project_id: &str) -> Result<Vec<(TaskStatus, i32)>>;
}
```

### myme-organizations::models

```rust
pub enum ProspectStage { Lead, Qualified, Contacted, Proposal, Negotiation, Won, Lost }
impl ProspectStage {
    pub fn all() -> &'static [ProspectStage];
    pub fn display_name(&self) -> &'static str;
}

pub struct Organization {
    pub id: String, pub name: String, pub description: Option<String>,
    pub website: Option<String>, pub contact_name: Option<String>,
    pub contact_email: Option<String>, pub contact_phone: Option<String>,
    pub contact_role: Option<String>, pub created_at: String, pub updated_at: String,
}

pub struct Prospect {
    pub id: String, pub organization_id: String, pub name: String,
    pub description: Option<String>, pub stage: ProspectStage,
    pub value: Option<String>, pub contact_name: Option<String>,
    pub contact_email: Option<String>, pub contact_role: Option<String>,
    pub created_at: String, pub updated_at: String,
}
```

### myme-organizations::store

```rust
pub struct OrganizationStore { conn: Connection }
impl OrganizationStore {
    pub fn open(path: &Path) -> Result<Self>;
    pub fn upsert_organization(&self, org: &Organization) -> Result<()>;
    pub fn list_organizations(&self) -> Result<Vec<Organization>>;
    pub fn get_organization(&self, id: &str) -> Result<Option<Organization>>;
    pub fn delete_organization(&self, id: &str) -> Result<()>;  // cascades prospects + project links
    pub fn upsert_prospect(&self, prospect: &Prospect) -> Result<()>;
    pub fn list_prospects(&self, organization_id: &str) -> Result<Vec<Prospect>>;
    pub fn delete_prospect(&self, id: &str) -> Result<()>;
    pub fn update_prospect_stage(&self, id: &str, stage: ProspectStage) -> Result<()>;
    pub fn count_prospects_by_stage(&self, organization_id: &str) -> Result<Vec<(ProspectStage, i32)>>;
    pub fn link_project(&self, organization_id: &str, project_id: &str) -> Result<()>;
    pub fn unlink_project(&self, organization_id: &str, project_id: &str) -> Result<()>;
    pub fn list_linked_projects(&self, organization_id: &str) -> Result<Vec<String>>;
}
```

## Data Flow

### Note CRUD (async via spawn_blocking)

```
QML calls noteModel.create_note(content)
  -> NoteModel sends NoteServiceMessage::Create via channel
  -> Background task: NoteClient.create_todo(request).await
     -> tokio::task::spawn_blocking(|| store.lock().create(...))
     -> SQLite INSERT
  -> Result sent back via channel
  -> QML Timer polls: noteModel.poll_channel()
  -> NoteModel receives result, emits signal
```

### GitHub API with Retry

```
GitHubClient.list_repos()
  -> send_with_retry(|| build_request(client.get(url)))
     -> with_retry(config, || req.send().await)
        attempt 0: send request
        on 5xx/429/timeout -> sleep(100ms * 2^attempt) -> retry
        on 4xx -> return immediately (NoRetry)
        on success -> return response
     -> response.json::<Vec<GitHubRepo>>()
```

### Git Repository Discovery

```
GitOperations::discover_repositories(base_path, max_depth)
  -> walk_directory(path, repos, 0, max_depth)
     -> try open as git repo (git2::Repository::open)
     -> if success: push LocalRepo, don't recurse children
     -> if not: recurse subdirectories (skip hidden except .git)
  -> Returns Vec<LocalRepo> with branch, clean status, remote URL, last commit
```

## Storage / Schema

### Notes Database (`~/.config/myme/notes.db`)

```sql
-- Managed by SqliteNoteStore (myme-services/src/note_store.rs)
CREATE TABLE notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    done INTEGER NOT NULL DEFAULT 0,
    color TEXT,
    pinned INTEGER NOT NULL DEFAULT 0,
    archived INTEGER NOT NULL DEFAULT 0,
    labels TEXT NOT NULL DEFAULT '[]',     -- JSON array
    is_checklist INTEGER NOT NULL DEFAULT 0,
    reminder TEXT,                          -- ISO 8601 datetime
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

### Projects Database (`~/.config/myme/projects.db`)

```sql
-- Schema version: 3 (auto-migrated from v1/v2)
CREATE TABLE projects (id TEXT PRIMARY KEY, name TEXT NOT NULL, description TEXT, created_at TEXT NOT NULL);
CREATE TABLE project_repos (project_id TEXT NOT NULL, repo_id TEXT NOT NULL, PRIMARY KEY (project_id, repo_id));
CREATE TABLE tasks (id TEXT PRIMARY KEY, project_id TEXT NOT NULL, title TEXT NOT NULL, body TEXT, status TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL);
CREATE INDEX idx_tasks_project ON tasks(project_id);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_project_repos_project ON project_repos(project_id);
CREATE INDEX idx_project_repos_repo ON project_repos(repo_id);
```

### Organizations Database (`~/.config/myme/organizations.db`)

```sql
-- Schema version: 1
CREATE TABLE organizations (id TEXT PRIMARY KEY, name TEXT NOT NULL, description TEXT, website TEXT, contact_name TEXT, contact_email TEXT, contact_phone TEXT, contact_role TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL);
CREATE TABLE prospects (id TEXT PRIMARY KEY, organization_id TEXT NOT NULL, name TEXT NOT NULL, description TEXT, stage TEXT NOT NULL, value TEXT, contact_name TEXT, contact_email TEXT, contact_role TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, FOREIGN KEY (organization_id) REFERENCES organizations(id));
CREATE TABLE organization_projects (organization_id TEXT NOT NULL, project_id TEXT NOT NULL, PRIMARY KEY (organization_id, project_id));
CREATE INDEX idx_prospects_org ON prospects(organization_id);
CREATE INDEX idx_prospects_stage ON prospects(stage);
```

## Configuration

| Config Key | Used By | Purpose |
|-----------|---------|---------|
| `notes.sqlite_path` | NoteClient | SQLite database path for notes |
| `repos.local_search_path` | GitOperations | Base directory for repo discovery |
| `github.client_id` | GitHubClient | OAuth app client ID |
| `github.client_secret` | GitHubClient | OAuth app client secret |
| `projects.sync_interval_minutes` | ProjectStore | How often to sync issues from GitHub |
| `projects.auto_create_labels` | ProjectStore | Auto-create status labels on repos |

## Edge Cases

- **Retry exhaustion**: After 3+1 attempts, `with_retry()` returns the last error; logs "All retry attempts exhausted"
- **Rate limiting (429)**: Classified as retryable; backoff gives GitHub time to reset
- **401/403 not retried**: Auth failures return immediately to avoid retry loops
- **Schema migration**: `ProjectStore` auto-migrates v1 (single repo) -> v2 (many-to-many) -> v3 (project-based tasks) on `open()`
- **Organization delete cascade**: Uses `unchecked_transaction` to atomically delete prospects, project links, and organization
- **Git merge conflicts**: `pull()` fails with "Merge conflicts; resolve manually" on conflicting merges
- **Git unrelated histories**: `pull()` bails if merge analysis shows unrelated histories
- **NoteClient async wrapping**: All SQLite operations use `tokio::task::spawn_blocking` to avoid blocking the tokio runtime
- **TaskStatus priority**: When multiple status labels present, priority order is blocked > review > in-progress > backlog > todo
- **TodoUpdateRequest partial updates**: `Option<Option<T>>` pattern: `None` = don't touch, `Some(None)` = clear value, `Some(Some(v))` = set value
- **ProspectStage serde**: Serialized as lowercase strings (`"lead"`, `"negotiation"`, etc.); invalid values default to `Lead` on deserialization
