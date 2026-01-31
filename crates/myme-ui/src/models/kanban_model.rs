// crates/myme-ui/src/models/kanban_model.rs

use core::pin::Pin;
use std::sync::Arc;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use myme_services::{
    CreateIssueRequest, GitHubClient, ProjectStore, Task, TaskStatus, UpdateIssueRequest,
};

use crate::bridge;
use crate::services::{
    request_kanban_create, request_kanban_sync, request_kanban_update, KanbanIssueResult,
    KanbanServiceMessage,
};

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

        /// Poll for async operation results. Call this from a QML Timer.
        #[qinvokable]
        fn poll_channel(self: Pin<&mut KanbanModel>);

        #[qsignal]
        fn tasks_changed(self: Pin<&mut KanbanModel>);
    }
}

/// Operation state tracking
#[derive(Clone, PartialEq, Eq, Default)]
enum OpState {
    #[default]
    Idle,
    MovingTask {
        index: i32,
        new_status: TaskStatus,
        new_labels: Vec<String>,
    },
    CreatingTask {
        title: String,
        body: Option<String>,
        status: TaskStatus,
    },
    UpdatingTask {
        index: i32,
        title: String,
        body: Option<String>,
    },
    Syncing,
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
    op_state: OpState,
}

impl KanbanModelRust {
    /// Auto-initialize from global services if not already initialized
    fn ensure_initialized(&mut self) {
        if self.store.is_some() {
            return;
        }

        // Get project store (initializes if needed)
        if let Some(store) = crate::bridge::get_project_store_or_init() {
            self.store = Some(store);
            tracing::info!("KanbanModel: project store initialized");
        } else {
            tracing::warn!("KanbanModel: project store not available");
        }

        // Get GitHub client
        if let Some((client, _runtime)) = crate::bridge::get_github_client_and_runtime() {
            self.client = Some(client);
            tracing::info!("KanbanModel: GitHub client initialized");
        } else {
            tracing::info!("KanbanModel: GitHub client not available (not authenticated)");
        }
    }

    /// Get task at index if valid
    fn get_task(&self, index: i32) -> Option<&Task> {
        if index < 0 {
            return None;
        }
        self.tasks.get(index as usize)
    }

    /// Parse "owner/repo" from github_repo property
    fn parse_owner_repo(&self) -> Option<(String, String)> {
        let repo_str = self.github_repo.to_string();
        let parts: Vec<&str> = repo_str.split('/').collect();
        if parts.len() == 2 {
            Some((parts[0].to_string(), parts[1].to_string()))
        } else {
            None
        }
    }

    /// Parse status string to TaskStatus
    fn status_from_string(s: &str) -> TaskStatus {
        match s.to_lowercase().as_str() {
            "backlog" => TaskStatus::Backlog,
            "todo" => TaskStatus::Todo,
            "in_progress" | "in-progress" | "inprogress" => TaskStatus::InProgress,
            "blocked" => TaskStatus::Blocked,
            "review" => TaskStatus::Review,
            "done" => TaskStatus::Done,
            _ => TaskStatus::Todo,
        }
    }

    /// Convert TaskStatus to lowercase string
    fn status_to_string(status: TaskStatus) -> &'static str {
        match status {
            TaskStatus::Backlog => "backlog",
            TaskStatus::Todo => "todo",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Blocked => "blocked",
            TaskStatus::Review => "review",
            TaskStatus::Done => "done",
        }
    }

    fn set_error(&mut self, msg: &str) {
        self.error_message = QString::from(msg);
    }

    fn clear_error(&mut self) {
        self.error_message = QString::from("");
    }
}

impl qobject::KanbanModel {
    /// Load a project and its tasks (synchronous - local database)
    pub fn load_project(mut self: Pin<&mut Self>, project_id: QString) {
        self.as_mut().rust_mut().ensure_initialized();

        let store = match &self.as_ref().rust().store {
            Some(s) => s.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Project store not initialized"));
                return;
            }
        };

        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().clear_error();
        self.as_mut().set_project_id(project_id.clone());

        let project_id_str = project_id.to_string();

        // Load project to get github_repo
        let store_guard = match store.lock() {
            Ok(g) => g,
            Err(e) => {
                self.as_mut()
                    .rust_mut()
                    .set_error(&format!("Failed to access store: {}", e));
                self.as_mut().set_loading(false);
                return;
            }
        };

        // Get project info
        match store_guard.get_project(&project_id_str) {
            Ok(Some(project)) => {
                self.as_mut()
                    .set_github_repo(QString::from(&project.github_repo));
            }
            Ok(None) => {
                self.as_mut()
                    .set_error_message(QString::from("Project not found"));
                self.as_mut().set_loading(false);
                return;
            }
            Err(e) => {
                self.as_mut()
                    .rust_mut()
                    .set_error(&format!("Failed to load project: {}", e));
                self.as_mut().set_loading(false);
                return;
            }
        }

        // Load tasks for this project
        match store_guard.list_tasks(&project_id_str) {
            Ok(tasks) => {
                tracing::info!("Loaded {} tasks for project {}", tasks.len(), project_id_str);
                drop(store_guard);
                self.as_mut().rust_mut().tasks = tasks;
                self.as_mut().set_loading(false);
                self.as_mut().tasks_changed();
            }
            Err(e) => {
                tracing::error!("Failed to load tasks: {}", e);
                drop(store_guard);
                self.as_mut()
                    .rust_mut()
                    .set_error(&format!("Failed to load tasks: {}", e));
                self.as_mut().set_loading(false);
            }
        }
    }

    /// Return number of tasks
    pub fn row_count(&self) -> i32 {
        self.rust().tasks.len() as i32
    }

    /// Get GitHub issue number at index
    pub fn get_issue_number(&self, index: i32) -> i32 {
        self.rust()
            .get_task(index)
            .map(|t| t.github_issue_number)
            .unwrap_or(0)
    }

    /// Get task title at index
    pub fn get_title(&self, index: i32) -> QString {
        self.rust()
            .get_task(index)
            .map(|t| QString::from(&t.title))
            .unwrap_or_else(|| QString::from(""))
    }

    /// Get task body at index
    pub fn get_body(&self, index: i32) -> QString {
        self.rust()
            .get_task(index)
            .and_then(|t| t.body.as_ref())
            .map(|b| QString::from(b))
            .unwrap_or_else(|| QString::from(""))
    }

    /// Get task status at index (lowercase)
    pub fn get_status(&self, index: i32) -> QString {
        self.rust()
            .get_task(index)
            .map(|t| QString::from(KanbanModelRust::status_to_string(t.status)))
            .unwrap_or_else(|| QString::from(""))
    }

    /// Get labels as JSON array
    pub fn get_labels(&self, index: i32) -> QString {
        self.rust()
            .get_task(index)
            .map(|t| {
                let json = serde_json::to_string(&t.labels).unwrap_or_else(|_| "[]".to_string());
                QString::from(json)
            })
            .unwrap_or_else(|| QString::from("[]"))
    }

    /// Get GitHub URL at index
    pub fn get_url(&self, index: i32) -> QString {
        self.rust()
            .get_task(index)
            .map(|t| QString::from(&t.html_url))
            .unwrap_or_else(|| QString::from(""))
    }

    /// Count tasks for a given status
    pub fn count_by_status(&self, status: QString) -> i32 {
        let target_status = KanbanModelRust::status_from_string(&status.to_string());
        self.rust()
            .tasks
            .iter()
            .filter(|t| t.status == target_status)
            .count() as i32
    }

    /// Return JSON array of indices for tasks in this status
    pub fn tasks_for_status(&self, status: QString) -> QString {
        let target_status = KanbanModelRust::status_from_string(&status.to_string());
        let indices: Vec<i32> = self
            .rust()
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.status == target_status)
            .map(|(i, _)| i as i32)
            .collect();

        let json = serde_json::to_string(&indices).unwrap_or_else(|_| "[]".to_string());
        QString::from(json)
    }

    /// Move task to new status (updates GitHub) - non-blocking
    pub fn move_task(mut self: Pin<&mut Self>, index: i32, new_status: QString) {
        self.as_mut().rust_mut().ensure_initialized();

        // Prevent concurrent operations
        if !matches!(self.as_ref().rust().op_state, OpState::Idle) {
            tracing::warn!("move_task: operation already in progress");
            return;
        }

        let task = match self.as_ref().rust().get_task(index) {
            Some(t) => t.clone(),
            None => return,
        };

        let client = match &self.as_ref().rust().client {
            Some(c) => c.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("GitHub not authenticated"));
                return;
            }
        };

        let (owner, repo) = match self.as_ref().rust().parse_owner_repo() {
            Some(pair) => pair,
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Invalid github_repo format"));
                return;
            }
        };

        let new_status_enum = KanbanModelRust::status_from_string(&new_status.to_string());
        let old_status = task.status;

        // Skip if status hasn't changed
        if old_status == new_status_enum {
            return;
        }

        // Initialize channel if needed
        bridge::init_kanban_service_channel();
        let tx = match bridge::get_kanban_service_tx() {
            Some(t) => t,
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Service channel not ready"));
                return;
            }
        };

        // Build new labels: remove old status label, add new status label
        let status_labels = [
            "backlog",
            "todo",
            "in-progress",
            "blocked",
            "review",
        ];
        let mut new_labels: Vec<String> = task
            .labels
            .iter()
            .filter(|l| !status_labels.contains(&l.as_str()))
            .cloned()
            .collect();

        // Add new status label (if not Done)
        if let Some(label) = new_status_enum.to_label() {
            new_labels.push(label.to_string());
        }

        // Determine if we need to close/reopen the issue
        let new_state = if new_status_enum == TaskStatus::Done {
            Some("closed".to_string())
        } else if old_status == TaskStatus::Done {
            Some("open".to_string())
        } else {
            None
        };

        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().clear_error();
        self.as_mut().rust_mut().op_state = OpState::MovingTask {
            index,
            new_status: new_status_enum,
            new_labels: new_labels.clone(),
        };

        // Update GitHub issue
        let update_req = UpdateIssueRequest {
            title: None,
            body: None,
            state: new_state,
            labels: Some(new_labels),
        };

        // Spawn async operation (non-blocking)
        request_kanban_update(&tx, client, index, owner, repo, task.github_issue_number, update_req);
    }

    /// Create new task (GitHub issue) - non-blocking
    pub fn create_task(
        mut self: Pin<&mut Self>,
        title: QString,
        body: QString,
        status: QString,
    ) {
        self.as_mut().rust_mut().ensure_initialized();

        // Prevent concurrent operations
        if !matches!(self.as_ref().rust().op_state, OpState::Idle) {
            tracing::warn!("create_task: operation already in progress");
            return;
        }

        let client = match &self.as_ref().rust().client {
            Some(c) => c.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("GitHub not authenticated"));
                return;
            }
        };

        let (owner, repo) = match self.as_ref().rust().parse_owner_repo() {
            Some(pair) => pair,
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Invalid github_repo format"));
                return;
            }
        };

        // Initialize channel if needed
        bridge::init_kanban_service_channel();
        let tx = match bridge::get_kanban_service_tx() {
            Some(t) => t,
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Service channel not ready"));
                return;
            }
        };

        let title_str = title.to_string();
        let body_str = body.to_string();
        let status_enum = KanbanModelRust::status_from_string(&status.to_string());

        // Build labels for the new issue
        let labels: Option<Vec<String>> = status_enum.to_label().map(|l| vec![l.to_string()]);

        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().clear_error();
        self.as_mut().rust_mut().op_state = OpState::CreatingTask {
            title: title_str.clone(),
            body: if body_str.is_empty() {
                None
            } else {
                Some(body_str.clone())
            },
            status: status_enum,
        };

        let create_req = CreateIssueRequest {
            title: title_str,
            body: if body_str.is_empty() {
                None
            } else {
                Some(body_str)
            },
            labels,
        };

        // Spawn async operation (non-blocking)
        request_kanban_create(&tx, client, owner, repo, create_req);
    }

    /// Update task title/body - non-blocking
    pub fn update_task(mut self: Pin<&mut Self>, index: i32, title: QString, body: QString) {
        self.as_mut().rust_mut().ensure_initialized();

        // Prevent concurrent operations
        if !matches!(self.as_ref().rust().op_state, OpState::Idle) {
            tracing::warn!("update_task: operation already in progress");
            return;
        }

        let task = match self.as_ref().rust().get_task(index) {
            Some(t) => t.clone(),
            None => return,
        };

        let client = match &self.as_ref().rust().client {
            Some(c) => c.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("GitHub not authenticated"));
                return;
            }
        };

        let (owner, repo) = match self.as_ref().rust().parse_owner_repo() {
            Some(pair) => pair,
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Invalid github_repo format"));
                return;
            }
        };

        // Initialize channel if needed
        bridge::init_kanban_service_channel();
        let tx = match bridge::get_kanban_service_tx() {
            Some(t) => t,
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Service channel not ready"));
                return;
            }
        };

        let title_str = title.to_string();
        let body_str = body.to_string();

        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().clear_error();
        self.as_mut().rust_mut().op_state = OpState::UpdatingTask {
            index,
            title: title_str.clone(),
            body: if body_str.is_empty() {
                None
            } else {
                Some(body_str.clone())
            },
        };

        let update_req = UpdateIssueRequest {
            title: Some(title_str),
            body: Some(body_str),
            state: None,
            labels: None,
        };

        // Spawn async operation (non-blocking)
        request_kanban_update(&tx, client, index, owner, repo, task.github_issue_number, update_req);
    }

    /// Refresh tasks from GitHub - non-blocking
    pub fn sync_tasks(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().ensure_initialized();

        // Prevent concurrent operations
        if !matches!(self.as_ref().rust().op_state, OpState::Idle) {
            tracing::warn!("sync_tasks: operation already in progress");
            return;
        }

        let client = match &self.as_ref().rust().client {
            Some(c) => c.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("GitHub not authenticated"));
                return;
            }
        };

        let (owner, repo) = match self.as_ref().rust().parse_owner_repo() {
            Some(pair) => pair,
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Invalid github_repo format"));
                return;
            }
        };

        // Initialize channel if needed
        bridge::init_kanban_service_channel();
        let tx = match bridge::get_kanban_service_tx() {
            Some(t) => t,
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Service channel not ready"));
                return;
            }
        };

        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().clear_error();
        self.as_mut().rust_mut().op_state = OpState::Syncing;

        // Spawn async operation (non-blocking)
        request_kanban_sync(&tx, client, owner, repo);
    }

    /// Poll for async operation results. Call this from a QML Timer (e.g., every 100ms).
    pub fn poll_channel(mut self: Pin<&mut Self>) {
        let msg = match bridge::try_recv_kanban_message() {
            Some(m) => m,
            None => return,
        };

        match msg {
            KanbanServiceMessage::UpdateIssueDone { index, result } => {
                self.as_mut().handle_update_done(index, result);
            }
            KanbanServiceMessage::CreateIssueDone(result) => {
                self.as_mut().handle_create_done(result);
            }
            KanbanServiceMessage::SyncDone(result) => {
                self.as_mut().handle_sync_done(result);
            }
        }
    }

    /// Handle update issue completion (move_task or update_task)
    fn handle_update_done(
        mut self: Pin<&mut Self>,
        index: i32,
        result: Result<KanbanIssueResult, crate::services::KanbanError>,
    ) {
        let op_state = self.as_ref().rust().op_state.clone();
        self.as_mut().rust_mut().op_state = OpState::Idle;

        match result {
            Ok(updated_issue) => {
                let store = match &self.as_ref().rust().store {
                    Some(s) => s.clone(),
                    None => {
                        self.as_mut().set_loading(false);
                        return;
                    }
                };

                let task = match self.as_ref().rust().get_task(index) {
                    Some(t) => t.clone(),
                    None => {
                        self.as_mut().set_loading(false);
                        return;
                    }
                };

                // Determine what changed based on op_state
                let mut updated_task = task.clone();
                updated_task.updated_at = updated_issue.updated_at;

                match op_state {
                    OpState::MovingTask {
                        new_status,
                        new_labels,
                        ..
                    } => {
                        tracing::info!(
                            "Moved task #{} to {:?}",
                            task.github_issue_number,
                            new_status
                        );
                        updated_task.status = new_status;
                        updated_task.labels = new_labels;
                    }
                    OpState::UpdatingTask { title, body, .. } => {
                        tracing::info!("Updated task #{}", task.github_issue_number);
                        updated_task.title = title;
                        updated_task.body = body;
                    }
                    _ => {}
                }

                // Save to store
                if let Ok(store_guard) = store.lock() {
                    if let Err(e) = store_guard.upsert_task(&updated_task) {
                        tracing::warn!("Failed to save task update: {}", e);
                    }
                }

                // Update in-memory task
                if let Some(t) = self.as_mut().rust_mut().tasks.get_mut(index as usize) {
                    *t = updated_task;
                }

                self.as_mut().set_loading(false);
                self.as_mut().tasks_changed();
            }
            Err(e) => {
                tracing::error!("Failed to update task on GitHub: {}", e);
                self.as_mut()
                    .rust_mut()
                    .set_error(&format!("Failed to update task: {}", e));
                self.as_mut().set_loading(false);
            }
        }
    }

    /// Handle create issue completion
    fn handle_create_done(
        mut self: Pin<&mut Self>,
        result: Result<KanbanIssueResult, crate::services::KanbanError>,
    ) {
        let op_state = self.as_ref().rust().op_state.clone();
        self.as_mut().rust_mut().op_state = OpState::Idle;

        match result {
            Ok(issue) => {
                tracing::info!("Created issue #{}", issue.number);

                let project_id_str = self.as_ref().rust().project_id.to_string();

                // Get status from op_state
                let status = match op_state {
                    OpState::CreatingTask { status, .. } => status,
                    _ => TaskStatus::Todo,
                };

                // Create local task
                let task = Task {
                    id: uuid::Uuid::new_v4().to_string(),
                    project_id: project_id_str,
                    github_issue_number: issue.number,
                    title: issue.title,
                    body: issue.body,
                    status,
                    labels: issue.labels,
                    html_url: issue.html_url,
                    created_at: issue.created_at,
                    updated_at: issue.updated_at,
                };

                // Save to store
                if let Some(store) = &self.as_ref().rust().store {
                    if let Ok(store_guard) = store.lock() {
                        if let Err(e) = store_guard.upsert_task(&task) {
                            tracing::warn!("Failed to save new task: {}", e);
                        }
                    }
                }

                // Add to in-memory list
                self.as_mut().rust_mut().tasks.push(task);

                self.as_mut().set_loading(false);
                self.as_mut().tasks_changed();
            }
            Err(e) => {
                tracing::error!("Failed to create issue on GitHub: {}", e);
                self.as_mut()
                    .rust_mut()
                    .set_error(&format!("Failed to create task: {}", e));
                self.as_mut().set_loading(false);
            }
        }
    }

    /// Handle sync completion
    fn handle_sync_done(
        mut self: Pin<&mut Self>,
        result: Result<Vec<KanbanIssueResult>, crate::services::KanbanError>,
    ) {
        self.as_mut().rust_mut().op_state = OpState::Idle;

        match result {
            Ok(issues) => {
                tracing::info!("Fetched {} issues", issues.len());

                let project_id_str = self.as_ref().rust().project_id.to_string();
                let store = match &self.as_ref().rust().store {
                    Some(s) => s.clone(),
                    None => {
                        self.as_mut().set_loading(false);
                        return;
                    }
                };

                let store_guard = match store.lock() {
                    Ok(g) => g,
                    Err(e) => {
                        self.as_mut()
                            .rust_mut()
                            .set_error(&format!("Failed to access store: {}", e));
                        self.as_mut().set_loading(false);
                        return;
                    }
                };

                let mut tasks = Vec::new();

                // Convert issues to tasks and save
                for issue in &issues {
                    let status = TaskStatus::from_github(&issue.state, &issue.labels);

                    let task = Task {
                        id: uuid::Uuid::new_v4().to_string(),
                        project_id: project_id_str.clone(),
                        github_issue_number: issue.number,
                        title: issue.title.clone(),
                        body: issue.body.clone(),
                        status,
                        labels: issue.labels.clone(),
                        html_url: issue.html_url.clone(),
                        created_at: issue.created_at.clone(),
                        updated_at: issue.updated_at.clone(),
                    };

                    if let Err(e) = store_guard.upsert_task(&task) {
                        tracing::warn!("Failed to save task for issue #{}: {}", issue.number, e);
                    }

                    tasks.push(task);
                }

                drop(store_guard);

                // Update in-memory tasks
                self.as_mut().rust_mut().tasks = tasks;

                self.as_mut().set_loading(false);
                self.as_mut().tasks_changed();

                tracing::info!("Synced tasks");
            }
            Err(e) => {
                tracing::error!("Failed to fetch issues from GitHub: {}", e);
                self.as_mut()
                    .rust_mut()
                    .set_error(&format!("Failed to sync tasks: {}", e));
                self.as_mut().set_loading(false);
            }
        }
    }
}
